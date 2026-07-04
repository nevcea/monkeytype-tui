use rand::prelude::IndexedRandom;
use rand::{Rng, RngExt};
use std::time::{Duration, Instant};

use crate::history::HistoryExpiry;
use crate::words::QuoteEntry;
use crate::words::load_words;

/// Standard WPM word length (monkeytype convention).
const CHARS_PER_WORD: f64 = 5.0;
/// Floor for a sampling interval to avoid divide-by-near-zero WPM spikes.
const MIN_SAMPLE_INTERVAL: f64 = 0.001;
/// Keystroke gaps longer than this (seconds) are counted as idle/AFK.
const AFK_THRESHOLD_SECS: f64 = 2.0;
/// Accuracy below this percentage fails the test.
const FAIL_ACCURACY: f64 = 75.0;
/// Word pool size sampled for time-based tests.
const TIME_MODE_POOL: usize = 500;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CharState {
    Correct,
    Wrong,
    Current,
    Pending,
}

#[derive(Clone)]
pub struct TypedChar {
    pub expected: char,
    pub typed: Option<char>,
    pub state: CharState,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Time(u64),
    Words(usize),
    Quote,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Time(s) => write!(f, "time {s}s"),
            Mode::Words(n) => write!(f, "words {n}"),
            Mode::Quote => write!(f, "quote"),
        }
    }
}

cycle_enum! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum CursorShape {
        Bar = "bar",
        Block = "block",
        Underline = "underline",
    }
    default = Bar;
}

cycle_enum! {
    #[derive(Clone, Copy, PartialEq)]
    pub enum QuoteFilter {
        All = "all",
        Short = "short",   // ≤ 100 chars
        Medium = "medium", // 101–300
        Long = "long",     // 301–600
        Thicc = "thicc",   // 601+
    }
    default = All;
}

impl QuoteFilter {
    pub fn matches(self, len: u64) -> bool {
        match self {
            Self::All => true,
            Self::Short => len <= 100,
            Self::Medium => (101..=300).contains(&len),
            Self::Long => (301..=600).contains(&len),
            Self::Thicc => len >= 601,
        }
    }
}

cycle_enum! {
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum Difficulty {
        Normal = "normal",
        Expert = "expert",
        Master = "master",
    }
    default = Normal;
}

#[derive(Clone, Default, PartialEq)]
pub struct Settings {
    pub punctuation: bool,
    pub numbers: bool,
    pub lang_idx: usize,
    pub size_idx: usize,
    pub cursor_shape: CursorShape,
    pub history_expiry: HistoryExpiry,
    pub difficulty: Difficulty,
    pub quote_filter: QuoteFilter,
}

pub struct GameState {
    pub mode: Mode,
    pub settings: Settings,
    pub words: Vec<String>,
    pub word_starts: Vec<usize>,
    pub chars: Vec<TypedChar>,
    pub cursor: usize,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
    pub total_keystrokes: usize,
    pub error_keystrokes: usize,
    pub quote_source: Option<String>,
    pub wpm_samples: Vec<f64>,
    pub raw_wpm_samples: Vec<f64>,
    pub error_samples: Vec<usize>,
    pub afk_secs: f64,
    pub difficulty_failed: bool,
    last_sample_secs: u64,
    last_sample_errors: usize,
    last_sample_correct: usize,
    last_sample_cursor: usize,
    last_sample_elapsed: f64,
    last_keystroke: Option<Instant>,
    last_quote_idx: Option<usize>,
    all_words: std::sync::Arc<Vec<String>>,
    pub all_quotes: Vec<QuoteEntry>,
}

impl GameState {
    pub fn new(mode: Mode, settings: Settings, quotes: Vec<QuoteEntry>) -> Self {
        let all_words = load_words(settings.lang_idx, settings.size_idx);
        let mut state = Self {
            mode,
            settings,
            words: vec![],
            word_starts: vec![],
            chars: vec![],
            cursor: 0,
            started_at: None,
            finished_at: None,
            total_keystrokes: 0,
            error_keystrokes: 0,
            quote_source: None,
            wpm_samples: vec![],
            raw_wpm_samples: vec![],
            error_samples: vec![],
            afk_secs: 0.0,
            difficulty_failed: false,
            last_sample_secs: 0,
            last_sample_errors: 0,
            last_sample_correct: 0,
            last_sample_cursor: 0,
            last_sample_elapsed: 0.0,
            last_keystroke: None,
            last_quote_idx: None,
            all_words,
            all_quotes: quotes,
        };
        state.reset();
        state
    }

