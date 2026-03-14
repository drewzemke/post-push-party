use anyhow::Result;

use crate::{
    clock::Clock,
    git,
    party::{self, RenderContext},
    scoring,
    state::State,
    storage::{BranchRefsStore, PatchIdStore, PushEntry, PushHistory},
};

pub fn post_push(
    state: &mut State,
    branch_refs: &BranchRefsStore,
    history: &PushHistory,
    patch_ids: &PatchIdStore,
) -> Result<()> {
    // HACK: should we do something else if this fails?
    let Some(push) = git::get_pushed_commits(branch_refs, patch_ids) else {
        return Ok(());
    };

    let clock = Clock::from_now();

    let breakdown = scoring::calculate_points(&push, state, history, &clock);
    let packs_earned = state.earn_points(breakdown.total);

    // record push to history AFTER scoring so first_push_of_day
    // bonus can correctly detect if this is the first push today.
    // only record if there are new commits - empty pushes (rebases) shouldn't
    // affect bonus track calculations like first_push_of_day
    if !push.branch().is_empty() && !push.commits().is_empty() {
        let lines_changed: u64 = push.commits().iter().map(|c| c.lines_changed()).sum();
        let entry = PushEntry::with_current_time(
            push.remote_url().to_string(),
            push.branch().to_string(),
            push.commits().len() as u64,
            lines_changed,
            breakdown.total,
        );
        history.record(&entry)?;
    }

    let ctx = RenderContext::new(&push, history, &breakdown, state, &clock, packs_earned);
    party::display(&ctx);

    Ok(())
}

/// stores ref start before pushing commits (only used in jj integration)
pub fn pre_push(branch_refs: &BranchRefsStore) -> Result<()> {
    let cwd = std::env::current_dir().expect("could not get current directory");
    git::snapshot_refs(&cwd, branch_refs)?;

    Ok(())
}
