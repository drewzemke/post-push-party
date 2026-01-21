//! End-to-end tests for jj integration
//!
//! These tests create real jj repos with local bare git remotes and verify
//! that pushing commits awards the correct number of points.
//!
//! Requirements:
//! - `jj` and `git` must be installed and available in PATH
//! - read/write access to the /tmp directory

use std::process::Command;

use tempfile::TempDir;

/// Helper to run a command and assert it succeeds
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

/// Test environment with isolated state, a bare git remote, and a jj repo
struct TestEnv {
    state_dir: TempDir,
    _remote_dir: TempDir,
    repo_dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        let state_dir = TempDir::new().expect("failed to create state tempdir");
        let remote_dir = TempDir::new().expect("failed to create remote tempdir");
        let repo_dir = TempDir::new().expect("failed to create repo tempdir");

        // create bare git repo as "remote"
        run(Command::new("git")
            .args(["init", "--bare"])
            .current_dir(remote_dir.path()));

        // create jj repo (colocated with git so git commands work)
        run(Command::new("jj")
            .args(["git", "init", "--colocate"])
            .current_dir(repo_dir.path()));

        // add remote
        let remote_path = remote_dir.path().to_str().unwrap();
        run(Command::new("jj")
            .args(["git", "remote", "add", "origin", remote_path])
            .current_dir(repo_dir.path()));

        Self {
            state_dir,
            _remote_dir: remote_dir,
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
        // prepend our test binary's directory to PATH so the alias finds it
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

    /// Create a file, commit it, and set the main bookmark to that commit
    fn commit_file(&self, name: &str, content: &str, message: &str) {
        let path = self.repo_dir.path().join(name);
        std::fs::write(&path, content).expect("failed to write file");
        self.jj(&["commit", "-m", message]);
        self.jj(&["bookmark", "set", "main", "-r", "@-"]);
    }

    /// Push using the jj push alias (which calls party hook automatically)
    fn push(&self) {
        self.jj(&["push", "--allow-new", "-b", "main"]);
        self.jj(&["git", "fetch"]);
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
fn happy_path_single_commit_awards_one_point() {
    let env = TestEnv::new();

    // init party first (installs jj push alias)
    env.party(&["init"]);

    // first commit and push
    env.commit_file("README.md", "# Test", "initial commit");
    env.push();

    // second commit and push
    env.commit_file("src.rs", "fn main() {}", "add source file");
    env.push();

    // 10 starter + 1 first push + 1 second push = 12 points
    assert_eq!(env.get_points(), 12);
}
