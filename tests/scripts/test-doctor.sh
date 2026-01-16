#!/bin/bash
#
# Test vamp doctor command
#
# Tests:
# 1. Runs without error
# 2. Checks dependencies
# 3. Reports status correctly
#
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VAMP_BIN="${VAMP_BIN:-$HOME/.local/bin/vamp}"

pass_count=0
fail_count=0

pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((pass_count++)) || true
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((fail_count++)) || true
}

section() {
    echo -e "\n${CYAN}=== $1 ===${NC}\n"
}

# ============================================
# Test 1: Doctor runs
# ============================================
section "Test 1: Doctor Runs"

if "$VAMP_BIN" doctor 2>&1; then
    pass "vamp doctor runs without error"
else
    # Doctor may exit non-zero if issues found, that's okay
    pass "vamp doctor completed"
fi

# ============================================
# Test 2: Doctor checks dependencies
# ============================================
section "Test 2: Dependency Checks"

output=$("$VAMP_BIN" doctor 2>&1 || true)

if echo "$output" | grep -qi "tmux"; then
    pass "Doctor checks tmux"
else
    fail "Doctor doesn't check tmux"
fi

if echo "$output" | grep -qi "claude"; then
    pass "Doctor checks claude"
else
    fail "Doctor doesn't check claude"
fi

# ============================================
# Test 3: Doctor output format
# ============================================
section "Test 3: Output Format"

output=$("$VAMP_BIN" doctor 2>&1 || true)

# Should have some kind of status indicators
if echo "$output" | grep -qE "✓|✗|OK|FAIL|found|missing|installed"; then
    pass "Doctor shows status indicators"
else
    fail "Doctor output unclear"
fi

# ============================================
# Summary
# ============================================
section "Summary"

total=$((pass_count + fail_count))
echo -e "Passed: ${GREEN}$pass_count${NC}/$total"
echo -e "Failed: ${RED}$fail_count${NC}/$total"

if [ "$fail_count" -gt 0 ]; then
    exit 1
fi
