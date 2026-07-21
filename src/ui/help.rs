//! Renders the Help overlay (keybinding reference).

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{
    App, LANG_PICKER_VISIBLE, THEME_PICKER_VISIBLE, filtered_languages, filtered_themes,
};
use crate::game::Mode;
use crate::words::LANGUAGES;

use super::*;

/// The keybinding reference, as data rather than pre-built `Line`s: the
/// scroll clamp needs the body's line count, and `app` can't build styled
/// lines (it has no access to the theme).
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "menu",
        &[
            ("1 / 2 / 3", "select mode  (time · words · quote)"),
            ("← / →", "change option value"),
            ("enter", "start test  (or open custom input)"),
            ("l", "open language picker"),
            ("t", "open theme picker"),
            ("[ / ]", "change word pool size"),
            ("p", "toggle punctuation"),
            ("n", "toggle numbers"),
            ("s", "settings  (sound, volume, …)"),
            ("h", "history"),
            ("?", "this help"),
            ("q  /  ctrl+c", "quit"),
        ],
    ),
    (
        "test",
        &[
            ("tab", "restart test (new words)"),
            ("esc", "back to menu"),
            ("backspace", "delete last character"),
            ("ctrl+backspace", "delete whole word"),
        ],
    ),
    (
        "result",
        &[
            ("r", "repeat same words"),
            ("enter / tab", "new test (new words)"),
            ("esc", "back to menu"),
        ],
    ),
    (
        "history",
        &[
            ("↑ / ↓", "move selection"),
            ("pgup / pgdn", "move by a page"),
            ("home / end", "first / last result"),
            ("esc", "back to menu"),
        ],
    ),
    (
        "language picker",
        &[
            ("↑ / ↓", "navigate languages"),
            ("← / →", "change word pool size"),
            ("enter", "confirm selection"),
            ("esc", "cancel"),
        ],
    ),
    (
        "theme picker",
        &[
            ("↑ / ↓", "preview themes live"),
            ("type", "search by name"),
            ("enter", "keep theme"),
            ("esc", "cancel  (restore previous)"),
        ],
    ),
];

/// Lines [`help_body`] produces. `App` clamps its scroll against this, so it
/// must stay derived from `HELP_SECTIONS` rather than hand-counted.
pub fn line_count() -> usize {
    let rows: usize = HELP_SECTIONS.iter().map(|(_, rows)| rows.len() + 1).sum();
    // One blank separator between consecutive sections.
    rows + HELP_SECTIONS.len().saturating_sub(1)
}

/// `── menu ─────…` filled to `width`. The rules used to be hardcoded
/// 46-column runs, which clipped in the ~43-column panel.
fn section_rule(title: &str, width: u16) -> Line<'static> {
    let head = format!("── {title} ");
    let fill = (width as usize).saturating_sub(head.chars().count());
    Line::from(Span::styled(
        format!("{head}{}", "─".repeat(fill)),
        Style::default().fg(th_dim()).add_modifier(Modifier::BOLD),
    ))
}

fn help_body(width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::with_capacity(line_count());
    for (i, (title, rows)) in HELP_SECTIONS.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        lines.push(section_rule(title, width));
        for (k, d) in *rows {
            lines.push(Line::from(vec![
                Span::styled(format!("{k:<16}"), Style::default().fg(th_accent())),
                Span::styled(*d, Style::default().fg(th_fg())),
            ]));
        }
    }
    lines
}

/// Columns the widest help row needs: the 16-column key gutter plus the
/// longest description ("select mode  (time · words · quote)", 35). Below
/// this the descriptions clip — 54% of an 80-column terminal is only 43.
const HELP_MIN_WIDTH: u16 = 52;

