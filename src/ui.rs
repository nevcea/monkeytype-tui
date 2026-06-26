use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};

use crate::app::{word_lines, App, MenuMode, Screen, TIME_OPTIONS, WORD_OPTIONS};
use crate::game::{CharState, Mode};
use crate::words::LANGUAGES;

// ── palette ───────────────────────────────────────────────────────────────────
const BG:        Color = Color::Rgb(28,  28,  30);
const C_CORRECT: Color = Color::Rgb(210, 200, 170);
const C_WRONG:   Color = Color::Rgb(202,  71,  71);
const C_PENDING: Color = Color::Rgb( 88,  88,  93);
const C_ACCENT:  Color = Color::Rgb(226, 183,  20);
const C_DIM:     Color = Color::Rgb( 72,  72,  77);
const C_FG:      Color = Color::Rgb(200, 200, 205);
const C_SUB:     Color = Color::Rgb(140, 140, 145);

// ── entry ─────────────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    let bg = Block::default().style(Style::default().bg(BG));
    f.render_widget(bg, f.area());

    match app.screen {
        Screen::Menu    => draw_menu(f, app),
        Screen::Test    => draw_test(f, app),
        Screen::Result  => draw_result(f, app),
        Screen::History => draw_history(f, app),
        Screen::Help    => draw_help(f),
    }

    // Lang picker drawn on top of whatever is below
    if app.lang_picker.is_some() {
        draw_lang_picker(f, app);
    }
}

// ── Menu ──────────────────────────────────────────────────────────────────────

fn draw_menu(f: &mut Frame, app: &App) {
    let area = centered_rect(56, 30, f.area());
    let [title_a, gap1, tabs_a, opts_a, gap2, lang_a, size_a, gap3, toggles_a, gap4, footer_a] =
        Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Length(1), // gap
            Constraint::Length(1), // mode tabs  1·time  2·words  3·quote
            Constraint::Length(1), // option row
            Constraint::Length(1), // gap
            Constraint::Length(1), // language
            Constraint::Length(1), // size options
            Constraint::Length(1), // gap
            Constraint::Length(1), // toggles
            Constraint::Min(0),    // spacer
            Constraint::Length(1), // footer hints
        ])
        .split(area)[..] else { return };

    // Title
    f.render_widget(
        Paragraph::new(Span::styled("Monkeytype",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center),
        title_a,
    );
    let _ = gap1; let _ = gap2; let _ = gap3; let _ = gap4;

    // Mode tabs — show number shortcut dimly
    f.render_widget(
        Paragraph::new(Line::from(vec![
            mode_tab_n("1", "time",  app.menu_mode == MenuMode::Time),
            Span::styled("    ", Style::default()),
            mode_tab_n("2", "words", app.menu_mode == MenuMode::Words),
            Span::styled("    ", Style::default()),
            mode_tab_n("3", "quote", app.menu_mode == MenuMode::Quote),
        ])).alignment(Alignment::Center),
        tabs_a,
    );

    // Options row
    let opts_line = match app.menu_mode {
        MenuMode::Time => {
            let mut spans = option_spans(TIME_OPTIONS, app.menu_time_idx, "s");
            spans.push(Span::styled("  ", Style::default()));
            spans.push(custom_slot(
                app.menu_time_idx == TIME_OPTIONS.len(),
                "s", &app.custom_input,
            ));
            Line::from(spans)
        }
        MenuMode::Words => {
            let mut spans = option_spans(WORD_OPTIONS, app.menu_word_idx, "");
            spans.push(Span::styled("  ", Style::default()));
            spans.push(custom_slot(
                app.menu_word_idx == WORD_OPTIONS.len(),
                "", &app.custom_input,
            ));
            Line::from(spans)
        }
        MenuMode::Quote => Line::from(Span::styled(
            "random quote from 6000+ collection",
            Style::default().fg(C_PENDING),
        )),
    };

    // Hint when custom input is active
    let opts_display = if app.custom_input.is_some() {
        Paragraph::new(vec![
            opts_line,
            Line::from(Span::styled("type a number and press enter",
                Style::default().fg(C_DIM))),
        ]).alignment(Alignment::Center)
    } else {
        Paragraph::new(opts_line).alignment(Alignment::Center)
    };
    f.render_widget(opts_display, opts_a);

    // Language row
    let lang = LANGUAGES.get(app.settings.lang_idx).unwrap_or(&LANGUAGES[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("language  ", Style::default().fg(C_DIM)),
            Span::styled(lang.name, Style::default().fg(C_FG)
                .add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {}/{}", app.settings.lang_idx + 1, LANGUAGES.len()),
                Style::default().fg(C_DIM)),
            Span::styled("   l·change", Style::default().fg(C_DIM)),
        ])).alignment(Alignment::Center),
        lang_a,
    );

    // Size row
    let size_spans: Vec<Span> = lang.sizes.iter().enumerate()
        .flat_map(|(i, sz)| {
            let span = if i == app.settings.size_idx {
                Span::styled(sz.label, Style::default().fg(C_ACCENT)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
            } else {
                Span::styled(sz.label, Style::default().fg(C_PENDING))
            };
            if i + 1 < lang.sizes.len() { vec![span, Span::styled("  ", Style::default())] }
            else { vec![span] }
        })
        .collect();
    f.render_widget(
        Paragraph::new(Line::from(size_spans)).alignment(Alignment::Center),
        size_a,
    );

    // Toggles
    f.render_widget(
        Paragraph::new(Line::from(vec![
            toggle_span("punctuation", app.settings.punctuation),
            Span::styled("     ", Style::default()),
            toggle_span("numbers", app.settings.numbers),
            Span::styled("     ", Style::default()),
            Span::styled("p·punct  n·numbers", Style::default().fg(C_DIM)),
        ])).alignment(Alignment::Center),
        toggles_a,
    );

    // Footer hints
    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("enter"), Span::raw(" start"),
            sep(),
            kh("1/2/3"), Span::raw(" mode"),
            sep(),
            kh("←/→"), Span::raw(" option"),
            sep(),
            kh("h"), Span::raw(" history"),
            sep(),
            kh("?"), Span::raw(" help"),
            sep(),
            kh("q"), Span::raw(" quit"),
        ])).alignment(Alignment::Center),
        footer_a,
    );
}

