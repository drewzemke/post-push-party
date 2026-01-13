use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn refs_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("refs.bin"))
}

pub fn log_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("log.json"))
}

pub fn load_log() -> PushLog {
    log_path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_log(log: &PushLog) -> std::io::Result<()> {
    let path = log_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "could not determine home directory")
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(log)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)
}

pub fn load_refs() -> RepoRefs {
    refs_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}

pub fn save_refs(refs: &RepoRefs) -> std::io::Result<()> {
    let path = refs_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "could not determine home directory")
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(refs)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, encoded)
}

pub fn get_remote_url(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn get_trunk_branch(repo_path: &Path) -> Option<String> {
    // try to get the default branch from origin
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        let full_ref = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // refs/remotes/origin/main -> main
        full_ref.strip_prefix("refs/remotes/origin/").map(|s| s.to_string())
    } else {
        // fallback to main or master
        for branch in ["main", "master"] {
            let check = Command::new("git")
                .args(["rev-parse", "--verify", &format!("refs/remotes/origin/{branch}")])
                .current_dir(repo_path)
                .output()
                .ok()?;
            if check.status.success() {
                return Some(branch.to_string());
            }
        }
        None
    }
}

pub fn get_remote_ref(repo_path: &Path, branch: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", &format!("refs/remotes/origin/{branch}")])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn count_commits(repo_path: &Path, old_sha: &str, new_sha: &str) -> Option<u64> {
    let output = Command::new("git")
        .args(["rev-list", "--count", &format!("{old_sha}..{new_sha}")])
        .current_dir(repo_path)
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .ok()
    } else {
        None
    }
}

#[derive(Debug)]
pub struct PushResult {
    pub commits: u64,
    pub points_earned: u64,
}

pub fn detect_push(repo_path: &Path) -> Option<PushResult> {
    let remote_url = get_remote_url(repo_path)?;
    let branch = get_trunk_branch(repo_path)?;
    let current_ref = get_remote_ref(repo_path, &branch)?;

    let mut refs = load_refs();
    let last_ref = refs.repos.get(&remote_url).cloned();

    // if same as before, no push happened (or it's a fetch)
    if last_ref.as_ref() == Some(&current_ref) {
        return None;
    }

    let commits = if let Some(old_sha) = &last_ref {
        count_commits(repo_path, old_sha, &current_ref).unwrap_or(1)
    } else {
        // first time seeing this repo, count as 1
        1
    };

    // update stored refs
    refs.repos.insert(remote_url.clone(), current_ref);
    let _ = save_refs(&refs);

    // log the push
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut log = load_log();
    log.add(PushLogEntry {
        timestamp,
        remote_url,
        branch,
        commits,
    });
    let _ = save_log(&log);

    let state = crate::state::load();
    let points_earned = commits * state.points_per_commit();

    Some(PushResult {
        commits,
        points_earned,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_refs_roundtrips() {
        let mut refs = RepoRefs::default();
        refs.repos.insert("git@github.com:user/repo.git".to_string(), "abc123".to_string());

        let encoded = bincode::serialize(&refs).unwrap();
        let decoded: RepoRefs = bincode::deserialize(&encoded).unwrap();

        assert_eq!(decoded.repos.get("git@github.com:user/repo.git"), Some(&"abc123".to_string()));
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
