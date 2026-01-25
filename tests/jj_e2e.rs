//! End-to-end tests for jj integration
//!
//! These tests create real jj repos with local bare git remotes and verify
//! that pushing commits awards the correct number of points.
//!
//! Requirements:
//! - `jj` and `git` must be installed and available in PATH

mod common;

use common::{jj_env, Vcs};

#[test]
fn happy_path_awards_points_for_main() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    env.vcs
        .commit_file("src.rs", "fn main() {}", "add source file");
    env.vcs.push();

    // 10 starter + 1 first push + 1 second push = 12 points
    assert_eq!(env.get_points(), 12);
}

#[test]
fn pushing_feature_branch_awards_points() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();
    let points_after_main = env.get_points();

    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_main + 1,
        "pushing feature branch should award points for new content"
    );
}

#[test]
fn pushing_main_after_feature_awards_points() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");
    let points_after_feature = env.get_points();

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    assert_eq!(
        env.get_points(),
        points_after_feature + 1,
        "pushing main should award points even after feature branch"
    );
}

#[test]
fn pushing_main_and_feature_together_awards_points_for_all() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);

    env.vcs
        .cmd(&["push", "--allow-new", "-b", "main", "-b", "feature"]);

    // 10 starter + 2 for both commits (new content, regardless of branch)
    assert_eq!(
        env.get_points(),
        12,
        "pushing main+feature together should award points for all new content"
    );
}

#[test]
fn rebasing_feature_onto_updated_main() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");

    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("main2.rs", "// main2", "more main work");
    env.vcs.cmd(&["bookmark", "set", "main", "-r", "@-"]);
    env.vcs.push();
    let points_after_main_update = env.get_points();

    env.vcs.cmd(&["rebase", "-b", "feature", "-d", "main"]);
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_main_update,
        "rebasing and pushing feature should not award points"
    );
}

#[test]
fn fetch_does_not_award_points() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();
    let points_after_my_push = env.get_points();

    // someone else pushes to main
    env.simulate_external_push_to_main("external.rs", "// external", "external commit");

    // I fetch their changes
    env.vcs.fetch();
    let points_after_fetch = env.get_points();

    // fetching should not award points
    assert_eq!(
        points_after_fetch, points_after_my_push,
        "fetching others' commits should not award points"
    );

    // I push a feature branch - this DOES award points now (new content)
    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "my feature");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_my_push + 1,
        "pushing feature branch should award points for my new commit"
    );
}

#[test]
fn pushing_same_content_to_different_branch_awards_no_points() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs
        .cmd(&["bookmark", "create", "feature-a", "-r", "@-"]);
    env.vcs.push_branch("feature-a");
    let points_after_feature_a = env.get_points();

    // create another branch from main with the same commit content
    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs
        .cmd(&["bookmark", "create", "feature-b", "-r", "@-"]);
    env.vcs.push_branch("feature-b");

    assert_eq!(
        env.get_points(),
        points_after_feature_a,
        "pushing same content to different branch should not award points"
    );
}

#[test]
fn init_after_existing_commits_only_counts_new() {
    let env = jj_env();

    // push a commit BEFORE init
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    // now init party
    env.party(&["init"]);

    // push a second commit
    env.vcs
        .commit_file("src.rs", "fn main() {}", "add source file");
    env.vcs.push();

    // should only get credit for the second commit
    // 10 starter + 1 (second commit only) = 11 points
    assert_eq!(
        env.get_points(),
        11,
        "init after existing commits should not retroactively award points"
    );
}
