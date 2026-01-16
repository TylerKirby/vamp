#!/bin/bash
#
# Git mock utilities for vamp tests
#
# Provides mock git commands and scenario helpers for testing
# vamp functionality without requiring real git operations.
#

# ============================================
# Git Command Mocks
# ============================================

# Setup mock git that intercepts specific commands
# Usage: setup_mock_git [scenario]
# Scenarios: clean, dirty, worktrees, merge_conflict
setup_mock_git() {
    local scenario="${1:-clean}"
    local mock_dir="$TEST_TEMP_DIR/mocks"
    mkdir -p "$mock_dir"

    # Create the mock git script
    cat > "$mock_dir/git" << 'MOCKSCRIPT'
#!/bin/bash
# Mock git command for testing

MOCK_SCENARIO="${MOCK_GIT_SCENARIO:-clean}"
MOCK_LOG="$TEST_TEMP_DIR/mocks/git.log"

# Log the call
echo "git $*" >> "$MOCK_LOG"

case "$1" in
    rev-parse)
        case "$2" in
            --show-toplevel)
                echo "$TEST_REPO_DIR"
                ;;
            --git-dir)
                echo "$TEST_REPO_DIR/.git"
                ;;
            --verify)
                # Check if branch exists
                if [ "$MOCK_SCENARIO" = "no_branches" ]; then
                    exit 1
                fi
                echo "abc123def456"
                ;;
            --abbrev-ref)
                echo "main"
                ;;
        esac
        ;;
    status)
        if [ "$2" = "--porcelain" ]; then
            case "$MOCK_SCENARIO" in
                dirty)
                    echo " M modified_file.txt"
                    echo "?? untracked_file.txt"
                    ;;
                clean|*)
                    # Output nothing for clean
                    ;;
            esac
        else
            echo "On branch main"
            if [ "$MOCK_SCENARIO" = "dirty" ]; then
                echo "Changes not staged for commit:"
                echo "  modified:   modified_file.txt"
            else
                echo "nothing to commit, working tree clean"
            fi
        fi
        ;;
    worktree)
        case "$2" in
            list)
                if [ "$MOCK_SCENARIO" = "worktrees" ]; then
                    echo "$TEST_REPO_DIR  abc1234 [main]"
                    echo "$TEST_REPO_DIR/.vamp-workers/worker-1  def5678 [swarm/worker-1]"
                    echo "$TEST_REPO_DIR/.vamp-workers/worker-2  ghi9012 [swarm/worker-2]"
                else
                    echo "$TEST_REPO_DIR  abc1234 [main]"
                fi
                ;;
            add)
                # Simulate worktree creation
                mkdir -p "$4"
                echo "Preparing worktree (new branch '$6')"
                ;;
            remove)
                echo "Removing worktree"
                ;;
        esac
        ;;
    branch)
        if [ "$2" = "-D" ] || [ "$2" = "-d" ]; then
            echo "Deleted branch $3"
        elif [ "$2" = "-a" ]; then
            echo "* main"
            if [ "$MOCK_SCENARIO" = "worktrees" ]; then
                echo "  swarm/worker-1"
                echo "  swarm/worker-2"
            fi
        fi
        ;;
    merge)
        if [ "$MOCK_SCENARIO" = "merge_conflict" ]; then
            echo "CONFLICT (content): Merge conflict in file.txt"
            exit 1
        else
            echo "Merge made by the 'ort' strategy."
        fi
        ;;
    log)
        echo "abc1234 feat: add feature"
        echo "def5678 fix: fix bug"
        ;;
    diff)
        if [ "$MOCK_SCENARIO" = "dirty" ]; then
            echo "diff --git a/file.txt b/file.txt"
            echo "--- a/file.txt"
            echo "+++ b/file.txt"
            echo "@@ -1 +1 @@"
            echo "-old"
            echo "+new"
        fi
        ;;
    stash)
        case "$2" in
            push)
                echo "Saved working directory"
                ;;
            pop)
                echo "Dropped stash"
                ;;
            list)
                if [ "$MOCK_SCENARIO" = "stashed" ]; then
                    echo "stash@{0}: WIP on main: abc1234 Initial commit"
                fi
                ;;
        esac
        ;;
    init)
        mkdir -p "$TEST_REPO_DIR/.git"
        echo "Initialized empty Git repository"
        ;;
    add|commit|checkout|fetch|pull|push)
        # These generally succeed silently or with minimal output
        echo "git $1: OK"
        ;;
    *)
        # Pass through to real git for unknown commands (fallback)
        command git "$@"
        ;;
