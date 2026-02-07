#!/usr/bin/env bats
#
# Unit tests for vamp state file management
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
# State File Structure Tests
# ============================================

@test "state: created on first agent add" {
    [ ! -f ".vamp/state.json" ]
    run_vamp agent add claude test-1
    assert_success
    [ -f ".vamp/state.json" ]
}

@test "state: has project info" {
    run_vamp agent add claude test-1
    assert_success

    if command -v jq &>/dev/null; then
        local project_name=$(jq -r '.project.name' .vamp/state.json)
        [ "$project_name" = "test-repo" ]

        local project_path=$(jq -r '.project.path' .vamp/state.json)
        [ "$project_path" = "$TEST_REPO_DIR" ]
    fi
}

@test "state: tracks multiple agents" {
    run_vamp agent add claude agent-1
    run_vamp agent add claude agent-2
    assert_success

    if command -v jq &>/dev/null; then
        local count=$(jq '.agents | length' .vamp/state.json)
        [ "$count" -eq 2 ]
    fi
}

@test "state: remove agent decrements count" {
    run_vamp agent add claude agent-1
    run_vamp agent add claude agent-2
    run_vamp agent kill agent-1 --remove

    if command -v jq &>/dev/null; then
        local count=$(jq '.agents | length' .vamp/state.json)
        [ "$count" -eq 1 ]

        local remaining=$(jq -r '.agents[0].id' .vamp/state.json)
        [ "$remaining" = "agent-2" ]
    fi
}

@test "state: has totals section" {
    run_vamp agent add claude test-1
    assert_success

    if command -v jq &>/dev/null; then
        local tokens=$(jq '.totals.tokens' .vamp/state.json)
        [ "$tokens" = "0" ]

        local cost=$(jq '.totals.cost_usd' .vamp/state.json)
        [ "$cost" = "0" ]
    fi
}

@test "state: has updated_at timestamp" {
    run_vamp agent add claude test-1
    assert_success

    if command -v jq &>/dev/null; then
        local updated=$(jq -r '.updated_at' .vamp/state.json)
        [ -n "$updated" ]
        [ "$updated" != "null" ]
    fi
}

@test "state: agent has worktree path" {
    run_vamp agent add claude test-1
    assert_success

    if command -v jq &>/dev/null; then
        local worktree=$(jq -r '.agents[0].worktree' .vamp/state.json)
        [ -n "$worktree" ]
        [ -d "$worktree" ]
    fi
}

@test "state: agent has branch" {
    run_vamp agent add claude test-1
    assert_success

    if command -v jq &>/dev/null; then
        local branch=$(jq -r '.agents[0].branch' .vamp/state.json)
        [ "$branch" = "agent/test-1" ]
    fi
}
