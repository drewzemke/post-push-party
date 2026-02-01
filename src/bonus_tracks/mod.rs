mod big_push;
mod clock;
mod commit_value;
mod context;
mod first_push;
mod friday_afternoon_push;
mod many_lines_changed;
mod multiple_repos;
mod one_line_change;
mod rapid_fire;
mod streak;
mod weekend_push;

pub use clock::Clock;
pub use context::PushContext;

use big_push::BigPush;
use commit_value::CommitValue;
use first_push::FirstPush;
use friday_afternoon_push::FridayAfternoon;
use many_lines_changed::ManyLinesChanged;
use multiple_repos::MultipleRepos;
use one_line_change::OneLineChange;
use rapid_fire::RapidFire;
use streak::Streak;
use weekend_push::WeekendPush;

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
    fn applies(&self, ctx: &PushContext) -> u32;

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
static BIG_PUSH: BigPush = BigPush;
static COMMIT_VALUE: CommitValue = CommitValue;
static FIRST_PUSH: FirstPush = FirstPush;
static FRIDAY_AFTERNOON: FridayAfternoon = FridayAfternoon;
static MANY_LINES_CHANGED: ManyLinesChanged = ManyLinesChanged;
static MULTIPLE_REPOS: MultipleRepos = MultipleRepos;
static ONE_LINE_CHANGE: OneLineChange = OneLineChange;
static RAPID_FIRE: RapidFire = RapidFire;
static STREAK: Streak = Streak;
static WEEKEND_PUSH: WeekendPush = WeekendPush;

/// all bonus tracks in display order
pub static ALL_TRACKS: &[&'static dyn BonusTrack] = &[
    &BIG_PUSH,
    &COMMIT_VALUE,
    &FIRST_PUSH,
    &FRIDAY_AFTERNOON,
    &MANY_LINES_CHANGED,
    &MULTIPLE_REPOS,
    &ONE_LINE_CHANGE,
    &RAPID_FIRE,
    &STREAK,
    &WEEKEND_PUSH,
];
