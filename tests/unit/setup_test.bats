#!/usr/bin/env bats
#
# Unit tests for vamp setup operations
#
# Tests setup_claude_md, setup_beads, and related functions
#

load '../helpers/test_helper'

setup() {
    setup_temp_dir
    # Create a mock HOME directory
    export REAL_HOME="$HOME"
    export HOME="$TEST_TEMP_DIR/home"
    mkdir -p "$HOME"
}

teardown() {
    export HOME="$REAL_HOME"
    teardown_temp_dir
}

# ============================================
# write_beads_section Tests
# ============================================

@test "write_beads_section: outputs beads workflow" {
    # Source the vamp script to get the function
    source "$VAMP_BIN"

    run write_beads_section
    assert_success
    assert_output --partial "Beads Workflow"
    assert_output --partial "Issue Tracking"
}

@test "write_beads_section: includes critical sync branch warning" {
    source "$VAMP_BIN"

    run write_beads_section
    assert_success
    assert_output --partial "CRITICAL"
    assert_output --partial "sync.branch"
}

@test "write_beads_section: includes core workflow commands" {
    source "$VAMP_BIN"

    run write_beads_section
    assert_success
    assert_output --partial "bd ready"
    assert_output --partial "bd close"
    assert_output --partial "bd create"
}

@test "write_beads_section: includes session close protocol" {
    source "$VAMP_BIN"

    run write_beads_section
    assert_success
    assert_output --partial "Session Close Protocol"
    assert_output --partial "git status"
    assert_output --partial "bd sync"
}

@test "write_beads_section: includes recommended permissions" {
    source "$VAMP_BIN"

    run write_beads_section
    assert_success
    assert_output --partial "Recommended Permissions"
    assert_output --partial "Bash(bd:*)"
}

# ============================================
# setup_claude_md Tests
# ============================================

@test "setup_claude_md: creates new CLAUDE.md when missing" {
    source "$VAMP_BIN"

    [ ! -f "$HOME/.claude/CLAUDE.md" ]

    run setup_claude_md
    assert_success
    assert_output --partial "Created"

    [ -f "$HOME/.claude/CLAUDE.md" ]
    grep -q "Beads Workflow" "$HOME/.claude/CLAUDE.md"
}

@test "setup_claude_md: creates .claude directory if needed" {
    source "$VAMP_BIN"

    [ ! -d "$HOME/.claude" ]

    run setup_claude_md
    assert_success

    [ -d "$HOME/.claude" ]
}

@test "setup_claude_md: skips if beads workflow already present" {
    source "$VAMP_BIN"

    mkdir -p "$HOME/.claude"
    echo "## Beads Workflow" > "$HOME/.claude/CLAUDE.md"

    run setup_claude_md
    assert_success
    assert_output --partial "already has beads workflow"
}

@test "setup_claude_md: new file includes header" {
    source "$VAMP_BIN"

    run setup_claude_md
    assert_success

    # Check header is present
    head -5 "$HOME/.claude/CLAUDE.md" | grep -q "Global Claude Code Instructions"
}

# ============================================
# Help Integration Tests
# ============================================

@test "setup command: appears in help" {
    run_vamp --help
    assert_success
    assert_output --partial "setup"
    assert_output --partial "Install beads hooks"
}
