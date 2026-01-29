use crate::history::PushHistory;

use super::{BonusTrack, Commit, Reward, Tier};

/// bonus for the first push of each calendar day
pub struct FirstPush;

impl BonusTrack for FirstPush {
    fn name(&self) -> &'static str {
        "First Push of the Day"
    }

    fn description(&self) -> &'static str {
        "Earn bonus points on your first push each day."
    }

    fn tiers(&self) -> Box<dyn Iterator<Item = Tier>> {
        todo!()
    }

    fn applies(&self, _commits: &[Commit], _history: &PushHistory, _now: u64) -> u32 {
        todo!()
    }

    fn reward_at_level(&self, _level: u32) -> Option<Reward> {
        todo!()
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

    // timestamps for testing (2026-01-28)
    const TODAY_9AM: u64 = 1769594400;  // 2026-01-28 09:00 UTC
    const TODAY_3PM: u64 = 1769616000;  // 2026-01-28 15:00 UTC
    const YESTERDAY_9AM: u64 = 1769508000;  // 2026-01-27 09:00 UTC

    #[test]
    fn applies_when_no_pushes_today() {
        let bonus = FirstPush;
        let commits = vec![make_commit(TODAY_9AM)];
        let history = make_history(vec![
            make_push(YESTERDAY_9AM),
        ]);

        assert_eq!(bonus.applies(&commits, &history, TODAY_9AM), 1);
    }

    #[test]
    fn does_not_apply_when_already_pushed_today() {
        let bonus = FirstPush;
        let commits = vec![make_commit(TODAY_3PM)];
        let history = make_history(vec![
            make_push(TODAY_9AM),  // already pushed earlier today
        ]);

        assert_eq!(bonus.applies(&commits, &history, TODAY_3PM), 0);
    }

    #[test]
    fn applies_on_first_push_ever() {
        let bonus = FirstPush;
        let commits = vec![make_commit(TODAY_9AM)];
        let history = make_history(vec![]);

        assert_eq!(bonus.applies(&commits, &history, TODAY_9AM), 1);
    }
}