// ── Test ──────────────────────────────────────────────────────────────────────

fn draw_test(f: &mut Frame, app: &App) {
    let area = f.area();
    let [header_a, _, words_a, _, stats_a, footer_a] = Layout::vertical([
        Constraint::Length(1), // gauge / counter
        Constraint::Length(1), // gap
        Constraint::Length(3), // 3-line word display
        Constraint::Length(1), // gap
        Constraint::Length(1), // live wpm/acc
        Constraint::Length(1), // key hints
    ]).split(area)[..] else { return };

    let pad = 6u16;

    // ── header gauge ──
    let gauge_area = horiz_pad(header_a, pad);
    match app.game.mode {
        Mode::Time(total) => {
            let left = app.game.time_left();
            let ratio = if app.game.started_at.is_some() {
                left as f64 / total as f64
            } else { 1.0 };
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(C_ACCENT).bg(Color::Rgb(48, 48, 52)))
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
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(C_ACCENT).bg(Color::Rgb(48, 48, 52)))
                    .ratio((done as f64 / total as f64).clamp(0.0, 1.0))
                    .label(format!("{done} / {total}")),
                gauge_area,
            );
        }
        Mode::Quote => {
            let total = app.game.chars.len();
            let done = app.game.cursor;
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(C_ACCENT).bg(Color::Rgb(48, 48, 52)))
                    .ratio((done as f64 / total.max(1) as f64).clamp(0.0, 1.0))
                    .label(format!("{done} / {total}")),
                gauge_area,
            );
        }
    }

    // ── word display ──
    let words_inner = horiz_pad(words_a, pad);
    let inner_w = words_inner.width as usize;
    let lines = word_lines(&app.game.words, app.scroll_word, inner_w.max(1));

    let sub = Layout::vertical([Constraint::Length(1); 3]).split(words_inner);
    for (i, word_idxs) in lines.iter().take(3).enumerate() {
        // Dim lines that are not the active line (line 0 = current)
        let is_active = i == 0;
        let line = build_word_line(&app.game, word_idxs, is_active);
        f.render_widget(Paragraph::new(line), sub[i]);
    }

    // "start typing" overlay before first keystroke
    if app.game.started_at.is_none() {
        let hint_area = Rect {
            x: words_inner.x,
            y: words_inner.y + 1, // centre row of the 3 lines
            width: words_inner.width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new(Span::styled(
                "start typing…",
                Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC),
            )).alignment(Alignment::Center),
            hint_area,
        );
    }

    // ── live stats ──
    let not_started = app.game.started_at.is_none();
    if not_started {
        // Show mode label instead of zeroed stats
        let mode_label = match app.game.mode {
            Mode::Time(s)  => format!("time  {s}s"),
            Mode::Words(n) => format!("words  {n}"),
            Mode::Quote    => "quote".to_string(),
        };
        let lang = LANGUAGES.get(app.settings.lang_idx).map(|l| l.name).unwrap_or("english");
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(mode_label, Style::default().fg(C_DIM)),
                Span::styled(format!("  ·  {lang}"), Style::default().fg(C_DIM)),
            ])).alignment(Alignment::Center),
            stats_a,
        );
    } else {
        let wpm = app.game.wpm();
        let acc = app.game.accuracy();
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{wpm:.0}"),
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)),
                Span::styled(" wpm", Style::default().fg(C_DIM)),
                Span::styled("   ", Style::default()),
                Span::styled(format!("{acc:.1}%"), Style::default().fg(C_FG)),
                Span::styled(" acc", Style::default().fg(C_DIM)),
            ])).alignment(Alignment::Center),
            stats_a,
        );
    }

    // ── footer hints ──
    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("tab"), Span::raw(" restart"),
            sep(),
            kh("esc"), Span::raw(" menu"),
            sep(),
            kh("backspace"), Span::raw(" delete"),
        ])).style(Style::default().fg(C_DIM))
          .alignment(Alignment::Center),
        footer_a,
    );
}

