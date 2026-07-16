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

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "monkeytype-tui-test-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn write_atomic_writes_content() {
        let path = scratch_path("write");
        write_atomic(&path, "hello");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_atomic_overwrites_on_second_call() {
        let path = scratch_path("overwrite");
        write_atomic(&path, "first");
        write_atomic(&path, "second");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        assert!(!path.with_extension("tmp").exists());
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_atomic_creates_missing_parent_dirs() {
        let dir = scratch_path("parent-dir");
        let path = dir.join("nested").join("file.json");
        write_atomic(&path, "content");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "content");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn data_dir_returns_some_path_under_normal_env() {
        assert!(data_dir().is_some());
    }
}
