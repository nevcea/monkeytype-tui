//! Renders the Result screen: WPM/accuracy/consistency summary after a test.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Chart, Dataset, GraphType, Paragraph, Wrap},
};

use crate::app::App;
use crate::game::{CharState, Difficulty};
use crate::words::lang_name;

use super::*;

pub(super) fn draw_result(f: &mut Frame, app: &App) {
    let area = centered_rect(90, 70, f.area());
    let [main_a, src_a, _] = Layout::vertical([
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
    draw_quote_source(f, src_a, app);
    draw_footer(f, pin_footer(f.area(), 1));
}

/// The attribution belongs to the whole test, and at 26% of the screen the
/// left panel gave it 18 columns — "— Frank Herbert, Dune" came out as
/// "— Frank Herbert,". It spans the full result width here instead.
fn draw_quote_source(f: &mut Frame, area: Rect, app: &App) {
    let Some(src) = &app.game.quote_source else {
        return;
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("— {src}"),
            Style::default()
                .fg(th_pending())
                .add_modifier(Modifier::ITALIC),
        ))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center),
        area,
    );
}

/// A labelled result figure: a dim caption, the headline value, then any
/// supporting lines. The result screen stacks seven of these, and each one
/// used to repeat this same caption-over-value construction inline.
fn stat_block<'a>(
    label: &'static str,
    value: Line<'a>,
    extra: Vec<Line<'a>>,
    align: Alignment,
) -> Paragraph<'a> {
    let mut lines = vec![Line::from(Span::styled(
        label,
        Style::default().fg(th_dim()),
    ))];
    lines.push(value);
    lines.extend(extra);
    Paragraph::new(lines).alignment(align)
}

/// The bold headline line a stat block leads with.
fn value_line(text: String, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

/// A plain (non-bold) supporting line under a stat block's value.
fn note_line(text: String, color: Color) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(color)))
}

fn draw_left_panel(f: &mut Frame, area: Rect, app: &App) {
    let failed = app.game.is_failed();
    let lang = lang_name(app.game.settings.lang_idx);

    let [lwpm_a, lacc_a, _, ltype_a, _, lraw_a, _] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .split(area)[..] else {
        return;
    };

    // A failed test scores zero; an otherwise-clean personal best gets a star.
    let score_color = if failed { th_wrong() } else { th_accent() };
    let wpm_line = if failed {
        value_line("0".to_string(), score_color)
    } else if app.is_new_pb {
        Line::from(vec![
            Span::styled(
                format!("{:.0}", app.game.wpm()),
                Style::default()
                    .fg(score_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ★", Style::default().fg(th_accent())),
        ])
    } else {
        value_line(format!("{:.0}", app.game.wpm()), score_color)
    };
    f.render_widget(stat_block("wpm", wpm_line, vec![], Alignment::Left), lwpm_a);
    f.render_widget(
        stat_block(
            "acc",
            value_line(format!("{:.1}%", app.game.accuracy()), score_color),
            vec![],
            Alignment::Left,
        ),
        lacc_a,
    );

    let mut type_extra = vec![note_line(lang.to_string(), th_accent())];
    let diff = app.game.settings.difficulty;
    if diff != Difficulty::Normal {
        type_extra.push(note_line(diff.label().to_string(), th_accent()));
    }
    if let Some(reason) = app.game.fail_reason() {
        type_extra.push(note_line(format!("invalid ({reason})"), th_wrong()));
    }
    f.render_widget(
        stat_block(
            "test type",
            note_line(app.game.mode.to_string(), th_accent()),
            type_extra,
            Alignment::Left,
        ),
        ltype_a,
    );
    f.render_widget(
        stat_block(
            "raw",
            value_line(format!("{:.0}", app.game.raw_wpm()), th_fg()),
            vec![],
            Alignment::Left,
        ),
        lraw_a,
    );
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
        stat_block(
            "characters",
            value_line(format!("{correct}/{wrong}"), th_accent()),
            vec![],
            Alignment::Right,
        ),
        rchars_a,
    );
    f.render_widget(
        stat_block(
            "consistency",
            value_line(format!("{cons:.0}%"), th_accent()),
            vec![],
            Alignment::Right,
        ),
        rcons_a,
    );

    let session_secs = app.result_session_secs;
    let afk_pct = if elapsed > 0.0 {
        (app.game.afk_secs / elapsed * 100.0).min(100.0)
    } else {
        0.0
    };
    f.render_widget(
        stat_block(
            "time",
            value_line(format!("{elapsed:.1}s"), th_accent()),
            vec![
                note_line(
                    format!(
                        "{:02}:{:02}:{:02} session",
                        session_secs / 3600,
                        session_secs % 3600 / 60,
                        session_secs % 60
                    ),
                    th_dim(),
                ),
                note_line(format!("{afk_pct:.0}% afk"), th_dim()),
            ],
            Alignment::Right,
        ),
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
    // Series are 1-indexed on x so the axis labels below land on real points.
    let burst_data: Vec<(f64, f64)> = samples
        .iter()
        .enumerate()
        .map(|(i, &w)| (i as f64 + 1.0, w))
        .collect();
    let raw_data: Vec<(f64, f64)> = app
        .game
        .raw_wpm_samples
        .iter()
        .enumerate()
        .map(|(i, &w)| (i as f64 + 1.0, w))
        .collect();

    // Smoothed burst: a 3-wide sliding mean, x-shifted to centre each window.
    let scale_data: Vec<(f64, f64)> = samples
        .windows(3)
        .enumerate()
        .map(|(i, w)| (i as f64 + 2.0, w.iter().sum::<f64>() / w.len() as f64))
        .collect();
    // Short tests have no window to smooth over, so plot the raw bursts instead.
    let wpm_data = if scale_data.len() >= 2 {
        &scale_data
    } else {
        &burst_data
    };

    // Round the axis up to a whole number of ticks so all five labels are
    // integral (0 / 25 / 50 / 75 / 100) instead of arbitrary fractions.
    let peak = wpm_data
        .iter()
        .chain(raw_data.iter())
        .map(|&(_, y)| y)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let step = if peak > 200.0 {
        50.0
    } else if peak > 80.0 {
        25.0
    } else {
        10.0
    };
    let y_max = ((peak * 1.1) / (step * 4.0)).ceil() * step * 4.0;

    let err_y = y_max * 0.06;
    let err_data: Vec<(f64, f64)> = app
        .game
        .error_samples
        .iter()
        .enumerate()
        .filter(|&(_, &d)| d > 0)
        .map(|(i, _)| (i as f64 + 1.0, err_y))
        .collect();

    let x_max = n as f64;
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
                format!("{}", n.div_ceil(2))
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
            .style(Style::default().fg(th_accent()))
            .data(wpm_data),
        Dataset::default()
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(th_sub()))
            .data(&raw_data),
    ];
    let mut legend = vec![
        Span::styled("─ wpm", Style::default().fg(th_accent())),
        Span::raw("  "),
        Span::styled("⋯ raw", Style::default().fg(th_sub())),
    ];
    if !err_data.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(th_wrong()))
                .data(&err_data),
        );
        legend.push(Span::raw("  "));
        legend.push(Span::styled("• errors", Style::default().fg(th_wrong())));
    }

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .bounds([1.0, x_max])
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
        Paragraph::new(Line::from(legend)).alignment(Alignment::Center),
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
