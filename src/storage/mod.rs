use anyhow::{Result, anyhow};
use std::path::PathBuf;

mod connection;
mod migrations;
mod state;

pub use connection::DbConnection;

const APP_DIR_NAME: &str = "post-push-party";

fn storage_dir() -> Result<PathBuf> {
    // allow overriding with env var (mostly for e2e tests)
    if let Ok(dir) = std::env::var("PARTY_STATE_DIR") {
        return Ok(PathBuf::from(dir));
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