pub(super) fn draw_help(f: &mut Frame, app: &App) {
    let width = pct(f.area().width, 54).max(HELP_MIN_WIDTH);
    let area = centered_block(f.area(), width, pct(f.area().height, 90));
    let [title_a, _, body_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area)[..] else {
        return;
    };

    f.render_widget(
        Paragraph::new(Span::styled(
            "help",
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    // Clamp against the rect actually granted, not `App`'s approximation of
    // it — the two disagree by a row or two after a resize.
    let total = line_count();
    let visible = body_a.height as usize;
    let scroll = app.help_scroll.min(total.saturating_sub(visible));

    f.render_widget(
        Paragraph::new(help_body(body_a.width)).scroll((scroll as u16, 0)),
        body_a,
    );

    let mut hints = vec![kh("esc"), Span::raw(" back")];
    if total > visible {
        hints.extend([
            sep(),
            kh("↑/↓"),
            Span::raw(" scroll"),
            Span::styled(
                format!("   {}/{}", scroll + visible.min(total), total),
                Style::default().fg(th_dim()),
            ),
        ]);
    }
    f.render_widget(
        Paragraph::new(Line::from(hints))
            .style(Style::default().fg(th_dim()))
            .alignment(Alignment::Center),
        pin_footer(f.area(), 1),
    );
}

/// Everything the shared picker chrome needs that isn't the row content.
struct PickerChrome<'a> {
    title: &'a str,
    search: &'a str,
    cursor: usize,
    scroll: usize,
    visible: usize,
    /// Unfiltered total, shown as the `(matched/total)` counter.
    total: usize,
    /// Key hints between "navigate" and "cancel" (the language picker adds
    /// `←/→ size`, the theme picker doesn't).
    extra_hints: Vec<Span<'static>>,
}

/// Draws the standard picker overlay — bordered popup, search line with match
/// counter, scrolled list, and footer hints with a position indicator. Shared
/// by the language and theme pickers, which differ only in how a row renders.
///
/// `row` receives each visible item and whether it is the highlighted one.
fn draw_picker<T>(
    f: &mut Frame,
    chrome: &PickerChrome,
    items: &[T],
    row: impl Fn(&T, bool) -> Line<'static>,
) {
    let area = centered_rect(54, 75, f.area());
    let inner = dialog_block(
        f,
        area,
        Some(Span::styled(
            format!(" {} ", chrome.title),
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD),
        )),
    );

    let [search_a, _, list_a, _, footer_a] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(inner)[..] else {
        return;
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("▶ {}_", chrome.search),
                Style::default().fg(th_fg()),
            ),
            Span::styled(
                format!(" ({}/{})", items.len(), chrome.total),
                Style::default().fg(th_dim()),
            ),
        ])),
        search_a,
    );

    let rows: Vec<Line> = items
        .iter()
        .enumerate()
        .skip(chrome.scroll)
        .take(chrome.visible)
        .map(|(i, item)| row(item, i == chrome.cursor))
        .collect();
    f.render_widget(Paragraph::new(rows), list_a);

    let scroll_info = if items.len() > chrome.visible {
        format!(" {}/{} ", chrome.cursor + 1, items.len())
    } else {
        String::new()
    };

    let mut hints = vec![kh("↑/↓"), Span::raw(" navigate"), sep()];
    hints.extend(chrome.extra_hints.iter().cloned());
    hints.extend([
        kh("enter"),
        Span::raw(" select"),
        sep(),
        kh("esc"),
        Span::raw(" cancel"),
        Span::styled(scroll_info, Style::default().fg(th_dim())),
    ]);

    f.render_widget(
        Paragraph::new(Line::from(hints))
            .style(Style::default().fg(th_dim()))
            .alignment(Alignment::Center),
        footer_a,
    );
}

/// Row prefix + name styling shared by both pickers.
fn picker_name(name: &str, width: usize, selected: bool) -> Vec<Span<'static>> {
    let name_style = if selected {
        Style::default()
            .fg(th_accent())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(th_fg())
    };
    vec![
        Span::styled(
            if selected { "▶ " } else { "  " },
            Style::default().fg(th_accent()),
        ),
        Span::styled(format!("{name:<width$}"), name_style),
    ]
}

pub(super) fn draw_lang_picker(f: &mut Frame, app: &App) {
    let Some(picker) = &app.menu.lang_picker else {
        return;
    };
    let filtered = filtered_languages(&picker.search);
    let quote_mode = matches!(app.menu.mode, Mode::Quote);

    draw_picker(
        f,
        &PickerChrome {
            title: "language",
            search: &picker.search,
            cursor: picker.cursor,
            scroll: picker.scroll,
            visible: LANG_PICKER_VISIBLE,
            total: LANGUAGES.len(),
            extra_hints: vec![kh("←/→"), Span::raw(" size"), sep()],
        },
        &filtered,
        |(_, lang), selected| {
            let mut spans = picker_name(lang.name, 12, selected);
            spans.push(Span::styled("  ", Style::default()));

            for (si, sz) in lang.sizes.iter().enumerate() {
                let size_style = if selected && si == picker.size_idx {
                    Style::default().fg(th_bg()).bg(th_accent())
                } else if selected {
                    Style::default().fg(th_pending())
                } else {
                    Style::default().fg(th_dim())
                };
                spans.push(Span::styled(sz.label, size_style));
                if si + 1 < lang.sizes.len() {
                    spans.push(Span::styled("  ", Style::default()));
                }
            }
            if quote_mode && lang.quotes.is_none() {
                spans.push(Span::styled("  no quotes", Style::default().fg(th_wrong())));
            }
            Line::from(spans)
        },
    );
}

