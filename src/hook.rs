use crate::{git, history, party, scoring, state};

pub fn post_push() {
    if let Some(push) = git::get_pushed_commits() {
        let mut s = state::load();
        let history = history::load();
        let clock = scoring::now();

        let breakdown = scoring::calculate_points(&push.commits, &s, &history, &clock, &push.remote_url);
        s.party_points += breakdown.total;

        if let Err(e) = state::save(&s) {
            eprintln!("warning: could not save state: {e}");
        }

        // record push to history AFTER scoring so first_push_of_day
        // bonus can correctly detect if this is the first push today
        if !push.branch.is_empty() {
            history::record(&push.remote_url, &push.branch, push.commits.len() as u64);
        }

        party::display(&breakdown);
    }
}

/// stores ref start before pushing commits (only used in jj integration)
pub fn pre_push() {
    let cwd = std::env::current_dir().expect("could not get current directory");
    git::snapshot_refs(&cwd);
}
