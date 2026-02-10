//! Point calculation for pushes, applying all bonus tracks.

use crate::bonus_track::{Clock, PushContext, Reward, ALL_TRACKS};
use crate::git::Push;
use crate::history::PushHistory;
use crate::state::State;

/// A bonus that was applied to this push.
#[derive(Debug, Clone)]
pub enum AppliedBonus {
    Multiplier {
        name: &'static str,
        value: u32,
    },
    FlatBonus {
        name: &'static str,
        points: u64,
        count: u32,
    },
}

/// Breakdown of points earned for a push.
#[derive(Debug, Clone)]
pub struct PointsBreakdown {
    pub commits: u64,
    pub points_per_commit: u64,
    pub total: u64,
    pub applied: Vec<AppliedBonus>,
}

/// Calculate points earned for a push.
pub fn calculate_points(
    push: &Push,
    state: &State,
    history: &PushHistory,
    clock: &Clock,
) -> PointsBreakdown {
    let points_per_commit = state.points_per_commit();
    let base_points = push.commits().len() as u64 * points_per_commit;

    let mut total_multiplier: u64 = 1;
    let mut flat_bonus_total: u64 = 0;
    let mut applied = Vec::new();

    let ctx = PushContext {
        push,
        history,
        clock,
    };

    for track in ALL_TRACKS.iter() {
        // skip commit_value, it's handled separately
        if track.id() == "commit_value" {
            continue;
        }

        let level = state.bonus_level(track.id());
        if level == 0 {
            continue;
        }

        let count = track.applies(&ctx);
        if count == 0 {
            continue;
        }

        match track.reward_at_level(level) {
            Some(Reward::Multiplier(m)) => {
                total_multiplier *= m as u64;
                applied.push(AppliedBonus::Multiplier {
                    name: track.name(),
                    value: m,
                });
            }
            Some(Reward::FlatPoints(pts)) => {
                let bonus = pts * count as u64;
                flat_bonus_total += bonus;
                applied.push(AppliedBonus::FlatBonus {
                    name: track.name(),
                    points: bonus,
                    count,
                });
            }
            None => {}
        }
    }

    // formula: final_points = (base_points + flat_bonus_total) * total_multiplier
    let total = (base_points + flat_bonus_total) * total_multiplier;

    PointsBreakdown {
        commits: push.commits().len() as u64,
        points_per_commit,
        total,
        applied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::Commit;

    fn get_multiplier(id: &str, level: u32) -> u64 {
        ALL_TRACKS
            .iter()
            .find(|t| t.id() == id)
            .and_then(|t| t.reward_at_level(level))
            .map(|r| match r {
                Reward::Multiplier(m) => m as u64,
                _ => 1,
            })
            .unwrap_or(1)
    }

    fn get_flat(id: &str, level: u32) -> u64 {
        ALL_TRACKS
            .iter()
            .find(|t| t.id() == id)
            .and_then(|t| t.reward_at_level(level))
            .map(|r| match r {
                Reward::FlatPoints(p) => p,
                _ => 0,
            })
            .unwrap_or(0)
    }

    #[test]
    fn base_points_without_bonuses() {
        let state = State::default();
        let push = Push::new(vec![
            Commit::with_lines(10),
            Commit::with_lines(20),
            Commit::with_lines(30),
        ]);

        let result = calculate_points(&push, &state, &PushHistory::default(), &Clock::at(1000));

        assert_eq!(result.total, 3); // 3 commits × 1 point
    }

    #[test]
    fn formula_applies_flat_before_multiplier() {
        let mut state = State::default();
        state.set_bonus_level("first_push", 1);
        state.set_bonus_level("one_line_change", 1);

        // 2 commits, 1 qualifies for sniper
        let push = Push::new(vec![Commit::with_lines(1), Commit::with_lines(10)]);

        let result = calculate_points(&push, &state, &PushHistory::default(), &Clock::at(1000));

        let mult = get_multiplier("first_push", 1);
        let flat = get_flat("one_line_change", 1);
        assert_eq!(result.total, (2 + flat) * mult);
    }

    #[test]
    fn flat_bonus_scales_with_qualifying_commits() {
        let mut state = State::default();
        state.set_bonus_level("one_line_change", 1);

        // 3 sniper commits, 1 non-sniper
        let push = Push::new(vec![
            Commit::with_lines(1),
            Commit::with_lines(1),
            Commit::with_lines(1),
            Commit::with_lines(50),
        ]);

        let result = calculate_points(&push, &state, &PushHistory::default(), &Clock::at(1000));

        let flat_per = get_flat("one_line_change", 1);
        // 4 base points + (3 sniper commits × flat_per)
        assert_eq!(result.total, 4 + 3 * flat_per);
    }
}
