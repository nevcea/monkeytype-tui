#[macro_use]
mod macros;
mod app;
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
    event::{self, Event, KeyEventKind},
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
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
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
    execute!(io::stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.show_cursor()?;
    run(&mut terminal)
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        // Track terminal size for scroll and layout calculations
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

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => app.on_key(key),
                // Redraw immediately on resize instead of waiting for the poll timeout.
                Event::Resize(_, _) => continue,
                _ => {}
            }
        }

        app.tick();

        if app.should_quit {
            return Ok(());
        }
    }
}
