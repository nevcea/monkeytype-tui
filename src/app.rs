use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;
use unicode_width::UnicodeWidthChar;

use crate::game::{GameState, Mode, Settings};
use crate::history::{self, HistoryEntry};
use crate::pb::{self, PersonalBests};
use crate::sound::{SoundPack, SoundPlayer};
use crate::words::{LANGUAGES, LangDef, load_quotes_for};

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
const CUSTOM_INPUT_MAX_LEN: usize = 5;

/// Settings-screen rows (cursor shape, sound, volume, history expiry,
/// difficulty, theme).
const SETTINGS_ROWS: usize = 6;
const SETTINGS_ROW_VOLUME: usize = 2;
const DEFAULT_VOLUME_PCT: u8 = 25;
const VOLUME_STEP: u8 = 5;
const VOLUME_MIN: u8 = 1;
const VOLUME_MAX: u8 = 100;
/// Clamps for custom time (seconds) and word-count input.
const CUSTOM_TIME_MAX: u64 = 3600;
const CUSTOM_WORDS_MAX: usize = 5000;

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

pub struct LangPicker {
    pub cursor: usize,
    pub size_idx: usize,
    pub scroll: usize,
    pub search: String,
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

pub struct App {
    pub screen: Screen,
    pub game: GameState,
    pub settings: Settings,
    pub menu_mode: Mode,
    pub menu_time_idx: usize,
    pub menu_word_idx: usize,
    pub custom_input: Option<String>,
    pub custom_time_val: u64,
    pub custom_words_val: usize,
    pub lang_picker: Option<LangPicker>,
    pub scroll_word: usize,
    pub last_width: u16,
    pub last_height: u16,
    pub history: Vec<HistoryEntry>,
    pub history_scroll: usize,
    pub should_quit: bool,
    pub quit_confirm: bool,
    pub quit_yes: bool,
    pub test_confirm: bool,
    pub test_confirm_yes: bool,
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
        let settings = Settings::default();
        let quotes = load_quotes_for("english");
        let game = GameState::new(Mode::Time(30), settings.clone(), quotes);
        let history_expiry = settings.history_expiry;
        Self {
            screen: Screen::Menu,
            game,
            settings,
            menu_mode: Mode::Time(30),
            menu_time_idx: 1,
            menu_word_idx: 2,
            custom_input: None,
            custom_time_val: 45,
            custom_words_val: 75,
            lang_picker: None,
            scroll_word: 0,
            last_width: 80,
            last_height: 24,
            history: history::load_history(history_expiry),
            history_scroll: 0,
            should_quit: false,
            quit_confirm: false,
            quit_yes: false,
            test_confirm: false,
            test_confirm_yes: false,
            settings_state: SettingsState {
                cursor: 0,
                pending_exit: false,
                snapshot: None,
                volume_input: None,
            },
            sound: SoundPlayer::new(),
            session_start: Instant::now(),
            result_session_secs: 0,
            pb: pb::load_pb(),
            is_new_pb: false,
            result_saved: false,
        }
    }

    pub fn is_custom_slot(&self) -> bool {
        match self.menu_mode {
            Mode::Time(_) => self.menu_time_idx == TIME_OPTIONS.len(),
            Mode::Words(_) => self.menu_word_idx == WORD_OPTIONS.len(),
            Mode::Quote => false,
        }
    }

    fn start_test(&mut self) {
        let mode = self.menu_mode;
        let quotes = std::mem::take(&mut self.game.all_quotes);
        self.game = GameState::new(mode, self.settings.clone(), quotes);
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
        if self.quit_confirm {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => self.quit_yes = !self.quit_yes,
                KeyCode::Enter => {
                    if self.quit_yes {
                        self.should_quit = true;
                    } else {
                        self.quit_confirm = false;
                        self.quit_yes = false;
                    }
                }
                KeyCode::Esc => {
                    self.quit_confirm = false;
                    self.quit_yes = false;
                }
                _ => {}
            }
            return;
        }
        if self.test_confirm {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.test_confirm_yes = !self.test_confirm_yes
                }
                KeyCode::Enter => {
                    let go = self.test_confirm_yes;
                    self.test_confirm = false;
                    self.test_confirm_yes = false;
                    if go {
                        self.screen = Screen::Menu;
                    }
                }
                KeyCode::Esc => {
                    self.test_confirm = false;
                    self.test_confirm_yes = false;
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
            if self.game.is_finished() {
                self.save_result();
                self.screen = Screen::Result;
            }
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
        history::save_entry(entry, &mut self.history);
    }

    // ── menu ─────────────────────────────────────────────────────────────────

    fn handle_menu(&mut self, key: KeyEvent) {
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

    // ── settings ─────────────────────────────────────────────────────────────

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
                let n = crate::ui::THEMES.len();
                self.settings.theme_idx = if rev {
                    (self.settings.theme_idx + n - 1) % n
                } else {
                    (self.settings.theme_idx + 1) % n
                };
            }
            _ => {}
        }
    }

    fn handle_settings(&mut self, key: KeyEvent) {
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

    // ── test ─────────────────────────────────────────────────────────────────

    /// True when the terminal is below the renderable minimum (see `ui::draw`).
    pub fn too_small(&self) -> bool {
        self.last_width < MIN_WIDTH || self.last_height < MIN_HEIGHT
    }

    fn handle_test(&mut self, key: KeyEvent) {
        // The test screen is hidden behind the too-small overlay; ignore typing
        // so keystrokes aren't recorded blindly. Esc still lets the user leave.
        if self.too_small() && key.code != KeyCode::Esc {
            return;
        }
        match key.code {
            KeyCode::Esc => {
                if self.game.started_at.is_some() {
                    self.test_confirm = true;
                    self.test_confirm_yes = false;
                } else {
                    self.screen = Screen::Menu;
                }
            }
            KeyCode::Tab => self.restart_test(),
            KeyCode::Backspace => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.game.delete_word();
                } else {
                    self.game.backspace();
                }
                self.update_scroll();
            }
            KeyCode::Char(c) => {
                self.game.type_char(c);
                self.update_scroll();
                if self.game.is_finished() {
                    self.save_result();
                    self.screen = Screen::Result;
                } else if let Some(s) = &self.sound {
                    if self.game.last_char_correct() {
                        s.play_correct();
                    } else {
                        s.play_error();
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_result(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') | KeyCode::Char('R') => self.repeat_test(),
            KeyCode::Enter | KeyCode::Tab => self.restart_test(),
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn handle_history(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.history_scroll = 0;
                self.screen = Screen::Menu;
            }
            KeyCode::Up => self.history_scroll = self.history_scroll.saturating_sub(1),
            KeyCode::Down => {
                let max = self.history.len().saturating_sub(1);
                if self.history_scroll < max {
                    self.history_scroll += 1;
                }
            }
            _ => {}
        }
    }

    fn handle_help(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => self.screen = Screen::Menu,
            _ => {}
        }
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
