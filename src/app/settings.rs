use crossterm::event::{KeyCode, KeyEvent};

use super::{App, DEFAULT_VOLUME_PCT, Screen};
use crate::history;
use crate::sound::SoundPack;

/// Settings-screen rows (cursor shape, sound, volume, history expiry,
/// difficulty, theme).
const SETTINGS_ROWS: usize = 6;
const SETTINGS_ROW_VOLUME: usize = 2;
const VOLUME_STEP: u8 = 5;
const VOLUME_MIN: u8 = 1;
const VOLUME_MAX: u8 = 100;

impl App {
    pub fn settings_max_cursor(&self) -> usize {
        SETTINGS_ROWS - 1
    }

    fn apply_volume_input(&mut self) {
        if let Some(input) = self.settings_state.volume_input.take()
            && let Ok(pct) = input.parse::<u16>()
            && let Some(s) = &mut self.sound
        {
            s.set_volume_pct(pct.clamp(1, 100) as u8);
        }
    }

    /// Handle digit / backspace editing of the volume field. Returns `true` when
    /// the key was consumed as text input.
    fn handle_settings_volume_input(&mut self, key: KeyEvent) -> bool {
        let on_volume = self.settings_state.cursor == SETTINGS_ROW_VOLUME;
        if !on_volume || self.settings_state.pending_exit || self.sound.is_none() {
            return false;
        }
        match key.code {
            KeyCode::Char(c @ '0'..='9') => {
                let buf = self.settings_state.volume_input.get_or_insert_default();
                if buf.len() < 3 {
                    buf.push(c);
                }
                true
            }
            KeyCode::Backspace => {
                if let Some(buf) = &mut self.settings_state.volume_input {
                    buf.pop();
                    if buf.is_empty() {
                        self.settings_state.volume_input = None;
                    }
                    return true;
                }
                false
            }
            _ => {
                self.apply_volume_input();
                false
            }
        }
    }

    /// Cycle the value on the current settings row (Left = reverse).
    fn adjust_setting_row(&mut self, rev: bool) {
        match self.settings_state.cursor {
            0 => {
                self.settings.cursor_shape = if rev {
                    self.settings.cursor_shape.prev()
                } else {
                    self.settings.cursor_shape.next()
                }
            }
            1 => {
                if let Some(s) = &mut self.sound {
                    s.pack = if rev { s.pack.prev() } else { s.pack.next() };
                }
            }
            SETTINGS_ROW_VOLUME => {
                if let Some(s) = &mut self.sound {
                    let new = if rev {
                        s.volume_pct.saturating_sub(VOLUME_STEP).max(VOLUME_MIN)
                    } else {
                        (s.volume_pct + VOLUME_STEP).min(VOLUME_MAX)
                    };
                    s.set_volume_pct(new);
                }
            }
            3 => {
                self.settings.history_expiry = if rev {
                    self.settings.history_expiry.prev()
                } else {
                    self.settings.history_expiry.next()
                };
            }
            4 => {
                self.settings.difficulty = if rev {
                    self.settings.difficulty.prev()
                } else {
                    self.settings.difficulty.next()
                };
            }
            5 => {
                let themes = crate::ui::all_themes();
                let n = themes.len();
                let cur = themes
                    .iter()
                    .position(|t| t.name == self.settings.theme_name)
                    .unwrap_or(0);
                let next = if rev {
                    (cur + n - 1) % n
                } else {
                    (cur + 1) % n
                };
                self.settings.theme_name = themes[next].name.to_string();
            }
            _ => {}
        }
    }

    pub(super) fn handle_settings(&mut self, key: KeyEvent) {
        let max_cursor = self.settings_max_cursor();

        if self.handle_settings_volume_input(key) {
            return;
        }

        match key.code {
            KeyCode::Char('y') if self.settings_state.pending_exit => {
                if let Some((snap, snap_pack, snap_vol)) = self.settings_state.snapshot.take() {
                    self.settings = snap;
                    if let Some(s) = &mut self.sound {
                        s.pack = snap_pack;
                        s.set_volume_pct(snap_vol);
                    }
                }
                self.settings_state.volume_input = None;
                self.settings_state.pending_exit = false;
                self.screen = Screen::Menu;
                self.history = history::load_history(self.settings.history_expiry);
                self.history_scroll = self
                    .history_scroll
                    .min(self.history.len().saturating_sub(1));
                self.persist();
            }
            KeyCode::Char('n') if self.settings_state.pending_exit => {
                self.settings_state.pending_exit = false;
            }
            KeyCode::Enter => {
                if !self.settings_state.pending_exit {
                    self.apply_volume_input();
                    // update snapshot to current state so * indicators reset
                    self.settings_state.snapshot = Some(self.make_settings_snapshot());
                    self.history = history::load_history(self.settings.history_expiry);
                    self.history_scroll = self
                        .history_scroll
                        .min(self.history.len().saturating_sub(1));
                    self.persist();
                }
            }
            KeyCode::Esc => {
                self.apply_volume_input();
                let sound_pack = self
                    .sound
                    .as_ref()
                    .map(|s| s.pack)
                    .unwrap_or(SoundPack::Off);
                let sound_vol = self
                    .sound
                    .as_ref()
                    .map(|s| s.volume_pct)
                    .unwrap_or(DEFAULT_VOLUME_PCT);
                let unchanged =
                    self.settings_state
                        .snapshot
                        .as_ref()
                        .is_some_and(|(snap, pack, vol)| {
                            self.settings == *snap && sound_pack == *pack && sound_vol == *vol
                        });
                if unchanged {
                    self.settings_state.snapshot = None;
                    self.screen = Screen::Menu;
                    self.persist();
                } else {
                    self.settings_state.pending_exit = !self.settings_state.pending_exit;
                }
            }
            _ if self.settings_state.pending_exit => {
                self.settings_state.pending_exit = false;
            }
            KeyCode::Up => {
                self.apply_volume_input();
                self.settings_state.cursor = self.settings_state.cursor.saturating_sub(1);
            }
            KeyCode::Down => {
                self.apply_volume_input();
                if self.settings_state.cursor < max_cursor {
                    self.settings_state.cursor += 1;
                }
            }
            KeyCode::Left | KeyCode::Right => {
                self.adjust_setting_row(matches!(key.code, KeyCode::Left));
            }
            _ => {}
        }
    }
}
