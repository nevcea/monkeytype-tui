//! Pure typing-test state machine: word generation, keystroke handling, and
//! WPM/accuracy/consistency scoring. No I/O and no dependency on `ui` — the
//! same `GameState` drives both the terminal renderer and the unit tests.

use rand::prelude::IndexedRandom;
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::words::QuoteEntry;
use crate::words::load_words;

/// Standard WPM word length (monkeytype convention).
const CHARS_PER_WORD: f64 = 5.0;
/// Floor for a sampling interval to avoid divide-by-near-zero WPM spikes.
const MIN_SAMPLE_INTERVAL: f64 = 0.001;
/// Shortest interval that counts as a real sampling window; see [`GameState::push_sample`].
const MIN_REAL_INTERVAL: f64 = 0.5;
/// Keystroke gaps longer than this (seconds) are counted as idle/AFK.
const AFK_THRESHOLD_SECS: f64 = 2.0;
/// Accuracy below this percentage fails the test.
const FAIL_ACCURACY: f64 = 75.0;
/// Minimum word pool size sampled for time-based tests.
const TIME_MODE_POOL: usize = 500;
/// Estimated words needed per second of a time-based test (well above any
/// realistic typing speed), so the pool outlasts the timer.
const TIME_MODE_WORDS_PER_SEC: usize = 4;

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

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
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
    #[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
    pub enum CursorShape {
        Bar = "bar",
        Block = "block",
        Underline = "underline",
    }
    default = Bar;
}

cycle_enum! {
    #[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
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
    #[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
    pub enum Difficulty {
        Normal = "normal",
        Expert = "expert",
        Master = "master",
    }
    default = Normal;
}

