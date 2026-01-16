use std::path::PathBuf;
use crate::models::PaletteCommand;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    GoBack,
    Render,

    // Navigation
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,

    // Input
    Character(char),
    Backspace,
    Enter,
    Escape,

    // Session actions
    SwitchSession(String),
    CreateSession(String, Option<PathBuf>),
    KillSession(String),

    // File actions
    OpenFile(PathBuf),
    OpenBuffer { socket: PathBuf, bufnr: i64 },

    // Worktree actions
    SwitchWorktree(PathBuf),
    CreateWorktree(String),
    DeleteWorktree(PathBuf),
    MergeWorktree(PathBuf),

    // Command palette
    ExecuteCommand(PaletteCommand),

    // Dialog actions
    ShowInput { title: String, callback: InputCallback },
    ShowConfirm { title: String, message: String, callback: ConfirmCallback },
    CloseDialog,

    // View switching
    ShowSessionPicker,
    ShowCommandPalette,
    ShowFilePicker,
    ShowWorktreePicker,
    ShowBufferPicker,

    // Git
    ShowGitDiff,
}

#[derive(Debug, Clone)]
pub enum InputCallback {
    CreateSession,
    CreateWorktree,
}

#[derive(Debug, Clone)]
pub enum ConfirmCallback {
    DeleteWorktree(PathBuf),
    MergeWorktree(PathBuf),
    KillSession(String),
}
