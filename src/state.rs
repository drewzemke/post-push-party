use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub party_points: u64,
    pub commit_value_level: u32,

    #[serde(default)]
    pub bonuses: HashMap<BonusType, u32>,

    #[serde(default)]
    pub unlocked_features: HashSet<PartyFeature>,
    #[serde(default)]
    pub enabled_features: HashSet<PartyFeature>,

    // pack_items: HashMap<PackItem, u32>,  // TODO: add when implementing packs

    #[serde(default)]
    pub unlocked_colors: HashMap<PartyFeature, HashSet<Color>>,
    #[serde(default)]
    pub active_color: HashMap<PartyFeature, ColorSelection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BonusType {
    FirstPushOfDay,
    // Streak, BigPush, SpreadTheLove, RapidFire, FridayDeploy, WeekendWarrior
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartyFeature {
    Exclamations,
    Quotes,
    BigText,
    // Stats, Fireworks, ...
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub enum PackItem {
//     SnakeToken,
//     // SlotsToken, ...
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    Red,
    // Blue, Green, ..., Rainbow, ...
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSelection {
    White,
    Specific(Color),
    Random,
}

impl Default for ColorSelection {
    fn default() -> Self {
        Self::White
    }
}

// costs for unlocking each party feature
pub fn feature_cost(feature: PartyFeature) -> u64 {
    match feature {
        PartyFeature::Exclamations => 15,
        PartyFeature::Quotes => 50,
        PartyFeature::BigText => 150,
    }
}

// ordered list of features for display in store/config
pub const PARTY_FEATURES: &[PartyFeature] = &[
    PartyFeature::Exclamations,
    PartyFeature::Quotes,
    PartyFeature::BigText,
];

impl PartyFeature {
    pub fn name(self) -> &'static str {
        match self {
            PartyFeature::Exclamations => "Exclamations",
            PartyFeature::Quotes => "Quotes",
            PartyFeature::BigText => "Big Text",
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            party_points: 0,
            commit_value_level: 1,
            bonuses: HashMap::new(),
            unlocked_features: HashSet::new(),
            enabled_features: HashSet::new(),
            unlocked_colors: HashMap::new(),
            active_color: HashMap::new(),
        }
    }
}

impl State {
    pub fn points_per_commit(&self) -> u64 {
        self.commit_value_level as u64
    }

    pub fn upgrade_cost(&self) -> u64 {
        // 25 → 100 → 400 → 1600 ...
        25 * 4u64.pow(self.commit_value_level - 1)
    }

    pub fn is_unlocked(&self, feature: PartyFeature) -> bool {
        self.unlocked_features.contains(&feature)
    }

    pub fn is_enabled(&self, feature: PartyFeature) -> bool {
        self.unlocked_features.contains(&feature) && self.enabled_features.contains(&feature)
    }

    pub fn unlock_feature(&mut self, feature: PartyFeature) {
        self.unlocked_features.insert(feature);
        self.enabled_features.insert(feature); // enable by default when unlocked
    }

    pub fn toggle_feature(&mut self, feature: PartyFeature) {
        if self.unlocked_features.contains(&feature) {
            if self.enabled_features.contains(&feature) {
                self.enabled_features.remove(&feature);
            } else {
                self.enabled_features.insert(feature);
            }
        }
    }
}

pub fn state_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("PARTY_STATE_DIR") {
        return Some(PathBuf::from(dir));
    }
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

pub fn status() {
    let state = load();
    println!("You have {} party points.", state.party_points);
}

pub fn dump() {
    let state = load();
    println!("party_points: {}", state.party_points);
    println!("commit_value_level: {}", state.commit_value_level);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("upgrade_cost: {}", state.upgrade_cost());
    println!("unlocked_features: {:?}", state.unlocked_features);
    println!("enabled_features: {:?}", state.enabled_features);
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
    fn default_state_has_no_unlocks() {
        let state = State::default();
        assert!(state.unlocked_features.is_empty());
        assert!(state.enabled_features.is_empty());
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
    fn unlock_feature_adds_to_both_sets() {
        let mut state = State::default();
        state.unlock_feature(PartyFeature::Exclamations);

        assert!(state.is_unlocked(PartyFeature::Exclamations));
        assert!(state.is_enabled(PartyFeature::Exclamations));
    }

    #[test]
    fn toggle_feature_works() {
        let mut state = State::default();
        state.unlock_feature(PartyFeature::Quotes);

        assert!(state.is_enabled(PartyFeature::Quotes));
        state.toggle_feature(PartyFeature::Quotes);
        assert!(!state.is_enabled(PartyFeature::Quotes));
        state.toggle_feature(PartyFeature::Quotes);
        assert!(state.is_enabled(PartyFeature::Quotes));
    }

    #[test]
    fn toggle_locked_feature_does_nothing() {
        let mut state = State::default();
        state.toggle_feature(PartyFeature::BigText);
        assert!(!state.is_enabled(PartyFeature::BigText));
    }

    #[test]
    fn save_and_load_roundtrips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.bin");

        let mut state = State {
            party_points: 42,
            commit_value_level: 3,
            ..State::default()
        };
        state.unlock_feature(PartyFeature::Exclamations);

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
