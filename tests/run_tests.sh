#!/bin/bash
#
# Run vamp tests using bats-core
#
# Usage:
#   ./tests/run_tests.sh              # Run all tests
#   ./tests/run_tests.sh unit         # Run only unit tests
#   ./tests/run_tests.sh integration  # Run only integration tests
#   ./tests/run_tests.sh --verbose    # Run with verbose output
#   ./tests/run_tests.sh path/to/test.bats  # Run specific test file

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BATS_BIN="$SCRIPT_DIR/bats/bats-core/bin/bats"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

# Check if bats is available
if [ ! -x "$BATS_BIN" ]; then
    echo -e "${RED}Error: bats-core not found${NC}"
    echo "Initialize submodules with: git submodule update --init --recursive"
    exit 1
fi

# Parse arguments
VERBOSE=""
TEST_FILTER=""
SPECIFIC_FILE=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --verbose|-v)
            VERBOSE="--verbose-run"
            shift
            ;;
        --tap)
            VERBOSE="--tap"
            shift
            ;;
        unit)
            TEST_FILTER="unit"
            shift
            ;;
        integration)
            TEST_FILTER="integration"
            shift
            ;;
        *.bats)
            SPECIFIC_FILE="$1"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

echo -e "${CYAN}Running vamp tests${NC}"
echo ""

# Determine which tests to run
if [ -n "$SPECIFIC_FILE" ]; then
    TEST_PATHS="$SPECIFIC_FILE"
elif [ "$TEST_FILTER" = "unit" ]; then
    TEST_PATHS="$SCRIPT_DIR/unit"
elif [ "$TEST_FILTER" = "integration" ]; then
    TEST_PATHS="$SCRIPT_DIR/integration"
else
    # Run all tests
    TEST_PATHS="$SCRIPT_DIR/unit $SCRIPT_DIR/integration"
fi

# Check if there are any test files
test_count=0
for path in $TEST_PATHS; do
    if [ -d "$path" ]; then
        count=$(find "$path" -name "*.bats" 2>/dev/null | wc -l | tr -d ' ')
        test_count=$((test_count + count))
    elif [ -f "$path" ]; then
        test_count=$((test_count + 1))
    fi
done

if [ "$test_count" -eq 0 ]; then
    echo -e "${CYAN}No test files found${NC}"
    exit 0
fi

# Run bats
export PROJECT_ROOT
export VAMP_BIN="$PROJECT_ROOT/bin/vamp"

# shellcheck disable=SC2086
"$BATS_BIN" $VERBOSE $TEST_PATHS

echo ""
echo -e "${GREEN}All tests passed!${NC}"