    pub fn reset(&mut self) {
        let mut rng = rand::rng();
        let (raw_words, source) = self.pick_words(&mut rng);
        // Punctuation / numbers decorate generated word/time modes only, not quotes.
        let words = match source {
            Some(_) => raw_words,
            None => self.decorate_words(raw_words, &mut rng),
        };

        self.quote_source = source;
        self.words = words;
        self.chars = words_to_chars(&self.words);
        self.word_starts = compute_word_starts(&self.words);
        self.reset_counters();
    }

    /// Produce the raw word list for the current mode plus the quote source
    /// (`Some` only in `Mode::Quote`).
    fn pick_words(&mut self, rng: &mut impl Rng) -> (Vec<String>, Option<String>) {
        match self.mode {
            Mode::Time(_) => (
                self.all_words
                    .sample(rng, TIME_MODE_POOL)
                    .cloned()
                    .collect(),
                None,
            ),
            Mode::Words(n) => (self.all_words.sample(rng, n).cloned().collect(), None),
            Mode::Quote => self.pick_quote(rng),
        }
    }

    fn pick_quote(&mut self, rng: &mut impl Rng) -> (Vec<String>, Option<String>) {
        let filter = self.settings.quote_filter;
        let mut pool: Vec<usize> = self
            .all_quotes
            .iter()
            .enumerate()
            .filter(|(_, q)| filter.matches(q.length))
            .map(|(i, _)| i)
            .collect();
        // Fall back to the full list if the filter matched nothing.
        if pool.is_empty() {
            pool = (0..self.all_quotes.len()).collect();
        }
        if pool.is_empty() {
            return (vec!["no quotes loaded".to_string()], None);
        }

        let mut pos = rng.random_range(0..pool.len());
        // Avoid repeating the previous quote when there's a choice.
        while pool.len() > 1 && Some(pool[pos]) == self.last_quote_idx {
            pos = rng.random_range(0..pool.len());
        }
        let idx = pool[pos];
        self.last_quote_idx = Some(idx);
        let q = &self.all_quotes[idx];
        let words = q.text.split_whitespace().map(String::from).collect();
        (words, Some(q.source.clone()))
    }

    fn decorate_words(&self, mut w: Vec<String>, rng: &mut impl Rng) -> Vec<String> {
        if self.settings.numbers {
            w = apply_numbers(w, rng);
        }
        if self.settings.punctuation {
            w = apply_punctuation(w, rng);
        }
        w
    }

