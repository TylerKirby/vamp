#!/usr/bin/env bats
#
# Unit tests for vamp argument parsing
#

load '../helpers/test_helper'

# ============================================
# Main Command Parsing
# ============================================

@test "args: no arguments shows help or starts session" {
    # Running with no args in a non-git directory should show help
    run_vamp
    # Either shows help or fails gracefully
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

@test "args: --help shows help" {
    run_vamp --help
    assert_success
    assert_output --partial "USAGE"
    assert_output --partial "vamp"
}

@test "args: -h shows help" {
    run_vamp -h
    assert_success
    assert_output --partial "USAGE"
}

@test "args: help shows help" {
    run_vamp help
    assert_success
    assert_output --partial "USAGE"
}

@test "args: --version shows version" {
    run_vamp --version
    assert_success
    assert_output --partial "vamp v"
}

@test "args: -v shows version" {
    run_vamp -v
    assert_success
    assert_output --partial "vamp v"
}

@test "args: version shows version" {
    run_vamp version
    assert_success
    assert_output --partial "vamp v"
}

@test "args: list shows sessions" {
    run_vamp list
    assert_success
    assert_output --partial "Active vamp sessions"
}

@test "args: ls shows sessions (alias)" {
    run_vamp ls
    assert_success
    assert_output --partial "Active vamp sessions"
}

@test "args: attach without name shows usage" {
    run_vamp attach
    assert_failure
    assert_output --partial "Usage: vamp attach"
}

@test "args: a is alias for attach" {
    run_vamp a
    assert_failure
    assert_output --partial "Usage: vamp attach"
}

@test "args: kill without name shows usage" {
    run_vamp kill
    assert_failure
    assert_output --partial "Usage: vamp kill"
}

@test "args: k is alias for kill" {
    run_vamp k
    assert_failure
    assert_output --partial "Usage: vamp kill"
}

@test "args: killall is recognized" {
    run_vamp killall
    # Will either succeed (no sessions) or show sessions to kill
    # Can't test actual kill without confirmation input
    [ "$status" -eq 0 ] || assert_output --partial "This will kill"
}

# ============================================
# Swarm Command Parsing
# ============================================

@test "args: swarm --help shows swarm help" {
    run_vamp swarm --help
    assert_success
    assert_output --partial "vamp swarm"
    assert_output --partial "workers"
    assert_output --partial "--status"
}

@test "args: swarm -h shows swarm help" {
    run_vamp swarm -h
    assert_success
    assert_output --partial "vamp swarm"
}

@test "args: swarm --status works" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --status
    # Should succeed (status check)
    assert_success
}

@test "args: swarm --merge --yes works" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --merge --yes
    # Should succeed (no worktrees to merge)
    assert_success
}

@test "args: swarm --cleanup --yes works" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --cleanup --yes
    # Should succeed (no worktrees to cleanup)
    assert_success
}

@test "args: swarm -w accepts worker count" {
    run_vamp swarm -w 2 --help
    assert_success
}

@test "args: swarm --workers accepts worker count" {
    run_vamp swarm --workers 3 --help
    assert_success
}

@test "args: swarm --workers=N accepts worker count" {
    run_vamp swarm --workers=4 --help
    assert_success
}

@test "args: swarm -y is alias for --yes" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --merge -y
    assert_success
}

@test "args: swarm --force flag is recognized" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --merge --force --yes
    assert_success
}

@test "args: swarm --autostash flag is recognized" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --merge --autostash --yes
    assert_success
}

# ============================================
# Worker Count Validation
# ============================================

@test "args: swarm rejects worker count 0" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm -w 0
    assert_failure
    assert_output --partial "Worker count must be 1-8"
}

@test "args: swarm rejects worker count > 8" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm -w 9
    assert_failure
    assert_output --partial "Worker count must be 1-8"
}

@test "args: swarm rejects non-numeric worker count" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm -w abc
    assert_failure
    assert_output --partial "Worker count must be 1-8"
}

# ============================================
# Combined Flags
# ============================================

@test "args: swarm accepts multiple flags" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp swarm --merge --force --autostash --yes
    assert_success
}

teardown() {
    cd /
    if [ -n "$TEST_TEMP_DIR" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}
