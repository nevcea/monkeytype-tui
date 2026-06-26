use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, Gauge, GraphType, Paragraph},
};

use crate::app::{
    App, LANG_PICKER_VISIBLE, Screen, TIME_OPTIONS, WORD_OPTIONS, filtered_languages, word_lines,
};
use crate::game::{CharState, CursorShape, Mode};
use crate::sound::SoundPack;
use crate::words::LANGUAGES;

// ── palette ───────────────────────────────────────────────────────────────────
const BG: Color = Color::Rgb(28, 28, 30);
const C_CORRECT: Color = Color::Rgb(210, 200, 170);
const C_WRONG: Color = Color::Rgb(202, 71, 71);
const C_PENDING: Color = Color::Rgb(88, 88, 93);
const C_ACCENT: Color = Color::Rgb(226, 183, 20);
const C_DIM: Color = Color::Rgb(72, 72, 77);
const C_WRONG_BG: Color = Color::Rgb(60, 15, 15);
const C_GAUGE_BG: Color = Color::Rgb(48, 48, 52);
const C_FG: Color = Color::Rgb(200, 200, 205);
const C_SUB: Color = Color::Rgb(140, 140, 145);

// ── entry ─────────────────────────────────────────────────────────────────────

pub fn draw(f: &mut Frame, app: &App) {
    let bg = Block::default().style(Style::default().bg(BG));
    f.render_widget(bg, f.area());

    let area = f.area();
    if area.width < 60 || area.height < 20 {
        f.render_widget(
            Paragraph::new(Span::styled(
                "terminal too small  (min 60×20)",
                Style::default().fg(C_DIM),
            ))
            .alignment(Alignment::Center),
            Rect {
                x: area.x,
                y: area.height / 2,
                width: area.width,
                height: 1,
            },
        );
        return;
    }

    match app.screen {
        Screen::Menu => draw_menu(f, app),
        Screen::Test => draw_test(f, app),
        Screen::Result => draw_result(f, app),
        Screen::History => draw_history(f, app),
        Screen::Help => draw_help(f),
        Screen::Settings => draw_settings(f, app),
    }

    // Lang picker drawn on top of whatever is below
    if app.lang_picker.is_some() {
        draw_lang_picker(f, app);
    }
    if app.quit_confirm {
        draw_confirm(f, "quit?", app.quit_yes);
    }
    if app.test_confirm {
        draw_confirm(f, "abandon test?", app.test_confirm_yes);
    }
}

fn draw_confirm(f: &mut Frame, title: &str, is_yes: bool) {
    let area = centered_rect(40, 0, f.area());
    let area = Rect {
        x: area.x,
        y: f.area().height.saturating_sub(5) / 2,
        width: area.width,
        height: 5,
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
        height: 3,
    };
    let sel = Style::default()
        .fg(C_ACCENT)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);
    let dim = Style::default().fg(C_PENDING);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                title,
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from(vec![
                Span::styled("yes", if is_yes { sel } else { dim }),
                Span::raw("   "),
                Span::styled("no", if !is_yes { sel } else { dim }),
            ]),
        ])
        .alignment(Alignment::Center),
        inner,
    );
}

// ── Menu ──────────────────────────────────────────────────────────────────────

