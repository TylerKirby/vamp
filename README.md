# vamp 🎹

> Like a jazz vamp - keeps the rhythm while you improvise with Claude

Terminal-native development environment for Claude Code with a Ratatui sidebar for multi-agent orchestration and beads integration for persistent context management.

## Features

- **One command** launches full dev environment
- **Multi-agent mode** - run Claude, Codex, and Cursor agents in parallel with git worktree isolation
- **Ratatui sidebar** - 6-tab TUI (players, charts, beads, setlist, files, metrics) with Blue Note jazz aesthetic
- **Mouse support** for scrolling, pane selection, and resizing
- **Beads integration** for persistent task/context management
- **Session persistence** - detach and reattach anytime

## Layout

```
+---------------------------+------------------+
|                           |   vamp-sidebar   |
|      Claude Code          |   (Ratatui)      |
|      (main agent)         |   6 tabs:        |
|         75%               |   players/charts |
|                           |   beads/setlist  |
|                           |   files/metrics  |
+---------------------------+------------------+
Left: 75%                   Right: 25%
```

- **Left pane** - Claude Code main agent, working directly in the project directory
- **Right pane** - Ratatui sidebar with real-time agent status, file changes, issues, and activity log
- **Additional agents** get their own tmux windows, each with a git worktree

## Install

```bash
# Clone
git clone https://github.com/yourusername/vamp.git
cd vamp

# Install (builds sidebar binary with cargo)
./install.sh

# Restart shell
source ~/.zshrc
```

### Dependencies

**Required:**
- tmux
- Claude Code CLI
- Rust/cargo (for building the sidebar)

**Recommended:**
```bash
brew install jq fzf
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

# Check setup health
vamp doctor
```

## Multi-Agent Mode

Run multiple AI coding agents in parallel, each in its own isolated git worktree. The sidebar manages agent lifecycle and provides real-time status.

### Agent Types

| Type | Group | Command |
|------|-------|---------|
| `claude` | BRASS | Claude Code CLI |
| `codex` | KEYS | Codex CLI |
| `cursor` | STRINGS | Cursor CLI |

### Agent Commands

```bash
# Add agents
vamp agent add claude           # Add a Claude agent
vamp agent add codex            # Add a Codex agent
vamp agent add cursor           # Add a Cursor agent

# Monitor
vamp agent list                 # Show all agents and status

# Control
vamp agent kill <name>          # Stop an agent
vamp agent kill <name> --remove # Stop and remove worktree

# Merge
vamp agent merge                # Merge agent branches to main
```

Or use the sidebar: press `A` to add, `X` to kill, `f` to focus, `p` to pause, `r` to restart, `R` to rename.

### How It Works

1. **Git Worktrees** - Each agent gets its own directory (`.vamp-agents/<name>/`) with a dedicated branch (`agent/<name>`)
2. **Beads Coordination** - Agents use beads to claim and track work, avoiding conflicts
3. **Isolated Changes** - Each agent's changes stay on its branch until merged
4. **Sidebar Monitoring** - Real-time view of all agents, file changes, and activity

### Sidebar Tabs

| Tab | Key | Content |
|-----|-----|---------|
| **players** | `1` | Agent list grouped by type, status, controls |
| **charts** | `2` | Git branches, remotes, worktrees, file changes |
| **beads** | `3` | Issue tracker (reads from `bd list`) |
| **setlist** | `4` | Activity log of agent events |
| **files** | `5` | Project file tree with git status |
| **metrics** | `6` | System load, Claude usage, agent health |

## Keybindings

### Tmux

| Key | Action |
|-----|--------|
| `Ctrl-b` + arrows | Navigate panes |
| `Ctrl-b` + `z` | Zoom pane (toggle) |
| `Ctrl-b` + `d` | Detach session |
| `Ctrl-b` + `[` | Scroll mode |
| `Ctrl-b` + `0` | Main window |
| Mouse scroll | Scroll pane content |
| Mouse click | Select pane |
| Mouse drag border | Resize pane |

### Sidebar

