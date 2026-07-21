//! Rendering entry point: `draw()` dispatches to the per-screen submodules
//! below based on `App::screen`. Reads `App` + `GameState`, never mutates
//! them — all state changes happen in `app::handle_*`.

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

pub use help::line_count as help_line_count;
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
        crate::app::Screen::Help => help::draw_help(f, app),
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
    let inner = dialog_block(
        f,
        centered_block(f.area(), pct(f.area().width, 40), 5),
        None,
    );
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

/// Clears `area`, draws the standard bordered popup chrome (optionally titled),
/// and returns the padded inner `Rect` for content.
pub(super) fn dialog_block(f: &mut Frame, area: Rect, title: Option<Span<'static>>) -> Rect {
    f.render_widget(Clear, area);
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th_dim()))
        .style(Style::default().bg(th_bg()));
    if let Some(title) = title {
        block = block.title(title);
    }
    f.render_widget(block, area);
    Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    }
}

pub(super) fn pin_footer(frame: Rect, height: u16) -> Rect {
    Rect {
        x: frame.x,
        y: frame.bottom().saturating_sub(height),
        width: frame.width,
        height: height.min(frame.height),
    }
}

/// A `width`×`height` rect centred in `frame`, clamped to it.
///
/// Prefer this over a percentage-sized rect whenever the content has a fixed
/// size. A percentage rect silently starves a fixed layout on small
/// terminals: the solver collapses rows to zero height, and the
/// `let [..] = split[..] else` guards never fire because the rect *count*
/// still matches. Here the content gets the size it asks for unless the
/// terminal itself is smaller.
pub(super) fn centered_block(frame: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(frame.width);
    let height = height.min(frame.height);
    Rect {
        x: frame.x + frame.width.saturating_sub(width) / 2,
        y: frame.y + frame.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

/// `percent`% of `total`, for blocks that scale with the frame. Pair it with
/// `.max(..)` when the content has a width below which it clips.
pub(super) fn pct(total: u16, percent: u16) -> u16 {
    total * percent / 100
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

/// A row of selectable labels: the one at `selected` accented + bold +
/// underlined, the rest dimmed, joined by two spaces. The menu's time/word
/// presets, its word-pool size row, and its quote-filter row each carried an
/// identical copy of this styling.
pub(super) fn label_row(
    labels: impl IntoIterator<Item = String>,
    selected: usize,
) -> Vec<Span<'static>> {
    let mut spans = vec![];
    for (i, label) in labels.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(if i == selected {
            Span::styled(
                label,
                Style::default()
                    .fg(th_accent())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
        } else {
            Span::styled(label, Style::default().fg(th_pending()))
        });
    }
    spans
}

/// [`label_row`] over values rendered as `{value}{suffix}`.
pub(super) fn option_spans<T: std::fmt::Display>(
    opts: &[T],
    selected: usize,
    suffix: &str,
) -> Vec<Span<'static>> {
    label_row(opts.iter().map(|v| format!("{v}{suffix}")), selected)
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
mod helper_tests {
    use super::*;

    /// `label_row` is the one place the menu's "selected option" styling lives
    /// now, so pin its contract: exactly one accented+underlined label, two
    /// spaces between labels, and no trailing separator.
    #[test]
    fn label_row_accents_only_the_selected_label() {
        let spans = label_row(["a", "b", "c"].map(String::from), 1);
        assert_eq!(spans.len(), 5, "3 labels + 2 separators: {spans:?}");
        assert_eq!(spans[1].content, "  ");
        assert_eq!(spans[3].content, "  ");

        assert_eq!(spans[2].content, "b");
        assert_eq!(spans[2].style.fg, Some(th_accent()));
        assert!(
            spans[2]
                .style
                .add_modifier
                .contains(Modifier::BOLD | Modifier::UNDERLINED)
        );

        for i in [0, 4] {
            assert_eq!(spans[i].style.fg, Some(th_pending()));
            assert!(!spans[i].style.add_modifier.contains(Modifier::UNDERLINED));
        }
    }

    #[test]
    fn option_spans_appends_the_suffix_to_every_value() {
        let spans = option_spans(&[15u64, 30], 0, "s");
        assert_eq!(spans[0].content, "15s");
        assert_eq!(spans[2].content, "30s");
    }

    #[test]
    fn label_row_with_no_labels_is_empty() {
        assert!(label_row(std::iter::empty(), 0).is_empty());
    }
}

/// Render helpers shared by every `ui/*` test module. `ui/mod.rs`,
/// `ui/menu.rs` and `ui/settings.rs` each carried their own copy of this
/// buffer-to-string loop; they differed only in which `draw_*` they called
/// and in the fixed 80×24 size, so both are parameters here.
#[cfg(test)]
pub(crate) mod test_render {
    use ratatui::{Frame, Terminal, backend::TestBackend};

    /// One `String` per terminal row.
    pub(crate) fn rows(w: u16, h: u16, draw: impl FnOnce(&mut Frame)) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(draw).unwrap();
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect()
    }

    /// [`rows`] joined by newlines, for `contains` assertions over the screen.
    pub(crate) fn text(w: u16, h: u16, draw: impl FnOnce(&mut Frame)) -> String {
        rows(w, h, draw).join("\n")
    }
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

    fn rendered_text(app: &App) -> String {
        super::test_render::text(80, 24, |f| draw(f, app))
    }

    /// Both pickers share one scaffold (`draw_picker`), so this pins the parts
    /// that scaffold owns: the title, the search line with its match counter,
    /// and the `▶` marker on the highlighted row.
    #[test]
    fn lang_picker_renders_title_search_counter_and_selection() {
        let mut app = App::new();
        app.menu.lang_picker = Some(crate::app::LangPicker {
            cursor: 0,
            size_idx: 0,
            scroll: 0,
            search: "eng".to_string(),
        });
        let text = rendered_text(&app);
        assert!(text.contains("language"), "missing title: {text}");
        assert!(text.contains("eng_"), "missing search line: {text}");
        assert!(text.contains("english"), "missing filtered row: {text}");
        assert!(text.contains('▶'), "missing selection marker: {text}");
    }

    #[test]
    fn theme_picker_renders_title_and_selection() {
        let mut app = App::new();
        let first = all_themes()[0].name;
        app.menu.theme_picker = Some(crate::app::ThemePicker {
            cursor: 0,
            scroll: 0,
            search: String::new(),
            original: first.to_string(),
        });
        let text = rendered_text(&app);
        assert!(text.contains("theme"), "missing title: {text}");
        assert!(text.contains(first), "missing first theme row: {text}");
        assert!(text.contains('▶'), "missing selection marker: {text}");
    }

    #[test]
    fn draw_below_min_size_shows_hint_instead_of_panicking() {
        let app = App::new();
        let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
    }
}
