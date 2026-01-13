use std::fs::OpenOptions;
use std::io::Write;

use crate::state::state_dir;

pub fn log_path() -> Option<std::path::PathBuf> {
    state_dir().map(|d| d.join("debug.log"))
}

pub fn log(message: &str) {
    let Some(path) = log_path() else { return };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    else {
        return;
    };

    let _ = writeln!(file, "[{timestamp}] {message}");
}

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        $crate::log::log(&format!($($arg)*))
    };
}
