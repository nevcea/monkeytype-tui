//! Shared persistence helpers: locating the data directory and writing files
//! atomically. Used by both `history` and `pb`.

use std::path::{Path, PathBuf};

/// Resolve the app's data directory, respecting `XDG_DATA_HOME` and falling back
/// to `$HOME/.local/share` on unix or `%APPDATA%` on Windows. Returns `None`
/// when no base directory can be determined (persistence is then skipped).
pub fn data_dir() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(xdg).join("monkeytype-tui"));
    }
    if let Some(home) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(home).join(".local/share/monkeytype-tui"));
    }
    if let Some(appdata) = std::env::var_os("APPDATA").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(appdata).join("monkeytype-tui"));
    }
    None
}

/// Write `contents` to `path` atomically: write a sibling temp file then rename
/// over the target, so a crash mid-write can't truncate the existing file.
/// All errors are swallowed — persistence is best-effort.
pub fn write_atomic(path: &Path, contents: &str) {
    let Some(parent) = path.parent() else { return };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, contents).is_err() {
        return;
    }
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}
