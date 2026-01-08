# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vamp is a terminal-native development environment for Claude Code. It creates a tmux-based workspace with integrated file browsing (yazi), system monitoring (htop), git interface (lazygit), and optional beads task tracking.

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
- Creates tmux sessions with a 4-pane layout: Claude Code (main), shell, file viewer, system monitor
- Adds two additional tmux windows: git (lazygit) and Claude monitor
- Generates a monitor script on-the-fly that displays Claude session stats and beads status

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

## Tmux Session Settings

Vamp configures each tmux session with:
- `mouse on` - enables mouse scrolling, pane selection, border dragging
- `focus-events on` - better iTerm2 integration
- Status bar showing `vamp v{VERSION} | HH:MM`

**iTerm2 Requirement:** For mouse support, enable "Mouse reporting" in iTerm2 → Preferences → Profiles → Terminal.

## Yazi File Viewer

Yazi config at `~/.config/yazi/yazi.toml`:
- Column ratio `[0, 2, 3]` - hides parent dir to maximize space in split pane
- Line wrapping enabled for code preview

**File openers:**
- `Enter` or `o` - Opens in bat (syntax-highlighted viewer)
- `Shift+O` - Menu to choose between bat or Cursor
- `q` in bat - Returns to yazi

## Dependencies

Required: tmux, Claude Code CLI
Recommended: yazi, htop, lazygit, fzf, jq, beads, bat
