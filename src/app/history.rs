//! Key handling for the History overlay: move the selection and dismiss.

use crossterm::event::{KeyCode, KeyEvent};

use super::{App, Screen, focus_cursor};

impl App {
    pub(super) fn handle_history(&mut self, key: KeyEvent) {
        let visible = self.history_visible_rows().max(1);
        let last = self.history.len().saturating_sub(1);
        let cursor = self.history_cursor;

        // Every movement key just names where the selection should land;
        // `focus_cursor` pulls the scroll along to keep it on screen.
        let to = match key.code {
            KeyCode::Esc | KeyCode::Char('q' | 'Q') => {
                self.history_scroll = 0;
                self.history_cursor = 0;
                self.screen = Screen::Menu;
                return;
            }
            KeyCode::Up => cursor.saturating_sub(1),
            KeyCode::Down => (cursor + 1).min(last),
            KeyCode::PageUp => cursor.saturating_sub(visible),
            KeyCode::PageDown => (cursor + visible).min(last),
            KeyCode::Home => 0,
            KeyCode::End => last,
            _ => return,
        };
        focus_cursor(
            &mut self.history_cursor,
            &mut self.history_scroll,
            to,
            visible,
        );
    }
}
