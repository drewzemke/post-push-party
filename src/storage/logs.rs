use anyhow::Result;
use std::{fs::OpenOptions, io::Write};

use crate::storage::storage_dir;

pub const LOG_FILE_NAME: &str = "party.log";

fn log_path() -> Result<std::path::PathBuf> {
    storage_dir().map(|d| d.join(LOG_FILE_NAME))
}

pub fn log(message: &str) {
    let Ok(path) = log_path() else { return };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };

    let _ = writeln!(file, "[{timestamp}] {message}");
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        $crate::storage::log(&format!($($arg)*))
    };
}
