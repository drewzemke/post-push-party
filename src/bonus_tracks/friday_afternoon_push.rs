/// bonus for deploying on friday afternoon (you daredevil)
pub struct FridayAfternoon;

use super::{BonusTrack, PushContext, Reward, Tier};

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

impl BonusTrack for FridayAfternoon {
    fn id(&self) -> &'static str {
        "friday_afternoon"
    }

    fn name(&self) -> &'static str {
        "Friday Afternoon Deploy"
    }

    fn description(&self) -> &'static str {
        "Bonus for pushing code on Friday after 3pm. Living dangerously."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        const FRIDAY: i64 = 1;
        const THREE_PM: i64 = 15 * 3600;

        let is_friday = ctx.clock.day_of_week() == FRIDAY;
        let is_afternoon = ctx.clock.local_seconds_since_midnight() >= THREE_PM;

        if is_friday && is_afternoon && !ctx.commits.is_empty() {
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

    const UTC_MINUS_8: i32 = -8 * 3600; // PST

    // day 20483 is a Friday (20483 % 7 = 1)
    // day 20483 midnight UTC = 1769731200
    // Friday midnight PST = Friday 8am UTC = 1769731200 + 28800 = 1769760000
    const FRIDAY_MIDNIGHT_PST_AS_UTC: u64 = 1769760000;

    fn ctx_at_hour(commits: &[Commit], hour: u64) -> PushContext<'_> {
        // leak to get 'static lifetime for test convenience
        let clock = Box::leak(Box::new(Clock {
            now: FRIDAY_MIDNIGHT_PST_AS_UTC + hour * 3600,
            tz_offset_secs: UTC_MINUS_8,
        }));
        let history = Box::leak(Box::new(PushHistory::default()));
        PushContext {
            commits,
            history,
            clock,
            repo: "git@github.com:user/repo.git",
        }
    }

    #[test]
    fn applies_on_friday_after_3pm() {
        let bonus = FridayAfternoon;
        let commits = vec![make_commit()];

        // exactly 3pm
        assert_eq!(bonus.applies(&ctx_at_hour(&commits, 15)), 1);
        // 4pm
        assert_eq!(bonus.applies(&ctx_at_hour(&commits, 16)), 1);
        // 11pm
        assert_eq!(bonus.applies(&ctx_at_hour(&commits, 23)), 1);
    }

    #[test]
    fn does_not_apply_before_3pm() {
        let bonus = FridayAfternoon;
        let commits = vec![make_commit()];

        // midnight
        assert_eq!(bonus.applies(&ctx_at_hour(&commits, 0)), 0);
        // 2pm (just before cutoff)
        assert_eq!(bonus.applies(&ctx_at_hour(&commits, 14)), 0);
    }

    #[test]
    fn does_not_apply_on_other_days() {
        let bonus = FridayAfternoon;
        let commits = vec![make_commit()];
        let history = PushHistory::default();

        // Saturday 4pm (day after)
        let saturday_4pm = Clock {
            now: FRIDAY_MIDNIGHT_PST_AS_UTC + 24 * 3600 + 16 * 3600,
            tz_offset_secs: UTC_MINUS_8,
        };
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &saturday_4pm,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);

        // Thursday 4pm (day before)
        let thursday_4pm = Clock {
            now: FRIDAY_MIDNIGHT_PST_AS_UTC - 24 * 3600 + 16 * 3600,
            tz_offset_secs: UTC_MINUS_8,
        };
        let ctx = PushContext {
            commits: &commits,
            history: &history,
            clock: &thursday_4pm,
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = FridayAfternoon;
        let commits = vec![];

        assert_eq!(bonus.applies(&ctx_at_hour(&commits, 16)), 0);
    }
}
