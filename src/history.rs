use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEntry {
    timestamp: u64, // unix timestamp
    remote_url: String,
    branch: String,
    commits: u64,
}

impl Default for PushEntry {
    fn default() -> Self {
        Self {
            timestamp: 0,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
            commits: 1,
        }
    }
}

impl PushEntry {
    pub fn new(
        timestamp: u64,
        remote_url: impl Into<String>,
        branch: impl Into<String>,
        commits: u64,
    ) -> Self {
        Self {
            timestamp,
            remote_url: remote_url.into(),
            branch: branch.into(),
            commits,
        }
    }

    #[cfg(test)]
    pub fn at(timestamp: u64) -> Self {
        Self {
            timestamp,
            ..Default::default()
        }
    }

    #[cfg(test)]
    pub fn with_repo(timestamp: u64, remote_url: impl Into<String>) -> Self {
        Self {
            timestamp,
            remote_url: remote_url.into(),
            ..Default::default()
        }
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn remote_url(&self) -> &str {
        &self.remote_url
    }

    #[cfg(test)]
    pub fn branch(&self) -> &str {
        &self.branch
    }

    #[cfg(test)]
    pub fn commits(&self) -> u64 {
        self.commits
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushHistory {
    entries: Vec<PushEntry>,
}

impl PushHistory {
    #[cfg(test)]
    pub fn from_entries(entries: impl IntoIterator<Item = PushEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn entries(&self) -> &[PushEntry] {
        &self.entries
    }

    pub fn add(&mut self, entry: PushEntry) {
        self.entries.push(entry);
    }
}

fn path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("history.json"))
}

pub fn load() -> PushHistory {
    path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(history: &PushHistory) -> std::io::Result<()> {
    let path = path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(history).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

pub fn record(remote_url: &str, branch: &str, commits: u64) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut history = load();
    history.add(PushEntry::new(timestamp, remote_url, branch, commits));
    let _ = save(&history);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_history_roundtrips() {
        let mut history = PushHistory::default();
        history.add(PushEntry::new(
            1234567890,
            "git@github.com:user/repo.git",
            "main",
            5,
        ));

        let json = serde_json::to_string(&history).unwrap();
        let decoded: PushHistory = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.entries().len(), 1);
        assert_eq!(decoded.entries()[0].commits(), 5);
        assert_eq!(decoded.entries()[0].branch(), "main");
    }

    #[test]
    fn empty_history() {
        let history = PushHistory::default();
        assert!(history.entries().is_empty());
    }
}
