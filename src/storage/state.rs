use std::collections::HashMap;

use rusqlite::{
    Result as RusqliteResult, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

use crate::{
    state::{PaletteSelection, State},
    storage::DbConnection,
};

impl State {
    pub fn load(conn: &DbConnection) -> RusqliteResult<Self> {
        // player
        let (party_points, points_earned, _packs_earned): (i64, i64, i64) = conn.query_one(
            "
            SELECT party_points, points_earned, packs_earned from player WHERE id = 1;
        ",
            (),
            |x| Ok((x.get(0)?, x.get(1)?, x.get(2)?)),
        )?;

        // bonus tracks
        let mut stmt = conn.prepare("SELECT id, level FROM bonus_tracks")?;
        let bonus_tracks = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<RusqliteResult<HashMap<String, u32>>>()?;

        // parties
        let mut stmt = conn.prepare("SELECT id, enabled, active_palette FROM parties")?;
        let parties = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<RusqliteResult<Vec<(String, bool, PaletteSelection)>>>()?;

        let unlocked_parties = parties.iter().map(|(id, _, _)| id.clone()).collect();
        let enabled_parties = parties
            .iter()
            .filter_map(|(id, enabled, _)| enabled.then_some(id.clone()))
            .collect();
        let active_palettes = parties
            .into_iter()
            .map(|(id, _, palette)| (id, palette))
            .collect();

        // unlocked palettes
        let mut stmt = conn.prepare("SELECT party_id, palette_name FROM unlocked_palettes")?;
        let unlocked_palette_pairs = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<RusqliteResult<Vec<(String, String)>>>()?;
        let mut unlocked_palettes: HashMap<String, Vec<String>> = HashMap::new();
        for (party_id, palette_name) in unlocked_palette_pairs {
            unlocked_palettes
                .entry(party_id)
                .and_modify(|v| v.push(palette_name.clone()))
                .or_insert(Vec::from([palette_name]));
        }

        let state = Self::new(
            party_points as u64,
            points_earned as u64,
            bonus_tracks,
            unlocked_parties,
            enabled_parties,
            unlocked_palettes,
            active_palettes,
            // FIXME: add packs earned
        );
        Ok(state)
    }

    pub fn save(&self, conn: &DbConnection) -> RusqliteResult<()> {
        let tx = conn.unchecked_transaction()?;

        // player
        conn.execute(
            "
                 UPDATE player SET
                     party_points = ?1,
                     points_earned = ?2,
                     packs_earned = ?3
                     WHERE id = 1;
    
            ",
            (
                self.party_points as i64,
                self.lifetime_points_earned as i64,
                // FIXME: restore this to `packs_earned`
                0,
            ),
        )?;

        // bonus_tracks
        {
            let mut stmt =
                tx.prepare("INSERT OR REPLACE INTO bonus_tracks (id, level) VALUES (?1, ?2)")?;
            for (track_id, level) in &self.bonus_tracks {
                stmt.execute((track_id, level))?;
            }
        }

        // parties
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO parties (id, enabled, active_palette ) VALUES (?1, ?2, ?3)",
            )?;
            for party_id in &self.unlocked_parties {
                let enabled = self.is_party_enabled(party_id);
                let selected_palette = self.selected_palette(party_id).cloned().unwrap_or_default();
                stmt.execute((party_id, enabled, selected_palette))?;
            }
        }

        // unlocked_palettes
        {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO unlocked_palettes (party_id, palette_name) VALUES (?1, ?2)",
            )?;
            for (party_id, palettes) in &self.unlocked_palettes {
                for palette in palettes {
                    stmt.execute((party_id, palette))?;
                }
            }
        }

        tx.commit()?;

        Ok(())
    }
}

impl ToSql for PaletteSelection {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        match self {
            PaletteSelection::Specific(s) => Ok(ToSqlOutput::from(s.as_str())),
            PaletteSelection::Random => Ok(ToSqlOutput::Owned(Value::Null)),
        }
    }
}

impl FromSql for PaletteSelection {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Null => Ok(Self::Random),
            ValueRef::Text(items) => {
                let str = String::from_utf8(items.to_vec())
                    .map_err(|_| FromSqlError::Other("Unparsable string".into()))?;
                Ok(Self::Specific(str))
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
