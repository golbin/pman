use std::path::Path;

use ratatui::{layout::Rect, Frame};

use crate::actions::{Action, ConfirmCallback, InputCallback};
use crate::components::{Component, FuzzyList};
use crate::error::Result;
use crate::integrations::GitClient;
use crate::models::GitWorktree;

pub struct WorktreePicker {
    fuzzy_list: FuzzyList<GitWorktree>,
    git: Option<GitClient>,
}

impl WorktreePicker {
    pub fn new(current_path: &Path) -> Self {
        let git = GitClient::new(current_path).ok();

        let mut picker = Self {
            fuzzy_list: FuzzyList::new(
                "Worktrees",
                GitWorktree::display_name,
                GitWorktree::search_text,
            ),
            git,
        };

        let _ = picker.refresh();
        picker
    }

    pub fn refresh(&mut self) -> Result<()> {
        if let Some(ref git) = self.git {
            let worktrees = git.list_worktrees()?;
            self.fuzzy_list.set_items(worktrees);
        }
        Ok(())
    }
}

impl Component for WorktreePicker {
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
                        // Delete worktree
                        if let Some(wt) = self.fuzzy_list.selected() {
                            if wt.is_main {
                                // Cannot delete main worktree
                                return Ok(None);
                            }
                            if wt.has_changes {
                                // Show warning about uncommitted changes
                                return Ok(Some(Action::ShowConfirm {
                                    title: "Delete Worktree".to_string(),
                                    message: format!(
                                        "Worktree '{}' has uncommitted changes. Delete anyway?",
                                        wt.branch
                                    ),
                                    callback: ConfirmCallback::DeleteWorktree(wt.path.clone()),
                                }));
                            }
                            return Ok(Some(Action::ShowConfirm {
                                title: "Delete Worktree".to_string(),
                                message: format!("Delete worktree '{}'?", wt.branch),
                                callback: ConfirmCallback::DeleteWorktree(wt.path.clone()),
                            }));
                        }
                        Ok(None)
                    }
                    'm' if self.fuzzy_list.query().is_empty() => {
                        // Merge to main
                        if let Some(wt) = self.fuzzy_list.selected() {
                            if wt.is_main {
                                return Ok(None);
                            }
                            return Ok(Some(Action::ShowConfirm {
                                title: "Merge to Main".to_string(),
                                message: format!(
                                    "Merge '{}' to main and delete worktree?",
                                    wt.branch
                                ),
                                callback: ConfirmCallback::MergeWorktree(wt.path.clone()),
                            }));
                        }
                        Ok(None)
                    }
                    'n' if self.fuzzy_list.query().is_empty() => {
                        // New worktree
                        Ok(Some(Action::ShowInput {
                            title: "New Worktree Branch".to_string(),
                            callback: InputCallback::CreateWorktree,
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
                if let Some(wt) = self.fuzzy_list.selected() {
                    Ok(Some(Action::SwitchWorktree(wt.path.clone())))
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
        "Enter:switch  n:new  d:delete  m:merge  Esc:back"
    }
}
