use std::collections::HashMap;

use rusqlite::Result;

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

    // FIXME: turn this into a "get ref for some branch of some repo"
    // that just queries and returns a string, rather than constructing the map
    pub fn get_refs_for_repo(&self, repo: &str) -> Result<HashMap<String, String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT branch, sha FROM branch_refs WHERE remote_url = ?1")?;
        stmt.query_map((repo,), |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect()
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

        let refs = branch_refs.get_refs_for_repo("repo-url").unwrap();

        assert_eq!(
            refs,
            HashMap::from([
                ("branch-1".to_string(), "sha-1".to_string()),
                ("branch-2".to_string(), "sha-3".to_string())
            ])
        )
    }
}
