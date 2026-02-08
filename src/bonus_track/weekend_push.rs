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
        "Multiplier for pushing code on Saturday and Sunday."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        let day_of_week = ctx.clock.day_of_week();
        if (day_of_week == 2 || day_of_week == 3) && !ctx.push.commits().is_empty() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_track::Clock;
    use crate::git::{Commit, Push};
    use crate::history::PushHistory;

    const FRI_2AM_LOCAL: u64 = 1769842800;
    const SAT_2AM_LOCAL: u64 = 1769853600;
    const SUN_11PM_LOCAL: u64 = 1770015600;
    const MON_2AM_LOCAL: u64 = 1770026400;
    const UTC_MINUS_8: i32 = -8 * 3600; // PST

    #[test]
    fn applies_on_saturday_and_sunday() {
        let bonus = WeekendPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::default();

        // saturday
        let clock = Clock::with_offset(SAT_2AM_LOCAL, UTC_MINUS_8);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);

        // sunday
        let clock = Clock::with_offset(SUN_11PM_LOCAL, UTC_MINUS_8);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_on_friday_or_monday() {
        let bonus = WeekendPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::default();

        // friday
        let clock = Clock::with_offset(FRI_2AM_LOCAL, UTC_MINUS_8);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);

        // monday
        let clock = Clock::with_offset(MON_2AM_LOCAL, UTC_MINUS_8);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = WeekendPush;
        let push = Push::new(vec![]);
        let history = PushHistory::default();
        let clock = Clock::with_offset(SUN_11PM_LOCAL, UTC_MINUS_8);

        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }
}
