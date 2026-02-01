use super::{BonusTrack, Clock, Commit, Reward, Tier};

/// bonus points for pushing on saturday or sunday
pub struct WeekendPush;

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

impl BonusTrack for WeekendPush {
    fn id(&self) -> &'static str {
        "weekend_push"
    }

    fn name(&self) -> &'static str {
        "Weekend Warrior"
    }

    fn description(&self) -> &'static str {
        "More points for pushing code on Saturday and Sunday."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(
        &self,
        commits: &[Commit],
        _history: &crate::history::PushHistory,
        clock: &Clock,
    ) -> u32 {
        let day_of_week = clock.day_of_week();
        if (day_of_week == 2 || day_of_week == 3) && !commits.is_empty() {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{PushEntry, PushHistory};

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

    const FRI_2AM_LOCAL: u64 = 1769842800;
    const SAT_2AM_LOCAL: u64 = 1769853600;
    const SUN_11PM_LOCAL: u64 = 1770015600;
    const MON_2AM_LOCAL: u64 = 1770026400;
    const UTC_MINUS_8: i32 = -8 * 3600; // PST

    #[test]
    fn applies_on_saturday_and_sunday() {
        // saturday
        let bonus = WeekendPush;
        let commits = vec![make_commit()];
        let history = make_history();

        let clock = Clock {
            now: SAT_2AM_LOCAL,
            tz_offset_secs: UTC_MINUS_8,
        };

        assert_eq!(bonus.applies(&commits, &history, &clock), 1);

        // sunday
        let bonus = WeekendPush;
        let commits = vec![make_commit()];
        let history = make_history();

        let clock = Clock {
            now: SUN_11PM_LOCAL,
            tz_offset_secs: UTC_MINUS_8,
        };

        assert_eq!(bonus.applies(&commits, &history, &clock), 1);
    }

    #[test]
    fn does_not_apply_on_friday_or_monday() {
        // friday
        let bonus = WeekendPush;
        let commits = vec![make_commit()];
        let history = make_history();

        let clock = Clock {
            now: FRI_2AM_LOCAL,
            tz_offset_secs: UTC_MINUS_8,
        };

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);

        // monday
        let bonus = WeekendPush;
        let commits = vec![make_commit()];
        let history = make_history();

        let clock = Clock {
            now: MON_2AM_LOCAL,
            tz_offset_secs: UTC_MINUS_8,
        };

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = WeekendPush;

        // no commits
        let commits = vec![];
        let history = make_history();

        let clock = Clock {
            now: SUN_11PM_LOCAL,
            tz_offset_secs: UTC_MINUS_8,
        };

        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }
}