    pub fn type_char(&mut self, c: char) {
        if self.is_finished() || self.cursor >= self.chars.len() {
            return;
        }
        let now = Instant::now();
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
        // accumulate AFK: gaps > 2s between keystrokes count as idle
        if let Some(last) = self.last_keystroke {
            let gap = now.duration_since(last).as_secs_f64();
            if gap > AFK_THRESHOLD_SECS {
                self.afk_secs += gap;
            }
        }
        self.last_keystroke = Some(now);
        self.total_keystrokes += 1;

        let expected = self.chars[self.cursor].expected;
        self.chars[self.cursor].typed = Some(c);
        let correct = c == expected;
        self.chars[self.cursor].state = if correct {
            CharState::Correct
        } else {
            self.error_keystrokes += 1;
            CharState::Wrong
        };

        // Difficulty checks before advancing cursor
        match self.settings.difficulty {
            Difficulty::Expert if !correct => {
                self.difficulty_fail();
                return;
            }
            Difficulty::Master if correct && expected == ' ' => {
                // Typed space correctly: check if the completed word had any errors
                if self.word_starts.is_empty() {
                    return;
                }
                let w = self.word_at_cursor();
                let word_start = self.word_starts[w];
                let has_error = self.chars[word_start..self.cursor]
                    .iter()
                    .any(|c| c.state == CharState::Wrong);
                if has_error {
                    self.difficulty_fail();
                    return;
                }
            }
            _ => {}
        }

        self.cursor += 1;
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].state = CharState::Current;
        }
        self.check_finished();
    }

    pub fn delete_word(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if self.word_starts.is_empty() {
            return;
        }
        let w = self.word_at_cursor();
        let word_start = self.word_starts[w];
        let target = if self.cursor <= word_start && w > 0 {
            self.word_starts[w - 1]
        } else {
            word_start
        };
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].state = CharState::Pending;
        }
        for i in target..self.cursor {
            self.chars[i].state = CharState::Pending;
            self.chars[i].typed = None;
        }
        self.cursor = target;
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].state = CharState::Current;
        }
    }

    pub fn repeat(&mut self) {
        self.chars = words_to_chars(&self.words);
        self.word_starts = compute_word_starts(&self.words);
        self.reset_counters();
    }

    fn reset_counters(&mut self) {
        self.cursor = 0;
        self.started_at = None;
        self.finished_at = None;
        self.total_keystrokes = 0;
        self.error_keystrokes = 0;
        self.wpm_samples.clear();
        self.raw_wpm_samples.clear();
        self.error_samples.clear();
        self.afk_secs = 0.0;
        self.difficulty_failed = false;
        self.last_sample_secs = 0;
        self.last_sample_errors = 0;
        self.last_sample_correct = 0;
        self.last_sample_cursor = 0;
        self.last_sample_elapsed = 0.0;
        self.last_keystroke = None;
        if !self.chars.is_empty() {
            self.chars[0].state = CharState::Current;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].state = CharState::Pending;
        }
        self.cursor -= 1;
        self.chars[self.cursor].state = CharState::Current;
        self.chars[self.cursor].typed = None;
    }

    fn push_sample(&mut self) {
        // Sample the *instantaneous* (per-interval) WPM, not the cumulative
        // average. Cumulative samples are inherently smooth and inflate
        // consistency; per-interval samples match monkeytype's per-second bursts.
        let elapsed = self.elapsed().as_secs_f64();
        let interval = (elapsed - self.last_sample_elapsed).max(MIN_SAMPLE_INTERVAL);
        let correct = self.correct_chars();
        let d_correct = correct.saturating_sub(self.last_sample_correct);
        let d_total = self.cursor.saturating_sub(self.last_sample_cursor);
        self.wpm_samples
            .push((d_correct as f64 / CHARS_PER_WORD) / (interval / 60.0));
        self.raw_wpm_samples
            .push((d_total as f64 / CHARS_PER_WORD) / (interval / 60.0));
        self.error_samples
            .push(self.error_keystrokes - self.last_sample_errors);
        self.last_sample_errors = self.error_keystrokes;
        self.last_sample_correct = correct;
        self.last_sample_cursor = self.cursor;
        self.last_sample_elapsed = elapsed;
    }

    pub fn tick(&mut self) {
        if self.is_finished() {
            return;
        }
        if let Mode::Time(_) = self.mode {
            self.check_finished();
        }
        if let Some(start) = self.started_at {
            let secs = start.elapsed().as_secs();
            if secs > self.last_sample_secs {
                self.last_sample_secs = secs;
                self.push_sample();
            }
        }
    }

    fn check_finished(&mut self) {
        if self.finished_at.is_some() {
            return;
        }
        let done = match self.mode {
            Mode::Words(_) | Mode::Quote => self.cursor >= self.chars.len(),
            Mode::Time(secs) => self
                .started_at
                .map(|t| t.elapsed() >= Duration::from_secs(secs))
                .unwrap_or(false),
        };
        if done {
            self.finished_at = Some(Instant::now());
            self.push_sample();
            // ponytail: u64::MAX sentinel blocks tick from pushing a duplicate final sample. ceiling: breaks if tick rate ever exceeds ~584 billion years; upgrade: use an Option<u64> if last_sample_secs becomes Optional anyway
            self.last_sample_secs = u64::MAX;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        match (self.started_at, self.finished_at) {
            (Some(s), Some(e)) => e.duration_since(s),
            (Some(s), None) => s.elapsed(),
            _ => Duration::ZERO,
        }
    }

    pub fn time_left(&self) -> u64 {
        if let Mode::Time(secs) = self.mode {
            let elapsed = self.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
            secs.saturating_sub(elapsed)
        } else {
            0
        }
    }

    fn correct_chars(&self) -> usize {
        self.chars
            .iter()
            .take(self.cursor)
            .filter(|c| c.state == CharState::Correct)
            .count()
    }

    pub fn wpm(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs < 0.1 {
            return 0.0;
        }
        (self.correct_chars() as f64 / CHARS_PER_WORD) / (secs / 60.0)
    }

    pub fn raw_wpm(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs < 0.1 {
            return 0.0;
        }
        (self.cursor as f64 / CHARS_PER_WORD) / (secs / 60.0)
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_keystrokes == 0 {
            return 100.0;
        }
        ((self.total_keystrokes - self.error_keystrokes) as f64 / self.total_keystrokes as f64)
            * 100.0
    }

    pub fn is_failed(&self) -> bool {
        self.difficulty_failed || self.accuracy() < FAIL_ACCURACY
    }

    pub fn fail_reason(&self) -> Option<&'static str> {
        if self.difficulty_failed {
            Some("difficulty")
        } else if self.accuracy() < FAIL_ACCURACY {
            Some("accuracy")
        } else {
            None
        }
    }

    fn difficulty_fail(&mut self) {
        self.difficulty_failed = true;
        self.finished_at = Some(Instant::now());
        self.push_sample();
        self.last_sample_secs = u64::MAX;
    }

    pub fn consistency(&self) -> f64 {
        let n = self.wpm_samples.len();
        if n < 2 {
            return 100.0;
        }
        let mean = self.wpm_samples.iter().sum::<f64>() / n as f64;
        if mean == 0.0 {
            return 100.0;
        }
        let var = self
            .wpm_samples
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / n as f64;
        (100.0 - (var.sqrt() / mean * 100.0)).clamp(0.0, 100.0)
    }

    pub fn last_char_correct(&self) -> bool {
        self.cursor > 0
            && self
                .chars
                .get(self.cursor - 1)
                .is_some_and(|c| c.state == CharState::Correct)
    }

    pub fn word_at_cursor(&self) -> usize {
        match self.word_starts.partition_point(|&s| s <= self.cursor) {
            0 => 0,
            n => n - 1,
        }
    }

    pub fn words_typed(&self) -> usize {
        self.chars
            .iter()
            .take(self.cursor)
            .filter(|c| c.expected == ' ')
            .count()
    }
}

