use rusqlite::{OptionalExtension, Result};

use crate::storage::DbConnection;

pub struct BranchRefsStore<'a> {
    conn: &'a DbConnection,
}

impl<'a> BranchRefsStore<'a> {
    pub fn new(conn: &'a DbConnection) -> Self {
        Self { conn }
    }

    pub fn update_ref(&self, repo: &str, branch: &str, sha: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO branch_refs (remote_url, branch, sha) VALUES (?1, ?2, ?3)",
            (repo, branch, sha),
        )?;

        Ok(())
    }

    pub fn get_ref(&self, repo: &str, branch: &str) -> Result<Option<String>> {
        self.conn
            .query_one(
                "SELECT sha FROM branch_refs WHERE remote_url = ?1 AND branch = ?2",
                (repo, branch),
                |row| row.get(0),
            )
            .optional()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let conn = DbConnection::create_in_memory().unwrap();
        let branch_refs = BranchRefsStore::new(&conn);

        branch_refs
            .update_ref("repo-url", "branch-1", "sha-1")
            .unwrap();
        branch_refs
            .update_ref("repo-url", "branch-2", "sha-2")
            .unwrap();

        // simulates overwrite
        branch_refs
            .update_ref("repo-url", "branch-2", "sha-3")
            .unwrap();

        let ref1 = branch_refs.get_ref("repo-url", "branch-1").unwrap();
        let ref2 = branch_refs.get_ref("repo-url", "branch-2").unwrap();
        let ref3 = branch_refs.get_ref("repo-url", "branch-3").unwrap();

        assert_eq!(ref1, Some("sha-1".to_string()));
        assert_eq!(ref2, Some("sha-3".to_string()));
        assert_eq!(ref3, None);
    }
}
