use crossterm::event::{KeyCode, KeyEvent};

use super::{
    App, LANG_PICKER_VISIBLE, LangPicker, Screen, TIME_OPTIONS, WORD_OPTIONS, filtered_languages,
};
use crate::game::Mode;
use crate::words::load_quotes_for;

const CUSTOM_INPUT_MAX_LEN: usize = 5;
/// Clamps for custom time (seconds) and word-count input.
const CUSTOM_TIME_MAX: u64 = 3600;
const CUSTOM_WORDS_MAX: usize = 5000;

impl App {
    pub(super) fn handle_menu(&mut self, key: KeyEvent) {
        if self.handle_menu_lang_picker(key) {
            return;
        }
        if self.handle_menu_custom_input(key) {
            return;
        }
        self.handle_menu_main(key);
    }

    fn handle_menu_lang_picker(&mut self, key: KeyEvent) -> bool {
        let Some(ref mut picker) = self.lang_picker else {
            return false;
        };
        let filtered = filtered_languages(&picker.search);
        let flen = filtered.len();
        match key.code {
            KeyCode::Up => {
                if picker.cursor > 0 {
                    picker.cursor -= 1;
                    picker.size_idx = 0;
                    if picker.cursor < picker.scroll {
                        picker.scroll = picker.cursor;
                    }
                }
            }
            KeyCode::Down => {
                if picker.cursor + 1 < flen {
                    picker.cursor += 1;
                    picker.size_idx = 0;
                    if picker.cursor >= picker.scroll + LANG_PICKER_VISIBLE {
                        picker.scroll = picker.cursor + 1 - LANG_PICKER_VISIBLE;
                    }
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
                }
                self.lang_picker = None;
            }
            KeyCode::Esc => {
                self.lang_picker = None;
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
        if self.custom_input.is_none() {
            return false;
        }
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let s = self.custom_input.as_mut().unwrap();
                if s.len() < CUSTOM_INPUT_MAX_LEN {
                    s.push(c);
                }
            }
            KeyCode::Backspace => {
                self.custom_input.as_mut().unwrap().pop();
            }
            KeyCode::Enter => {
                let s = self.custom_input.as_deref().unwrap_or("");
                if s.is_empty() {
                    return true;
                }
                let val: u64 = self
                    .custom_input
                    .take()
                    .unwrap_or_default()
                    .parse()
                    .unwrap_or(0);
                match self.menu_mode {
                    Mode::Time(_) => {
                        self.custom_time_val = val.clamp(1, CUSTOM_TIME_MAX);
                        self.menu_mode = Mode::Time(self.custom_time_val);
                    }
                    Mode::Words(_) => {
                        self.custom_words_val = (val as usize).clamp(1, CUSTOM_WORDS_MAX);
                        self.menu_mode = Mode::Words(self.custom_words_val);
                    }
                    _ => {}
                }
                self.start_test();
            }
            KeyCode::Esc => {
                self.custom_input = None;
            }
            _ => {}
        }
        true
    }

    fn handle_menu_main(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('1') => {
                self.menu_mode = Mode::Time(
                    TIME_OPTIONS
                        .get(self.menu_time_idx)
                        .copied()
                        .unwrap_or(self.custom_time_val),
                )
            }
            KeyCode::Char('2') => {
                self.menu_mode = Mode::Words(
                    WORD_OPTIONS
                        .get(self.menu_word_idx)
                        .copied()
                        .unwrap_or(self.custom_words_val),
                )
            }
            KeyCode::Char('3') => self.menu_mode = Mode::Quote,

            KeyCode::Left => self.step_menu(false),
            KeyCode::Right => self.step_menu(true),

            KeyCode::Enter | KeyCode::Tab => {
                if self.is_custom_slot() {
                    self.custom_input = Some(String::new());
                } else if matches!(self.menu_mode, Mode::Quote) && self.game.all_quotes.is_empty() {
                    // no-op: warning already shown in UI
                } else {
                    self.start_test();
                }
            }

            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.settings.punctuation = !self.settings.punctuation
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.settings.numbers = !self.settings.numbers
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.settings_state.cursor = 0;
                self.settings_state.snapshot = Some(self.make_settings_snapshot());
                self.screen = Screen::Settings;
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.lang_picker = Some(LangPicker::new(
                    self.settings.lang_idx,
                    self.settings.size_idx,
                ));
            }
            KeyCode::Char('h') | KeyCode::Char('H') => self.screen = Screen::History,
            KeyCode::Char('?') => self.screen = Screen::Help,
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.quit_confirm = true,
            _ => {}
        }
    }

    fn step_menu(&mut self, forward: bool) {
        match self.menu_mode {
            Mode::Time(_) => {
                let idx = &mut self.menu_time_idx;
                if forward && *idx < TIME_OPTIONS.len() {
                    *idx += 1;
                } else if !forward {
                    *idx = idx.saturating_sub(1);
                }
                self.menu_mode = Mode::Time(
                    TIME_OPTIONS
                        .get(*idx)
                        .copied()
                        .unwrap_or(self.custom_time_val),
                );
            }
            Mode::Words(_) => {
                let idx = &mut self.menu_word_idx;
                if forward && *idx < WORD_OPTIONS.len() {
                    *idx += 1;
                } else if !forward {
                    *idx = idx.saturating_sub(1);
                }
                self.menu_mode = Mode::Words(
                    WORD_OPTIONS
                        .get(*idx)
                        .copied()
                        .unwrap_or(self.custom_words_val),
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
