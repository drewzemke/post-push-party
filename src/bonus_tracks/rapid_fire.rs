use crate::history::PushHistory;

use super::{BonusTrack, Clock, Commit, Reward, Tier};

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

    fn applies(&self, commits: &[Commit], history: &PushHistory, clock: &Clock) -> u32 {
        if commits.is_empty() {
            return 0;
        }

        let cutoff = clock.now.saturating_sub(RAPID_FIRE_WINDOW_SECS);
        let has_recent_push = history.entries().iter().any(|e| e.timestamp >= cutoff);

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
    use crate::history::PushEntry;

    fn make_commit() -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed: 10,
            timestamp: 0,
        }
    }

    fn make_history(entries: Vec<PushEntry>) -> PushHistory {
        let mut history = PushHistory::default();
        for entry in entries {
            history.add(entry);
        }
        history
    }

    fn push_at(timestamp: u64) -> PushEntry {
        PushEntry {
            timestamp,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
            commits: 1,
        }
    }

    fn clock_at(now: u64) -> Clock {
        Clock {
            now,
            tz_offset_secs: 0,
        }
    }

    #[test]
    fn applies_when_pushed_within_window() {
        let bonus = RapidFire;
        let commits = vec![make_commit()];

        // pushed 5 minutes ago
        let history = make_history(vec![push_at(1000)]);
        let clock = clock_at(1000 + 5 * 60);

        assert_eq!(bonus.applies(&commits, &history, &clock), 1);
    }

    #[test]
    fn applies_at_exact_boundary() {
        let bonus = RapidFire;
        let commits = vec![make_commit()];

        // pushed exactly 15 minutes ago
        let history = make_history(vec![push_at(1000)]);
        let clock = clock_at(1000 + RAPID_FIRE_WINDOW_SECS);

        assert_eq!(bonus.applies(&commits, &history, &clock), 1);
    }

    #[test]
    fn does_not_apply_outside_window() {
        let bonus = RapidFire;
        let commits = vec![make_commit()];

        // pushed 16 minutes ago
        let history = make_history(vec![push_at(1000)]);
        let clock = clock_at(1000 + RAPID_FIRE_WINDOW_SECS + 60);

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }

    #[test]
    fn does_not_apply_with_no_history() {
        let bonus = RapidFire;
        let commits = vec![make_commit()];
        let history = PushHistory::default();
        let clock = clock_at(1000);

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = RapidFire;
        let commits = vec![];
        let history = make_history(vec![push_at(1000)]);
        let clock = clock_at(1000 + 5 * 60);

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }
}
