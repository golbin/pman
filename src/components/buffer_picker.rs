use std::path::PathBuf;

use ratatui::{layout::Rect, Frame};

use crate::actions::Action;
use crate::components::{Component, FuzzyList};
use crate::error::Result;
use crate::integrations::NvimIntegration;
use crate::models::NvimBuffer;

#[derive(Clone)]
pub struct BufferEntry {
    pub socket: PathBuf,
    pub buffer: NvimBuffer,
}

impl BufferEntry {
    pub fn display_name(&self) -> String {
        self.buffer.display_name()
    }

    pub fn search_text(&self) -> String {
        self.buffer.search_text()
    }
}

pub struct BufferPicker {
    fuzzy_list: FuzzyList<BufferEntry>,
}

impl BufferPicker {
    pub fn new() -> Self {
        let mut picker = Self {
            fuzzy_list: FuzzyList::new(
                "Nvim Buffers",
                BufferEntry::display_name,
                BufferEntry::search_text,
            ),
        };

        let _ = picker.refresh();
        picker
    }

    pub fn refresh(&mut self) -> Result<()> {
        let buffers = NvimIntegration::list_buffers().unwrap_or_default();

        let entries: Vec<BufferEntry> = buffers
            .into_iter()
            .map(|(socket, buffer)| BufferEntry { socket, buffer })
            .collect();

        self.fuzzy_list.set_items(entries);
        Ok(())
    }
}

impl Default for BufferPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for BufferPicker {
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
                    Ok(Some(Action::OpenBuffer {
                        socket: entry.socket.clone(),
                        bufnr: entry.buffer.bufnr,
                    }))
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
        "Enter:open  Esc:back"
    }
}
