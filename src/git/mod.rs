//! Git operations for detecting pushes and tracking seen commits.

mod commands;
mod detection;
mod patch_ids;

pub use detection::{get_pushed_commits, snapshot_refs};

/// data about a single commit in a push
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub sha: String,
    pub lines_changed: u64,
    pub timestamp: u64,
}

/// data about a single push
#[derive(Debug)]
pub struct PushInfo {
    pub commits: Vec<CommitInfo>,
    pub commits_pushed: u64,
    pub commits_counted: u64,
    pub remote_url: String,
    pub branch: String,
}
