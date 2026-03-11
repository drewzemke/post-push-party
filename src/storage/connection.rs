use anyhow::Result;
use rusqlite::Connection;
use std::{fs, path::PathBuf};

use crate::{
    log::{self},
    state,
    storage::{migrations::MIGRATIONS, storage_dir},
};

#[derive(Debug)]
pub struct DbConnection(Connection);

impl std::ops::Deref for DbConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const DB_FILE_NAME: &str = "party.db";

fn db_path() -> Result<PathBuf> {
    super::storage_dir().map(|p| p.join(DB_FILE_NAME))
}

impl DbConnection {
    /// creates a connection to the sqlite db in the user's data storage directory,
    /// then runs migrations
    pub fn create() -> Result<Self> {
        let db_path = db_path()?;
        let conn = Connection::open(db_path)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let conn = Self(conn);
        conn.run_migrations()?;

        Ok(conn)
    }

    /// creates a connection to a fresh db in memory, then runs all migrations
    #[cfg(test)]
    pub fn create_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let conn = Self(conn);
        conn.run_migrations()?;

        Ok(conn)
    }

    fn run_migrations(&self) -> Result<()> {
        let current_version: u32 = self.pragma_query_value(None, "user_version", |r| r.get(0))?;

        for (i, migration) in MIGRATIONS.iter().enumerate() {
            let version = i as u32 + 1;
            if version > current_version {
                let tx = self.unchecked_transaction()?;

                migration(self)?;

                // update version
                tx.execute(&format!("PRAGMA user_version = {version}"), [])?;

                tx.commit()?;

                // HACK: delete after we don't need to handle old state anymore
                // logs
                let in_memory = self.path().is_some_and(|s| s.is_empty());
                if version == 1 && !in_memory {
                    if let Some(from_path) = log::old_log_path()
                        && from_path.exists()
                        && let Ok(dir) = storage_dir()
                    {
                        let dest_path = dir.join("party.log");
                        fs::copy(from_path, dest_path)?;
                    }

                    // delete old state
                    if let Some(dir) = state::old_state_dir_no_override()
                        && dir.exists()
                    {
                        fs::remove_dir_all(dir)?
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_run_on_fresh_db() {
        let conn = DbConnection::create_in_memory();

        // should not panic
        conn.unwrap();
    }
}
