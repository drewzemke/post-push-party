use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::bonus_tracks::{Reward, ALL_TRACKS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub party_points: u64,

    #[serde(default)]
    pub lifetime_points_earned: u64,

    #[serde(default)]
    pub bonus_levels: HashMap<String, u32>,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorSelection {
    #[default]
    White,
    Specific(Color),
    Random,
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

    pub fn description(self) -> &'static str {
        match self {
            PartyFeature::Exclamations => "Adds an excited shout to your party.",
            PartyFeature::Quotes => "An inspirational quote after each push.",
            PartyFeature::BigText => "Finish your party with a full screen word. NICE!",
        }
    }
}

impl Default for State {
    fn default() -> Self {
        let mut bonus_levels = HashMap::new();
        bonus_levels.insert("commit_value".to_string(), 1); // users start at tier 1

        Self {
            party_points: 0,
            lifetime_points_earned: 0,
            bonus_levels,
            unlocked_features: HashSet::new(),
            enabled_features: HashSet::new(),
            unlocked_colors: HashMap::new(),
            active_color: HashMap::new(),
        }
    }
}

impl State {
    pub fn earn_points(&mut self, amount: u64) {
        self.party_points += amount;
        self.lifetime_points_earned += amount;
    }

    pub fn bonus_level(&self, id: &str) -> u32 {
        self.bonus_levels.get(id).copied().unwrap_or(0)
    }

    pub fn set_bonus_level(&mut self, id: &str, level: u32) {
        self.bonus_levels.insert(id.to_string(), level);
    }

    pub fn points_per_commit(&self) -> u64 {
        let level = self.bonus_level("commit_value");
        if level == 0 {
            return 1;
        }
        // find commit_value track and get reward
        for track in ALL_TRACKS.iter() {
            if track.id() == "commit_value" {
                if let Some(Reward::FlatPoints(n)) = track.reward_at_level(level) {
                    return n;
                }
            }
        }
        1
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
    println!("lifetime_points_earned: {}", state.lifetime_points_earned);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("bonus_levels: {:?}", state.bonus_levels);
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
        assert_eq!(state.lifetime_points_earned, 0);
    }

    #[test]
    fn earn_points_updates_both_balances() {
        let mut state = State::default();
        state.earn_points(100);
        assert_eq!(state.party_points, 100);
        assert_eq!(state.lifetime_points_earned, 100);

        // spend some points
        state.party_points -= 30;
        state.earn_points(50);
        assert_eq!(state.party_points, 120);
        assert_eq!(state.lifetime_points_earned, 150);
    }

    #[test]
    fn default_state_has_commit_value_at_level_one() {
        let state = State::default();
        assert_eq!(state.bonus_level("commit_value"), 1);
    }

    #[test]
    fn default_state_has_no_unlocks() {
        let state = State::default();
        assert!(state.unlocked_features.is_empty());
        assert!(state.enabled_features.is_empty());
    }

    #[test]
    fn bonus_level_returns_zero_for_missing() {
        let state = State::default();
        assert_eq!(state.bonus_level("nonexistent"), 0);
    }

    #[test]
    fn set_bonus_level_works() {
        let mut state = State::default();
        state.set_bonus_level("first_push", 3);
        assert_eq!(state.bonus_level("first_push"), 3);
    }

    #[test]
    fn points_per_commit_uses_commit_value_level() {
        let mut state = State::default();
        assert_eq!(state.points_per_commit(), 1);

        state.set_bonus_level("commit_value", 2);
        assert_eq!(state.points_per_commit(), 2);

        state.set_bonus_level("commit_value", 5);
        assert_eq!(state.points_per_commit(), 5);
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
            ..State::default()
        };
        state.set_bonus_level("commit_value", 3);
        state.set_bonus_level("first_push", 2);
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
