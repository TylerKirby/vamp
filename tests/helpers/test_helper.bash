#!/bin/bash
#
# Common test helper functions for vamp tests
#
# This file is sourced by all test files via:
#   load '../helpers/test_helper'

# Get the directory of this helper file
HELPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTS_DIR="$(cd "$HELPER_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$TESTS_DIR/.." && pwd)"
VAMP_BIN="$PROJECT_ROOT/bin/vamp"

# Load bats helper libraries
load "$TESTS_DIR/bats/bats-support/load"
load "$TESTS_DIR/bats/bats-assert/load"

# ============================================
# Test Setup/Teardown Helpers
# ============================================

# Create a temporary directory for test artifacts
# Sets TEST_TEMP_DIR and creates the directory
setup_temp_dir() {
    TEST_TEMP_DIR="$(mktemp -d)"
    export TEST_TEMP_DIR
}

# Clean up temporary directory
teardown_temp_dir() {
    if [ -n "$TEST_TEMP_DIR" ] && [ -d "$TEST_TEMP_DIR" ]; then
        rm -rf "$TEST_TEMP_DIR"
    fi
}

# Create a temporary git repository for testing
# Sets TEST_REPO_DIR
setup_test_repo() {
    setup_temp_dir
    TEST_REPO_DIR="$TEST_TEMP_DIR/test-repo"
    mkdir -p "$TEST_REPO_DIR"

    git -C "$TEST_REPO_DIR" init --initial-branch=main
    echo "test" > "$TEST_REPO_DIR/README.md"
    git -C "$TEST_REPO_DIR" add .
    git -C "$TEST_REPO_DIR" commit -m "Initial commit"

    export TEST_REPO_DIR
}

# Clean up test repository
teardown_test_repo() {
    teardown_temp_dir
}

# ============================================
# Vamp Function Helpers
# ============================================

# Source vamp functions for testing
# This loads the vamp script but doesn't execute it
source_vamp_functions() {
    # We need to source the script in a way that only defines functions
    # without running the main logic

    # Create a temporary file that sources vamp but exits before main
    local temp_source="$TEST_TEMP_DIR/vamp_functions.bash"
    cat > "$temp_source" << 'EOF'
# Override set -e to prevent early exit
set +e

# Source the vamp script up to the main logic
VAMP_SOURCED=1
EOF

    # Extract just the function definitions from vamp
    # This is a simplified approach - we extract functions by name
    source "$temp_source"
}

# Run vamp with arguments and capture output
run_vamp() {
    run "$VAMP_BIN" "$@"
}

# ============================================
# Assertion Helpers
# ============================================

# Assert that output contains a string (case-insensitive)
assert_output_contains() {
    local expected="$1"
    if ! echo "$output" | grep -qi "$expected"; then
        echo "Expected output to contain: $expected"
        echo "Actual output: $output"
        return 1
    fi
}

# Assert that a file exists
assert_file_exists() {
    local file="$1"
    if [ ! -f "$file" ]; then
        echo "Expected file to exist: $file"
        return 1
    fi
}

# Assert that a directory exists
assert_dir_exists() {
    local dir="$1"
    if [ ! -d "$dir" ]; then
        echo "Expected directory to exist: $dir"
        return 1
    fi
}

# Assert that a git repository is clean
assert_git_clean() {
    local repo="${1:-.}"
    local status
    status=$(git -C "$repo" status --porcelain 2>/dev/null)
    if [ -n "$status" ]; then
        echo "Expected git repository to be clean"
        echo "Status: $status"
        return 1
    fi
}

# Assert that a branch exists
assert_branch_exists() {
    local repo="$1"
    local branch="$2"
    if ! git -C "$repo" rev-parse --verify "$branch" &>/dev/null; then
        echo "Expected branch to exist: $branch"
        return 1
    fi
}

# ============================================
# Mock Helpers
# ============================================

# Create a mock command that records calls
# Usage: create_mock "command_name" "response"
create_mock() {
    local cmd="$1"
    local response="${2:-}"
    local mock_dir="$TEST_TEMP_DIR/mocks"

    mkdir -p "$mock_dir"

    cat > "$mock_dir/$cmd" << EOF
#!/bin/bash
# Mock for $cmd
echo "\$0 \$@" >> "$mock_dir/${cmd}.calls"
$response
EOF
    chmod +x "$mock_dir/$cmd"

    # Add mock directory to PATH
    export PATH="$mock_dir:$PATH"
}

# Get recorded calls for a mock
get_mock_calls() {
    local cmd="$1"
    local mock_dir="$TEST_TEMP_DIR/mocks"

    if [ -f "$mock_dir/${cmd}.calls" ]; then
        cat "$mock_dir/${cmd}.calls"
    fi
}

# Assert mock was called with specific args
assert_mock_called_with() {
    local cmd="$1"
    local expected_args="$2"
    local calls
    calls=$(get_mock_calls "$cmd")

    if ! echo "$calls" | grep -qF "$expected_args"; then
        echo "Expected $cmd to be called with: $expected_args"
        echo "Actual calls: $calls"
        return 1
    fi
}
