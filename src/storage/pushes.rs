use rusqlite::Result;

use crate::history::PushEntry;
use crate::storage::DbConnection;

pub struct PushHistory<'a> {
    conn: &'a DbConnection,
}

impl<'a> PushHistory<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

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
