use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    git::{self, Commit, Push},
    storage::BranchRefsStore,
};

// TODO: remove
/// Tracks last-known SHA per branch per repo.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchRefs {
    pub repos: HashMap<String, HashMap<String, String>>,
}

fn refs_path() -> Option<std::path::PathBuf> {
    crate::state::old_state_dir().map(|d| d.join("refs.bin"))
}

// TODO: remove, used only for migration
pub fn load_old_refs() -> BranchRefs {
    refs_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}

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
pub fn get_pushed_commits(branch_refs: &BranchRefsStore) -> Option<Push> {
    let repo_path = std::env::current_dir().expect("could not get current directory");

    let remote_url = git::commands::get_remote_url(&repo_path)?;
    crate::debug_log!("hook: remote_url = {}", remote_url);

    let current_refs = git::commands::get_all_remote_refs(&repo_path);
    crate::debug_log!("hook: current_refs = {:?}", current_refs);

    let stored = branch_refs.get_refs_for_repo(&remote_url).ok()?;

    let mut patch_store = super::patch_ids::load();
    let mut seen = patch_store.get_set(&remote_url);

    // collect commits from pushed branches
    let mut commits = Vec::new();
    let mut first_time_branches = Vec::new();
    let mut pushed_branches = Vec::new();

    for (branch, new_sha) in &current_refs {
        let local_sha = git::commands::get_local_ref(&repo_path, branch);
        if local_sha.as_ref() != Some(new_sha) {
            continue; // fetch, not push
        }
        let old_sha = stored.get(branch);
        if old_sha == Some(new_sha) {
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
                    &repo_path, old, new_sha,
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

    let mut new_patch_ids = Vec::new();
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
            && !seen.contains(&patch_id)
        {
            let lines_changed = git::commands::get_lines_changed(&repo_path, &sha).unwrap_or(0);
            crate::debug_log!(
                "hook: new commit {} ({}) - {} lines",
                sha,
                patch_id,
                lines_changed
            );
            seen.insert(patch_id.clone());
            new_patch_ids.push(patch_id);
            new_commits.push(Commit::new(sha, lines_changed, now));
        }
    }

    // update stored refs
    for (branch, sha) in current_refs {
        branch_refs.update_ref(&remote_url, &branch, &sha).ok()?;
    }

    // persist new patch-ids (if any)
    if !new_patch_ids.is_empty() {
        patch_store.record(&remote_url, &new_patch_ids);
        let _ = super::patch_ids::save(&patch_store);
    }

    crate::debug_log!("hook: {} new commits", new_commits.len());

    Some(Push::from_parts(
        new_commits,
        remote_url,
        pushed_branches.into_iter().next().unwrap_or_default(),
    ))
}
