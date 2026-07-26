//! `App`: all UI state and input handling. `on_key`/`tick` are the only entry
//! points `main.rs` calls each frame; screen-specific key routing lives in
//! the `app::{help,history,menu,result,settings,test}` submodules.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::Instant;
use unicode_width::UnicodeWidthChar;

use crate::config::DEFAULT_VOLUME_PCT;
use crate::game::{GameState, Mode, Settings};
use crate::history::HistoryEntry;
use crate::pb::{self, PersonalBests};
use crate::sound::{SoundPack, SoundPlayer};
use crate::words::{LANGUAGES, LangDef, lang_name, load_quotes_for};

mod help;
mod history;
mod menu;
mod result;
mod settings;
mod test;

pub use settings::SettingsRow;

#[derive(Clone, Copy, PartialEq, Debug)]
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

enum DialogResult {
    /// Dialog is still open (a toggle key, or a key with no effect).
    Open,
    /// Closed via Enter (with the chosen yes/no) or Esc (always `false`).
    Confirmed(bool),
}

/// Shared Left/Right/Tab/Enter/Esc handling for a yes/no confirmation dialog.
fn handle_confirm_dialog(open: &mut bool, yes: &mut bool, key: KeyEvent) -> DialogResult {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => *yes = !*yes,
        KeyCode::Enter => {
            let chosen = *yes;
            *open = false;
            *yes = false;
            return DialogResult::Confirmed(chosen);
        }
        KeyCode::Esc => {
            *open = false;
            *yes = false;
            return DialogResult::Confirmed(false);
        }
        _ => {}
    }
    DialogResult::Open
}

pub struct SettingsState {
    pub cursor: usize,
    pub pending_exit: bool,
    pub snapshot: Option<(Settings, SoundPack, u8)>,
    pub volume_input: Option<String>,
}

/// Put `cursor` at `to` and pull `scroll` so the cursor stays inside the
/// `visible`-row window. Shared by both menu pickers and the History list.
pub(super) fn focus_cursor(cursor: &mut usize, scroll: &mut usize, to: usize, visible: usize) {
    *cursor = to;
    if *cursor < *scroll {
        *scroll = *cursor;
    } else if *cursor >= *scroll + visible {
        *scroll = *cursor + 1 - visible;
    }
}

