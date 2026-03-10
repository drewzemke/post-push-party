use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatchIdStore {
    // repo_url -> ordered list of patch-ids (newest first)
    pub repos: HashMap<String, VecDeque<String>>,
}

fn store_path() -> Option<std::path::PathBuf> {
    crate::state::old_state_dir().map(|d| d.join("patch_ids.bin"))
}

// TODO: remove, only used for migration
pub fn load() -> PatchIdStore {
    store_path()
        .and_then(|p| std::fs::read(&p).ok())
        .and_then(|bytes| bincode::deserialize(&bytes).ok())
        .unwrap_or_default()
}
