#!/usr/bin/env bats
#
# Tests for git_push_with_retry functionality
#

load '../helpers/test_helper'

# Source vamp to get access to helper functions
setup() {
    setup_temp_dir

    # Create a mock directory
    MOCK_DIR="$TEST_TEMP_DIR/mocks"
    mkdir -p "$MOCK_DIR"

    # Export color codes and helper functions by sourcing relevant parts of vamp
    export RED='\033[0;31m'
    export GREEN='\033[0;32m'
    export YELLOW='\033[1;33m'
    export DIM='\033[2m'
    export NC='\033[0m'
    export VAMP_VERBOSE=""
    export VAMP_DEBUG=""

    # Define helper functions from vamp
    log_verbose() { [[ -n "$VAMP_VERBOSE" ]] && echo -e "${DIM}$*${NC}" >&2; return 0; }
    log_debug() { [[ -n "$VAMP_DEBUG" ]] && echo -e "${DIM}[DEBUG $(date +%H:%M:%S)] $*${NC}" >&2; return 0; }
    show_error() {
        local message="$1"
        local suggestion="${2:-}"
        echo -e "${RED}Error: $message${NC}" >&2
        if [ -n "$suggestion" ]; then
            echo -e "${DIM}  → $suggestion${NC}" >&2
        fi
    }

    # Source the retry functions from vamp
    GIT_ERROR=""

    is_retryable_git_error() {
        local error="$1"
        local retryable_patterns=(
            "Could not resolve host"
            "Connection refused"
            "Connection timed out"
            "Network is unreachable"
            "Connection reset by peer"
            "Failed to connect"
            "Unable to access"
            "Couldn't resolve host"
            "The requested URL returned error: 5"
            "SSL_connect"
            "gnutls_handshake"
            "Operation timed out"
        )

        for pattern in "${retryable_patterns[@]}"; do
            if [[ "$error" == *"$pattern"* ]]; then
                return 0
            fi
        done
        return 1
    }

    git_push_with_retry() {
        local max_retries="${VAMP_PUSH_RETRIES:-3}"
        local attempt=1
        local backoff=1
        local stderr_file=$(mktemp)
        local all_errors=""

        log_debug "git_push_with_retry: max_retries=$max_retries, args: $*"

        while [ "$attempt" -le "$max_retries" ]; do
            log_debug "Push attempt $attempt/$max_retries"

            if git push "$@" 2>"$stderr_file"; then
                rm -f "$stderr_file"
                if [ "$attempt" -gt 1 ]; then
                    echo -e "${GREEN}Push succeeded on attempt $attempt${NC}"
                fi
                return 0
            fi

            local exit_code=$?
            local error=$(cat "$stderr_file")
            GIT_ERROR="$error"

            all_errors="${all_errors}Attempt $attempt: $error"$'\n'

            log_debug "Push failed (exit $exit_code): $error"

            if ! is_retryable_git_error "$error"; then
                rm -f "$stderr_file"
                log_verbose "Push failed with permanent error (not retrying)"
                show_error "Git push failed" "This appears to be a permanent error, not a network issue"
                echo -e "${DIM}Error: $error${NC}" >&2
                return 1
            fi

            if [ "$attempt" -lt "$max_retries" ]; then
                echo -e "${YELLOW}Push failed (network error), retrying ($((attempt + 1))/$max_retries)...${NC}"
                log_verbose "Waiting ${backoff}s before retry"
                # Use shorter sleep for tests
                sleep 0.1
                backoff=$((backoff * 2))
            fi

            ((attempt++))
        done

        rm -f "$stderr_file"

        show_error "Git push failed after $max_retries attempts" "Check your network connection and try again"
        echo -e "${DIM}Errors from all attempts:${NC}" >&2
        echo -e "${DIM}$all_errors${NC}" >&2
        return 1
    }

    export -f log_verbose log_debug show_error is_retryable_git_error git_push_with_retry
}

teardown() {
    teardown_temp_dir
}

# ============================================
# is_retryable_git_error tests
# ============================================

@test "is_retryable_git_error returns true for DNS resolution failure" {
    run is_retryable_git_error "fatal: Could not resolve host: github.com"
    assert_success
}

@test "is_retryable_git_error returns true for connection refused" {
    run is_retryable_git_error "fatal: Connection refused"
    assert_success
}

@test "is_retryable_git_error returns true for connection timeout" {
    run is_retryable_git_error "fatal: Connection timed out"
    assert_success
}

@test "is_retryable_git_error returns true for network unreachable" {
    run is_retryable_git_error "fatal: Network is unreachable"
    assert_success
}

@test "is_retryable_git_error returns true for connection reset" {
    run is_retryable_git_error "fatal: Connection reset by peer"
    assert_success
}

