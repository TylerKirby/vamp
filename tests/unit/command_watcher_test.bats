#!/usr/bin/env bats
#
# Regression tests for command_watcher jq parsing logic
#
# The command_watcher uses pipe-delimited jq output to parse commands.
# A previous bug used @tsv (tab-delimited) which collapsed empty fields
# because bash's `read` with IFS=$'\t' treats consecutive tabs as one
# separator. This caused agent types to shift into wrong positions
# (e.g., codex→claude).
#
# These tests verify the jq parsing pipeline directly.

load '../helpers/test_helper'

setup() {
    setup_temp_dir

    # We need jq for these tests
    if ! command -v jq &>/dev/null; then
        skip "jq not installed"
    fi
}

teardown() {
    cd /
    teardown_temp_dir
}

# ============================================
# Helper: run the jq pipeline from command_watcher
# ============================================

# This mirrors the exact jq expression from bin/vamp command_watcher()
parse_commands() {
    local json="$1"
    echo "$json" | jq -r '.[] | [.action, (.id // ""), (.type // ""), (.name // "")] | join("|")' 2>/dev/null
}

# ============================================
# Create command parsing
# ============================================

@test "parse: create command preserves type and name" {
    local json='[{"action":"create","type":"claude","name":"miles"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "create"
    assert_equal "$cmd_id" ""
    assert_equal "$cmd_type" "claude"
    assert_equal "$cmd_name" "miles"
}

@test "parse: create codex command preserves type" {
    local json='[{"action":"create","type":"codex","name":"coltrane"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "create"
    assert_equal "$cmd_type" "codex"
    assert_equal "$cmd_name" "coltrane"
}

@test "parse: create cursor command preserves type" {
    local json='[{"action":"create","type":"cursor","name":"monk"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "create"
    assert_equal "$cmd_type" "cursor"
    assert_equal "$cmd_name" "monk"
}

# ============================================
# Kill command parsing
# ============================================

@test "parse: kill command preserves id with empty type/name" {
    local json='[{"action":"kill","id":"agent-001"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "kill"
    assert_equal "$cmd_id" "agent-001"
    # These should be empty, NOT shifted
    assert_equal "$cmd_type" ""
    assert_equal "$cmd_name" ""
}

# ============================================
# Regression: empty field collapse
# ============================================

@test "regression: empty name field does NOT shift type" {
    # This was the original bug: create with empty name caused
    # type to shift into name position with @tsv
    local json='[{"action":"create","type":"codex","name":""}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "create"
    assert_equal "$cmd_id" ""
    assert_equal "$cmd_type" "codex"
    assert_equal "$cmd_name" ""
}

@test "regression: codex type specifically preserved (not shifted to claude)" {
    # The original bug: sidebar sends create with type=codex, empty name.
    # @tsv collapsed the empty fields, and bash read shifted codex
    # from the type position into the wrong field.
    local json='[{"action":"create","type":"codex"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "create"
    assert_equal "$cmd_type" "codex"
    # Type must NOT be empty (the core regression)
    [ -n "$cmd_type" ]
}

@test "regression: multiple empty fields don't collapse" {
    # kill has id but no type or name — two consecutive empty fields
    local json='[{"action":"kill","id":"agent-xyz"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "kill"
    assert_equal "$cmd_id" "agent-xyz"
    assert_equal "$cmd_type" ""
    assert_equal "$cmd_name" ""
}

# ============================================
# Multiple commands
# ============================================

@test "parse: multiple commands all processed" {
    local json='[
        {"action":"create","type":"claude","name":"miles"},
        {"action":"kill","id":"agent-001"},
        {"action":"focus","id":"agent-002"}
    ]'
    local parsed
    parsed=$(parse_commands "$json")
    local count
    count=$(echo "$parsed" | wc -l | tr -d ' ')
    assert_equal "$count" "3"
}

@test "parse: multiple commands preserve order" {
    local json='[
        {"action":"create","type":"codex","name":"first"},
        {"action":"kill","id":"agent-x"}
    ]'
    local parsed
    parsed=$(parse_commands "$json")

    local line1 line2
    line1=$(echo "$parsed" | head -1)
    line2=$(echo "$parsed" | tail -1)

    IFS='|' read -r action1 _ _ _ <<< "$line1"
    IFS='|' read -r action2 _ _ _ <<< "$line2"
    assert_equal "$action1" "create"
    assert_equal "$action2" "kill"
}

# ============================================
# Edge cases
# ============================================

@test "parse: empty commands array produces no output" {
    local json='[]'
    local parsed
    parsed=$(parse_commands "$json")
    assert_equal "$parsed" ""
}

@test "parse: malformed JSON fails gracefully" {
    local parsed
    # jq returns non-zero on invalid input — that's expected, not a crash
    parsed=$(parse_commands "not valid json!!!" || true)
    # Should produce empty output
    assert_equal "$parsed" ""
}

@test "parse: focus command" {
    local json='[{"action":"focus","id":"agent-002"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "focus"
    assert_equal "$cmd_id" "agent-002"
}

@test "parse: pause command" {
    local json='[{"action":"pause","id":"agent-003"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "pause"
    assert_equal "$cmd_id" "agent-003"
}

@test "parse: rename command preserves name" {
    local json='[{"action":"rename","id":"agent-001","name":"dizzy"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "rename"
    assert_equal "$cmd_id" "agent-001"
    assert_equal "$cmd_name" "dizzy"
}

@test "parse: merge command" {
    local json='[{"action":"merge","id":"--all"}]'
    local parsed
    parsed=$(parse_commands "$json")

    IFS='|' read -r action cmd_id cmd_type cmd_name <<< "$parsed"
    assert_equal "$action" "merge"
    assert_equal "$cmd_id" "--all"
}
