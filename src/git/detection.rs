use anyhow::Result;

use crate::{
    git::{self, Commit, Push},
    storage::{BranchRefsStore, PatchIdStore},
};

/// Snapshot current remote refs so future pushes are calculated correctly.
/// Called during init to avoid crediting pre-existing commits.
pub fn snapshot_refs(repo_path: &std::path::Path, branch_refs: &BranchRefsStore) -> Result<()> {
    // HACK: should we report an error here somehow?
    let Some(remote_url) = git::commands::get_remote_url(repo_path) else {
        return Ok(());
    };

    let current_refs = git::commands::get_all_remote_refs(repo_path);
    if current_refs.is_empty() {
        return Ok(());
    }

    for (branch, sha) in current_refs {
        branch_refs.update_ref(&remote_url, &branch, &sha)?;
    }

    Ok(())
}

/// Detect commits from recent push. Loads/saves refs and patch-id state as side effects.
pub fn get_pushed_commits(branch_refs: &BranchRefsStore, patch_ids: &PatchIdStore) -> Option<Push> {
    let repo_path = std::env::current_dir().expect("could not get current directory");

    let remote_url = git::commands::get_remote_url(&repo_path)?;
    crate::debug_log!("hook: remote_url = {}", remote_url);

    let current_refs = git::commands::get_all_remote_refs(&repo_path);
    crate::debug_log!("hook: current_refs = {:?}", current_refs);

    // collect commits from pushed branches
    let mut commits = Vec::new();
    let mut first_time_branches = Vec::new();
    let mut pushed_branches = Vec::new();

    for (branch, new_sha) in &current_refs {
        let local_sha = git::commands::get_local_ref(&repo_path, branch);
        if local_sha.as_ref() != Some(new_sha) {
            continue; // fetch, not push
        }
        let old_sha = branch_refs.get_ref(&remote_url, branch).ok()?;
        if old_sha.as_ref() == Some(new_sha) {
            continue; // no change
        }

        crate::debug_log!(
            "hook: branch {} pushed ({:?} -> {})",
            branch,
            old_sha,
            new_sha
        );
        pushed_branches.push(branch.clone());

        match old_sha {
            Some(old) => {
                // update: get exact range (fast)
                commits.extend(git::commands::list_commits_in_range(
                    &repo_path, &old, new_sha,
                ));
            }
            None => {
                // first-time push: need to process with other first-time branches
                first_time_branches.push(branch.as_str());
            }
        }
    }

    // first-time pushes processed together to handle shared history
    if !first_time_branches.is_empty() {
        commits.extend(git::commands::list_unique_commits(
            &repo_path,
            &first_time_branches,
        ));
    }

    if commits.is_empty() {
        for (branch, sha) in current_refs {
            branch_refs.update_ref(&remote_url, &branch, &sha).ok()?;
        }
        return None;
    }

    crate::debug_log!("hook: {} commits to check", commits.len());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut new_commits = Vec::new();

    // build list of branches to exclude from filtering (all branches we're pushing)
    let exclude_branches: Vec<&str> = pushed_branches.iter().map(|s| s.as_str()).collect();

    for sha in commits {
        // skip commits reachable from other remote branches (handles stale refs after jj fetch)
        if !exclude_branches.is_empty()
            && git::commands::is_reachable_from_other_remote(&repo_path, &sha, &exclude_branches)
        {
            crate::debug_log!("hook: skipping {} (reachable from other remote)", sha);
            continue;
        }

        if let Some(patch_id) = git::commands::get_patch_id(&repo_path, &sha)
            && !patch_ids.contains(&remote_url, &patch_id).ok()?
        {
            let lines_changed = git::commands::get_lines_changed(&repo_path, &sha).unwrap_or(0);
            crate::debug_log!(
                "hook: new commit {} ({}) - {} lines",
                sha,
                patch_id,
                lines_changed
            );
            patch_ids.record(&remote_url, &patch_id).ok()?;
            new_commits.push(Commit::new(sha, lines_changed, now));
        }
    }

    // update stored refs
    for (branch, sha) in current_refs {
        branch_refs.update_ref(&remote_url, &branch, &sha).ok()?;
    }

    crate::debug_log!("hook: {} new commits", new_commits.len());

    Some(Push::from_parts(
        new_commits,
        remote_url,
        pushed_branches.into_iter().next().unwrap_or_default(),
    ))
}
