//! Point calculation for pushes, applying all bonus tracks.

use crate::bonus_tracks::{Clock, Commit, Reward, ALL_TRACKS};
use crate::git::detection::CommitInfo;
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
    pub flat_bonus_total: u64,
    pub total_multiplier: u64,
    pub total: u64,
    pub applied: Vec<AppliedBonus>,
}

/// Calculate points earned for a push.
pub fn calculate_points(
    commit_info: &[CommitInfo],
    state: &State,
    history: &PushHistory,
    clock: &Clock,
) -> PointsBreakdown {
    // convert CommitInfo to bonus_tracks::Commit
    let commits: Vec<Commit> = commit_info
        .iter()
        .map(|c| Commit {
            sha: c.sha.clone(),
            lines_changed: c.lines_changed,
            timestamp: c.timestamp,
        })
        .collect();

    let points_per_commit = state.points_per_commit();
    let base_points = commits.len() as u64 * points_per_commit;

    let mut total_multiplier: u64 = 1;
    let mut flat_bonus_total: u64 = 0;
    let mut applied = Vec::new();

    for track in ALL_TRACKS.iter() {
        // skip commit_value, it's handled separately
        if track.id() == "commit_value" {
            continue;
        }

        let level = state.bonus_level(track.id());
        if level == 0 {
            continue;
        }

        let count = track.applies(&commits, history, clock);
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
        commits: commits.len() as u64,
        points_per_commit,
        flat_bonus_total,
        total_multiplier,
        total,
        applied,
    }
}

/// Create a Clock for the current moment.
pub fn now() -> Clock {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // get local timezone offset
    let tz_offset_secs = chrono::Local::now().offset().local_minus_utc();

    Clock {
        now,
        tz_offset_secs,
    }
}
