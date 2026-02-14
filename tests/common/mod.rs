//! Shared test infrastructure for E2E tests

#![allow(dead_code)]

use std::path::Path;
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
    // combine stdout and stderr since party output might go to either
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}{}", stdout, stderr)
}

fn party_bin() -> String {
    env!("CARGO_BIN_EXE_party").to_string()
}

/// prepares PATH with party binary directory for hooks to find it
fn path_with_party() -> String {
    let party_bin = party_bin();
    let party_dir = Path::new(&party_bin).parent().unwrap();
    let path = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", party_dir.display(), path)
}

pub trait Vcs {
    /// initialize repo and add remote
    fn init(repo_dir: &Path, remote_dir: &Path);

    /// run VCS command in repo
    fn cmd(&self, args: &[&str]) -> String;

    /// create a commit with a file (should update main bookmark/branch)
    fn commit_file(&self, name: &str, content: &str, message: &str);

    /// ensure main branch exists (git needs explicit branch -M, jj doesn't)
    fn ensure_main(&self) {}

    /// push main to origin
    fn push(&self);

    /// push a specific branch to origin
    fn push_branch(&self, branch: &str);

    /// create and switch to a feature branch
    fn create_feature_branch(&self, name: &str);

    /// switch to an existing branch
    fn checkout(&self, branch: &str);

    /// fetch from origin
    fn fetch(&self);
}

pub struct TestEnv<V> {
    pub state_dir: TempDir,
    pub remote_dir: TempDir,
    pub repo_dir: TempDir,
    pub vcs: V,
}

impl<V: Vcs> TestEnv<V> {
    /// runs `party` with subcommands
    pub fn party(&self, args: &[&str]) -> String {
        run(Command::new(party_bin())
            .args(args)
            .env("PARTY_STATE_DIR", self.state_dir.path())
            .current_dir(self.repo_dir.path()))
    }

    pub fn get_points(&self) -> u64 {
        let output = self.party(&["dump"]);
        for line in output.lines() {
            if let Some(rest) = line.strip_prefix("party_points: ") {
                return rest.trim().parse().expect("failed to parse points");
            }
        }
        panic!("party_points not found in dump output: {}", output);
    }

    /// clones the test repo separately (using git) and pushes a commit to main.
    /// used to test interactions with fetching new commits
    pub fn simulate_external_push_to_main(&self, filename: &str, content: &str, message: &str) {
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
}

// --- Git implementation ---

pub struct Git<'a> {
    pub repo_dir: &'a Path,
    pub state_dir: &'a Path,
}

impl Vcs for Git<'_> {
    fn init(repo_dir: &Path, remote_dir: &Path) {
        run(Command::new("git").args(["init"]).current_dir(repo_dir));

        run(Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_dir));
        run(Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_dir));

        let remote_path = remote_dir.to_str().unwrap();
        run(Command::new("git")
            .args(["remote", "add", "origin", remote_path])
            .current_dir(repo_dir));
    }

    fn cmd(&self, args: &[&str]) -> String {
        run(Command::new("git")
            .args(args)
            .env("PATH", path_with_party())
            .env("PARTY_STATE_DIR", self.state_dir)
            .current_dir(self.repo_dir))
    }

    fn commit_file(&self, name: &str, content: &str, message: &str) {
        let path = self.repo_dir.join(name);
        std::fs::write(&path, content).expect("failed to write file");
        self.cmd(&["add", name]);
        self.cmd(&["commit", "-m", message]);
    }

    fn ensure_main(&self) {
        self.cmd(&["branch", "-M", "main"]);
    }

    fn push(&self) {
        self.cmd(&["push", "-u", "origin", "main"]);
    }

    fn push_branch(&self, branch: &str) {
        self.cmd(&["push", "-u", "origin", branch]);
    }

    fn create_feature_branch(&self, name: &str) {
        self.cmd(&["checkout", "-b", name]);
    }

    fn checkout(&self, branch: &str) {
        self.cmd(&["checkout", branch]);
    }

    fn fetch(&self) {
        self.cmd(&["fetch", "origin"]);
    }
}