/// Move `cursor` by one (up if `!down`, down if `down`) within `[0, len)`,
/// keeping it inside the `visible`-row scroll window. Returns whether the
/// cursor actually moved, so callers can reset per-item state (e.g. the
/// language picker's `size_idx`) only when it did.
pub(super) fn step_picker_cursor(
    cursor: &mut usize,
    scroll: &mut usize,
    len: usize,
    visible: usize,
    down: bool,
) -> bool {
    let to = if down {
        if *cursor + 1 >= len {
            return false;
        }
        *cursor + 1
    } else {
        if *cursor == 0 {
            return false;
        }
        *cursor - 1
    };
    focus_cursor(cursor, scroll, to, visible);
    true
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
        let max_size = crate::words::lang_at(lang_idx)
            .sizes
            .len()
            .saturating_sub(1);
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
    pub history_cursor: usize,
    pub help_scroll: usize,
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
        let quotes = load_quotes_for(lang_name(settings.lang_idx));
        let game = GameState::new(cfg.mode, settings.clone(), quotes);
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
            history: crate::history::load_history(),
            history_scroll: 0,
            history_cursor: 0,
            help_scroll: 0,
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
        // NOTE: when no audio device is present we can't read live sound
        // prefs, so fall back to sensible defaults rather than clobbering to Off.
        let (pack, vol) = self
            .sound
            .as_ref()
            .map_or((SoundPack::Click, DEFAULT_VOLUME_PCT), |s| {
                (s.pack, s.volume_pct)
            });
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
            if let DialogResult::Confirmed(true) = handle_confirm_dialog(
                &mut self.dialog.quit_confirm,
                &mut self.dialog.quit_yes,
                key,
            ) {
                self.should_quit = true;
            }
            return;
        }
        if self.dialog.test_confirm {
            if let DialogResult::Confirmed(true) = handle_confirm_dialog(
                &mut self.dialog.test_confirm,
                &mut self.dialog.test_confirm_yes,
                key,
            ) {
                self.screen = Screen::Menu;
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
        let lang = lang_name(self.settings.lang_idx).to_string();
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
            self.sound.as_ref().map_or(SoundPack::Off, |s| s.pack),
            self.sound
                .as_ref()
                .map_or(DEFAULT_VOLUME_PCT, |s| s.volume_pct),
        )
    }

    // ── test ─────────────────────────────────────────────────────────────────

    /// True when the terminal is below the renderable minimum (see `ui::draw`).
    pub fn too_small(&self) -> bool {
        self.last_width < MIN_WIDTH || self.last_height < MIN_HEIGHT
    }

    /// Rows the History overlay can show at once. Mirrors `ui::history`'s
    /// layout (an 80%-height `centered_rect` minus title, summary, header,
    /// footer and their gaps) the same way `update_scroll` mirrors the test
    /// screen's padding. Off-by-one from percentage rounding is harmless —
    /// this only clamps how far the list may scroll.
    pub fn history_visible_rows(&self) -> usize {
        const CHROME_ROWS: usize = 6;
        (self.last_height as usize * 80 / 100).saturating_sub(CHROME_ROWS)
    }

    /// Rows the Help body shows at once, mirroring `ui::help`'s layout (a
    /// 90%-height `centered_rect` minus title, gap and footer) the same way
    /// [`Self::history_visible_rows`] mirrors the History overlay. Only used
    /// to clamp the scroll, so rounding drift is harmless — `ui::help` clamps
    /// again against the rect it actually got.
    pub fn help_visible_rows(&self) -> usize {
        const CHROME_ROWS: usize = 3;
        (self.last_height as usize * 90 / 100)
            .saturating_sub(CHROME_ROWS)
            .max(1)
    }

    /// How far the Help body can scroll before its last line is on screen.
    pub fn help_max_scroll(&self) -> usize {
        crate::ui::help_line_count().saturating_sub(self.help_visible_rows())
    }

    // ── scroll ───────────────────────────────────────────────────────────────

    pub fn update_scroll(&mut self) {
        if self.game.cursor == 0 {
            self.scroll_word = 0;
            return;
        }
        let width = word_block_width(self.last_width) as usize;
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
        let Some(line0_last) = lines.first().and_then(|l| l.last()).copied() else {
            return;
        };
        if cursor_word <= line0_last {
            return;
        }
        // Cursor has moved past the first visible line: scroll to the second.
        if let Some(line1_start) = lines.get(1).and_then(|l| l.first()).copied() {
            self.scroll_word = line1_start;
        }
    }
}

/// Widest the word display is allowed to get. A typing test is read like
/// prose, and much past ~70 columns the sweep back to the start of the next
/// line gets hard to track — on a 200-column terminal the block was 180
/// characters wide, seven lines deep.
pub const WORD_BLOCK_MAX_WIDTH: u16 = 72;

