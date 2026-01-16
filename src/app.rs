use std::path::PathBuf;

use ratatui::layout::{Constraint, Direction, Layout};

use crate::actions::{Action, ConfirmCallback, InputCallback};
use crate::components::{
    CommandPalette, Component, ConfirmDialog, FilePicker, HelpBar, InputDialog, SessionPicker,
    WorktreePicker,
};
use crate::error::Result;
use crate::integrations::{GitClient, NvimIntegration, TmuxClient};
use crate::models::PaletteCommand;
use crate::tui::{key_to_action, Event, EventHandler, Tui};

pub enum View {
    SessionPicker,
    CommandPalette,
    FilePicker,
    WorktreePicker,
}

pub enum Dialog {
    None,
    Input(InputDialog),
    Confirm(ConfirmDialog),
}

pub struct App {
    tui: Tui,
    event_handler: EventHandler,
    view: View,
    dialog: Dialog,
    running: bool,
    current_path: PathBuf,

    // Components
    session_picker: SessionPicker,
    command_palette: Option<CommandPalette>,
    file_picker: Option<FilePicker>,
    worktree_picker: Option<WorktreePicker>,

    // Integrations
    tmux: TmuxClient,
}

impl App {
    pub fn new(initial_view: View) -> Result<Self> {
        let tmux = TmuxClient::new();
        let current_path = tmux.current_path().unwrap_or_else(|_| PathBuf::from("."));

        let mut session_picker = SessionPicker::new();
        session_picker.refresh()?;

        // Initialize component based on initial view
        let command_palette = match &initial_view {
            View::CommandPalette => Some(CommandPalette::new(&current_path)),
            _ => None,
        };

        Ok(Self {
            tui: Tui::new()?,
            event_handler: EventHandler::new(100),
            view: initial_view,
            dialog: Dialog::None,
            running: true,
            current_path,
            session_picker,
            command_palette,
            file_picker: None,
            worktree_picker: None,
            tmux,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.tui.enter()?;

        while self.running {
            self.render()?;

            match self.event_handler.next()? {
                Event::Key(key) => {
                    if let Some(action) = key_to_action(key) {
                        self.handle_action(action)?;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal will handle resize automatically
                }
                Event::Tick => {
                    // Could be used for async updates
                }
            }
        }

        self.tui.exit()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let help_text = self.current_help_text();

        self.tui.terminal().draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(frame.area());

            // Render main component
            match &mut self.view {
                View::SessionPicker => {
                    self.session_picker.render(frame, chunks[0]);
                }
                View::CommandPalette => {
                    if let Some(ref mut palette) = self.command_palette {
                        palette.render(frame, chunks[0]);
                    }
                }
                View::FilePicker => {
                    if let Some(ref mut picker) = self.file_picker {
                        picker.render(frame, chunks[0]);
                    }
                }
                View::WorktreePicker => {
                    if let Some(ref mut picker) = self.worktree_picker {
                        picker.render(frame, chunks[0]);
                    }
                }
            }

            // Render help bar
            HelpBar::render(frame, chunks[1], help_text);

            // Render dialog if active
            match &self.dialog {
                Dialog::None => {}
                Dialog::Input(dialog) => {
                    dialog.render(frame, frame.area());
                }
                Dialog::Confirm(dialog) => {
                    dialog.render(frame, frame.area());
                }
            }
        })?;

        Ok(())
    }

    fn current_help_text(&self) -> &'static str {
        match &self.dialog {
            Dialog::Input(_) => "Enter:confirm  Esc:cancel",
            Dialog::Confirm(_) => "Y:yes  N:no  ←→:select  Esc:cancel",
            Dialog::None => match &self.view {
                View::SessionPicker => self.session_picker.help_text(),
                View::CommandPalette => self
                    .command_palette
                    .as_ref()
                    .map(|p| p.help_text())
                    .unwrap_or(""),
                View::FilePicker => self
                    .file_picker
                    .as_ref()
                    .map(|p| p.help_text())
                    .unwrap_or(""),
                View::WorktreePicker => self
                    .worktree_picker
                    .as_ref()
                    .map(|p| p.help_text())
                    .unwrap_or(""),
            },
        }
    }

