//! Shared persistence helpers: locating the data directory and writing files
//! atomically. Used by both `history` and `pb`.

use std::path::{Path, PathBuf};

/// Resolve the app's data directory, respecting `XDG_DATA_HOME` and falling back
/// to `$HOME/.local/share` on unix or `%APPDATA%` on Windows. Returns `None`
/// when no base directory can be determined (persistence is then skipped).
pub fn data_dir() -> Option<PathBuf> {
    // Unit tests build `App`, and nearly every state change it models calls
    // `persist()`. Pointed at the real data dir they read and then overwrite
    // the user's own config, history and personal bests — a `cargo test` run
    // reset saved preferences, and values written by one test leaked into the
    // next run. Redirect the suite to a scratch dir, keyed by pid so runs
    // start clean and concurrent runs don't collide.
    if cfg!(test) {
        return Some(
            std::env::temp_dir().join(format!("monkeytype-tui-test-{}", std::process::id())),
        );
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(xdg).join("monkeytype-tui"));
    }
    // APPDATA must win over HOME on Windows: Git Bash / MSYS set HOME there
    // too, so preferring HOME would split one machine's data between
    // `$HOME/.local/share` and `%APPDATA%` depending on which shell launched
    // the app.
    #[cfg(windows)]
    if let Some(appdata) = std::env::var_os("APPDATA").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(appdata).join("monkeytype-tui"));
    }
    if let Some(home) = std::env::var_os("HOME").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(home).join(".local/share/monkeytype-tui"));
    }
    #[cfg(not(windows))]
    if let Some(appdata) = std::env::var_os("APPDATA").filter(|s| !s.is_empty()) {
        return Some(PathBuf::from(appdata).join("monkeytype-tui"));
    }
    None
}

/// Write `contents` to `path` atomically: write a sibling temp file then rename
/// over the target, so a crash mid-write can't truncate the existing file.
/// All errors are swallowed — persistence is best-effort.
///
/// Known constraint: the pid+counter-unique temp file only guards against two
/// `write_atomic` calls *within this process* racing on the same path. There
/// is no cross-process file lock, so if two instances of the app run at once
/// (e.g. two terminal panes), the last one to save wins and the other's write
/// is lost. Acceptable for a single-user local TUI; not addressed.
pub fn write_atomic(path: &Path, contents: &str) {
    use std::sync::atomic::{AtomicU64, Ordering};

    let Some(parent) = path.parent() else { return };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    // Unique per call (pid + monotonic counter) so two write_atomic calls to
    // the same path can never race on the same tmp file.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = format!(
        "{}-{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let tmp = path.with_extension(unique);
    if std::fs::write(&tmp, contents).is_err() {
        // write() can fail after partially creating/writing `tmp` (e.g. disk
        // full mid-write); clean up rather than leaving debris behind.
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    if std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Read `name` from [`data_dir`] and deserialize it. Returns `T::default()`
/// when there is no data dir, the file is missing, or its contents are
/// malformed — persistence is best-effort and must never fail a startup.
///
/// A malformed file is renamed aside to `<name>.bak` first: the caller's next
/// save (e.g. `history`'s `save_entry`, which always rewrites the whole file)
/// would otherwise overwrite it with a fresh `Default`, permanently losing
/// whatever was there — up to 50 history entries or every personal best —
/// with no warning.
pub fn load_json<T: serde::de::DeserializeOwned + Default>(name: &str) -> T {
    let Some(path) = data_dir().map(|d| d.join(name)) else {
        return T::default();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return T::default();
    };
    match serde_json::from_str(&data) {
        Ok(value) => value,
        Err(_) => {
            let _ = std::fs::rename(&path, path.with_extension("bak"));
            T::default()
        }
    }
}

/// Serialize `value` as pretty JSON and write it to `name` in [`data_dir`] via
/// [`write_atomic`]. Silently does nothing if there is no data dir or the
/// value can't be serialized.
pub fn save_json<T: serde::Serialize>(name: &str, value: &T) {
    let Some(path) = data_dir().map(|d| d.join(name)) else {
        return;
    };
    if let Ok(json) = serde_json::to_string_pretty(value) {
        write_atomic(&path, &json);
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

        // The real temp name is `<path>.<pid>-<counter>.tmp` (from
        // `path.with_extension(unique)`), not `<path>.tmp` — check the
        // actual pattern rather than one the code never produces.
        let prefix = path.file_name().unwrap().to_string_lossy().into_owned();
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(&prefix) && n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "leftover temp files: {leftovers:?}");

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

    /// `load_json` backs every persisted file (config, pb, history). A missing
    /// or hand-corrupted file must degrade to `Default` rather than propagate
    /// an error — a broken config.json must not stop the app from starting.
    #[test]
    fn load_json_falls_back_to_default_on_missing_and_corrupt_files() {
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Sample {
            n: u32,
        }
        impl Default for Sample {
            fn default() -> Self {
                Self { n: 7 }
            }
        }

        // Missing file.
        let missing: Sample = load_json("definitely-not-a-real-file-xyz.json");
        assert_eq!(missing, Sample::default());

        // Corrupt file: write garbage into the real data dir, then read it back.
        // `load_json` renames the corrupt file aside to `.bak` (see the
        // dedicated backup test below), so clean that up too.
        let name = "monkeytype-tui-test-corrupt.json";
        let path = data_dir().unwrap().join(name);
        write_atomic(&path, "{ this is not json");
        let corrupt: Sample = load_json(name);
        assert_eq!(corrupt, Sample::default());
        std::fs::remove_file(path.with_extension("bak")).unwrap();

        // Well-formed file round-trips instead of silently defaulting.
        write_atomic(&path, r#"{"n": 42}"#);
        let good: Sample = load_json(name);
        assert_eq!(good, Sample { n: 42 });

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn load_json_backs_up_a_corrupt_file_instead_of_discarding_it() {
        let name = "monkeytype-tui-test-corrupt-backup.json";
        let path = data_dir().unwrap().join(name);
        let bak = path.with_extension("bak");
        std::fs::remove_file(&bak).ok(); // in case a prior failed run left one

        write_atomic(&path, "{ not valid json");
        let restored: Vec<u8> = load_json(name);
        assert_eq!(restored, Vec::<u8>::default());
        assert!(
            !path.exists(),
            "corrupt file must be moved aside, not left in place"
        );
        assert_eq!(std::fs::read_to_string(&bak).unwrap(), "{ not valid json");

        std::fs::remove_file(&bak).unwrap();
    }

    #[test]
    fn save_json_writes_readable_json() {
        let name = "monkeytype-tui-test-savejson.json";
        save_json(name, &vec![1u8, 2, 3]);
        let back: Vec<u8> = load_json(name);
        assert_eq!(back, vec![1, 2, 3]);
        std::fs::remove_file(data_dir().unwrap().join(name)).unwrap();
    }
}
