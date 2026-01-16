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

@test "vamp swarm --help shows swarm usage" {
    run_vamp swarm --help
    assert_success
    assert_output --partial "swarm"
    assert_output --partial "workers"
}

@test "vamp binary exists and is executable" {
    assert_file_exists "$VAMP_BIN"
    [ -x "$VAMP_BIN" ]
}