    fn handle_action(&mut self, action: Action) -> Result<()> {
        // Handle dialog first if active
        if let Dialog::Input(ref mut dialog) = self.dialog {
            if let Some(result_action) = dialog.handle_action(&action)? {
                return self.handle_action(result_action);
            }
            return Ok(());
        }

        if let Dialog::Confirm(ref mut dialog) = self.dialog {
            if let Some(result_action) = dialog.handle_action(&action)? {
                return self.handle_action(result_action);
            }
            return Ok(());
        }

        // Handle global actions
        match action {
            Action::Quit => {
                self.running = false;
                return Ok(());
            }
            Action::CloseDialog => {
                self.dialog = Dialog::None;
                return Ok(());
            }
            Action::ShowInput { title, callback } => {
                self.dialog = Dialog::Input(InputDialog::new(title, callback));
                return Ok(());
            }
            Action::ShowConfirm {
                title,
                message,
                callback,
            } => {
                self.dialog = Dialog::Confirm(ConfirmDialog::new(title, message, callback));
                return Ok(());
            }
            Action::SwitchSession(name) => {
                self.tui.exit()?;
                self.tmux.switch_session(&name)?;
                self.running = false;
                return Ok(());
            }
            Action::CreateSession(name, path) => {
                self.tmux.create_session(&name, path.as_ref())?;
                self.tmux.switch_session(&name)?;
                self.dialog = Dialog::None;
                self.running = false;
                return Ok(());
            }
            Action::KillSession(name) => {
                self.tmux.kill_session(&name)?;
                self.dialog = Dialog::None;
                self.session_picker.refresh()?;
                return Ok(());
            }
            Action::OpenFile(path) => {
                self.tui.exit()?;
                let nvim = NvimIntegration::new(TmuxClient::new());
                nvim.open_file(&path)?;
                self.running = false;
                return Ok(());
            }
            Action::SwitchWorktree(path) => {
                // Create or switch to session for this worktree
                let session_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "worktree".to_string());

                // Check if session exists
                let sessions = self.tmux.list_sessions()?;
                let session_exists = sessions.iter().any(|s| s.name == session_name);

                if !session_exists {
                    self.tmux.create_session(&session_name, Some(&path))?;
                }

                self.tui.exit()?;
                self.tmux.switch_session(&session_name)?;
                self.running = false;
                return Ok(());
            }
            Action::CreateWorktree(branch_name) => {
                if let Some(ref git) = GitClient::new(&self.current_path).ok() {
                    let worktree_path = git.create_worktree(&branch_name)?;
                    self.dialog = Dialog::None;

                    // Switch to the new worktree session
                    return self.handle_action(Action::SwitchWorktree(worktree_path));
                }
                self.dialog = Dialog::None;
                return Ok(());
            }
            Action::DeleteWorktree(path) => {
                if let Some(ref git) = GitClient::new(&self.current_path).ok() {
                    git.delete_worktree(&path)?;
                    if let Some(ref mut picker) = self.worktree_picker {
                        picker.refresh()?;
                    }
                }
                self.dialog = Dialog::None;
                return Ok(());
            }
            Action::MergeWorktree(path) => {
                if let Some(ref git) = GitClient::new(&self.current_path).ok() {
                    // Get branch name from worktree
                    let worktrees = git.list_worktrees()?;
                    if let Some(wt) = worktrees.iter().find(|w| w.path == path) {
                        git.merge_to_main(&path, &wt.branch)?;
                    }
                    if let Some(ref mut picker) = self.worktree_picker {
                        picker.refresh()?;
                    }
                }
                self.dialog = Dialog::None;
                return Ok(());
            }
            Action::ExecuteCommand(cmd) => {
                return self.execute_command(cmd);
            }
            Action::ShowSessionPicker => {
                self.view = View::SessionPicker;
                self.session_picker.refresh()?;
                return Ok(());
            }
            Action::ShowCommandPalette => {
                self.view = View::CommandPalette;
                if self.command_palette.is_none() {
                    self.command_palette = Some(CommandPalette::new(&self.current_path));
                }
                return Ok(());
            }
            Action::ShowFilePicker => {
                self.view = View::FilePicker;
                if self.file_picker.is_none() {
                    self.file_picker = Some(FilePicker::new(&self.current_path));
                }
                return Ok(());
            }
            Action::ShowWorktreePicker => {
                self.view = View::WorktreePicker;
                if self.worktree_picker.is_none() {
                    self.worktree_picker = Some(WorktreePicker::new(&self.current_path));
                }
                return Ok(());
            }
            Action::ShowGitDiff => {
                self.tui.exit()?;
                self.tmux
                    .popup_command("git diff HEAD | delta", "90%", "90%")?;
                self.tui.enter()?;
                return Ok(());
            }
            Action::Render => {
                // Just render on next loop
                return Ok(());
            }
            _ => {}
        }

        // Delegate to current view
        let result_action = match &mut self.view {
            View::SessionPicker => self.session_picker.handle_action(&action)?,
            View::CommandPalette => self
                .command_palette
                .as_mut()
                .and_then(|p| p.handle_action(&action).ok())
                .flatten(),
            View::FilePicker => self
                .file_picker
                .as_mut()
                .and_then(|p| p.handle_action(&action).ok())
                .flatten(),
            View::WorktreePicker => self
                .worktree_picker
                .as_mut()
                .and_then(|p| p.handle_action(&action).ok())
                .flatten(),
        };

        if let Some(result_action) = result_action {
            self.handle_action(result_action)?;
        }

        Ok(())
    }

    fn execute_command(&mut self, cmd: PaletteCommand) -> Result<()> {
        match cmd {
            PaletteCommand::OpenFile => {
                self.view = View::FilePicker;
                if self.file_picker.is_none() {
                    self.file_picker = Some(FilePicker::new(&self.current_path));
                }
            }
            PaletteCommand::NewSession => {
                self.dialog = Dialog::Input(InputDialog::new(
                    "New Session Name",
                    InputCallback::CreateSession,
                ));
            }
            PaletteCommand::KillSession => {
                let current = self.tmux.current_session()?;
                self.dialog = Dialog::Confirm(ConfirmDialog::new(
                    "Kill Session",
                    format!("Kill current session '{}'?", current),
                    ConfirmCallback::KillSession(current),
                ));
            }
            PaletteCommand::ListWorktrees => {
                self.view = View::WorktreePicker;
                if self.worktree_picker.is_none() {
                    self.worktree_picker = Some(WorktreePicker::new(&self.current_path));
                }
            }
            PaletteCommand::CreateWorktree => {
                self.dialog = Dialog::Input(InputDialog::new(
                    "New Worktree Branch",
                    InputCallback::CreateWorktree,
                ));
            }
            PaletteCommand::GitStatus => {
                return self.handle_action(Action::ShowGitDiff);
            }
        }
        Ok(())
    }
}
