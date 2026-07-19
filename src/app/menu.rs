//! Key handling for the Menu screen: mode/option selection, the language and
//! theme picker overlays, and the custom time/word-count input slot.

use crossterm::event::{KeyCode, KeyEvent};

use super::{
    App, LANG_PICKER_VISIBLE, LangPicker, Screen, THEME_PICKER_VISIBLE, TIME_OPTIONS, ThemePicker,
    WORD_OPTIONS, filtered_languages, filtered_themes,
};
use crate::game::Mode;
use crate::words::load_quotes_for;

const CUSTOM_INPUT_MAX_LEN: usize = 5;
/// Clamps for custom time (seconds) and word-count input.
const CUSTOM_TIME_MAX: u64 = 3600;
const CUSTOM_WORDS_MAX: usize = 5000;

/// Move a picker's `cursor` by one (up if `!down`, down if `down`) within
/// `[0, len)`, keeping it inside the `visible`-row scroll window. Returns
/// whether the cursor actually moved, so callers can reset per-item state
/// (e.g. the language picker's `size_idx`) only when it did.
fn step_picker_cursor(
    cursor: &mut usize,
    scroll: &mut usize,
    len: usize,
    visible: usize,
    down: bool,
) -> bool {
    if down {
        if *cursor + 1 >= len {
            return false;
        }
        *cursor += 1;
        if *cursor >= *scroll + visible {
            *scroll = *cursor + 1 - visible;
        }
    } else {
        if *cursor == 0 {
            return false;
        }
        *cursor -= 1;
        if *cursor < *scroll {
            *scroll = *cursor;
        }
    }
    true
}

/// Step an option index forward (clamped to the custom slot at `len`) or
/// backward (saturating at 0).
fn step_option_idx(idx: usize, len: usize, forward: bool) -> usize {
    if forward {
        (idx + 1).min(len)
    } else {
        idx.saturating_sub(1)
    }
}

impl App {
    pub(super) fn handle_menu(&mut self, key: KeyEvent) {
        if self.handle_menu_lang_picker(key) {
            return;
        }
        if self.handle_menu_theme_picker(key) {
            return;
        }
        if self.handle_menu_custom_input(key) {
            return;
        }
        self.handle_menu_main(key);
    }

    fn handle_menu_theme_picker(&mut self, key: KeyEvent) -> bool {
        let Some(ref mut picker) = self.menu.theme_picker else {
            return false;
        };
        let filtered = filtered_themes(&picker.search);
        let flen = filtered.len();
        // Live-apply the currently highlighted theme so the whole UI previews it.
        let apply_preview = |app: &mut App| {
            if let Some(p) = &app.menu.theme_picker {
                let filtered = filtered_themes(&p.search);
                if let Some(&(_, t)) = filtered.get(p.cursor) {
                    app.settings.theme_name = t.name.to_string();
                }
            }
        };
        match key.code {
            KeyCode::Up => {
                step_picker_cursor(
                    &mut picker.cursor,
                    &mut picker.scroll,
                    flen,
                    THEME_PICKER_VISIBLE,
                    false,
                );
                apply_preview(self);
            }
            KeyCode::Down => {
                step_picker_cursor(
                    &mut picker.cursor,
                    &mut picker.scroll,
                    flen,
                    THEME_PICKER_VISIBLE,
                    true,
                );
                apply_preview(self);
            }
            KeyCode::Enter => {
                // Preview already applied; keep it and persist.
                self.menu.theme_picker = None;
                self.persist();
            }
            KeyCode::Esc => {
                // Restore the theme that was active before opening the picker.
                let original = picker.original.clone();
                self.settings.theme_name = original;
                self.menu.theme_picker = None;
            }
            KeyCode::Backspace => {
                picker.search.pop();
                picker.cursor = 0;
                picker.scroll = 0;
                apply_preview(self);
            }
            KeyCode::Char(c) => {
                picker.search.push(c);
                picker.cursor = 0;
                picker.scroll = 0;
                apply_preview(self);
            }
            _ => {}
        }
        true
    }

