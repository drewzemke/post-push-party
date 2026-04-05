use std::collections::HashMap;

use rusqlite::{
    Result as RusqliteResult, ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
};

use crate::{
    pack::Pack,
    state::{PaletteSelection, State},
    storage::DbConnection,
};

impl State {
    pub fn load(conn: &DbConnection) -> RusqliteResult<Self> {
        // player
        let (party_points, points_earned, packs_earned): (i64, i64, i64) = conn.query_one(
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
        let mut stmt = conn.prepare("SELECT party_id, palette_id FROM palettes")?;
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

        // packs
        let mut stmt = conn.prepare("SELECT pack_type, count FROM packs")?;
        let packs = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<RusqliteResult<HashMap<Pack, u32>>>()?;

        let state = Self::new(
            party_points as u64,
            points_earned as u64,
            packs_earned as u64,
            bonus_tracks,
            unlocked_parties,
            enabled_parties,
            unlocked_palettes,
            active_palettes,
            packs,
            todo!(),
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
                self.lifetime_packs_earned as i64,
            ),
        )?;

        // bonus_tracks
        {
            tx.execute("DELETE FROM bonus_tracks", ())?;
            let mut stmt =
                tx.prepare("INSERT OR REPLACE INTO bonus_tracks (id, level) VALUES (?1, ?2)")?;
            for (track_id, level) in &self.bonus_tracks {
                stmt.execute((track_id, level))?;
            }
        }

        // parties
        {
            tx.execute("DELETE FROM parties", ())?;
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO parties (id, enabled, active_palette ) VALUES (?1, ?2, ?3)",
            )?;
            for party_id in &self.unlocked_parties {
                let enabled = self.is_party_enabled(party_id);
                let selected_palette = self.selected_palette(party_id).cloned().unwrap_or_default();
                stmt.execute((party_id, enabled, selected_palette))?;
            }
        }

        // palettes
        {
            tx.execute("DELETE FROM palettes", ())?;
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO palettes (party_id, palette_id) VALUES (?1, ?2)",
            )?;
            for (party_id, palette_ids) in &self.unlocked_palettes {
                for palette_id in palette_ids {
                    stmt.execute((party_id, palette_id))?;
                }
            }
        }

        // packs
        {
            tx.execute("DELETE FROM packs", ())?;
            let mut stmt =
                tx.prepare("INSERT OR REPLACE INTO packs (pack_type, count) VALUES (?1, ?2)")?;
            for (pack, count) in &self.packs {
                stmt.execute((pack, count))?;
            }
        }

        tx.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod state_storage_tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let conn = DbConnection::create_in_memory().unwrap();

        let mut state = State {
            lifetime_points_earned: 12,
            party_points: 42,
            ..State::default()
        };
        state.set_bonus_level("commit_value", 3);
        state.set_bonus_level("first_push", 2);
        state.unlock_party("exclamations");
        state.unlock_palette("base", "Rainbow");
        state.set_selected_palette("base", 1);
        state.set_selected_palette("exclamations", 3);
        state.add_pack(Pack::Basic);

        state.save(&conn).unwrap();
        let loaded = State::load(&conn).unwrap();

        assert_eq!(loaded, state);
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

impl ToSql for Pack {
    fn to_sql(&self) -> RusqliteResult<ToSqlOutput<'_>> {
        match self {
            Pack::Basic => Ok(ToSqlOutput::from("basic")),
            Pack::Premium => Ok(ToSqlOutput::from("premium")),
        }
    }
}

impl FromSql for Pack {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(items) => {
                let str = String::from_utf8(items.to_vec())
                    .map_err(|_| FromSqlError::Other("Unparsable string".into()))?;
                match str.as_str() {
                    "basic" => Ok(Pack::Basic),
                    "premium" => Ok(Pack::Premium),
                    _ => Err(FromSqlError::InvalidType),
                }
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[cfg(test)]
mod pack_sql_conversion_tests {
    use crate::pack::ALL_PACKS;

    use super::*;

    #[test]
    fn all_variants_covered() {
        for pack in ALL_PACKS {
            let sql = pack.to_sql().unwrap();
            let ToSqlOutput::Borrowed(sql) = sql else {
                panic!();
            };
            let pack_after = Pack::column_result(sql).unwrap();
            assert_eq!(*pack, pack_after);
        }
    }
}
