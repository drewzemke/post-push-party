use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::bonus_track::{ALL_TRACKS, Reward};
use crate::pack::Pack;
use crate::party::{Palette, Party};

/// measures how quickly the player gains packs automatically based
/// on lifetime points. specifically it's the rate of increase of
/// difference between subsequent break points
///
/// eg. if the value is 25, then the player will get points
/// at: 0  + 1 * 25 = 25
///     25 + 2 * 25 = 75
///     75 + 3 * 25 = 150
///     ... etc
const PACK_ACCRUAL_RATE: u64 = 25;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub party_points: u64,

    #[serde(default)]
    pub lifetime_points_earned: u64,

    /// refers to bonus tracks by their identifier string
    #[serde(default)]
    bonus_levels: HashMap<String, u32>,

    /// which parties the player has unlocked via the store.
    /// refers to parties by their identifier string
    #[serde(default)]
    unlocked_parties: HashSet<String>,

    /// which parties have been enabled by the player.
    /// refers to parties by their identifier string
    #[serde(default)]
    enabled_parties: HashSet<String>,

    /// which palettes the player has unlocked for each party.
    /// refers to parties by their identifier string, and to palettes by their names
    #[serde(default)]
    pub unlocked_palettes: HashMap<String, Vec<String>>,

    /// which palette is currently configured for each party.
    /// refers to parties by their identifier string, palettes by their name
    #[serde(default)]
    active_palettes: HashMap<String, PaletteSelection>,

    /// how many packs of each time the player has
    #[serde(default)]
    pub packs: HashMap<Pack, u32>,

    /// how many packs have been earned though the points accrual mechanism
    #[serde(default)]
    pub lifetime_packs_earned: u64,
}

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

        let white = Palette::WHITE.name().to_string();

        Self {
            party_points: 0,
            lifetime_points_earned: 0,
            bonus_levels,
            enabled_parties: unlocked_parties.clone(),
            unlocked_parties,
            unlocked_palettes: HashMap::from([("base".to_string(), vec![white.clone()])]),
            active_palettes: HashMap::from([(
                "base".to_string(),
                PaletteSelection::Specific(white),
            )]),
            packs: HashMap::new(),
            lifetime_packs_earned: 0,
        }
    }
}

