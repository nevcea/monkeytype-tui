use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;
use unicode_width::UnicodeWidthChar;

use crate::game::{GameState, Mode, Settings};
use crate::history::HistoryEntry;
use crate::pb::{self, PersonalBests};
use crate::sound::{SoundPack, SoundPlayer};
use crate::words::{LANGUAGES, LangDef, load_quotes_for};

mod help;
mod history;
mod menu;
mod result;
mod settings;
mod test;

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Menu,
    Test,
    Result,
    History,
    Help,
    Settings,
}

/// Minimum terminal size the UI renders at; below this `ui::draw` shows a hint.
pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 20;

pub const TIME_OPTIONS: &[u64] = &[15, 30, 60, 120];
pub const WORD_OPTIONS: &[usize] = &[10, 25, 50, 100];
pub const LANG_PICKER_VISIBLE: usize = 12;
pub const THEME_PICKER_VISIBLE: usize = 12;

const DEFAULT_VOLUME_PCT: u8 = 25;

pub struct SettingsState {
    pub cursor: usize,
    pub pending_exit: bool,
    pub snapshot: Option<(Settings, SoundPack, u8)>,
    pub volume_input: Option<String>,
}

pub fn filtered_languages(search: &str) -> Vec<(usize, &'static LangDef)> {
    // Lowercase the query once instead of per-language.
    let needle = search.to_lowercase();
    LANGUAGES
        .iter()
        .enumerate()
        .filter(|(_, l)| l.name.to_lowercase().contains(&needle))
        .collect()
}

pub fn filtered_themes(search: &str) -> Vec<(usize, &'static crate::ui::Theme)> {
    let needle = search.to_lowercase();
    crate::ui::all_themes()
        .iter()
        .enumerate()
        .filter(|(_, t)| t.name.to_lowercase().contains(&needle))
        .collect()
}

pub struct LangPicker {
    pub cursor: usize,
    pub size_idx: usize,
    pub scroll: usize,
    pub search: String,
}

/// Theme-picker overlay state. Moving the cursor live-applies the highlighted
/// theme (`settings.theme_name`); `original` restores it if the user cancels.
pub struct ThemePicker {
    pub cursor: usize,
    pub scroll: usize,
    pub search: String,
    pub original: String,
}

impl ThemePicker {
    fn new(current: &str) -> Self {
        let cursor = crate::ui::all_themes()
            .iter()
            .position(|t| t.name == current)
            .unwrap_or(0);
        Self {
            cursor,
            scroll: cursor.saturating_sub(4),
            search: String::new(),
            original: current.to_string(),
        }
    }
}

impl LangPicker {
    fn new(lang_idx: usize, size_idx: usize) -> Self {
        let max_size = LANGUAGES
            .get(lang_idx)
            .map(|l| l.sizes.len().saturating_sub(1))
            .unwrap_or(0);
        Self {
            cursor: lang_idx,
            size_idx: size_idx.min(max_size),
            scroll: lang_idx.saturating_sub(4),
            search: String::new(),
        }
    }
}

/// Menu-screen selection state: which mode/option is highlighted, the pending
/// custom-value input, and the language-picker overlay.
pub struct MenuState {
    pub mode: Mode,
    pub time_idx: usize,
    pub word_idx: usize,
    pub custom_input: Option<String>,
    pub custom_time_val: u64,
    pub custom_words_val: usize,
    pub lang_picker: Option<LangPicker>,
    pub theme_picker: Option<ThemePicker>,
}

/// Modal-dialog state (quit / abandon-test confirmations) with each dialog's
/// currently highlighted yes/no choice.
#[derive(Default)]
pub struct DialogState {
    pub quit_confirm: bool,
    pub quit_yes: bool,
    pub test_confirm: bool,
    pub test_confirm_yes: bool,
}

pub struct App {
    pub screen: Screen,
    pub game: GameState,
    pub settings: Settings,
    pub menu: MenuState,
    pub dialog: DialogState,
    pub scroll_word: usize,
    pub last_width: u16,
    pub last_height: u16,
    pub history: Vec<HistoryEntry>,
    pub history_scroll: usize,
    pub should_quit: bool,
    pub settings_state: SettingsState,
    pub sound: Option<SoundPlayer>,
    pub session_start: Instant,
    pub result_session_secs: u64,
    pub pb: PersonalBests,
    pub is_new_pb: bool,
    result_saved: bool,
}

