//! Tracks seen patch-ids per repo to detect "new" commits.

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

const MAX_PATCH_IDS: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchIdStore {
    // repo_url -> ordered list of patch-ids (newest first)
    repos: HashMap<String, VecDeque<String>>,
}

fn store_path() -> Option<std::path::PathBuf> {
    crate::state::state_dir().map(|d| d.join("patch_ids.bin"))
}

pub fn load() -> PatchIdStore {
    store_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}

pub fn save(store: &PatchIdStore) -> std::io::Result<()> {
    let path = store_path().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine state directory",
        )
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = bincode::serialize(store).map_err(std::io::Error::other)?;
    std::fs::write(path, encoded)
}

impl PatchIdStore {
    /// Check if a patch-id has been seen for this repo.
    #[cfg(test)]
    pub fn contains(&self, repo_url: &str, patch_id: &str) -> bool {
        self.repos
            .get(repo_url)
            .map(|ids| ids.iter().any(|id| id == patch_id))
            .unwrap_or(false)
    }

    /// Record new patch-ids for a repo. Maintains max size by dropping oldest.
    pub fn record(&mut self, repo_url: &str, new_ids: &[String]) {
        let ids = self.repos.entry(repo_url.to_string()).or_default();
        for id in new_ids {
            ids.push_front(id.clone());
        }
        // trim to max size
        while ids.len() > MAX_PATCH_IDS {
            ids.pop_back();
        }
    }

    /// Returns the set of patch-ids for a repo.
    pub fn get_set(&self, repo_url: &str) -> HashSet<String> {
        self.repos
            .get(repo_url)
            .map(|ids| ids.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_checks_patch_ids() {
        let mut store = PatchIdStore::default();
        let repo = "git@github.com:user/repo.git";

        assert!(!store.contains(repo, "abc123"));

        store.record(repo, &["abc123".to_string(), "def456".to_string()]);

        assert!(store.contains(repo, "abc123"));
        assert!(store.contains(repo, "def456"));
        assert!(!store.contains(repo, "xyz789"));
    }

    #[test]
    fn trims_to_max_size() {
        let mut store = PatchIdStore::default();
        let repo = "git@github.com:user/repo.git";

        // add more than MAX_PATCH_IDS
        let ids: Vec<String> = (0..MAX_PATCH_IDS + 100)
            .map(|i| format!("patch_{}", i))
            .collect();
        store.record(repo, &ids);

        assert_eq!(store.repos.get(repo).unwrap().len(), MAX_PATCH_IDS);
        // newest should be kept (patch_599, patch_598, ...)
        assert!(store.contains(repo, &format!("patch_{}", MAX_PATCH_IDS + 99)));
        // oldest should be dropped (patch_0, patch_1, ...)
        assert!(!store.contains(repo, "patch_0"));
    }

    #[test]
    fn roundtrips_through_bincode() {
        let mut store = PatchIdStore::default();
        store.record(
            "git@github.com:user/repo.git",
            &["abc".to_string(), "def".to_string()],
        );

        let encoded = bincode::serialize(&store).unwrap();
        let decoded: PatchIdStore = bincode::deserialize(&encoded).unwrap();

        assert!(decoded.contains("git@github.com:user/repo.git", "abc"));
        assert!(decoded.contains("git@github.com:user/repo.git", "def"));
    }
}
