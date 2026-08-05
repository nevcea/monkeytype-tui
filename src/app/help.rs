//! Key handling for the Help overlay (any key returns to the menu).

use crossterm::event::{KeyCode, KeyEvent};

use super::{App, Screen};

impl App {
    pub(super) fn handle_help(&mut self, key: KeyEvent) {
        let max = self.help_max_scroll();
        let page = self.help_visible_rows();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q' | 'Q' | '?') => {
                self.help_scroll = 0;
                self.screen = Screen::Menu;
            }
            KeyCode::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
            KeyCode::Down => self.help_scroll = (self.help_scroll + 1).min(max),
            KeyCode::PageUp => self.help_scroll = self.help_scroll.saturating_sub(page),
            KeyCode::PageDown => self.help_scroll = (self.help_scroll + page).min(max),
            KeyCode::Home => self.help_scroll = 0,
            KeyCode::End => self.help_scroll = max,
            _ => {}
        }
    }
}
