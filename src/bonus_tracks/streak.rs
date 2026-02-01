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
        .map(|e| clock.day_of(e.timestamp))
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
        "Bonus for pushing 3+ days in a row."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        if ctx.commits.is_empty() {
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
    use crate::bonus_tracks::Commit;
    use crate::history::PushEntry;

    fn make_commit() -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed: 10,
            timestamp: 0,
        }
    }

    const SECONDS_PER_DAY: u64 = 86400;

    fn clock_at_day(day: u64) -> Clock {
        Clock {
            now: day * SECONDS_PER_DAY + 3600, // 1am on that day
            tz_offset_secs: 0,
        }
    }

    fn push_on_day(day: u64) -> PushEntry {
        PushEntry {
            timestamp: day * SECONDS_PER_DAY + 3600,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
            commits: 1,
        }
    }

    #[test]
    fn applies_with_3_day_streak() {
        let bonus = Streak;
        let commits = vec![make_commit()];

        let mut history = PushHistory::default();
        history.add(push_on_day(100));
        history.add(push_on_day(101));
        history.add(push_on_day(102));

        let clock = clock_at_day(102);
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn applies_with_longer_streak() {
        let bonus = Streak;
        let commits = vec![make_commit()];

        let mut history = PushHistory::default();
        for day in 95..=102 {
            history.add(push_on_day(day));
        }

        let clock = clock_at_day(102);
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_with_2_day_streak() {
        let bonus = Streak;
        let commits = vec![make_commit()];

        let mut history = PushHistory::default();
        history.add(push_on_day(101));
        history.add(push_on_day(102));

        let clock = clock_at_day(102);
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_with_gap() {
        let bonus = Streak;
        let commits = vec![make_commit()];

        let mut history = PushHistory::default();
        history.add(push_on_day(99));
        history.add(push_on_day(100));
        // gap on day 101
        history.add(push_on_day(102));

        let clock = clock_at_day(102);
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_if_no_push_today() {
        let bonus = Streak;
        let commits = vec![make_commit()];

        let mut history = PushHistory::default();
        history.add(push_on_day(99));
        history.add(push_on_day(100));
        history.add(push_on_day(101));

        // clock is on day 102, but no push today
        let clock = clock_at_day(102);
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = Streak;
        let commits = vec![];

        let mut history = PushHistory::default();
        history.add(push_on_day(100));
        history.add(push_on_day(101));
        history.add(push_on_day(102));

        let clock = clock_at_day(102);
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }
}
