//! Result history: persists the last [`HISTORY_LIMIT`] test results to
//! `history.json` (via `storage::write_atomic`).

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const HISTORY_LIMIT: usize = 50;

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
        } else if diff < 86400 * 365 {
            format!("{}mo ago", diff / (86400 * 30))
        } else {
            format!("{}y ago", diff / (86400 * 365))
        }
    }
}

const HISTORY_FILE: &str = "history.json";

/// Every stored entry, newest first. Missing/malformed files read as empty.
pub fn load_history() -> Vec<HistoryEntry> {
    crate::storage::load_json(HISTORY_FILE)
}

/// Prepend `entry`, keeping the list newest-first and capped at
/// [`HISTORY_LIMIT`].
fn push_capped(history: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    history.insert(0, entry);
    history.truncate(HISTORY_LIMIT);
}

/// Record `entry` and persist the list. `history` mirrors the file exactly, so
/// writing it back is lossless.
pub fn save_entry(entry: HistoryEntry, history: &mut Vec<HistoryEntry>) {
    push_capped(history, entry);
    crate::storage::save_json(HISTORY_FILE, history);
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
    fn years_ago() {
        assert_eq!(entry_ago(86400 * 365 * 2).time_ago(), "2y ago");
    }

    #[test]
    fn push_capped_puts_newest_first() {
        let mut history = vec![entry_ago(60)];
        let mut newest = entry_ago(0);
        newest.wpm = 99.0;
        push_capped(&mut history, newest);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].wpm, 99.0);
    }

    #[test]
    fn push_capped_caps_at_history_limit() {
        let mut history = vec![entry_ago(60); HISTORY_LIMIT];
        push_capped(&mut history, entry_ago(0));
        assert_eq!(history.len(), HISTORY_LIMIT);
    }
}