impl App {
    pub fn new() -> Self {
        let cfg = crate::config::load_config();
        let settings = cfg.settings;
        // Load quotes for the persisted language so quote mode works immediately.
        let lang = LANGUAGES
            .get(settings.lang_idx)
            .map(|l| l.name)
            .unwrap_or("english");
        let quotes = load_quotes_for(lang);
        let game = GameState::new(cfg.mode, settings.clone(), quotes);
        let history_expiry = settings.history_expiry;
        // Apply persisted sound preferences on top of the default player.
        let sound = SoundPlayer::new().map(|mut s| {
            s.pack = cfg.sound_pack;
            s.set_volume_pct(cfg.volume_pct);
            s
        });
        Self {
            screen: Screen::Menu,
            game,
            settings,
            menu: MenuState {
                mode: cfg.mode,
                time_idx: cfg.menu_time_idx,
                word_idx: cfg.menu_word_idx,
                custom_input: None,
                custom_time_val: cfg.custom_time_val,
                custom_words_val: cfg.custom_words_val,
                lang_picker: None,
                theme_picker: None,
            },
            dialog: DialogState::default(),
            scroll_word: 0,
            last_width: 80,
            last_height: 24,
            history: crate::history::load_history(history_expiry),
            history_scroll: 0,
            should_quit: false,
            settings_state: SettingsState {
                cursor: 0,
                pending_exit: false,
                snapshot: None,
                volume_input: None,
            },
            sound,
            session_start: Instant::now(),
            result_session_secs: 0,
            pb: pb::load_pb(),
            is_new_pb: false,
            result_saved: false,
        }
    }

    /// Persist the current settings, menu selection, and sound preferences to
    /// `config.json`. Called at commit points (mode/language/toggle changes and
    /// settings-screen exits) so preferences survive a restart.
    pub(super) fn persist(&self) {
        // ponytail: when no audio device is present we can't read live sound
        // prefs, so fall back to sensible defaults rather than clobbering to Off.
        let (pack, vol) = self
            .sound
            .as_ref()
            .map(|s| (s.pack, s.volume_pct))
            .unwrap_or((SoundPack::Click, DEFAULT_VOLUME_PCT));
        crate::config::save_config(&crate::config::PersistedConfig {
            settings: self.settings.clone(),
            sound_pack: pack,
            volume_pct: vol,
            mode: self.menu.mode,
            menu_time_idx: self.menu.time_idx,
            menu_word_idx: self.menu.word_idx,
            custom_time_val: self.menu.custom_time_val,
            custom_words_val: self.menu.custom_words_val,
        });
    }

    pub fn is_custom_slot(&self) -> bool {
        match self.menu.mode {
            Mode::Time(_) => self.menu.time_idx == TIME_OPTIONS.len(),
            Mode::Words(_) => self.menu.word_idx == WORD_OPTIONS.len(),
            Mode::Quote => false,
        }
    }

    fn start_test(&mut self) {
        let mode = self.menu.mode;
        let quotes = std::mem::take(&mut self.game.all_quotes);
        self.game = GameState::new(mode, self.settings.clone(), quotes);
        // Remember the chosen mode/menu selection for next launch.
        self.persist();
        self.begin_replay();
    }

    /// Restart with a freshly generated set of words/quote.
    fn restart_test(&mut self) {
        self.game.settings = self.settings.clone();
        self.game.reset();
        self.begin_replay();
    }

    /// Replay the exact same words (used from the result screen).
    fn repeat_test(&mut self) {
        self.game.settings = self.settings.clone();
        self.game.repeat();
        self.begin_replay();
    }

