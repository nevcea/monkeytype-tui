use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct QuoteEntry {
    pub text: String,
    pub source: String,
}

#[derive(Deserialize)]
struct QuoteFile {
    quotes: Vec<QuoteEntry>,
}

fn quotes_json_for(lang: &str) -> &'static str {
    match lang {
        "english"    => include_str!("../static/quotes/english.json"),
        "spanish"    => include_str!("../static/quotes/spanish.json"),
        "french"     => include_str!("../static/quotes/french.json"),
        "german"     => include_str!("../static/quotes/german.json"),
        "portuguese" => include_str!("../static/quotes/portuguese.json"),
        "italian"    => include_str!("../static/quotes/italian.json"),
        "dutch"      => include_str!("../static/quotes/dutch.json"),
        "polish"     => include_str!("../static/quotes/polish.json"),
        "russian"    => include_str!("../static/quotes/russian.json"),
        "turkish"    => include_str!("../static/quotes/turkish.json"),
        "swedish"    => include_str!("../static/quotes/swedish.json"),
        "norwegian"  => include_str!("../static/quotes/norwegian.json"),
        "danish"     => include_str!("../static/quotes/danish.json"),
        "czech"      => include_str!("../static/quotes/czech.json"),
        "romanian"   => include_str!("../static/quotes/romanian.json"),
        "hungarian"  => include_str!("../static/quotes/hungarian.json"),
        "korean"     => include_str!("../static/quotes/korean.json"),
        "finnish"    => include_str!("../static/quotes/finnish.json"),
        "latin"      => include_str!("../static/quotes/latin.json"),
        "ukrainian"  => include_str!("../static/quotes/ukrainian.json"),
        "vietnamese" => include_str!("../static/quotes/vietnamese.json"),
        "albanian"   => include_str!("../static/quotes/albanian.json"),
        _            => include_str!("../static/quotes/english.json"),
    }
}

pub fn load_quotes_for(lang: &str) -> Vec<QuoteEntry> {
    serde_json::from_str::<QuoteFile>(quotes_json_for(lang))
        .expect("embedded quotes JSON is valid")
        .quotes
}
