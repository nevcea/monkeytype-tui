//! Renders the Test screen: progress gauge, wrapped word lines with
//! per-character coloring, and the live WPM/accuracy readout.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Gauge, Paragraph},
};

use crate::app::{App, word_lines};
use crate::game::{CharState, CursorShape, GameState, Mode};
use crate::words::lang_name;

use super::*;

/// Upper bound on word-display lines so very tall terminals stay readable.
const WORD_LINES_MAX: usize = 7;

pub(super) fn draw_test(f: &mut Frame, app: &App) {
    let area = f.area();
    let [header_a, _, words_a, _, stats_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(3), // word display (grows with terminal height)
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area)[..] else {
        return;
    };

    let pad = (area.width / 10).clamp(4, 10);

    // ── header gauge ──
    let gauge_area = horiz_pad(header_a, pad);
    match app.game.mode {
        Mode::Time(total) => {
            let left = app.game.time_left();
            // `.max(1)`: a zero total would make the ratio NaN, and
            // `Gauge::ratio` asserts the value is within 0.0..=1.0.
            let ratio = if app.game.started_at.is_some() {
                left as f64 / total.max(1) as f64
            } else {
                1.0
            };
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(th_accent()).bg(th_gauge_bg()))
                    .ratio(ratio.clamp(0.0, 1.0))
                    .label(if app.game.started_at.is_none() {
                        format!("{total}s")
                    } else {
                        format!("{left}s")
                    }),
                gauge_area,
            );
        }
        Mode::Words(total) => {
            let done = app.game.words_typed();
            let (ratio, label) = if app.game.cursor >= app.game.chars.len() {
                (1.0, total)
            } else {
                ((done as f64 / total.max(1) as f64).clamp(0.0, 1.0), done)
            };
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(th_accent()).bg(th_gauge_bg()))
                    .ratio(ratio)
                    .label(format!("{label} / {total}")),
                gauge_area,
            );
        }
        Mode::Quote => {
            let total = app.game.chars.len();
            let done = app.game.cursor;
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(th_accent()).bg(th_gauge_bg()))
                    .ratio((done as f64 / total.max(1) as f64).clamp(0.0, 1.0))
                    .label(format!("{done} / {total}")),
                gauge_area,
            );
        }
    }

    // ── word display ──
    // The scroll keeps the cursor line at the top row, so the extra lines simply
    // show more upcoming context. Cap the count so tall terminals stay focused,
    // and vertically center the block in the available area.
    let words_area = horiz_pad(words_a, pad);
    let visible = (words_area.height as usize).clamp(1, WORD_LINES_MAX);
    let block_h = visible as u16;
    let words_inner = Rect {
        x: words_area.x,
        y: words_area.y + words_area.height.saturating_sub(block_h) / 2,
        width: words_area.width,
        height: block_h,
    };
    let inner_w = words_inner.width as usize;
    let lines = word_lines(&app.game.words, app.scroll_word, inner_w.max(1));

    let sub = Layout::vertical(vec![Constraint::Length(1); visible]).split(words_inner);
    for (i, word_idxs) in lines.iter().take(visible).enumerate() {
        let is_active = i == 0;
        let cursor_shape = if is_active {
            Some(app.settings.cursor_shape)
        } else {
            None
        };
        let line = build_word_line(&app.game, word_idxs, is_active, cursor_shape);
        f.render_widget(Paragraph::new(line), sub[i]);
    }

    if !app.game.is_finished() {
        let cursor_word = app.game.word_at_cursor();
        for (row, word_idxs) in lines.iter().take(visible).enumerate() {
            if word_idxs.is_empty() {
                continue;
            }
            if word_idxs.last().copied().unwrap_or(0) >= cursor_word
                && word_idxs.first().copied().unwrap_or(usize::MAX) <= cursor_word
            {
                let &row_first = word_idxs.first().unwrap();
                let row_start = app.game.word_starts.get(row_first).copied().unwrap_or(0);
                let col: u16 = app.game.chars[row_start..app.game.cursor.min(app.game.chars.len())]
                    .iter()
                    .map(|c| {
                        unicode_width::UnicodeWidthChar::width(c.typed.unwrap_or(c.expected))
                            .unwrap_or(1) as u16
                    })
                    .sum();
                let col = col.min(words_inner.width.saturating_sub(1));
                f.set_cursor_position((words_inner.x + col, words_inner.y + row as u16));
                break;
            }
        }
    }

    if app.game.started_at.is_none() {
        let hint_area = Rect {
            x: words_inner.x,
            y: words_inner.y + 1,
            width: words_inner.width,
            height: 1,
        };
        f.render_widget(Clear, hint_area);
        f.render_widget(
            Paragraph::new(Span::styled(
                "start typing…",
                Style::default().fg(th_dim()).add_modifier(Modifier::ITALIC),
            ))
            .alignment(Alignment::Center),
            hint_area,
        );
    }

    // ── live stats ──
    let not_started = app.game.started_at.is_none();
    if not_started {
        let mode_label = match app.game.mode {
            Mode::Time(s) => format!("time  {s}s"),
            Mode::Words(n) => format!("words  {n}"),
            Mode::Quote => "quote".to_string(),
        };
        let lang = lang_name(app.settings.lang_idx);
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(mode_label, Style::default().fg(th_dim())),
                Span::styled(format!("  ·  {lang}"), Style::default().fg(th_dim())),
            ]))
            .alignment(Alignment::Center),
            stats_a,
        );
    } else {
        let wpm = app.game.wpm();
        let acc = app.game.accuracy();
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{wpm:.0}"),
                    Style::default()
                        .fg(th_accent())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" wpm", Style::default().fg(th_dim())),
                Span::styled("   ", Style::default()),
                Span::styled(format!("{acc:.1}%"), Style::default().fg(th_fg())),
                Span::styled(" acc", Style::default().fg(th_dim())),
                Span::styled("   ", Style::default()),
                Span::styled(
                    format!("{}", app.game.error_keystrokes),
                    Style::default().fg(th_wrong()),
                ),
                Span::styled(" err", Style::default().fg(th_dim())),
            ]))
            .alignment(Alignment::Center),
            stats_a,
        );
    }

    let footer_a = pin_footer(area, 1);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("tab"),
            Span::raw(" restart"),
            sep(),
            kh("esc"),
            Span::raw(" menu"),
            sep(),
            kh("backspace"),
            Span::raw(" char"),
            sep(),
            kh("ctrl+bksp"),
            Span::raw(" word"),
            sep(),
            kh("ctrl+c"),
            Span::raw(" quit"),
        ]))
        .style(Style::default().fg(th_dim()))
        .alignment(Alignment::Center),
        footer_a,
    );
}

