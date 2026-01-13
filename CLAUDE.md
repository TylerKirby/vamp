# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **Note:** When adding or changing features, always update both this file and README.md to keep documentation in sync.

## Project Overview

Vamp is a terminal-native development environment for Claude Code. It creates a tmux-based workspace with integrated file browsing (yazi), system monitoring (htop), beads issue viewer, and optional beads task tracking.

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
```

No build or test commands - this is a bash-only project.

## Architecture

**bin/vamp** - Main entry point. A single bash script (~400 lines) that:
- Parses subcommands (list, attach, kill, init, help)
- Creates tmux sessions with a 5-pane layout: Claude Code, shell, file viewer, htop, usage checker
- The shell pane runs `bd ready` on startup if beads is detected, then is available for any commands
- Adds one additional tmux window: beads viewer
- Includes a usage checker pane (Claude instance) for running `/usage` to check session limits

**Tmux Windows:**
- Window 0 "main": 5-pane layout with Claude Code, shell, yazi, htop, and usage checker
- Window 1 "beads": beads issue viewer (`bv` if installed, simple dashboard fallback)

**Main Window Panes:**
- Pane 0: Claude Code (top left, 75%) - main work area
- Pane 1: Shell (bottom left, 25%) - runs `bd ready` on startup if beads detected
- Pane 2: File viewer/yazi (top right)
- Pane 3: htop (middle right)
- Pane 4: Usage checker (bottom right) - Claude instance for running `/usage` to check session limits

**lib/vamp-utils.sh** - Shell aliases and functions sourced in user's shell:
- Launcher shortcuts: `v`, `vp`, `va`, `vk`, `vl`, `vi`
- Beads shortcuts: `bds`, `bdl`, `bdn`, `bdp`, `bdcp`, `bdd`
- Claude shortcuts: `ccr`, `ccc`, `ccs`, `cco`
- Workflow helpers: `standup`, `eod`

**install.sh** - Installer that:
- Installs dependencies via brew/apt/dnf/pacman
- Copies files to `~/.local/bin` and `~/.local/share/vamp`
- Creates config at `~/.config/vamp/config`
- Adds PATH and source lines to shell rc file

## Configuration

User config lives at `~/.config/vamp/config`:
- `VAMP_FILE_VIEWER` - yazi, lf, ranger, nnn
- `VAMP_MONITOR` - htop, btop, glances
- `VAMP_CLAUDE_CMD` - claude command
- `VAMP_PROJECTS_DIR` - for project picker

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

### Usage Checker Pane

The bottom-right pane contains a Claude instance dedicated to checking session limits. Run `/usage` in this pane to see:

- Current session context usage (how close to compaction)
- Token counts and model info
- Cost estimates

This helps you know when you're approaching session limits before context compaction occurs.

## Tmux Session Settings

Vamp configures each tmux session with:
- `mouse on` - enables mouse scrolling, pane selection, border dragging
- `focus-events on` - better iTerm2 integration
- Status bar showing `vamp v{VERSION} | HH:MM`

**iTerm2 Requirement:** For mouse support, enable "Mouse reporting" in iTerm2 → Preferences → Profiles → Terminal.

## Yazi File Viewer

Yazi config at `~/.config/yazi/yazi.toml`:
- Git integration enabled - shows file status indicators (M=modified, A=added, D=deleted, etc.)
- Column ratio `[0, 2, 3]` - hides parent dir to maximize space in split pane
- Line wrapping enabled for code preview

**File openers:**
- `Enter` or `o` - Opens in bat (syntax-highlighted viewer)
- `Shift+O` - Menu to choose between bat, Cursor, or micro
- `q` in bat/micro - Returns to yazi

## Versioning

Version is defined in `bin/vamp` as `VAMP_VERSION`. Follow semantic versioning:
- **MAJOR** (x.0.0) - Breaking changes
- **MINOR** (0.x.0) - New features, backward compatible
- **PATCH** (0.0.x) - Bug fixes, backward compatible

Increment the version when adding features or fixes. The version displays in the tmux status bar.

## Dependencies

Required: tmux, Claude Code CLI
Recommended: yazi, htop, fzf, jq, beads, bv (beads_viewer), bat
