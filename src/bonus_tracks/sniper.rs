use crate::history::PushHistory;

use super::{BonusTrack, Clock, Commit, Reward, Tier};

/// bonus for surgical single-line commits
pub struct Sniper;

static TIERS: &[Tier] = &[
    Tier { cost: 50, reward: Reward::FlatPoints(5) },
    Tier { cost: 200, reward: Reward::FlatPoints(10) },
    Tier { cost: 800, reward: Reward::FlatPoints(20) },
    Tier { cost: 3000, reward: Reward::FlatPoints(50) },
];

impl BonusTrack for Sniper {
    fn id(&self) -> &'static str {
        "one_line_change"
    }

    fn name(&self) -> &'static str {
        "Sniper"
    }

    fn description(&self) -> &'static str {
        "Bonus points for surgical single-line commits."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, commits: &[Commit], _history: &PushHistory, _clock: &Clock) -> u32 {
        commits.iter().filter(|c| c.lines_changed == 1).count() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(lines_changed: u64) -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed,
            timestamp: 0,
        }
    }

    fn empty_history() -> PushHistory {
        PushHistory { entries: vec![] }
    }

    fn clock() -> Clock {
        Clock { now: 0, tz_offset_secs: 0 }
    }

    #[test]
    fn has_correct_id() {
        assert_eq!(Sniper.id(), "one_line_change");
    }

    #[test]
    fn applies_to_single_line_commits() {
        let commits = vec![
            make_commit(1),
            make_commit(10),
            make_commit(1),
            make_commit(5),
        ];

        assert_eq!(Sniper.applies(&commits, &empty_history(), &clock()), 2);
    }

    #[test]
    fn does_not_apply_to_zero_line_commits() {
        let commits = vec![make_commit(0)];
        assert_eq!(Sniper.applies(&commits, &empty_history(), &clock()), 0);
    }

    #[test]
    fn does_not_apply_to_multi_line_commits() {
        let commits = vec![make_commit(2), make_commit(100)];
        assert_eq!(Sniper.applies(&commits, &empty_history(), &clock()), 0);
    }

    #[test]
    fn tiers_are_flat_points() {
        let tiers = Sniper.tiers();
        assert_eq!(tiers.len(), 4);
        assert_eq!(tiers[0].reward, Reward::FlatPoints(5));
        assert_eq!(tiers[3].reward, Reward::FlatPoints(50));
    }

    #[test]
    fn reward_at_level_returns_flat_points() {
        assert_eq!(Sniper.reward_at_level(0), None);
        assert_eq!(Sniper.reward_at_level(1), Some(Reward::FlatPoints(5)));
        assert_eq!(Sniper.reward_at_level(4), Some(Reward::FlatPoints(50)));
        assert_eq!(Sniper.reward_at_level(5), None);
    }
}
