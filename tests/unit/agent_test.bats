#!/usr/bin/env bats
#
# Unit tests for vamp agent operations
#
# Tests agent CRUD, state file management, and related functions
#

load '../helpers/test_helper'

setup() {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"
    cd "$TEST_REPO_DIR"

    git init --initial-branch=main
    echo ".vamp/" > .gitignore
    echo ".vamp-agents/" >> .gitignore
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
# agent help Tests
# ============================================

@test "agent: shows help with no subcommand" {
    run_vamp agent
    assert_success
    assert_output --partial "vamp agent add"
    assert_output --partial "vamp agent list"
    assert_output --partial "vamp agent kill"
}

@test "agent: help mentions agent types" {
    run_vamp agent
    assert_success
    assert_output --partial "claude"
    assert_output --partial "codex"
    assert_output --partial "cursor"
}

# ============================================
# agent add Tests
# ============================================

@test "agent add: creates worktree and branch" {
    run_vamp agent add claude test-agent
    assert_success
    assert_output --partial "Created agent: test-agent"

    # Worktree should exist
    [ -d ".vamp-agents/test-agent" ]

    # Branch should exist
    git rev-parse --verify "agent/test-agent" &>/dev/null
}

@test "agent add: creates state file" {
    run_vamp agent add claude test-agent
    assert_success

    # State file should exist
    [ -f ".vamp/state.json" ]

    # State should contain the agent
    if command -v jq &>/dev/null; then
        local agent_count=$(jq '.agents | length' .vamp/state.json)
        [ "$agent_count" -ge 1 ]
    fi
}

@test "agent add: rejects invalid type" {
    run_vamp agent add invalid-type
    assert_failure
    assert_output --partial "Unknown agent type"
}

@test "agent add: creates .vamp directory" {
    [ ! -d ".vamp" ]
    run_vamp agent add claude test-agent
    assert_success
    [ -d ".vamp" ]
}

@test "agent add: writes agent context file" {
    run_vamp agent add claude test-agent
    assert_success
    [ -f ".vamp-agents/test-agent/.vamp-agent" ]
}

# ============================================
# agent list Tests
# ============================================

@test "agent list: shows no agents message" {
    mkdir -p .vamp
    echo '{"version":"2.0.0","project":{},"agents":[],"totals":{"tokens":0,"cost_usd":0},"updated_at":""}' > .vamp/state.json
    run_vamp agent list
    assert_success
    assert_output --partial "No agents running"
}

@test "agent list: shows agents after creation" {
    run_vamp agent add claude test-agent
    run_vamp agent list
    assert_success
    assert_output --partial "test-agent"
}

# ============================================
# agent kill Tests
# ============================================

@test "agent kill: removes agent from state" {
    run_vamp agent add claude test-agent

    run_vamp agent kill test-agent --remove
    assert_success
    assert_output --partial "Removed agent: test-agent"

    # Worktree should be gone
    [ ! -d ".vamp-agents/test-agent" ]
}

@test "agent kill: preserves worktree without --remove" {
    run_vamp agent add claude test-agent

    run_vamp agent kill test-agent
    assert_success
    assert_output --partial "Killed agent: test-agent"

    # Worktree should still exist
    [ -d ".vamp-agents/test-agent" ]
}

# ============================================
# State File Tests
# ============================================

@test "state: init creates valid JSON" {
    run_vamp agent add claude test-agent
    assert_success

    if command -v jq &>/dev/null; then
        run jq '.' .vamp/state.json
        assert_success
    fi
}

@test "state: contains version field" {
    run_vamp agent add claude test-agent
    assert_success

    if command -v jq &>/dev/null; then
        local version=$(jq -r '.version' .vamp/state.json)
        [ "$version" = "2.0.0" ]
    fi
}

@test "state: agent has required fields" {
    run_vamp agent add claude test-agent
    assert_success

    if command -v jq &>/dev/null; then
        local agent_type=$(jq -r '.agents[0].type' .vamp/state.json)
        local agent_id=$(jq -r '.agents[0].id' .vamp/state.json)
        local status=$(jq -r '.agents[0].status' .vamp/state.json)

        [ "$agent_type" = "claude" ]
        [ "$agent_id" = "test-agent" ]
        [ "$status" = "running" ]
    fi
}

# ============================================
# Help Output Tests
# ============================================

@test "help: shows agent commands instead of swarm" {
    run_vamp help
    assert_success
    assert_output --partial "agent"
    assert_output --partial "MULTI-AGENT"
}

@test "help: no swarm references" {
    run_vamp help
    assert_success
    refute_output --partial "swarm"
}

@test "version: shows 2.0.0" {
    run_vamp version
    assert_success
    assert_output --partial "2.0.0"
}