fn draw_menu(f: &mut Frame, app: &App) {
    let area = centered_rect(56, 30, f.area());
    let [
        title_a,
        gap1,
        tabs_a,
        opts_a,
        gap2,
        lang_a,
        size_a,
        gap3,
        toggles_a,
        gap4,
        _,
    ] = Layout::vertical([
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
        Constraint::Length(2), // footer hints
    ])
    .split(area)[..]
    else {
        return;
    };

    // Title
    f.render_widget(
        Paragraph::new(Span::styled(
            "monkeytype",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );
    let _ = gap1;
    let _ = gap2;
    let _ = gap3;
    let _ = gap4;

    // Mode tabs — active tab shows current value
    let time_label = match app.menu_mode {
        Mode::Time(n) => format!("time·{n}s"),
        _ => "time".to_owned(),
    };
    let words_label = match app.menu_mode {
        Mode::Words(n) => format!("words·{n}"),
        _ => "words".to_owned(),
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            mode_tab_n("1", time_label, matches!(app.menu_mode, Mode::Time(_))),
            Span::raw("    "),
            mode_tab_n("2", words_label, matches!(app.menu_mode, Mode::Words(_))),
            Span::raw("    "),
            mode_tab_n("3", "quote", matches!(app.menu_mode, Mode::Quote)),
        ]))
        .alignment(Alignment::Center),
        tabs_a,
    );

    // Options row
    let opts_line = match app.menu_mode {
        Mode::Time(_) => {
            let mut spans = option_spans(TIME_OPTIONS, app.menu_time_idx, "s");
            spans.push(Span::raw("  "));
            spans.push(custom_slot(
                app.menu_time_idx == TIME_OPTIONS.len(),
                "s",
                &app.custom_input,
            ));
            Line::from(spans)
        }
        Mode::Words(_) => {
            let mut spans = option_spans(WORD_OPTIONS, app.menu_word_idx, "");
            spans.push(Span::raw("  "));
            spans.push(custom_slot(
                app.menu_word_idx == WORD_OPTIONS.len(),
                "",
                &app.custom_input,
            ));
            Line::from(spans)
        }
        Mode::Quote => {
            if LANGUAGES
                .get(app.settings.lang_idx)
                .and_then(|l| l.quotes)
                .is_some()
            {
                Line::from(Span::styled(
                    "random quote from 6000+ collection",
                    Style::default().fg(C_PENDING),
                ))
            } else {
                Line::from(Span::styled(
                    "no quotes available for this language",
                    Style::default().fg(C_WRONG),
                ))
            }
        }
    };

    // Hint when custom input is active
    let opts_display = if app.custom_input.is_some() {
        Paragraph::new(vec![
            opts_line,
            Line::from(Span::styled(
                "type a number and press enter",
                Style::default().fg(C_DIM),
            )),
        ])
        .alignment(Alignment::Center)
    } else {
        Paragraph::new(opts_line).alignment(Alignment::Center)
    };
    f.render_widget(opts_display, opts_a);

    // Language row
    let lang = LANGUAGES
        .get(app.settings.lang_idx)
        .unwrap_or(&LANGUAGES[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("language  ", Style::default().fg(C_DIM)),
            Span::styled(
                lang.name,
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}/{}", app.settings.lang_idx + 1, LANGUAGES.len()),
                Style::default().fg(C_DIM),
            ),
        ]))
        .alignment(Alignment::Center),
        lang_a,
    );

    // Size row
    let size_spans: Vec<Span> = lang
        .sizes
        .iter()
        .enumerate()
        .flat_map(|(i, sz)| {
            let span = if i == app.settings.size_idx {
                Span::styled(
                    sz.label,
                    Style::default()
                        .fg(C_ACCENT)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
            } else {
                Span::styled(sz.label, Style::default().fg(C_PENDING))
            };
            if i + 1 < lang.sizes.len() {
                vec![span, Span::raw("  ")]
            } else {
                vec![span]
            }
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
            Span::raw("     "),
            toggle_span("numbers", app.settings.numbers),
        ]))
        .alignment(Alignment::Center),
        toggles_a,
    );

    // Footer hints — pinned to bottom of frame so they survive small terminals
    let footer_a = pin_footer(f.area(), 2);
    let footer_paragraph = if app.custom_input.is_some() {
        Paragraph::new(Line::from(vec![
            kh("enter"),
            Span::raw(" start"),
            sep(),
            kh("esc"),
            Span::raw(" cancel"),
        ]))
    } else {
        Paragraph::new(vec![
            Line::from(vec![
                kh("enter"),
                Span::raw(" start"),
                sep(),
                kh("1/2/3"),
                Span::raw(" mode"),
                sep(),
                kh("←/→"),
                Span::raw(" option"),
                sep(),
                kh("h"),
                Span::raw(" history"),
                sep(),
                kh("?"),
                Span::raw(" help"),
                sep(),
                kh("q"),
                Span::raw(" quit"),
            ]),
            Line::from(vec![
                kh("l"),
                Span::raw(" lang"),
                sep(),
                kh("p"),
                Span::raw(" punct"),
                sep(),
                kh("n"),
                Span::raw(" numbers"),
                sep(),
                kh("s"),
                Span::raw(" settings"),
            ]),
        ])
    };
    f.render_widget(footer_paragraph.alignment(Alignment::Center), footer_a);
}

