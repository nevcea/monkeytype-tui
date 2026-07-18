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
mod theme;

use theme::*;
pub use theme::{Theme, all_themes, theme_by_name};

// ── entry ─────────────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    set_active_theme(&app.settings.theme_name);

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

    if app.menu.lang_picker.is_some() {
        help::draw_lang_picker(f, app);
    }
    if app.menu.theme_picker.is_some() {
        help::draw_theme_picker(f, app);
    }
    if app.dialog.quit_confirm {
        draw_confirm(f, "quit?", app.dialog.quit_yes);
    }
    if app.dialog.test_confirm {
        draw_confirm(f, "abandon test?", app.dialog.test_confirm_yes);
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

#[cfg(test)]
mod render_smoke_tests {
    use super::*;
    use crate::app::{App, Screen};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// Rendering every screen at a normal size must never panic, regardless of
    /// what state the screen holds (e.g. empty history, a fresh unstarted game).
    #[test]
    fn draw_does_not_panic_on_any_screen() {
        for screen in [
            Screen::Menu,
            Screen::Test,
            Screen::Result,
            Screen::History,
            Screen::Help,
            Screen::Settings,
        ] {
            let mut app = App::new();
            app.screen = screen;
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|f| draw(f, &app)).unwrap();
        }
    }

    #[test]
    fn draw_below_min_size_shows_hint_instead_of_panicking() {
        let app = App::new();
        let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
    }
}
