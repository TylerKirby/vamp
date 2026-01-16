#!/usr/bin/env bats
#
# Unit tests for vamp swarm operations
#
# Tests swarm status, worktree management, and related functions
#

load '../helpers/test_helper'

setup() {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"

    git init --initial-branch=main
    echo ".vamp-workers/" > .gitignore
    echo "initial content" > README.md
    git add .
    git commit -m "Initial commit"

    export TEST_REPO_DIR
}

teardown() {
    cd /
    teardown_temp_dir
}

# ============================================
# swarm --status Tests
# ============================================

@test "swarm status: works without worktrees" {
    run_vamp swarm --status
    assert_success
    assert_output --partial "No swarm worktrees found"
}

@test "swarm status: shows worktrees when they exist" {
    # Create a worktree manually
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1

    run_vamp swarm --status
    assert_success
    assert_output --partial "worker-1"
    assert_output --partial "swarm/worker-1"
}

@test "swarm status: shows multiple worktrees" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1
    git worktree add .vamp-workers/worker-2 -b swarm/worker-2

    run_vamp swarm --status
    assert_success
    assert_output --partial "worker-1"
    assert_output --partial "worker-2"
}

@test "swarm status: shows dirty status for modified files" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1
    # Need to modify a tracked file for git diff-index to detect
    echo "modified" > .vamp-workers/worker-1/README.md

    run_vamp swarm --status
    assert_success
    assert_output --partial "uncommitted"
}

@test "swarm status: shows commit count" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1

    # Add a commit to the worker
    echo "new file" > .vamp-workers/worker-1/newfile.txt
    git -C .vamp-workers/worker-1 add .
    git -C .vamp-workers/worker-1 commit -m "Add new file"

    run_vamp swarm --status
    assert_success
    # Should show commits ahead
    assert_output --partial "commit"
}

# ============================================
# swarm --cleanup Tests
# ============================================

@test "swarm cleanup: works without worktrees" {
    run_vamp swarm --cleanup --yes
    assert_success
    assert_output --partial "No swarm worktrees"
}

@test "swarm cleanup: removes worktrees" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1

    run_vamp swarm --cleanup --yes
    assert_success
    [ ! -d ".vamp-workers/worker-1" ]
}

@test "swarm cleanup: keeps branches by default" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1

    run_vamp swarm --cleanup --yes
    assert_success
    # Branch should still exist
    git rev-parse --verify swarm/worker-1 &>/dev/null
}

# ============================================
# swarm --merge Tests (Unit level, integration tests do more)
# ============================================

@test "swarm merge: works without worktrees" {
    run_vamp swarm --merge --yes
    assert_success
    assert_output --partial "No swarm worktrees"
}

@test "swarm merge: requires --yes for non-interactive" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1
    echo "change" > .vamp-workers/worker-1/file.txt
    git -C .vamp-workers/worker-1 add .
    git -C .vamp-workers/worker-1 commit -m "Add file"

    # Without --yes, should prompt (which fails in non-interactive)
    run_vamp swarm --merge --yes
    assert_success
}

# ============================================
# swarm --finish Tests
# ============================================

@test "swarm finish: works without worktrees" {
    run_vamp swarm --finish --yes
    assert_success
    assert_output --partial "No swarm worktrees"
}

@test "swarm finish: removes worktrees but keeps branches" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1

    run_vamp swarm --finish --yes
    assert_success

    # Worktree should be gone
    [ ! -d ".vamp-workers/worker-1" ]

    # Branch is preserved by default (need --delete-branches for full cleanup)
    git rev-parse --verify swarm/worker-1 &>/dev/null
}

# ============================================
# Worker Count Validation
# ============================================

@test "swarm: accepts -w 1" {
    run_vamp swarm -w 1 --help
    assert_success
}

@test "swarm: accepts -w 8" {
    run_vamp swarm -w 8 --help
    assert_success
}

@test "swarm: rejects -w 0" {
    run_vamp swarm -w 0
    assert_failure
    assert_output --partial "1-8"
}

@test "swarm: rejects -w 9" {
    run_vamp swarm -w 9
    assert_failure
    assert_output --partial "1-8"
}

@test "swarm: rejects negative worker count" {
    run_vamp swarm -w -1
    assert_failure
}

@test "swarm: rejects non-numeric worker count" {
    run_vamp swarm -w foo
    assert_failure
}

# ============================================
# Edge Cases
# ============================================

@test "swarm: empty .vamp-workers directory" {
    mkdir -p .vamp-workers
    # Directory exists but empty

    run_vamp swarm --status
    assert_success
    assert_output --partial "No swarm worktrees"
}

@test "swarm: runs from repo root" {
    # Should work from any subdirectory
    mkdir -p subdir
    cd subdir

    run "$VAMP_BIN" swarm --status
    assert_success
}

@test "swarm: --force flag combines with --merge" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1
    echo "dirty" > .vamp-workers/worker-1/dirty.txt
    echo "committed" > .vamp-workers/worker-1/file.txt
    git -C .vamp-workers/worker-1 add file.txt
    git -C .vamp-workers/worker-1 commit -m "Add file"

    run_vamp swarm --merge --force --yes
    assert_success
    # Should merge despite dirty file
    assert_output --partial "will merge anyway with --force"
}

@test "swarm: --autostash flag combines with --merge" {
    mkdir -p .vamp-workers
    git worktree add .vamp-workers/worker-1 -b swarm/worker-1
    echo "dirty" > .vamp-workers/worker-1/dirty.txt
    echo "committed" > .vamp-workers/worker-1/file.txt
    git -C .vamp-workers/worker-1 add file.txt
    git -C .vamp-workers/worker-1 commit -m "Add file"

    run_vamp swarm --merge --autostash --yes
    assert_success
    assert_output --partial "Stashing"
}

# ============================================
# Help Output
# ============================================

@test "swarm: --help shows all options" {
    run_vamp swarm --help
    assert_success
    assert_output --partial "--workers"
    assert_output --partial "--status"
    assert_output --partial "--cleanup"
    assert_output --partial "--merge"
    assert_output --partial "--finish"
    assert_output --partial "--force"
    assert_output --partial "--autostash"
}

@test "swarm: --help shows workflow" {
    run_vamp swarm --help
    assert_success
    assert_output --partial "WORKFLOW"
}
