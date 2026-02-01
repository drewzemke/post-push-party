use crate::history::PushHistory;

use super::{BonusTrack, Clock, Commit, Reward, Tier};

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

    fn applies(&self, commits: &[Commit], _history: &PushHistory, _clock: &Clock) -> u32 {
        commits
            .iter()
            .filter(|c| c.lines_changed >= MANY_LINES_COUNT)
            .count() as u32
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
        PushHistory::default()
    }

    fn clock() -> Clock {
        Clock::default()
    }

    #[test]
    fn applies_to_multiple_commits() {
        let commits = vec![
            make_commit(MANY_LINES_COUNT + 1),
            make_commit(10),
            make_commit(MANY_LINES_COUNT + 1),
            make_commit(5),
        ];

        assert_eq!(
            ManyLinesChanged.applies(&commits, &empty_history(), &clock()),
            2
        );
    }

    #[test]
    fn applies_to_big_commits() {
        let commits = vec![make_commit(MANY_LINES_COUNT)];
        assert_eq!(
            ManyLinesChanged.applies(&commits, &empty_history(), &clock()),
            1
        );

        let commits = vec![make_commit(MANY_LINES_COUNT + 1)];
        assert_eq!(
            ManyLinesChanged.applies(&commits, &empty_history(), &clock()),
            1
        );
    }

    #[test]
    fn does_not_apply_to_small_commits() {
        let commits = vec![make_commit(0)];
        assert_eq!(
            ManyLinesChanged.applies(&commits, &empty_history(), &clock()),
            0
        );

        let commits = vec![make_commit(MANY_LINES_COUNT - 1)];
        assert_eq!(
            ManyLinesChanged.applies(&commits, &empty_history(), &clock()),
            0
        );
    }
}
