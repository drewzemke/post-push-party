use crate::{party, state};

pub fn cheat(amount: i64) {
    let mut state = state::load();
    let old = state.party_points;
    if amount < 0 {
        state.party_points = state.party_points.saturating_sub(amount.unsigned_abs());
    } else {
        state.party_points = state.party_points.saturating_add(amount as u64);
    }
    if let Err(e) = state::save(&state) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("{} → {} party points", old, state.party_points);
}

pub fn push(commits: u64) {
    let mut state = state::load();
    let points_earned = commits * state.points_per_commit();
    state.party_points += points_earned;
    if let Err(e) = state::save(&state) {
        eprintln!("warning: could not save state: {e}");
    }
    party::display(&state, commits, points_earned);
}

pub fn reset() {
    let state = state::State::default();
    if let Err(e) = state::save(&state) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("state reset to defaults");
}
