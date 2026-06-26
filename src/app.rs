use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::game::{GameState, Mode, Settings};
use crate::history::{self, HistoryEntry};
use crate::quotes::{load_quotes_for, QuoteEntry};
use crate::words::LANGUAGES;

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Menu,
    Test,
    Result,
    History,
    Help,
}

#[derive(Clone, Copy, PartialEq)]
pub enum MenuMode {
    Time,
    Words,
    Quote,
}

pub const TIME_OPTIONS: &[u64] = &[5, 15, 30, 60, 120, 180];
pub const WORD_OPTIONS: &[usize] = &[10, 25, 50, 100, 200, 500];

pub struct LangPicker {
    pub cursor: usize,   // index into filtered list
    pub size_idx: usize,
    pub scroll: usize,
    pub search: String,
}

impl LangPicker {
    fn new(lang_idx: usize, size_idx: usize) -> Self {
        Self { cursor: lang_idx, size_idx, scroll: lang_idx.saturating_sub(4), search: String::new() }
    }
}

pub struct App {
    pub screen: Screen,
    pub game: GameState,
    pub settings: Settings,
    pub menu_mode: MenuMode,
    pub menu_time_idx: usize,   // index into TIME_OPTIONS; == len() → custom
    pub menu_word_idx: usize,   // index into WORD_OPTIONS; == len() → custom
    pub custom_input: Option<String>,
    pub custom_time_val: u64,
    pub custom_words_val: usize,
    pub lang_picker: Option<LangPicker>,
    pub scroll_word: usize,
    pub last_width: u16,
    pub history: Vec<HistoryEntry>,
    pub should_quit: bool,
    result_saved: bool,
    all_quotes: Vec<QuoteEntry>,
}

impl App {
    pub fn new() -> Self {
        let settings = Settings::default();
        let all_quotes = load_quotes_for("english");
        Self {
            screen: Screen::Menu,
            game: GameState::new(Mode::Time(30), settings.clone(), all_quotes.clone()),
            settings,
            menu_mode: MenuMode::Time,
            menu_time_idx: 2, // 30s
            menu_word_idx: 2, // 50 words
            custom_input: None,
            custom_time_val: 45,
            custom_words_val: 75,
            lang_picker: None,
            scroll_word: 0,
            last_width: 80,
            history: history::load_history(),
            should_quit: false,
            result_saved: false,
            all_quotes,
        }
    }

    pub fn current_mode(&self) -> Mode {
        match self.menu_mode {
            MenuMode::Time => {
                let secs = TIME_OPTIONS.get(self.menu_time_idx).copied()
                    .unwrap_or(self.custom_time_val);
                Mode::Time(secs)
            }
            MenuMode::Words => {
                let n = WORD_OPTIONS.get(self.menu_word_idx).copied()
                    .unwrap_or(self.custom_words_val);
                Mode::Words(n)
            }
            MenuMode::Quote => Mode::Quote,
        }
    }

    pub fn is_custom_slot(&self) -> bool {
        match self.menu_mode {
            MenuMode::Time  => self.menu_time_idx == TIME_OPTIONS.len(),
            MenuMode::Words => self.menu_word_idx == WORD_OPTIONS.len(),
            MenuMode::Quote => false,
        }
    }

    fn start_test(&mut self) {
        let mode = self.current_mode();
        self.game = GameState::new(mode, self.settings.clone(), self.all_quotes.clone());
        self.scroll_word = 0;
        self.result_saved = false;
        self.screen = Screen::Test;
    }

