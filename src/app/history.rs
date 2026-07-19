//! Key handling for the History overlay: scroll and dismiss.

use crossterm::event::{KeyCode, KeyEvent};

use super::{App, Screen};

impl App {
    pub(super) fn handle_history(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.history_scroll = 0;
                self.screen = Screen::Menu;
            }
            KeyCode::Up => self.history_scroll = self.history_scroll.saturating_sub(1),
            KeyCode::Down => {
                let max = self.history.len().saturating_sub(1);
                if self.history_scroll < max {
                    self.history_scroll += 1;
                }
            }
            _ => {}
        }
    }
}
