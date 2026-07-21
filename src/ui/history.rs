//! Renders the History overlay: a scrollable list of past results.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::App;

use super::*;

pub(super) fn draw_history(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 80, f.area());
    let [title_a, _, summary_a, _, header_a, entries_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
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
            "history",
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    if !app.history.is_empty() {
        let best = app.history.iter().map(|e| e.wpm).fold(0.0f64, f64::max);
        let avg = app.history.iter().map(|e| e.wpm).sum::<f64>() / app.history.len() as f64;
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("best ", Style::default().fg(th_dim())),
                Span::styled(format!("{best:.0}"), Style::default().fg(th_fg())),
                Span::styled(" wpm", Style::default().fg(th_dim())),
                Span::styled("     ", Style::default()),
                Span::styled("avg ", Style::default().fg(th_dim())),
                Span::styled(format!("{avg:.0}"), Style::default().fg(th_fg())),
                Span::styled(" wpm", Style::default().fg(th_dim())),
                Span::styled("     ", Style::default()),
                Span::styled(
                    format!("{} tests", app.history.len()),
                    Style::default().fg(th_dim()),
                ),
            ]))
            .alignment(Alignment::Center),
            summary_a,
        );
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            col("wpm", 6, th_dim()),
            col("acc", 7, th_dim()),
            col("lang", 11, th_dim()),
            col("mode", 10, th_dim()),
            col("when", 10, th_dim()),
        ])),
        header_a,
    );

    let max_rows = entries_a.height as usize;
    let lines: Vec<Line> = if app.history.is_empty() {
        vec![Line::from(Span::styled(
            "no results yet",
            Style::default().fg(th_dim()),
        ))]
    } else {
        app.history
            .iter()
            .skip(app.history_scroll)
            .take(max_rows)
            .map(|e| {
                Line::from(vec![
                    col(format!("{:.0}", e.wpm), 6, th_accent()),
                    col(format!("{:.1}%", e.accuracy), 7, th_fg()),
                    col(
                        if e.language.is_empty() {
                            "—".to_string()
                        } else {
                            e.language.clone()
                        },
                        11,
                        th_pending(),
                    ),
                    col(&e.mode, 10, th_fg()),
                    col(e.time_ago(), 10, th_pending()),
                ])
            })
            .collect()
    };
    f.render_widget(Paragraph::new(lines), entries_a);

    let scroll_hint = if app.history.len() > max_rows {
        format!("  {}/{}", app.history_scroll + 1, app.history.len())
    } else {
        String::new()
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("↑/↓"),
            Span::raw(" scroll"),
            sep(),
            kh("esc"),
            Span::raw(" back"),
            Span::styled(scroll_hint, Style::default().fg(th_dim())),
        ]))
        .style(Style::default().fg(th_dim()))
        .alignment(Alignment::Center),
        pin_footer(f.area(), 1),
    );
}
