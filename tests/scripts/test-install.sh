#!/bin/bash
#
# Test vamp installation (install.sh)
#
# Tests:
# 1. Fresh install - no existing vamp
# 2. Partial install - some files exist
# 3. Upgrade - existing vamp install
#
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VAMP_DIR="${VAMP_DIR:-$HOME/vamp}"

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

cleanup() {
    # Clean up any previous installation
    rm -rf "$HOME/.local/bin/vamp"
    rm -rf "$HOME/.local/share/vamp"
    rm -rf "$HOME/.config/vamp"
}

# ============================================
# Test 1: Fresh Install
# ============================================
section "Test 1: Fresh Install"

cleanup

# Run installer (non-interactive)
cd "$VAMP_DIR"
echo "n" | ./install.sh 2>&1 || true  # Skip the "run vamp setup?" prompt

# Verify installation
if [ -x "$HOME/.local/bin/vamp" ]; then
    pass "vamp binary installed"
else
    fail "vamp binary not installed"
fi

if [ -d "$HOME/.local/share/vamp" ]; then
    pass "vamp share directory created"
else
    fail "vamp share directory not created"
fi

if [ -f "$HOME/.config/vamp/config" ]; then
    pass "vamp config created"
else
    fail "vamp config not created"
fi

# Check vamp is executable
if "$HOME/.local/bin/vamp" --version >/dev/null 2>&1; then
    pass "vamp --version works"
else
    fail "vamp --version failed"
fi

# ============================================
# Test 2: Partial Install (re-run)
# ============================================
section "Test 2: Re-installation"

# Run installer again
cd "$VAMP_DIR"
echo "n" | ./install.sh 2>&1 || true

# Should still work
if "$HOME/.local/bin/vamp" --version >/dev/null 2>&1; then
    pass "vamp works after re-install"
else
    fail "vamp broken after re-install"
fi

# ============================================
# Test 3: Verify Commands
# ============================================
section "Test 3: Verify Commands"

export PATH="$HOME/.local/bin:$PATH"

if vamp --help >/dev/null 2>&1; then
    pass "vamp --help works"
else
    fail "vamp --help failed"
fi

if vamp list >/dev/null 2>&1; then
    pass "vamp list works"
else
    fail "vamp list failed"
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
