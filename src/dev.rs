use crate::{history, party, scoring, state};

pub fn cheat(amount: i64) {
    let mut s = state::load();
    let old = s.party_points;
    if amount < 0 {
        s.party_points = s.party_points.saturating_sub(amount.unsigned_abs());
    } else {
        s.party_points = s.party_points.saturating_add(amount as u64);
    }
    if let Err(e) = state::save(&s) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("{} → {} party points", old, s.party_points);
}

pub fn push(commits: u64) {
    // mirror the actual hook flow as closely as possible
    let mut s = state::load();
    let hist = history::load();
    let clock = scoring::now();

    let breakdown = scoring::calculate_points(commits, &s, &hist, &clock);
    s.party_points += breakdown.total;

    if let Err(e) = state::save(&s) {
        eprintln!("warning: could not save state: {e}");
    }

    // record this push in history (like the real hook does)
    history::record("dev://fake", "main", commits);

    party::display(&breakdown);
}

pub fn reset() {
    let state = state::State::default();
    if let Err(e) = state::save(&state) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }

    let history = history::PushHistory::default();
    if let Err(e) = history::save(&history) {
        eprintln!("error saving history: {e}");
        std::process::exit(1);
    }

    println!("state and history reset to defaults");
}

pub fn unlock(track_id: &str, level: u32) {
    use crate::bonus_tracks::ALL_TRACKS;

    // verify track exists
    let track = ALL_TRACKS.iter().find(|t| t.id() == track_id);
    if track.is_none() {
        eprintln!("unknown track: {}", track_id);
        eprintln!("available: {:?}", ALL_TRACKS.iter().map(|t| t.id()).collect::<Vec<_>>());
        std::process::exit(1);
    }

    let mut s = state::load();
    s.set_bonus_level(track_id, level);
    if let Err(e) = state::save(&s) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("{} set to level {}", track_id, level);
}
