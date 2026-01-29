# Bonus Tracks Implementation

## Overview

Bonus tracks are upgradeable bonuses that trigger under specific conditions when pushing code. Each bonus:
1. Must be unlocked (tier 1 purchase)
2. Can be upgraded to increase its reward
3. Has a computation function to determine if it applies to a given push

## Formula

```
final_points = total_flat_bonuses * total_multiplier
```

Where:
- `total_flat_bonuses` = sum of all applicable flat bonuses (including Commit Value)
- `total_multiplier` = product of all applicable multipliers

For flat bonuses, the count from `applies()` is multiplied by the flat amount.
For multipliers, if count > 0, the multiplier applies once (multipliers don't stack with themselves).

## Core Types

### Commit

Data about a single commit in the current push, gathered by the hook.

```rust
pub struct Commit {
    pub sha: String,
    pub lines_changed: u64,
    pub timestamp: u64,  // unix timestamp
}
```

### Reward

What a bonus awards when it applies.

```rust
pub enum Reward {
    Multiplier(u32),  // whole number multipliers only (2x, 3x, etc.)
    FlatPoints(u64),
}
```

### Tier

Returned by the tiers iterator — cost to reach this tier and reward at this tier.

```rust
pub struct Tier {
    pub cost: u64,
    pub reward: Reward,
}
```

### Clock

Time context for bonus calculations, passed explicitly for testability.

```rust
pub struct Clock {
    pub now: u64,           // unix timestamp
    pub tz_offset_secs: i32, // positive = east of UTC, negative = west
}

impl Clock {
    pub fn day_of(&self, timestamp: u64) -> i64;  // local day number
    pub fn today(&self) -> i64;                    // day_of(self.now)
}
```

### BonusTrack Trait

```rust
pub trait BonusTrack {
    /// Display name for the UI
    fn name(&self) -> &'static str;

    /// Description for the UI
    fn description(&self) -> &'static str;

    /// Iterator over tiers (cost, reward). May be infinite.
    fn tiers(&self) -> impl Iterator<Item = Tier>;

    /// How many times does this bonus apply to the current push?
    /// Returns 0 if it doesn't apply, 1+ if it does.
    /// For multipliers, any non-zero count means the multiplier applies once.
    /// For flat bonuses, the count is multiplied by the flat amount.
    fn applies(&self, commits: &[Commit], history: &PushHistory, clock: &Clock) -> u32;

    /// What reward does the user get at the given level?
    /// Level 0 = not unlocked, level 1 = first tier, etc.
    fn reward_at_level(&self, level: u32) -> Option<Reward>;
}
```

## PushHistory Helpers

Add convenience methods to `PushHistory` for common queries:

```rust
impl PushHistory {
    /// All pushes from today (based on provided timestamp)
    fn pushes_today(&self, now: u64) -> impl Iterator<Item = &PushEntry>;

    /// All pushes in the last N hours
    fn pushes_in_last_hours(&self, now: u64, hours: u64) -> impl Iterator<Item = &PushEntry>;

    /// Distinct repos pushed to today
    fn distinct_repos_today(&self, now: u64) -> HashSet<&str>;

    /// Number of consecutive days with at least one push, ending today
    fn consecutive_push_days(&self, now: u64) -> u32;

    /// Whether there's been a push today before the current one
    fn has_pushed_today(&self, now: u64) -> bool;
}
```

## Git Module Additions

Add function to get lines changed for a commit:

```rust
/// Returns total lines added + removed for a commit
pub fn get_lines_changed(repo_path: &Path, sha: &str) -> Option<u64>;
```

Implementation: `git show --stat --format="" <sha>` and parse the summary line.

## Hook Changes

Enhance `hook::run()` to:
1. Build `Vec<Commit>` with sha, lines_changed, and timestamp for each new commit
2. Return this along with the counts (or replace `PushInfo` with richer struct)

```rust
pub struct PushResult {
    pub commits: Vec<Commit>,
    pub commits_pushed: u64,   // total in push (including dupes)
    pub commits_counted: u64,  // new commits (awarded points)
}
```

## Module Structure

```
src/bonus_tracks/
  mod.rs           # trait definition, Commit, Reward, Tier types
  commit_value.rs  # Base points per commit (users start at tier 1)
  first_push.rs    # First Push of the Day
  weekend.rs       # Weekend Warrior
  friday.rs        # Friday Afternoon Deploy
  big_push.rs      # Big Push (10+ commits)
  streak.rs        # 3+ consecutive days
  rapid_fire.rs    # Push twice within an hour
  spread_love.rs   # 3+ repos in a day
  loc_champion.rs  # Commit with 1000+ lines changed
  bug_sniper.rs    # Commit with exactly 1 line changed
```

Each file contains:
- A struct implementing `BonusTrack`
- Tier data (const array or generator)
- Unit tests

## Bonus Tracks Summary

| Bonus                   | Trigger                              | Reward     | Notes                 |
|-------------------------|--------------------------------------|------------|-----------------------|
| Commit Value            | Always (per commit)                  | Flat       | Users start at tier 1 |
| First Push of the Day   | First push each calendar day         | Multiplier |                       |
| Weekend Warrior         | Push on Saturday or Sunday           | Multiplier |                       |
| Friday Afternoon Deploy | Push on Friday after 3pm             | Multiplier |                       |
| Big Push                | 10+ commits in one push              | Multiplier |                       |
| Streak                  | 3+ consecutive days pushing          | Multiplier |                       |
| Rapid Fire              | Second+ push within an hour          | Multiplier |                       |
| Spread the Love         | 3+ different repos in a day          | Multiplier |                       |
| LoC Champion            | Any commit with 1000+ lines changed  | Flat       | Per-commit            |
| Bug Sniper              | Any commit with exactly 1 line       | Flat       | Per-commit            |

## Calculation Flow

1. Hook detects push, gathers `Vec<Commit>` with line counts and timestamps
2. Hook returns `PushResult` to main
3. Main loads `PushHistory` and `State`
4. For each bonus type where `state.bonuses.get(type) >= 1`:
   - Call `applies(commits, history, now)` → returns count
   - If count > 0, call `reward_at_level(user_level)`
   - For `FlatPoints(n)`: add `n * count` to flat_total
   - For `Multiplier(m)`: multiply multiplier_total by `m` (count ignored, applies once)
5. Calculate: `flat_total * multiplier_total`
6. Award points, run party, record to history

## Open Questions

- Exact tier costs and rewards for each bonus (balance tuning)
