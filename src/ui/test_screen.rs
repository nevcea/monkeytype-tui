//! Renders the Test screen: progress gauge, wrapped word lines with
//! per-character coloring, and the live WPM/accuracy readout.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
};

use crate::app::{App, word_lines};
use crate::game::{CharState, CursorShape, GameState, Mode};
use crate::words::lang_name;

use super::*;

/// Upper bound on word-display lines so very tall terminals stay readable.
const WORD_LINES_MAX: usize = 7;

pub(super) fn draw_test(f: &mut Frame, app: &App) {
    let area = f.area();
    let [header_a, _, words_a, hint_a, stats_a, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(3),    // word display (grows with terminal height)
        Constraint::Length(1), // pre-start hint
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area)[..] else {
        return;
    };

    let pad = (area.width / 10).clamp(4, 10);

    draw_progress_gauge(f, horiz_pad(header_a, pad), &app.game);

    let block = WordBlock::new(horiz_pad(words_a, pad), app);
    draw_word_lines(f, &block, app);
    if !app.game.is_finished() {
        place_cursor(f, &block, &app.game);
    }
    if app.game.started_at.is_none() {
        draw_start_hint(f, hint_a);
    }

    draw_live_stats(f, stats_a, app);
    draw_footer(f, pin_footer(area, 1));
}

/// Header progress bar. All three modes render the same gauge and differ only
/// in the filled ratio and its label, so the mode match produces just those.
fn draw_progress_gauge(f: &mut Frame, area: Rect, game: &GameState) {
    let (ratio, label) = match game.mode {
        Mode::Time(total) => {
            // `.max(1)`: a zero total would make the ratio NaN, and
            // `Gauge::ratio` asserts the value is within 0.0..=1.0.
            if game.started_at.is_some() {
                let left = game.time_left();
                (left as f64 / total.max(1) as f64, format!("{left}s"))
            } else {
                (1.0, format!("{total}s"))
            }
        }
        Mode::Words(total) => {
            if game.cursor >= game.chars.len() {
                (1.0, format!("{total} / {total}"))
            } else {
                let done = game.words_typed();
                (
                    done as f64 / total.max(1) as f64,
                    format!("{done} / {total}"),
                )
            }
        }
        Mode::Quote => {
            let total = game.chars.len();
            let done = game.cursor;
            (
                done as f64 / total.max(1) as f64,
                format!("{done} / {total}"),
            )
        }
    };

    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(th_accent()).bg(th_gauge_bg()))
            .ratio(ratio.clamp(0.0, 1.0))
            .label(label),
        area,
    );
}

/// Geometry of the word display, shared by the three passes that draw into it
/// (the lines themselves, the terminal cursor, and the pre-start hint).
struct WordBlock {
    /// The rect the lines actually occupy, centred in the available area.
    inner: Rect,
    /// How many rows fit, capped by [`WORD_LINES_MAX`].
    visible: usize,
    /// Wrapped word-index groups, one per line, from the current scroll point.
    lines: Vec<Vec<usize>>,
}

impl WordBlock {
    /// The scroll keeps the cursor line at the top row, so the extra lines
    /// simply show more upcoming context. Cap the count so tall terminals stay
    /// focused, and vertically center the block in the available area.
    fn new(area: Rect, app: &App) -> Self {
        let visible = (area.height as usize).clamp(1, WORD_LINES_MAX);
        let block_h = visible as u16;
        let inner = Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(block_h) / 2,
            width: area.width,
            height: block_h,
        };
        let lines = word_lines(
            &app.game.words,
            app.scroll_word,
            (inner.width as usize).max(1),
        );
        Self {
            inner,
            visible,
            lines,
        }
    }
}

fn draw_word_lines(f: &mut Frame, block: &WordBlock, app: &App) {
    let sub = Layout::vertical(vec![Constraint::Length(1); block.visible]).split(block.inner);
    for (i, word_idxs) in block.lines.iter().take(block.visible).enumerate() {
        // Only the top line is being typed; the rest render as dimmed context.
        let is_active = i == 0;
        let cursor_shape = is_active.then_some(app.settings.cursor_shape);
        let line = build_word_line(&app.game, word_idxs, is_active, cursor_shape);
        f.render_widget(Paragraph::new(line), sub[i]);
    }
}

/// Park the terminal's own cursor on the character about to be typed, so the
/// shape chosen in settings appears where the user is looking.
fn place_cursor(f: &mut Frame, block: &WordBlock, game: &GameState) {
    let cursor_word = game.word_at_cursor();
    for (row, word_idxs) in block.lines.iter().take(block.visible).enumerate() {
        let (Some(&first), Some(&last)) = (word_idxs.first(), word_idxs.last()) else {
            continue;
        };
        if !(first..=last).contains(&cursor_word) {
            continue;
        }
        let row_start = game.word_starts.get(first).copied().unwrap_or(0);
        // Sum display widths, not char counts, so wide (CJK) glyphs land right.
        let col: u16 = game.chars[row_start..game.cursor.min(game.chars.len())]
            .iter()
            .map(|c| {
                unicode_width::UnicodeWidthChar::width(c.typed.unwrap_or(c.expected)).unwrap_or(1)
                    as u16
            })
            .sum();
        let col = col.min(block.inner.width.saturating_sub(1));
        f.set_cursor_position((block.inner.x + col, block.inner.y + row as u16));
        break;
    }
}

