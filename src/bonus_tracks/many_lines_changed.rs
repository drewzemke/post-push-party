use super::{BonusTrack, PushContext, Reward, Tier};

/// how many lines is considered "many"
const MANY_LINES_COUNT: u64 = 1_000;

/// bonus for commits with lots of lines changed
pub struct ManyLinesChanged;

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

impl BonusTrack for ManyLinesChanged {
    fn id(&self) -> &'static str {
        "many_lines_changed"
    }

    fn name(&self) -> &'static str {
        "Moby Diff"
    }

    fn description(&self) -> &'static str {
        "More points for big commits with at least 1,000 lines changed."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        ctx.push
            .commits()
            .iter()
            .filter(|c| c.lines_changed() >= MANY_LINES_COUNT)
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
    fn applies_to_multiple_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = Push::new(vec![
            Commit::with_lines(MANY_LINES_COUNT + 1),
            Commit::with_lines(10),
            Commit::with_lines(MANY_LINES_COUNT + 1),
            Commit::with_lines(5),
        ]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };

        assert_eq!(ManyLinesChanged.applies(&ctx), 2);
    }

    #[test]
    fn applies_to_big_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();

        let push = Push::new(vec![Commit::with_lines(MANY_LINES_COUNT)]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(ManyLinesChanged.applies(&ctx), 1);

        let push = Push::new(vec![Commit::with_lines(MANY_LINES_COUNT + 1)]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(ManyLinesChanged.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_to_small_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();

        let push = Push::new(vec![Commit::with_lines(0)]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(ManyLinesChanged.applies(&ctx), 0);

        let push = Push::new(vec![Commit::with_lines(MANY_LINES_COUNT - 1)]);
        let ctx = PushContext {
            push: &push,
            history: &history,
            clock: &clock,
        };
        assert_eq!(ManyLinesChanged.applies(&ctx), 0);
    }
}
