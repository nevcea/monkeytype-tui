//! Renders the Settings screen: the editable-rows list, with sound/volume
//! rows shown as unavailable when no audio device is present.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
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
        (
            "theme",
            theme_by_name(&app.settings.theme_name).name.into(),
            app.settings.theme_name != crate::game::DEFAULT_THEME,
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

    let inner = dialog_block(f, area, None);
    let [title_a, _gap_a, items_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner)[..] else {
        return;
    };

    let changed: [bool; 6] = if let Some((snap, snap_pack, snap_vol)) = &app.settings_state.snapshot
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
            app.settings.theme_name != snap.theme_name,
        ]
    } else {
        [false; 6]
    };
    let title = if changed.iter().any(|&c| c) {
        "settings [*]"
    } else {
        "settings"
    };
    f.render_widget(
        Paragraph::new(Span::styled(
            title,
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    let lines: Vec<Line> = rows
        .iter()
        .enumerate()
        .map(|(i, (label, val, active))| {
            let unavailable = (i == 1 || i == 2) && app.sound.is_none();
            let cursor = if i == app.settings_state.cursor && !unavailable {
                Span::styled("> ", Style::default().fg(th_accent()))
            } else {
                Span::raw("  ")
            };
            let lbl = Span::styled(format!("{label:<label_w$}"), Style::default().fg(th_dim()));
            let val_span = if i == app.settings_state.cursor && !unavailable {
                Span::styled(
                    format!("< {val} >"),
                    Style::default()
                        .fg(th_accent())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                toggle_span(val, *active)
            };
            let dirty = Span::styled(
                if changed[i] { " *" } else { "  " },
                Style::default().fg(th_dim()),
            );
            Line::from(vec![cursor, lbl, val_span, dirty])
        })
        .collect();

    f.render_widget(Paragraph::new(lines), items_a);

    let footer = if app.settings_state.pending_exit {
        Paragraph::new(Line::from(vec![
            Span::styled("discard changes?  ", Style::default().fg(th_wrong())),
            kh("y"),
            Span::styled(" yes", Style::default().fg(th_wrong())),
            sep(),
            kh("n"),
            Span::raw(" no"),
        ]))
        .style(Style::default().fg(th_dim()))
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
            .style(Style::default().fg(th_dim()))
            .alignment(Alignment::Center)
    };
    f.render_widget(footer, pin_footer(f.area(), 1));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const ROW_SOUND: usize = 1;
    const ROW_VOLUME: usize = 2;
    const ROW_HISTORY_EXPIRY: usize = 3;
    const ROW_DIFFICULTY: usize = 4;

    fn render_lines(app: &App) -> Vec<String> {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| draw_settings(f, app)).unwrap();
        let buffer = terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect()
    }

    /// Regression test for 4ac7f07: when no audio device is available, the
    /// sound/volume rows must render as unavailable (no `< … >` edit
    /// indicator) while unrelated rows (history expiry/difficulty) stay
    /// editable, no matter which row the cursor is actually on.
    #[test]
    fn sound_unavailable_rows_never_show_edit_indicator() {
        let mut app = App::new();
        app.sound = None;
        for cursor in [ROW_SOUND, ROW_VOLUME] {
            app.settings_state.cursor = cursor;
            let lines = render_lines(&app);
            assert!(
                !lines.iter().any(|l| l.contains("< ") && l.contains('>')),
                "row {cursor} (sound/volume) rendered as editable with no sound device: {lines:?}"
            );
        }
    }

    #[test]
    fn unrelated_rows_stay_editable_when_sound_is_unavailable() {
        let mut app = App::new();
        app.sound = None;
        for cursor in [ROW_HISTORY_EXPIRY, ROW_DIFFICULTY] {
            app.settings_state.cursor = cursor;
            let lines = render_lines(&app);
            assert!(
                lines.iter().any(|l| l.contains("< ") && l.contains('>')),
                "row {cursor} should stay editable when sound is unavailable: {lines:?}"
            );
        }
    }
}