    fn restart_test(&mut self) {
        self.game.settings = self.settings.clone();
        self.game.reset();
        self.scroll_word = 0;
        self.result_saved = false;
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        match self.screen {
            Screen::Menu    => self.handle_menu(key),
            Screen::Test    => self.handle_test(key),
            Screen::Result  => self.handle_result(key),
            Screen::History => self.handle_history(key),
            Screen::Help    => self.handle_help(key),
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
        if self.result_saved { return; }
        self.result_saved = true;
        let mode_str = match self.game.mode {
            Mode::Time(s)  => format!("time {s}s"),
            Mode::Words(n) => format!("words {n}"),
            Mode::Quote    => "quote".to_string(),
        };
        let entry = HistoryEntry {
            wpm: self.game.wpm(),
            accuracy: self.game.accuracy(),
            mode: mode_str,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        history::save_entry(entry.clone(), &self.history);
        self.history.insert(0, entry);
        self.history.truncate(50);
    }

    // ── menu ─────────────────────────────────────────────────────────────────

    fn handle_menu(&mut self, key: KeyEvent) {
        // ── lang picker overlay ──────────────────────────────────────────────
        if let Some(ref mut picker) = self.lang_picker {
            const VISIBLE: usize = 10;
            let filtered: Vec<(usize, &_)> = LANGUAGES.iter().enumerate()
                .filter(|(_, l)| l.name.to_lowercase().contains(&picker.search.to_lowercase()))
                .collect();
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
                        if picker.cursor >= picker.scroll + VISIBLE {
                            picker.scroll = picker.cursor + 1 - VISIBLE;
                        }
                    }
                }
                KeyCode::Left => {
                    if picker.size_idx > 0 {
                        picker.size_idx -= 1;
                    }
                }
                KeyCode::Right => {
                    let max = filtered.get(picker.cursor)
                        .map(|(_, l)| l.sizes.len().saturating_sub(1))
                        .unwrap_or(0);
                    if picker.size_idx < max {
                        picker.size_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(&(real_idx, lang)) = filtered.get(picker.cursor) {
                        self.settings.lang_idx = real_idx;
                        self.settings.size_idx = picker.size_idx;
                        self.all_quotes = load_quotes_for(lang.name);
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
            return;
        }

        // ── custom value input ───────────────────────────────────────────────
        if self.custom_input.is_some() {
            match key.code {
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let s = self.custom_input.as_mut().unwrap();
                    if s.len() < 5 { s.push(c); }
                }
                KeyCode::Backspace => { self.custom_input.as_mut().unwrap().pop(); }
                KeyCode::Enter => {
                    let s = self.custom_input.as_deref().unwrap_or("");
                    if s.is_empty() { return; }
                    let val: u64 = self.custom_input.take().unwrap_or_default()
                        .parse().unwrap_or(0);
                    match self.menu_mode {
                        MenuMode::Time  => self.custom_time_val  = val.clamp(1, 3600),
                        MenuMode::Words => self.custom_words_val = (val as usize).clamp(1, 5000),
                        _ => {}
                    }
                    self.start_test();
                }
                KeyCode::Esc => { self.custom_input = None; }
                _ => {}
            }
            return;
        }

        // ── normal menu ──────────────────────────────────────────────────────
        match key.code {
            // Mode selection with number keys
            KeyCode::Char('1') => self.menu_mode = MenuMode::Time,
            KeyCode::Char('2') => self.menu_mode = MenuMode::Words,
            KeyCode::Char('3') => self.menu_mode = MenuMode::Quote,

            // Option value with ←/→  (more natural for horizontal list)
            KeyCode::Left => match self.menu_mode {
                MenuMode::Time  => self.menu_time_idx = self.menu_time_idx.saturating_sub(1),
                MenuMode::Words => self.menu_word_idx = self.menu_word_idx.saturating_sub(1),
                MenuMode::Quote => {}
            },
            KeyCode::Right => match self.menu_mode {
                MenuMode::Time  => {
                    if self.menu_time_idx < TIME_OPTIONS.len() { self.menu_time_idx += 1; }
                }
                MenuMode::Words => {
                    if self.menu_word_idx < WORD_OPTIONS.len() { self.menu_word_idx += 1; }
                }
                MenuMode::Quote => {}
            },

            // Start
            KeyCode::Enter => {
                if self.is_custom_slot() {
                    self.custom_input = Some(String::new());
                } else {
                    self.start_test();
                }
            }

            // Settings toggles
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.settings.punctuation = !self.settings.punctuation;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.settings.numbers = !self.settings.numbers;
            }

            // Language picker
            KeyCode::Char('l') | KeyCode::Char('L') => {
                self.lang_picker = Some(LangPicker::new(
                    self.settings.lang_idx,
                    self.settings.size_idx,
                ));
            }

            // Other screens
            KeyCode::Char('h') | KeyCode::Char('H') => self.screen = Screen::History,
            KeyCode::Char('?') => self.screen = Screen::Help,
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            _ => {}
        }
    }

    // ── test ─────────────────────────────────────────────────────────────────

    fn handle_test(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc       => self.screen = Screen::Menu,
            KeyCode::Tab       => self.restart_test(),
            KeyCode::Backspace => {
                self.game.backspace();
                self.update_scroll();
            }
            KeyCode::Char(c) => {
                self.game.type_char(c);
                self.update_scroll();
                if self.game.is_finished() {
                    self.save_result();
                    self.screen = Screen::Result;
                }
            }
            _ => {}
        }
    }

    fn handle_result(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => { self.restart_test(); self.screen = Screen::Test; }
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }

    fn handle_history(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => self.screen = Screen::Menu,
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
        if self.game.cursor == 0 { self.scroll_word = 0; return; }
        let width = (self.last_width as usize).saturating_sub(4);
        if width < 10 { return; }
        let cursor_word = self.game.word_at_cursor();
        if cursor_word < self.scroll_word { self.scroll_word = 0; return; }
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
        let wlen = word.chars().count();
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
    if !line.is_empty() { lines.push(line); }
    lines
}
