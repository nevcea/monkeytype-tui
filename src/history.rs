//! Result history: persists the last [`HISTORY_LIMIT`] test results to
//! `history.json` (via `storage::write_atomic`) and filters them by
//! [`HistoryExpiry`] on load.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const HISTORY_LIMIT: usize = 50;

cycle_enum! {
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub enum HistoryExpiry {
        Days7 = "7 days",
        Days30 = "30 days",
        Days90 = "90 days",
        Off = "off",
    }
    default = Days90;
}

impl HistoryExpiry {
    fn cutoff_secs(self) -> Option<u64> {
        let days: u64 = match self {
            Self::Days7 => 7,
            Self::Days30 => 30,
            Self::Days90 => 90,
            Self::Off => return None,
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Some(now.saturating_sub(days * 86400))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub wpm: f64,
    pub accuracy: f64,
    pub mode: String,
    pub timestamp: u64,
    #[serde(default)]
    pub language: String,
}

impl HistoryEntry {
    pub fn time_ago(&self) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let diff = now.saturating_sub(self.timestamp);
        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else if diff < 86400 * 7 {
            format!("{}d ago", diff / 86400)
        } else if diff < 86400 * 30 {
            format!("{}w ago", diff / (86400 * 7))
        } else {
            format!("{}mo ago", diff / (86400 * 30))
        }
    }
}

fn history_path() -> Option<PathBuf> {
    Some(crate::storage::data_dir()?.join("history.json"))
}

/// Every entry on disk, unfiltered. Missing/malformed files read as empty.
fn read_all() -> Vec<HistoryEntry> {
    let Some(path) = history_path() else {
        return vec![];
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return vec![];
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn load_history(expiry: HistoryExpiry) -> Vec<HistoryEntry> {
    let entries = read_all();
    match expiry.cutoff_secs() {
        Some(cutoff) => entries
            .into_iter()
            .filter(|e| e.timestamp >= cutoff)
            .collect(),
        None => entries,
    }
}

/// Prepend `entry` to the full on-disk list, capped at [`HISTORY_LIMIT`].
fn merged_for_disk(entry: &HistoryEntry, mut on_disk: Vec<HistoryEntry>) -> Vec<HistoryEntry> {
    on_disk.insert(0, entry.clone());
    on_disk.truncate(HISTORY_LIMIT);
    on_disk
}

pub fn save_entry(entry: HistoryEntry, history: &mut Vec<HistoryEntry>) {
    // Merge into what's on disk rather than writing `history` back: `history`
    // is the expiry-*filtered* view, so persisting it would permanently delete
    // entries the user only asked to hide.
    let all = merged_for_disk(&entry, read_all());
    history.insert(0, entry);
    history.truncate(HISTORY_LIMIT);
    let Some(path) = history_path() else { return };
    if let Ok(json) = serde_json::to_string_pretty(&all) {
        crate::storage::write_atomic(&path, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_ago(secs: u64) -> HistoryEntry {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        HistoryEntry {
            wpm: 0.0,
            accuracy: 0.0,
            mode: String::new(),
            timestamp: now.saturating_sub(secs),
            language: String::new(),
        }
    }

    #[test]
    fn just_now() {
        assert_eq!(entry_ago(30).time_ago(), "just now");
    }

    #[test]
    fn minutes_ago() {
        assert_eq!(entry_ago(120).time_ago(), "2m ago");
    }

    #[test]
    fn hours_ago() {
        assert_eq!(entry_ago(7200).time_ago(), "2h ago");
    }

    #[test]
    fn days_ago() {
        assert_eq!(entry_ago(86400 * 2).time_ago(), "2d ago");
    }

    #[test]
    fn weeks_ago() {
        assert_eq!(entry_ago(86400 * 14).time_ago(), "2w ago");
    }

    #[test]
    fn months_ago() {
        assert_eq!(entry_ago(86400 * 60).time_ago(), "2mo ago");
    }

    #[test]
    fn expiry_cutoff_filters_old_entries() {
        let cutoff = HistoryExpiry::Days90.cutoff_secs().unwrap();
        assert!(entry_ago(90 * 86400 + 1).timestamp < cutoff);
        assert!(entry_ago(86400).timestamp >= cutoff);
    }

    #[test]
    fn expiry_off_has_no_cutoff() {
        assert!(HistoryExpiry::Off.cutoff_secs().is_none());
    }

    /// Saving must not drop entries the active expiry window hides — the
    /// in-memory list is a filtered view, the file is the source of truth.
    #[test]
    fn saving_keeps_entries_hidden_by_the_expiry_filter() {
        let expired = entry_ago(120 * 86400);
        let merged = merged_for_disk(&entry_ago(0), vec![expired]);
        assert_eq!(merged.len(), 2);
        let cutoff = HistoryExpiry::Days90.cutoff_secs().unwrap();
        assert!(merged.iter().any(|e| e.timestamp < cutoff));
    }

    #[test]
    fn merge_caps_at_history_limit() {
        let on_disk = vec![entry_ago(60); HISTORY_LIMIT];
        assert_eq!(merged_for_disk(&entry_ago(0), on_disk).len(), HISTORY_LIMIT);
    }

    #[test]
    fn expiry_cycles() {
        assert_eq!(HistoryExpiry::Days90.next(), HistoryExpiry::Off);
        assert_eq!(HistoryExpiry::Off.next(), HistoryExpiry::Days7);
        assert_eq!(HistoryExpiry::Days7.prev(), HistoryExpiry::Off);
    }
}