pub(super) fn draw_theme_picker(f: &mut Frame, app: &App) {
    let Some(picker) = &app.menu.theme_picker else {
        return;
    };
    let filtered = filtered_themes(&picker.search);

    // Each row previews the theme's own palette via colored swatches, so the
    // list is legible even before live-applying the highlighted theme.
    let swatch = |c| Span::styled("███", Style::default().fg(c));

    draw_picker(
        f,
        &PickerChrome {
            title: "theme",
            search: &picker.search,
            cursor: picker.cursor,
            scroll: picker.scroll,
            visible: THEME_PICKER_VISIBLE,
            total: all_themes().len(),
            extra_hints: vec![],
        },
        &filtered,
        |(_, t), selected| {
            let mut spans = picker_name(t.name, 16, selected);
            spans.extend([
                swatch(t.accent),
                Span::raw(" "),
                swatch(t.correct),
                Span::raw(" "),
                swatch(t.wrong),
                Span::raw(" "),
                swatch(t.sub),
            ]);
            Line::from(spans)
        },
    );
}

#[cfg(test)]
mod help_tests {
    use super::*;
    use crate::app::{MIN_HEIGHT, MIN_WIDTH, Screen};
    use crate::ui::test_render::text;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn help_app() -> App {
        let mut app = App::new();
        app.screen = Screen::Help;
        app.last_width = 80;
        app.last_height = 24;
        app
    }

    /// The body is 35 lines but only ~19 rows fit at 80x24, and `Paragraph`
    /// clips without complaint. With no scroll key, everything from the
    /// `result` section down — including both picker sections — was
    /// unreachable. Every section title must now be reachable.
    #[test]
    fn every_help_section_is_reachable_at_80x24() {
        let mut app = help_app();
        let top = text(80, 24, |f| draw_help(f, &app));
        assert!(top.contains("── menu"), "top of help missing:\n{top}");

        app.on_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        let bottom = text(80, 24, |f| draw_help(f, &app));
        for expected in ["── theme picker", "restore previous"] {
            assert!(
                bottom.contains(expected),
                "{expected:?} unreachable at the bottom of help:\n{bottom}"
            );
        }
    }

    /// The scroll must stop with the last line on screen rather than running
    /// off into blank rows, and must never go negative.
    #[test]
    fn help_scroll_is_clamped_at_both_ends() {
        let mut app = help_app();
        for _ in 0..200 {
            app.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        }
        assert_eq!(app.help_scroll, app.help_max_scroll());
        assert!(
            app.help_max_scroll() > 0,
            "help should need scrolling at 24 rows"
        );

        for _ in 0..200 {
            app.on_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        }
        assert_eq!(app.help_scroll, 0);
    }

    /// `line_count` drives `App`'s scroll clamp, so it must stay derived from
    /// `HELP_SECTIONS` rather than drifting from what `help_body` renders.
    #[test]
    fn line_count_matches_the_rendered_body() {
        assert_eq!(line_count(), help_body(40).len());
    }

    /// The panel is floored at `HELP_MIN_WIDTH` because 54% of an 80-column
    /// terminal is 43 — too narrow for the 16-column key gutter plus the
    /// longest description, which used to be cut off mid-word.
    #[test]
    fn the_longest_description_is_not_clipped() {
        let app = help_app();
        for (w, h) in [(80, 24), (MIN_WIDTH, MIN_HEIGHT)] {
            let screen = text(w, h, |f| draw_help(f, &app));
            assert!(
                screen.contains("select mode  (time · words · quote)"),
                "description clipped at {w}x{h}:\n{screen}"
            );
        }
    }
}