| Key | Action |
|-----|--------|
| `Tab` / `Shift-Tab` | Cycle tabs |
| `1`-`6` | Direct tab select |
| `j` / `k` | Navigate list |
| `A` | Add agent (Players tab) |
| `X` | Kill agent (Players tab) |
| `R` | Rename agent (Players tab) |
| `f` | Focus agent (Players tab) |
| `p` | Pause/resume agent (Players tab) |
| `r` | Restart agent (Players tab) |
| `t` | Toggle beads filter (Beads tab) |
| `s` | Cycle status filter (Beads tab) |
| `l` | Cycle type filter (Beads tab) |
| `Enter` | Open bead detail / toggle dir (Beads/Files) |
| `m` | Merge all branches (Charts tab) |
| `q` | Quit sidebar |

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
| `vin` | Initialize project |

### Agent
| Command | Action |
|---------|--------|
| `vag` | Agent commands |
| `vaa` | Add Claude agent |
| `val` | List agents |
| `vak` | Kill agent |
| `vam` | Merge all agent branches |

### Beads
| Command | Action |
|---------|--------|
| `bds` | Show ready tasks |
| `bdl` | List all tasks |
| `bda` | List all (incl. closed) |
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
# Claude command and flags
export VAMP_CLAUDE_CMD="claude"
export VAMP_CLAUDE_FLAGS="--dangerously-skip-permissions"

# Agent type overrides (optional)
# export VAMP_AGENT_CODEX_CMD="codex"
# export VAMP_AGENT_CURSOR_CMD="cursor"

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
vamp                  # Launches tmux environment with sidebar
ss                    # Prime beads context, show ready tasks
```

The `ss` command (session start) runs `bd prime` which loads all your beads context into Claude's memory. Claude now knows about all your tasks, dependencies, and progress.

### During Work

```bash
# Check what's available
bds                   # Ready tasks (no blockers)
bdip                  # In-progress tasks

# Add parallel agents for more throughput
vamp agent add claude # Or press 'a' in sidebar

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

## Testing

Vamp uses [bats-core](https://github.com/bats-core/bats-core) for bash tests and standard Rust tests for the sidebar.

### Running Tests

```bash
# Run all bash tests
./tests/run_tests.sh

# Run only unit tests
./tests/run_tests.sh unit

# Run specific test file
./tests/run_tests.sh tests/unit/args_test.bats

# Run sidebar tests
cd sidebar && cargo test
```

### Test Structure

```
tests/
├── bats/                    # bats-core submodules
│   ├── bats-core/
│   ├── bats-support/
│   └── bats-assert/
├── helpers/
│   ├── test_helper.bash     # Shared utilities
│   ├── mock_git.bash        # Git mock utilities
│   └── mock_tmux.bash       # Tmux mock utilities
├── unit/
│   ├── smoke_test.bats      # Basic sanity checks
│   ├── args_test.bats       # Argument parsing tests
│   ├── agent_test.bats      # Agent lifecycle tests
│   ├── state_test.bats      # State file tests
│   ├── session_test.bats    # Session management tests
│   ├── setup_test.bats      # Setup/init tests
│   └── update_test.bats     # Self-update tests
└── integration/

sidebar/
└── src/                     # Rust unit tests (cargo test)
```

### CI

Tests run automatically on push and PR via GitHub Actions. See `.github/workflows/test.yml`.

## Project Structure

```
~/.local/
├── bin/
│   ├── vamp              # Main script
│   └── vamp-sidebar      # Ratatui sidebar binary
└── share/
    └── vamp/
        └── vamp-utils.sh # Shell helpers

~/.config/vamp/
└── config                # Configuration

# Per-project (created at runtime)
.vamp/
├── state.json            # Agent state (bash → sidebar)
├── commands.json         # Sidebar commands (sidebar → bash)
├── files.json            # File changes across worktrees
├── beads.json            # Beads issue cache
├── tree.json             # Project file tree with git status
├── metrics.json          # System, Claude usage, agent health
└── activity.log          # Activity log (cleared per session)

.vamp-agents/             # Git worktrees for additional agents
├── claude-1/
├── codex-1/
└── ...
```

## Why "vamp"?

In jazz, a **vamp** is a repeating musical figure that accompanies soloists during improvisation. It provides a stable foundation while allowing creative freedom.

That's what this environment does - it keeps the rhythm (agent orchestration, file monitoring, task tracking) while you improvise solutions with Claude.

## License

MIT