// ── Test ──────────────────────────────────────────────────────────────────────

fn draw_test(f: &mut Frame, app: &App) {
    let area = f.area();
    let [header_a, _, words_a, _, stats_a, _] = Layout::vertical([
        Constraint::Length(1), // gauge / counter
        Constraint::Length(1), // gap
        Constraint::Length(3), // 3-line word display
        Constraint::Length(1), // gap
        Constraint::Length(1), // live wpm/acc
        Constraint::Length(1), // key hints
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
            let ratio = if app.game.started_at.is_some() {
                left as f64 / total as f64
            } else {
                1.0
            };
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(C_ACCENT).bg(C_GAUGE_BG))
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
            // words_typed counts spaces (n-1 for n words); clamp to total when all chars done
            let (ratio, label) = if app.game.cursor >= app.game.chars.len() {
                (1.0, total)
            } else {
                ((done as f64 / total as f64).clamp(0.0, 1.0), done)
            };
            f.render_widget(
                Gauge::default()
                    .gauge_style(Style::default().fg(C_ACCENT).bg(C_GAUGE_BG))
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
                    .gauge_style(Style::default().fg(C_ACCENT).bg(C_GAUGE_BG))
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
        let cursor_shape = if is_active {
            Some(app.settings.cursor_shape)
        } else {
            None
        };
        let line = build_word_line(&app.game, word_idxs, is_active, cursor_shape);
        f.render_widget(Paragraph::new(line), sub[i]);
    }

    // Position terminal cursor at current char so SetCursorStyle is visible.
    // Find which render row the cursor is actually on and compute the col within that row.
    if !app.game.is_finished() {
        let cursor_word = app.game.word_at_cursor();
        for (row, word_idxs) in lines.iter().take(3).enumerate() {
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

    // "start typing" overlay before first keystroke
    if app.game.started_at.is_none() {
        let hint_area = Rect {
            x: words_inner.x,
            y: words_inner.y + 1, // centre row of the 3 lines
            width: words_inner.width,
            height: 1,
        };
        f.render_widget(Clear, hint_area);
        f.render_widget(
            Paragraph::new(Span::styled(
                "start typing…",
                Style::default().fg(C_DIM).add_modifier(Modifier::ITALIC),
            ))
            .alignment(Alignment::Center),
            hint_area,
        );
    }

    // ── live stats ──
    let not_started = app.game.started_at.is_none();
    if not_started {
        // Show mode label instead of zeroed stats
        let mode_label = match app.game.mode {
            Mode::Time(s) => format!("time  {s}s"),
            Mode::Words(n) => format!("words  {n}"),
            Mode::Quote => "quote".to_string(),
        };
        let lang = LANGUAGES
            .get(app.settings.lang_idx)
            .map(|l| l.name)
            .unwrap_or("english");
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(mode_label, Style::default().fg(C_DIM)),
                Span::styled(format!("  ·  {lang}"), Style::default().fg(C_DIM)),
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
                    Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" wpm", Style::default().fg(C_DIM)),
                Span::styled("   ", Style::default()),
                Span::styled(format!("{acc:.1}%"), Style::default().fg(C_FG)),
                Span::styled(" acc", Style::default().fg(C_DIM)),
            ]))
            .alignment(Alignment::Center),
            stats_a,
        );
    }

    // ── footer hints — pinned to bottom of frame ──
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
        ]))
        .style(Style::default().fg(C_DIM))
        .alignment(Alignment::Center),
        footer_a,
    );
}

