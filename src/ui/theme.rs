//! Color palette: built-in `THEMES` plus user themes loaded once from
//! `data_dir()/themes/*.json`, selected via `Settings::theme_name` and read
//! per-frame through the `th_*()` accessors used by every `ui/*` screen.

use ratatui::style::Color;
use serde::Deserialize;
use std::sync::LazyLock;

/// Declares the palette's colour fields exactly once, generating the `Theme`
/// struct, its `Theme::new` positional constructor, the `ThemeFile`
/// deserialization mirror, and the per-frame `th_*()` accessors.
macro_rules! theme_fields {
    ($($field:ident => $accessor:ident),* $(,)?) => {
        #[derive(Clone, Copy)]
        pub struct Theme {
            pub name: &'static str,
            $(pub $field: Color,)*
        }

        impl Theme {
            /// Colours are positional, in the field order declared below.
            #[allow(clippy::too_many_arguments)]
            const fn new(name: &'static str, $($field: (u8, u8, u8)),*) -> Theme {
                Theme { name, $($field: Color::Rgb($field.0, $field.1, $field.2),)* }
            }
        }

        /// Deserialized form of a user theme file (`data_dir()/themes/*.json`).
        /// Colors are `#rrggbb` hex strings, converted to `Color::Rgb` on load.
        #[derive(Deserialize)]
        struct ThemeFile {
            name: String,
            $($field: String,)*
        }

        impl ThemeFile {
            fn into_theme(self) -> Option<Theme> {
                Some(Theme {
                    // ponytail: user themes load once at startup and live for the whole
                    // run, so leaking the name to get a &'static str keeps Theme: Copy.
                    name: Box::leak(self.name.into_boxed_str()),
                    $($field: parse_hex(&self.$field)?,)*
                })
            }
        }

        $(pub(super) fn $accessor() -> Color { theme().$field })*
    };
}

theme_fields! {
    bg => th_bg,
    correct => th_correct,
    wrong => th_wrong,
    pending => th_pending,
    accent => th_accent,
    dim => th_dim,
    wrong_bg => th_wrong_bg,
    gauge_bg => th_gauge_bg,
    fg => th_fg,
    sub => th_sub,
}

