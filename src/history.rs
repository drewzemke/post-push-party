use serde::{Deserialize, Serialize};

use crate::storage::PushEntry;

// TODO remove, used only for migration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PushHistory {
    entries: Vec<PushEntry>,
}

impl PushHistory {
    pub fn entries(&self) -> &[PushEntry] {
        &self.entries
    }
}

fn path() -> Option<std::path::PathBuf> {
    crate::state::old_state_dir().map(|d| d.join("history.json"))
}

// TODO: remove, used only for migration
pub fn load() -> PushHistory {
    path()
        .and_then(|p| std::fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
