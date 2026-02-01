mod commit_value;
mod first_push;
mod one_line_change;

pub use commit_value::CommitValue;
pub use first_push::FirstPush;
pub use one_line_change::OneLineChange;

use crate::history::PushHistory;

/// time context for bonus calculations
#[derive(Debug, Default, Clone, Copy)]
pub struct Clock {
    pub now: u64,
    pub tz_offset_secs: i32,
}

impl Clock {
    /// convert a utc timestamp to local day number
    pub fn day_of(&self, timestamp: u64) -> i64 {
        const SECONDS_PER_DAY: i64 = 86400;
        (timestamp as i64 + self.tz_offset_secs as i64) / SECONDS_PER_DAY
    }

    /// local day number for `now`
    pub fn today(&self) -> i64 {
        self.day_of(self.now)
    }
}

/// data about a single commit in the current push
#[derive(Debug, Clone)]
pub struct Commit {
    pub lines_changed: u64,

    #[expect(dead_code)]
    pub sha: String,
    #[expect(dead_code)]
    pub timestamp: u64,
}

/// what a bonus awards when it applies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reward {
    Multiplier(u32),
    FlatPoints(u64),
}

/// cost to reach a tier and reward at that tier
#[derive(Debug, Clone, Copy)]
pub struct Tier {
    pub cost: u64,
    pub reward: Reward,
}

/// a bonus track that can be unlocked and upgraded
pub trait BonusTrack: Sync {
    /// unique identifier for state storage
    fn id(&self) -> &'static str;

    /// display name for the UI
    fn name(&self) -> &'static str;

    /// description for the UI
    fn description(&self) -> &'static str;

    /// all tiers for this track
    fn tiers(&self) -> &'static [Tier];

    /// how many times does this bonus apply to the current push?
    /// returns 0 if it doesn't apply, 1+ if it does.
    /// for multipliers, any non-zero count means the multiplier applies once.
    /// for flat bonuses, the count is multiplied by the flat amount.
    fn applies(&self, commits: &[Commit], history: &PushHistory, clock: &Clock) -> u32;

    /// what reward does the user get at the given level?
    /// level 0 = not unlocked, level 1 = first tier, etc.
    fn reward_at_level(&self, level: u32) -> Option<Reward> {
        if level == 0 {
            return None;
        }
        self.tiers().get(level as usize - 1).map(|t| t.reward)
    }
}

// static instances for ALL_TRACKS
static COMMIT_VALUE: CommitValue = CommitValue;
static FIRST_PUSH: FirstPush = FirstPush;
static ONE_LINE_CHANGE: OneLineChange = OneLineChange;

/// all bonus tracks in display order
pub static ALL_TRACKS: &[&'static dyn BonusTrack] = &[&COMMIT_VALUE, &FIRST_PUSH, &ONE_LINE_CHANGE];
