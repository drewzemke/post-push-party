use std::collections::HashSet;

use super::{BonusTrack, PushContext, Reward, Tier};

/// bonus for pushing to multiple repos in a day
pub struct MultipleRepos;

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

impl BonusTrack for MultipleRepos {
    fn id(&self) -> &'static str {
        "multiple_repos"
    }

    fn name(&self) -> &'static str {
        "Spread the Love"
    }

    fn description(&self) -> &'static str {
        "Bonus each time you push to a different repo today (after the first)."
    }

    fn tiers(&self) -> &'static [Tier] {
        TIERS
    }

    fn applies(&self, ctx: &PushContext) -> u32 {
        if ctx.push.commits.is_empty() {
            return 0;
        }

        let today = ctx.clock.today();
        let repos_pushed_today: HashSet<&str> = ctx
            .history
            .entries()
            .iter()
            .filter(|e| ctx.clock.day_of(e.timestamp) == today)
            .map(|e| e.remote_url.as_str())
            .collect();

        // bonus applies if:
        // 1. we've pushed to at least one repo today (so this isn't our first)
        // 2. the current repo is not one we've already pushed to today
        let is_new_repo = !repos_pushed_today.contains(ctx.push.remote_url.as_str());
        let has_pushed_before_today = !repos_pushed_today.is_empty();

        if is_new_repo && has_pushed_before_today {
            1
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::Clock;
    use crate::git::{Commit, Push};
    use crate::history::{PushEntry, PushHistory};

    fn make_commit() -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed: 10,
            timestamp: 0,
        }
    }

    fn make_push(commits: Vec<Commit>, repo: &str) -> Push {
        Push {
            commits,
            remote_url: repo.to_string(),
            branch: "main".to_string(),
        }
    }

    fn make_history(entries: Vec<PushEntry>) -> PushHistory {
        let mut history = PushHistory::default();
        for entry in entries {
            history.add(entry);
        }
        history
    }

    fn history_entry(timestamp: u64, repo: &str) -> PushEntry {
        PushEntry {
            timestamp,
            remote_url: repo.to_string(),
            branch: "main".to_string(),
            commits: 1,
        }
    }

    const SECONDS_PER_DAY: u64 = 86400;

    fn clock_at_day(day: u64) -> Clock {
        Clock {
            now: day * SECONDS_PER_DAY + 3600,
            tz_offset_secs: 0,
        }
    }

    fn timestamp_on_day(day: u64) -> u64 {
        day * SECONDS_PER_DAY + 3600
    }

    #[test]
    fn applies_on_second_repo() {
        let bonus = MultipleRepos;
        let clock = clock_at_day(100);

        // already pushed to repo1 today
        let history = make_history(vec![
            history_entry(timestamp_on_day(100), "git@github.com:user/repo1.git"),
        ]);

        // now pushing to repo2
        let push = make_push(vec![make_commit()], "git@github.com:user/repo2.git");
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn applies_on_third_repo() {
        let bonus = MultipleRepos;
        let clock = clock_at_day(100);

        let history = make_history(vec![
            history_entry(timestamp_on_day(100), "git@github.com:user/repo1.git"),
            history_entry(timestamp_on_day(100), "git@github.com:user/repo2.git"),
        ]);

        // now pushing to repo3
        let push = make_push(vec![make_commit()], "git@github.com:user/repo3.git");
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 1);
    }

    #[test]
    fn does_not_apply_on_first_repo() {
        let bonus = MultipleRepos;
        let clock = clock_at_day(100);

        // no pushes today yet
        let history = PushHistory::default();

        let push = make_push(vec![make_commit()], "git@github.com:user/repo1.git");
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_on_repeat_push_to_same_repo() {
        let bonus = MultipleRepos;
        let clock = clock_at_day(100);

        // already pushed to repo1 today
        let history = make_history(vec![
            history_entry(timestamp_on_day(100), "git@github.com:user/repo1.git"),
        ]);

        // pushing to repo1 again
        let push = make_push(vec![make_commit()], "git@github.com:user/repo1.git");
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_count_repos_from_other_days() {
        let bonus = MultipleRepos;
        let clock = clock_at_day(100);

        // pushed to repos yesterday, not today
        let history = make_history(vec![
            history_entry(timestamp_on_day(99), "git@github.com:user/repo1.git"),
            history_entry(timestamp_on_day(99), "git@github.com:user/repo2.git"),
        ]);

        // first push today
        let push = make_push(vec![make_commit()], "git@github.com:user/repo3.git");
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_empty_pushes() {
        let bonus = MultipleRepos;
        let clock = clock_at_day(100);

        let history = make_history(vec![
            history_entry(timestamp_on_day(100), "git@github.com:user/repo1.git"),
        ]);

        let push = make_push(vec![], "git@github.com:user/repo2.git");
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(bonus.applies(&ctx), 0);
    }
}
