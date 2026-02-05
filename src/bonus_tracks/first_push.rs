use super::{BonusTrack, PushContext, Reward, Tier};

/// bonus for the first push of each calendar day
pub struct FirstPush;

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

impl BonusTrack for FirstPush {
    fn id(&self) -> &'static str {
        "first_push"
    }

    fn name(&self) -> &'static str {
        "First Push of the Day"
    }

    fn description(&self) -> &'static str {
        "Multiplier for your first push each day."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        let pushed_today = ctx
            .history
            .entries()
            .iter()
            .any(|e| ctx.clock.day_of(e.timestamp()) == ctx.clock.today());

        if pushed_today {
            0
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::Clock;
    use crate::git::{Commit, Push};
    use crate::history::{PushEntry, PushHistory};

    // timestamps for testing (2026-01-28 in UTC)
    const TODAY_9AM: u64 = 1769594400; // 2026-01-28 09:00 UTC
    const TODAY_3PM: u64 = 1769616000; // 2026-01-28 15:00 UTC
    const YESTERDAY_9AM: u64 = 1769508000; // 2026-01-27 09:00 UTC

    // for timezone tests (UTC-5, e.g. EST)
    // local day 20480 starts at UTC 1769490000, so:
    const JAN28_9AM_LOCAL: u64 = 1769522400; // local 09:00 = UTC 14:00
    const JAN28_11PM_LOCAL: u64 = 1769572800; // local 23:00 = UTC 04:00 next day
    const UTC_MINUS_5: i32 = -5 * 3600;

    #[test]
    fn respects_local_timezone() {
        // push at 11pm local time - in UTC this is already Jan 29,
        // but in local time it's still Jan 28
        let bonus = FirstPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([PushEntry::at(JAN28_9AM_LOCAL)]);
        let clock = Clock::with_offset(JAN28_11PM_LOCAL, UTC_MINUS_5);

        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };

        // should NOT apply - already pushed today in local time
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn applies_when_no_pushes_today() {
        let bonus = FirstPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([PushEntry::at(YESTERDAY_9AM)]);
        let clock = Clock::at(TODAY_9AM);

        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };

        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_when_already_pushed_today() {
        let bonus = FirstPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([PushEntry::at(TODAY_9AM)]);
        let clock = Clock::at(TODAY_3PM);

        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };

        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn applies_on_first_push_ever() {
        let bonus = FirstPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::default();
        let clock = Clock::at(TODAY_9AM);

        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };

        assert_eq!(bonus.applies(&ctx), 1);
    }
}
