use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::git;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RepoRefs {
    repos: HashMap<String, String>, // remote_url -> last_known_sha
}

fn refs_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("refs.bin"))
}

fn load_refs() -> RepoRefs {
    refs_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}

fn save_refs(refs: &RepoRefs) -> std::io::Result<()> {
    let path = refs_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(refs).map_err(std::io::Error::other)?;
    std::fs::write(path, encoded)
}

/// Result of detecting a push - just the commit count.
/// Points calculation happens in the caller.
#[derive(Debug)]
pub struct PushInfo {
    pub commits: u64,
}

/// Entry point called by git hook or jj alias.
/// Detects if a push occurred and returns commit count if so.
pub fn run() -> Option<PushInfo> {
    let repo_path = std::env::current_dir().expect("could not get current directory");
    run_in(&repo_path)
}

fn run_in(repo_path: &Path) -> Option<PushInfo> {
    crate::debug_log!("hook: run called for {:?}", repo_path);

    let remote_url = match git::get_remote_url(repo_path) {
        Some(url) => url,
        None => {
            crate::debug_log!("hook: no remote url found");
            return None;
        }
    };
    crate::debug_log!("hook: remote_url = {}", remote_url);

    let branch = match git::get_trunk_branch(repo_path) {
        Some(b) => b,
        None => {
            crate::debug_log!("hook: no trunk branch found");
            return None;
        }
    };
    crate::debug_log!("hook: branch = {}", branch);

    let current_ref = match git::get_remote_ref(repo_path, &branch) {
        Some(r) => r,
        None => {
            crate::debug_log!("hook: no remote ref found for branch {}", branch);
            return None;
        }
    };
    crate::debug_log!("hook: current_ref = {}", current_ref);

    // if local doesn't match remote, this was a fetch, not a push
    let local_ref = git::get_local_ref(repo_path, &branch);
    crate::debug_log!("hook: local_ref = {:?}", local_ref);
    if local_ref.as_ref() != Some(&current_ref) {
        crate::debug_log!("hook: local ref doesn't match remote, not a push");
        return None;
    }

    let mut refs = load_refs();
    let last_ref = refs.repos.get(&remote_url).cloned();
    crate::debug_log!("hook: last_ref = {:?}", last_ref);

    // if same as before, no push happened
    if last_ref.as_ref() == Some(&current_ref) {
        crate::debug_log!("hook: ref unchanged, no push detected");
        return None;
    }

    let commits = match &last_ref {
        Some(old_sha) => git::count_commits(repo_path, old_sha, &current_ref).unwrap_or(1),
        None => 1, // first time seeing this repo
    };

    // update refs
    refs.repos.insert(remote_url.clone(), current_ref);
    let _ = save_refs(&refs);

    // record in history
    crate::history::record(&remote_url, &branch, commits);

    crate::debug_log!("hook: push detected! {} commits", commits);

    Some(PushInfo { commits })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_refs_roundtrips() {
        let mut refs = RepoRefs::default();
        refs.repos.insert(
            "git@github.com:user/repo.git".to_string(),
            "abc123".to_string(),
        );

        let encoded = bincode::serialize(&refs).unwrap();
        let decoded: RepoRefs = bincode::deserialize(&encoded).unwrap();

        assert_eq!(
            decoded.repos.get("git@github.com:user/repo.git"),
            Some(&"abc123".to_string())
        );
    }

    #[test]
    fn empty_refs_deserializes() {
        let refs = RepoRefs::default();
        assert!(refs.repos.is_empty());
    }
}