/// Rendered in the gap below the word block. It used to `Clear` the row
/// beneath the first word line and draw over it, which meant the second line
/// of upcoming words was missing until the first keystroke put it back.
fn draw_start_hint(f: &mut Frame, area: Rect) {
    f.render_widget(
        Paragraph::new(Span::styled(
            "start typing…",
            Style::default().fg(th_dim()).add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center),
        area,
    );
}

/// The mode/language summary before the first keystroke, live wpm/accuracy/
/// error counts once typing has started.
fn draw_live_stats(f: &mut Frame, area: Rect, app: &App) {
    let line = if app.game.started_at.is_none() {
        let mode_label = match app.game.mode {
            Mode::Time(s) => format!("time  {s}s"),
            Mode::Words(n) => format!("words  {n}"),
            Mode::Quote => "quote".to_string(),
        };
        let lang = lang_name(app.settings.lang_idx);
        Line::from(vec![
            Span::styled(mode_label, Style::default().fg(th_dim())),
            Span::styled(format!("  ·  {lang}"), Style::default().fg(th_dim())),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                format!("{:.0}", app.game.wpm()),
                Style::default()
                    .fg(th_accent())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" wpm", Style::default().fg(th_dim())),
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("{:.1}%", app.game.accuracy()),
                Style::default().fg(th_fg()),
            ),
            Span::styled(" acc", Style::default().fg(th_dim())),
            Span::styled("   ", Style::default()),
            Span::styled(
                format!("{}", app.game.error_keystrokes),
                Style::default().fg(th_wrong()),
            ),
            Span::styled(" err", Style::default().fg(th_dim())),
        ])
    };
    f.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
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
        area,
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

    /// All three modes share one `draw_progress_gauge` now, so pin the label
    /// each of them produces — that label is the only user-visible difference
    /// left between them, and the mode arms no longer carry their own gauge.
    #[test]
    fn progress_gauge_label_per_mode() {
        use crate::app::{App, Screen};
        use ratatui::Terminal;
        use ratatui::backend::TestBackend;

        fn header(app: &App) -> String {
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
            terminal.draw(|f| crate::ui::draw(f, app)).unwrap();
            let buffer = terminal.backend().buffer();
            (0..80).map(|x| buffer[(x, 0)].symbol()).collect()
        }

        // Fresh test: time shows the full duration, counters start at zero.
        for (mode, expected) in [
            (Mode::Time(30), "30s"),
            (Mode::Words(20), "0 / 20"),
            (Mode::Quote, "0 / "),
        ] {
            let mut app = App::new();
            app.screen = Screen::Test;
            app.game = GameState::new(mode, Settings::default(), vec![]);
            let row = header(&app);
            assert!(
                row.contains(expected),
                "{mode} gauge should show {expected:?}, got {row:?}"
            );
        }

        // A completed word-mode test reports the full count, not the space-
        // counted total (which is one short once the last word has no trailing
        // space to type).
        let mut app = App::new();
        app.screen = Screen::Test;
        app.game = GameState::new(Mode::Words(5), Settings::default(), vec![]);
        while app.game.cursor < app.game.chars.len() {
            let ch = app.game.chars[app.game.cursor].expected;
            app.game.type_char(ch);
        }
        let row = header(&app);
        assert!(row.contains("5 / 5"), "finished gauge: {row:?}");
    }

    /// The hint used to `Clear` the row beneath the first word line and draw
    /// itself there, so the second line of upcoming words was blank until the
    /// first keystroke restored it — the block visibly jumped as you started.
    /// Typing must not change any row of the word block.
    #[test]
    fn the_start_hint_does_not_blank_a_line_of_words() {
        use crate::app::{App, Screen};
        use crate::ui::test_render::rows;

        let mut app = App::new();
        app.screen = Screen::Test;
        app.game = GameState::new(Mode::Words(50), Settings::default(), vec![]);

        // At 80x24 the layout is: gauge 0, gap 1, words 2..=20, hint 21,
        // stats 22, footer 23.
        const WORDS: std::ops::RangeInclusive<usize> = 2..=20;
        const HINT: usize = 21;

        let before = rows(80, 24, |f| crate::ui::draw(f, &app));
        assert!(
            before[HINT].contains("start typing"),
            "the hint belongs in the gap below the word block, not inside it:\n{before:#?}"
        );

        let ch = app.game.chars[0].expected;
        app.game.type_char(ch);
        let after = rows(80, 24, |f| crate::ui::draw(f, &app));

        for i in WORDS {
            assert_eq!(
                before[i].trim_end(),
                after[i].trim_end(),
                "word row {i} changed when typing started:\n{before:#?}\n{after:#?}"
            );
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