fn build_word_line<'a>(
    game: &'a crate::game::GameState,
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
        CharState::Correct => Style::default().fg(C_CORRECT),
        CharState::Wrong => {
            let s = Style::default()
                .fg(C_WRONG)
                .add_modifier(Modifier::UNDERLINED);
            if dim_pending { s } else { s.bg(C_WRONG_BG) }
        }
        CharState::Current => match cursor_shape {
            Some(CursorShape::Block) | None => Style::default().fg(BG).bg(C_ACCENT),
            Some(CursorShape::Underline) => Style::default()
                .fg(C_ACCENT)
                .add_modifier(Modifier::UNDERLINED),
            Some(CursorShape::Bar) => Style::default().fg(C_ACCENT),
        },
        CharState::Pending => {
            let color = if dim_pending { C_DIM } else { C_PENDING };
            Style::default().fg(color)
        }
    }
}

// ── Result ────────────────────────────────────────────────────────────────────

fn draw_result(f: &mut Frame, app: &App) {
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

    draw_result_left_panel(f, left_a, app);
    draw_result_right_panel(f, right_a, app);
    draw_result_chart(f, chart_a, app);
    draw_result_footer(f, pin_footer(f.area(), 1));
}

fn draw_result_left_panel(f: &mut Frame, area: Rect, app: &App) {
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

    let wpm_color = if failed { C_WRONG } else { C_ACCENT };
    let wpm_line = if app.is_new_pb && !failed {
        Line::from(vec![
            Span::styled(
                format!("{wpm:.0}"),
                Style::default().fg(wpm_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ★", Style::default().fg(C_ACCENT)),
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
            Line::from(Span::styled("wpm", Style::default().fg(C_DIM))),
            wpm_line,
        ]),
        lwpm_a,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("acc", Style::default().fg(C_DIM))),
            Line::from(Span::styled(
                format!("{acc:.1}%"),
                Style::default()
                    .fg(if failed { C_WRONG } else { C_ACCENT })
                    .add_modifier(Modifier::BOLD),
            )),
        ]),
        lacc_a,
    );
    let diff = app.game.settings.difficulty;
    let mut type_lines = vec![
        Line::from(Span::styled("test type", Style::default().fg(C_DIM))),
        Line::from(Span::styled(mode_str, Style::default().fg(C_ACCENT))),
        Line::from(Span::styled(lang, Style::default().fg(C_ACCENT))),
    ];
    if diff != crate::game::Difficulty::Normal {
        type_lines.push(Line::from(Span::styled(
            diff.label(),
            Style::default().fg(C_ACCENT),
        )));
    }
    if let Some(reason) = app.game.fail_reason() {
        type_lines.push(Line::from(Span::styled(
            format!("invalid ({reason})"),
            Style::default().fg(C_WRONG),
        )));
    }
    f.render_widget(Paragraph::new(type_lines), ltype_a);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("raw", Style::default().fg(C_DIM))),
            Line::from(Span::styled(
                format!("{raw:.0}"),
                Style::default().fg(C_FG).add_modifier(Modifier::BOLD),
            )),
        ]),
        lraw_a,
    );
    if let Some(src) = &app.game.quote_source {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!("— {src}"),
                Style::default()
                    .fg(C_PENDING)
                    .add_modifier(Modifier::ITALIC),
            )),
            src_a,
        );
    }
}

