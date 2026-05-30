pub struct EarlyPush;

use super::{BonusTrack, PushContext, Reward, Tier};

static TIERS: &[Tier] = &[
    Tier {
        cost: 50,
        reward: Reward::Multiplier(2),
    },
    Tier {
        cost: 500,
        reward: Reward::Multiplier(3),
    },
    Tier {
        cost: 3000,
        reward: Reward::Multiplier(4),
    },
    Tier {
        cost: 20000,
        reward: Reward::Multiplier(5),
    },
    Tier {
        cost: 120000,
        reward: Reward::Multiplier(6),
    },
];

impl BonusTrack for EarlyPush {
    fn id(&self) -> &'static str {
        "early_push"
    }

    fn name(&self) -> &'static str {
        "Early Bird"
    }

    fn description(&self) -> &'static str {
        "Multiplier for waking up and pushing code before 9am."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        const SIX_AM: i64 = 6 * 3600;
        const NINE_AM: i64 = 9 * 3600;

        let after_six = ctx.clock.local_seconds_since_midnight() >= SIX_AM;
        let before_nine = ctx.clock.local_seconds_since_midnight() <= NINE_AM;

        if after_six && before_nine && !ctx.push.commits().is_empty() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bonus_track::Clock,
        git::{Commit, Push},
        storage::{DbConnection, PushHistory},
    };

    const UTC_MINUS_8: i32 = -8 * 3600; // PST

    // day 20483 midnight UTC = 1769731200
    // day 20483 PST = 1769731200 + 28800 = 1769760000
    const MIDNIGHT_PST_AS_UTC: u64 = 1769760000;

    fn clock_at_hour(hour: u64) -> Clock {
        Clock::with_offset(MIDNIGHT_PST_AS_UTC + hour * 3600, UTC_MINUS_8)
    }

    #[test]
    fn applies_between_6_and_9am() {
        let conn = DbConnection::create_in_memory().unwrap();

        let bonus = EarlyPush;
        let history = PushHistory::new(&conn);

        // exactly 6am
        let push = Push::new(vec![Commit::default()]);
        let clock = clock_at_hour(6);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);

        // 7am
        let clock = clock_at_hour(7);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);

        // 9am
        let clock = clock_at_hour(9);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_outside_6_to_9am() {
        let conn = DbConnection::create_in_memory().unwrap();

        let bonus = EarlyPush;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::new(&conn);

        // 5am
        let clock = clock_at_hour(5);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);

        // 10am
        let clock = clock_at_hour(10);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let conn = DbConnection::create_in_memory().unwrap();

        let bonus = EarlyPush;
        let push = Push::new(vec![]);
        let history = PushHistory::new(&conn);
        let clock = clock_at_hour(8);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(bonus.applies(&ctx), 0);
    }
}