/// Fallback theme when a persisted `theme_name` no longer resolves. Kept as a
/// bare string so the pure `game` layer stays free of any `ui` dependency.
pub const DEFAULT_THEME: &str = "serika";

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub punctuation: bool,
    pub numbers: bool,
    pub lang_idx: usize,
    pub size_idx: usize,
    pub cursor_shape: CursorShape,
    pub difficulty: Difficulty,
    pub quote_filter: QuoteFilter,
    pub theme_name: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            punctuation: false,
            numbers: false,
            lang_idx: 0,
            size_idx: 0,
            cursor_shape: CursorShape::default(),
            difficulty: Difficulty::default(),
            quote_filter: QuoteFilter::default(),
            theme_name: DEFAULT_THEME.to_string(),
        }
    }
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
    /// Seconds-since-start of the last WPM sample; `None` once the test has
    /// finished (via normal completion or a difficulty fail), so `tick()`
    /// can never push a duplicate final sample.
    last_sample_secs: Option<u64>,
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
    /// Build a fresh test for `mode`, immediately generating its first word set.
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
            last_sample_secs: Some(0),
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

    /// Regenerate the word/quote set for the current mode and settings, and
    /// clear all progress. Used both for a brand-new test and for "restart".
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
            Mode::Time(secs) => (
                sample_words(&self.all_words, time_mode_pool(secs), rng),
                None,
            ),
            Mode::Words(n) => (sample_words(&self.all_words, n, rng), None),
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
        // NOTE: fall back to the full list if the filter matched nothing,
        // rather than surfacing an empty-selection error to the user.
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

    /// Record a keystroke at the cursor, advance it, and apply any
    /// difficulty rule (Expert/Master can end the test here via
    /// [`Self::difficulty_fail`]). No-op once the test is finished.
    pub fn type_char(&mut self, c: char) {
        if self.is_finished() || self.cursor >= self.chars.len() {
            return;
        }
        self.mark_keystroke();

        let expected = self.chars[self.cursor].expected;
        let correct = self.record_typed_char(c, expected);

        if self.difficulty_violated(correct, expected) {
            self.difficulty_fail();
            return;
        }

        self.advance_cursor();
    }

    /// Track test-start time, total keystroke count, and accumulated AFK
    /// time (gaps between keystrokes longer than [`AFK_THRESHOLD_SECS`]).
    fn mark_keystroke(&mut self) {
        let now = Instant::now();
        if self.started_at.is_none() {
            self.started_at = Some(now);
        }
        if let Some(last) = self.last_keystroke {
            let gap = now.duration_since(last).as_secs_f64();
            if gap > AFK_THRESHOLD_SECS {
                self.afk_secs += gap;
            }
        }
        self.last_keystroke = Some(now);
        self.total_keystrokes += 1;
    }

    /// Store `c` at the cursor and mark it correct/wrong. Returns whether it matched.
    fn record_typed_char(&mut self, c: char, expected: char) -> bool {
        self.chars[self.cursor].typed = Some(c);
        let correct = c == expected;
        self.chars[self.cursor].state = if correct {
            CharState::Correct
        } else {
            self.error_keystrokes += 1;
            CharState::Wrong
        };
        correct
    }

    /// Whether the active [`Difficulty`] ends the test on this keystroke:
    /// Expert fails on any wrong char, Master fails when a word containing
    /// an error is completed (space typed correctly).
    fn difficulty_violated(&self, correct: bool, expected: char) -> bool {
        match self.settings.difficulty {
            Difficulty::Expert => !correct,
            Difficulty::Master if correct && expected == ' ' => self.current_word_has_error(),
            _ => false,
        }
    }

    /// Whether the word ending at the cursor contains a `Wrong` char.
    fn current_word_has_error(&self) -> bool {
        if self.word_starts.is_empty() {
            return false;
        }
        let word_start = self.word_starts[self.word_at_cursor()];
        self.chars[word_start..self.cursor]
            .iter()
            .any(|c| c.state == CharState::Wrong)
    }

    fn advance_cursor(&mut self) {
        self.cursor += 1;
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].state = CharState::Current;
        }
        self.check_finished();
    }

    /// Ctrl+Backspace: erase back to the start of the current word (or the
    /// previous word if the cursor sits exactly on a word boundary).
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

    /// Restart with the exact same words/quote (used from the result screen),
    /// as opposed to [`Self::reset`] which generates a new set.
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
        self.last_sample_secs = Some(0);
        self.last_sample_errors = 0;
        self.last_sample_correct = 0;
        self.last_sample_cursor = 0;
        self.last_sample_elapsed = 0.0;
        self.last_keystroke = None;
        if !self.chars.is_empty() {
            self.chars[0].state = CharState::Current;
        }
    }

    /// Erase the character immediately before the cursor.
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
        // NOTE: the end-of-test sample often covers a tiny fraction of a
        // second, turning a couple of keystrokes into a 250+ WPM spike that
        // skews both the result chart's y-axis and consistency. Drop the
        // fragment (bookkeeping included, so its delta rolls into the next
        // sample) rather than special-casing every push_sample call site.
        if interval < MIN_REAL_INTERVAL && !self.wpm_samples.is_empty() {
            return;
        }
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

    /// Per-frame upkeep: end a `Time` mode test once its duration elapses,
    /// and push one WPM/accuracy sample per whole second of elapsed time
    /// (used to compute [`Self::consistency`]).
    pub fn tick(&mut self) {
        if self.is_finished() {
            return;
        }
        if let Mode::Time(_) = self.mode {
            self.check_finished();
        }
        if let Some(start) = self.started_at {
            let secs = start.elapsed().as_secs();
            if self.last_sample_secs.is_some_and(|last| secs > last) {
                self.last_sample_secs = Some(secs);
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
                .is_some_and(|t| t.elapsed() >= Duration::from_secs(secs)),
        };
        if done {
            self.finished_at = Some(Instant::now());
            self.push_sample();
            self.last_sample_secs = None;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished_at.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        // NOTE: afk_secs is a display-only stat; it is deliberately NOT
        // subtracted from the WPM timing window (matches monkeytype behavior).
        match (self.started_at, self.finished_at) {
            (Some(s), Some(e)) => e.duration_since(s),
            (Some(s), None) => s.elapsed(),
            _ => Duration::ZERO,
        }
    }

    pub fn time_left(&self) -> u64 {
        if let Mode::Time(secs) = self.mode {
            let elapsed = self.started_at.map_or(0, |t| t.elapsed().as_secs());
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

    /// Words per minute counting only correctly-typed characters (the
    /// number shown as the test's headline result).
    pub fn wpm(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs < 0.1 {
            return 0.0;
        }
        (self.correct_chars() as f64 / CHARS_PER_WORD) / (secs / 60.0)
    }

    /// Words per minute counting every character typed, correct or not
    /// (unlike [`Self::wpm`], errors don't reduce this).
    pub fn raw_wpm(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs < 0.1 {
            return 0.0;
        }
        (self.cursor as f64 / CHARS_PER_WORD) / (secs / 60.0)
    }

    /// Percentage of keystrokes (not final characters) that were correct;
    /// a corrected typo still counts as one error against the total.
    pub fn accuracy(&self) -> f64 {
        if self.total_keystrokes == 0 {
            return 100.0;
        }
        ((self.total_keystrokes - self.error_keystrokes) as f64 / self.total_keystrokes as f64)
            * 100.0
    }

    /// Whether the test ended in failure: a difficulty-rule violation, or
    /// final accuracy below [`FAIL_ACCURACY`].
    pub fn is_failed(&self) -> bool {
        self.difficulty_failed || self.accuracy() < FAIL_ACCURACY
    }

    /// Which failure condition (if any) ended the test; `None` on a clean pass.
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
        self.last_sample_secs = None;
    }

    /// 0-100 score from the coefficient of variation of per-second WPM
    /// samples: steady typing scores near 100, bursty typing scores lower.
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

    /// Index of the word the cursor is currently inside (or about to start).
    pub fn word_at_cursor(&self) -> usize {
        match self.word_starts.partition_point(|&s| s <= self.cursor) {
            0 => 0,
            n => n - 1,
        }
    }

    /// Count of completed words (spaces passed) up to the cursor.
    pub fn words_typed(&self) -> usize {
        self.chars
            .iter()
            .take(self.cursor)
            .filter(|c| c.expected == ' ')
            .count()
    }
}

/// Number of words to draw for a `Time(secs)` test: generous enough that
/// typing at any realistic speed never exhausts the pool before the timer
/// does, but not so large that short tests waste memory.
fn time_mode_pool(secs: u64) -> usize {
    (secs as usize * TIME_MODE_WORDS_PER_SEC).max(TIME_MODE_POOL)
}

/// Draw `target` words from `pool`, repeating shuffled passes over the whole
/// pool when `target` exceeds its length so long tests never run out of
/// text. Words can repeat across passes but never trivially — each pass is
/// an independent shuffle of the full pool.
fn sample_words(pool: &[String], target: usize, rng: &mut impl Rng) -> Vec<String> {
    if pool.is_empty() {
        return Vec::new();
    }
    let mut words = Vec::with_capacity(target);
    while words.len() < target {
        words.extend(pool.sample(rng, pool.len()).cloned());
    }
    words.truncate(target);
    words
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

// NOTE: sentence/comma insertion ratios are fixed constants rather than a
// user-tunable intensity — good enough for typing practice, no config surface.
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
    fn push_sample_drops_trailing_fragment() {
        let mut g = game();
        g.started_at = Some(Instant::now());
        g.push_sample();
        assert_eq!(g.wpm_samples.len(), 1, "first sample always lands");
        // Back-to-back call: a sub-second fragment, as produced at test end.
        g.push_sample();
        assert_eq!(g.wpm_samples.len(), 1, "fragment must be dropped");
        // A full sampling window still lands.
        g.last_sample_elapsed -= MIN_REAL_INTERVAL;
        g.push_sample();
        assert_eq!(g.wpm_samples.len(), 2);
    }

    #[test]
    fn word_starts_correct() {
        let words = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(compute_word_starts(&words), vec![0, 6]);
    }

    #[test]
    fn sample_words_repeats_pool_to_reach_target() {
        let pool = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut rng = rand::rng();
        let words = sample_words(&pool, 10, &mut rng);
        assert_eq!(words.len(), 10, "target exceeding pool length is still met");
        assert!(words.iter().all(|w| pool.contains(w)));
    }

    #[test]
    fn sample_words_empty_pool_returns_empty() {
        let mut rng = rand::rng();
        assert!(sample_words(&[], 10, &mut rng).is_empty());
    }

    #[test]
    fn time_mode_pool_scales_with_duration_and_has_a_floor() {
        assert_eq!(
            time_mode_pool(15),
            TIME_MODE_POOL,
            "short tests hit the floor"
        );
        assert!(time_mode_pool(3600) > TIME_MODE_POOL, "long tests scale up");
    }

    #[test]
    fn words_mode_reaches_target_even_when_pool_is_smaller() {
        // The default english word list is 200 words; ask for far more.
        let g = GameState::new(Mode::Words(1000), Settings::default(), vec![]);
        assert_eq!(g.words.len(), 1000);
    }

    #[test]
    fn time_mode_leaves_text_for_long_durations() {
        // With the old fixed 500-word pool, a small language's word list
        // (e.g. ~175 words) could be exhausted well before a long timer ends.
        let g = GameState::new(Mode::Time(600), Settings::default(), vec![]);
        assert!(
            g.chars.len() > 1000,
            "long time tests must generate enough text to type through"
        );
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

    /// Once the cursor reaches the end of the text there is no overtype
    /// buffer — further keystrokes must be dropped silently rather than
    /// panicking on an out-of-bounds `self.chars[self.cursor]` or corrupting
    /// keystroke/accuracy counters.
    #[test]
    fn type_char_past_the_end_of_the_text_is_a_no_op() {
        let mut g = game();
        let total_chars = g.chars.len();
        for _ in 0..total_chars {
            let expected = g.chars[g.cursor].expected;
            g.type_char(expected);
        }
        assert_eq!(g.cursor, total_chars);
        assert!(g.is_finished());

        let keystrokes_before = g.total_keystrokes;
        g.type_char('x');
        g.type_char('y');

        assert_eq!(g.cursor, total_chars, "cursor must not move past the end");
        assert_eq!(
            g.total_keystrokes, keystrokes_before,
            "keystrokes after the end must not be counted"
        );
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
