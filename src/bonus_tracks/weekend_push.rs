use super::{BonusTrack, PushContext, Reward, Tier};

/// bonus points for pushing on saturday or sunday
pub struct WeekendPush;

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

impl BonusTrack for WeekendPush {
    fn id(&self) -> &'static str {
        "weekend_push"
    }

    fn name(&self) -> &'static str {
        "Weekend Warrior"
    }

    fn description(&self) -> &'static str {
        "More points for pushing code on Saturday and Sunday."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        let day_of_week = ctx.clock.day_of_week();
        if (day_of_week == 2 || day_of_week == 3) && !ctx.commits.is_empty() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::{Clock, Commit};
    use crate::history::PushHistory;

    fn make_commit() -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed: 10,
            timestamp: 0,
        }
    }

    const FRI_2AM_LOCAL: u64 = 1769842800;
    const SAT_2AM_LOCAL: u64 = 1769853600;
    const SUN_11PM_LOCAL: u64 = 1770015600;
    const MON_2AM_LOCAL: u64 = 1770026400;
    const UTC_MINUS_8: i32 = -8 * 3600; // PST

    #[test]
    fn applies_on_saturday_and_sunday() {
        let bonus = WeekendPush;
        let commits = vec![make_commit()];
        let history = PushHistory::default();

        // saturday
        let clock = Clock { now: SAT_2AM_LOCAL, tz_offset_secs: UTC_MINUS_8 };
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 1);

        // sunday
        let clock = Clock { now: SUN_11PM_LOCAL, tz_offset_secs: UTC_MINUS_8 };
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_on_friday_or_monday() {
        let bonus = WeekendPush;
        let commits = vec![make_commit()];
        let history = PushHistory::default();

        // friday
        let clock = Clock { now: FRI_2AM_LOCAL, tz_offset_secs: UTC_MINUS_8 };
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);

        // monday
        let clock = Clock { now: MON_2AM_LOCAL, tz_offset_secs: UTC_MINUS_8 };
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
        let bonus = WeekendPush;
        let commits = vec![];
        let history = PushHistory::default();
        let clock = Clock { now: SUN_11PM_LOCAL, tz_offset_secs: UTC_MINUS_8 };

        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &clock,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }
}