pub(super) fn build_word_line<'a>(
    game: &'a GameState,
    word_idxs: &[usize],
    active: bool,
    cursor_shape: Option<CursorShape>,
) -> Line<'a> {
    let dim_pending = !active;
    let mut spans: Vec<Span<'a>> = vec![];
    for (pos, &wi) in word_idxs.iter().enumerate() {
        if pos > 0 {
            let space_idx = game.word_starts[wi] - 1;
            let sp = &game.chars[space_idx];
            let sp_display = if sp.state == CharState::Wrong {
                sp.typed.unwrap_or(' ').to_string()
            } else {
                " ".to_string()
            };
            spans.push(Span::styled(
                sp_display,
                char_style(sp.state, dim_pending, cursor_shape),
            ));
        }
        let start = game.word_starts[wi];
        let end = if wi + 1 < game.words.len() {
            game.word_starts[wi + 1].saturating_sub(1)
        } else {
            game.chars.len()
        };
        for ci in start..end.min(game.chars.len()) {
            let ch = &game.chars[ci];
            let display: String = match ch.state {
                CharState::Wrong if ch.typed == Some(' ') => "·".to_string(),
                _ => ch.typed.unwrap_or(ch.expected).to_string(),
            };
            spans.push(Span::styled(
                display,
                char_style(ch.state, dim_pending, cursor_shape),
            ));
        }
    }
    Line::from(spans)
}

fn char_style(state: CharState, dim_pending: bool, cursor_shape: Option<CursorShape>) -> Style {
    match state {
        // Bold gives correct chars a non-color cue vs. pending, for low-vision /
        // monochrome terminals where the brightness difference is hard to read.
        CharState::Correct => Style::default()
            .fg(th_correct())
            .add_modifier(Modifier::BOLD),
        CharState::Wrong => {
            let s = Style::default()
                .fg(th_wrong())
                .add_modifier(Modifier::UNDERLINED);
            if dim_pending { s } else { s.bg(th_wrong_bg()) }
        }
        CharState::Current => match cursor_shape {
            Some(CursorShape::Block) | None => Style::default().fg(th_bg()).bg(th_accent()),
            Some(CursorShape::Underline) => Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::UNDERLINED),
            Some(CursorShape::Bar) => Style::default().fg(th_accent()),
        },
        CharState::Pending => {
            let color = if dim_pending { th_dim() } else { th_pending() };
            Style::default().fg(color)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Settings;

    fn game() -> GameState {
        GameState::new(Mode::Words(3), Settings::default(), vec![])
    }

    #[test]
    fn correct_char_renders_with_correct_color_and_bold() {
        let mut g = game();
        let ch = g.chars[0].expected;
        g.type_char(ch);
        let line = build_word_line(&g, &[0], true, None);
        let span = &line.spans[0];
        assert_eq!(span.content, ch.to_string());
        assert_eq!(span.style.fg, Some(th_correct()));
        assert!(span.style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn wrong_char_renders_with_wrong_color_and_underline() {
        let mut g = game();
        let expected = g.chars[0].expected;
        let wrong = if expected == 'a' { 'z' } else { 'a' };
        g.type_char(wrong);
        let line = build_word_line(&g, &[0], true, None);
        let span = &line.spans[0];
        assert_eq!(span.content, wrong.to_string());
        assert_eq!(span.style.fg, Some(th_wrong()));
        assert!(span.style.add_modifier.contains(Modifier::UNDERLINED));
    }

    /// A zero-length test (only reachable via a hand-edited `config.json`)
    /// must not make the gauge ratio NaN — `Gauge::ratio` asserts on it.
    #[test]
    fn zero_length_modes_do_not_panic_the_gauge() {
        use crate::app::{App, Screen};
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        for mode in [Mode::Time(0), Mode::Words(0)] {
            let mut app = App::new();
            app.screen = Screen::Test;
            app.game = GameState::new(mode, Settings::default(), vec![]);
            app.game.started_at = Some(std::time::Instant::now());
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|f| crate::ui::draw(f, &app)).unwrap();
        }
    }

    /// A letter mistyped as a space renders as a middle dot rather than a
    /// blank, so the error stays visible instead of looking like an unfilled
    /// pending char.
    #[test]
    fn letter_mistyped_as_space_renders_as_middle_dot() {
        let mut g = game();
        assert!(g.chars[0].expected != ' ');
        g.type_char(' ');
        let line = build_word_line(&g, &[0], true, None);
        assert_eq!(line.spans[0].content, "\u{b7}");
    }
}
