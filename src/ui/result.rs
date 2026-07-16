use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Chart, Dataset, GraphType, Paragraph},
};

use crate::app::App;
use crate::game::CharState;
use crate::words::LANGUAGES;

use super::*;

pub(super) fn draw_result(f: &mut Frame, app: &App) {
    let area = centered_rect(90, 70, f.area());
    let [main_a, _, _] = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area)[..] else {
        return;
    };

    let [left_a, chart_a, right_a] = Layout::horizontal([
        Constraint::Percentage(26),
        Constraint::Percentage(48),
        Constraint::Percentage(26),
    ])
    .split(main_a)[..] else {
        return;
    };

    draw_left_panel(f, left_a, app);
    draw_right_panel(f, right_a, app);
    draw_chart(f, chart_a, app);
    draw_footer(f, pin_footer(f.area(), 1));
}

fn draw_left_panel(f: &mut Frame, area: Rect, app: &App) {
    let failed = app.game.is_failed();
    let wpm = app.game.wpm();
    let raw = app.game.raw_wpm();
    let acc = app.game.accuracy();
    let mode_str = app.game.mode.to_string();
    let lang = LANGUAGES
        .get(app.game.settings.lang_idx)
        .map(|l| l.name)
        .unwrap_or("unknown");

    let [lwpm_a, lacc_a, _, ltype_a, _, lraw_a, src_a] = Layout::vertical([
        Constraint::Length(2), // wpm
        Constraint::Length(2), // acc
        Constraint::Length(1), // gap
        Constraint::Length(3), // test type
        Constraint::Length(1), // gap
        Constraint::Length(2), // raw
        Constraint::Min(1),    // quote source (or spacer)
    ])
    .split(area)[..] else {
        return;
    };

    let wpm_color = if failed { th_wrong() } else { th_accent() };
    let wpm_line = if app.is_new_pb && !failed {
        Line::from(vec![
            Span::styled(
                format!("{wpm:.0}"),
                Style::default().fg(wpm_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ★", Style::default().fg(th_accent())),
        ])
    } else {
        Line::from(Span::styled(
            if failed {
                "0".to_string()
            } else {
                format!("{wpm:.0}")
            },
            Style::default().fg(wpm_color).add_modifier(Modifier::BOLD),
        ))
    };
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("wpm", Style::default().fg(th_dim()))),
            wpm_line,
        ]),
        lwpm_a,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("acc", Style::default().fg(th_dim()))),
            Line::from(Span::styled(
                format!("{acc:.1}%"),
                Style::default()
                    .fg(if failed { th_wrong() } else { th_accent() })
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        lacc_a,
    );
    let diff = app.game.settings.difficulty;
    let mut type_lines = vec![
        Line::from(Span::styled("test type", Style::default().fg(th_dim()))),
        Line::from(Span::styled(mode_str, Style::default().fg(th_accent()))),
        Line::from(Span::styled(lang, Style::default().fg(th_accent()))),
    ];
    if diff != crate::game::Difficulty::Normal {
        type_lines.push(Line::from(Span::styled(
            diff.label(),
            Style::default().fg(th_accent()),
        )));
    }
    if let Some(reason) = app.game.fail_reason() {
        type_lines.push(Line::from(Span::styled(
            format!("invalid ({reason})"),
            Style::default().fg(th_wrong()),
        )));
    }
    f.render_widget(Paragraph::new(type_lines), ltype_a);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("raw", Style::default().fg(th_dim()))),
            Line::from(Span::styled(
                format!("{raw:.0}"),
                Style::default().fg(th_fg()).add_modifier(Modifier::BOLD),
            )),
        ]),
        lraw_a,
    );
    if let Some(src) = &app.game.quote_source {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!("— {src}"),
                Style::default()
                    .fg(th_pending())
                    .add_modifier(Modifier::ITALIC),
            )),
            src_a,
        );
    }
}

