use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, TIME_OPTIONS, WORD_OPTIONS};
use crate::game::{Mode, QuoteFilter};
use crate::words::LANGUAGES;

use super::*;

pub(super) fn draw_menu(f: &mut Frame, app: &App) {
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

    f.render_widget(
        Paragraph::new(Span::styled(
            "monkeytype",
            Style::default()
                .fg(th_accent())
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        title_a,
    );
    let _ = gap1;
    let _ = gap2;
    let _ = gap3;
    let _ = gap4;

    let time_label = match app.menu.mode {
        Mode::Time(n) => format!("time·{n}s"),
        _ => "time".to_owned(),
    };
    let words_label = match app.menu.mode {
        Mode::Words(n) => format!("words·{n}"),
        _ => "words".to_owned(),
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            mode_tab_n("1", time_label, matches!(app.menu.mode, Mode::Time(_))),
            Span::raw("    "),
            mode_tab_n("2", words_label, matches!(app.menu.mode, Mode::Words(_))),
            Span::raw("    "),
            mode_tab_n("3", "quote", matches!(app.menu.mode, Mode::Quote)),
        ]))
        .alignment(Alignment::Center),
        tabs_a,
    );

    let opts_line = match app.menu.mode {
        Mode::Time(_) => {
            let mut spans = option_spans(TIME_OPTIONS, app.menu.time_idx, "s");
            spans.push(Span::raw("  "));
            spans.push(custom_slot(
                app.menu.time_idx == TIME_OPTIONS.len(),
                "s",
                &app.menu.custom_input,
            ));
            Line::from(spans)
        }
        Mode::Words(_) => {
            let mut spans = option_spans(WORD_OPTIONS, app.menu.word_idx, "");
            spans.push(Span::raw("  "));
            spans.push(custom_slot(
                app.menu.word_idx == WORD_OPTIONS.len(),
                "",
                &app.menu.custom_input,
            ));
            Line::from(spans)
        }
        Mode::Quote => {
            let has_quotes = LANGUAGES
                .get(app.settings.lang_idx)
                .and_then(|l| l.quotes)
                .is_some();
            if has_quotes {
                let f = app.settings.quote_filter;
                let filters = [
                    QuoteFilter::All,
                    QuoteFilter::Short,
                    QuoteFilter::Medium,
                    QuoteFilter::Long,
                    QuoteFilter::Thicc,
                ];
                let mut spans = vec![];
                for (i, &filter) in filters.iter().enumerate() {
                    let active = f == filter;
                    if active {
                        spans.push(Span::styled(
                            filter.label(),
                            Style::default()
                                .fg(th_accent())
                                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                        ));
                    } else {
                        spans.push(Span::styled(
                            filter.label(),
                            Style::default().fg(th_pending()),
                        ));
                    }
                    if i + 1 < filters.len() {
                        spans.push(Span::raw("  "));
                    }
                }
                Line::from(spans)
            } else {
                Line::from(Span::styled(
                    "no quotes available for this language",
                    Style::default().fg(th_wrong()),
                ))
            }
        }
    };

    let opts_display = if app.menu.custom_input.is_some() {
        Paragraph::new(vec![
            opts_line,
            Line::from(Span::styled(
                "type a number and press enter",
                Style::default().fg(th_dim()),
            )),
        ])
        .alignment(Alignment::Center)
    } else {
        Paragraph::new(opts_line).alignment(Alignment::Center)
    };
    f.render_widget(opts_display, opts_a);

    let lang = LANGUAGES
        .get(app.settings.lang_idx)
        .unwrap_or(&LANGUAGES[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("language  ", Style::default().fg(th_dim())),
            Span::styled(
                lang.name,
                Style::default().fg(th_fg()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}/{}", app.settings.lang_idx + 1, LANGUAGES.len()),
                Style::default().fg(th_dim()),
            ),
        ]))
        .alignment(Alignment::Center),
        lang_a,
    );

    let size_spans: Vec<Span> = lang
        .sizes
        .iter()
        .enumerate()
        .flat_map(|(i, sz)| {
            let span = if i == app.settings.size_idx {
                Span::styled(
                    sz.label,
                    Style::default()
                        .fg(th_accent())
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
            } else {
                Span::styled(sz.label, Style::default().fg(th_pending()))
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

    f.render_widget(
        Paragraph::new(Line::from(vec![
            toggle_span("punctuation", app.settings.punctuation),
            Span::raw("     "),
            toggle_span("numbers", app.settings.numbers),
        ]))
        .alignment(Alignment::Center),
        toggles_a,
    );

    let footer_a = pin_footer(f.area(), 2);
    let footer_paragraph = if app.menu.custom_input.is_some() {
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