/// Columns the word display occupies on a `term_width`-wide terminal.
///
/// Both `ui::test_screen` (laying the block out) and [`App::update_scroll`]
/// (wrapping the same words to decide when to scroll) need this. They used to
/// carry separate copies of the padding arithmetic, so a change to one
/// silently made the scroll wrap differently from the render.
pub fn word_block_width(term_width: u16) -> u16 {
    let pad = (term_width / 10).clamp(4, 10);
    term_width.saturating_sub(2 * pad).min(WORD_BLOCK_MAX_WIDTH)
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

#[cfg(test)]
mod input_flow_tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn ctrl_c_quits_from_any_screen() {
        let mut app = App::new();
        app.screen = Screen::Test;
        app.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn menu_digit_selects_mode_without_starting_test() {
        let mut app = App::new();
        app.on_key(key(KeyCode::Char('1')));
        assert!(matches!(app.menu.mode, Mode::Time(_)));
        assert_eq!(app.screen, Screen::Menu);
    }

    #[test]
    fn menu_esc_opens_quit_confirm_instead_of_quitting_immediately() {
        let mut app = App::new();
        app.on_key(key(KeyCode::Esc));
        assert!(app.dialog.quit_confirm);
        assert!(!app.should_quit);

        // Default highlighted choice is "no"; Enter dismisses without quitting.
        app.on_key(key(KeyCode::Enter));
        assert!(!app.dialog.quit_confirm);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_esc_without_typing_returns_to_menu_directly() {
        let mut app = App::new();
        app.screen = Screen::Test;
        assert!(app.game.started_at.is_none());
        app.on_key(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Menu);
        assert!(!app.dialog.test_confirm);
    }

    #[test]
    fn test_esc_after_typing_asks_for_confirmation_before_leaving() {
        let mut app = App::new();
        app.screen = Screen::Test;
        let ch = app.game.chars[0].expected;
        app.on_key(key(KeyCode::Char(ch)));
        assert!(app.game.started_at.is_some());

        app.on_key(key(KeyCode::Esc));
        assert!(app.dialog.test_confirm);
        assert_eq!(app.screen, Screen::Test);

        // Toggle to "yes" then confirm; only then does it leave the test.
        app.on_key(key(KeyCode::Left));
        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Menu);
        assert!(!app.dialog.test_confirm);
    }

    fn history_app(entries: usize) -> App {
        let mut app = App::new();
        app.screen = Screen::History;
        app.last_height = 24;
        app.history = (0..entries)
            .map(|_| HistoryEntry {
                wpm: 0.0,
                accuracy: 0.0,
                mode: String::new(),
                timestamp: 0,
                language: String::new(),
            })
            .collect();
        app
    }

    /// Scrolling must stop once the final entry is on screen, instead of
    /// running past it and leaving rows above a blank list.
    #[test]
    fn history_scroll_stops_when_the_last_entry_is_visible() {
        let mut app = history_app(0);
        let visible = app.history_visible_rows();
        assert!(visible > 1, "layout math should leave room for rows");
        app = history_app(visible + 3);
        for _ in 0..50 {
            app.on_key(key(KeyCode::Down));
        }
        assert_eq!(app.history_scroll, 3);
        assert_eq!(app.history_cursor, app.history.len() - 1);
    }

    /// The list used to move a viewport offset with no selected row, so when
    /// everything already fit on screen both arrow keys were dead: nothing
    /// moved and nothing said why. With a selection they always move.
    #[test]
    fn history_arrows_move_the_selection_when_the_list_fits_on_screen() {
        let mut app = history_app(3);
        assert!(
            app.history.len() < app.history_visible_rows(),
            "this test is about a list shorter than the viewport"
        );
        assert_eq!(app.history_scroll, 0);

        app.on_key(key(KeyCode::Down));
        assert_eq!(app.history_cursor, 1);
        app.on_key(key(KeyCode::Down));
        assert_eq!(app.history_cursor, 2);
        // Clamps at the last entry rather than running off the end.
        app.on_key(key(KeyCode::Down));
        assert_eq!(app.history_cursor, 2);
        app.on_key(key(KeyCode::Up));
        assert_eq!(app.history_cursor, 1);
        // The viewport never moved, because it never needed to.
        assert_eq!(app.history_scroll, 0);
    }

    #[test]
    fn history_home_and_end_jump_to_the_ends() {
        let mut app = history_app(40);
        app.on_key(key(KeyCode::End));
        assert_eq!(app.history_cursor, 39);
        assert!(
            app.history_scroll > 0,
            "the viewport should follow the selection to the end"
        );

        app.on_key(key(KeyCode::Home));
        assert_eq!(app.history_cursor, 0);
        assert_eq!(app.history_scroll, 0);
    }

    /// An empty list must not panic or leave the cursor pointing at a row.
    #[test]
    fn history_keys_are_safe_with_no_entries() {
        let mut app = history_app(0);
        for code in [KeyCode::Down, KeyCode::End, KeyCode::PageDown, KeyCode::Up] {
            app.on_key(key(code));
        }
        assert_eq!(app.history_cursor, 0);
        assert_eq!(app.history_scroll, 0);
    }

    /// Stepping onto the custom slot and re-selecting the mode with `1`/`2`
    /// are two separate code paths that must resolve the slot the same way —
    /// to the stored custom value, not to a preset.
    #[test]
    fn custom_slot_resolves_to_the_stored_value_from_both_paths() {
        let mut app = App::new();
        app.menu.custom_time_val = 45;
        app.menu.custom_words_val = 75;

        app.menu.time_idx = 0;
        app.menu.mode = Mode::Time(TIME_OPTIONS[0]);
        for _ in 0..TIME_OPTIONS.len() {
            app.on_key(key(KeyCode::Right));
        }
        assert!(app.is_custom_slot());
        assert_eq!(app.menu.mode, Mode::Time(45));
        app.on_key(key(KeyCode::Char('1')));
        assert_eq!(app.menu.mode, Mode::Time(45));

        app.menu.word_idx = 0;
        app.menu.mode = Mode::Words(WORD_OPTIONS[0]);
        for _ in 0..WORD_OPTIONS.len() {
            app.on_key(key(KeyCode::Right));
        }
        assert!(app.is_custom_slot());
        assert_eq!(app.menu.mode, Mode::Words(75));
        app.on_key(key(KeyCode::Char('2')));
        assert_eq!(app.menu.mode, Mode::Words(75));
    }

    /// The menu renders the word-pool size with the same "selectable" styling
    /// as the option row above it, but for a long time no menu key moved it —
    /// it could only be changed from inside the language picker.
    #[test]
    fn bracket_keys_step_the_word_pool_size_within_range() {
        let mut app = App::new();
        // english has default/1k/5k/10k, so there is something to step through.
        app.settings.lang_idx = 0;
        app.settings.size_idx = 0;
        let last = crate::words::lang_at(0).sizes.len() - 1;
        assert!(last > 0, "test needs a multi-size language");

        app.on_key(key(KeyCode::Char(']')));
        assert_eq!(app.settings.size_idx, 1);
        app.on_key(key(KeyCode::Char('[')));
        assert_eq!(app.settings.size_idx, 0);

        // Both ends clamp rather than wrapping or running past the list.
        app.on_key(key(KeyCode::Char('[')));
        assert_eq!(app.settings.size_idx, 0);
        for _ in 0..20 {
            app.on_key(key(KeyCode::Char(']')));
        }
        assert_eq!(app.settings.size_idx, last);
    }

    /// Opens the custom-time input slot (the entry past the last preset in
    /// `TIME_OPTIONS`) the same way `Enter`/`Tab` would in the real menu.
    fn open_custom_time_input(app: &mut App) {
        app.menu.time_idx = TIME_OPTIONS.len();
        app.menu.mode = Mode::Time(app.menu.custom_time_val);
        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.menu.custom_input.as_deref(), Some(""));
    }

    #[test]
    fn custom_input_ignores_non_digits_and_caps_at_max_len() {
        let mut app = App::new();
        open_custom_time_input(&mut app);
        for c in "1a2b3c4d5e6f".chars() {
            app.on_key(key(KeyCode::Char(c)));
        }
        // CUSTOM_INPUT_MAX_LEN is 5: digits 1-5 are kept, '6' is dropped, letters ignored.
        assert_eq!(app.menu.custom_input.as_deref(), Some("12345"));
    }

    #[test]
    fn custom_input_backspace_removes_last_char() {
        let mut app = App::new();
        open_custom_time_input(&mut app);
        app.on_key(key(KeyCode::Char('9')));
        app.on_key(key(KeyCode::Char('0')));
        app.on_key(key(KeyCode::Backspace));
        assert_eq!(app.menu.custom_input.as_deref(), Some("9"));
    }

    #[test]
    fn custom_time_input_clamps_to_max_on_enter() {
        let mut app = App::new();
        open_custom_time_input(&mut app);
        for c in "99999".chars() {
            app.on_key(key(KeyCode::Char(c)));
        }
        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.menu.mode, Mode::Time(3600));
        assert_eq!(app.screen, Screen::Test);
    }

    #[test]
    fn custom_words_input_clamps_to_max_on_enter() {
        let mut app = App::new();
        app.menu.word_idx = WORD_OPTIONS.len();
        app.menu.mode = Mode::Words(app.menu.custom_words_val);
        app.on_key(key(KeyCode::Enter));
        for c in "99999".chars() {
            app.on_key(key(KeyCode::Char(c)));
        }
        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.menu.mode, Mode::Words(5000));
        assert_eq!(app.screen, Screen::Test);
    }

    #[test]
    fn custom_input_empty_enter_does_not_start_test() {
        let mut app = App::new();
        open_custom_time_input(&mut app);
        app.on_key(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Menu);
        assert_eq!(app.menu.custom_input.as_deref(), Some(""));
    }

    #[test]
    fn custom_input_esc_cancels_without_starting_test() {
        let mut app = App::new();
        open_custom_time_input(&mut app);
        app.on_key(key(KeyCode::Char('5')));
        app.on_key(key(KeyCode::Esc));
        assert!(app.menu.custom_input.is_none());
        assert_eq!(app.screen, Screen::Menu);
    }
}
