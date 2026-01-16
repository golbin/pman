use std::path::Path;

use ratatui::{layout::Rect, Frame};

use crate::actions::Action;
use crate::components::{Component, FuzzyList};
use crate::error::Result;
use crate::integrations::GitClient;
use crate::models::PaletteCommand;

pub struct CommandPalette {
    fuzzy_list: FuzzyList<PaletteCommand>,
    is_git_repo: bool,
}

impl CommandPalette {
    pub fn new(current_path: &Path) -> Self {
        let is_git_repo = GitClient::is_git_repo(current_path);

        let mut palette = Self {
            fuzzy_list: FuzzyList::new(
                "Commands",
                |cmd: &PaletteCommand| format!("{} - {}", cmd.display_name(), cmd.description()),
                PaletteCommand::search_text,
            ),
            is_git_repo,
        };

        palette.refresh();
        palette
    }

    pub fn refresh(&mut self) {
        let commands = if self.is_git_repo {
            PaletteCommand::all()
        } else {
            PaletteCommand::non_git_commands()
        };
        self.fuzzy_list.set_items(commands);
    }

    pub fn set_git_repo(&mut self, is_git_repo: bool) {
        self.is_git_repo = is_git_repo;
        self.refresh();
    }
}

impl Component for CommandPalette {
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
                self.fuzzy_list.push_char(*c);
                Ok(Some(Action::Render))
            }
            Action::Backspace => {
                self.fuzzy_list.pop_char();
                Ok(Some(Action::Render))
            }
            Action::Enter => {
                if let Some(cmd) = self.fuzzy_list.selected() {
                    Ok(Some(Action::ExecuteCommand(*cmd)))
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
        "Enter:execute  Esc:back"
    }
}
