mod common;

use common::{TestEnv, Vcs, git_env, jj_env};

macro_rules! shared_test {
    ($name:ident) => {
        paste::paste! {
            #[test]
            fn [<$name _git>]() { $name(&git_env()); }
            #[test]
            fn [<$name _jj>]() { $name(&jj_env()); }
        }
    };
}

shared_test!(happy_path_awards_points);
fn happy_path_awards_points<V: Vcs>(env: &TestEnv<V>) {
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

shared_test!(init_after_existing_commits_only_counts_new);
fn init_after_existing_commits_only_counts_new<V: Vcs>(env: &TestEnv<V>) {
    // push a commit BEFORE init
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.raw_push();

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

shared_test!(pushing_feature_branch_awards_points);
fn pushing_feature_branch_awards_points<V: Vcs>(env: &TestEnv<V>) {
    env.party(&["init"]);

    // push to main
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();
    let points_after_main = env.get_points();

    // push to branch
    env.vcs
        .commit_file_on_branch("feature.rs", "// feature", "feature work", "feature");
    env.vcs.push_branch("feature");

    // make sure we got credit for the branch push
    assert_eq!(
        env.get_points(),
        points_after_main + 1,
        "pushing feature branch should award points for new content"
    );
}

shared_test!(pushing_main_after_feature_awards_points);
fn pushing_main_after_feature_awards_points<V: Vcs>(env: &TestEnv<V>) {
    env.party(&["init"]);

    // need to make sure main exists first
    env.vcs.commit_file("base.rs", "// base", "base commit");
    env.vcs.ensure_main();

    // push feature first
    env.vcs
        .commit_file_on_branch("feature.rs", "// feature", "feature work", "feature");
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

shared_test!(pushing_same_content_to_different_branch_awards_no_points);
fn pushing_same_content_to_different_branch_awards_no_points<V: Vcs>(env: &TestEnv<V>) {
    env.party(&["init"]);

    // base commit on main
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();

    // commit on feature branch
    env.vcs
        .commit_file_on_branch("feature.rs", "// feature", "feature work", "feature-a");
    env.vcs.push_branch("feature-a");
    let points_after_feature_a = env.get_points();

    // create another branch from main with the same commit content
    env.vcs.checkout("main");
    env.vcs
        .commit_file_on_branch("feature.rs", "// feature", "feature work", "feature-b");
    env.vcs.push_branch("feature-b");

    assert_eq!(
        env.get_points(),
        points_after_feature_a,
        "pushing same content to different branch should not award points"
    );
}

shared_test!(fetch_does_not_award_points);
fn fetch_does_not_award_points<V: Vcs>(env: &TestEnv<V>) {
    env.party(&["init"]);

    // commit to main
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();
    let points_after_push = env.get_points();

    // someone else pushes to main
    env.simulate_external_push_to_main("external.rs", "// external", "external commit");

    // we fetch their changes
    env.vcs.fetch();
    let points_after_fetch = env.get_points();

    assert_eq!(
        points_after_fetch, points_after_push,
        "fetching others' commits should not award points"
    );

    // we push a feature branch - this DOES award points now (new content)
    env.vcs
        .commit_file_on_branch("feature.rs", "// feature", "my feature", "feature");
    env.vcs.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_push + 1,
        "pushing feature branch should award points for my new commit"
    );
}

shared_test!(fetch_then_rebase_onto_main_only_awards_for_my_work);
fn fetch_then_rebase_onto_main_only_awards_for_my_work<V: Vcs>(env: &TestEnv<V>) {
    env.party(&["init"]);

    // push initial commit to main
    env.vcs.commit_file("README.md", "# Test", "initial commit");
    env.vcs.ensure_main();
    env.vcs.push();
    let points_after_push = env.get_points();

    // someone else pushes to main
    env.simulate_external_push_to_main("external.rs", "// external", "external commit");

    // fetch their changes
    env.vcs.fetch();

    // rebase onto the updated origin/main and push new work
    env.vcs.rebase_onto_remote_main();
    env.vcs
        .commit_file("mywork.rs", "// my work", "my new commit");
    env.vcs.push();

    // should only get credit for my 1 commit, not the external one
    assert_eq!(
        env.get_points(),
        points_after_push + 1,
        "pushing main after fetch+rebase should only award points for my commits, not fetched ones"
    );
}
