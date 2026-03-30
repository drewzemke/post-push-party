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
