//! Persisted user configuration (`config.json`). Mirrors the same atomic
//! load/save pattern as `history`/`pb`, reusing `storage::{data_dir,write_atomic}`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::game::{Mode, Settings};
use crate::sound::SoundPack;

/// Everything that should survive a restart: the settings block plus the menu
/// selection and sound preferences (which live outside `Settings` at runtime).
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedConfig {
    pub settings: Settings,
    pub sound_pack: SoundPack,
    pub volume_pct: u8,
    pub mode: Mode,
    pub menu_time_idx: usize,
    pub menu_word_idx: usize,
    pub custom_time_val: u64,
    pub custom_words_val: usize,
}

impl Default for PersistedConfig {
    fn default() -> Self {
        // Mirrors the first-run defaults in `App::new`.
        Self {
            settings: Settings::default(),
            sound_pack: SoundPack::Click,
            volume_pct: 25,
            mode: Mode::Time(30),
            menu_time_idx: 1,
            menu_word_idx: 2,
            custom_time_val: 45,
            custom_words_val: 75,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    Some(crate::storage::data_dir()?.join("config.json"))
}

pub fn load_config() -> PersistedConfig {
    let Some(path) = config_path() else {
        return PersistedConfig::default();
    };
    let Ok(data) = std::fs::read_to_string(&path) else {
        return PersistedConfig::default();
    };
    // Unknown/missing fields fall back to Default thanks to `#[serde(default)]`.
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_config(cfg: &PersistedConfig) {
    let Some(path) = config_path() else { return };
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        crate::storage::write_atomic(&path, &json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{CursorShape, Difficulty};

    #[test]
    fn round_trip_preserves_fields() {
        let mut cfg = PersistedConfig::default();
        cfg.settings.punctuation = true;
        cfg.settings.difficulty = Difficulty::Expert;
        cfg.settings.cursor_shape = CursorShape::Underline;
        cfg.settings.theme_name = "nord".to_string();
        cfg.mode = Mode::Words(42);
        cfg.volume_pct = 80;
        let json = serde_json::to_string(&cfg).unwrap();
        let back: PersistedConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.settings, cfg.settings);
        assert_eq!(back.mode, Mode::Words(42));
        assert_eq!(back.volume_pct, 80);
    }

    #[test]
    fn missing_fields_use_defaults() {
        // An older/partial config with only some keys still deserializes.
        let back: PersistedConfig = serde_json::from_str(r#"{"volume_pct": 50}"#).unwrap();
        assert_eq!(back.volume_pct, 50);
        assert_eq!(back.mode, Mode::Time(30));
        assert_eq!(back.settings.theme_name, crate::game::DEFAULT_THEME);
    }

    #[test]
    fn empty_object_is_all_defaults() {
        let back: PersistedConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(back.settings, Settings::default());
        assert_eq!(back.menu_time_idx, 1);
    }
}
