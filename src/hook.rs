//! Hook entry point for detecting pushes and counting new commits.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::git;
use crate::patch_ids;

/// Tracks last-known SHA per branch per repo.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BranchRefs {
    repos: HashMap<String, HashMap<String, String>>,
}

fn refs_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("refs.bin"))
}

fn load_refs() -> BranchRefs {
    refs_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}

fn save_refs(refs: &BranchRefs) -> std::io::Result<()> {
    let path = refs_path()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no state directory"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(refs).map_err(std::io::Error::other)?;
    std::fs::write(path, encoded)
}

/// Snapshot current remote refs so future pushes are calculated correctly.
/// Called during init to avoid crediting pre-existing commits.
pub fn snapshot_refs(repo_path: &std::path::Path) {
    let Some(remote_url) = git::get_remote_url(repo_path) else {
        return;
    };

    let current_refs = git::get_all_remote_refs(repo_path);
    if current_refs.is_empty() {
        return;
    }

    let mut branch_refs = load_refs();
    let stored = branch_refs.repos.entry(remote_url).or_default();
    for (branch, sha) in current_refs {
        stored.insert(branch, sha);
    }
    let _ = save_refs(&branch_refs);
}

/// data about a single commit in a push
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub sha: String,
    pub lines_changed: u64,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct PushInfo {
    pub commits: Vec<CommitInfo>,
    pub commits_pushed: u64,
    pub commits_counted: u64,
    pub remote_url: String,
    pub branch: String,
}

pub fn run() -> Option<PushInfo> {
    let repo_path = std::env::current_dir().expect("could not get current directory");

    let remote_url = git::get_remote_url(&repo_path)?;
    crate::debug_log!("hook: remote_url = {}", remote_url);

    let current_refs = git::get_all_remote_refs(&repo_path);
    crate::debug_log!("hook: current_refs = {:?}", current_refs);

    let mut branch_refs = load_refs();
    let stored = branch_refs.repos.entry(remote_url.clone()).or_default();

    let mut patch_store = patch_ids::load();
    let mut seen = patch_store.get_set(&remote_url);

    // collect commits from pushed branches
    let mut commits = Vec::new();
    let mut first_time_branches = Vec::new();
    let mut pushed_branch = None;

    for (branch, new_sha) in &current_refs {
        let local_sha = git::get_local_ref(&repo_path, branch);
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
        if pushed_branch.is_none() {
            pushed_branch = Some(branch.clone());
        }

        match old_sha {
            Some(old) => {
                // update: get exact range (fast)
                commits.extend(git::list_commits_in_range(&repo_path, old, new_sha));
            }
            None => {
                // first-time push: need to process with other first-time branches
                first_time_branches.push(branch.as_str());
            }
        }
    }

    // first-time pushes processed together to handle shared history
    if !first_time_branches.is_empty() {
        commits.extend(git::list_unique_commits(&repo_path, &first_time_branches));
    }

    if commits.is_empty() {
        for (branch, sha) in &current_refs {
            stored.insert(branch.clone(), sha.clone());
        }
        let _ = save_refs(&branch_refs);
        return None;
    }

    let total_commits = commits.len();
    crate::debug_log!("hook: {} commits to check", total_commits);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut new_patch_ids = Vec::new();
    let mut new_commits = Vec::new();

    for sha in commits {
        if let Some(patch_id) = git::get_patch_id(&repo_path, &sha) {
            if !seen.contains(&patch_id) {
                let lines_changed = git::get_lines_changed(&repo_path, &sha).unwrap_or(0);
                crate::debug_log!("hook: new commit {} ({}) - {} lines", sha, patch_id, lines_changed);
                seen.insert(patch_id.clone());
                new_patch_ids.push(patch_id);
                new_commits.push(CommitInfo {
                    sha,
                    lines_changed,
                    timestamp: now,
                });
            }
        }
    }

    // update stored refs
    for (branch, sha) in &current_refs {
        stored.insert(branch.clone(), sha.clone());
    }
    let _ = save_refs(&branch_refs);

    let commits_counted = new_commits.len() as u64;

    // persist new patch-ids (if any)
    if !new_patch_ids.is_empty() {
        patch_store.record(&remote_url, &new_patch_ids);
        let _ = patch_ids::save(&patch_store);
    }

    crate::debug_log!("hook: {} new commits", commits_counted);

    // note: history::record() must be called by the caller AFTER scoring,
    // so that first_push_of_day bonus can see history without the current push
    Some(PushInfo {
        commits: new_commits,
        commits_pushed: total_commits as u64,
        commits_counted,
        remote_url,
        branch: pushed_branch.unwrap_or_default(),
    })
}
