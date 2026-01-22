//! End-to-end tests for jj integration
//!
//! These tests create real jj repos with local bare git remotes and verify
//! that pushing commits awards the correct number of points.
//!
//! Requirements:
//! - `jj` and `git` must be installed and available in PATH

use std::process::Command;

use tempfile::TempDir;

fn run(cmd: &mut Command) -> String {
    let output = cmd.output().expect("failed to execute command");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "command failed: {:?}\nstdout: {}\nstderr: {}",
            cmd, stdout, stderr
        );
    }
    String::from_utf8_lossy(&output.stdout).to_string()
}

struct TestEnv {
    state_dir: TempDir,
    remote_dir: TempDir,
    repo_dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        let state_dir = TempDir::new().expect("failed to create state tempdir");
        let remote_dir = TempDir::new().expect("failed to create remote tempdir");
        let repo_dir = TempDir::new().expect("failed to create repo tempdir");

        run(Command::new("git")
            .args(["init", "--bare"])
            .current_dir(remote_dir.path()));

        run(Command::new("jj")
            .args(["git", "init", "--colocate"])
            .current_dir(repo_dir.path()));

        let remote_path = remote_dir.path().to_str().unwrap();
        run(Command::new("jj")
            .args(["git", "remote", "add", "origin", remote_path])
            .current_dir(repo_dir.path()));

        Self {
            state_dir,
            remote_dir,
            repo_dir,
        }
    }

    fn party_bin() -> String {
        env!("CARGO_BIN_EXE_party").to_string()
    }

    fn party(&self, args: &[&str]) -> String {
        run(Command::new(Self::party_bin())
            .args(args)
            .env("PARTY_STATE_DIR", self.state_dir.path())
            .current_dir(self.repo_dir.path()))
    }

    fn jj(&self, args: &[&str]) -> String {
        let party_bin = Self::party_bin();
        let party_dir = std::path::Path::new(&party_bin).parent().unwrap();
        let path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", party_dir.display(), path);

        run(Command::new("jj")
            .args(args)
            .env("PATH", new_path)
            .env("PARTY_STATE_DIR", self.state_dir.path())
            .current_dir(self.repo_dir.path()))
    }

    fn commit_file(&self, name: &str, content: &str, message: &str) {
        let path = self.repo_dir.path().join(name);
        std::fs::write(&path, content).expect("failed to write file");
        self.jj(&["commit", "-m", message]);
        self.jj(&["bookmark", "set", "main", "-r", "@-"]);
    }

    fn push(&self) {
        self.jj(&["push", "--allow-new", "-b", "main"]);
        self.jj(&["git", "fetch"]);
    }

    fn push_branch(&self, branch: &str) {
        self.jj(&["push", "--allow-new", "-b", branch]);
        self.jj(&["git", "fetch"]);
    }

    fn simulate_external_push_to_main(&self, filename: &str, content: &str, message: &str) {
        let temp_clone = TempDir::new().expect("failed to create temp clone dir");

        run(Command::new("git")
            .args(["clone", self.remote_dir.path().to_str().unwrap(), "."])
            .current_dir(temp_clone.path()));

        run(Command::new("git")
            .args(["config", "user.email", "other@example.com"])
            .current_dir(temp_clone.path()));
        run(Command::new("git")
            .args(["config", "user.name", "Other User"])
            .current_dir(temp_clone.path()));

        let file_path = temp_clone.path().join(filename);
        std::fs::write(&file_path, content).expect("failed to write file");
        run(Command::new("git")
            .args(["add", filename])
            .current_dir(temp_clone.path()));
        run(Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(temp_clone.path()));

        run(Command::new("git")
            .args(["push", "origin", "main"])
            .current_dir(temp_clone.path()));
    }

    fn get_points(&self) -> u64 {
        let output = self.party(&["dump"]);
        for line in output.lines() {
            if let Some(rest) = line.strip_prefix("party_points: ") {
                return rest.trim().parse().expect("failed to parse points");
            }
        }
        panic!("party_points not found in dump output: {}", output);
    }
}

