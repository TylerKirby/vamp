# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Note:** When adding or changing features, always update both this file and README.md to keep documentation in sync.

## Project Overview

Vamp is a terminal-native development environment for Claude Code. It creates a 2-panel tmux workspace: Claude Code main stage (left) + a Ratatui sidebar (right) that manages multiple agents across 4 tabs with a "Blue Note Sessions" jazz aesthetic.

## Commands

```bash
# Install
./install.sh

# Run vamp (starts tmux session)
vamp                    # Current directory
vamp ~/Projects/app     # Specific directory

# Session management
vamp list               # List active sessions
vamp attach <name>      # Attach to session
vamp kill <name>        # Kill session
vamp killall            # Kill all sessions

# Project initialization
vamp init               # Creates .git, beads, CLAUDE.md template

# Multi-agent mode
vamp agent add <type>   # Add agent (claude, codex, cursor)
vamp agent list         # List all agents and status
vamp agent kill <id>    # Kill an agent
vamp agent merge        # Merge agent branches to main

# Self-update
vamp update             # Update to latest GitHub release
vamp update --check     # Check for updates without installing

# Diagnostics
vamp doctor             # Check setup and show recommendations
```

## Testing

Run tests using bats-core:

```bash
# Run all tests
./tests/run_tests.sh

# Run specific category
./tests/run_tests.sh unit
./tests/run_tests.sh integration

# Run specific file
./tests/run_tests.sh tests/unit/args_test.bats
```

Test files are in `tests/unit/` and `tests/integration/`. CI runs automatically via GitHub Actions.

## Architecture

```
bin/vamp (bash)                          sidebar/ (Rust/Ratatui)
├── creates tmux session                 ├── runs in right tmux pane
├── agent lifecycle (create/kill)        ├── reads .vamp/state.json
├── worktree management                  ├── writes .vamp/commands.json
├── state file writer                    ├── renders 4 tabs
└── command watcher (bg loop)            └── keyboard controls
```

**bin/vamp** - Main entry point. Bash script that:
- Parses subcommands (list, attach, kill, init, agent, help)
- Creates tmux sessions with a 2-pane layout: Claude Code (left) + sidebar (right)
- Manages agent lifecycle: create/kill agents with git worktree isolation
- Writes `.vamp/state.json` for sidebar to read
- Runs background command watcher that processes sidebar commands

