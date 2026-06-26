use serde::Deserialize;

pub struct LangSize {
    pub label: &'static str,
    json: &'static str,
}

pub struct LangDef {
    pub name: &'static str,
    pub sizes: &'static [LangSize],
    pub quotes: Option<&'static str>,
}

macro_rules! size {
    ($label:literal, $path:literal) => {
        LangSize {
            label: $label,
            json: include_str!($path),
        }
    };
}

macro_rules! lang {
    ($name:literal, [$($size:expr),+ $(,)?]) => {
        LangDef { name: $name, sizes: &[$($size),+], quotes: None }
    };
    ($name:literal, [$($size:expr),+ $(,)?], $quotes:literal) => {
        LangDef { name: $name, sizes: &[$($size),+], quotes: Some(include_str!($quotes)) }
    };
}

pub static LANGUAGES: &[LangDef] = &[
    lang!(
        "english",
        [
            size!("default", "../static/languages/english.json"),
            size!("1k", "../static/languages/english_1k.json"),
            size!("5k", "../static/languages/english_5k.json"),
            size!("10k", "../static/languages/english_10k.json"),
        ],
        "../static/quotes/english.json"
    ),
    lang!(
        "spanish",
        [
            size!("default", "../static/languages/spanish.json"),
            size!("1k", "../static/languages/spanish_1k.json"),
            size!("10k", "../static/languages/spanish_10k.json"),
        ],
        "../static/quotes/spanish.json"
    ),
    lang!(
        "french",
        [
            size!("default", "../static/languages/french.json"),
            size!("1k", "../static/languages/french_1k.json"),
            size!("10k", "../static/languages/french_10k.json"),
        ],
        "../static/quotes/french.json"
    ),
    lang!(
        "german",
        [
            size!("default", "../static/languages/german.json"),
            size!("1k", "../static/languages/german_1k.json"),
            size!("10k", "../static/languages/german_10k.json"),
        ],
        "../static/quotes/german.json"
    ),
    lang!(
        "portuguese",
        [
            size!("default", "../static/languages/portuguese.json"),
            size!("1k", "../static/languages/portuguese_1k.json"),
            size!("5k", "../static/languages/portuguese_5k.json"),
        ],
        "../static/quotes/portuguese.json"
    ),
    lang!(
        "italian",
        [
            size!("default", "../static/languages/italian.json"),
            size!("1k", "../static/languages/italian_1k.json"),
        ],
        "../static/quotes/italian.json"
    ),
    lang!(
        "dutch",
        [
            size!("default", "../static/languages/dutch.json"),
            size!("1k", "../static/languages/dutch_1k.json"),
            size!("10k", "../static/languages/dutch_10k.json"),
        ],
        "../static/quotes/dutch.json"
    ),
    lang!(
        "polish",
        [
            size!("default", "../static/languages/polish.json"),
            size!("5k", "../static/languages/polish_5k.json"),
            size!("10k", "../static/languages/polish_10k.json"),
        ],
        "../static/quotes/polish.json"
    ),
    lang!(
        "russian",
        [
            size!("default", "../static/languages/russian.json"),
            size!("1k", "../static/languages/russian_1k.json"),
            size!("5k", "../static/languages/russian_5k.json"),
            size!("10k", "../static/languages/russian_10k.json"),
        ],
        "../static/quotes/russian.json"
    ),
    lang!(
        "turkish",
        [
            size!("default", "../static/languages/turkish.json"),
            size!("1k", "../static/languages/turkish_1k.json"),
            size!("5k", "../static/languages/turkish_5k.json"),
        ],
        "../static/quotes/turkish.json"
    ),
    lang!(
        "swedish",
        [
            size!("default", "../static/languages/swedish.json"),
            size!("1k", "../static/languages/swedish_1k.json"),
        ],
        "../static/quotes/swedish.json"
    ),
    lang!(
        "norwegian",
        [
            size!("default", "../static/languages/norwegian.json"),
            size!("1k", "../static/languages/norwegian_1k.json"),
            size!("5k", "../static/languages/norwegian_5k.json"),
            size!("10k", "../static/languages/norwegian_10k.json"),
        ],
        "../static/quotes/norwegian.json"
    ),
    lang!(
        "danish",
        [
            size!("default", "../static/languages/danish.json"),
            size!("1k", "../static/languages/danish_1k.json"),
            size!("10k", "../static/languages/danish_10k.json"),
        ],
        "../static/quotes/danish.json"
    ),
    lang!(
        "czech",
        [
            size!("default", "../static/languages/czech.json"),
            size!("1k", "../static/languages/czech_1k.json"),
            size!("10k", "../static/languages/czech_10k.json"),
        ],
        "../static/quotes/czech.json"
    ),
    lang!(
        "romanian",
        [
            size!("default", "../static/languages/romanian.json"),
            size!("1k", "../static/languages/romanian_1k.json"),
            size!("5k", "../static/languages/romanian_5k.json"),
            size!("10k", "../static/languages/romanian_10k.json"),
        ],
        "../static/quotes/romanian.json"
    ),
    lang!(
        "hungarian",
        [
            size!("default", "../static/languages/hungarian.json"),
            size!("1k", "../static/languages/hungarian_1k.json"),
        ],
        "../static/quotes/hungarian.json"
    ),
    lang!(
        "korean",
        [
            size!("default", "../static/languages/korean.json"),
            size!("1k", "../static/languages/korean_1k.json"),
            size!("5k", "../static/languages/korean_5k.json"),
        ],
        "../static/quotes/korean.json"
    ),
    lang!(
        "japanese",
        [size!("default", "../static/languages/japanese.json"),]
    ),
    lang!(
        "catalan",
        [
            size!("default", "../static/languages/catalan.json"),
            size!("1k", "../static/languages/catalan_1k.json"),
        ]
    ),
    lang!(
        "finnish",
        [
            size!("default", "../static/languages/finnish.json"),
            size!("1k", "../static/languages/finnish_1k.json"),
            size!("10k", "../static/languages/finnish_10k.json"),
        ],
        "../static/quotes/finnish.json"
    ),
    lang!(
        "greek",
        [
            size!("default", "../static/languages/greek.json"),
            size!("1k", "../static/languages/greek_1k.json"),
            size!("10k", "../static/languages/greek_10k.json"),
        ]
    ),
    lang!(
        "latin",
        [size!("default", "../static/languages/latin.json"),],
        "../static/quotes/latin.json"
    ),
    lang!(
        "ukrainian",
        [
            size!("default", "../static/languages/ukrainian.json"),
            size!("1k", "../static/languages/ukrainian_1k.json"),
            size!("10k", "../static/languages/ukrainian_10k.json"),
        ],
        "../static/quotes/ukrainian.json"
    ),
    lang!(
        "vietnamese",
        [
            size!("default", "../static/languages/vietnamese.json"),
            size!("1k", "../static/languages/vietnamese_1k.json"),
            size!("5k", "../static/languages/vietnamese_5k.json"),
        ],
        "../static/quotes/vietnamese.json"
    ),
    lang!(
        "albanian",
        [
            size!("default", "../static/languages/albanian.json"),
            size!("1k", "../static/languages/albanian_1k.json"),
        ],
        "../static/quotes/albanian.json"
    ),
    lang!(
        "bosnian",
        [size!("default", "../static/languages/bosnian.json"),]
    ),
    lang!(
        "afrikaans",
        [
            size!("default", "../static/languages/afrikaans.json"),
            size!("1k", "../static/languages/afrikaans_1k.json"),
            size!("10k", "../static/languages/afrikaans_10k.json"),
        ],
        "../static/quotes/afrikaans.json"
    ),
    lang!(
        "azerbaijani",
        [
            size!("default", "../static/languages/azerbaijani.json"),
            size!("1k", "../static/languages/azerbaijani_1k.json"),
        ],
        "../static/quotes/azerbaijani.json"
    ),
    lang!(
        "belarusian",
        [
            size!("default", "../static/languages/belarusian.json"),
            size!("1k", "../static/languages/belarusian_1k.json"),
            size!("5k", "../static/languages/belarusian_5k.json"),
            size!("10k", "../static/languages/belarusian_10k.json"),
        ],
        "../static/quotes/belarusian.json"
    ),
    lang!(
        "bulgarian",
        [
            size!("default", "../static/languages/bulgarian.json"),
            size!("1k", "../static/languages/bulgarian_1k.json"),
        ],
        "../static/quotes/bulgarian.json"
    ),
    lang!(
        "croatian",
        [
            size!("default", "../static/languages/croatian.json"),
            size!("1k", "../static/languages/croatian_1k.json"),
        ]
    ),
    lang!(
        "esperanto",
        [
            size!("default", "../static/languages/esperanto.json"),
            size!("1k", "../static/languages/esperanto_1k.json"),
            size!("10k", "../static/languages/esperanto_10k.json"),
        ],
        "../static/quotes/esperanto.json"
    ),
    lang!(
        "estonian",
        [
            size!("default", "../static/languages/estonian.json"),
            size!("1k", "../static/languages/estonian_1k.json"),
            size!("5k", "../static/languages/estonian_5k.json"),
            size!("10k", "../static/languages/estonian_10k.json"),
        ],
        "../static/quotes/estonian.json"
    ),
    lang!(
        "filipino",
        [
            size!("default", "../static/languages/filipino.json"),
            size!("1k", "../static/languages/filipino_1k.json"),
        ],
        "../static/quotes/filipino.json"
    ),
    lang!(
        "icelandic",
        [
            size!("default", "../static/languages/icelandic.json"),
            size!("1k", "../static/languages/icelandic_1k.json"),
        ],
        "../static/quotes/icelandic.json"
    ),
    lang!(
        "indonesian",
        [
            size!("default", "../static/languages/indonesian.json"),
            size!("1k", "../static/languages/indonesian_1k.json"),
            size!("10k", "../static/languages/indonesian_10k.json"),
        ],
        "../static/quotes/indonesian.json"
    ),
    lang!(
        "irish",
        [
            size!("default", "../static/languages/irish.json"),
            size!("1k", "../static/languages/irish_1k.json"),
        ],
        "../static/quotes/irish.json"
    ),
    lang!(
        "kazakh",
        [
            size!("default", "../static/languages/kazakh.json"),
            size!("1k", "../static/languages/kazakh_1k.json"),
        ],
        "../static/quotes/kazakh.json"
    ),
    lang!(
        "latvian",
        [
            size!("default", "../static/languages/latvian.json"),
            size!("1k", "../static/languages/latvian_1k.json"),
        ]
    ),
    lang!(
        "lithuanian",
        [
            size!("default", "../static/languages/lithuanian.json"),
            size!("1k", "../static/languages/lithuanian_1k.json"),
        ],
        "../static/quotes/lithuanian.json"
    ),
    lang!(
        "macedonian",
        [
            size!("default", "../static/languages/macedonian.json"),
            size!("1k", "../static/languages/macedonian_1k.json"),
            size!("10k", "../static/languages/macedonian_10k.json"),
        ]
    ),
    lang!(
        "malay",
        [
            size!("default", "../static/languages/malay.json"),
            size!("1k", "../static/languages/malay_1k.json"),
        ]
    ),
    lang!(
        "maltese",
        [
            size!("default", "../static/languages/maltese.json"),
            size!("1k", "../static/languages/maltese_1k.json"),
        ]
    ),
    lang!(
        "mongolian",
        [
            size!("default", "../static/languages/mongolian.json"),
            size!("10k", "../static/languages/mongolian_10k.json"),
        ],
        "../static/quotes/mongolian.json"
    ),
    lang!(
        "serbian",
        [
            size!("default", "../static/languages/serbian.json"),
            size!("10k", "../static/languages/serbian_10k.json"),
        ],
        "../static/quotes/serbian.json"
    ),
    lang!(
        "slovak",
        [
            size!("default", "../static/languages/slovak.json"),
            size!("1k", "../static/languages/slovak_1k.json"),
            size!("10k", "../static/languages/slovak_10k.json"),
        ],
        "../static/quotes/slovak.json"
    ),
    lang!(
        "slovenian",
        [
            size!("default", "../static/languages/slovenian.json"),
            size!("1k", "../static/languages/slovenian_1k.json"),
            size!("5k", "../static/languages/slovenian_5k.json"),
        ]
    ),
    lang!(
        "welsh",
        [
            size!("default", "../static/languages/welsh.json"),
            size!("1k", "../static/languages/welsh_1k.json"),
        ]
    ),
];

pub fn load_words(lang_idx: usize, size_idx: usize) -> Vec<String> {
    #[derive(Deserialize)]
    struct WordList {
        words: Vec<String>,
    }
    let lang = LANGUAGES.get(lang_idx).unwrap_or(&LANGUAGES[0]);
    let size = lang.sizes.get(size_idx).unwrap_or(&lang.sizes[0]);
    serde_json::from_str::<WordList>(size.json)
        .expect("embedded word list is valid JSON")
        .words
}

#[derive(Deserialize, Clone)]
pub struct QuoteEntry {
    pub text: String,
    pub source: String,
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
    serde_json::from_str::<QuoteFile>(json)
        .expect("embedded quotes JSON is valid")
        .quotes
}
