//! Word/quote data: the `LANGUAGES` table of word lists and quotes embedded
//! at compile time via `include_str!`, plus a cache so switching modes never
//! re-parses the same JSON twice.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

pub struct LangSize {
    pub label: &'static str,
    json: &'static str,
}

pub struct LangDef {
    pub name: &'static str,
    pub sizes: &'static [LangSize],
    pub quotes: Option<&'static str>,
}

/// Builds a `LangDef` from the language name alone: word lists live at
/// `static/languages/<name>[_<size>].json` (the implicit `default` size has no
/// suffix) and quotes at `static/quotes/<name>.json`. Pass the bare `quotes`
/// marker when a quote file exists.
macro_rules! lang {
    (@size $name:literal, "default") => {
        LangSize { label: "default", json: include_str!(concat!("../static/languages/", $name, ".json")) }
    };
    (@size $name:literal, $size:literal) => {
        LangSize { label: $size, json: include_str!(concat!("../static/languages/", $name, "_", $size, ".json")) }
    };
    ($name:literal, [$($size:literal),* $(,)?]) => {
        LangDef {
            name: $name,
            sizes: &[lang!(@size $name, "default") $(, lang!(@size $name, $size))*],
            quotes: None,
        }
    };
    ($name:literal, [$($size:literal),* $(,)?], quotes) => {
        LangDef {
            quotes: Some(include_str!(concat!("../static/quotes/", $name, ".json"))),
            ..lang!($name, [$($size),*])
        }
    };
}

pub static LANGUAGES: &[LangDef] = &[
    lang!("english", ["1k", "5k", "10k"], quotes),
    lang!("spanish", ["1k", "10k"], quotes),
    lang!("french", ["1k", "10k"], quotes),
    lang!("german", ["1k", "10k"], quotes),
    lang!("portuguese", ["1k", "5k"], quotes),
    lang!("italian", ["1k"], quotes),
    lang!("dutch", ["1k", "10k"], quotes),
    lang!("polish", ["5k", "10k"], quotes),
    lang!("russian", ["1k", "5k", "10k"], quotes),
    lang!("turkish", ["1k", "5k"], quotes),
    lang!("swedish", ["1k"], quotes),
    lang!("norwegian", ["1k", "5k", "10k"], quotes),
    lang!("danish", ["1k", "10k"], quotes),
    lang!("czech", ["1k", "10k"], quotes),
    lang!("romanian", ["1k", "5k", "10k"], quotes),
    lang!("hungarian", ["1k"], quotes),
    lang!("korean", ["1k", "5k"], quotes),
    lang!("japanese", []),
    lang!("catalan", ["1k"]),
    lang!("finnish", ["1k", "10k"], quotes),
    lang!("greek", ["1k", "10k"]),
    lang!("latin", [], quotes),
    lang!("ukrainian", ["1k", "10k"], quotes),
    lang!("vietnamese", ["1k", "5k"], quotes),
    lang!("albanian", ["1k"], quotes),
    lang!("bosnian", []),
    lang!("afrikaans", ["1k", "10k"], quotes),
    lang!("azerbaijani", ["1k"], quotes),
    lang!("belarusian", ["1k", "5k", "10k"], quotes),
    lang!("bulgarian", ["1k"], quotes),
    lang!("croatian", ["1k"]),
    lang!("esperanto", ["1k", "10k"], quotes),
    lang!("estonian", ["1k", "5k", "10k"], quotes),
    lang!("filipino", ["1k"], quotes),
    lang!("icelandic", ["1k"], quotes),
    lang!("indonesian", ["1k", "10k"], quotes),
    lang!("irish", ["1k"], quotes),
    lang!("kazakh", ["1k"], quotes),
    lang!("latvian", ["1k"]),
    lang!("lithuanian", ["1k"], quotes),
    lang!("macedonian", ["1k", "10k"]),
    lang!("malay", ["1k"]),
    lang!("maltese", ["1k"]),
    lang!("mongolian", ["10k"], quotes),
    lang!("serbian", ["10k"], quotes),
    lang!("slovak", ["1k", "10k"], quotes),
    lang!("slovenian", ["1k", "5k"]),
    lang!("welsh", ["1k"]),
];

type WordCache = HashMap<(usize, usize), Arc<Vec<String>>>;

/// Parsed word lists cached by (lang_idx, size_idx) so restarting a test never
/// re-deserializes the embedded JSON (some lists hold ~10k words).
static WORD_CACHE: LazyLock<Mutex<WordCache>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn load_words(lang_idx: usize, size_idx: usize) -> Arc<Vec<String>> {
    #[derive(Deserialize)]
    struct WordList {
        words: Vec<String>,
    }
    let key = (lang_idx, size_idx);
    if let Ok(cache) = WORD_CACHE.lock()
        && let Some(words) = cache.get(&key)
    {
        return Arc::clone(words);
    }
    let lang = LANGUAGES.get(lang_idx).unwrap_or(&LANGUAGES[0]);
    let size = lang.sizes.get(size_idx).unwrap_or(&lang.sizes[0]);
    // Degrade gracefully rather than panic if an embedded file is malformed;
    // `tests::all_word_lists_parse` guards against this at build/test time.
    let words: Arc<Vec<String>> = Arc::new(
        serde_json::from_str::<WordList>(size.json)
            .map(|w| w.words)
            .unwrap_or_default(),
    );
    if let Ok(mut cache) = WORD_CACHE.lock() {
        cache.insert(key, Arc::clone(&words));
    }
    words
}

#[derive(Deserialize, Clone)]
pub struct QuoteEntry {
    pub text: String,
    pub source: String,
    #[serde(default)]
    pub length: u64,
}

pub fn load_quotes_for(lang: &str) -> Vec<QuoteEntry> {
    #[derive(Deserialize)]
    struct QuoteFile {
        quotes: Vec<QuoteEntry>,
    }
    let Some(json) = LANGUAGES
        .iter()
        .find(|l| l.name == lang)
        .and_then(|l| l.quotes)
    else {
        return vec![];
    };
    // Degrade gracefully on malformed data (see `tests::all_quote_files_parse`).
    let mut quotes = serde_json::from_str::<QuoteFile>(json)
        .map(|q| q.quotes)
        .unwrap_or_default();
    // Some entries omit `length`; derive it from the text so QuoteFilter works.
    for q in &mut quotes {
        if q.length == 0 {
            q.length = q.text.chars().count() as u64;
        }
    }
    quotes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_word_lists_parse() {
        // load_words degrades to an empty vec on malformed JSON, so a non-empty
        // result proves every embedded size file parsed into the expected shape.
        for (li, lang) in LANGUAGES.iter().enumerate() {
            for (si, size) in lang.sizes.iter().enumerate() {
                let words = load_words(li, si);
                assert!(
                    !words.is_empty(),
                    "empty/malformed word list: {} / {}",
                    lang.name,
                    size.label
                );
            }
        }
    }

    #[test]
    fn all_quote_files_parse() {
        for lang in LANGUAGES.iter().filter(|l| l.quotes.is_some()) {
            let quotes = load_quotes_for(lang.name);
            assert!(
                !quotes.is_empty(),
                "empty/malformed quotes for {}",
                lang.name
            );
            // Every quote must have a positive length so QuoteFilter buckets work.
            assert!(
                quotes.iter().all(|q| q.length > 0),
                "quote with zero length in {}",
                lang.name
            );
        }
    }
}