esac
MOCKSCRIPT

    chmod +x "$mock_dir/git"

    # Export scenario and add mock to PATH
    export MOCK_GIT_SCENARIO="$scenario"
    export PATH="$mock_dir:$PATH"
}

# Get calls made to mock git
get_git_calls() {
    local mock_log="$TEST_TEMP_DIR/mocks/git.log"
    if [ -f "$mock_log" ]; then
        cat "$mock_log"
    fi
}

# Assert git was called with specific arguments
assert_git_called_with() {
    local expected="$1"
    local calls
    calls=$(get_git_calls)

    if ! echo "$calls" | grep -qF "$expected"; then
        echo "Expected git to be called with: $expected"
        echo "Actual calls:"
        echo "$calls"
        return 1
    fi
}

# Assert git was NOT called with specific arguments
refute_git_called_with() {
    local unexpected="$1"
    local calls
    calls=$(get_git_calls)

    if echo "$calls" | grep -qF "$unexpected"; then
        echo "Expected git NOT to be called with: $unexpected"
        echo "But found in calls:"
        echo "$calls"
        return 1
    fi
}

# Clear git call log
clear_git_calls() {
    local mock_log="$TEST_TEMP_DIR/mocks/git.log"
    rm -f "$mock_log"
}

# ============================================
# Real Git Scenario Helpers
# ============================================

# These helpers use real git for integration tests
# where we need actual git behavior

# Create a git repo with specific state
# Usage: create_repo_with_state [clean|dirty|worktrees]
create_repo_with_state() {
    local state="${1:-clean}"

    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"

    git -C "$TEST_REPO_DIR" init --initial-branch=main
    echo "initial" > "$TEST_REPO_DIR/README.md"
    echo ".vamp-workers/" > "$TEST_REPO_DIR/.gitignore"
    git -C "$TEST_REPO_DIR" add .
    git -C "$TEST_REPO_DIR" commit -m "Initial commit"

    case "$state" in
        dirty)
            echo "uncommitted" > "$TEST_REPO_DIR/dirty.txt"
            ;;
        worktrees)
            mkdir -p "$TEST_REPO_DIR/.vamp-workers"
            git -C "$TEST_REPO_DIR" worktree add "$TEST_REPO_DIR/.vamp-workers/worker-1" -b swarm/worker-1
            git -C "$TEST_REPO_DIR" worktree add "$TEST_REPO_DIR/.vamp-workers/worker-2" -b swarm/worker-2
            ;;
    esac

    export TEST_REPO_DIR
}

# Add commits to a worktree
add_worktree_commits() {
    local worktree="$1"
    local count="${2:-1}"

    for i in $(seq 1 "$count"); do
        echo "content $i" > "$worktree/file_$i.txt"
        git -C "$worktree" add .
        git -C "$worktree" commit -m "Add file $i"
    done
}

# Make a worktree dirty
make_worktree_dirty() {
    local worktree="$1"
    local filename="${2:-dirty.txt}"

    echo "uncommitted changes" > "$worktree/$filename"
}

# Create merge conflict scenario
create_merge_conflict() {
    local repo="$1"
    local branch="$2"

    # Add conflicting change on main
    echo "main version" > "$repo/conflict.txt"
    git -C "$repo" add conflict.txt
    git -C "$repo" commit -m "Main branch change"

    # Add conflicting change on feature branch
    git -C "$repo" checkout "$branch"
    echo "branch version" > "$repo/conflict.txt"
    git -C "$repo" add conflict.txt
    git -C "$repo" commit -m "Branch change"
    git -C "$repo" checkout main
}