fn build_word_line<'a>(
    game: &'a crate::game::GameState,
    word_idxs: &[usize],
    active: bool,
) -> Line<'a> {
    let dim_pending = !active;
    let mut spans: Vec<Span<'a>> = vec![];
    for (pos, &wi) in word_idxs.iter().enumerate() {
        if pos > 0 {
            let space_idx = game.word_starts[wi] - 1;
            let sp = &game.chars[space_idx];
            spans.push(Span::styled(" ", char_style(sp.state, dim_pending)));
        }
        let start = game.word_starts[wi];
        let end = if wi + 1 < game.words.len() {
            game.word_starts[wi + 1].saturating_sub(1)
        } else {
            game.chars.len()
        };
        for ci in start..end.min(game.chars.len()) {
            let ch = &game.chars[ci];
            let display: String = ch.typed.unwrap_or(ch.expected).to_string();
            spans.push(Span::styled(display, char_style(ch.state, dim_pending)));
        }
    }
    Line::from(spans)
}

fn char_style(state: CharState, dim_pending: bool) -> Style {
    match state {
        CharState::Correct => Style::default().fg(C_CORRECT),
        CharState::Wrong   => Style::default().fg(C_WRONG).add_modifier(Modifier::UNDERLINED),
        CharState::Current => Style::default().fg(BG).bg(C_ACCENT),
        CharState::Pending => {
            let color = if dim_pending { C_DIM } else { C_PENDING };
            Style::default().fg(color)
        }
    }
}

// ── Result ────────────────────────────────────────────────────────────────────

fn draw_result(f: &mut Frame, app: &App) {
    let area = centered_rect(48, 72, f.area());
    let [wpm_a, acc_a, _, stats_a, _, spark_a, _, src_a, _, footer_a] =
        Layout::vertical([
            Constraint::Length(2), // wpm big
            Constraint::Length(1), // accuracy
            Constraint::Length(1), // gap
            Constraint::Length(2), // stats grid
            Constraint::Length(1), // gap
            Constraint::Length(2), // sparkline
            Constraint::Min(1),    // spacer
            Constraint::Length(1), // quote source
            Constraint::Length(1), // gap
            Constraint::Length(1), // footer
        ]).split(area)[..] else { return };

    let wpm     = app.game.wpm();
    let raw     = app.game.raw_wpm();
    let acc     = app.game.accuracy();
    let elapsed = app.game.elapsed().as_secs_f64();
    let correct = app.game.chars.iter().filter(|c| c.state == CharState::Correct).count();
    let wrong   = app.game.chars.iter().filter(|c| c.state == CharState::Wrong).count();

    // Big WPM
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(format!("{wpm:.0}"),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("wpm", Style::default().fg(C_DIM))),
        ]).alignment(Alignment::Center),
        wpm_a,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{acc:.1}%"),
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD)),
            Span::styled(" accuracy", Style::default().fg(C_DIM)),
        ])).alignment(Alignment::Center),
        acc_a,
    );

    f.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                sv("time "), sf(format!("{elapsed:.1}s")),
                Span::styled("   ", Style::default()),
                sv("raw "),  sf(format!("{raw:.0} wpm")),
            ]),
            Line::from(vec![
                sv("correct "), sf(correct.to_string()),
                Span::styled("   ", Style::default()),
                sv("wrong "),   sf(wrong.to_string()),
            ]),
        ]).alignment(Alignment::Center),
        stats_a,
    );

    let spark = sparkline(&app.game.wpm_samples);
    if !spark.is_empty() {
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("wpm over time", Style::default().fg(C_DIM))),
                Line::from(Span::styled(spark, Style::default().fg(C_ACCENT))),
            ]).alignment(Alignment::Center),
            spark_a,
        );
    }

    if let Some(src) = &app.game.quote_source {
        f.render_widget(
            Paragraph::new(Span::styled(format!("— {src}"),
                Style::default().fg(C_PENDING).add_modifier(Modifier::ITALIC)))
                .alignment(Alignment::Center),
            src_a,
        );
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            kh("tab"), Span::raw(" restart"),
            sep(),
            kh("esc"), Span::raw(" menu"),
        ])).style(Style::default().fg(C_DIM))
          .alignment(Alignment::Center),
        footer_a,
    );
}

