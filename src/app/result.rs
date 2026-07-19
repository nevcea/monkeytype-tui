//! Key handling for the Result screen: repeat the same test, restart with a
//! fresh one, or return to the menu.

use crossterm::event::{KeyCode, KeyEvent};

use super::{App, Screen};

impl App {
    pub(super) fn handle_result(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r' | 'R') => self.repeat_test(),
            KeyCode::Enter | KeyCode::Tab => self.restart_test(),
            KeyCode::Esc => self.screen = Screen::Menu,
            _ => {}
        }
    }
}
