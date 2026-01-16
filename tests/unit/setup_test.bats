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

# ============================================
# setup_permissions Tests
# ============================================

@test "setup_permissions: creates settings.local.json when missing" {
    source "$VAMP_BIN"

    [ ! -f "$HOME/.claude/settings.local.json" ]

    # Non-interactive - provide "y" via stdin
    echo "y" | setup_permissions
    [ -f "$HOME/.claude/settings.local.json" ]
}

@test "setup_permissions: new file includes all recommended permissions" {
    source "$VAMP_BIN"

    echo "y" | setup_permissions

    # Check all recommended permissions are present
    grep -q "Bash(bd:\*)" "$HOME/.claude/settings.local.json"
    grep -q "Bash(git add:\*)" "$HOME/.claude/settings.local.json"
    grep -q "Bash(docker:\*)" "$HOME/.claude/settings.local.json"
}

@test "setup_permissions: skips if all permissions present" {
    source "$VAMP_BIN"

    # Create file with all permissions
    mkdir -p "$HOME/.claude"
    cat > "$HOME/.claude/settings.local.json" << 'EOF'
{
  "permissions": {
    "allow": [
      "Bash(bd:*)",
      "Bash(git add:*)",
      "Bash(git commit:*)",
      "Bash(git status:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)",
      "Bash(docker:*)",
      "Bash(docker-compose:*)"
    ]
  }
}
EOF

    run setup_permissions
    assert_success
    assert_output --partial "All recommended permissions already configured"
}

@test "setup_permissions: detects missing permissions" {
    source "$VAMP_BIN"

    # Create file with only some permissions
    mkdir -p "$HOME/.claude"
    cat > "$HOME/.claude/settings.local.json" << 'EOF'
{
  "permissions": {
    "allow": [
      "Bash(bd:*)"
    ]
  }
}
EOF

    run setup_permissions
    # Will fail due to read prompt, but output should show missing count
    assert_output --partial "Missing"
    assert_output --partial "recommended permissions"
}

@test "setup_permissions: --permissions flag works" {
    run_vamp setup --permissions <<< "n"
    # Should run permissions setup only (may fail due to prompts but shows correct output)
    assert_output --partial "Setting up Claude Code permissions"
}

# ============================================
# install_dependencies Tests
# ============================================

@test "install_dependencies: --deps flag works" {
    run_vamp setup --deps <<< "n"
    assert_output --partial "Installing vamp dependencies"
}

@test "install_dependencies: shows OS detection" {
    run_vamp setup --deps <<< "n"
    # Should show either macOS or Linux
    [[ "$output" == *"macOS"* ]] || [[ "$output" == *"Linux"* ]]
}

@test "install_dependencies: checks for tmux" {
    run_vamp setup --deps <<< "n"
    assert_output --partial "tmux"
}

@test "install_dependencies: checks for beads" {
    run_vamp setup --deps <<< "n"
    assert_output --partial "Beads"
}

@test "install_dependencies: checks for beads_viewer" {
    run_vamp setup --deps <<< "n"
    assert_output --partial "beads_viewer"
}

# ============================================
# init_project validation Tests
# ============================================

@test "init: checks for global setup" {
    source "$VAMP_BIN"

    # Ensure hooks dir doesn't exist in test HOME
    rm -rf "$HOME/.claude/hooks"

    # Run init with "n" to skip setup
    cd "$TEST_TEMP_DIR"
    mkdir test-project && cd test-project

    run init_project <<< "n"
    assert_output --partial "Claude Code hooks not installed"
}

@test "init: checks for global CLAUDE.md" {
    source "$VAMP_BIN"

    # Ensure CLAUDE.md doesn't exist in test HOME
    rm -f "$HOME/.claude/CLAUDE.md"
    # Create hooks dir so hooks check passes
    mkdir -p "$HOME/.claude/hooks"
    touch "$HOME/.claude/hooks/test.sh"

    cd "$TEST_TEMP_DIR"
    mkdir test-project2 && cd test-project2

    run init_project <<< "n"
    assert_output --partial "Global CLAUDE.md not found"
}

@test "init: offers to run vamp setup when needed" {
    source "$VAMP_BIN"

    rm -rf "$HOME/.claude"

    cd "$TEST_TEMP_DIR"
    mkdir test-project3 && cd test-project3

    run init_project <<< "n"
    assert_output --partial "Global setup incomplete"
}

@test "init: continues with project init after setup check" {
    source "$VAMP_BIN"

    # Setup complete scenario
    mkdir -p "$HOME/.claude/hooks"
    touch "$HOME/.claude/hooks/test.sh"
    mkdir -p "$HOME/.claude"
    echo "## Beads Workflow" > "$HOME/.claude/CLAUDE.md"

    cd "$TEST_TEMP_DIR"
    mkdir test-project4 && cd test-project4
    git init

    run init_project
    # Should proceed to project init (check for CLAUDE.md creation)
    assert_output --partial "Initializing project"
}
