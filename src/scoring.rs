//! Point calculation for pushes, applying all bonus tracks.

use crate::bonus_tracks::{Clock, Commit, Reward, ALL_TRACKS};
use crate::history::PushHistory;
use crate::state::State;

/// Calculate points earned for a push.
pub fn calculate_points(
    commits_counted: u64,
    state: &State,
    history: &PushHistory,
    clock: &Clock,
) -> u64 {
    // build fake commit data for now (we don't have real line counts yet)
    let commits: Vec<Commit> = (0..commits_counted)
        .map(|i| Commit {
            sha: format!("fake{}", i),
            lines_changed: 0,
            timestamp: clock.now,
        })
        .collect();

    // base points from commit value
    let base_points = commits_counted * state.points_per_commit();

    // calculate multiplier from all applicable bonus tracks
    let mut total_multiplier: u64 = 1;

    for track in ALL_TRACKS.iter() {
        let level = state.bonus_level(track.id());
        if level == 0 {
            continue; // not unlocked
        }

        let count = track.applies(&commits, history, clock);
        if count == 0 {
            continue; // doesn't apply to this push
        }

        if let Some(reward) = track.reward_at_level(level) {
            if let Reward::Multiplier(m) = reward {
                total_multiplier *= m as u64;
            }
            // FlatPoints bonuses would be added here when we have tracks that use them
            // (CommitValue is handled separately via points_per_commit)
        }
    }

    base_points * total_multiplier
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
