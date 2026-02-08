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
# Agent Command Parsing
# ============================================

@test "args: agent shows help with no subcommand" {
    run_vamp agent
    assert_success
    assert_output --partial "vamp agent add"
    assert_output --partial "vamp agent list"
    assert_output --partial "vamp agent kill"
}

@test "args: agent add requires type" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp agent add claude test-agent
    assert_success
    assert_output --partial "Created agent"
}

@test "args: agent list works" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"
    mkdir -p .vamp
    echo '{"version":"2.0.0","project":{},"agents":[],"totals":{"tokens":0,"cost_usd":0},"updated_at":""}' > .vamp/state.json

    run_vamp agent list
    assert_success
}

@test "args: agent kill works" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp agent add claude test-agent
    run_vamp agent kill test-agent --remove
    assert_success
}

@test "args: agent merge works without agents" {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"
    git init --initial-branch=main
    echo "test" > README.md
    git add . && git commit -m "Initial"

    run_vamp agent merge
    assert_success
}

teardown() {
    cd /
    if [ -n "$TEST_TEMP_DIR" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}