fn compute_word_starts(words: &[String]) -> Vec<usize> {
    words
        .iter()
        .scan(0usize, |pos, w| {
            let start = *pos;
            *pos += w.chars().count() + 1;
            Some(start)
        })
        .collect()
}

fn words_to_chars(words: &[String]) -> Vec<TypedChar> {
    let pending = |ch| TypedChar {
        expected: ch,
        typed: None,
        state: CharState::Pending,
    };
    words
        .iter()
        .enumerate()
        .flat_map(|(i, w)| {
            let space = (i + 1 < words.len()).then(|| pending(' '));
            w.chars().map(pending).chain(space)
        })
        .collect()
}

const PUNCT_SENTENCE_LEN: usize = 4;

fn apply_punctuation(words: Vec<String>, rng: &mut impl Rng) -> Vec<String> {
    let endings = ['.', '!', '?'];
    let n = words.len();
    let mut result = Vec::with_capacity(n);
    let mut since_sentence = 0usize;
    let mut cap_next = false;

    for (i, word) in words.into_iter().enumerate() {
        let word = if cap_next {
            cap_next = false;
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => word,
            }
        } else {
            word
        };

        since_sentence += 1;
        if i + 1 < n && since_sentence >= PUNCT_SENTENCE_LEN && rng.random_ratio(1, 3) {
            let end = endings[rng.random_range(0..endings.len())];
            result.push(format!("{word}{end}"));
            since_sentence = 0;
            cap_next = true;
        } else if i + 1 < n && rng.random_ratio(1, 4) {
            result.push(format!("{word},"));
        } else {
            result.push(word);
        }
    }
    result
}

