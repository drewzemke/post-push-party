use super::{BonusTrack, PushContext, Reward, Tier};

/// bonus for pushing multiple times in quick succession
pub struct RapidFire;

/// time window in seconds for rapid fire bonus
const RAPID_FIRE_WINDOW_SECS: u64 = 15 * 60;

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

impl BonusTrack for RapidFire {
    fn id(&self) -> &'static str {
        "rapid_fire"
    }

    fn name(&self) -> &'static str {
        "Rapid Fire"
    }

    fn description(&self) -> &'static str {
        "Bonus for pushing twice within 15 minutes."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        if ctx.push.commits().is_empty() {
            return 0;
        }

        let cutoff = ctx.clock.now().saturating_sub(RAPID_FIRE_WINDOW_SECS);
        let has_recent_push = ctx.history.entries().iter().any(|e| e.timestamp() >= cutoff);

        if has_recent_push {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::Clock;
    use crate::git::{Commit, Push};
    use crate::history::{PushEntry, PushHistory};

    #[test]
    fn applies_when_pushed_within_window() {
        let bonus = RapidFire;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([PushEntry::at(1000)]);
        let clock = Clock::at(1000 + 5 * 60);

        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn applies_at_exact_boundary() {
        let bonus = RapidFire;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([PushEntry::at(1000)]);
        let clock = Clock::at(1000 + RAPID_FIRE_WINDOW_SECS);

        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_outside_window() {
        let bonus = RapidFire;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::from_entries([PushEntry::at(1000)]);
        let clock = Clock::at(1000 + RAPID_FIRE_WINDOW_SECS + 60);

        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_with_no_history() {
        let bonus = RapidFire;
        let push = Push::new(vec![Commit::default()]);
        let history = PushHistory::default();
        let clock = Clock::at(1000);

        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = RapidFire;
        let push = Push::new(vec![]);
        let history = PushHistory::from_entries([PushEntry::at(1000)]);
        let clock = Clock::at(1000 + 5 * 60);

        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }
}
