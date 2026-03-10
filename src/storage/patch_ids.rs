use rusqlite::Result;

use crate::storage::DbConnection;

pub struct PatchIdStore<'a> {
    conn: &'a DbConnection,
}

impl<'a> PatchIdStore<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    pub fn record(&self, remote_url: &str, patch_id: &str) -> Result<()> {
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO patch_ids VALUES (?1, ?2)",
            (remote_url, patch_id),
        )?;
        Ok(())
    }

    pub fn contains(&self, remote_url: &str, patch_id: &str) -> Result<bool> {
        self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM patch_ids WHERE remote_url = ?1 AND patch_id = ?2)",
            (remote_url, patch_id),
            |r| r.get(0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let conn = DbConnection::create_in_memory().unwrap();
        let store = PatchIdStore::new(&conn);

        store.record("remote-url-1", "patch-id-1").unwrap();
        store.record("remote-url-2", "patch-id-2").unwrap();

        assert!(store.contains("remote-url-1", "patch-id-1").unwrap());
        assert!(store.contains("remote-url-2", "patch-id-2").unwrap());
        assert!(!store.contains("remote-url-1", "patch-id-2").unwrap());
    }
}