fn apply_numbers(words: Vec<String>, rng: &mut impl Rng) -> Vec<String> {
    words
        .into_iter()
        .map(|w| {
            if rng.random_ratio(1, 7) {
                rng.random_range(0u32..1000).to_string()
            } else {
                w
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> GameState {
        GameState::new(Mode::Words(10), Settings::default(), vec![])
    }

    #[test]
    fn wpm_zero_before_start() {
        assert_eq!(game().wpm(), 0.0);
    }

    #[test]
    fn accuracy_full_when_no_keystrokes() {
        assert_eq!(game().accuracy(), 100.0);
    }

    #[test]
    fn word_starts_correct() {
        let words = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(compute_word_starts(&words), vec![0, 6]);
    }

    #[test]
    fn backspace_restores_state() {
        let mut g = game();
        let expected = g.chars[0].expected;
        g.type_char(expected);
        assert_eq!(g.cursor, 1);
        g.backspace();
        assert_eq!(g.cursor, 0);
        assert_eq!(g.chars[0].state, CharState::Current);
    }

    #[test]
    fn type_char_correct_marks_correct() {
        let mut g = game();
        let ch = g.chars[0].expected;
        g.type_char(ch);
        assert_eq!(g.chars[0].state, CharState::Correct);
    }

    #[test]
    fn type_char_wrong_marks_wrong() {
        let mut g = game();
        let expected = g.chars[0].expected;
        let wrong = if expected == 'a' { 'z' } else { 'a' };
        g.type_char(wrong);
        assert_eq!(g.chars[0].state, CharState::Wrong);
    }

    #[test]
    fn type_char_starts_game() {
        let mut g = game();
        assert!(g.started_at.is_none());
        let ch = g.chars[0].expected;
        g.type_char(ch);
        assert!(g.started_at.is_some());
    }

    #[test]
    fn accuracy_with_errors() {
        let mut g = game();
        for i in 0..3 {
            let ch = g.chars[i].expected;
            g.type_char(ch);
        }
        for i in 3..5 {
            let wrong = if g.chars[i].expected == 'z' { 'a' } else { 'z' };
            g.type_char(wrong);
        }
        assert_eq!(g.accuracy(), 60.0);
    }

    #[test]
    fn words_typed_counts_spaces() {
        let mut g = game();
        let space_idx = g.chars.iter().position(|c| c.expected == ' ').unwrap();
        for i in 0..=space_idx {
            let ch = g.chars[i].expected;
            g.type_char(ch);
        }
        assert_eq!(g.words_typed(), 1);
    }

    #[test]
    fn word_at_cursor_in_second_word() {
        let mut g = game();
        let space_idx = g.chars.iter().position(|c| c.expected == ' ').unwrap();
        for i in 0..=space_idx {
            let ch = g.chars[i].expected;
            g.type_char(ch);
        }
        assert_eq!(g.word_at_cursor(), 1);
    }

    #[test]
    fn delete_word_resets_to_word_start() {
        let mut g = game();
        for i in 0..3 {
            let ch = g.chars[i].expected;
            g.type_char(ch);
        }
        assert_eq!(g.cursor, 3);
        g.delete_word();
        assert_eq!(g.cursor, 0);
        assert_eq!(g.chars[0].state, CharState::Current);
    }

    #[test]
    fn words_to_chars_includes_spaces() {
        let words = vec!["a".to_string(), "b".to_string()];
        let chars = words_to_chars(&words);
        let expected: Vec<char> = chars.iter().map(|c| c.expected).collect();
        assert_eq!(expected, vec!['a', ' ', 'b']);
    }

    #[test]
    fn wpm_counts_correct_chars_over_minutes() {
        let mut g = game();
        // Ten correct chars = two 5-char words.
        for i in 0..10 {
            let ch = g.chars[i].expected;
            g.type_char(ch);
        }
        // Freeze elapsed at exactly one minute so timing is deterministic.
        let start = Instant::now();
        g.started_at = Some(start);
        g.finished_at = Some(start + Duration::from_secs(60));
        assert!((g.wpm() - 2.0).abs() < 1e-9);
        assert!((g.raw_wpm() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn expert_difficulty_fails_on_wrong_char() {
        let mut g = GameState::new(
            Mode::Words(10),
            Settings {
                difficulty: Difficulty::Expert,
                ..Settings::default()
            },
            vec![],
        );
        let wrong = if g.chars[0].expected == 'z' { 'a' } else { 'z' };
        g.type_char(wrong);
        assert!(g.difficulty_failed);
        assert!(g.is_failed());
        assert_eq!(g.fail_reason(), Some("difficulty"));
    }

    #[test]
    fn master_difficulty_fails_on_wrong_char_before_space() {
        let mut g = GameState::new(
            Mode::Words(10),
            Settings {
                difficulty: Difficulty::Master,
                ..Settings::default()
            },
            vec![],
        );
        let space_idx = g.chars.iter().position(|c| c.expected == ' ').unwrap();
        // Type the first word with one deliberate error, then the space.
        for i in 0..space_idx {
            let wrong = if g.chars[i].expected == 'z' { 'a' } else { 'z' };
            g.type_char(wrong);
        }
        g.type_char(' ');
        assert!(g.difficulty_failed);
    }

    #[test]
    fn quote_filter_length_buckets() {
        assert!(QuoteFilter::Short.matches(100));
        assert!(!QuoteFilter::Short.matches(101));
        assert!(QuoteFilter::Medium.matches(101));
        assert!(QuoteFilter::Medium.matches(300));
        assert!(QuoteFilter::Long.matches(301));
        assert!(QuoteFilter::Thicc.matches(601));
        assert!(QuoteFilter::All.matches(9999));
    }

    #[test]
    fn cycle_enum_wraps_both_directions() {
        assert_eq!(Difficulty::Normal.prev(), Difficulty::Master);
        assert_eq!(Difficulty::Master.next(), Difficulty::Normal);
        assert!(CursorShape::default() == CursorShape::Bar);
        assert!(CursorShape::Bar.prev() == CursorShape::Underline);
    }
}
