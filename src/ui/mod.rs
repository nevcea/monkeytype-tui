use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

mod help;
mod history;
mod menu;
mod result;
mod settings;
mod test_screen;

// ── palette / themes ────────────────────────────────────────────────────────
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

// ── entry ─────────────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    set_active_theme(app.settings.theme_idx);

    let bg = Block::default().style(Style::default().bg(th_bg()));
    f.render_widget(bg, f.area());

    let area = f.area();
    if area.width < crate::app::MIN_WIDTH || area.height < crate::app::MIN_HEIGHT {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!(
                    "terminal too small  (min {}×{})",
                    crate::app::MIN_WIDTH,
                    crate::app::MIN_HEIGHT
                ),
                Style::default().fg(th_dim()),
            ))
            .alignment(Alignment::Center),
            Rect {
                x: area.x,
                y: area.height / 2,
                width: area.width,
                height: 1,
            },
        );
        return;
    }

    match app.screen {
        crate::app::Screen::Menu => menu::draw_menu(f, app),
        crate::app::Screen::Test => test_screen::draw_test(f, app),
        crate::app::Screen::Result => result::draw_result(f, app),
        crate::app::Screen::History => history::draw_history(f, app),
        crate::app::Screen::Help => help::draw_help(f),
        crate::app::Screen::Settings => settings::draw_settings(f, app),
    }

    if app.lang_picker.is_some() {
        help::draw_lang_picker(f, app);
    }
    if app.quit_confirm {
        draw_confirm(f, "quit?", app.quit_yes);
    }
    if app.test_confirm {
        draw_confirm(f, "abandon test?", app.test_confirm_yes);
    }
}

fn draw_confirm(f: &mut Frame, title: &str, is_yes: bool) {
    let area = centered_rect(40, 0, f.area());
    let area = Rect {
        x: area.x,
        y: f.area().height.saturating_sub(5) / 2,
        width: area.width,
        height: 5,
    };
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(th_dim()))
            .style(Style::default().bg(th_bg())),
        area,
    );
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: 3,
    };
    let sel = Style::default()
        .fg(th_accent())
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let dim = Style::default().fg(th_pending());
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                title,
                Style::default()
                    .fg(th_accent())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from(vec![
                Span::styled("yes", if is_yes { sel } else { dim }),
                Span::raw("   "),
                Span::styled("no", if !is_yes { sel } else { dim }),
            ]),
        ])
        .alignment(Alignment::Center),
        inner,
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(super) fn pin_footer(frame: Rect, height: u16) -> Rect {
    Rect {
        x: frame.x,
        y: frame.bottom().saturating_sub(height),
        width: frame.width,
        height: height.min(frame.height),
    }
}

pub(super) fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - pct_y) / 2),
        Constraint::Percentage(pct_y),
        Constraint::Percentage((100 - pct_y) / 2),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ])
    .split(v[1])[1]
}

pub(super) fn horiz_pad(r: Rect, pad: u16) -> Rect {
    Rect {
        x: r.x + pad,
        y: r.y,
        width: r.width.saturating_sub(pad * 2),
        height: r.height,
    }
}

pub(super) fn mode_tab_n(
    num: &'static str,
    label: impl Into<String>,
    active: bool,
) -> Span<'static> {
    let text = format!("{num}·{}", label.into());
    if active {
        Span::styled(
            text,
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else {
        Span::styled(text, Style::default().fg(th_pending()))
    }
}

pub(super) fn toggle_span(label: &str, on: bool) -> Span<'static> {
    if on {
        Span::styled(
            format!("[{label}]"),
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!(" {label} "), Style::default().fg(th_dim()))
    }
}

pub(super) fn option_spans<T: std::fmt::Display>(
    opts: &[T],
    selected: usize,
    suffix: &str,
) -> Vec<Span<'static>> {
    opts.iter()
        .enumerate()
        .flat_map(|(i, v)| {
            let label = format!("{v}{suffix}");
            let span = if i == selected {
                Span::styled(
                    label,
                    Style::default()
                        .fg(th_accent())
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
            } else {
                Span::styled(label, Style::default().fg(th_pending()))
            };
            vec![span, Span::raw("  ")]
        })
        .collect()
}

pub(super) fn custom_slot<'a>(selected: bool, suffix: &str, input: &Option<String>) -> Span<'a> {
    if selected {
        let text = if let Some(s) = input {
            format!("custom: {s}▌{suffix}")
        } else {
            "custom".to_string()
        };
        Span::styled(
            text,
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else {
        Span::styled("custom", Style::default().fg(th_pending()))
    }
}

pub(super) fn sep() -> Span<'static> {
    Span::raw("   ")
}

pub(super) fn kh(key: &str) -> Span<'static> {
    Span::styled(key.to_string(), Style::default().fg(th_sub()))
}

pub(super) fn col<S: Into<String>>(s: S, w: usize, color: Color) -> Span<'static> {
    use unicode_width::UnicodeWidthStr;
    // Pad by display width, not char count, so wide (CJK) names stay aligned.
    let s = s.into();
    let pad = w.saturating_sub(s.width());
    Span::styled(
        format!("{s}{}", " ".repeat(pad)),
        Style::default().fg(color),
    )
}
