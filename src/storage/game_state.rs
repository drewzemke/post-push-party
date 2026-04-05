use crate::storage::DbConnection;
use rusqlite::{OptionalExtension, Result};

pub fn load(conn: &DbConnection, game_id: &str) -> Result<Option<String>> {
    let row: Option<Option<String>> = conn
        .query_one("SELECT state FROM games WHERE id = ?1", (game_id,), |row| {
            row.get::<_, Option<String>>(0)
        })
        .optional()?;
    Ok(row.flatten())
}

pub fn save(conn: &DbConnection, game_id: &str, state: &str) -> Result<()> {
    conn.execute(
        "UPDATE games SET state = ?2 WHERE id = ?1",
        (game_id, state),
    )?;
    Ok(())
}
