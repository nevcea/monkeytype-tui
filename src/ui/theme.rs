//! Color palette: built-in `THEMES` plus user themes loaded once from
//! `data_dir()/themes/*.json`, selected via `Settings::theme_name` and read
//! per-frame through the `th_*()` accessors used by every `ui/*` screen.

use ratatui::style::Color;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub bg: Color,
    pub correct: Color,
    pub wrong: Color,
    pub pending: Color,
    pub accent: Color,
    pub dim: Color,
    pub wrong_bg: Color,
    pub gauge_bg: Color,
    pub fg: Color,
    pub sub: Color,
}

/// Built-in themes. Index 0 is the default; `Settings::theme_name` selects one
/// (see `theme_by_name`), and `all_themes()` appends any user-defined themes.
pub const THEMES: &[Theme] = &[
    Theme {
        name: "serika",
        bg: Color::Rgb(28, 28, 30),
        correct: Color::Rgb(210, 200, 170),
        wrong: Color::Rgb(202, 71, 71),
        pending: Color::Rgb(88, 88, 93),
        accent: Color::Rgb(226, 183, 20),
        dim: Color::Rgb(72, 72, 77),
        wrong_bg: Color::Rgb(60, 15, 15),
        gauge_bg: Color::Rgb(48, 48, 52),
        fg: Color::Rgb(200, 200, 205),
        sub: Color::Rgb(140, 140, 145),
    },
    Theme {
        name: "mono",
        bg: Color::Rgb(24, 24, 24),
        correct: Color::Rgb(230, 230, 230),
        wrong: Color::Rgb(190, 90, 90),
        pending: Color::Rgb(96, 96, 96),
        accent: Color::Rgb(235, 235, 235),
        dim: Color::Rgb(70, 70, 70),
        wrong_bg: Color::Rgb(55, 20, 20),
        gauge_bg: Color::Rgb(50, 50, 50),
        fg: Color::Rgb(210, 210, 210),
        sub: Color::Rgb(150, 150, 150),
    },
    Theme {
        name: "matrix",
        bg: Color::Rgb(10, 16, 10),
        correct: Color::Rgb(120, 220, 120),
        wrong: Color::Rgb(220, 90, 70),
        pending: Color::Rgb(60, 100, 60),
        accent: Color::Rgb(80, 240, 120),
        dim: Color::Rgb(40, 70, 40),
        wrong_bg: Color::Rgb(45, 15, 12),
        gauge_bg: Color::Rgb(24, 44, 24),
        fg: Color::Rgb(170, 230, 170),
        sub: Color::Rgb(90, 150, 90),
    },
    Theme {
        name: "rose",
        bg: Color::Rgb(26, 20, 28),
        correct: Color::Rgb(230, 200, 220),
        wrong: Color::Rgb(220, 90, 110),
        pending: Color::Rgb(100, 80, 100),
        accent: Color::Rgb(232, 120, 180),
        dim: Color::Rgb(78, 62, 78),
        wrong_bg: Color::Rgb(60, 18, 30),
        gauge_bg: Color::Rgb(52, 40, 52),
        fg: Color::Rgb(220, 200, 215),
        sub: Color::Rgb(160, 130, 155),
    },
    Theme {
        name: "serika_dark",
        bg: Color::Rgb(50, 52, 55),
        correct: Color::Rgb(209, 208, 197),
        wrong: Color::Rgb(202, 71, 84),
        pending: Color::Rgb(100, 102, 105),
        accent: Color::Rgb(226, 183, 20),
        dim: Color::Rgb(74, 76, 79),
        wrong_bg: Color::Rgb(58, 20, 20),
        gauge_bg: Color::Rgb(42, 44, 46),
        fg: Color::Rgb(209, 208, 197),
        sub: Color::Rgb(154, 156, 158),
    },
    Theme {
        name: "dracula",
        bg: Color::Rgb(40, 42, 54),
        correct: Color::Rgb(248, 248, 242),
        wrong: Color::Rgb(255, 85, 85),
        pending: Color::Rgb(98, 114, 164),
        accent: Color::Rgb(189, 147, 249),
        dim: Color::Rgb(68, 71, 90),
        wrong_bg: Color::Rgb(74, 37, 48),
        gauge_bg: Color::Rgb(56, 58, 74),
        fg: Color::Rgb(248, 248, 242),
        sub: Color::Rgb(154, 165, 196),
    },
    Theme {
        name: "nord",
        bg: Color::Rgb(46, 52, 64),
        correct: Color::Rgb(216, 222, 233),
        wrong: Color::Rgb(191, 97, 106),
        pending: Color::Rgb(76, 86, 106),
        accent: Color::Rgb(136, 192, 208),
        dim: Color::Rgb(67, 76, 94),
        wrong_bg: Color::Rgb(59, 42, 46),
        gauge_bg: Color::Rgb(59, 66, 82),
        fg: Color::Rgb(216, 222, 233),
        sub: Color::Rgb(129, 137, 155),
    },
    Theme {
        name: "gruvbox_dark",
        bg: Color::Rgb(40, 40, 40),
        correct: Color::Rgb(235, 219, 178),
        wrong: Color::Rgb(251, 73, 52),
        pending: Color::Rgb(102, 92, 84),
        accent: Color::Rgb(250, 189, 47),
        dim: Color::Rgb(80, 73, 69),
        wrong_bg: Color::Rgb(58, 36, 34),
        gauge_bg: Color::Rgb(60, 56, 54),
        fg: Color::Rgb(235, 219, 178),
        sub: Color::Rgb(168, 153, 132),
    },
    Theme {
        name: "catppuccin",
        bg: Color::Rgb(30, 30, 46),
        correct: Color::Rgb(205, 214, 244),
        wrong: Color::Rgb(243, 139, 168),
        pending: Color::Rgb(108, 112, 134),
        accent: Color::Rgb(203, 166, 247),
        dim: Color::Rgb(69, 71, 90),
        wrong_bg: Color::Rgb(58, 37, 48),
        gauge_bg: Color::Rgb(49, 50, 68),
        fg: Color::Rgb(205, 214, 244),
        sub: Color::Rgb(147, 153, 178),
    },
    Theme {
        name: "tokyonight",
        bg: Color::Rgb(26, 27, 38),
        correct: Color::Rgb(192, 202, 245),
        wrong: Color::Rgb(247, 118, 142),
        pending: Color::Rgb(86, 95, 137),
        accent: Color::Rgb(122, 162, 247),
        dim: Color::Rgb(59, 66, 97),
        wrong_bg: Color::Rgb(58, 37, 48),
        gauge_bg: Color::Rgb(41, 46, 66),
        fg: Color::Rgb(192, 202, 245),
        sub: Color::Rgb(121, 130, 169),
    },
    Theme {
        name: "rose_pine",
        bg: Color::Rgb(25, 23, 36),
        correct: Color::Rgb(224, 222, 244),
        wrong: Color::Rgb(235, 111, 146),
        pending: Color::Rgb(110, 106, 134),
        accent: Color::Rgb(235, 188, 186),
        dim: Color::Rgb(64, 61, 82),
        wrong_bg: Color::Rgb(58, 37, 48),
        gauge_bg: Color::Rgb(38, 35, 58),
        fg: Color::Rgb(224, 222, 244),
        sub: Color::Rgb(144, 140, 170),
    },
    Theme {
        name: "solarized_dark",
        bg: Color::Rgb(0, 43, 54),
        correct: Color::Rgb(147, 161, 161),
        wrong: Color::Rgb(220, 50, 47),
        pending: Color::Rgb(88, 110, 117),
        accent: Color::Rgb(38, 139, 210),
        dim: Color::Rgb(10, 74, 90),
        wrong_bg: Color::Rgb(58, 26, 26),
        gauge_bg: Color::Rgb(7, 54, 66),
        fg: Color::Rgb(147, 161, 161),
        sub: Color::Rgb(101, 123, 131),
    },
    Theme {
        name: "github_dark",
        bg: Color::Rgb(13, 17, 23),
        correct: Color::Rgb(201, 209, 217),
        wrong: Color::Rgb(248, 81, 73),
        pending: Color::Rgb(72, 79, 88),
        accent: Color::Rgb(88, 166, 255),
        dim: Color::Rgb(48, 54, 61),
        wrong_bg: Color::Rgb(58, 29, 29),
        gauge_bg: Color::Rgb(33, 38, 45),
        fg: Color::Rgb(201, 209, 217),
        sub: Color::Rgb(139, 148, 158),
    },
    Theme {
        name: "oled",
        bg: Color::Rgb(0, 0, 0),
        correct: Color::Rgb(230, 230, 230),
        wrong: Color::Rgb(255, 85, 85),
        pending: Color::Rgb(102, 102, 102),
        accent: Color::Rgb(255, 176, 0),
        dim: Color::Rgb(51, 51, 51),
        wrong_bg: Color::Rgb(51, 0, 0),
        gauge_bg: Color::Rgb(26, 26, 26),
        fg: Color::Rgb(230, 230, 230),
        sub: Color::Rgb(153, 153, 153),
    },
];

