# pman

**Run multiple AI agents in parallel. Switch between them in 2 keystrokes.**

Claude Code, Codex, Aider... AI agents are powerful, but slow. Why wait for one when you can run several? pman lets you manage multiple tmux sessions and git worktrees with fuzzy search—perfect for running AI agents in parallel without conflicts.

```
┌ Sessions ─────────────────────────────┐
│ > _                                   │
└───────────────────────────────────────┘
┌───────────────────────────────────────┐
│ ▶ ● claude-auth (myproject)           │
│   ○ codex-tests (myproject)           │
│   ○ aider-refactor (myproject)        │
│   ○ manual-review (myproject)         │
└───────────────────────────────────────┘
```

## Why pman?

- **Parallel Agents** - Run multiple AI coding agents, each in its own tmux session
- **Isolated Worktrees** - Each agent works on a separate git branch, no conflicts
- **Instant Switch** - Fuzzy search and jump between sessions in milliseconds

## Installation

```bash
brew install golbin/tap/pman
```

## Quick Start

```bash
# 1. Install keybindings
pman install && tmux source-file ~/.tmux.conf

# 2. Inside tmux, press Prefix+s to open session picker
#    (Prefix is usually Ctrl+b)
```

## Usage Scenarios

### Scenario 1: Multiple Agents on Separate Features

Give each AI agent its own worktree to avoid conflicts:

```bash
# Create worktrees (Prefix+p → List Worktrees → n)
# Each worktree = separate branch

┌ Worktrees ────────────────────────────┐
│ > _                                   │
└───────────────────────────────────────┘
┌───────────────────────────────────────┐
│ ▶ feature/auth (a1b2c3d)              │
│   feature/payment* (e4f5g6h)          │
│   feature/notifications (i9j0k1l)     │
│   main (m2n3o4p) [main]               │
└───────────────────────────────────────┘

# Start agents in separate sessions:
# Session 1: Claude Code → feature/auth
# Session 2: Codex → feature/payment
# Session 3: Aider → feature/notifications

# Use Prefix+s to monitor each agent's progress
# Merge completed branches with 'm' key
```

### Scenario 2: Review While Agents Work

Work in parallel with your AI agents:

```bash
┌ Sessions ─────────────────────────────┐
│ > rev                                 │
└───────────────────────────────────────┘
┌───────────────────────────────────────┐
│ ▶ ○ review (myproject)                │
└───────────────────────────────────────┘

# Session: agent-impl    → AI implementing feature (attached ●)
# Session: review        → You reviewing code
# Session: hotfix        → You fixing urgent bugs

# Agents don't block you. You don't block agents.
```

## Keybindings

### Tmux (after `pman install`)

| Key | Action |
|-----|--------|
| `Prefix + s` | Session Picker |
| `Prefix + p` | Command Palette |

### Session Picker

| Key | Action |
|-----|--------|
| Type | Fuzzy search |
| `Enter` | Switch to session |
| `n` | New session |
| `d` | Delete session |
| `Esc` | Close |

### Worktree Picker

| Key | Action |
|-----|--------|
| Type | Fuzzy search |
| `Enter` | Switch to worktree |
| `n` | New worktree |
| `d` | Delete worktree |
| `m` | Merge to main |
| `Esc` | Close |

### Navigation (All Views)

| Key | Action |
|-----|--------|
| `Ctrl+k` / `↑` | Move up |
| `Ctrl+j` / `↓` | Move down |

## Prerequisites

```bash
brew install tmux neovim git-delta
```

## Uninstall

```bash
pman uninstall && tmux source-file ~/.tmux.conf
brew uninstall pman
```

## License

MIT
