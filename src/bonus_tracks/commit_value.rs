use super::{BonusTrack, PushContext, Reward, Tier};

/// base points earned per commit
pub struct CommitValue;

static TIERS: &[Tier] = &[
    Tier {
        cost: 0,
        reward: Reward::FlatPoints(1),
    },
    Tier {
        cost: 25,
        reward: Reward::FlatPoints(2),
    },
    Tier {
        cost: 100,
        reward: Reward::FlatPoints(3),
    },
    Tier {
        cost: 400,
        reward: Reward::FlatPoints(4),
    },
    Tier {
        cost: 1600,
        reward: Reward::FlatPoints(5),
    },
];

impl BonusTrack for CommitValue {
    fn id(&self) -> &'static str {
        "commit_value"
    }

    fn name(&self) -> &'static str {
        "Commit Value"
    }

    fn description(&self) -> &'static str {
        "How many party points you earn per commit."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, _ctx: &PushContext) -> u32 {
        1 // always applies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::Clock;
    use crate::history::PushHistory;

    #[test]
    fn tiers_start_free() {
        let tiers = CommitValue.tiers();
        assert_eq!(tiers[0].cost, 0);
        assert_eq!(tiers[0].reward, Reward::FlatPoints(1));
    }

    #[test]
    fn always_applies() {
        let ctx = PushContext {
            commits: &[],
            history: &PushHistory::default(),
            clock: &Clock::default(),
            repo: "git@github.com:user/repo.git",
        };
        assert_eq!(CommitValue.applies(&ctx), 1);
    }
}
