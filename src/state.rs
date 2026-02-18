use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::bonus_track::{ALL_TRACKS, Reward};
use crate::party::{Palette, Party};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub party_points: u64,

    #[serde(default)]
    pub lifetime_points_earned: u64,

    /// refers to bonus tracks by their identifier string
    #[serde(default)]
    pub bonus_levels: HashMap<String, u32>,

    /// which parties the user has unlocked via the store.
    /// refers to parties by their identifier string
    #[serde(default)]
    pub unlocked_parties: HashSet<String>,

    /// which parties have been enabled by the user.
    /// refers to parties by their identifier string
    #[serde(default)]
    pub enabled_parties: HashSet<String>,

    // pack_items: HashMap<PackItem, u32>,  // TODO: add when implementing packs
    //
    /// which palettes the user has unlocked for each party.
    /// refers to parties by their identifier string, and to palettes by their names
    #[serde(default)]
    pub unlocked_palettes: HashMap<String, HashSet<String>>,

    /// which palette is currently configured for each party.
    /// refers to parties by their identifier string, palettes by their name
    #[serde(default)]
    pub active_palette: HashMap<String, PaletteSelection>,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub enum PackItem {
//     SnakeToken,
//     // SlotsToken, ...
// }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaletteSelection {
    Specific(String), // palette name
    Random,
}

impl Default for PaletteSelection {
    fn default() -> Self {
        Self::Specific(Palette::WHITE.name().to_string())
    }
}

impl Default for State {
    fn default() -> Self {
        let mut bonus_levels = HashMap::new();
        bonus_levels.insert("commit_value".to_string(), 1);

        let mut unlocked_parties = HashSet::new();
        unlocked_parties.insert("base".to_string());

        Self {
            party_points: 0,
            lifetime_points_earned: 0,
            bonus_levels,
            enabled_parties: unlocked_parties.clone(),
            unlocked_parties,
            unlocked_palettes: HashMap::new(),
            active_palette: HashMap::new(),
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
            if track.id() == "commit_value"
                && let Some(Reward::FlatPoints(n)) = track.reward_at_level(level)
            {
                return n;
            }
        }
        1
    }

    pub fn is_party_unlocked(&self, id: &str) -> bool {
        self.unlocked_parties.contains(id)
    }

    pub fn is_party_enabled(&self, id: &str) -> bool {
        self.unlocked_parties.contains(id) && self.enabled_parties.contains(id)
    }

    pub fn unlock_party(&mut self, id: &str) {
        self.unlocked_parties.insert(id.to_string());

        // enable by default when unlocked
        self.enabled_parties.insert(id.to_string());
    }

    pub fn toggle_party(&mut self, id: &str) {
        if self.unlocked_parties.contains(id) {
            if self.enabled_parties.contains(id) {
                self.enabled_parties.remove(id);
            } else {
                self.enabled_parties.insert(id.to_string());
            }
        }
    }

    pub fn unlocked_palettes(&self, party_id: &str) -> Option<&HashSet<String>> {
        self.unlocked_palettes.get(party_id)
    }

    pub fn selected_palette(&self, party_id: &str) -> Option<&PaletteSelection> {
        self.active_palette.get(party_id)
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

pub fn points() {
    let state = load();
    println!("You have {} party points.", state.party_points);
}

pub fn stats() {
    let state = load();

    if !state.is_party_unlocked("stats") {
        println!("You haven't unlocked the Stats party yet.");
        return;
    }

    let history = crate::history::load();
    let clock = crate::clock::Clock::from_now();
    let push = crate::git::Push::default();
    let breakdown = crate::scoring::PointsBreakdown {
        commits: 0,
        points_per_commit: 0,
        total: 0,
        applied: vec![],
    };

    let ctx = crate::party::RenderContext::new(&push, &history, &breakdown, &state, &clock);
    crate::party::stats::Stats.render(&ctx, &crate::party::Palette::WHITE);
}

pub fn dump() {
    let state = load();
    println!("party_points: {}", state.party_points);
    println!("lifetime_points_earned: {}", state.lifetime_points_earned);
    println!("points_per_commit: {}", state.points_per_commit());
    println!("bonus_levels: {:?}", state.bonus_levels);
    println!("unlocked_parties: {:?}", state.unlocked_parties);
    println!("enabled_parties: {:?}", state.enabled_parties);
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
    fn default_state_has_one_unlock() {
        let state = State::default();
        assert_eq!(state.unlocked_parties.len(), 1);
        assert_eq!(state.enabled_parties.len(), 1);
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
        let id = "exclamations";
        state.unlock_party(id);

        assert!(state.is_party_unlocked(id));
        assert!(state.is_party_enabled(id));
    }

    #[test]
    fn toggle_feature_works() {
        let mut state = State::default();
        let id = "exclamations";

        state.unlock_party(id);

        assert!(state.is_party_enabled(id));
        state.toggle_party(id);
        assert!(!state.is_party_enabled(id));
        state.toggle_party(id);
        assert!(state.is_party_enabled(id));
    }

    #[test]
    fn toggle_locked_party_does_nothing() {
        let mut state = State::default();
        let id = "big_text";

        state.toggle_party(id);
        assert!(!state.is_party_enabled(id));
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
        state.unlock_party("exclamations");

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
