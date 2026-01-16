#!/bin/bash
#
# Tmux mock utilities for vamp tests
#
# Provides mock tmux commands for testing vamp session
# management without requiring a real tmux server.
#

# ============================================
# Tmux Command Mocks
# ============================================

# Setup mock tmux that intercepts tmux commands
# Usage: setup_mock_tmux [scenario]
# Scenarios: no_sessions, with_sessions, running
setup_mock_tmux() {
    local scenario="${1:-no_sessions}"
    local mock_dir="$TEST_TEMP_DIR/mocks"
    mkdir -p "$mock_dir"

    # Define mock sessions based on scenario
    local sessions_file="$mock_dir/tmux_sessions"
    case "$scenario" in
        no_sessions)
            echo "" > "$sessions_file"
            ;;
        with_sessions)
            cat > "$sessions_file" << 'EOF'
vamp-project1:2:1
vamp-project2:2:0
vamp-test:3:1
EOF
            ;;
        running)
            cat > "$sessions_file" << 'EOF'
vamp-myproject:2:1
EOF
            ;;
    esac

    # Create the mock tmux script
    cat > "$mock_dir/tmux" << 'MOCKSCRIPT'
#!/bin/bash
# Mock tmux command for testing

MOCK_DIR="$TEST_TEMP_DIR/mocks"
MOCK_LOG="$MOCK_DIR/tmux.log"
SESSIONS_FILE="$MOCK_DIR/tmux_sessions"

# Log the call
echo "tmux $*" >> "$MOCK_LOG"

case "$1" in
    has-session)
        # Check if session exists
        local session_name="${3:-}"  # -t session_name
        if [ -f "$SESSIONS_FILE" ] && grep -q "^${session_name}:" "$SESSIONS_FILE"; then
            exit 0
        else
            exit 1
        fi
        ;;
    list-sessions)
        if [ -f "$SESSIONS_FILE" ] && [ -s "$SESSIONS_FILE" ]; then
            cat "$SESSIONS_FILE"
        else
            echo "no server running on /tmp/tmux-1000/default"
            exit 1
        fi
        ;;
    new-session)
        # Extract session name from args
        local session_name=""
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -s) session_name="$2"; shift 2 ;;
                -d) shift ;;  # detached
                *) shift ;;
            esac
        done
        if [ -n "$session_name" ]; then
            echo "${session_name}:1:1" >> "$SESSIONS_FILE"
        fi
        echo "Created session: $session_name"
        ;;
    kill-session)
        local session_name=""
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -t) session_name="$2"; shift 2 ;;
                *) shift ;;
            esac
        done
        if [ -n "$session_name" ] && [ -f "$SESSIONS_FILE" ]; then
            # Remove session from list
            grep -v "^${session_name}:" "$SESSIONS_FILE" > "$SESSIONS_FILE.tmp"
            mv "$SESSIONS_FILE.tmp" "$SESSIONS_FILE"
            echo "Killed session: $session_name"
        fi
        ;;
    attach-session|attach)
        local session_name=""
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -t) session_name="$2"; shift 2 ;;
                *) shift ;;
            esac
        done
        if [ -f "$SESSIONS_FILE" ] && grep -q "^${session_name}:" "$SESSIONS_FILE"; then
            echo "Attached to session: $session_name"
        else
            echo "can't find session: $session_name"
            exit 1
        fi
        ;;
    split-window|select-pane|select-layout|send-keys|set-option|resize-pane|new-window)
        # These commands generally succeed silently
        echo "tmux $1: OK"
        ;;
    display-message)
        # Handle format strings
        shift
        while [[ $# -gt 0 ]]; do
            case "$1" in
                -p) shift ;;
                -t) shift 2 ;;
                *)
                    # Return mock values for common format strings
                    case "$1" in
                        *session_name*) echo "vamp-test" ;;
                        *window_index*) echo "0" ;;
                        *pane_index*) echo "0" ;;
                        *) echo "$1" ;;
                    esac
                    shift
                    ;;
            esac
        done
        ;;
    *)
        echo "tmux mock: unhandled command '$1'"
        ;;
esac
MOCKSCRIPT

    chmod +x "$mock_dir/tmux"
    export PATH="$mock_dir:$PATH"
}

# Get calls made to mock tmux
get_tmux_calls() {
    local mock_log="$TEST_TEMP_DIR/mocks/tmux.log"
    if [ -f "$mock_log" ]; then
        cat "$mock_log"
    fi
}

# Assert tmux was called with specific arguments
assert_tmux_called_with() {
    local expected="$1"
    local calls
    calls=$(get_tmux_calls)

    if ! echo "$calls" | grep -qF "$expected"; then
        echo "Expected tmux to be called with: $expected"
        echo "Actual calls:"
        echo "$calls"
        return 1
    fi
}

# Assert tmux was NOT called with specific arguments
refute_tmux_called_with() {
    local unexpected="$1"
    local calls
    calls=$(get_tmux_calls)

    if echo "$calls" | grep -qF "$unexpected"; then
        echo "Expected tmux NOT to be called with: $unexpected"
        echo "But found in calls:"
        echo "$calls"
        return 1
    fi
}

# Clear tmux call log
clear_tmux_calls() {
    local mock_log="$TEST_TEMP_DIR/mocks/tmux.log"
    rm -f "$mock_log"
}

# Count how many times tmux was called with specific command
count_tmux_calls() {
    local pattern="$1"
    local calls
    calls=$(get_tmux_calls)

    echo "$calls" | grep -c "$pattern" || echo "0"
}

# ============================================
# Session Simulation Helpers
# ============================================

# Add a mock session to the session list
add_mock_session() {
    local session_name="$1"
    local windows="${2:-2}"
    local attached="${3:-0}"

    local sessions_file="$TEST_TEMP_DIR/mocks/tmux_sessions"
    echo "${session_name}:${windows}:${attached}" >> "$sessions_file"
}

# Remove a mock session from the session list
remove_mock_session() {
    local session_name="$1"
    local sessions_file="$TEST_TEMP_DIR/mocks/tmux_sessions"

    if [ -f "$sessions_file" ]; then
        grep -v "^${session_name}:" "$sessions_file" > "$sessions_file.tmp"
        mv "$sessions_file.tmp" "$sessions_file"
    fi
}

# List current mock sessions
list_mock_sessions() {
    local sessions_file="$TEST_TEMP_DIR/mocks/tmux_sessions"
    if [ -f "$sessions_file" ]; then
        cat "$sessions_file"
    fi
}

# Check if a mock session exists
mock_session_exists() {
    local session_name="$1"
    local sessions_file="$TEST_TEMP_DIR/mocks/tmux_sessions"

    if [ -f "$sessions_file" ] && grep -q "^${session_name}:" "$sessions_file"; then
        return 0
    else
        return 1
    fi
}