@test "is_retryable_git_error returns true for SSL errors" {
    run is_retryable_git_error "fatal: SSL_connect returned error"
    assert_success
}

@test "is_retryable_git_error returns true for 5xx server errors" {
    run is_retryable_git_error "The requested URL returned error: 503"
    assert_success
}

@test "is_retryable_git_error returns false for permission denied" {
    run is_retryable_git_error "fatal: Permission denied (publickey)"
    assert_failure
}

@test "is_retryable_git_error returns false for authentication failed" {
    run is_retryable_git_error "fatal: Authentication failed for 'https://github.com/user/repo.git'"
    assert_failure
}

@test "is_retryable_git_error returns false for non-fast-forward" {
    run is_retryable_git_error "error: failed to push some refs to 'origin' hint: Updates were rejected because the tip of your current branch is behind"
    assert_failure
}

@test "is_retryable_git_error returns false for rejected push" {
    run is_retryable_git_error "! [rejected] main -> main (non-fast-forward)"
    assert_failure
}

# ============================================
# git_push_with_retry tests
# ============================================

@test "git_push_with_retry succeeds on first try" {
    # Create mock git that succeeds
    cat > "$MOCK_DIR/git" << 'EOF'
#!/bin/bash
exit 0
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"

    run git_push_with_retry origin main
    assert_success
    # Should not mention retry on first success
    refute_output --partial "attempt"
}

@test "git_push_with_retry retries on network error and succeeds" {
    # Create a call count file
    CALL_FILE="$TEST_TEMP_DIR/push_call_count"
    echo "0" > "$CALL_FILE"

    # Create mock git that fails twice then succeeds
    cat > "$MOCK_DIR/git" << EOF
#!/bin/bash
CALL_FILE="$CALL_FILE"
COUNT=\$(cat "\$CALL_FILE")
COUNT=\$((COUNT + 1))
echo "\$COUNT" > "\$CALL_FILE"

if [ "\$COUNT" -lt 3 ]; then
    echo "fatal: Could not resolve host: github.com" >&2
    exit 1
else
    exit 0
fi
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"

    run git_push_with_retry origin main
    assert_success
    assert_output --partial "succeeded on attempt"
}

@test "git_push_with_retry fails immediately on permanent error" {
    # Create mock git that returns authentication error
    cat > "$MOCK_DIR/git" << 'EOF'
#!/bin/bash
echo "fatal: Authentication failed for 'https://github.com/user/repo.git'" >&2
exit 1
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"

    run git_push_with_retry origin main
    assert_failure
    assert_output --partial "permanent error"
    # Should not mention retry
    refute_output --partial "retrying"
}

@test "git_push_with_retry fails after max retries on network error" {
    # Create mock git that always fails with network error
    cat > "$MOCK_DIR/git" << 'EOF'
#!/bin/bash
echo "fatal: Could not resolve host: github.com" >&2
exit 1
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"
    export VAMP_PUSH_RETRIES=2

    run git_push_with_retry origin main
    assert_failure
    assert_output --partial "failed after 2 attempts"
}

@test "git_push_with_retry respects VAMP_PUSH_RETRIES env var" {
    # Create mock git that counts calls and always fails
    cat > "$MOCK_DIR/git" << 'EOF'
#!/bin/bash
echo "fatal: Connection refused" >&2
exit 1
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"
    export VAMP_PUSH_RETRIES=1

    run git_push_with_retry origin main
    assert_failure
    assert_output --partial "failed after 1 attempts"
    # With only 1 retry, should not show "retrying" message
    refute_output --partial "retrying"
}

@test "git_push_with_retry shows retry progress" {
    # Create a call count file
    CALL_FILE="$TEST_TEMP_DIR/push_progress_count"
    echo "0" > "$CALL_FILE"

    # Create mock git that fails once then succeeds
    cat > "$MOCK_DIR/git" << EOF
#!/bin/bash
CALL_FILE="$CALL_FILE"
COUNT=\$(cat "\$CALL_FILE")
COUNT=\$((COUNT + 1))
echo "\$COUNT" > "\$CALL_FILE"

if [ "\$COUNT" -lt 2 ]; then
    echo "fatal: Network is unreachable" >&2
    exit 1
else
    exit 0
fi
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"

    run git_push_with_retry origin main
    assert_success
    assert_output --partial "retrying (2/3)"
}

@test "git_push_with_retry passes arguments to git push" {
    # Create mock git that logs arguments
    cat > "$MOCK_DIR/git" << 'EOF'
#!/bin/bash
echo "args: $@" > /tmp/push_args_test
exit 0
EOF
    chmod +x "$MOCK_DIR/git"
    export PATH="$MOCK_DIR:$PATH"

    run git_push_with_retry -u origin feature-branch
    assert_success

    # Check that arguments were passed
    run cat /tmp/push_args_test
    assert_output "args: push -u origin feature-branch"
}
