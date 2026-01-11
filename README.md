# vamp 🎹

> Like a jazz vamp - keeps the rhythm while you improvise with Claude

Terminal-native development environment for Claude Code with beads integration for persistent context management.

## Features

- **One command** launches full dev environment
- **Mouse support** for scrolling, pane selection, and resizing
- **File browser** (yazi) for project navigation
- **System monitor** (htop) for resource tracking
- **Usage checker** pane for monitoring Claude session limits via `/usage`
- **Beads integration** for persistent task/context management
- **Beads window** with `bv` (beads_viewer) - Kanban, graph, insights, live reload
- **Session persistence** - detach and reattach anytime

## Layout

```
+------------------------+--------------+
|                        |              |
|     Claude Code        |  File Viewer |
|       (75%)            |   (yazi)     |
|                        +-------+------+
+------------------------+  htop | usage|
|   Shell (25%)          |       |checker
+------------------------+-------+------+

Window 0: main          Window 1: beads (bv viewer)
```

- **Shell pane** runs `bd ready` on startup if beads is detected, then available for any commands
- **Usage checker** is a Claude instance for running `/usage` to check session limits

## Install

```bash
# Clone
git clone https://github.com/yourusername/vamp.git
cd vamp

# Install
./install.sh

# Restart shell
source ~/.zshrc
```

### Dependencies

**Required:**
- tmux
- Claude Code CLI

**Recommended (installed automatically on macOS):**
```bash
brew install yazi htop fzf jq
brew tap steveyegge/beads && brew install beads

# beads_viewer (bv) - rich TUI for beads
curl -fsSL https://raw.githubusercontent.com/Dicklesworthstone/beads_viewer/main/install.sh | bash
```

## Usage

```bash
# Start in current directory
vamp

# Start in specific directory  
vamp ~/Projects/my-app

# Project picker (with fzf)
vp my-app

# List active sessions
vamp list

# Attach to existing session
vamp attach my-app

# Kill a session
vamp kill my-app

# Initialize new project
cd ~/Projects/new-thing
vamp init
```

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl-b` + arrows | Navigate panes |
| `Ctrl-b` + `z` | Zoom pane (toggle) |
| `Ctrl-b` + `d` | Detach session |
| `Ctrl-b` + `[` | Scroll mode |
| `Ctrl-b` + `0` | Main window |
| `Ctrl-b` + `1` | Beads window (`bv` viewer) |
| Mouse scroll | Scroll pane content |
| Mouse click | Select pane |
| Mouse drag border | Resize pane |

### Beads Viewer (`bv`) Keys

| Key | Action |
|-----|--------|
| `j` / `k` | Move down / up |
| `o` / `c` / `r` / `a` | Filter: Open / Closed / Ready / All |
| `/` | Fuzzy search |
| `b` | Kanban board view |
| `i` | Insights dashboard |
| `g` | Dependency graph |
| `h` | History & git correlation |
| `?` | Help overlay |
| `q` | Quit |

## Shell Shortcuts

After install, these shortcuts are available:

### Launcher
| Command | Action |
|---------|--------|
| `v` | Start vamp (current dir) |
| `vp [name]` | Project picker with fzf |
| `va <name>` | Attach to session |
| `vk <name>` | Kill session |
| `vl` | List sessions |
| `vi` | Initialize project |

### Beads
| Command | Action |
|---------|--------|
| `bds` | Show ready tasks |
| `bdl` | List all tasks |
| `bdip` | In-progress tasks |
| `bdb` | Blocked tasks |
| `bdn <title>` | Create new task |
| `bdp <title>` | Create P0 task |
| `bdcp <id> <notes>` | Checkpoint task |
| `bdd <id>` | Close task |
| `bdpr` | Prime context for Claude |
| `bdsy` | Sync beads with git |
| `bdco` | Compact (memory decay) |

### Claude Code
| Command | Action |
|---------|--------|
| `ccr` | Resume last session |
| `ccc` | Continue last session |
| `ccs` | Use Sonnet model |
| `cco` | Use Opus model |

### Workflow
| Command | Action |
|---------|--------|
| `ss` | Session start (prime + status) |
| `se` | Session end (sync + status) |
| `standup` | Morning status check |
| `eod` | End of day checkpoint |

## Configuration

Edit `~/.config/vamp/config`:

```bash
# File viewer: yazi, lf, ranger, nnn
export VAMP_FILE_VIEWER="yazi"

# System monitor: htop, btop, glances
export VAMP_MONITOR="htop"

# Claude command
export VAMP_CLAUDE_CMD="claude"

# Projects directory
export VAMP_PROJECTS_DIR="$HOME/Projects"
```

## Workflow

### First-Time Setup

```bash
# Install beads hooks for Claude Code (run once globally)
vamp setup

# Restart Claude Code for hooks to take effect
```

### Starting a Session

```bash
cd ~/Projects/my-app
vamp                  # Launches tmux environment
ss                    # Prime beads context, show ready tasks
```

The `ss` command (session start) runs `bd prime` which loads all your beads context into Claude's memory. Claude now knows about all your tasks, dependencies, and progress.

### During Work

```bash
# Check what's available
bds                   # Ready tasks (no blockers)
bdip                  # In-progress tasks

# Claim a task
bd update <id> --status=in_progress

# ... work with Claude Code ...

# Checkpoint progress (before context compaction)
bdcp <id> "Implemented auth, need tests"

# Close completed work
bdd <id>
```

**Claude Code Integration:**
- Ask Claude "What should I work on?" - it checks `bd ready`
- Tell Claude "I finished the auth flow" - it can run `bd close`
- Hooks auto-run `bd prime` on session start and before compaction

### End of Session

```bash
se                    # Sync beads with git, show status
# Ctrl-b d to detach (session keeps running)
```

The `se` command (session end) runs `bd sync` to push your beads changes to the git remote.

### Resuming Later

```bash
va my-app             # Reattach to tmux session
ss                    # Prime context again
# Claude instantly knows where you left off
```

### Morning Routine

```bash
standup               # Git status + ready tasks
```

### End of Day

```bash
eod                   # Checkpoint prompt + status
```

## Project Structure

```
~/.local/
├── bin/
│   └── vamp              # Main script
└── share/
    └── vamp/
        └── vamp-utils.sh # Shell helpers

~/.config/vamp/
└── config                # Configuration
```

## Why "vamp"?

In jazz, a **vamp** is a repeating musical figure that accompanies soloists during improvisation. It provides a stable foundation while allowing creative freedom.

That's what this environment does - it keeps the rhythm (file watching, monitoring, task tracking) while you improvise solutions with Claude.

## License

MIT
