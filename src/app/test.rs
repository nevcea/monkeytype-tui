use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, Screen};

impl App {
    pub(super) fn handle_test(&mut self, key: KeyEvent) {
        // The test screen is hidden behind the too-small overlay; ignore typing
        // so keystrokes aren't recorded blindly. Esc still lets the user leave.
        if self.too_small() && key.code != KeyCode::Esc {
            return;
        }
        match key.code {
            KeyCode::Esc => {
                if self.game.started_at.is_some() {
                    self.test_confirm = true;
                    self.test_confirm_yes = false;
                } else {
                    self.screen = Screen::Menu;
                }
            }
            KeyCode::Tab => self.restart_test(),
            KeyCode::Backspace => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.game.delete_word();
                } else {
                    self.game.backspace();
                }
                self.update_scroll();
            }
            KeyCode::Char(c) => {
                self.game.type_char(c);
                self.update_scroll();
                if self.game.is_finished() {
                    self.save_result();
                    self.screen = Screen::Result;
                } else if let Some(s) = &self.sound {
                    if self.game.last_char_correct() {
                        s.play_correct();
                    } else {
                        s.play_error();
                    }
                }
            }
            _ => {}
        }
    }
}
