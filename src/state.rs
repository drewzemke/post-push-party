use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub party_points: u64,
    pub commit_value_level: u32,
    #[serde(default)]
    pub party_level: u32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            party_points: 0,
            commit_value_level: 1,
            party_level: 0,
        }
    }
}

pub const PARTY_LEVELS: &[PartyLevel] = &[
    PartyLevel {
        name: "Basic",
        cost: 0,
    },
    PartyLevel {
        name: "Colorful",
        cost: 15,
    },
    PartyLevel {
        name: "Quotes",
        cost: 50,
    },
    PartyLevel {
        name: "Big Text",
        cost: 150,
    },
];

pub struct PartyLevel {
    pub name: &'static str,
    pub cost: u64,
}

impl State {
    pub fn points_per_commit(&self) -> u64 {
        self.commit_value_level as u64
    }

    pub fn upgrade_cost(&self) -> u64 {
        // 25 → 100 → 400 → 1600 ...
        25 * 4u64.pow(self.commit_value_level - 1)
    }

    pub fn party_level_name(&self) -> &'static str {
        PARTY_LEVELS
            .get(self.party_level as usize)
            .map(|l| l.name)
            .unwrap_or("Max")
    }

    pub fn next_party_level(&self) -> Option<&'static PartyLevel> {
        PARTY_LEVELS.get(self.party_level as usize + 1)
    }

    pub fn party_upgrade_cost(&self) -> Option<u64> {
        self.next_party_level().map(|l| l.cost)
    }
}

pub fn state_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".post-push-party"))
}

pub fn state_path() -> Option<PathBuf> {
    state_dir().map(|d| d.join("state.bin"))
}

pub fn load() -> State {
    state_path().map(|p| load_from_path(&p)).unwrap_or_default()
}

pub fn save(state: &State) -> std::io::Result<()> {
    let path = state_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })?;
    save_to_path(state, &path)
}

pub fn load_from_path(path: &std::path::Path) -> State {
    match std::fs::read(path) {
        Ok(bytes) => bincode::deserialize(&bytes).unwrap_or_default(),
        Err(_) => State::default(),
    }
}

pub fn save_to_path(state: &State, path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(state).map_err(std::io::Error::other)?;
    std::fs::write(path, encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn default_state_has_zero_points() {
        let state = State::default();
        assert_eq!(state.party_points, 0);
    }

    #[test]
    fn default_state_has_level_one() {
        let state = State::default();
        assert_eq!(state.commit_value_level, 1);
    }

    #[test]
    fn points_per_commit_equals_level() {
        let mut state = State::default();
        assert_eq!(state.points_per_commit(), 1);

        state.commit_value_level = 5;
        assert_eq!(state.points_per_commit(), 5);
    }

    #[test]
    fn upgrade_cost_scales() {
        let mut state = State::default();
        assert_eq!(state.upgrade_cost(), 25); // 25 * 4^0

        state.commit_value_level = 2;
        assert_eq!(state.upgrade_cost(), 100); // 25 * 4^1

        state.commit_value_level = 3;
        assert_eq!(state.upgrade_cost(), 400); // 25 * 4^2
    }

    #[test]
    fn save_and_load_roundtrips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.bin");

        let state = State {
            party_points: 42,
            commit_value_level: 3,
            party_level: 1,
        };

        save_to_path(&state, &path).unwrap();
        let loaded = load_from_path(&path);

        assert_eq!(loaded, state);
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.bin");

        let loaded = load_from_path(&path);
        assert_eq!(loaded, State::default());
    }
}
