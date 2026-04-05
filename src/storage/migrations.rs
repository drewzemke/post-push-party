use crate::storage::DbConnection;
use anyhow::Result;

pub type Migration = fn(&DbConnection) -> Result<()>;

pub const MIGRATIONS: &[Migration] = &[migrate_v1, migrate_v2, migrate_v3];

/// initial table construction and state population
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
            -- NOTE: default has changed and is now handled in the code
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

/// update palette refs to be by id rather than name
fn migrate_v2(conn: &DbConnection) -> Result<()> {
    // convert `active_palette` in `parties` values from their old White/Red/Cyan to ansi-white, ansi-red, ansi-cyan
    conn.execute(
        "
        UPDATE parties SET active_palette = CASE active_palette
            WHEN 'White'     THEN 'white-ansi'
            WHEN 'Red'       THEN 'red-ansi' 
            WHEN 'Green'     THEN 'green-ansi' 
            WHEN 'Blue'      THEN 'blue-ansi' 
            WHEN 'Cyan'      THEN 'cyan-ansi' 
            WHEN 'Yellow'    THEN 'yellow-ansi' 
            WHEN 'Magenta'   THEN 'magenta-ansi' 
            WHEN 'Synthwave' THEN 'synthwave' 
            ELSE 'white-ansi'
        END;
        ",
        [],
    )?;

    // add a column `palette_id` to `unlocked_palettes` with converted values,
    // remove `palette_name` column and update the primary key in that table
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS palettes (
            party_id      TEXT NOT NULL,
            palette_id  TEXT NOT NULL,
            PRIMARY KEY (party_id, palette_id)
        );
       
        INSERT INTO palettes (palette_id, party_id)
        SELECT CASE palette_name
            WHEN 'White'     THEN 'white-ansi'
            WHEN 'Red'       THEN 'red-ansi' 
            WHEN 'Green'     THEN 'green-ansi' 
            WHEN 'Blue'      THEN 'blue-ansi' 
            WHEN 'Cyan'      THEN 'cyan-ansi' 
            WHEN 'Yellow'    THEN 'yellow-ansi' 
            WHEN 'Magenta'   THEN 'magenta-ansi' 
            WHEN 'Synthwave' THEN 'synthwave' 
            ELSE 'white-ansi'
        END, party_id
        FROM unlocked_palettes;

        DROP TABLE unlocked_palettes;
        ",
    )?;

    Ok(())
}

/// add games table
fn migrate_v3(conn: &DbConnection) -> Result<()> {
    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS games (
            id     TEXT PRIMARY KEY,
            count  INTEGER NOT NULL
        );
        ",
        [],
    )?;

    Ok(())
}
