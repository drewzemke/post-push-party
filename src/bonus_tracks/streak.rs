use std::collections::HashSet;

use crate::history::PushHistory;

use super::{BonusTrack, Clock, PushContext, Reward, Tier};

/// bonus for pushing consistently over multiple days
pub struct Streak;

/// count consecutive days with at least one push, ending today
fn consecutive_push_days(history: &PushHistory, clock: &Clock) -> u32 {
    let days_with_pushes: HashSet<i64> = history
        .entries()
        .iter()
        .map(|e| clock.day_of(e.timestamp()))
        .collect();

    let mut count = 0;
    let mut day = clock.today();

    while days_with_pushes.contains(&day) {
        count += 1;
        day -= 1;
    }

    count
}

/// minimum consecutive days to trigger the bonus
const MIN_STREAK_DAYS: u32 = 3;

static TIERS: &[Tier] = &[
    Tier {
        cost: 100,
        reward: Reward::Multiplier(2),
    },
    Tier {
        cost: 500,
        reward: Reward::Multiplier(3),
    },
    Tier {
        cost: 1500,
        reward: Reward::Multiplier(4),
    },
    Tier {
        cost: 5000,
        reward: Reward::Multiplier(5),
    },
    Tier {
        cost: 15000,
        reward: Reward::Multiplier(6),
    },
];

impl BonusTrack for Streak {
    fn id(&self) -> &'static str {
        "streak"
    }

    fn name(&self) -> &'static str {
        "Hot Streak"
    }

    fn description(&self) -> &'static str {
        "Multiplier for pushing 3+ days in a row."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        if ctx.push.commits().is_empty() {
            return 0;
        }

        if consecutive_push_days(ctx.history, ctx.clock) >= MIN_STREAK_DAYS {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{Commit, Push};
    use crate::history::PushEntry;

    const SECONDS_PER_DAY: u64 = 86400;

    fn clock_at_day(day: u64) -> Clock {
        Clock::at(day * SECONDS_PER_DAY + 3600) // 1am on that day
    }

    fn entry_on_day(day: u64) -> PushEntry {
        PushEntry::at(day * SECONDS_PER_DAY + 3600)
    }

    #[test]
    fn applies_with_3_day_streak() {
        let bonus = Streak;
        let push = Push::new(vec![Commit::default()]);
        let history =
            PushHistory::from_entries([entry_on_day(100), entry_on_day(101), entry_on_day(102)]);

        let clock = clock_at_day(102);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn applies_with_longer_streak() {
        let bonus = Streak;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries((95..=102).map(entry_on_day));

        let clock = clock_at_day(102);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_with_2_day_streak() {
        let bonus = Streak;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([entry_on_day(101), entry_on_day(102)]);

        let clock = clock_at_day(102);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_with_gap() {
        let bonus = Streak;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([
            entry_on_day(99),
            entry_on_day(100),
            // gap on day 101
            entry_on_day(102),
        ]);

        let clock = clock_at_day(102);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_if_no_push_today() {
        let bonus = Streak;
        let push = Push::new(vec![Commit::default()]);
        let history =
            PushHistory::from_entries([entry_on_day(99), entry_on_day(100), entry_on_day(101)]);

        // clock is on day 102, but no push today
        let clock = clock_at_day(102);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = Streak;
        let push = Push::new(vec![]);
        let history =
            PushHistory::from_entries([entry_on_day(100), entry_on_day(101), entry_on_day(102)]);

        let clock = clock_at_day(102);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }
}
