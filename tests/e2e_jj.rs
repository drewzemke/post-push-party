mod common;

use common::{Vcs, jj_env};

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
fn duplicate_feature_onto_fetched_trunk_only_awards_for_my_work() {
    let env = jj_env();
    env.party(&["init"]);

    // push initial commit to main
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.push();

    // create and push a feature branch
    env.vcs.cmd(&["new", "main"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&["bookmark", "create", "feature", "-r", "@-"]);
    env.vcs.push_branch("feature");
    let points_after_feature = env.get_points();

    // someone else pushes to main (multiple commits to simulate "behind by many")
    env.simulate_external_push_to_main("external1.rs", "// ext1", "external commit 1");
    env.simulate_external_push_to_main("external2.rs", "// ext2", "external commit 2");
    env.simulate_external_push_to_main("external3.rs", "// ext3", "external commit 3");

    // I fetch their changes
    env.vcs.fetch();

    // I duplicate my feature commit onto the new main (jj duplicate style)
    // This creates a new commit with the same content but different parents
    env.vcs.cmd(&["new", "main@origin"]);
    env.vcs
        .commit_file("feature.rs", "// feature", "feature work");
    env.vcs.cmd(&[
        "bookmark",
        "set",
        "feature",
        "-r",
        "@-",
        "--allow-backwards",
    ]);

    // push the feature branch (which is now based on updated main)
    env.vcs.push_branch("feature");

    // should NOT get credit for external commits, only 0 for the duplicate
    // (same patch-id as original feature commit)
    assert_eq!(
        env.get_points(),
        points_after_feature,
        "duplicating feature onto fetched trunk should not award points for fetched commits"
    );
}
