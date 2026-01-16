use ratatui::{layout::Rect, Frame};

use crate::actions::{Action, ConfirmCallback, InputCallback};
use crate::components::{Component, FuzzyList};
use crate::error::Result;
use crate::integrations::TmuxClient;
use crate::models::TmuxSession;

pub struct SessionPicker {
    fuzzy_list: FuzzyList<TmuxSession>,
    tmux: TmuxClient,
}

impl SessionPicker {
    pub fn new() -> Self {
        Self {
            fuzzy_list: FuzzyList::new(
                "Sessions",
                TmuxSession::display_name,
                TmuxSession::search_text,
            ),
            tmux: TmuxClient::new(),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        let sessions = self.tmux.list_sessions()?;
        self.fuzzy_list.set_items(sessions);
        Ok(())
    }
}

impl Default for SessionPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SessionPicker {
    fn handle_action(&mut self, action: &Action) -> Result<Option<Action>> {
        match action {
            Action::MoveUp => {
                self.fuzzy_list.move_up();
                Ok(Some(Action::Render))
            }
            Action::MoveDown => {
                self.fuzzy_list.move_down();
                Ok(Some(Action::Render))
            }
            Action::PageUp => {
                self.fuzzy_list.page_up(10);
                Ok(Some(Action::Render))
            }
            Action::PageDown => {
                self.fuzzy_list.page_down(10);
                Ok(Some(Action::Render))
            }
            Action::Character(c) => {
                match c {
                    'd' if self.fuzzy_list.query().is_empty() => {
                        // Delete session
                        if let Some(session) = self.fuzzy_list.selected() {
                            return Ok(Some(Action::ShowConfirm {
                                title: "Delete Session".to_string(),
                                message: format!("Delete session '{}'?", session.name),
                                callback: ConfirmCallback::KillSession(session.name.clone()),
                            }));
                        }
                        Ok(None)
                    }
                    'n' if self.fuzzy_list.query().is_empty() => {
                        // New session
                        Ok(Some(Action::ShowInput {
                            title: "New Session".to_string(),
                            callback: InputCallback::CreateSession,
                        }))
                    }
                    _ => {
                        self.fuzzy_list.push_char(*c);
                        Ok(Some(Action::Render))
                    }
                }
            }
            Action::Backspace => {
                self.fuzzy_list.pop_char();
                Ok(Some(Action::Render))
            }
            Action::Enter => {
                if let Some(session) = self.fuzzy_list.selected() {
                    Ok(Some(Action::SwitchSession(session.name.clone())))
                } else {
                    Ok(None)
                }
            }
            Action::Escape => {
                if !self.fuzzy_list.query().is_empty() {
                    self.fuzzy_list.clear_query();
                    Ok(Some(Action::Render))
                } else {
                    Ok(Some(Action::GoBack))
                }
            }
            _ => Ok(None),
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.fuzzy_list.render(frame, area);
    }

    fn help_text(&self) -> &'static str {
        "Enter:switch  n:new  d:delete  Esc:back"
    }
}
