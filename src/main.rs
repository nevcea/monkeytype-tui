mod app;
mod game;
mod history;
mod pb;
mod sound;
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
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> io::Result<()> {
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
        // Track terminal width for scroll calculations
        app.last_width = terminal.size()?.width;

        terminal.draw(|f| ui::draw(f, &app))?;

        // Re-apply every frame: terminal.draw() can reset cursor style when repositioning
        let cursor_style = match app.settings.cursor_shape {
            CursorShape::Bar => SetCursorStyle::BlinkingBar,
            CursorShape::Block => SetCursorStyle::BlinkingBlock,
            CursorShape::Underline => SetCursorStyle::BlinkingUnderScore,
        };
        execute!(terminal.backend_mut(), cursor_style)?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key);
        }

        app.tick();

        if app.should_quit {
            return Ok(());
        }
    }
}
