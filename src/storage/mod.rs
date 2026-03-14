use anyhow::{Result, anyhow};
use std::path::PathBuf;

mod branch_refs;
mod connection;
mod logs;
mod migrations;
mod patch_ids;
mod pushes;
mod state;

pub use branch_refs::BranchRefsStore;
pub use connection::DbConnection;
pub use logs::log;
pub use patch_ids::PatchIdStore;
pub use pushes::PushHistory;

#[cfg(test)]
pub use pushes::PushEntry;

const APP_DIR_NAME: &str = "post-push-party";

fn storage_dir() -> Result<PathBuf> {
    // allow overriding with env var (mostly for e2e tests)
    if let Ok(dir) = std::env::var("PARTY_STATE_DIR") {
        let path = PathBuf::from(dir);
        std::fs::create_dir_all(&path)?;
        return Ok(path);
    }

    let mut path = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join(".local").join("share"))
    } else {
        dirs::data_local_dir()
    }
    .ok_or_else(|| anyhow!("failed to find os local data dir."))?;

    path.push(APP_DIR_NAME);

    std::fs::create_dir_all(&path)?;
    Ok(path)
}
