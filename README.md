# vamp 🎹

> Like a jazz vamp - keeps the rhythm while you improvise with Claude

Terminal-native development environment for Claude Code with beads integration for persistent context management.

## Features

- **One command** launches full dev environment
- **Mouse support** for scrolling, pane selection, and resizing
- **File browser** (yazi) for project navigation
- **System monitor** for resource tracking
- **Claude monitor** window showing usage and beads status
- **Beads integration** for persistent task/context management
- **Git window** with lazygit
- **Session persistence** - detach and reattach anytime

## Layout

```
+------------------------+--------------+
|                        |              |
|     Claude Code        |  File Viewer |
|     (main work)        |   (yazi)     |
|                        +--------------+
+------------------------+              |
|   Shell / Beads        |   Monitor    |
|                        |   (htop)     |
+------------------------+--------------+

Window 0: main    Window 1: git    Window 2: claude monitor
```

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
brew install yazi htop lazygit fzf jq
brew tap steveyegge/beads && brew install beads
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
| `Ctrl-b` + `1` | Git window |
| `Ctrl-b` + `2` | Claude monitor |
| Mouse scroll | Scroll pane content |
| Mouse click | Select pane |
| Mouse drag border | Resize pane |

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
| `bdn <title>` | Create new task |
| `bdp <title>` | Create P0 task |
| `bdcp <id> <notes>` | Checkpoint task |
| `bdd <id>` | Close task |

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

### Starting a Session

```bash
cd ~/Projects/my-app
vamp
```

### Morning Routine

```bash
vamp attach my-app    # or just: va my-app
standup               # see what's ready
```

### During Work

```bash
# In Claude Code, it knows about beads:
# "What should I work on?" -> checks bd ready
# "I finished the auth flow" -> can update beads

# Checkpoint before context runs out:
bdcp auth-123 "Implemented JWT refresh, need to add tests"
```

### End of Day

```bash
eod                   # checkpoint in-progress work
# Ctrl-b d to detach
```

### Resume Later

```bash
va my-app
# Claude: "What was I working on?"
# It checks beads and knows the full context
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
