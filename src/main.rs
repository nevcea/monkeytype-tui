#[macro_use]
mod macros;
mod app;
mod config;
mod game;
mod history;
mod pb;
mod sound;
mod storage;
mod ui;
mod words;

use std::{io, time::Duration};

use crossterm::{
    cursor::SetCursorStyle,
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyCode, KeyEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use game::CursorShape;
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// Best-effort terminal restoration, safe to call from a panic hook.
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        DisableMouseCapture,
        DisableBracketedPaste,
        LeaveAlternateScreen
    );
}

fn main() -> io::Result<()> {
    // Restore the terminal before the default hook prints the panic, otherwise
    // the message is swallowed by the alternate screen / raw mode. The Drop
    // guard also restores on normal unwind, but the hook makes the panic legible.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    enable_raw_mode()?;
    // Constructed immediately after raw mode is enabled, not after the
    // `execute!` below: if that call fails, the `?` must still unwind
    // through a live guard, or the terminal is left in raw mode with no
    // Drop to restore it.
    let _guard = TerminalGuard;
    // Mouse capture routes drag/click to the app instead of the terminal's own
    // selection, so text can't be highlighted and copied out of the test.
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        EnableBracketedPaste,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.show_cursor()?;
    run(&mut terminal)
}

/// Input poll interval; also the window over which a burst is measured.
const POLL_MS: u64 = 50;
/// Cap on one drain so a huge paste can't stall rendering.
const MAX_BATCH: usize = 256;
/// Printable keys queued *simultaneously* that mark a batch as a paste rather
/// than typing. Crossterm never reports `Event::Paste` on Windows (its console
/// input source only produces Key/Mouse/Resize), so Ctrl+V shows up as a burst
/// of synthetic key presses and this count is the only signal left. Six chars
/// inside one `POLL_MS` window is ~1400 WPM — unreachable by hand.
const PASTE_BURST_CHARS: usize = 6;

/// Read every event already queued, so a batch reflects what arrived together.
fn drain_events() -> io::Result<Vec<Event>> {
    let mut batch = Vec::new();
    while batch.len() < MAX_BATCH && event::poll(Duration::ZERO)? {
        batch.push(event::read()?);
    }
    Ok(batch)
}

/// Whether a batch looks like a paste rather than typing.
fn is_paste_burst(batch: &[Event]) -> bool {
    batch
        .iter()
        .filter(|e| {
            matches!(e, Event::Key(k)
                if k.kind == KeyEventKind::Press && matches!(k.code, KeyCode::Char(_)))
        })
        .count()
        >= PASTE_BURST_CHARS
}

/// Events to actually deliver from a drained batch. A paste burst only drops
/// the flood of `Char` keystrokes — a slow frame (SSH lag, a big resize) can
/// queue a real Esc or Ctrl+C alongside enough typed chars to look like a
/// paste, and discarding the *whole* batch used to swallow that keypress too.
fn paste_filtered(batch: Vec<Event>) -> Vec<Event> {
    if !is_paste_burst(&batch) {
        return batch;
    }
    batch
        .into_iter()
        .filter(|e| {
            !matches!(e, Event::Key(k)
                if k.kind == KeyEventKind::Press && matches!(k.code, KeyCode::Char(_)))
        })
        .collect()
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        let size = terminal.size()?;
        app.last_width = size.width;
        app.last_height = size.height;

        terminal.draw(|f| ui::draw(f, &app))?;

        // Re-apply every frame: terminal.draw() can reset cursor style when repositioning
        let cursor_style = match app.settings.cursor_shape {
            CursorShape::Bar => SetCursorStyle::BlinkingBar,
            CursorShape::Block => SetCursorStyle::BlinkingBlock,
            CursorShape::Underline => SetCursorStyle::BlinkingUnderScore,
        };
        execute!(terminal.backend_mut(), cursor_style)?;

        if event::poll(Duration::from_millis(POLL_MS))? {
            let batch = drain_events()?;
            for ev in paste_filtered(batch) {
                // On unix, bracketed paste arrives as a single Paste event
                // and is dropped here; `is_paste_burst` covers Windows,
                // where crossterm's console input source never emits one.
                if let Event::Key(key) = ev
                    && key.kind == KeyEventKind::Press
                {
                    app.on_key(key);
                }
            }
        }

        app.tick();

        if app.should_quit {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod paste_tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    fn chars(s: &str) -> Vec<Event> {
        s.chars()
            .map(|c| Event::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)))
            .collect()
    }

    #[test]
    fn a_burst_of_queued_chars_is_treated_as_a_paste() {
        assert!(is_paste_burst(&chars("pasted text")));
    }

    #[test]
    fn a_few_queued_chars_are_treated_as_typing() {
        assert!(!is_paste_burst(&chars("ab")));
    }

    #[test]
    fn non_char_keys_never_form_a_burst() {
        let held = vec![Event::Key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)); 20];
        assert!(!is_paste_burst(&held));
    }

    #[test]
    fn paste_filtered_drops_only_the_char_presses_in_a_burst() {
        let mut batch = chars("pasted text"); // 11 chars, well past the threshold
        batch.push(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));

        let kept = paste_filtered(batch);

        assert!(
            kept.iter()
                .all(|e| !matches!(e, Event::Key(k) if matches!(k.code, KeyCode::Char(_)))),
            "char presses from the paste must be dropped: {kept:?}"
        );
        assert!(
            kept.iter()
                .any(|e| matches!(e, Event::Key(k) if k.code == KeyCode::Esc)),
            "a real Esc queued alongside the paste must still be delivered: {kept:?}"
        );
    }

    #[test]
    fn paste_filtered_is_a_no_op_below_the_burst_threshold() {
        let mut batch = chars("ab");
        batch.push(Event::Key(KeyEvent::new(
            KeyCode::Backspace,
            KeyModifiers::NONE,
        )));
        assert_eq!(paste_filtered(batch.clone()).len(), batch.len());
    }
}