fn draw_result_right_panel(f: &mut Frame, area: Rect, app: &App) {
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
            Line::from(Span::styled("characters", Style::default().fg(C_DIM))),
            Line::from(Span::styled(
                format!("{correct}/{wrong}"),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Right),
        rchars_a,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("consistency", Style::default().fg(C_DIM))),
            Line::from(Span::styled(
                format!("{cons:.0}%"),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Right),
        rcons_a,
    );
    let session_secs = app.result_session_secs;
    let afk = app.game.afk_secs;
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("time", Style::default().fg(C_DIM))),
            Line::from(Span::styled(
                format!("{elapsed:.1}s"),
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "{:02}:{:02}:{:02} session",
                    session_secs / 3600,
                    session_secs % 3600 / 60,
                    session_secs % 60
                ),
                Style::default().fg(C_DIM),
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
                Style::default().fg(C_DIM),
            )),
        ])
        .alignment(Alignment::Right),
        rtime_a,
    );
}

fn draw_result_chart(f: &mut Frame, area: Rect, app: &App) {
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

    // moving average window=3 for scale line
    let scale_data: Vec<(f64, f64)> = samples
        .windows(3)
        .enumerate()
        .map(|(i, w)| (i as f64 + 1.0, w.iter().sum::<f64>() / w.len() as f64))
        .collect();

    // error scatter: fixed at bottom of chart where errors occurred
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
                Style::default().fg(C_DIM),
            ))
        })
        .collect();
    let x_labels = vec![
        Line::from(Span::styled("1", Style::default().fg(C_DIM))),
        Line::from(Span::styled(
            if n <= 2 {
                String::new()
            } else {
                format!("{}", n / 2)
            },
            Style::default().fg(C_DIM),
        )),
        Line::from(Span::styled(format!("{n}"), Style::default().fg(C_DIM))),
    ];

    let mut datasets = vec![
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(C_PENDING))
            .data(&burst_data),
        Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(C_SUB))
            .data(&raw_data),
    ];
    if scale_data.len() >= 2 {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(C_ACCENT))
                .data(&scale_data),
        );
    }
    if !err_data.is_empty() {
        datasets.push(
            Dataset::default()
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(C_WRONG))
                .data(&err_data),
        );
    }

    let chart = Chart::new(datasets)
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .labels(x_labels)
                .style(Style::default().fg(C_DIM)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_max])
                .labels(y_labels)
                .style(Style::default().fg(C_DIM)),
        );

    f.render_widget(chart, chart_body_a);

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("─ burst  ", Style::default().fg(C_PENDING)),
            Span::styled("─ raw  ", Style::default().fg(C_SUB)),
            Span::styled("─ scale  ", Style::default().fg(C_ACCENT)),
            Span::styled("• errors", Style::default().fg(C_WRONG)),
        ]))
        .alignment(Alignment::Center),
        legend_a,
    );
}

fn draw_result_footer(f: &mut Frame, area: Rect) {
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
        .style(Style::default().fg(C_DIM))
        .alignment(Alignment::Center),
        area,
    );
}

// ── History ───────────────────────────────────────────────────────────────────

