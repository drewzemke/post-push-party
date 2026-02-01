use super::{BonusTrack, PushContext, Reward, Tier};

/// bonus for surgical single-line commits
pub struct OneLineChange;

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

impl BonusTrack for OneLineChange {
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

    fn applies(&self, ctx: &PushContext) -> u32 {
        ctx.push.commits.iter().filter(|c| c.lines_changed == 1).count() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bonus_tracks::Clock;
    use crate::git::{Commit, Push};
    use crate::history::PushHistory;

    fn make_commit(lines_changed: u64) -> Commit {
        Commit {
            sha: "abc123".to_string(),
            lines_changed,
            timestamp: 0,
        }
    }

    fn make_push(commits: Vec<Commit>) -> Push {
        Push {
            commits,
            remote_url: "git@github.com:user/repo.git".to_string(),
            branch: "main".to_string(),
        }
    }

    #[test]
    fn applies_to_single_line_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = make_push(vec![
            make_commit(1),
            make_commit(10),
            make_commit(1),
            make_commit(5),
        ]);
        let ctx = PushContext { push: &push, history: &history, clock: &clock };

        assert_eq!(OneLineChange.applies(&ctx), 2);
    }

    #[test]
    fn does_not_apply_to_zero_line_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = make_push(vec![make_commit(0)]);
        let ctx = PushContext { push: &push, history: &history, clock: &clock };
        assert_eq!(OneLineChange.applies(&ctx), 0);
    }

    #[test]
    fn does_not_apply_to_multi_line_commits() {
        let history = PushHistory::default();
        let clock = Clock::default();
        let push = make_push(vec![make_commit(2), make_commit(100)]);
        let ctx = PushContext { push: &push, history: &history, clock: &clock };
        assert_eq!(OneLineChange.applies(&ctx), 0);
    }
}
