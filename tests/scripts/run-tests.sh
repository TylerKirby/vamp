#!/bin/bash
#
# Orchestration script for vamp setup flow tests
#
# Runs all test scripts in order:
# 1. test-install.sh - Tests install.sh
# 2. test-setup.sh   - Tests vamp setup
# 3. test-init.sh    - Tests vamp init
# 4. test-doctor.sh  - Tests vamp doctor
#
# Usage:
#   ./run-tests.sh              # Run all tests
#   ./run-tests.sh install      # Run only install tests
#   ./run-tests.sh setup init   # Run specific tests
#
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VAMP_DIR="${VAMP_DIR:-$HOME/vamp}"

# Export paths for test scripts
export VAMP_BIN="$HOME/.local/bin/vamp"
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:$PATH"

# Track results
declare -A results

run_test() {
    local name=$1
    local script="$SCRIPT_DIR/test-${name}.sh"

    if [ ! -f "$script" ]; then
        echo -e "${RED}Test script not found: $script${NC}"
        results[$name]="SKIP"
        return 1
    fi

    echo -e "\n${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${CYAN}  Running: test-${name}.sh${NC}"
    echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

    if bash "$script"; then
        results[$name]="PASS"
        return 0
    else
        results[$name]="FAIL"
        return 1
    fi
}

print_summary() {
    echo -e "\n${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${CYAN}  TEST SUMMARY${NC}"
    echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

    local pass=0
    local fail=0
    local skip=0

    for name in install setup init doctor; do
        local status="${results[$name]:-SKIP}"
        case "$status" in
            PASS)
                echo -e "  ${GREEN}✓${NC} test-${name}.sh"
                ((pass++))
                ;;
            FAIL)
                echo -e "  ${RED}✗${NC} test-${name}.sh"
                ((fail++))
                ;;
            SKIP)
                echo -e "  ${YELLOW}○${NC} test-${name}.sh (skipped)"
                ((skip++))
                ;;
        esac
    done

    echo -e "\n  ${GREEN}Passed:${NC}  $pass"
    echo -e "  ${RED}Failed:${NC}  $fail"
    echo -e "  ${YELLOW}Skipped:${NC} $skip"
    echo ""

    if [ "$fail" -gt 0 ]; then
        return 1
    fi
    return 0
}

# Main
echo -e "${BOLD}${CYAN}"
echo "╔═══════════════════════════════════════════════════════╗"
echo "║            VAMP SETUP FLOW TEST SUITE                 ║"
echo "╚═══════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Determine which tests to run
if [ $# -eq 0 ]; then
    tests_to_run=(install setup init doctor)
else
    tests_to_run=("$@")
fi

# Run tests
failed=0
for test_name in "${tests_to_run[@]}"; do
    if ! run_test "$test_name"; then
        ((failed++))
    fi
done

# Print summary
if ! print_summary; then
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi

echo -e "${GREEN}All tests passed!${NC}"
