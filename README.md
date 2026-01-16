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
- **Swarm mode** - run multiple Claude instances in parallel with git worktree isolation

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

# Check setup health
vamp doctor
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

## Swarm Mode

Swarm mode runs multiple Claude Code instances in parallel, each in its own isolated git worktree. Perfect for tackling multiple beads issues simultaneously.

### How It Works

1. **Git Worktrees** - Each worker gets its own directory with a dedicated branch (`swarm/worker-1`, `swarm/worker-2`, etc.)
2. **Beads Coordination** - Workers use beads to claim and track work, avoiding conflicts
3. **Isolated Changes** - Each worker's changes stay on its branch until merged
4. **Easy Cleanup** - Merge all branches back to main when done

### Swarm Commands

```bash
# Start swarm
vamp swarm              # 4 workers (default)
vamp swarm -w 2         # Custom worker count (1-8)

# Monitor progress
vamp swarm --status     # Show worker branches and changes

# Finish up
vamp swarm --merge      # Merge branches to main
vamp swarm --cleanup    # Remove worktrees (keep branches)
vamp swarm --finish     # Merge + cleanup + delete branches
```

### Swarm Workflow

```bash
# 1. Start with issues ready to work
cd ~/Projects/my-app
bd ready                # Check available work

# 2. Launch swarm
vamp swarm -w 3         # Start 3 workers

# 3. Each worker claims an issue
# Worker 1: bd update <id> --status=in_progress
# Worker 2: bd update <id> --status=in_progress
# Worker 3: bd update <id> --status=in_progress

# 4. Workers complete work on their branches
# Each commits to: swarm/worker-1, swarm/worker-2, etc.

# 5. Check progress
vamp swarm --status

# 6. Merge when done
vamp swarm --finish     # Merge all branches, cleanup
```

### Swarm Layout

```
Window 2: swarm
+------------+------------+
|  Worker 1  |  Worker 2  |
|  (branch   |  (branch   |
|   swarm/   |   swarm/   |
|   worker-1)|   worker-2)|
+------------+------------+
|  Worker 3  |  Worker 4  |
+------------+------------+
```

Navigate with `Ctrl-b + arrows`, zoom with `Ctrl-b + z`.

### Swarm Best Practices

- **Claim issues first** - Run `bd update <id> --status=in_progress` to avoid duplicate work
- **Commit frequently** - Keep changes small and atomic
- **Check status** - Run `vamp swarm --status` periodically to track progress
- **Handle conflicts** - If merge fails, resolve manually and continue with `git commit`

## Testing

Vamp uses [bats-core](https://github.com/bats-core/bats-core) for testing.

### Running Tests

```bash
# Run all tests
./tests/run_tests.sh

# Run only unit tests
./tests/run_tests.sh unit

# Run only integration tests
./tests/run_tests.sh integration

# Run specific test file
./tests/run_tests.sh tests/unit/args_test.bats

# Verbose output
./tests/run_tests.sh --verbose
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
│   └── args_test.bats       # Argument parsing tests
└── integration/
    └── swarm_merge_test.bats # Swarm merge workflow tests
```

### Writing Tests

Tests are written using bats syntax:

```bash
@test "description of what is being tested" {
    run_vamp some-command
    assert_success
    assert_output --partial "expected output"
}
```

Use the helpers in `test_helper.bash`:
- `setup_temp_dir` / `teardown_temp_dir` - Temp directory management
- `setup_test_repo` / `teardown_test_repo` - Git repo setup
- `run_vamp` - Run vamp with arguments
- `assert_file_exists` / `assert_dir_exists` - File assertions
- `load_git_mocks` / `load_tmux_mocks` - Mock utilities

### CI

Tests run automatically on push and PR via GitHub Actions. See `.github/workflows/test.yml`.

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