fn draw_right_panel(f: &mut Frame, area: Rect, app: &App) {
    let cons = app.game.consistency();
    let elapsed = app.game.elapsed().as_secs_f64();
    let correct = app
        .game
        .chars
        .iter()
        .filter(|c| c.state == CharState::Correct)
        .count();
    let wrong = app
        .game
        .chars
        .iter()
        .filter(|c| c.state == CharState::Wrong)
        .count();

    let [rchars_a, _, rcons_a, _, rtime_a] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(4),
    ])
    .split(area)[..] else {
        return;
    };

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("characters", Style::default().fg(th_dim()))),
            Line::from(Span::styled(
                format!("{correct}/{wrong}"),
                Style::default()
                    .fg(th_accent())
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Right),
        rchars_a,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("consistency", Style::default().fg(th_dim()))),
            Line::from(Span::styled(
                format!("{cons:.0}%"),
                Style::default()
                    .fg(th_accent())
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Right),
        rcons_a,
    );
    let session_secs = app.result_session_secs;
    let afk = app.game.afk_secs;
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("time", Style::default().fg(th_dim()))),
            Line::from(Span::styled(
                format!("{elapsed:.1}s"),
                Style::default()
                    .fg(th_accent())
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "{:02}:{:02}:{:02} session",
                    session_secs / 3600,
                    session_secs % 3600 / 60,
                    session_secs % 60
                ),
                Style::default().fg(th_dim()),
            )),
            Line::from(Span::styled(
                format!(
                    "{:.0}% afk",
                    if elapsed > 0.0 {
                        (afk / elapsed * 100.0).min(100.0)
                    } else {
                        0.0
                    }
                ),
                Style::default().fg(th_dim()),
            )),
        ])
        .alignment(Alignment::Right),
        rtime_a,
    );
}

fn draw_chart(f: &mut Frame, area: Rect, app: &App) {
    let samples = &app.game.wpm_samples;
    if samples.len() < 2 {
        return;
    }
    let [chart_body_a, legend_a] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area)[..]
    else {
        return;
    };

    let n = samples.len();
    let max_wpm = samples
        .iter()
        .cloned()
        .chain(app.game.raw_wpm_samples.iter().cloned())
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let y_max = (max_wpm * 1.25).ceil();

    let burst_data: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, &w)| (i as f64, w))
        .collect();
    let raw_data: Vec<(f64, f64)> = app
        .game
        .raw_wpm_samples
        .iter()
        .enumerate()
        .map(|(i, &w)| (i as f64, w))
        .collect();

    let scale_data: Vec<(f64, f64)> = samples
        .windows(3)
        .enumerate()
        .map(|(i, w)| (i as f64 + 1.0, w.iter().sum::<f64>() / w.len() as f64))
        .collect();

    let err_y = y_max * 0.06;
    let err_data: Vec<(f64, f64)> = app
        .game
        .error_samples
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d > 0)
        .map(|(i, _)| (i as f64, err_y))
        .collect();

    let x_max = (n - 1) as f64;
    let y_labels: Vec<Line> = (0..=4)
        .map(|i| {
            Line::from(Span::styled(
                format!("{:.0}", y_max * i as f64 / 4.0),
                Style::default().fg(th_dim()),
            ))
        })
        .collect();
    let x_labels = vec![
        Line::from(Span::styled("1", Style::default().fg(th_dim()))),
        Line::from(Span::styled(
            if n <= 2 {
                String::new()
            } else {
                format!("{}", n / 2)
            },
            Style::default().fg(th_dim()),
        )),
        Line::from(Span::styled(format!("{n}"), Style::default().fg(th_dim()))),
    ];

    // Distinct markers per series so the lines are also distinguishable by
    // texture, not color alone (accessibility for colorblind/monochrome).
    let mut datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(th_pending()))
            .data(&burst_data),
        Dataset::default()
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(th_sub()))
            .data(&raw_data),
    ];
    if scale_data.len() >= 2 {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Block)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(th_accent()))
                .data(&scale_data),
        );
    }
    if !err_data.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(th_wrong()))
                .data(&err_data),
        );
    }

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(x_labels)
                .style(Style::default().fg(th_dim())),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(y_labels)
                .style(Style::default().fg(th_dim())),
        );

    f.render_widget(chart, chart_body_a);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("─ burst  ", Style::default().fg(th_pending())),
            Span::styled("⋯ raw  ", Style::default().fg(th_sub())),
            Span::styled("▬ scale  ", Style::default().fg(th_accent())),
            Span::styled("• errors", Style::default().fg(th_wrong())),
        ]))
        .alignment(Alignment::Center),
        legend_a,
    );
}

fn draw_footer(f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("r"),
            Span::raw(" repeat"),
            sep(),
            kh("enter/tab"),
            Span::raw(" restart"),
            sep(),
            kh("esc"),
            Span::raw(" menu"),
        ]))
        .style(Style::default().fg(th_dim()))
        .alignment(Alignment::Center),
        area,
    );
}