pub fn git_env() -> TestEnv<Git<'static>> {
    let state_dir = TempDir::new().expect("failed to create state tempdir");
    let remote_dir = TempDir::new().expect("failed to create remote tempdir");
    let repo_dir = TempDir::new().expect("failed to create repo tempdir");

    run(Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path()));
    run(Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(remote_dir.path()));

    Git::init(repo_dir.path(), remote_dir.path());

    // leak the paths so we can store references in Git
    let repo_path: &'static Path = Box::leak(repo_dir.path().to_path_buf().into_boxed_path());
    let state_path: &'static Path = Box::leak(state_dir.path().to_path_buf().into_boxed_path());

    TestEnv {
        vcs: Git {
            repo_dir: repo_path,
            state_dir: state_path,
        },
        state_dir,
        remote_dir,
        repo_dir,
    }
}

// --- JJ implementation ---

pub struct Jj<'a> {
    pub repo_dir: &'a Path,
    pub state_dir: &'a Path,
}

impl Vcs for Jj<'_> {
    fn init(repo_dir: &Path, remote_dir: &Path) {
        run(Command::new("jj")
            .args(["git", "init", "--colocate"])
            .current_dir(repo_dir));

        let remote_path = remote_dir.to_str().unwrap();
        run(Command::new("jj")
            .args(["git", "remote", "add", "origin", remote_path])
            .current_dir(repo_dir));
    }

    fn cmd(&self, args: &[&str]) -> String {
        run(Command::new("jj")
            .args(args)
            .env("PATH", path_with_party())
            .env("PARTY_STATE_DIR", self.state_dir)
            .current_dir(self.repo_dir))
    }

    fn commit_file(&self, name: &str, content: &str, message: &str) {
        let path = self.repo_dir.join(name);
        std::fs::write(&path, content).expect("failed to write file");
        self.cmd(&["commit", "-m", message]);
        self.cmd(&["bookmark", "set", "main", "-r", "@-"]);
    }

    fn push(&self) {
        self.cmd(&["push", "--allow-new", "-b", "main"]);
        self.cmd(&["git", "fetch"]);
    }

    fn push_branch(&self, branch: &str) {
        self.cmd(&["push", "--allow-new", "-b", branch]);
        self.cmd(&["git", "fetch"]);
    }

    fn create_feature_branch(&self, _name: &str) {
        self.cmd(&["new", "main"]);
        // jj: bookmark gets created after commit_file, not here
    }

    fn checkout(&self, branch: &str) {
        self.cmd(&["new", branch]);
    }

    fn fetch(&self) {
        self.cmd(&["git", "fetch"]);
    }
}

impl Jj<'_> {
    /// used to push before the party hook is installed
    pub fn git_push(&self) {
        self.cmd(&["git", "push", "--allow-new", "-b", "main"]);
    }
}

pub fn jj_env() -> TestEnv<Jj<'static>> {
    let state_dir = TempDir::new().expect("failed to create state tempdir");
    let remote_dir = TempDir::new().expect("failed to create remote tempdir");
    let repo_dir = TempDir::new().expect("failed to create repo tempdir");

    run(Command::new("git")
        .args(["init", "--bare"])
        .current_dir(remote_dir.path()));
    run(Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(remote_dir.path()));

    Jj::init(repo_dir.path(), remote_dir.path());

    let repo_path: &'static Path = Box::leak(repo_dir.path().to_path_buf().into_boxed_path());
    let state_path: &'static Path = Box::leak(state_dir.path().to_path_buf().into_boxed_path());

    TestEnv {
        vcs: Jj {
            repo_dir: repo_path,
            state_dir: state_path,
        },
        state_dir,
        remote_dir,
        repo_dir,
    }
}
