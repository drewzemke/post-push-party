use crate::storage::DbConnection;
use anyhow::Result;

pub type Migration = fn(&DbConnection) -> Result<()>;

pub const MIGRATIONS: &[Migration] = &[migrate_v1];

fn migrate_v1(conn: &DbConnection) -> Result<()> {
    // create tables
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS player (
            party_points   INTEGER NOT NULL DEFAULT 0,
            points_earned  INTEGER NOT NULL DEFAULT 0,
            packs_earned   INTEGER NOT NULL DEFAULT 0,
            id INTEGER PRIMARY KEY CHECK (id = 1)  -- forces single row
        );

        CREATE TABLE IF NOT EXISTS bonus_tracks (
            id     TEXT PRIMARY KEY,
            level  INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS parties (
            id              TEXT PRIMARY KEY,
            enabled         BOOLEAN NOT NULL DEFAULT true,
            active_palette  TEXT NULL DEFAULT 'White'
        );

        CREATE TABLE IF NOT EXISTS unlocked_palettes (
            party_id      TEXT NOT NULL,
            palette_name  TEXT NOT NULL,
            PRIMARY KEY (party_id, palette_name)
        );

        CREATE TABLE IF NOT EXISTS packs (
            pack_type TEXT PRIMARY KEY,
            count     INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS pushes (
            timestamp      INTEGER NOT NULL,
            remote_url     TEXT NOT NULL,
            branch         TEXT NOT NULL,
            commits        INTEGER NOT NULL,
            lines_changed  INTEGER NOT NULL,
            points_earned  INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS patch_ids (
            remote_url  TEXT NOT NULL,
            patch_id    TEXT NOT NULL,
            PRIMARY KEY (remote_url, patch_id)
        );

        CREATE TABLE IF NOT EXISTS branch_refs (
            remote_url  TEXT NOT NULL,
            branch      TEXT NOT NULL,
            sha         TEXT NOT NULL,
            PRIMARY KEY (remote_url, branch)
        );
        ",
    )?;

    // write default state into the db

    // player
    conn.execute("DELETE FROM player", ())?;
    conn.execute(
        "
                 INSERT INTO player
                     (party_points, points_earned, packs_earned)
                     VALUES (?1, ?2, ?3)
    
            ",
        (0, 0, 0),
    )?;

    // bonus_tracks
    conn.execute("DELETE FROM bonus_tracks", ())?;
    conn.execute(
        "
                 INSERT INTO bonus_tracks
                     (id, level)
                     VALUES (?1, ?2)
    
            ",
        ("commit_value", 1),
    )?;

    // parties
    conn.execute("DELETE FROM parties", ())?;
    conn.execute("INSERT INTO parties (id) VALUES (?1)", ("base",))?;

    // unlocked_palettes
    conn.execute("DELETE FROM unlocked_palettes", ())?;
    conn.execute(
        "
                 INSERT INTO unlocked_palettes
                     (party_id, palette_name)
                     VALUES (?1, ?2)
    
            ",
        ("base", "White"),
    )?;

    Ok(())
}
