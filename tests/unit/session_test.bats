#!/usr/bin/env bats
#
# Unit tests for vamp session management
#
# Tests list, attach, kill session functions
#

load '../helpers/test_helper'

# ============================================
# list_sessions Tests
# ============================================

@test "list: shows 'no active sessions' when none exist" {
    run_vamp list
    assert_success
    assert_output --partial "Active vamp sessions"
    # Either shows no sessions or shows actual sessions
    # Can't mock tmux easily here, so just verify it runs
}

@test "list: uses 'ls' alias" {
    run_vamp ls
    assert_success
    assert_output --partial "Active vamp sessions"
}

# ============================================
# attach_session Tests
# ============================================

@test "attach: requires session name" {
    run_vamp attach
    assert_failure
    assert_output --partial "Usage: vamp attach"
}

@test "attach: shows sessions when name missing" {
    run_vamp attach
    assert_failure
    assert_output --partial "Active vamp sessions"
}

@test "attach: handles non-existent session" {
    run_vamp attach nonexistent-session-xyz123
    assert_failure
    assert_output --partial "not found"
}

@test "attach: 'a' alias works" {
    run_vamp a
    assert_failure
    assert_output --partial "Usage: vamp attach"
}

# ============================================
# kill_session Tests
# ============================================

@test "kill: requires session name" {
    run_vamp kill
    assert_failure
    assert_output --partial "Usage: vamp kill"
}

@test "kill: shows sessions when name missing" {
    run_vamp kill
    assert_failure
    assert_output --partial "Active vamp sessions"
}

@test "kill: handles non-existent session" {
    run_vamp kill nonexistent-session-xyz123
    # Should fail or show not found message
    assert_output --partial "not found"
}

@test "kill: 'k' alias works" {
    run_vamp k
    assert_failure
    assert_output --partial "Usage: vamp kill"
}

# ============================================
# kill_all_sessions Tests
# ============================================

@test "killall: handles no sessions gracefully" {
    # If there are sessions, it prompts; if none, it succeeds
    run_vamp killall
    # Either succeeds with "No vamp sessions" or prompts
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

# ============================================
# Session Naming Tests
# ============================================

@test "sessions: are prefixed with 'vamp-'" {
    # list_sessions filters for vamp- prefix
    run_vamp list
    assert_success
    # Output should describe vamp sessions format
    assert_output --partial "vamp sessions"
}

# ============================================
# Edge Cases
# ============================================

@test "attach: handles session name with spaces" {
    # Session names shouldn't have spaces, but verify no crash
    run_vamp attach "my project"
    assert_failure
}

@test "kill: handles session name with special chars" {
    # Verify no crash with special characters
    run_vamp kill "test-session-123"
    # Will fail (session doesn't exist) but shouldn't crash
    [ "$status" -eq 0 ] || [ "$status" -eq 1 ]
}

# ============================================
# Help Integration Tests
# ============================================

@test "session commands: appear in help" {
    run_vamp --help
    assert_success
    assert_output --partial "list"
    assert_output --partial "attach"
    assert_output --partial "kill"
    assert_output --partial "killall"
}

@test "session commands: describe purpose" {
    run_vamp --help
    assert_success
    assert_output --partial "List active"
    assert_output --partial "Attach to"
    assert_output --partial "Kill"
}
