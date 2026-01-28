//! End-to-end tests for git integration
//!
//! These tests create real git repos with local bare git remotes and verify
//! that pushing commits awards the correct number of points.
//!
//! Requirements:
//! - `git` must be installed and available in PATH

mod common;

use common::{git_env, Vcs};

#[test]
fn happy_path_awards_points_for_main() {
    let env = git_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();

    env.vcs
        .commit_file("src.rs", "fn main() {}", "add source file");
    env.vcs.push();

    // 10 starter + 1 first push + 1 second push = 12 points
    assert_eq!(env.get_points(), 12);
}

#[test]
fn pushing_feature_branch_awards_points() {
    let env = git_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();
    let points_after_main = env.get_points();

    env.vcs.create_feature_branch("feature");
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_main + 1,
        "pushing feature branch should award points for new content"
    );
}

#[test]
fn pushing_main_after_feature_awards_points() {
    let env = git_env();
    env.party(&["init"]);

    // push feature first (need main to exist first for branching)
    env.vcs.commit_file("base.rs", "// base", "base commit");
    env.vcs.ensure_main();
    env.vcs.create_feature_branch("feature");
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.push_branch("feature");
    let points_after_feature = env.get_points();

    // now push main
    env.vcs.checkout("main");
    env.vcs.commit_file("README.md", "# Test", "main commit");
    env.vcs.push();

    assert_eq!(
        env.get_points(),
        points_after_feature + 1,
        "pushing main should award points even after feature branch"
    );
}

#[test]
fn fetch_does_not_award_points() {
    let env = git_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
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
    env.vcs.create_feature_branch("feature");
    env.vcs
        .commit_file("feature.rs", "// feature", "my feature");
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_my_push + 1,
        "pushing feature branch should award points for my new commit"
    );
}

#[test]
fn rebase_and_force_push_awards_no_points() {
    let env = git_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();

    env.vcs.create_feature_branch("feature");
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.push_branch("feature");
    let points_after_feature = env.get_points();

    // rebase onto main (creates new SHA but same content)
    env.vcs.cmd(&["rebase", "main"]);
    env.vcs.cmd(&["push", "--force", "origin", "feature"]);

    assert_eq!(
        env.get_points(),
        points_after_feature,
        "rebasing and force pushing should not award points (same patch-id)"
    );
}

#[test]
fn pushing_same_content_to_different_branch_awards_no_points() {
    let env = git_env();
    env.party(&["init"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();

    env.vcs.create_feature_branch("feature-a");
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.push_branch("feature-a");
    let points_after_feature_a = env.get_points();

    // create another branch from main with the same commit content
    env.vcs.checkout("main");
    env.vcs.create_feature_branch("feature-b");
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.push_branch("feature-b");

    assert_eq!(
        env.get_points(),
        points_after_feature_a,
        "pushing same content to different branch should not award points"
    );
}

#[test]
fn fetch_then_rebase_onto_main_only_awards_for_my_work() {
    let env = git_env();
    env.party(&["init"]);

    // push initial commit to main
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();
    let points_after_initial = env.get_points();

    // someone else pushes to main
    env.simulate_external_push_to_main("external.rs", "// external", "external commit");

    // I fetch their changes
    env.vcs.fetch();

    // I rebase onto the updated origin/main and create new work
    env.vcs.cmd(&["rebase", "origin/main"]);
    env.vcs
        .commit_file("mywork.rs", "// my work", "my new commit");

    // push main (which now includes my rebased work)
    env.vcs.push();

    // I should only get credit for my 1 commit, not the external one
    assert_eq!(
        env.get_points(),
        points_after_initial + 1,
        "pushing main after fetch+rebase should only award points for my commits, not fetched ones"
    );
}

#[test]
fn init_after_existing_commits_only_counts_new() {
    let env = git_env();

    // push a commit BEFORE init
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
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