// ── History ───────────────────────────────────────────────────────────────────

fn draw_history(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 80, f.area());
    let [title_a, _, header_a, entries_a, footer_a] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).split(area)[..] else { return };

    f.render_widget(
        Paragraph::new(Span::styled("history",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center),
        title_a,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![
            col("wpm",  8, C_DIM), col("acc", 8, C_DIM),
            col("mode", 14, C_DIM), col("when", 12, C_DIM),
        ])),
        header_a,
    );

    let max_rows = entries_a.height as usize;
    let lines: Vec<Line> = if app.history.is_empty() {
        vec![Line::from(Span::styled("no results yet", Style::default().fg(C_DIM)))]
    } else {
        app.history.iter().take(max_rows).map(|e| {
            Line::from(vec![
                col(format!("{:.0}", e.wpm), 8, C_ACCENT),
                col(format!("{:.1}%", e.accuracy), 8, C_FG),
                col(&e.mode, 14, C_FG),
                col(e.time_ago(), 12, C_PENDING),
            ])
        }).collect()
    };
    f.render_widget(Paragraph::new(lines), entries_a);

    f.render_widget(
        Paragraph::new(Line::from(vec![kh("esc"), Span::raw(" back")]))
            .style(Style::default().fg(C_DIM))
            .alignment(Alignment::Center),
        footer_a,
    );
}

// ── Help ──────────────────────────────────────────────────────────────────────

fn draw_help(f: &mut Frame) {
    let area = centered_rect(54, 90, f.area());
    let [title_a, _, body_a, footer_a] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ]).split(area)[..] else { return };

    f.render_widget(
        Paragraph::new(Span::styled("help",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center),
        title_a,
    );

    let kw  = |k: &'static str| Span::styled(format!("{k:<16}"), Style::default().fg(C_ACCENT));
    let dsc = |d: &'static str| Span::styled(d, Style::default().fg(C_FG));
    let sec = |s: &'static str| Line::from(
        Span::styled(s, Style::default().fg(C_DIM).add_modifier(Modifier::BOLD)));
    let row = |k, d| Line::from(vec![kw(k), dsc(d)]);

    f.render_widget(Paragraph::new(vec![
        sec("── menu ──────────────────────────────────────"),
        row("1 / 2 / 3",    "select mode  (time · words · quote)"),
        row("← / →",        "change option value"),
        row("enter",         "start test  (or open custom input)"),
        row("l",             "open language picker"),
        row("p",             "toggle punctuation"),
        row("n",             "toggle numbers"),
        row("h",             "history"),
        row("? ",            "this help"),
        row("q  /  ctrl+c",  "quit"),
        Line::from(""),
        sec("── test ────────────────────────────────────────"),
        row("tab",           "restart test"),
        row("esc",           "back to menu"),
        row("backspace",     "delete last character"),
        Line::from(""),
        sec("── result ──────────────────────────────────────"),
        row("tab",           "restart same test"),
        row("esc",           "back to menu"),
        Line::from(""),
        sec("── language picker ─────────────────────────────"),
        row("↑ / ↓",         "navigate languages"),
        row("← / →",         "change word pool size"),
        row("enter",         "confirm selection"),
        row("esc",           "cancel"),
    ]), body_a);

    f.render_widget(
        Paragraph::new(Line::from(vec![kh("esc"), Span::raw(" back")]))
            .style(Style::default().fg(C_DIM))
            .alignment(Alignment::Center),
        footer_a,
    );
}

