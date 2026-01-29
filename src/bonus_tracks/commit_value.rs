use crate::history::PushHistory;

use super::{BonusTrack, Clock, Commit, Reward, Tier};

/// base points earned per commit
pub struct CommitValue;

static TIERS: &[Tier] = &[
    Tier { cost: 0, reward: Reward::FlatPoints(1) },
    Tier { cost: 25, reward: Reward::FlatPoints(2) },
    Tier { cost: 100, reward: Reward::FlatPoints(3) },
    Tier { cost: 400, reward: Reward::FlatPoints(4) },
    Tier { cost: 1600, reward: Reward::FlatPoints(5) },
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

    fn applies(&self, _commits: &[Commit], _history: &PushHistory, _clock: &Clock) -> u32 {
        1 // always applies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_correct_id() {
        assert_eq!(CommitValue.id(), "commit_value");
    }

    #[test]
    fn tiers_start_free() {
        let tiers = CommitValue.tiers();
        assert_eq!(tiers[0].cost, 0);
        assert_eq!(tiers[0].reward, Reward::FlatPoints(1));
    }

    #[test]
    fn reward_at_level_returns_flat_points() {
        assert_eq!(CommitValue.reward_at_level(0), None);
        assert_eq!(CommitValue.reward_at_level(1), Some(Reward::FlatPoints(1)));
        assert_eq!(CommitValue.reward_at_level(2), Some(Reward::FlatPoints(2)));
        assert_eq!(CommitValue.reward_at_level(5), Some(Reward::FlatPoints(5)));
        assert_eq!(CommitValue.reward_at_level(6), None); // beyond max
    }

    #[test]
    fn always_applies() {
        let clock = Clock { now: 0, tz_offset_secs: 0 };
        let history = PushHistory::default();
        assert_eq!(CommitValue.applies(&[], &history, &clock), 1);
    }
}
