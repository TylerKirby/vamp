#!/usr/bin/env bats
#
# Smoke tests to verify vamp basic functionality
#

load '../helpers/test_helper'

@test "vamp --help shows usage" {
    run_vamp --help
    assert_success
    assert_output --partial "vamp"
    assert_output --partial "USAGE"
}

@test "vamp --version shows version" {
    run_vamp --version
    assert_success
    assert_output --partial "vamp"
}

@test "vamp list works without error" {
    run_vamp list
    # Should succeed even with no sessions
    assert_success
}

@test "vamp agent shows agent usage" {
    run_vamp agent
    assert_success
    assert_output --partial "agent"
    assert_output --partial "add"
}

@test "vamp binary exists and is executable" {
    assert_file_exists "$VAMP_BIN"
    [ -x "$VAMP_BIN" ]
}

@test "vamp doctor runs checks" {
    run_vamp doctor
    # May fail if setup is incomplete, but should run checks
    assert_output --partial "vamp doctor"
    assert_output --partial "tmux"
    assert_output --partial "Claude Code"
}

@test "vamp doctor shows in help" {
    run_vamp --help
    assert_success
    assert_output --partial "doctor"
    assert_output --partial "Validate setup"
}
