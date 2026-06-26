use rand::seq::SliceRandom;
use rand::Rng;
use std::time::{Duration, Instant};

use crate::quotes::QuoteEntry;
use crate::words::load_words;

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

#[derive(Clone, Default)]
pub struct Settings {
    pub punctuation: bool,
    pub numbers: bool,
    pub lang_idx: usize,  // index into words::LANGUAGES
    pub size_idx: usize,  // index into LANGUAGES[lang_idx].sizes
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
    last_sample_secs: u64,
    all_words: Vec<String>,
    all_quotes: Vec<QuoteEntry>,
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
            last_sample_secs: 0,
            all_words,
            all_quotes: quotes,
        };
        state.reset();
        state
    }

    pub fn reset(&mut self) {
        let mut rng = rand::thread_rng();

        let (raw_words, source): (Vec<String>, Option<String>) = match self.mode {
            Mode::Time(_) => {
                let w = self.all_words.choose_multiple(&mut rng, 100).cloned().collect();
                (w, None)
            }
            Mode::Words(n) => {
                let w = self.all_words.choose_multiple(&mut rng, n).cloned().collect();
                (w, None)
            }
            Mode::Quote => {
                if self.all_quotes.is_empty() {
                    (vec!["no quotes loaded".to_string()], None)
                } else {
                    let q = &self.all_quotes[rng.gen_range(0..self.all_quotes.len())];
                    let words = q.text.split_whitespace().map(String::from).collect();
                    (words, Some(q.source.clone()))
                }
            }
        };

        // Punctuation / numbers only apply to word/time modes
        let words = if source.is_none() {
            let mut w = raw_words;
            if self.settings.numbers {
                w = apply_numbers(w, &mut rng);
            }
            if self.settings.punctuation {
                w = apply_punctuation(w, &mut rng);
            }
            w
        } else {
            raw_words
        };

        self.quote_source = source;
        self.words = words;
        self.chars = words_to_chars(&self.words);
        self.word_starts = compute_word_starts(&self.words);
        self.cursor = 0;
        self.started_at = None;
        self.finished_at = None;
        self.total_keystrokes = 0;
        self.error_keystrokes = 0;
        self.wpm_samples = vec![];
        self.last_sample_secs = 0;

        if !self.chars.is_empty() {
            self.chars[0].state = CharState::Current;
        }
    }

    pub fn type_char(&mut self, c: char) {
        if self.is_finished() || self.cursor >= self.chars.len() {
            return;
        }
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }
        self.total_keystrokes += 1;

        let expected = self.chars[self.cursor].expected;
        self.chars[self.cursor].typed = Some(c);
        self.chars[self.cursor].state = if c == expected {
            CharState::Correct
        } else {
            self.error_keystrokes += 1;
            CharState::Wrong
        };

        self.cursor += 1;
        if self.cursor < self.chars.len() {
            self.chars[self.cursor].state = CharState::Current;
        }
        self.check_finished();
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
                self.wpm_samples.push(self.wpm());
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
            self.wpm_samples.push(self.wpm());
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

    pub fn wpm(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs < 0.1 {
            return 0.0;
        }
        let correct = self
            .chars
            .iter()
            .take(self.cursor)
            .filter(|c| c.state == CharState::Correct)
            .count() as f64;
        (correct / 5.0) / (secs / 60.0)
    }

    pub fn raw_wpm(&self) -> f64 {
        let secs = self.elapsed().as_secs_f64();
        if secs < 0.1 {
            return 0.0;
        }
        (self.cursor as f64 / 5.0) / (secs / 60.0)
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_keystrokes == 0 {
            return 100.0;
        }
        ((self.total_keystrokes - self.error_keystrokes) as f64 / self.total_keystrokes as f64)
            * 100.0
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
    words.iter().scan(0usize, |pos, w| {
        let start = *pos;
        *pos += w.chars().count() + 1;
        Some(start)
    }).collect()
}

fn words_to_chars(words: &[String]) -> Vec<TypedChar> {
    let mut chars = vec![];
    for (i, word) in words.iter().enumerate() {
        for ch in word.chars() {
            chars.push(TypedChar { expected: ch, typed: None, state: CharState::Pending });
        }
        if i + 1 < words.len() {
            chars.push(TypedChar { expected: ' ', typed: None, state: CharState::Pending });
        }
    }
    chars
}

fn apply_punctuation(words: Vec<String>, rng: &mut impl Rng) -> Vec<String> {
    let endings = ['.', '!', '?'];
    let n = words.len();
    let mut result = Vec::with_capacity(n);
    let mut since_sentence = 0usize;

    for (i, word) in words.into_iter().enumerate() {
        since_sentence += 1;
        if i + 1 < n && since_sentence >= 4 && rng.gen_ratio(1, 3) {
            let end = endings[rng.gen_range(0..endings.len())];
            result.push(format!("{word}{end}"));
            since_sentence = 0;
        } else if i + 1 < n && rng.gen_ratio(1, 4) {
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
            if rng.gen_ratio(1, 7) {
                rng.gen_range(0u32..1000).to_string()
            } else {
                w
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wpm_zero_before_start() {
        let g = GameState::new(Mode::Words(10), Settings::default(), vec![]);
        assert_eq!(g.wpm(), 0.0);
    }

    #[test]
    fn accuracy_full_when_no_keystrokes() {
        let g = GameState::new(Mode::Words(10), Settings::default(), vec![]);
        assert_eq!(g.accuracy(), 100.0);
    }

    #[test]
    fn word_starts_correct() {
        let words = vec!["hello".to_string(), "world".to_string()];
        let starts = compute_word_starts(&words);
        assert_eq!(starts, vec![0, 6]);
    }

    #[test]
    fn backspace_restores_state() {
        let mut g = GameState::new(Mode::Words(10), Settings::default(), vec![]);
        let expected = g.chars[0].expected;
        g.type_char(expected);
        assert_eq!(g.cursor, 1);
        g.backspace();
        assert_eq!(g.cursor, 0);
        assert_eq!(g.chars[0].state, CharState::Current);
    }
}
