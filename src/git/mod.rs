//! Git operations for detecting pushes and tracking seen commits.

mod commands;
mod detection;
mod patch_ids;

pub use detection::{get_pushed_commits, snapshot_refs};

/// data about a single commit in a push
#[derive(Debug, Clone, Default)]
pub struct Commit {
    lines_changed: u64,
    #[expect(dead_code)]
    sha: String,
    #[expect(dead_code)]
    timestamp: u64,
}

impl Commit {
    pub fn new(sha: impl Into<String>, lines_changed: u64, timestamp: u64) -> Self {
        Self { sha: sha.into(), lines_changed, timestamp }
    }

    pub fn with_lines(lines_changed: u64) -> Self {
        Self { lines_changed, ..Default::default() }
    }

    pub fn lines_changed(&self) -> u64 {
        self.lines_changed
    }
}

/// data about a single push
#[derive(Debug)]
pub struct Push {
    commits: Vec<Commit>,
    remote_url: String,
    branch: String,
}

impl Default for Push {
    fn default() -> Self {
        Self {
            commits: Vec::new(),
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
        }
    }
}

impl Push {
    pub fn new(commits: Vec<Commit>) -> Self {
        Self { commits, ..Default::default() }
    }

    pub fn with_repo(commits: Vec<Commit>, remote_url: impl Into<String>) -> Self {
        Self { commits, remote_url: remote_url.into(), ..Default::default() }
    }

    pub fn from_parts(
        commits: Vec<Commit>,
        remote_url: impl Into<String>,
        branch: impl Into<String>,
    ) -> Self {
        Self { commits, remote_url: remote_url.into(), branch: branch.into() }
    }

    pub fn commits(&self) -> &[Commit] {
        &self.commits
    }

    pub fn remote_url(&self) -> &str {
        &self.remote_url
    }

    pub fn branch(&self) -> &str {
        &self.branch
    }
}
