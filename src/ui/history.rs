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
            // Blank stand-in for the row marker, so the columns line up.
            Span::raw("  "),
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
            .enumerate()
            .skip(app.history_scroll)
            .take(max_rows)
            .map(|(i, e)| {
                // Same selection marker the pickers use, so "the highlighted
                // row" looks the same everywhere in the app.
                let selected = i == app.history_cursor;
                Line::from(vec![
                    Span::styled(
                        if selected { "▶ " } else { "  " },
                        Style::default().fg(th_accent()),
                    ),
                    col(
                        format!("{:.0}", e.wpm),
                        6,
                        if selected { th_accent() } else { th_fg() },
                    ),
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
    draw_scrollbar(
        f,
        entries_a,
        app.history.len(),
        max_rows,
        app.history_scroll,
    );

    // Position of the selection, not of the viewport — with a selected row the
    // count only means something if it tracks what is highlighted.
    let position = if app.history.is_empty() {
        String::new()
    } else {
        format!("  {}/{}", app.history_cursor + 1, app.history.len())
    };

    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("↑/↓"),
            Span::raw(" select"),
            sep(),
            kh("esc"),
            Span::raw(" back"),
            Span::styled(position, Style::default().fg(th_dim())),
        ]))
        .style(Style::default().fg(th_dim()))
        .alignment(Alignment::Center),
        pin_footer(f.area(), 1),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryEntry;
    use crate::ui::test_render::rows;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    /// The marker has to move on screen, not just in `App`: a list shorter
    /// than the viewport used to produce no visible change at all for ↑/↓.
    #[test]
    fn the_selection_marker_moves_with_the_arrow_keys() {
        let mut app = App::new();
        app.screen = crate::app::Screen::History;
        app.last_height = 24;
        app.history = (0..3)
            .map(|i| HistoryEntry {
                wpm: 60.0 + i as f64,
                accuracy: 95.0,
                mode: "words 25".into(),
                timestamp: 0,
                language: "english".into(),
            })
            .collect();

        let marked = |app: &App| {
            rows(80, 24, |f| draw_history(f, app))
                .into_iter()
                .position(|r| r.contains('▶'))
        };

        let first = marked(&app).expect("a row should be marked");
        app.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        let second = marked(&app).expect("a row should still be marked");
        assert_eq!(second, first + 1, "the marker should have moved down a row");
    }
}