/// Deserialized form of a user theme file (`data_dir()/themes/*.json`). Colors
/// are `#rrggbb` hex strings, converted to `Color::Rgb` on load.
#[derive(Deserialize)]
struct ThemeFile {
    name: String,
    bg: String,
    correct: String,
    wrong: String,
    pending: String,
    accent: String,
    dim: String,
    wrong_bg: String,
    gauge_bg: String,
    fg: String,
    sub: String,
}

fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    // `len()` counts bytes, so the ASCII check is what makes the byte-range
    // slicing below safe: "aábcd" is 6 bytes but splits mid-char.
    if !s.is_ascii() || s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

impl ThemeFile {
    fn into_theme(self) -> Option<Theme> {
        Some(Theme {
            // ponytail: user themes load once at startup and live for the whole
            // run, so leaking the name to get a &'static str keeps Theme: Copy.
            name: Box::leak(self.name.into_boxed_str()),
            bg: parse_hex(&self.bg)?,
            correct: parse_hex(&self.correct)?,
            wrong: parse_hex(&self.wrong)?,
            pending: parse_hex(&self.pending)?,
            accent: parse_hex(&self.accent)?,
            dim: parse_hex(&self.dim)?,
            wrong_bg: parse_hex(&self.wrong_bg)?,
            gauge_bg: parse_hex(&self.gauge_bg)?,
            fg: parse_hex(&self.fg)?,
            sub: parse_hex(&self.sub)?,
        })
    }
}

