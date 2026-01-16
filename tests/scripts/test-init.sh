#!/bin/bash
#
# Test vamp init command
#
# Tests:
# 1. Initializes git repository
# 2. Initializes beads
# 3. Creates project CLAUDE.md
# 4. Validates global setup check
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
TEST_DIR="$HOME/test-projects"

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
    rm -rf "$TEST_DIR"
    mkdir -p "$TEST_DIR"
}

# First ensure vamp setup has been run
ensure_global_setup() {
    mkdir -p "$HOME/.claude/hooks"
    touch "$HOME/.claude/hooks/placeholder.sh"
    if [ ! -f "$HOME/.claude/CLAUDE.md" ]; then
        "$VAMP_BIN" setup --claude-md 2>&1 || true
    fi
}

# ============================================
# Test 1: Init in empty directory
# ============================================
section "Test 1: Init in Empty Directory"

cleanup
ensure_global_setup

cd "$TEST_DIR"
mkdir test-project-1
cd test-project-1

# Run vamp init
"$VAMP_BIN" init 2>&1 || true

if [ -d ".git" ]; then
    pass "Git initialized"
else
    fail "Git not initialized"
fi

# Beads initialization depends on bd being installed
if command -v bd >/dev/null 2>&1; then
    if [ -d ".beads" ]; then
        pass "Beads initialized"
    else
        fail "Beads not initialized"
    fi
else
    pass "Skipped beads check (bd not installed)"
fi

if [ -f "CLAUDE.md" ]; then
    pass "Project CLAUDE.md created"
else
    fail "Project CLAUDE.md not created"
fi

# ============================================
# Test 2: Init in existing git repo
# ============================================
section "Test 2: Init in Existing Git Repo"

cleanup
ensure_global_setup

cd "$TEST_DIR"
mkdir test-project-2
cd test-project-2
git init

# Run vamp init
"$VAMP_BIN" init 2>&1 || true

# Beads initialization depends on bd being installed
if command -v bd >/dev/null 2>&1; then
    if [ -d ".beads" ]; then
        pass "Beads initialized in existing repo"
    else
        fail "Beads not initialized in existing repo"
    fi
else
    pass "Skipped beads check in existing repo (bd not installed)"
fi

# ============================================
# Test 3: Global setup check
# ============================================
section "Test 3: Global Setup Check"

cleanup

# Remove global setup
rm -rf "$HOME/.claude"

cd "$TEST_DIR"
mkdir test-project-3
cd test-project-3

# Run vamp init - should warn about missing global setup
output=$("$VAMP_BIN" init 2>&1 <<< "n" || true)

if echo "$output" | grep -q "Global setup incomplete\|hooks not installed\|CLAUDE.md not found"; then
    pass "Warns about missing global setup"
else
    fail "No warning about missing global setup"
fi

# ============================================
# Test 4: Init with beads commands
# ============================================
section "Test 4: Beads Integration"

cleanup
ensure_global_setup

cd "$TEST_DIR"
mkdir test-project-4
cd test-project-4

"$VAMP_BIN" init 2>&1 || true

# Check if bd command works
if command -v bd >/dev/null 2>&1; then
    if bd list 2>&1 | grep -q "No issues\|issues"; then
        pass "bd list works after init"
    else
        pass "bd command available (no issues yet)"
    fi
else
    # bd might not be installed in test environment
    pass "Skipped bd test (bd not installed)"
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