#[test]
fn happy_path_awards_points_for_main() {
    let env = TestEnv::new();
    env.party(&["init"]);

    env.commit_file("README.md", "# Test", "initial commit");
    env.push();

    env.commit_file("src.rs", "fn main() {}", "add source file");
    env.push();

    // 10 starter + 1 first push + 1 second push = 12 points
    assert_eq!(env.get_points(), 12);
}

#[test]
fn pushing_feature_branch_awards_no_points() {
    let env = TestEnv::new();
    env.party(&["init"]);

    env.commit_file("README.md", "# Test", "initial commit");
    env.push();
    let points_after_main = env.get_points();

    env.jj(&["new", "main"]);
    env.commit_file("feature.rs", "// feature", "feature work");
    env.jj(&["bookmark", "create", "feature", "-r", "@-"]);
    env.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_main,
        "pushing feature branch should not award points"
    );
}

#[test]
fn pushing_main_after_feature_awards_points() {
    let env = TestEnv::new();
    env.party(&["init"]);

    env.commit_file("feature.rs", "// feature", "feature work");
    env.jj(&["bookmark", "create", "feature", "-r", "@-"]);
    env.push_branch("feature");
    let points_after_feature = env.get_points();

    env.commit_file("README.md", "# Test", "initial commit");
    env.push();

    assert_eq!(
        env.get_points(),
        points_after_feature + 1,
        "pushing main should award points even after feature branch"
    );
}

#[test]
fn pushing_main_and_feature_together_awards_one_point() {
    let env = TestEnv::new();
    env.party(&["init"]);

    env.commit_file("README.md", "# Test", "initial commit");
    env.commit_file("feature.rs", "// feature", "feature work");
    env.jj(&["bookmark", "create", "feature", "-r", "@-"]);

    env.jj(&["push", "--allow-new", "-b", "main", "-b", "feature"]);
    env.jj(&["git", "fetch"]);

    // 10 starter + 1 for main only
    assert_eq!(
        env.get_points(),
        11,
        "pushing main+feature together should only award points for main"
    );
}

#[test]
fn rebasing_feature_onto_updated_main() {
    let env = TestEnv::new();
    env.party(&["init"]);

    env.commit_file("README.md", "# Test", "initial commit");
    env.push();

    env.jj(&["new", "main"]);
    env.commit_file("feature.rs", "// feature", "feature work");
    env.jj(&["bookmark", "create", "feature", "-r", "@-"]);
    env.push_branch("feature");

    env.jj(&["new", "main"]);
    env.commit_file("main2.rs", "// main2", "more main work");
    env.jj(&["bookmark", "set", "main", "-r", "@-"]);
    env.push();
    let points_after_main_update = env.get_points();

    env.jj(&["rebase", "-b", "feature", "-d", "main"]);
    env.push_branch("feature");

    assert_eq!(
        env.get_points(),
        points_after_main_update,
        "rebasing and pushing feature should not award points"
    );
}

#[test]
fn fetch_does_not_award_points() {
    let env = TestEnv::new();
    env.party(&["init"]);

    env.commit_file("README.md", "# Test", "initial commit");
    env.push();
    let points_after_my_push = env.get_points();

    // someone else pushes to main
    env.simulate_external_push_to_main("external.rs", "// external", "external commit");

    // I fetch their changes
    env.jj(&["git", "fetch"]);

    // I push a feature branch
    env.jj(&["new", "main"]);
    env.commit_file("feature.rs", "// feature", "my feature");
    env.jj(&["bookmark", "create", "feature", "-r", "@-"]);
    env.push_branch("feature");

    // BUG: should not award points for commits I didn't push
    assert_eq!(
        env.get_points(),
        points_after_my_push,
        "fetching others' commits then pushing feature should not award points"
    );
}
