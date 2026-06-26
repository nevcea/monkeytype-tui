use serde::Deserialize;

pub struct LangSize {
    pub label: &'static str,
    json: &'static str,
}

pub struct LangDef {
    pub name: &'static str,
    pub sizes: &'static [LangSize],
}

macro_rules! size {
    ($label:literal, $path:literal) => {
        LangSize { label: $label, json: include_str!($path) }
    };
}

macro_rules! lang {
    ($name:literal, [$($size:expr),+ $(,)?]) => {
        LangDef { name: $name, sizes: &[$($size),+] }
    };
}

pub static LANGUAGES: &[LangDef] = &[
    lang!("english", [
        size!("default", "../static/languages/english.json"),
        size!("1k",     "../static/languages/english_1k.json"),
        size!("5k",     "../static/languages/english_5k.json"),
        size!("10k",    "../static/languages/english_10k.json"),
    ]),
    lang!("spanish", [
        size!("default", "../static/languages/spanish.json"),
        size!("1k",      "../static/languages/spanish_1k.json"),
        size!("10k",     "../static/languages/spanish_10k.json"),
    ]),
    lang!("french", [
        size!("default", "../static/languages/french.json"),
        size!("1k",      "../static/languages/french_1k.json"),
        size!("10k",     "../static/languages/french_10k.json"),
    ]),
    lang!("german", [
        size!("default", "../static/languages/german.json"),
        size!("1k",      "../static/languages/german_1k.json"),
        size!("10k",     "../static/languages/german_10k.json"),
    ]),
    lang!("portuguese", [
        size!("default", "../static/languages/portuguese.json"),
        size!("1k",      "../static/languages/portuguese_1k.json"),
        size!("5k",      "../static/languages/portuguese_5k.json"),
    ]),
    lang!("italian", [
        size!("default", "../static/languages/italian.json"),
        size!("1k",      "../static/languages/italian_1k.json"),
    ]),
    lang!("dutch", [
        size!("default", "../static/languages/dutch.json"),
        size!("1k",      "../static/languages/dutch_1k.json"),
        size!("10k",     "../static/languages/dutch_10k.json"),
    ]),
    lang!("polish", [
        size!("default", "../static/languages/polish.json"),
        size!("5k",      "../static/languages/polish_5k.json"),
        size!("10k",     "../static/languages/polish_10k.json"),
    ]),
    lang!("russian", [
        size!("default", "../static/languages/russian.json"),
        size!("1k",      "../static/languages/russian_1k.json"),
        size!("5k",      "../static/languages/russian_5k.json"),
        size!("10k",     "../static/languages/russian_10k.json"),
    ]),
    lang!("turkish", [
        size!("default", "../static/languages/turkish.json"),
        size!("1k",      "../static/languages/turkish_1k.json"),
        size!("5k",      "../static/languages/turkish_5k.json"),
    ]),
    lang!("swedish", [
        size!("default", "../static/languages/swedish.json"),
        size!("1k",      "../static/languages/swedish_1k.json"),
    ]),
    lang!("norwegian", [
        size!("default", "../static/languages/norwegian.json"),
        size!("1k",      "../static/languages/norwegian_1k.json"),
        size!("5k",      "../static/languages/norwegian_5k.json"),
        size!("10k",     "../static/languages/norwegian_10k.json"),
    ]),
    lang!("danish", [
        size!("default", "../static/languages/danish.json"),
        size!("1k",      "../static/languages/danish_1k.json"),
        size!("10k",     "../static/languages/danish_10k.json"),
    ]),
    lang!("czech", [
        size!("default", "../static/languages/czech.json"),
        size!("1k",      "../static/languages/czech_1k.json"),
        size!("10k",     "../static/languages/czech_10k.json"),
    ]),
    lang!("romanian", [
        size!("default", "../static/languages/romanian.json"),
        size!("1k",      "../static/languages/romanian_1k.json"),
        size!("5k",      "../static/languages/romanian_5k.json"),
        size!("10k",     "../static/languages/romanian_10k.json"),
    ]),
    lang!("hungarian", [
        size!("default", "../static/languages/hungarian.json"),
        size!("1k",      "../static/languages/hungarian_1k.json"),
    ]),
    lang!("korean", [
        size!("default", "../static/languages/korean.json"),
        size!("1k",      "../static/languages/korean_1k.json"),
        size!("5k",      "../static/languages/korean_5k.json"),
    ]),
    lang!("japanese", [
        size!("default", "../static/languages/japanese.json"),
    ]),
    lang!("catalan", [
        size!("default", "../static/languages/catalan.json"),
        size!("1k",      "../static/languages/catalan_1k.json"),
    ]),
    lang!("finnish", [
        size!("default", "../static/languages/finnish.json"),
        size!("1k",      "../static/languages/finnish_1k.json"),
        size!("10k",     "../static/languages/finnish_10k.json"),
    ]),
    lang!("greek", [
        size!("default", "../static/languages/greek.json"),
        size!("1k",      "../static/languages/greek_1k.json"),
        size!("10k",     "../static/languages/greek_10k.json"),
    ]),
    lang!("latin", [
        size!("default", "../static/languages/latin.json"),
    ]),
    lang!("ukrainian", [
        size!("default", "../static/languages/ukrainian.json"),
        size!("1k",      "../static/languages/ukrainian_1k.json"),
        size!("10k",     "../static/languages/ukrainian_10k.json"),
    ]),
    lang!("vietnamese", [
        size!("default", "../static/languages/vietnamese.json"),
        size!("1k",      "../static/languages/vietnamese_1k.json"),
        size!("5k",      "../static/languages/vietnamese_5k.json"),
    ]),
    lang!("albanian", [
        size!("default", "../static/languages/albanian.json"),
        size!("1k",      "../static/languages/albanian_1k.json"),
    ]),
    lang!("bosnian", [
        size!("default", "../static/languages/bosnian.json"),
    ]),
];

#[derive(Deserialize)]
struct WordList {
    words: Vec<String>,
}

pub fn load_words(lang_idx: usize, size_idx: usize) -> Vec<String> {
    let lang = LANGUAGES.get(lang_idx).unwrap_or(&LANGUAGES[0]);
    let size = lang.sizes.get(size_idx).unwrap_or(&lang.sizes[0]);
    serde_json::from_str::<WordList>(size.json)
        .expect("embedded word list is valid JSON")
        .words
}
