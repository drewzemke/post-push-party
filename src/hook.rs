use crate::{
    clock::Clock,
    git, history,
    party::{self, RenderContext},
    scoring, state,
};

pub fn post_push() {
    if let Some(push) = git::get_pushed_commits() {
        let mut state = state::load();
        let mut history = history::load();
        let clock = Clock::from_now();

        let breakdown = scoring::calculate_points(&push, &state, &history, &clock);
        let packs_earned = state.earn_points(breakdown.total);

        if let Err(e) = state::save(&state) {
            eprintln!("warning: could not save state: {e}");
        }

        // record push to history AFTER scoring so first_push_of_day
        // bonus can correctly detect if this is the first push today.
        // only record if there are new commits - empty pushes (rebases) shouldn't
        // affect bonus track calculations like first_push_of_day
        if !push.branch().is_empty() && !push.commits().is_empty() {
            let lines_changed: u64 = push.commits().iter().map(|c| c.lines_changed()).sum();
            history = history::record(
                push.remote_url(),
                push.branch(),
                push.commits().len() as u64,
                lines_changed,
                breakdown.total,
            );
        }

        let ctx = RenderContext::new(&push, &history, &breakdown, &state, &clock, packs_earned);
        party::display(&ctx);
    }
}

/// stores ref start before pushing commits (only used in jj integration)
pub fn pre_push() {
    let cwd = std::env::current_dir().expect("could not get current directory");
    git::snapshot_refs(&cwd);
}