// ── Lang Picker ───────────────────────────────────────────────────────────────

fn draw_lang_picker(f: &mut Frame, app: &App) {
    const VISIBLE: usize = 12;
    let picker = match &app.lang_picker { Some(p) => p, None => return };

    let filtered: Vec<(usize, &_)> = LANGUAGES.iter().enumerate()
        .filter(|(_, l)| l.name.to_lowercase().contains(&picker.search.to_lowercase()))
        .collect();

    let area = centered_rect(54, 75, f.area());

    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_ACCENT))
            .title(Span::styled(" language ", Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD)))
            .style(Style::default().bg(Color::Rgb(36, 36, 40))),
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
    ]).split(inner)[..] else { return };

    // Search bar
    let search_display = format!("▶ {}_", picker.search);
    f.render_widget(
        Paragraph::new(Span::styled(search_display, Style::default().fg(C_FG))),
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
            kh("↑/↓"), Span::raw(" navigate"),
            sep(),
            kh("←/→"), Span::raw(" size"),
            sep(),
            kh("enter"), Span::raw(" select"),
            sep(),
            kh("esc"), Span::raw(" cancel"),
            Span::styled(scroll_info, Style::default().fg(C_DIM)),
        ])).style(Style::default().fg(C_DIM))
          .alignment(Alignment::Center),
        footer_a,
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - pct_y) / 2),
        Constraint::Percentage(pct_y),
        Constraint::Percentage((100 - pct_y) / 2),
    ]).split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ]).split(v[1])[1]
}

fn horiz_pad(r: Rect, pad: u16) -> Rect {
    Rect { x: r.x + pad, y: r.y, width: r.width.saturating_sub(pad * 2), height: r.height }
}

fn mode_tab_n(num: &'static str, label: &'static str, active: bool) -> Span<'static> {
    let text = format!("{num}·{label}");
    if active {
        Span::styled(text, Style::default().fg(C_ACCENT)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
    } else {
        Span::styled(text, Style::default().fg(C_PENDING))
    }
}

fn toggle_span(label: &'static str, on: bool) -> Span<'static> {
    if on {
        Span::styled(format!("[{label}]"),
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(format!(" {label} "), Style::default().fg(C_DIM))
    }
}

fn option_spans<T: std::fmt::Display>(opts: &[T], selected: usize, suffix: &str) -> Vec<Span<'static>> {
    opts.iter().enumerate().flat_map(|(i, v)| {
        let label = format!("{v}{suffix}");
        let span = if i == selected {
            Span::styled(label, Style::default().fg(C_ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        } else {
            Span::styled(label, Style::default().fg(C_PENDING))
        };
        vec![span, Span::styled("  ", Style::default())]
    }).collect()
}

fn custom_slot<'a>(selected: bool, suffix: &str, input: &Option<String>) -> Span<'a> {
    if selected {
        let text = if let Some(s) = input {
            format!("custom: {s}▌{suffix}")
        } else {
            "custom".to_string()
        };
        Span::styled(text, Style::default().fg(C_ACCENT)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
    } else {
        Span::styled("custom", Style::default().fg(C_PENDING))
    }
}

fn sep<'a>() -> Span<'a> { Span::styled("   ", Style::default()) }

// key highlight span
fn kh(key: &str) -> Span<'static> {
    Span::styled(key.to_string(), Style::default().fg(C_SUB))
}

fn sv(s: &'static str) -> Span<'static> { Span::styled(s, Style::default().fg(C_DIM)) }
fn sf(s: String) -> Span<'static>       { Span::styled(s, Style::default().fg(C_FG)) }

fn col<S: Into<String>>(s: S, w: usize, color: Color) -> Span<'static> {
    Span::styled(format!("{:<w$}", s.into()), Style::default().fg(color))
}

fn sparkline(samples: &[f64]) -> String {
    if samples.is_empty() { return String::new(); }
    let max = samples.iter().cloned().fold(0.0f64, f64::max).max(1.0);
    let blocks: Vec<char> = "▁▂▃▄▅▆▇█".chars().collect();
    samples.iter()
        .map(|&v| blocks[(((v / max) * 7.0).round() as usize).min(7)])
        .collect()
}
