use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::sound::SoundPack;

use super::*;

pub(super) fn draw_settings(f: &mut Frame, app: &App) {
    let label_w = 16usize;
    let (sound_label, sound_active) = match &app.sound {
        Some(s) => (s.pack.label().to_string(), s.pack != SoundPack::Off),
        None => ("unavailable".to_string(), false),
    };
    let volume_label = match (&app.sound, &app.settings_state.volume_input) {
        (_, Some(buf)) => format!("{buf}_"),
        (Some(s), None) => format!("{}%", s.volume_pct),
        (None, _) => "-".to_string(),
    };
    let rows: Vec<(&str, String, bool)> = vec![
        (
            "cursor shape",
            app.settings.cursor_shape.label().into(),
            true,
        ),
        ("sound", sound_label, sound_active),
        ("volume (1-100)", volume_label, true),
        (
            "history expiry",
            app.settings.history_expiry.label().into(),
            app.settings.history_expiry != crate::history::HistoryExpiry::Off,
        ),
        (
            "difficulty",
            app.settings.difficulty.label().into(),
            app.settings.difficulty != crate::game::Difficulty::Normal,
        ),
    ];

    let height = (rows.len() + 5) as u16;
    let area = centered_rect(40, 0, f.area());
    let area = Rect {
        x: area.x,
        y: f.area().height.saturating_sub(height) / 2,
        width: area.width,
        height: height.min(f.area().height),
    };

    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_DIM))
            .style(Style::default().bg(BG)),
        area,
    );

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };
    let [title_a, _gap_a, items_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner)[..] else {
        return;
    };

    let changed: [bool; 5] = if let Some((snap, snap_pack, snap_vol)) = &app.settings_state.snapshot
    {
        [
            app.settings.cursor_shape != snap.cursor_shape,
            app.sound
                .as_ref()
                .map(|s| s.pack != *snap_pack)
                .unwrap_or(false),
            app.sound
                .as_ref()
                .map(|s| s.volume_pct != *snap_vol)
                .unwrap_or(false),
            app.settings.history_expiry != snap.history_expiry,
            app.settings.difficulty != snap.difficulty,
        ]
    } else {
        [false; 5]
    };
    let title = if changed.iter().any(|&c| c) {
        "settings [*]"
    } else {
        "settings"
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            title,
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .map(|(i, (label, val, active))| {
            let unavailable = (i == 3 || i == 4) && app.sound.is_none();
            let cursor = if i == app.settings_state.cursor && !unavailable {
                Span::styled("> ", Style::default().fg(C_ACCENT))
            } else {
                Span::raw("  ")
            };
            let lbl = Span::styled(format!("{label:<label_w$}"), Style::default().fg(C_DIM));
            let val_span = if i == app.settings_state.cursor && !unavailable {
                Span::styled(
                    format!("< {val} >"),
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                )
            } else {
                toggle_span(val, *active)
            };
            let dirty = Span::styled(
                if changed[i] { " *" } else { "  " },
                Style::default().fg(C_DIM),
            );
            Line::from(vec![cursor, lbl, val_span, dirty])
        })
        .collect();

    f.render_widget(Paragraph::new(lines), items_a);

    let footer = if app.settings_state.pending_exit {
        Paragraph::new(Line::from(vec![
            Span::styled("discard changes?  ", Style::default().fg(C_WRONG)),
            kh("y"),
            Span::styled(" yes", Style::default().fg(C_WRONG)),
            sep(),
            kh("n"),
            Span::raw(" no"),
        ]))
        .style(Style::default().fg(C_DIM))
        .alignment(Alignment::Center)
    } else {
        let any_changed = changed.iter().any(|&c| c);
        let mut spans = vec![
            kh("↑↓"),
            sep(),
            kh("←→"),
            Span::raw(" change"),
            sep(),
            kh("enter"),
            Span::raw(" save"),
        ];
        if any_changed {
            spans.extend([sep(), kh("esc"), Span::raw(" discard")]);
        }
        Paragraph::new(Line::from(spans))
            .style(Style::default().fg(C_DIM))
            .alignment(Alignment::Center)
    };
    f.render_widget(footer, pin_footer(f.area(), 1));
}