**sidebar/** - Rust/Ratatui sidebar application:
- 4 tabs: players (agents), charts (files), beads (issues), setlist (activity)
- Blue Note jazz aesthetic with warm color palette
- Reads `.vamp/state.json` for agent data (polls mtime every 500ms)
- Writes `.vamp/commands.json` for agent control
- Keyboard: Tab/Shift-Tab cycles tabs, 1-4 direct select, a/x/f/p/r agent controls

**Tmux Layout:**
```
+---------------------------+------------------+
|                           |   vamp-sidebar   |
|      Claude Code          |   (Ratatui)      |
|      (main agent)         |   4 tabs:        |
|                           |   players/charts |
|                           |   beads/setlist  |
+---------------------------+------------------+
Left: 75%                   Right: 25%
```
Additional agents get their own tmux windows, each with a worktree.

**Communication:**
- `bin/vamp` writes `.vamp/state.json` → sidebar reads it
- Sidebar writes `.vamp/commands.json` → bash command watcher reads it
- Both use atomic writes (temp file + rename) to prevent races

**lib/vamp-utils.sh** - Shell aliases and functions sourced in user's shell:
- Launcher shortcuts: `v`, `vp`, `va`, `vk`, `vl`, `vin`
- Agent shortcuts: `vag`, `vaa`, `val`, `vak`, `vam`
- Beads shortcuts: `bds`, `bdl`, `bdn`, `bdp`, `bdcp`, `bdd`
- Claude shortcuts: `ccr`, `ccc`, `ccs`, `cco`
- Workflow helpers: `standup`, `eod`

**install.sh** - Installer that:
- Installs dependencies via brew/apt/dnf/pacman
- Copies files to `~/.local/bin` and `~/.local/share/vamp`
- Builds and installs sidebar binary (requires Rust/cargo)
- Creates config at `~/.config/vamp/config`
- Adds PATH and source lines to shell rc file

## Configuration

User config lives at `~/.config/vamp/config`:
- `VAMP_CLAUDE_CMD` - claude command
- `VAMP_CLAUDE_FLAGS` - flags passed to all Claude instances (default: `--dangerously-skip-permissions`)
- `VAMP_AGENT_CLAUDE_CMD` / `VAMP_AGENT_CODEX_CMD` / `VAMP_AGENT_CURSOR_CMD` - per-type agent commands
- `VAMP_PROJECTS_DIR` - for project picker

By default, all Claude Code instances launched by vamp use `--dangerously-skip-permissions`. To disable this, set `VAMP_CLAUDE_FLAGS=""` in your config.

## Beads Integration

Vamp integrates with [beads](https://github.com/steveyegge/beads) for AI-native task tracking that maintains context across Claude Code sessions.

### Setup

```bash
# Global setup (installs Claude Code hooks)
vamp setup

# Per-project setup (during vamp init)
vamp init              # Initializes git, beads, git hooks, CLAUDE.md
```

The `vamp setup` command installs:
- **SessionStart hook** - runs `bd prime` when Claude Code starts
- **PreCompact hook** - runs `bd prime` before context compaction
- **Git hooks** - auto-sync beads with commits

### Auto-Approval

To avoid prompts for beads commands, add to `.claude/settings.local.json`:
```json
{
  "permissions": {
    "allow": ["Bash(bd:*)"]
  }
}
```

### Workflow

**Session Start:**
```bash
ss                     # Primes beads context, shows ready tasks, git status
# Or manually:
bd prime               # Load beads context into Claude
bd ready               # See available work
```

**During Session:**
```bash
bds                    # Show ready tasks
bdip                   # Show in-progress tasks
bdcp <id> <notes>      # Checkpoint progress
bdn "Task title"       # Create new task
bdd <id>               # Close task
```

**Session End:**
```bash
se                     # Syncs beads, shows status
# Or manually:
bd sync                # Sync beads with git
eod                    # End-of-day checkpoint prompt
```

### Shell Aliases

| Alias | Command | Description |
|-------|---------|-------------|
| `bds` | `bd ready` | Ready tasks |
| `bdl` | `bd list` | All tasks |
| `bda` | `bd list --all` | Including closed |
| `bdip` | `bd list --status in_progress` | In progress |
| `bdb` | `bd list --status blocked` | Blocked |
| `bdn` | `bd create ... -t task` | New task |
| `bdp` | `bd create ... --priority 0` | New P0 task |
| `bdcp` | `bd update + bd show` | Checkpoint |
| `bdd` | `bd close` | Close task |
| `bdpr` | `bd prime` | Prime context |
| `bdsy` | `bd sync` | Sync with git |
| `bdco` | `bd compact --stats` | Compact (memory decay) |
| `ss` | `session_start` | Start workflow |
| `se` | `session_end` | End workflow |

## Multi-Agent Mode

Multi-agent mode enables running multiple AI coding agents (Claude, Codex, Cursor) in parallel, each with git worktree isolation managed by the sidebar.

### Architecture

**Agent Types:**
- `claude` (BRASS) - Claude Code CLI
- `codex` (KEYS) - Codex CLI
- `cursor` (STRINGS) - Cursor CLI

**Git Worktrees:**
- Each agent gets its own directory: `.vamp-agents/<name>/`
- Each directory is a git worktree with its own branch: `agent/<name>`
- The main agent works directly in the project directory (no worktree)
- Changes are isolated until explicitly merged

**Sidebar Tabs:**
- **1:players** - Agent list grouped by type, controls (add/kill/pause/focus/restart)
- **2:charts** - File changes across all agent worktrees, conflict detection
- **3:beads** - Issue tracker (reads from beads via `bd list`)
- **4:setlist** - Activity log of agent events

### Workflow

**Adding agents:**
```bash
vamp agent add claude       # Add a Claude agent
vamp agent add codex        # Add a Codex agent
# Or press 'a' in the sidebar Players tab
```

**Monitoring:**
```bash
vamp agent list             # Show all agents
# Or use the sidebar tabs for real-time status
```

**Merging work:**
```bash
vamp agent merge            # Merge agent branches to main
vamp agent kill <id> --remove  # Kill and clean up agent
```

### Agent Shell Aliases

| Alias | Command | Description |
|-------|---------|-------------|
| `vag` | `vamp agent` | Agent commands |
| `vaa` | `vamp agent add claude` | Add Claude agent |
| `val` | `vamp agent list` | List agents |
| `vak` | `vamp agent kill` | Kill agent |
| `vam` | `vamp agent merge --all` | Merge all |

## Tmux Session Settings

Vamp configures each tmux session with:
- `mouse on` - enables mouse scrolling, pane selection, border dragging
- `focus-events on` - better iTerm2 integration
- Status bar showing `vamp v{VERSION} | HH:MM`

**iTerm2 Requirement:** For mouse support, enable "Mouse reporting" in iTerm2 → Preferences → Profiles → Terminal.

## Versioning

Version is defined in `bin/vamp` as `VAMP_VERSION`. Follow semantic versioning:
- **MAJOR** (x.0.0) - Breaking changes
- **MINOR** (0.x.0) - New features, backward compatible
- **PATCH** (0.0.x) - Bug fixes, backward compatible

Increment the version when adding features or fixes. The version displays in the tmux status bar.

## Dependencies

Required: tmux, Claude Code CLI, Rust/cargo (for building sidebar)
Recommended: jq, fzf, beads, bv (beads_viewer)
