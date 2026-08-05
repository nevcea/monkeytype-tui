//! Renders the Settings screen: the editable-rows list, with sound/volume
//! rows shown as unavailable when no audio device is present.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, SettingsRow};
use crate::sound::SoundPack;

use super::*;

/// Width floor for the settings popup; see the comment at its use below.
const SETTINGS_MIN_WIDTH: u16 = 44;

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
    // Rows follow `SettingsRow::ORDER` so this list and the key handling in
    // `app::settings` can never drift out of order.
    let rows: Vec<(&str, String, bool)> = SettingsRow::ORDER
        .iter()
        .map(|row| match row {
            SettingsRow::CursorShape => (
                "cursor shape",
                app.settings.cursor_shape.label().to_string(),
                true,
            ),
            SettingsRow::Sound => ("sound", sound_label.clone(), sound_active),
            SettingsRow::Volume => ("volume (1-100)", volume_label.clone(), true),
            SettingsRow::Difficulty => (
                "difficulty",
                app.settings.difficulty.label().to_string(),
                app.settings.difficulty != crate::game::Difficulty::Normal,
            ),
            SettingsRow::Theme => (
                "theme",
                theme_by_name(&app.settings.theme_name).name.to_string(),
                app.settings.theme_name != crate::game::DEFAULT_THEME,
            ),
        })
        .collect();

    let height = (rows.len() + 5) as u16;
    // Floored so the longest built-in theme name ("solarized_dark", 14 chars)
    // still fits inside the `< {label:<16} value >` row at 80% or 40%. At
    // MIN_WIDTH (60), 40% alone gives an inner width of 20 — well under the
    // ~38 columns that row needs.
    let width = pct(f.area().width, 40).max(SETTINGS_MIN_WIDTH);
    let inner = dialog_block(f, centered_block(f.area(), width, height), None);
    let [title_a, _gap_a, items_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(inner)[..] else {
        return;
    };

    let changed: Vec<bool> = SettingsRow::ORDER
        .iter()
        .map(|row| {
            let Some((snap, snap_pack, snap_vol)) = &app.settings_state.snapshot else {
                return false;
            };
            match row {
                SettingsRow::CursorShape => app.settings.cursor_shape != snap.cursor_shape,
                SettingsRow::Sound => app.sound.as_ref().is_some_and(|s| s.pack != *snap_pack),
                SettingsRow::Volume => app
                    .sound
                    .as_ref()
                    .is_some_and(|s| s.volume_pct != *snap_vol),
                SettingsRow::Difficulty => app.settings.difficulty != snap.difficulty,
                SettingsRow::Theme => app.settings.theme_name != snap.theme_name,
            }
        })
        .collect();
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
            let unavailable = SettingsRow::ORDER[i].needs_audio() && app.sound.is_none();
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

    fn row_index(row: SettingsRow) -> usize {
        SettingsRow::ORDER.iter().position(|&r| r == row).unwrap()
    }

    /// `SettingsRow::ORDER` drives both this screen's rows and the key
    /// handling in `app::settings`. If an entry is ever duplicated or dropped
    /// from ORDER, `from_index` stops agreeing with position and the two
    /// modules silently disagree about which row the cursor is on.
    #[test]
    fn row_order_and_from_index_agree() {
        for (i, &row) in SettingsRow::ORDER.iter().enumerate() {
            assert_eq!(SettingsRow::from_index(i), Some(row));
            assert_eq!(row_index(row), i, "duplicate entry in ORDER for {row:?}");
        }
        assert_eq!(SettingsRow::from_index(SettingsRow::ORDER.len()), None);
    }

    /// Every row in ORDER must render a line, so the rendered list and the
    /// cursor range the key handler allows stay the same length.
    #[test]
    fn every_row_renders_a_line() {
        let app = App::new();
        let lines = render_lines(&app);
        for label in ["cursor shape", "sound", "volume", "difficulty", "theme"] {
            assert!(
                lines.iter().any(|l| l.contains(label)),
                "row {label} missing from render: {lines:?}"
            );
        }
    }

    fn render_lines(app: &App) -> Vec<String> {
        crate::ui::test_render::rows(80, 24, |f| draw_settings(f, app))
    }

    /// Regression test for 4ac7f07: when no audio device is available, the
    /// sound/volume rows must render as unavailable (no `< … >` edit
    /// indicator) while unrelated rows (cursor shape/difficulty) stay
    /// editable, no matter which row the cursor is actually on.
    #[test]
    fn sound_unavailable_rows_never_show_edit_indicator() {
        let mut app = App::new();
        app.sound = None;
        for cursor in [
            row_index(SettingsRow::Sound),
            row_index(SettingsRow::Volume),
        ] {
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
        for cursor in [
            row_index(SettingsRow::CursorShape),
            row_index(SettingsRow::Difficulty),
        ] {
            app.settings_state.cursor = cursor;
            let lines = render_lines(&app);
            assert!(
                lines.iter().any(|l| l.contains("< ") && l.contains('>')),
                "row {cursor} should stay editable when sound is unavailable: {lines:?}"
            );
        }
    }

    /// Regression test: at `MIN_WIDTH`, a 40%-of-frame popup with no floor
    /// gave only 28 usable columns — too narrow for the longest built-in
    /// theme name ("solarized_dark") in a selected `< … >` row, clipping the
    /// closing `>` and dirty marker.
    #[test]
    fn selected_row_with_a_long_theme_name_is_not_clipped_at_min_width() {
        let mut app = App::new();
        app.settings.theme_name = "solarized_dark".to_string();
        app.settings_state.cursor = row_index(SettingsRow::Theme);
        let lines =
            crate::ui::test_render::rows(crate::app::MIN_WIDTH, 24, |f| draw_settings(f, &app));
        assert!(
            lines.iter().any(|l| l.contains("< solarized_dark >")),
            "theme value clipped at MIN_WIDTH: {lines:?}"
        );
    }
}
