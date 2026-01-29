mod first_push;

use crate::history::PushHistory;

/// data about a single commit in the current push
#[derive(Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub lines_changed: u64,
    pub timestamp: u64,
}

/// what a bonus awards when it applies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reward {
    Multiplier(f64),
    FlatPoints(u64),
}

/// cost to reach a tier and reward at that tier
#[derive(Debug, Clone, Copy)]
pub struct Tier {
    pub cost: u64,
    pub reward: Reward,
}

/// a bonus track that can be unlocked and upgraded
pub trait BonusTrack {
    /// display name for the UI
    fn name(&self) -> &'static str;

    /// description for the UI
    fn description(&self) -> &'static str;

    /// iterator over tiers (cost, reward). may be infinite.
    fn tiers(&self) -> Box<dyn Iterator<Item = Tier>>;

    /// how many times does this bonus apply to the current push?
    /// returns 0 if it doesn't apply, 1+ if it does.
    /// for multipliers, any non-zero count means the multiplier applies once.
    /// for flat bonuses, the count is multiplied by the flat amount.
    fn applies(&self, commits: &[Commit], history: &PushHistory, now: u64) -> u32;

    /// what reward does the user get at the given level?
    /// level 0 = not unlocked, level 1 = first tier, etc.
    fn reward_at_level(&self, level: u32) -> Option<Reward>;
}
