use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoRefs {
    pub repos: HashMap<String, String>, // remote_url -> last_known_sha
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushLogEntry {
    pub timestamp: u64, // unix timestamp
    pub remote_url: String,
    pub branch: String,
    pub commits: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushLog {
    pub entries: Vec<PushLogEntry>,
}

impl PushLog {
    pub fn add(&mut self, entry: PushLogEntry) {
        self.entries.push(entry);
    }
}

fn refs_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("refs.bin"))
}

fn log_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("log.json"))
}

fn load_log() -> PushLog {
    log_path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_log(log: &PushLog) -> std::io::Result<()> {
    let path = log_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(log).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

fn load_refs() -> RepoRefs {
    refs_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}

fn save_refs(refs: &RepoRefs) -> std::io::Result<()> {
    let path = refs_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(refs).map_err(std::io::Error::other)?;
    std::fs::write(path, encoded)
}

/// Result of detecting a push - just the commit count.
/// Points calculation happens in the caller.
#[derive(Debug)]
pub struct PushInfo {
    pub commits: u64,
}

/// Entry point called by git hook or jj alias.
/// Returns push info if a push was detected, None otherwise.
pub fn run() -> Option<PushInfo> {
    let cwd = std::env::current_dir().expect("could not get current directory");
    detect_push(&cwd)
}

fn detect_push(repo_path: &Path) -> Option<PushInfo> {
    crate::debug_log!("hook: detect_push called for {:?}", repo_path);

    let remote_url = match git::get_remote_url(repo_path) {
        Some(url) => url,
        None => {
            crate::debug_log!("hook: no remote url found");
            return None;
        }
    };
    crate::debug_log!("hook: remote_url = {}", remote_url);

    let branch = match git::get_trunk_branch(repo_path) {
        Some(b) => b,
        None => {
            crate::debug_log!("hook: no trunk branch found");
            return None;
        }
    };
    crate::debug_log!("hook: branch = {}", branch);

    let current_ref = match git::get_remote_ref(repo_path, &branch) {
        Some(r) => r,
        None => {
            crate::debug_log!("hook: no remote ref found for branch {}", branch);
            return None;
        }
    };
    crate::debug_log!("hook: current_ref = {}", current_ref);

    // check if local branch matches remote - if not, this was a fetch, not a push
    let local_ref = git::get_local_ref(repo_path, &branch);
    crate::debug_log!("hook: local_ref = {:?}", local_ref);
    if local_ref.as_ref() != Some(&current_ref) {
        crate::debug_log!("hook: local ref doesn't match remote, not a push");
        return None;
    }

    let mut refs = load_refs();
    let last_ref = refs.repos.get(&remote_url).cloned();
    crate::debug_log!("hook: last_ref = {:?}", last_ref);

    // if same as before, no push happened
    if last_ref.as_ref() == Some(&current_ref) {
        crate::debug_log!("hook: ref unchanged, no push detected");
        return None;
    }

    let commits = match &last_ref {
        Some(old_sha) => git::count_commits(repo_path, old_sha, &current_ref).unwrap_or(1),
        None => 1, // first time seeing this repo
    };

    record_push(&mut refs, &remote_url, &branch, &current_ref, commits);

    crate::debug_log!("hook: push detected! {} commits", commits);

    Some(PushInfo { commits })
}

/// Update refs.bin and append to log.json
fn record_push(
    refs: &mut RepoRefs,
    remote_url: &str,
    branch: &str,
    current_ref: &str,
    commits: u64,
) {
    refs.repos
        .insert(remote_url.to_string(), current_ref.to_string());
    let _ = save_refs(refs);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut log = load_log();
    log.add(PushLogEntry {
        timestamp,
        remote_url: remote_url.to_string(),
        branch: branch.to_string(),
        commits,
    });
    let _ = save_log(&log);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_refs_roundtrips() {
        let mut refs = RepoRefs::default();
        refs.repos.insert(
            "git@github.com:user/repo.git".to_string(),
            "abc123".to_string(),
        );

        let encoded = bincode::serialize(&refs).unwrap();
        let decoded: RepoRefs = bincode::deserialize(&encoded).unwrap();

        assert_eq!(
            decoded.repos.get("git@github.com:user/repo.git"),
            Some(&"abc123".to_string())
        );
    }

    #[test]
    fn empty_refs_deserializes() {
        let refs = RepoRefs::default();
        assert!(refs.repos.is_empty());
    }

    #[test]
    fn push_log_roundtrips() {
        let mut log = PushLog::default();
        log.add(PushLogEntry {
            timestamp: 1234567890,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
            commits: 5,
        });

        let json = serde_json::to_string(&log).unwrap();
        let decoded: PushLog = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries[0].commits, 5);
        assert_eq!(decoded.entries[0].branch, "main");
    }

    #[test]
    fn empty_push_log() {
        let log = PushLog::default();
        assert!(log.entries.is_empty());
    }
}
