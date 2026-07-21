//! Renders the Menu screen: mode/option selection plus the language and
//! theme picker overlays.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, TIME_OPTIONS, WORD_OPTIONS};
use crate::game::{Difficulty, Mode, QuoteFilter};
use crate::words::{LANGUAGES, lang_at};

use super::*;

/// Rows the vertical layout in [`draw_menu`] asks for. Declared next to that
/// layout so the two can't drift apart.
const MENU_HEIGHT: u16 = 10;

/// Columns below which menu rows start clipping. The binding culprits are the
/// mode tabs (`1·time·3600s    2·words    3·quote`) and the toggles row, which
/// also carries the difficulty tag and the theme name. At `MIN_WIDTH` a plain
/// 56% would be 33 columns — one short of the tab row alone.
const MENU_MIN_WIDTH: u16 = 56;

pub(super) fn draw_menu(f: &mut Frame, app: &App) {
    let width = pct(f.area().width, 56).max(MENU_MIN_WIDTH);
    let area = centered_block(f.area(), width, MENU_HEIGHT);
    let [title_a, _, tabs_a, opts_a, _, lang_a, size_a, _, toggles_a] = Layout::vertical([
        Constraint::Length(1), // title
        Constraint::Length(1), // gap
        Constraint::Length(1), // mode tabs  1·time  2·words  3·quote
        Constraint::Length(2), // option row, plus the custom-input hint below it
        Constraint::Length(1), // gap
        Constraint::Length(1), // language
        Constraint::Length(1), // word-pool size
        Constraint::Length(1), // gap
        Constraint::Length(1), // toggles
    ])
    .split(area)[..] else {
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
            // Wider than `label_row`'s inter-label gap, to set the custom slot apart.
            spans.push(Span::raw("    "));
            spans.push(custom_slot(
                app.menu.time_idx == TIME_OPTIONS.len(),
                "s",
                &app.menu.custom_input,
            ));
            Line::from(spans)
        }
        Mode::Words(_) => {
            let mut spans = option_spans(WORD_OPTIONS, app.menu.word_idx, "");
            spans.push(Span::raw("    "));
            spans.push(custom_slot(
                app.menu.word_idx == WORD_OPTIONS.len(),
                "",
                &app.menu.custom_input,
            ));
            Line::from(spans)
        }
        Mode::Quote => {
            if lang_at(app.settings.lang_idx).quotes.is_some() {
                // Driven by the same ORDER `next`/`prev` cycle through, so a new
                // filter variant can't appear in one and be missing from the other.
                let selected = QuoteFilter::ALL
                    .iter()
                    .position(|&f| f == app.settings.quote_filter)
                    .unwrap_or(0);
                Line::from(label_row(
                    QuoteFilter::ALL.iter().map(|f| f.label().to_string()),
                    selected,
                ))
            } else {
                Line::from(Span::styled(
                    "no quotes available for this language",
                    Style::default().fg(th_wrong()),
                ))
            }
        }
    };

    // `opts_a` is two rows tall whether or not the custom slot is open, so the
    // hint appearing doesn't shift every row beneath it.
    let hint_line = if app.menu.custom_input.is_some() {
        Line::from(Span::styled(
            "type a number and press enter",
            Style::default().fg(th_dim()),
        ))
    } else {
        Line::default()
    };
    f.render_widget(
        Paragraph::new(vec![opts_line, hint_line]).alignment(Alignment::Center),
        opts_a,
    );

    let lang = lang_at(app.settings.lang_idx);
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

    let size_spans = label_row(
        lang.sizes.iter().map(|sz| sz.label.to_string()),
        app.settings.size_idx,
    );
    f.render_widget(
        Paragraph::new(Line::from(size_spans)).alignment(Alignment::Center),
        size_a,
    );

    let mut toggle_line = vec![
        toggle_span("punctuation", app.settings.punctuation),
        Span::raw("     "),
        toggle_span("numbers", app.settings.numbers),
    ];
    // Surface persisted difficulty (when not default) and the active theme so
    // saved preferences are visible at a glance.
    if app.settings.difficulty != Difficulty::Normal {
        toggle_line.push(Span::raw("     "));
        toggle_line.push(Span::styled(
            app.settings.difficulty.label(),
            Style::default().fg(th_accent()),
        ));
    }
    toggle_line.push(Span::raw("     "));
    toggle_line.push(Span::styled(
        format!("◆ {}", app.settings.theme_name),
        Style::default().fg(th_accent()),
    ));
    f.render_widget(
        Paragraph::new(Line::from(toggle_line)).alignment(Alignment::Center),
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
                kh("t"),
                Span::raw(" theme"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, MIN_HEIGHT, MIN_WIDTH};
    use crate::ui::test_render::text;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn rendered_text(app: &App) -> String {
        text(80, 24, |f| draw_menu(f, app))
    }

    /// The menu block is sized from `MENU_HEIGHT` rather than a percentage of
    /// the terminal, so every row survives at every height.
    ///
    /// With the old 30%-height rect the fixed `Length(1)` rows were
    /// over-constrained and the solver zeroed some of them out. Which rows
    /// were lost depended on how 30% rounded: at h=21/23/25 the mode-tab and
    /// word-pool-size rows both vanished, while h=24 happened to absorb the
    /// deficit in trailing padding and looked fine. Nothing errored — the
    /// `let [..] else` guard can't catch it, because the rect count still
    /// matches. Hence the sweep: a two-size check passes on the old code.
    #[test]
    fn every_menu_row_renders_at_every_height() {
        let app = App::new();
        for h in MIN_HEIGHT..=44 {
            let screen = text(80, h, |f| draw_menu(f, &app));
            for expected in [
                "monkeytype",
                "3·quote",
                "language",
                "default",
                "punctuation",
            ] {
                assert!(
                    screen.contains(expected),
                    "row {expected:?} missing at 80x{h}:\n{screen}"
                );
            }
        }
    }

    #[test]
    fn every_menu_row_renders_at_the_minimum_terminal_width() {
        let app = App::new();
        let screen = text(MIN_WIDTH, MIN_HEIGHT, |f| draw_menu(f, &app));
        for expected in ["monkeytype", "3·quote", "language", "punctuation"] {
            assert!(
                screen.contains(expected),
                "row {expected:?} missing at {MIN_WIDTH}x{MIN_HEIGHT}:\n{screen}"
            );
        }
    }

    /// The hint was the second line of a Paragraph rendered into a
    /// `Length(1)` rect, so it was clipped and had never once been visible.
    #[test]
    fn custom_input_hint_is_visible_while_typing_a_custom_value() {
        let mut app = App::new();
        app.menu.time_idx = TIME_OPTIONS.len();
        app.menu.mode = Mode::Time(app.menu.custom_time_val);
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(
            app.menu.custom_input.is_some(),
            "custom input should be open"
        );

        let screen = rendered_text(&app);
        assert!(
            screen.contains("type a number and press enter"),
            "custom-input hint missing:\n{screen}"
        );
    }

    /// The quote-filter row renders `QuoteFilter::ALL`, the same order
    /// `next`/`prev` cycle through. This catches a variant that is reachable
    /// with ←/→ but never drawn (what a hand-written second list allowed).
    #[test]
    fn quote_mode_renders_every_quote_filter() {
        let mut app = App::new();
        app.menu.mode = Mode::Quote;
        app.settings.lang_idx = 0; // english, which has quotes
        let text = rendered_text(&app);
        for filter in QuoteFilter::ALL {
            assert!(
                text.contains(filter.label()),
                "filter {} missing from the menu: {text}",
                filter.label()
            );
        }
    }
}
