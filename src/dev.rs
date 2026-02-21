use crate::clock::Clock;
use crate::git::{Commit, Push};
use crate::party::RenderContext;
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

pub fn push(num_commits: u64, lines: Option<Vec<u64>>) {
    // mirror the actual hook flow as closely as possible
    let mut state = state::load();
    let mut history = history::load();
    let clock = Clock::from_now();

    // build fake commits with specified or default line counts
    let commits: Vec<Commit> = (0..num_commits)
        .map(|i| {
            let lines_changed = lines
                .as_ref()
                .map(|l| l[i as usize % l.len()])
                .unwrap_or(10); // default 10 lines per commit
            Commit::new(format!("fake{}", i), lines_changed, clock.now())
        })
        .collect();

    let push = Push::with_repo(commits, "dev://fake");

    let breakdown = scoring::calculate_points(&push, &state, &history, &clock);
    let packs_earned = state.earn_points(breakdown.total);

    if let Err(e) = state::save(&state) {
        eprintln!("warning: could not save state: {e}");
    }

    // record this push in history (like the real hook does)
    let lines_changed: u64 = push.commits().iter().map(|c| c.lines_changed()).sum();
    history = history::record(
        "dev://fake",
        "main",
        num_commits,
        lines_changed,
        breakdown.total,
    );

    let ctx = RenderContext::new(&push, &history, &breakdown, &state, &clock, packs_earned);
    party::display(&ctx);
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

pub fn bonus(track_id: &str, level: u32) {
    use crate::bonus_track::ALL_TRACKS;

    // verify track exists
    let track = ALL_TRACKS.iter().find(|t| t.id() == track_id);
    if track.is_none() {
        eprintln!("unknown track: {}", track_id);
        eprintln!(
            "available: {:?}",
            ALL_TRACKS.iter().map(|t| t.id()).collect::<Vec<_>>()
        );
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

pub fn palette(party_id: &str) {
    use crate::party::{ALL_PARTIES, palette::ALL_PALETTES};

    let ids: Vec<&str> = if party_id == "all" {
        ALL_PARTIES.iter().map(|p| p.id()).collect()
    } else {
        if !ALL_PARTIES.iter().any(|p| p.id() == party_id) {
            eprintln!("unknown party: {}", party_id);
            eprintln!(
                "available: {:?} (or \"all\")",
                ALL_PARTIES.iter().map(|p| p.id()).collect::<Vec<_>>()
            );
            std::process::exit(1);
        }
        vec![party_id]
    };

    let palette_names: Vec<String> = ALL_PALETTES.iter().map(|p| p.name().to_string()).collect();

    let mut s = state::load();
    for id in &ids {
        s.unlocked_palettes
            .insert(id.to_string(), palette_names.clone());
    }
    if let Err(e) = state::save(&s) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("unlocked all palettes for: {}", ids.join(", "));
}

pub fn party(party_id: &str) {
    use crate::party::ALL_PARTIES;

    // verify party exists
    let p = ALL_PARTIES.iter().find(|p| p.id() == party_id);
    if p.is_none() {
        eprintln!("unknown party: {}", party_id);
        eprintln!(
            "available: {:?}",
            ALL_PARTIES.iter().map(|p| p.id()).collect::<Vec<_>>()
        );
        std::process::exit(1);
    }

    let mut s = state::load();
    s.unlock_party(party_id);
    if let Err(e) = state::save(&s) {
        eprintln!("error saving state: {e}");
        std::process::exit(1);
    }
    println!("{} unlocked and enabled", party_id);
}
