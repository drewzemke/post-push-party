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

    env.vcs.commit_file("src.rs", "fn main() {}", "add source file");
    env.vcs.push();

    // 10 starter + 1 first push + 1 second push = 12 points
    assert_eq!(env.get_points(), 12);
}

#[test]
fn pushing_feature_branch_awards_no_points() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();
    let points_after_main = env.get_points();

    env.vcs.cmd(&["new", "main"]);
    env.vcs.commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_main,
        "pushing feature branch should not award points"
    );
}

#[test]
fn pushing_main_after_feature_awards_points() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("feature.rs", "// feature", "feature work");
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
fn pushing_main_and_feature_together_awards_one_point() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);

    env.vcs.cmd(&["push", "--allow-new", "-b", "main", "-b", "feature"]);
    env.vcs.cmd(&["git", "fetch"]);

    // 10 starter + 1 for main only
    assert_eq!(
        env.get_points(),
        11,
        "pushing main+feature together should only award points for main"
    );
}

#[test]
fn rebasing_feature_onto_updated_main() {
    let env = jj_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    env.vcs.cmd(&["new", "main"]);
    env.vcs.commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");

    env.vcs.cmd(&["new", "main"]);
    env.vcs.commit_file("main2.rs", "// main2", "more main work");
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

    // I push a feature branch
    env.vcs.cmd(&["new", "main"]);
    env.vcs.commit_file("feature.rs", "// feature", "my feature");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_my_push,
        "fetching others' commits then pushing feature should not award points"
    );
}
