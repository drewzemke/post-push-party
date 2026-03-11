use crate::state::old_state_dir;

// FIXME: remove, only used for migration
pub fn old_log_path() -> Option<std::path::PathBuf> {
    old_state_dir().map(|d| d.join("debug.log"))
}
