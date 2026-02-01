use crate::history::PushHistory;

use super::{BonusTrack, Clock, Commit, Reward, Tier};

/// bonus for the first push of each calendar day
pub struct FirstPush;

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

impl BonusTrack for FirstPush {
    fn id(&self) -> &'static str {
        "first_push"
    }

    fn name(&self) -> &'static str {
        "First Push of the Day"
    }

    fn description(&self) -> &'static str {
        "Earn bonus points on your first push each day."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, _commits: &[Commit], history: &PushHistory, clock: &Clock) -> u32 {
        let pushed_today = history
            .entries
            .iter()
            .any(|e| clock.day_of(e.timestamp) == clock.today());

        if pushed_today {
            0
        } else {
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::PushEntry;

    fn make_commit(timestamp: u64) -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed: 10,
            timestamp,
        }
    }

    fn make_history(entries: Vec<PushEntry>) -> PushHistory {
        PushHistory { entries }
    }

    fn make_push(timestamp: u64) -> PushEntry {
        PushEntry {
            timestamp,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
            commits: 1,
        }
    }

    fn utc(now: u64) -> Clock {
        Clock {
            now,
            tz_offset_secs: 0,
        }
    }

    // timestamps for testing (2026-01-28 in UTC)
    const TODAY_9AM: u64 = 1769594400; // 2026-01-28 09:00 UTC
    const TODAY_3PM: u64 = 1769616000; // 2026-01-28 15:00 UTC
    const YESTERDAY_9AM: u64 = 1769508000; // 2026-01-27 09:00 UTC

    // for timezone tests (UTC-5, e.g. EST)
    // local day 20480 starts at UTC 1769490000, so:
    const JAN28_9AM_LOCAL: u64 = 1769522400; // local 09:00 = UTC 14:00
    const JAN28_11PM_LOCAL: u64 = 1769572800; // local 23:00 = UTC 04:00 next day
    const UTC_MINUS_5: i32 = -5 * 3600;

    #[test]
    fn respects_local_timezone() {
        // push at 11pm local time - in UTC this is already Jan 29,
        // but in local time it's still Jan 28
        let bonus = FirstPush;
        let commits = vec![make_commit(JAN28_11PM_LOCAL)];

        // pushed at 9am local same day
        let history = make_history(vec![make_push(JAN28_9AM_LOCAL)]);

        let clock = Clock {
            now: JAN28_11PM_LOCAL,
            tz_offset_secs: UTC_MINUS_5,
        };

        // should NOT apply - already pushed today in local time
        assert_eq!(bonus.applies(&commits, &history, &clock), 0);
    }

    #[test]
    fn applies_when_no_pushes_today() {
        let bonus = FirstPush;
        let commits = vec![make_commit(TODAY_9AM)];
        let history = make_history(vec![make_push(YESTERDAY_9AM)]);

        assert_eq!(bonus.applies(&commits, &history, &utc(TODAY_9AM)), 1);
    }

    #[test]
    fn does_not_apply_when_already_pushed_today() {
        let bonus = FirstPush;
        let commits = vec![make_commit(TODAY_3PM)];

        // already pushed earlier today
        let history = make_history(vec![make_push(TODAY_9AM)]);

        assert_eq!(bonus.applies(&commits, &history, &utc(TODAY_3PM)), 0);
    }

    #[test]
    fn applies_on_first_push_ever() {
        let bonus = FirstPush;
        let commits = vec![make_commit(TODAY_9AM)];
        let history = make_history(vec![]);

        assert_eq!(bonus.applies(&commits, &history, &utc(TODAY_9AM)), 1);
    }
}