    fn handle_menu_lang_picker(&mut self, key: KeyEvent) -> bool {
        let Some(ref mut picker) = self.menu.lang_picker else {
            return false;
        };
        let filtered = filtered_languages(&picker.search);
        let flen = filtered.len();
        match key.code {
            KeyCode::Up => {
                if step_picker_cursor(
                    &mut picker.cursor,
                    &mut picker.scroll,
                    flen,
                    LANG_PICKER_VISIBLE,
                    false,
                ) {
                    picker.size_idx = 0;
                }
            }
            KeyCode::Down => {
                if step_picker_cursor(
                    &mut picker.cursor,
                    &mut picker.scroll,
                    flen,
                    LANG_PICKER_VISIBLE,
                    true,
                ) {
                    picker.size_idx = 0;
                }
            }
            KeyCode::Left => {
                if picker.size_idx > 0 {
                    picker.size_idx -= 1;
                }
            }
            KeyCode::Right => {
                let max = filtered
                    .get(picker.cursor)
                    .map(|(_, l)| l.sizes.len().saturating_sub(1))
                    .unwrap_or(0);
                if picker.size_idx < max {
                    picker.size_idx += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(&(real_idx, lang)) = filtered.get(picker.cursor) {
                    self.settings.lang_idx = real_idx;
                    self.settings.size_idx =
                        picker.size_idx.min(lang.sizes.len().saturating_sub(1));
                    self.game.all_quotes = load_quotes_for(lang.name);
                    self.persist();
                }
                self.menu.lang_picker = None;
            }
            KeyCode::Esc => {
                self.menu.lang_picker = None;
            }
            KeyCode::Backspace => {
                picker.search.pop();
                picker.cursor = 0;
                picker.scroll = 0;
                picker.size_idx = 0;
            }
            KeyCode::Char(c) => {
                picker.search.push(c);
                picker.cursor = 0;
                picker.scroll = 0;
                picker.size_idx = 0;
            }
            _ => {}
        }
        true
    }

    fn handle_menu_custom_input(&mut self, key: KeyEvent) -> bool {
        if self.menu.custom_input.is_none() {
            return false;
        }
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let s = self.menu.custom_input.as_mut().unwrap();
                if s.len() < CUSTOM_INPUT_MAX_LEN {
                    s.push(c);
                }
            }
            KeyCode::Backspace => {
                self.menu.custom_input.as_mut().unwrap().pop();
            }
            KeyCode::Enter => {
                let s = self.menu.custom_input.as_deref().unwrap_or("");
                if s.is_empty() {
                    return true;
                }
                let val: u64 = self
                    .menu
                    .custom_input
                    .take()
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(0);
                match self.menu.mode {
                    Mode::Time(_) => {
                        self.menu.custom_time_val = val.clamp(1, CUSTOM_TIME_MAX);
                        self.menu.mode = Mode::Time(self.menu.custom_time_val);
                    }
                    Mode::Words(_) => {
                        self.menu.custom_words_val = (val as usize).clamp(1, CUSTOM_WORDS_MAX);
                        self.menu.mode = Mode::Words(self.menu.custom_words_val);
                    }
                    _ => {}
                }
                self.start_test();
            }
            KeyCode::Esc => {
                self.menu.custom_input = None;
            }
            _ => {}
        }
        true
    }

    fn handle_menu_main(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('1') => {
                self.menu.mode = Mode::Time(
                    TIME_OPTIONS
                        .get(self.menu.time_idx)
                        .copied()
                        .unwrap_or(self.menu.custom_time_val),
                )
            }
            KeyCode::Char('2') => {
                self.menu.mode = Mode::Words(
                    WORD_OPTIONS
                        .get(self.menu.word_idx)
                        .copied()
                        .unwrap_or(self.menu.custom_words_val),
                )
            }
            KeyCode::Char('3') => self.menu.mode = Mode::Quote,

            KeyCode::Left => self.step_menu(false),
            KeyCode::Right => self.step_menu(true),

            KeyCode::Enter | KeyCode::Tab => {
                if self.is_custom_slot() {
                    self.menu.custom_input = Some(String::new());
                } else if matches!(self.menu.mode, Mode::Quote) && self.game.all_quotes.is_empty() {
                    // no-op: warning already shown in UI
                } else {
                    self.start_test();
                }
            }

            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.settings.punctuation = !self.settings.punctuation;
                self.persist();
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.settings.numbers = !self.settings.numbers;
                self.persist();
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.settings_state.cursor = 0;
                self.settings_state.snapshot = Some(self.make_settings_snapshot());
                self.screen = Screen::Settings;
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.menu.lang_picker = Some(LangPicker::new(
                    self.settings.lang_idx,
                    self.settings.size_idx,
                ));
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.menu.theme_picker = Some(ThemePicker::new(&self.settings.theme_name));
            }
            KeyCode::Char('h') | KeyCode::Char('H') => self.screen = Screen::History,
            KeyCode::Char('?') => self.screen = Screen::Help,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                self.dialog.quit_confirm = true
            }
            _ => {}
        }
    }

    fn step_menu(&mut self, forward: bool) {
        match self.menu.mode {
            Mode::Time(_) => {
                self.menu.time_idx =
                    step_option_idx(self.menu.time_idx, TIME_OPTIONS.len(), forward);
                self.menu.mode = Mode::Time(
                    TIME_OPTIONS
                        .get(self.menu.time_idx)
                        .copied()
                        .unwrap_or(self.menu.custom_time_val),
                );
            }
            Mode::Words(_) => {
                self.menu.word_idx =
                    step_option_idx(self.menu.word_idx, WORD_OPTIONS.len(), forward);
                self.menu.mode = Mode::Words(
                    WORD_OPTIONS
                        .get(self.menu.word_idx)
                        .copied()
                        .unwrap_or(self.menu.custom_words_val),
                );
            }
            Mode::Quote => {
                self.settings.quote_filter = if forward {
                    self.settings.quote_filter.next()
                } else {
                    self.settings.quote_filter.prev()
                };
            }
        }
    }
}
