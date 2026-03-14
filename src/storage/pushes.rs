use rusqlite::Result;
use serde::{Deserialize, Serialize};

use crate::storage::DbConnection;

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

    #[cfg(test)]
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

pub struct PushHistory<'a> {
    conn: &'a DbConnection,
}

impl<'a> PushHistory<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    #[cfg(any(feature = "dev", test))]
    pub fn reset(&self) -> Result<()> {
        let _ = self.conn.execute("DELETE FROM pushes", ())?;
        Ok(())
    }

    // FIXME: this should take a PushEntry as input
    pub fn record(
        &self,
        remote_url: &str,
        branch: &str,
        commits: u64,
        lines_changed: u64,
        points_earned: u64,
    ) -> Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.conn.execute(
            "
                INSERT INTO pushes
                    (timestamp, remote_url, branch, commits, lines_changed, points_earned)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            (
                timestamp as i64,
                remote_url,
                branch,
                commits as i64,
                lines_changed as i64,
                points_earned as i64,
            ),
        )?;

        Ok(())
    }

    // FIXME: this should probably just return a result, not bail on errors
    pub fn entries_since(&self, timestamp: u64) -> Vec<PushEntry> {
        let stmt = self.conn.prepare(
            "
                SELECT 
                    timestamp, remote_url, branch, commits, lines_changed, points_earned
                FROM pushes
                WHERE timestamp >= ?1
                ",
        );
        let Ok(mut stmt) = stmt else {
            return Vec::new();
        };

        let map = stmt.query_map((timestamp as i64,), |row| {
            Ok(PushEntry::new(
                row.get::<_, i64>(0)? as u64,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, i64>(3)? as u64,
                row.get::<_, i64>(4)? as u64,
                row.get::<_, i64>(5)? as u64,
            ))
        });

        let Ok(map) = map else {
            return Vec::new();
        };

        map.filter_map(|entry| entry.ok()).collect()
    }

    #[cfg(test)]
    // FIXME: refactor to use Self::record
    pub fn with_entries(self, entries: impl IntoIterator<Item = PushEntry>) -> Self {
        let mut stmt = self
            .conn
            .prepare(
                "
                INSERT INTO pushes
                    (timestamp, remote_url, branch, commits, lines_changed, points_earned)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            )
            .expect("sql should work in tests");

        for entry in entries {
            stmt.execute((
                entry.timestamp() as i64,
                entry.remote_url(),
                entry.branch(),
                entry.commits() as i64,
                entry.lines_changed() as i64,
                entry.points_earned() as i64,
            ))
            .expect("sql should work in tests");
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let conn = DbConnection::create_in_memory().unwrap();
        let pushes = PushHistory::new(&conn);

        pushes.record("url/repo.git", "main", 5, 120, 42).unwrap();

        let entries = pushes.entries_since(0);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].remote_url(), "url/repo.git");
        assert_eq!(entries[0].branch(), "main");
        assert_eq!(entries[0].commits(), 5);
        assert_eq!(entries[0].lines_changed(), 120);
        assert_eq!(entries[0].points_earned(), 42);

        // reset to clear entries
        pushes.reset().unwrap();
        let entries = pushes.entries_since(0);
        assert_eq!(entries.len(), 0);
    }
}
