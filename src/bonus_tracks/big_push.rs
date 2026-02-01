use crate::history::PushHistory;

use super::{BonusTrack, Clock, Commit, Reward, Tier};

/// bonus for pushing a lot of commits at once
pub struct BigPush;

/// how many commits is considered "big"
const BIG_PUSH_COMMIT_COUNT: usize = 10;

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

impl BonusTrack for BigPush {
    fn id(&self) -> &'static str {
        "big_push"
    }

    fn name(&self) -> &'static str {
        "Big Push"
    }

    fn description(&self) -> &'static str {
        // NOTE: gotta keep this in sync with BIG_PUSH_COMMIT_COUNT above
        "More points if you push 10+ commits at once."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, commits: &[Commit], _history: &PushHistory, _clock: &Clock) -> u32 {
        if commits.len() >= BIG_PUSH_COMMIT_COUNT {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit() -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed: 10,
            timestamp: 0,
        }
    }

    fn make_history() -> PushHistory {
        PushHistory::default()
    }

    #[test]
    fn applies_to_big_pushes() {
        let bonus = BigPush;
        let history = make_history();
        let clock = Clock::default();

        let commits = vec![make_commit(); BIG_PUSH_COMMIT_COUNT];

        assert_eq!(bonus.applies(&commits, &history, &clock), 1);

        let commits = vec![make_commit(); 1_000];

        assert_eq!(bonus.applies(&commits, &history, &clock), 1);
    }

    #[test]
    fn does_not_apply_to_small_pushes() {
        let bonus = BigPush;
        let history = make_history();
        let clock = Clock::default();

        let commits = vec![make_commit(); BIG_PUSH_COMMIT_COUNT - 1];

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);

        let commits = vec![make_commit(); 1];

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }
}
