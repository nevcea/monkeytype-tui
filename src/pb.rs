//! Personal-best persistence: one [`PbEntry`] per mode+language key, saved
//! to `pb.json` via `storage::write_atomic`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct PbEntry {
    pub wpm: f64,
    pub acc: f64,
}

pub type PersonalBests = HashMap<String, PbEntry>;

fn pb_path() -> Option<PathBuf> {
    Some(crate::storage::data_dir()?.join("pb.json"))
}

pub fn load_pb() -> PersonalBests {
    let Some(path) = pb_path() else {
        return HashMap::new();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_pb(pb: &PersonalBests) {
    let Some(path) = pb_path() else { return };
    if let Ok(json) = serde_json::to_string_pretty(pb) {
        crate::storage::write_atomic(&path, &json);
    }
}

/// Returns true if this result is a new personal best.
pub fn update_pb(pb: &mut PersonalBests, key: String, wpm: f64, acc: f64) -> bool {
    // Higher WPM wins; on an equal WPM, prefer the cleaner (higher-accuracy) run
    // so a sloppier attempt can't overwrite a tidy personal best.
    let is_new = pb
        .get(&key)
        .is_none_or(|e| wpm > e.wpm || (wpm == e.wpm && acc > e.acc));
    if is_new {
        pb.insert(key, PbEntry { wpm, acc });
    }
    is_new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_result_is_always_a_pb() {
        let mut pb = PersonalBests::new();
        assert!(update_pb(&mut pb, "k".into(), 50.0, 95.0));
    }

    #[test]
    fn higher_wpm_beats_existing() {
        let mut pb = PersonalBests::new();
        update_pb(&mut pb, "k".into(), 50.0, 99.0);
        assert!(update_pb(&mut pb, "k".into(), 60.0, 80.0));
        assert_eq!(pb["k"].wpm, 60.0);
    }

    #[test]
    fn lower_wpm_does_not_overwrite() {
        let mut pb = PersonalBests::new();
        update_pb(&mut pb, "k".into(), 60.0, 90.0);
        assert!(!update_pb(&mut pb, "k".into(), 50.0, 100.0));
        assert_eq!(pb["k"].wpm, 60.0);
    }

    #[test]
    fn equal_wpm_breaks_tie_on_accuracy() {
        let mut pb = PersonalBests::new();
        update_pb(&mut pb, "k".into(), 60.0, 90.0);
        assert!(update_pb(&mut pb, "k".into(), 60.0, 95.0));
        assert_eq!(pb["k"].acc, 95.0);
        assert!(!update_pb(&mut pb, "k".into(), 60.0, 92.0));
    }
}