/// Built-in themes. Index 0 is the default; `Settings::theme_name` selects one
/// (see `theme_by_name`), and `all_themes()` appends any user-defined themes.
/// Colours are positional, matching the `theme_fields!` order above:
/// bg, correct, wrong, pending, accent / dim, wrong_bg, gauge_bg, fg, sub.
// ponytail: positional tuples plus `rustfmt::skip` keep each palette to three
// readable rows; naming all ten fields per theme cost ~170 lines of noise.
#[rustfmt::skip]
const THEMES: &[Theme] = &[
    Theme::new("serika",
        (28,28,30), (210,200,170), (202,71,71), (88,88,93), (226,183,20),
        (72,72,77), (60,15,15), (48,48,52), (200,200,205), (140,140,145)),
    Theme::new("mono",
        (24,24,24), (230,230,230), (190,90,90), (96,96,96), (235,235,235),
        (70,70,70), (55,20,20), (50,50,50), (210,210,210), (150,150,150)),
    Theme::new("matrix",
        (10,16,10), (120,220,120), (220,90,70), (60,100,60), (80,240,120),
        (40,70,40), (45,15,12), (24,44,24), (170,230,170), (90,150,90)),
    Theme::new("rose",
        (26,20,28), (230,200,220), (220,90,110), (100,80,100), (232,120,180),
        (78,62,78), (60,18,30), (52,40,52), (220,200,215), (160,130,155)),
    Theme::new("serika_dark",
        (50,52,55), (209,208,197), (202,71,84), (100,102,105), (226,183,20),
        (74,76,79), (58,20,20), (42,44,46), (209,208,197), (154,156,158)),
    Theme::new("dracula",
        (40,42,54), (248,248,242), (255,85,85), (98,114,164), (189,147,249),
        (68,71,90), (74,37,48), (56,58,74), (248,248,242), (154,165,196)),
    Theme::new("nord",
        (46,52,64), (216,222,233), (191,97,106), (76,86,106), (136,192,208),
        (67,76,94), (59,42,46), (59,66,82), (216,222,233), (129,137,155)),
    Theme::new("gruvbox_dark",
        (40,40,40), (235,219,178), (251,73,52), (102,92,84), (250,189,47),
        (80,73,69), (58,36,34), (60,56,54), (235,219,178), (168,153,132)),
    Theme::new("catppuccin",
        (30,30,46), (205,214,244), (243,139,168), (108,112,134), (203,166,247),
        (69,71,90), (58,37,48), (49,50,68), (205,214,244), (147,153,178)),
    Theme::new("tokyonight",
        (26,27,38), (192,202,245), (247,118,142), (86,95,137), (122,162,247),
        (59,66,97), (58,37,48), (41,46,66), (192,202,245), (121,130,169)),
    Theme::new("rose_pine",
        (25,23,36), (224,222,244), (235,111,146), (110,106,134), (235,188,186),
        (64,61,82), (58,37,48), (38,35,58), (224,222,244), (144,140,170)),
    Theme::new("solarized_dark",
        (0,43,54), (147,161,161), (220,50,47), (88,110,117), (38,139,210),
        (10,74,90), (58,26,26), (7,54,66), (147,161,161), (101,123,131)),
    Theme::new("github_dark",
        (13,17,23), (201,209,217), (248,81,73), (72,79,88), (88,166,255),
        (48,54,61), (58,29,29), (33,38,45), (201,209,217), (139,148,158)),
    Theme::new("oled",
        (0,0,0), (230,230,230), (255,85,85), (102,102,102), (255,176,0),
        (51,51,51), (51,0,0), (26,26,26), (230,230,230), (153,153,153)),
    Theme::new("8008",
        (51,58,69), (233,236,240), (218,51,51), (147,158,174), (244,76,127),
        (46,52,61), (65,15,15), (67,74,85), (233,236,240), (184,191,202)),
    Theme::new("aurora",
        (1,25,38), (255,255,255), (185,77,161), (36,92,105), (0,233,128),
        (0,12,19), (56,23,48), (17,41,54), (255,255,255), (112,149,157)),
    Theme::new("botanical",
        (123,156,152), (234,241,243), (246,201,180), (73,87,85), (234,241,243),
        (114,144,141), (74,60,54), (139,172,168), (234,241,243), (136,145,144)),
    Theme::new("carbon",
        (49,49,49), (245,230,200), (231,45,45), (97,97,97), (246,110,13),
        (43,43,43), (69,14,14), (65,65,65), (245,230,200), (152,152,152)),
    Theme::new("horizon",
        (28,30,38), (187,187,187), (213,81,112), (219,136,111), (196,168,138),
        (23,24,31), (64,24,34), (44,46,54), (187,187,187), (231,177,161)),
    Theme::new("ishtar",
        (32,32,32), (250,225,195), (187,30,16), (132,120,105), (145,23,12),
        (39,39,39), (56,9,5), (48,48,48), (250,225,195), (175,167,157)),
    Theme::new("laser",
        (34,27,68), (219,231,232), (168,212,0), (184,35,86), (0,158,175),
        (30,23,59), (50,64,0), (50,43,84), (219,231,232), (208,112,145)),
    Theme::new("olive",
        (233,229,204), (55,55,49), (207,47,47), (183,179,158), (146,148,111),
        (212,207,188), (224,165,149), (217,213,188), (55,55,49), (118,116,102)),
    Theme::new("terminal",
        (25,26,27), (231,234,224), (166,23,23), (72,73,75), (121,166,23),
        (20,21,22), (50,7,7), (41,42,43), (231,234,224), (136,136,138)),
    Theme::new("vesper",
        (16,16,16), (255,255,255), (255,128,128), (160,160,160), (255,199,153),
        (28,28,28), (77,38,38), (32,32,32), (255,255,255), (193,193,193)),
    Theme::new("vaporwave",
        (164,167,234), (241,235,241), (87,60,169), (124,127,175), (227,104,218),
        (152,155,217), (133,124,208), (180,183,250), (241,235,241), (169,171,203)),
    Theme::new("monokai",
        (39,40,34), (226,226,220), (249,38,114), (230,219,116), (166,226,46),
        (31,32,27), (75,11,34), (55,56,50), (226,226,220), (238,231,164)),
];

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