fn load_user_themes() -> Vec<Theme> {
    let Some(dir) = crate::storage::data_dir().map(|d| d.join("themes")) else {
        return vec![];
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };
    let mut out: Vec<Theme> = vec![];
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        if let Ok(data) = std::fs::read_to_string(&path)
            && let Ok(tf) = serde_json::from_str::<ThemeFile>(&data)
            && let Some(t) = tf.into_theme()
        {
            out.push(t);
        }
    }
    out.sort_by(|a, b| a.name.cmp(b.name));
    out
}

/// All available themes: built-ins followed by user themes loaded once from
/// `data_dir()/themes/*.json`.
static ALL_THEMES: LazyLock<Vec<Theme>> = LazyLock::new(|| {
    let mut v = THEMES.to_vec();
    v.extend(load_user_themes());
    v
});

pub fn all_themes() -> &'static [Theme] {
    &ALL_THEMES
}

thread_local! {
    /// Active theme for the current frame, set by `draw()`. This is render state
    /// local to the ui layer, not `App` state — the immutability rule still holds.
    static ACTIVE_THEME: std::cell::Cell<Theme> = const { std::cell::Cell::new(THEMES[0]) };
}

/// Resolve a theme by name, falling back to the first built-in when the name is
/// empty or no longer present (e.g. a persisted theme that was removed).
pub fn theme_by_name(name: &str) -> Theme {
    all_themes()
        .iter()
        .find(|t| t.name == name)
        .copied()
        .unwrap_or(THEMES[0])
}

pub(super) fn set_active_theme(name: &str) {
    ACTIVE_THEME.with(|c| c.set(theme_by_name(name)));
}

fn theme() -> Theme {
    ACTIVE_THEME.with(|c| c.get())
}

pub(super) fn th_bg() -> Color {
    theme().bg
}
pub(super) fn th_correct() -> Color {
    theme().correct
}
pub(super) fn th_wrong() -> Color {
    theme().wrong
}
pub(super) fn th_pending() -> Color {
    theme().pending
}
pub(super) fn th_accent() -> Color {
    theme().accent
}
pub(super) fn th_dim() -> Color {
    theme().dim
}
pub(super) fn th_wrong_bg() -> Color {
    theme().wrong_bg
}
pub(super) fn th_gauge_bg() -> Color {
    theme().gauge_bg
}
pub(super) fn th_fg() -> Color {
    theme().fg
}
pub(super) fn th_sub() -> Color {
    theme().sub
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_accepts_with_and_without_hash() {
        assert_eq!(parse_hex("#1a2b3c"), Some(Color::Rgb(26, 43, 60)));
        assert_eq!(parse_hex("1a2b3c"), Some(Color::Rgb(26, 43, 60)));
    }

    #[test]
    fn parse_hex_rejects_malformed() {
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("zzzzzz"), None);
        assert_eq!(parse_hex(""), None);
        // Six *bytes* but only five chars — must not panic on a char boundary.
        assert_eq!(parse_hex("aábcd"), None);
    }

    #[test]
    fn theme_file_converts_all_fields() {
        let tf = ThemeFile {
            name: "test".into(),
            bg: "#000000".into(),
            correct: "#ffffff".into(),
            wrong: "#ff0000".into(),
            pending: "#808080".into(),
            accent: "#00ff00".into(),
            dim: "#333333".into(),
            wrong_bg: "#330000".into(),
            gauge_bg: "#111111".into(),
            fg: "#eeeeee".into(),
            sub: "#999999".into(),
        };
        let t = tf.into_theme().unwrap();
        assert_eq!(t.name, "test");
        assert_eq!(t.bg, Color::Rgb(0, 0, 0));
        assert_eq!(t.accent, Color::Rgb(0, 255, 0));
    }

    #[test]
    fn theme_file_with_bad_color_is_rejected() {
        let tf = ThemeFile {
            name: "bad".into(),
            bg: "not-a-color".into(),
            correct: "#ffffff".into(),
            wrong: "#ff0000".into(),
            pending: "#808080".into(),
            accent: "#00ff00".into(),
            dim: "#333333".into(),
            wrong_bg: "#330000".into(),
            gauge_bg: "#111111".into(),
            fg: "#eeeeee".into(),
            sub: "#999999".into(),
        };
        assert!(tf.into_theme().is_none());
    }

    #[test]
    fn theme_by_name_falls_back_to_first_builtin() {
        assert_eq!(theme_by_name("does-not-exist").name, THEMES[0].name);
        assert_eq!(theme_by_name("dracula").name, "dracula");
    }
}
