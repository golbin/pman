use std::fs;
use std::path::{Path, PathBuf};

use ratatui::{layout::Rect, Frame};

use crate::actions::Action;
use crate::components::{Component, FuzzyList};
use crate::error::Result;

#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub name: String,
}

impl FileEntry {
    pub fn display_name(&self) -> String {
        if self.is_dir {
            format!("📁 {}/", self.name)
        } else {
            format!("   {}", self.name)
        }
    }

    pub fn search_text(&self) -> String {
        self.name.clone()
    }
}

pub struct FilePicker {
    fuzzy_list: FuzzyList<FileEntry>,
    current_dir: PathBuf,
}

impl FilePicker {
    pub fn new(start_path: &Path) -> Self {
        let current_dir = if start_path.is_dir() {
            start_path.to_path_buf()
        } else {
            start_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };

        let mut picker = Self {
            fuzzy_list: FuzzyList::new(
                "Files",
                FileEntry::display_name,
                FileEntry::search_text,
            ),
            current_dir,
        };

        let _ = picker.refresh();
        picker
    }

    pub fn refresh(&mut self) -> Result<()> {
        let mut entries: Vec<FileEntry> = Vec::new();

        // Add parent directory entry
        if let Some(parent) = self.current_dir.parent() {
            entries.push(FileEntry {
                path: parent.to_path_buf(),
                is_dir: true,
                name: "..".to_string(),
            });
        }

        // Read directory contents
        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut dir_entries: Vec<FileEntry> = read_dir
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Skip hidden files by default
                    if name.starts_with('.') {
                        return None;
                    }

                    Some(FileEntry { path, is_dir, name })
                })
                .collect();

            // Sort: directories first, then files, alphabetically
            dir_entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            entries.extend(dir_entries);
        }

        self.fuzzy_list.set_items(entries);
        Ok(())
    }

    pub fn current_dir(&self) -> &Path {
        &self.current_dir
    }

    fn navigate_to(&mut self, path: PathBuf) -> Result<()> {
        self.current_dir = path;
        self.fuzzy_list.clear_query();
        self.refresh()
    }
}

impl Component for FilePicker {
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
                if let Some(entry) = self.fuzzy_list.selected() {
                    if entry.is_dir {
                        let path = entry.path.clone();
                        self.navigate_to(path)?;
                        Ok(Some(Action::Render))
                    } else {
                        Ok(Some(Action::OpenFile(entry.path.clone())))
                    }
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
        "Enter:open/navigate  Esc:back"
    }
}
