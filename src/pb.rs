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
    Some(PathBuf::from(std::env::var("HOME").ok()?).join(".local/share/monkeytype-tui/pb.json"))
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
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(pb) {
        let _ = std::fs::write(&path, json);
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
