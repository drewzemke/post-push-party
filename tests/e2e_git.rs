mod common;

use common::{Vcs, git_env};

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
#[cfg(feature = "dev")]
fn first_push_of_day_bonus_applies() {
    let env = git_env();
    env.party(&["init"]);

    // unlock first_push bonus at level 1 (2x multiplier)
    // NOTE: uses `dev` feature
    env.party(&["unlock", "first_push", "1"]);

    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();

    // with first_push bonus at level 1 (2x multiplier):
    // 10 starter + (1 commit × 2) = 12 points
    assert_eq!(
        env.get_points(),
        12,
        "first push of day should apply 2x multiplier"
    );
}
