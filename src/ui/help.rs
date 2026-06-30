use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, LANG_PICKER_VISIBLE, filtered_languages};
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
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    let kw = |k: &'static str| Span::styled(format!("{k:<16}"), Style::default().fg(C_ACCENT));
    let dsc = |d: &'static str| Span::styled(d, Style::default().fg(C_FG));
    let sec = |s: &'static str| {
        Line::from(Span::styled(
            s,
            Style::default().fg(C_DIM).add_modifier(Modifier::BOLD),
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
        ]),
        body_a,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![kh("esc"), Span::raw(" back")]))
            .style(Style::default().fg(C_DIM))
            .alignment(Alignment::Center),
        pin_footer(f.area(), 1),
    );
}

pub(super) fn draw_lang_picker(f: &mut Frame, app: &App) {
    let picker = match &app.lang_picker {
        Some(p) => p,
        None => return,
    };
    const VISIBLE: usize = LANG_PICKER_VISIBLE;

    let filtered = filtered_languages(&picker.search);

    let area = centered_rect(54, 75, f.area());

    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_DIM))
            .title(Span::styled(
                " language ",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(BG)),
        area,
    );

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

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
            Span::styled(format!("▶ {}_", picker.search), Style::default().fg(C_FG)),
            Span::styled(
                format!(" ({}/{})", filtered.len(), LANGUAGES.len()),
                Style::default().fg(C_DIM),
            ),
        ])),
        search_a,
    );

    let visible_langs: Vec<Line> = filtered
        .iter()
        .enumerate()
        .skip(picker.scroll)
        .take(VISIBLE)
        .map(|(fi, (_, lang))| {
            let selected = fi == picker.cursor;
            let name_style = if selected {
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(C_FG)
            };
            let prefix = if selected { "▶ " } else { "  " };

            let mut spans = vec![
                Span::styled(prefix, Style::default().fg(C_ACCENT)),
                Span::styled(format!("{:<12}", lang.name), name_style),
                Span::styled("  ", Style::default()),
            ];

            for (si, sz) in lang.sizes.iter().enumerate() {
                let size_style = if selected && si == picker.size_idx {
                    Style::default().fg(BG).bg(C_ACCENT)
                } else if selected {
                    Style::default().fg(C_PENDING)
                } else {
                    Style::default().fg(C_DIM)
                };
                spans.push(Span::styled(sz.label, size_style));
                if si + 1 < lang.sizes.len() {
                    spans.push(Span::styled("  ", Style::default()));
                }
            }
            if matches!(app.menu_mode, Mode::Quote) && lang.quotes.is_none() {
                spans.push(Span::styled("  no quotes", Style::default().fg(C_WRONG)));
            }
            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(visible_langs), list_a);

    let total = filtered.len();
    let scroll_info = if total > VISIBLE {
        format!(" {}/{total} ", picker.cursor + 1)
    } else {
        String::new()
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("↑/↓"),
            Span::raw(" navigate"),
            sep(),
            kh("←/→"),
            Span::raw(" size"),
            sep(),
            kh("enter"),
            Span::raw(" select"),
            sep(),
            kh("esc"),
            Span::raw(" cancel"),
            Span::styled(scroll_info, Style::default().fg(C_DIM)),
        ]))
        .style(Style::default().fg(C_DIM))
        .alignment(Alignment::Center),
        footer_a,
    );
}
