//! Point calculation for pushes, applying all bonus tracks.

use crate::bonus_tracks::{Clock, Commit, Reward, ALL_TRACKS};
use crate::history::PushHistory;
use crate::state::State;

/// A bonus that was applied to this push.
#[derive(Debug, Clone)]
pub struct AppliedBonus {
    pub name: &'static str,
    pub multiplier: u32,
}

/// Breakdown of points earned for a push.
#[derive(Debug, Clone)]
pub struct PointsBreakdown {
    pub commits: u64,
    pub points_per_commit: u64,
    pub total_multiplier: u64,
    pub total: u64,
    pub applied: Vec<AppliedBonus>,
}

/// Calculate points earned for a push.
pub fn calculate_points(
    commits_counted: u64,
    state: &State,
    history: &PushHistory,
    clock: &Clock,
) -> PointsBreakdown {
    // FIXME: build fake commit data for now (we don't have real line counts yet)
    let commits: Vec<Commit> = (0..commits_counted)
        .map(|i| Commit {
            sha: format!("fake{}", i),
            lines_changed: 0,
            timestamp: clock.now,
        })
        .collect();

    let points_per_commit = state.points_per_commit();
    let base_points = commits_counted * points_per_commit;

    let mut total_multiplier: u64 = 1;
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

        if let Some(Reward::Multiplier(m)) = track.reward_at_level(level) {
            total_multiplier *= m as u64;
            applied.push(AppliedBonus {
                name: track.name(),
                multiplier: m,
            });
        }
    }

    PointsBreakdown {
        commits: commits_counted,
        points_per_commit,
        total_multiplier,
        total: base_points * total_multiplier,
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
