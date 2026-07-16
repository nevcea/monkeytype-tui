use ratatui::style::Color;

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

/// Built-in themes. Index 0 is the default; `Settings::theme_idx` selects one.
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
];

thread_local! {
    /// Active theme for the current frame, set by `draw()`. This is render state
    /// local to the ui layer, not `App` state — the immutability rule still holds.
    static ACTIVE_THEME: std::cell::Cell<Theme> = const { std::cell::Cell::new(THEMES[0]) };
}

pub(super) fn set_active_theme(idx: usize) {
    let t = THEMES.get(idx).copied().unwrap_or(THEMES[0]);
    ACTIVE_THEME.with(|c| c.set(t));
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