fn draw_history(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 80, f.area());
    let [title_a, _, summary_a, _, header_a, entries_a, _] = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Length(1), // gap
        Constraint::Length(1), // summary stats
        Constraint::Length(1), // gap
        Constraint::Length(1), // column header
        Constraint::Min(0),    // entries
        Constraint::Length(1), // footer
    ])
    .split(area)[..] else {
        return;
    };

    f.render_widget(
        Paragraph::new(Span::styled(
            "history",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    if !app.history.is_empty() {
        let best = app.history.iter().map(|e| e.wpm).fold(0.0f64, f64::max);
        let avg = app.history.iter().map(|e| e.wpm).sum::<f64>() / app.history.len() as f64;
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("best ", Style::default().fg(C_DIM)),
                Span::styled(format!("{best:.0}"), Style::default().fg(C_FG)),
                Span::styled(" wpm", Style::default().fg(C_DIM)),
                Span::styled("     ", Style::default()),
                Span::styled("avg ", Style::default().fg(C_DIM)),
                Span::styled(format!("{avg:.0}"), Style::default().fg(C_FG)),
                Span::styled(" wpm", Style::default().fg(C_DIM)),
                Span::styled("     ", Style::default()),
                Span::styled(
                    format!("{} tests", app.history.len()),
                    Style::default().fg(C_DIM),
                ),
            ]))
            .alignment(Alignment::Center),
            summary_a,
        );
    }

    f.render_widget(
        Paragraph::new(Line::from(vec![
            col("wpm", 6, C_DIM),
            col("acc", 7, C_DIM),
            col("lang", 11, C_DIM),
            col("mode", 10, C_DIM),
            col("when", 10, C_DIM),
        ])),
        header_a,
    );

    let max_rows = entries_a.height as usize;
    let lines: Vec<Line> = if app.history.is_empty() {
        vec![Line::from(Span::styled(
            "no results yet",
            Style::default().fg(C_DIM),
        ))]
    } else {
        app.history
            .iter()
            .skip(app.history_scroll)
            .take(max_rows)
            .map(|e| {
                Line::from(vec![
                    col(format!("{:.0}", e.wpm), 6, C_ACCENT),
                    col(format!("{:.1}%", e.accuracy), 7, C_FG),
                    col(
                        if e.language.is_empty() {
                            "—".to_string()
                        } else {
                            e.language.clone()
                        },
                        11,
                        C_PENDING,
                    ),
                    col(&e.mode, 10, C_FG),
                    col(e.time_ago(), 10, C_PENDING),
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
            Span::styled(scroll_hint, Style::default().fg(C_DIM)),
        ]))
        .style(Style::default().fg(C_DIM))
        .alignment(Alignment::Center),
        pin_footer(f.area(), 1),
    );
}

// ── Settings ──────────────────────────────────────────────────────────────────

fn draw_settings(f: &mut Frame, app: &App) {
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

// ── Help ──────────────────────────────────────────────────────────────────────

fn draw_help(f: &mut Frame) {
    let area = centered_rect(54, 90, f.area());
    let [title_a, _, body_a, _] = Layout::vertical([
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
            "help",
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );

    let kw = |k: &'static str| Span::styled(format!("{k:<16}"), Style::default().fg(C_ACCENT));
    let dsc = |d: &'static str| Span::styled(d, Style::default().fg(C_FG));
    let sec = |s: &'static str| {
        Line::from(Span::styled(
            s,
            Style::default().fg(C_DIM).add_modifier(Modifier::BOLD),
        ))
    };
    let row = |k, d| Line::from(vec![kw(k), dsc(d)]);

    f.render_widget(
        Paragraph::new(vec![
            sec("── menu ──────────────────────────────────────"),
            row("1 / 2 / 3", "select mode  (time · words · quote)"),
            row("← / →", "change option value"),
            row("enter", "start test  (or open custom input)"),
            row("l", "open language picker"),
            row("p", "toggle punctuation"),
            row("n", "toggle numbers"),
            row("s", "settings  (sound, volume, …)"),
            row("h", "history"),
            row("? ", "this help"),
            row("q  /  ctrl+c", "quit"),
            Line::from(""),
            sec("── test ────────────────────────────────────────"),
            row("tab", "restart test (new words)"),
            row("esc", "back to menu"),
            row("backspace", "delete last character"),
            row("ctrl+backspace", "delete whole word"),
            Line::from(""),
            sec("── result ──────────────────────────────────────"),
            row("r", "repeat same words"),
            row("enter / tab", "new test (new words)"),
            row("esc", "back to menu"),
            Line::from(""),
            sec("── language picker ─────────────────────────────"),
            row("↑ / ↓", "navigate languages"),
            row("← / →", "change word pool size"),
            row("enter", "confirm selection"),
            row("esc", "cancel"),
        ]),
        body_a,
    );

    f.render_widget(
        Paragraph::new(Line::from(vec![kh("esc"), Span::raw(" back")]))
            .style(Style::default().fg(C_DIM))
            .alignment(Alignment::Center),
        pin_footer(f.area(), 1),
    );
}

// ── Lang Picker ───────────────────────────────────────────────────────────────

fn draw_lang_picker(f: &mut Frame, app: &App) {
    let picker = match &app.lang_picker {
        Some(p) => p,
        None => return,
    };
    const VISIBLE: usize = LANG_PICKER_VISIBLE;

    let filtered = filtered_languages(&picker.search);

    let area = centered_rect(54, 75, f.area());

    f.render_widget(Clear, area);
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_DIM))
            .title(Span::styled(
                " language ",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(BG)),
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
    ])
    .split(inner)[..] else {
        return;
    };

    // Search bar with filtered count
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!("▶ {}_", picker.search), Style::default().fg(C_FG)),
            Span::styled(
                format!(" ({}/{})", filtered.len(), LANGUAGES.len()),
                Style::default().fg(C_DIM),
            ),
        ])),
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
            if matches!(app.menu_mode, Mode::Quote) && lang.quotes.is_none() {
                spans.push(Span::styled("  no quotes", Style::default().fg(C_WRONG)));
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
            kh("↑/↓"),
            Span::raw(" navigate"),
            sep(),
            kh("←/→"),
            Span::raw(" size"),
            sep(),
            kh("enter"),
            Span::raw(" select"),
            sep(),
            kh("esc"),
            Span::raw(" cancel"),
            Span::styled(scroll_info, Style::default().fg(C_DIM)),
        ]))
        .style(Style::default().fg(C_DIM))
        .alignment(Alignment::Center),
        footer_a,
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns a rect pinned to the absolute bottom of `frame`, always visible
/// regardless of layout constraints.
fn pin_footer(frame: Rect, height: u16) -> Rect {
    Rect {
        x: frame.x,
        y: frame.bottom().saturating_sub(height),
        width: frame.width,
        height: height.min(frame.height),
    }
}

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - pct_y) / 2),
        Constraint::Percentage(pct_y),
        Constraint::Percentage((100 - pct_y) / 2),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ])
    .split(v[1])[1]
}