    /// Shared post-reset bookkeeping to (re)enter the Test screen.
    fn begin_replay(&mut self) {
        self.scroll_word = 0;
        self.result_saved = false;
        self.screen = Screen::Test;
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        if self.dialog.quit_confirm {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.dialog.quit_yes = !self.dialog.quit_yes
                }
                KeyCode::Enter => {
                    if self.dialog.quit_yes {
                        self.should_quit = true;
                    } else {
                        self.dialog.quit_confirm = false;
                        self.dialog.quit_yes = false;
                    }
                }
                KeyCode::Esc => {
                    self.dialog.quit_confirm = false;
                    self.dialog.quit_yes = false;
                }
                _ => {}
            }
            return;
        }
        if self.dialog.test_confirm {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.dialog.test_confirm_yes = !self.dialog.test_confirm_yes
                }
                KeyCode::Enter => {
                    let go = self.dialog.test_confirm_yes;
                    self.dialog.test_confirm = false;
                    self.dialog.test_confirm_yes = false;
                    if go {
                        self.screen = Screen::Menu;
                    }
                }
                KeyCode::Esc => {
                    self.dialog.test_confirm = false;
                    self.dialog.test_confirm_yes = false;
                }
                _ => {}
            }
            return;
        }
        match self.screen {
            Screen::Menu => self.handle_menu(key),
            Screen::Test => self.handle_test(key),
            Screen::Result => self.handle_result(key),
            Screen::History => self.handle_history(key),
            Screen::Help => self.handle_help(key),
            Screen::Settings => self.handle_settings(key),
        }
    }

    pub fn tick(&mut self) {
        if self.screen == Screen::Test {
            self.game.tick();
            self.maybe_finish();
        }
    }

    /// Single completion path: if the game has finished, persist the result and
    /// switch to the Result screen. Called from both `tick` (time mode) and after
    /// a keystroke in `handle_test` (word/quote completion). Idempotent — the
    /// `result_saved` guard in `save_result` prevents double-saving.
    pub(super) fn maybe_finish(&mut self) {
        if self.game.is_finished() {
            self.save_result();
            self.screen = Screen::Result;
        }
    }

    fn save_result(&mut self) {
        if self.result_saved {
            return;
        }
        self.result_saved = true;
        self.is_new_pb = false;
        self.result_session_secs = self.session_start.elapsed().as_secs();
        if let Some(s) = &self.sound {
            s.play_complete();
        }
        if self.game.is_failed() {
            return;
        }
        let wpm = self.game.wpm();
        let acc = self.game.accuracy();
        let mode_str = self.game.mode.to_string();
        let lang = LANGUAGES
            .get(self.settings.lang_idx)
            .map(|l| l.name)
            .unwrap_or("english")
            .to_string();
        let key = format!("{mode_str}_{lang}");
        self.is_new_pb = pb::update_pb(&mut self.pb, key, wpm, acc);
        if self.is_new_pb {
            pb::save_pb(&self.pb);
        }
        let entry = HistoryEntry {
            wpm,
            accuracy: acc,
            mode: mode_str,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            language: lang,
        };
        crate::history::save_entry(entry, &mut self.history);
    }

    fn make_settings_snapshot(&self) -> (Settings, SoundPack, u8) {
        (
            self.settings.clone(),
            self.sound
                .as_ref()
                .map(|s| s.pack)
                .unwrap_or(SoundPack::Off),
            self.sound
                .as_ref()
                .map(|s| s.volume_pct)
                .unwrap_or(DEFAULT_VOLUME_PCT),
        )
    }

    // ── test ─────────────────────────────────────────────────────────────────

    /// True when the terminal is below the renderable minimum (see `ui::draw`).
    pub fn too_small(&self) -> bool {
        self.last_width < MIN_WIDTH || self.last_height < MIN_HEIGHT
    }

    // ── scroll ───────────────────────────────────────────────────────────────

    pub fn update_scroll(&mut self) {
        if self.game.cursor == 0 {
            self.scroll_word = 0;
            return;
        }
        let pad = (self.last_width as usize / 10).clamp(4, 10);
        let width = (self.last_width as usize).saturating_sub(2 * pad);
        if width < 10 {
            return;
        }
        let cursor_word = self.game.word_at_cursor();
        if cursor_word < self.scroll_word {
            // Scroll back to the line that contains cursor_word (don't jump to 0)
            let all_lines = word_lines(&self.game.words, 0, width);
            self.scroll_word = all_lines
                .iter()
                .find(|line| line.last().copied().unwrap_or(0) >= cursor_word)
                .and_then(|line| line.first())
                .copied()
                .unwrap_or(0);
            return;
        }
        let lines = word_lines(&self.game.words, self.scroll_word, width);
        if let Some(line0_last) = lines.first().and_then(|l| l.last()).copied()
            && cursor_word > line0_last
            && let Some(line1_start) = lines.get(1).and_then(|l| l.first()).copied()
        {
            self.scroll_word = line1_start;
        }
    }
}

/// Layout words into wrapped lines. Returns word-index groups per line.
pub fn word_lines(words: &[String], from: usize, width: usize) -> Vec<Vec<usize>> {
    let mut lines: Vec<Vec<usize>> = vec![];
    let mut line: Vec<usize> = vec![];
    let mut used = 0usize;
    for (i, word) in words.iter().enumerate().skip(from) {
        let wlen: usize = word
            .chars()
            .map(|c| UnicodeWidthChar::width(c).unwrap_or(1))
            .sum();
        let needed = if used == 0 { wlen } else { wlen + 1 };
        if used > 0 && used + needed > width {
            lines.push(std::mem::take(&mut line));
            line.push(i);
            used = wlen;
        } else {
            line.push(i);
            used += needed;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::word_lines;

    #[test]
    fn word_lines_single_word() {
        let words = vec!["hello".to_string()];
        assert_eq!(word_lines(&words, 0, 20), vec![vec![0usize]]);
    }

    #[test]
    fn word_lines_wraps_at_width() {
        // "hello world" = 11 chars > width 10, so "world" wraps to next line
        let words = vec!["hello".to_string(), "world".to_string(), "foo".to_string()];
        let lines = word_lines(&words, 0, 10);
        assert_eq!(lines, vec![vec![0], vec![1, 2]]);
    }

    #[test]
    fn word_lines_empty() {
        assert!(word_lines(&[], 0, 80).is_empty());
    }
}
