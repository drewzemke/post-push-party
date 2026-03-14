use anyhow::Result;

use crate::{
    clock::Clock,
    git::{Commit, Push},
    party::{self, RenderContext},
    scoring,
    state::{self, State},
    storage::{PushEntry, PushHistory},
};

pub fn cheat(amount: i64, state: &mut State) {
    let old = state.party_points;
    if amount < 0 {
        state.party_points = state.party_points.saturating_sub(amount.unsigned_abs());
    } else {
        state.party_points = state.party_points.saturating_add(amount as u64);
    }
    println!("{} → {} party points", old, state.party_points);
}

pub fn push(
    num_commits: u64,
    lines: Option<Vec<u64>>,
    state: &mut State,
    history: &PushHistory,
) -> Result<()> {
    // mirror the actual hook flow as closely as possible
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

    let breakdown = scoring::calculate_points(&push, state, history, &clock);
    let packs_earned = state.earn_points(breakdown.total);

    // record this push in history (like the real hook does)
    let lines_changed: u64 = push.commits().iter().map(|c| c.lines_changed()).sum();
    let entry = PushEntry::with_current_time(
        "dev://fake".to_string(),
        "main".to_string(),
        num_commits,
        lines_changed,
        breakdown.total,
    );
    history.record(&entry)?;

    let ctx = RenderContext::new(&push, history, &breakdown, state, &clock, packs_earned);
    party::display(&ctx);

    Ok(())
}

pub fn reset(state: &mut State, pushes: &PushHistory) -> Result<()> {
    *state = state::State::default();

    pushes.reset()?;

    println!("state and history reset to defaults");
    Ok(())
}

pub fn bonus(track_id: &str, level: u32, state: &mut State) {
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

    state.set_bonus_level(track_id, level);
    println!("{} set to level {}", track_id, level);
}

pub fn palette(party_id: &str, state: &mut State) {
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

    for id in &ids {
        state
            .unlocked_palettes
            .insert(id.to_string(), palette_names.clone());
    }
    println!("unlocked all palettes for: {}", ids.join(", "));
}

pub fn party(party_id: &str, state: &mut State) {
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

    state.unlock_party(party_id);
    println!("{} unlocked and enabled", party_id);
}