fn horiz_pad(r: Rect, pad: u16) -> Rect {
    Rect {
        x: r.x + pad,
        y: r.y,
        width: r.width.saturating_sub(pad * 2),
        height: r.height,
    }
}

fn mode_tab_n(num: &'static str, label: impl Into<String>, active: bool) -> Span<'static> {
    let text = format!("{num}·{}", label.into());
    if active {
        Span::styled(
            text,
            Style::default()
                .fg(C_ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else {
        Span::styled(text, Style::default().fg(C_PENDING))
    }
}

fn toggle_span(label: &str, on: bool) -> Span<'static> {
    if on {
        Span::styled(
            format!("[{label}]"),
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(format!(" {label} "), Style::default().fg(C_DIM))
    }
}

fn option_spans<T: std::fmt::Display>(
    opts: &[T],
    selected: usize,
    suffix: &str,
) -> Vec<Span<'static>> {
    opts.iter()
        .enumerate()
        .flat_map(|(i, v)| {
            let label = format!("{v}{suffix}");
            let span = if i == selected {
                Span::styled(
                    label,
                    Style::default()
                        .fg(C_ACCENT)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
            } else {
                Span::styled(label, Style::default().fg(C_PENDING))
            };
            vec![span, Span::raw("  ")]
        })
        .collect()
}

fn custom_slot<'a>(selected: bool, suffix: &str, input: &Option<String>) -> Span<'a> {
    if selected {
        let text = if let Some(s) = input {
            format!("custom: {s}▌{suffix}")
        } else {
            "custom".to_string()
        };
        Span::styled(
            text,
            Style::default()
                .fg(C_ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
    } else {
        Span::styled("custom", Style::default().fg(C_PENDING))
    }
}

fn sep() -> Span<'static> {
    Span::raw("   ")
}

// key highlight span
fn kh(key: &str) -> Span<'static> {
    Span::styled(key.to_string(), Style::default().fg(C_SUB))
}

fn col<S: Into<String>>(s: S, w: usize, color: Color) -> Span<'static> {
    Span::styled(format!("{:<w$}", s.into()), Style::default().fg(color))
}
