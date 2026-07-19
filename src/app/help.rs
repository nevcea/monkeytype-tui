//! Key handling for the Help overlay (any key returns to the menu).

use crossterm::event::{KeyCode, KeyEvent};

use super::{App, Screen};

impl App {
    pub(super) fn handle_help(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q' | '?') => self.screen = Screen::Menu,
            _ => {}
        }
    }
}
