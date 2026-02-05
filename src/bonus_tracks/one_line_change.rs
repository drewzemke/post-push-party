use super::{BonusTrack, PushContext, Reward, Tier};

/// bonus for surgical single-line commits
pub struct OneLineChange;

static TIERS: &[Tier] = &[
    Tier {
        cost: 50,
        reward: Reward::FlatPoints(5),
    },
    Tier {
        cost: 200,
        reward: Reward::FlatPoints(10),
    },
    Tier {
        cost: 800,
        reward: Reward::FlatPoints(20),
    },
    Tier {
        cost: 3000,
        reward: Reward::FlatPoints(50),
    },
];

impl BonusTrack for OneLineChange {
    fn id(&self) -> &'static str {
        "one_line_change"
    }

    fn name(&self) -> &'static str {
        "Sniper"
    }

    fn description(&self) -> &'static str {
        "More points for surgical single-line commits."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        ctx.push
            .commits()
            .iter()
            .filter(|c| c.lines_changed() == 1)
            .count() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::Clock;
    use crate::git::{Commit, Push};
    use crate::history::PushHistory;

    #[test]
    fn applies_to_single_line_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = Push::new(vec![
            Commit::with_lines(1),
            Commit::with_lines(10),
            Commit::with_lines(1),
            Commit::with_lines(5),
        ]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };

        assert_eq!(OneLineChange.applies(&ctx), 2);
    }

    #[test]
    fn does_not_apply_to_zero_line_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = Push::new(vec![Commit::with_lines(0)]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(OneLineChange.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_multi_line_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = Push::new(vec![Commit::with_lines(2), Commit::with_lines(100)]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(OneLineChange.applies(&ctx), 0);
    }
}