impl State {
    /// if packs were earned as a result of earning these points,
    /// this returns the list of point thresholds that were
    /// crossed. otherwise an empty list
    pub fn earn_points(&mut self, amount: u64) -> Vec<u64> {
        self.party_points += amount;
        self.lifetime_points_earned += amount;

        let mut thresholds = Vec::new();

        // check if we've crossed a threshold for which we should
        // grant packs. the thresholds values are
        //   PACK_ACCRUAL_RATE * (n+1) * (n+2) / 2
        // where n is the number of packs earned in this way so far
        let mut threshold =
            PACK_ACCRUAL_RATE * (self.lifetime_packs_earned + 1) * (self.lifetime_packs_earned + 2)
                / 2;
        while threshold <= self.lifetime_points_earned {
            self.lifetime_packs_earned += 1;
            self.add_pack(Pack::Basic);
            thresholds.push(threshold);

            threshold = PACK_ACCRUAL_RATE
                * (self.lifetime_packs_earned + 1)
                * (self.lifetime_packs_earned + 2)
                / 2
        }

        thresholds
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
        self.enabled_parties.insert(id.to_string());

        // seed with white palette if no palettes unlocked yet
        if !self.unlocked_palettes.contains_key(id) {
            let white = Palette::WHITE.name().to_string();
            self.unlocked_palettes
                .insert(id.to_string(), vec![white.clone()]);
            self.active_palettes
                .insert(id.to_string(), PaletteSelection::Specific(white));
        }
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

    pub fn unlocked_palettes(&self, party_id: &str) -> Option<&Vec<String>> {
        self.unlocked_palettes.get(party_id)
    }

    pub fn selected_palette(&self, party_id: &str) -> Option<&PaletteSelection> {
        self.active_palettes.get(party_id)
    }

    /// the index of the selected palette for the given party.
    /// returns the length of the unlocked palettes list if "random" is selected.
    /// falls back to 0 if state is somehow missing.
    pub fn selected_palette_idx(&self, party_id: &str) -> usize {
        let Some(palettes) = self.unlocked_palettes(party_id) else {
            return 0;
        };
        let Some(selected) = self.selected_palette(party_id) else {
            return 0;
        };
        match selected {
            PaletteSelection::Specific(palette_name) => palettes
                .iter()
                .position(|name| *name == *palette_name)
                .unwrap_or(0),
            PaletteSelection::Random => palettes.len(),
        }
    }

    /// sets the selected palette for a party based on its index in the list of available palettes
    ///
    /// NOTE: if the index is outside of the valid range, the palette selection will be set to "random"
    pub fn set_selected_palette(&mut self, party_id: &str, palette_idx: usize) {
        let palettes = self.unlocked_palettes(party_id);
        let palette_name = palettes.and_then(|palettes| palettes.get(palette_idx));
        let selection = match palette_name {
            Some(name) => PaletteSelection::Specific(name.to_string()),
            None => PaletteSelection::Random,
        };
        self.active_palettes.insert(party_id.to_string(), selection);
    }

    /// adds a pack to the player's inventory
    pub fn add_pack(&mut self, pack: Pack) {
        self.packs.entry(pack).and_modify(|n| *n += 1).or_insert(1);
    }

    /// how many packs of the given type the player has
    pub fn pack_count(&self, pack: &Pack) -> u32 {
        self.packs.get(pack).copied().unwrap_or_default()
    }

    /// decrements the number of packs of a given type
    pub fn open_pack(&mut self, pack: Pack) {
        self.packs
            .entry(pack)
            .and_modify(|n| *n = n.saturating_sub(1));
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

    let ctx =
        crate::party::RenderContext::new(&push, &history, &breakdown, &state, &clock, Vec::new());
    crate::party::stats::Stats.render(&ctx, &crate::party::Palette::WHITE);
}

pub fn dump() {
    let state = load();
    println!("party_points: {}", state.party_points);
    println!("lifetime_points_earned: {}", state.lifetime_points_earned);
    println!("lifetime_packs_earned: {}", state.lifetime_packs_earned);
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
    fn default_state_has_zero_points_or_packs() {
        let state = State::default();
        assert_eq!(state.party_points, 0);
        assert_eq!(state.lifetime_points_earned, 0);
        assert_eq!(state.packs.values().count(), 0)
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

    #[test]
    fn test_add_and_open_pack() {
        let mut state = State::default();
        assert_eq!(state.pack_count(&Pack::Basic), 0);

        state.add_pack(Pack::Basic);
        assert_eq!(state.pack_count(&Pack::Basic), 1);

        state.open_pack(Pack::Basic);
        assert_eq!(state.pack_count(&Pack::Basic), 0);

        // nothing breaks
        state.open_pack(Pack::Basic);
        assert_eq!(state.pack_count(&Pack::Basic), 0);
    }

    #[test]
    fn get_packs_based_on_lifetime_points() {
        let mut state = State::default();

        assert_eq!(state.lifetime_packs_earned, 0);
        assert_eq!(state.pack_count(&Pack::Basic), 0);

        // should earn 1 pack
        let thresholds = state.earn_points(PACK_ACCRUAL_RATE);

        assert_eq!(thresholds, vec![PACK_ACCRUAL_RATE]);
        assert_eq!(state.lifetime_packs_earned, 1);
        assert_eq!(state.pack_count(&Pack::Basic), 1);

        // should earn 2 packs at once
        let thresholds = state.earn_points(5 * PACK_ACCRUAL_RATE);

        assert_eq!(
            thresholds,
            vec![3 * PACK_ACCRUAL_RATE, 6 * PACK_ACCRUAL_RATE]
        );
        assert_eq!(state.lifetime_packs_earned, 3);
        assert_eq!(state.pack_count(&Pack::Basic), 3);
    }
}
