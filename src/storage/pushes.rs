use rusqlite::Result;

use crate::{clock::Clock, storage::DbConnection};

#[derive(Debug, Clone)]
pub struct PushEntry {
    timestamp: u64, // unix timestamp
    remote_url: String,

    #[allow(dead_code)]
    branch: String,

    commits: u64,
    lines_changed: u64,
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

    pub fn with_current_time(
        remote_url: String,
        branch: String,
        commits: u64,
        lines_changed: u64,
        points_earned: u64,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self::new(
            timestamp,
            remote_url,
            branch,
            commits,
            lines_changed,
            points_earned,
        )
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

    #[cfg(test)]
    pub fn lines_changed(&self) -> u64 {
        self.lines_changed
    }

    #[cfg(test)]
    pub fn points_earned(&self) -> u64 {
        self.points_earned
    }
}

/// numerical summary of activity during a given time period
pub struct HistoryStats {
    /// how many commits were pushed in total
    pub commits: u64,

    /// how many lines were changed across all commits
    pub lines: u64,

    /// how many points were earned (from pushes)
    pub points: u64,

    /// how many separate days are represented in the data
    pub active_days: u64,

    // the most points that were scored in a single commit
    pub max_points: u64,
}

impl HistoryStats {
    pub fn new(commits: u64, lines: u64, points: u64, active_days: u64, max_points: u64) -> Self {
        Self {
            commits,
            lines,
            points,
            active_days,
            max_points,
        }
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

    pub fn record(&self, entry: &PushEntry) -> Result<()> {
        self.conn.execute(
            "
                INSERT INTO pushes
                    (timestamp, remote_url, branch, commits, lines_changed, points_earned)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            (
                entry.timestamp as i64,
                &entry.remote_url,
                &entry.branch,
                entry.commits as i64,
                entry.lines_changed as i64,
                entry.points_earned as i64,
            ),
        )?;

        Ok(())
    }

    pub fn entries_since(&self, timestamp: u64) -> Result<Vec<PushEntry>> {
        let mut stmt = self.conn.prepare(
            "
                SELECT 
                    timestamp, remote_url, branch, commits, lines_changed, points_earned
                FROM pushes
                WHERE timestamp >= ?1
                ",
        )?;

        let map = stmt.query_map((timestamp as i64,), |row| {
            Ok(PushEntry::new(
                row.get::<_, i64>(0)? as u64,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, i64>(3)? as u64,
                row.get::<_, i64>(4)? as u64,
                row.get::<_, i64>(5)? as u64,
            ))
        })?;

        let entries = map.filter_map(|entry| entry.ok()).collect();
        Ok(entries)
    }

    pub fn count_since(&self, timestamp: u64) -> Result<u32> {
        self.conn.query_one(
            "
                SELECT COUNT (*)
                FROM pushes
                WHERE timestamp >= ?1
                ",
            (timestamp as i64,),
            |r| r.get(0),
        )
    }

    pub fn stats_since(&self, timestamp: u64, tz_offset_secs: i32) -> Result<HistoryStats> {
        let (commits, lines, points, days, max_pts) = self.conn.query_one(
            "
                SELECT 
                    COALESCE( SUM(commits), 0 ),
                    COALESCE( SUM(lines_changed), 0 ),
                    COALESCE( SUM(points_earned), 0 ),
                    COUNT( DISTINCT (timestamp + ?2) / ?3 ),
                    COALESCE( MAX(points_earned), 0 )
                FROM pushes
                WHERE timestamp >= ?1
                ",
            (timestamp as i64, tz_offset_secs, Clock::SECONDS_PER_DAY),
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            },
        )?;

        Ok(HistoryStats::new(
            commits as u64,
            lines as u64,
            points as u64,
            days.max(1) as u64,
            max_pts as u64,
        ))
    }

    #[cfg(test)]
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
    fn record_and_entries_since() {
        let conn = DbConnection::create_in_memory().unwrap();
        let pushes = PushHistory::new(&conn);

        let entry = PushEntry::with_current_time(
            "url/repo.git".to_string(),
            "main".to_string(),
            5,
            120,
            42,
        );
        pushes.record(&entry).unwrap();

        let entries = pushes.entries_since(0).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].remote_url(), "url/repo.git");
        assert_eq!(entries[0].branch(), "main");
        assert_eq!(entries[0].commits(), 5);
        assert_eq!(entries[0].lines_changed(), 120);
        assert_eq!(entries[0].points_earned(), 42);

        // reset to clear entries
        pushes.reset().unwrap();
        let entries = pushes.entries_since(0).unwrap();
        assert_eq!(entries.len(), 0);
    }

    #[test]
    fn with_entries_and_entries_since() {
        let conn = DbConnection::create_in_memory().unwrap();

        let entry = PushEntry::with_current_time(
            "url/repo.git".to_string(),
            "main".to_string(),
            5,
            120,
            42,
        );
        let pushes = PushHistory::new(&conn).with_entries([entry]);

        let entries = pushes.entries_since(0).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].remote_url(), "url/repo.git");
        assert_eq!(entries[0].branch(), "main");
        assert_eq!(entries[0].commits(), 5);
        assert_eq!(entries[0].lines_changed(), 120);
        assert_eq!(entries[0].points_earned(), 42);
    }

    #[test]
    fn with_entries_and_count_since() {
        let conn = DbConnection::create_in_memory().unwrap();

        let entries = [
            PushEntry::with_current_time("url/repo.git".to_string(), "main".to_string(), 1, 2, 3),
            PushEntry::with_current_time("url/repo.git".to_string(), "main".to_string(), 4, 5, 6),
            PushEntry::with_current_time("url/repo.git".to_string(), "main".to_string(), 7, 8, 9),
        ];
        let pushes = PushHistory::new(&conn).with_entries(entries);

        let count = pushes.count_since(0).unwrap();

        assert_eq!(count, 3);
    }

    #[test]
    fn with_entries_and_stats_since() {
        let conn = DbConnection::create_in_memory().unwrap();

        let entries = [
            PushEntry::with_current_time("url/repo.git".to_string(), "main".to_string(), 1, 2, 3),
            PushEntry::with_current_time("url/repo.git".to_string(), "main".to_string(), 4, 5, 6),
            PushEntry::with_current_time("url/repo.git".to_string(), "main".to_string(), 7, 8, 9),
        ];
        let pushes = PushHistory::new(&conn).with_entries(entries);

        let stats = pushes.stats_since(0, 0).unwrap();

        assert_eq!(stats.commits, 12);
        assert_eq!(stats.lines, 15);
        assert_eq!(stats.points, 18);
        assert_eq!(stats.active_days, 1);
        assert_eq!(stats.max_points, 9);
    }
}
