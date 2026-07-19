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

pub(super) fn draw_help(f: &mut Frame) {
    let area = centered_rect(54, 90, f.area());
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

    let kw = |k: &'static str| Span::styled(format!("{k:<16}"), Style::default().fg(th_accent()));
    let dsc = |d: &'static str| Span::styled(d, Style::default().fg(th_fg()));
    let sec = |s: &'static str| {
        Line::from(Span::styled(
            s,
            Style::default().fg(th_dim()).add_modifier(Modifier::BOLD),
        ))
    };
    let row = |k, d| Line::from(vec![kw(k), dsc(d)]);

    f.render_widget(
        Paragraph::new(vec![
            sec("── menu ──────────────────────────────────────"),
            row("1 / 2 / 3", "select mode  (time · words · quote)"),
            row("← / →", "change option value"),
            row("enter", "start test  (or open custom input)"),
            row("l", "open language picker"),
            row("t", "open theme picker"),
            row("p", "toggle punctuation"),
            row("n", "toggle numbers"),
            row("s", "settings  (sound, volume, …)"),
            row("h", "history"),
            row("? ", "this help"),
            row("q  /  ctrl+c", "quit"),
            Line::from(""),
            sec("── test ────────────────────────────────────────"),
            row("tab", "restart test (new words)"),
            row("esc", "back to menu"),
            row("backspace", "delete last character"),
            row("ctrl+backspace", "delete whole word"),
            Line::from(""),
            sec("── result ──────────────────────────────────────"),
            row("r", "repeat same words"),
            row("enter / tab", "new test (new words)"),
            row("esc", "back to menu"),
            Line::from(""),
            sec("── language picker ─────────────────────────────"),
            row("↑ / ↓", "navigate languages"),
            row("← / →", "change word pool size"),
            row("enter", "confirm selection"),
            row("esc", "cancel"),
            Line::from(""),
            sec("── theme picker ────────────────────────────────"),
            row("↑ / ↓", "preview themes live"),
            row("type", "search by name"),
            row("enter", "keep theme"),
            row("esc", "cancel  (restore previous)"),
        ]),
        body_a,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![kh("esc"), Span::raw(" back")]))
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
