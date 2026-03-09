use serde::{Deserialize, Serialize};

// FIXME: move this somewhere else?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushEntry {
    timestamp: u64, // unix timestamp
    remote_url: String,
    branch: String,
    commits: u64,
    #[serde(default)]
    lines_changed: u64,
    #[serde(default)]
    points_earned: u64,
}

impl Default for PushEntry {
    fn default() -> Self {
        Self {
            timestamp: 0,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
            commits: 1,
            lines_changed: 0,
            points_earned: 0,
        }
    }
}

impl PushEntry {
    pub fn new(
        timestamp: u64,
        remote_url: String,
        branch: String,
        commits: u64,
        lines_changed: u64,
        points_earned: u64,
    ) -> Self {
        Self {
            timestamp,
            remote_url,
            branch,
            commits,
            lines_changed,
            points_earned,
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

    pub fn branch(&self) -> &str {
        &self.branch
    }

    pub fn commits(&self) -> u64 {
        self.commits
    }

    pub fn lines_changed(&self) -> u64 {
        self.lines_changed
    }

    pub fn points_earned(&self) -> u64 {
        self.points_earned
    }
}

// TODO remove, used only for migration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushHistory {
    entries: Vec<PushEntry>,
}

impl PushHistory {
    pub fn entries(&self) -> &[PushEntry] {
        &self.entries
    }
}

fn path() -> Option<std::path::PathBuf> {
    crate::state::old_state_dir().map(|d| d.join("history.json"))
}

// TODO: remove, used only for migration
pub fn load() -> PushHistory {
    path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
