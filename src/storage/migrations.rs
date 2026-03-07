use crate::{
    git::{detection, patch_ids},
    history,
    state::{load_from_path, old_state_dir, old_state_path},
    storage::DbConnection,
};
use anyhow::Result;

pub type Migration = fn(&DbConnection) -> Result<()>;

pub const MIGRATIONS: &[Migration] = &[migrate_v1];

// FIXME: turn this off before shipping
pub const PRINT_DEBUG: bool = true;
fn debug(s: &str) {
    if PRINT_DEBUG {
        println!("{s}");
    }
}

fn migrate_v1(conn: &DbConnection) -> Result<()> {
    debug("\n---\nmigrating party storage to sqlite\n");

    // create tables
    debug("- creating tables");
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

    // look for existing state directory
    // don't do this if the db is in-memory
    let in_memory = conn.path().is_some_and(|s| s.is_empty());

    let state_dir = old_state_dir();
    if let Some(dir) = state_dir
        && !in_memory
        && std::fs::exists(dir)?
    {
        debug("- found old state directory");
        // load state the old way
        let state = old_state_path()
            .map(|p| load_from_path(&p))
            .unwrap_or_default();

        debug("-- writing old state into db");

        // player
        conn.execute("DELETE FROM player", ())?;
        conn.execute(
            "
                 INSERT INTO player
                     (party_points, points_earned, packs_earned)
                     VALUES (?1, ?2, ?3)
    
            ",
            (
                state.party_points as i64,
                state.lifetime_points_earned as i64,
                0,
            ),
        )?;

        // bonus_tracks
        conn.execute("DELETE FROM bonus_tracks", ())?;
        let mut stmt = conn.prepare("INSERT INTO bonus_tracks (id, level) VALUES (?1, ?2)")?;
        for (track_id, level) in &state.bonus_levels {
            stmt.execute((track_id, level))?;
        }

        // parties
        conn.execute("DELETE FROM parties", ())?;
        let mut stmt = conn.prepare("INSERT INTO parties (id, enabled) VALUES (?1, ?2)")?;
        for party_id in &state.unlocked_parties {
            let enabled = state.enabled_parties.contains(party_id);
            stmt.execute((party_id, enabled))?;
        }

        // unlocked_palettes -- don't need to read from state since no one
        // has been able to unlock things
        conn.execute("DELETE FROM unlocked_palettes", ())?;
        conn.execute(
            "
                 INSERT INTO unlocked_palettes
                     (party_id, palette_name)
                     VALUES (?1, ?2)
    
            ",
            ("base", "White"),
        )?;

        // pushes
        conn.execute("DELETE FROM pushes", ())?;
        let history = history::load();
        let mut stmt = conn.prepare(
            "
                INSERT INTO pushes
                    (timestamp, remote_url, branch, commits, lines_changed, points_earned)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ",
        )?;
        for entry in history.entries() {
            stmt.execute((
                entry.timestamp() as i64,
                entry.remote_url(),
                entry.branch(),
                entry.commits() as i64,
                entry.lines_changed() as i64,
                entry.points_earned() as i64,
            ))?;
        }

        // patch_ids
        conn.execute("DELETE FROM patch_ids", ())?;
        let patch_ids = patch_ids::load();
        let mut stmt =
            conn.prepare("INSERT INTO patch_ids (remote_url, patch_id) VALUES (?1, ?2)")?;
        for (repo, ids) in &patch_ids.repos {
            for id in ids {
                stmt.execute((repo, id))?;
            }
        }

        // branch_refs
        conn.execute("DELETE FROM branch_refs", ())?;
        let branch_refs = detection::load_refs();
        let mut stmt =
            conn.prepare("INSERT INTO branch_refs (remote_url, branch, sha) VALUES (?1, ?2, ?3)")?;
        for (repo, ref_map) in &branch_refs.repos {
            for (branch, sha) in ref_map {
                stmt.execute((repo, branch, sha))?;
            }
        }

        debug("-- done migrating db state");
    } else {
        debug("- no existing old state directory found");
        debug("-- writing default state into db");

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
        debug("-- done initializing db state");
    }

    debug("\ndone with v1 migration\n---\n");
    Ok(())
}
