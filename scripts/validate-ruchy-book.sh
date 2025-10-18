#!/bin/bash
# Fast ruchy-book validation script
# Runs critical chapter tests in parallel for speed
# Exit immediately on first failure (fail-fast)

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

BOOK_DIR="${RUCHY_BOOK_DIR:-/home/noah/src/ruchy-book}"
PARALLEL_JOBS="${RUCHY_BOOK_JOBS:-4}"

# Check if ruchy-book exists
if [ ! -d "$BOOK_DIR" ]; then
    echo -e "${YELLOW}⚠️  ruchy-book not found at $BOOK_DIR${NC}"
    echo -e "${YELLOW}   Set RUCHY_BOOK_DIR env var or skip with: git commit --no-verify${NC}"
    exit 0  # Don't fail if book doesn't exist
fi

echo -e "${YELLOW}📚 Validating ruchy-book (parallel, fail-fast)${NC}"
echo ""

# Critical chapters that MUST pass (covers all major functionality)
CRITICAL_CHAPTERS=(
    "01"  # Getting Started - Basic functionality
    "02"  # Variables and Types
    "03"  # Control Flow
    "05"  # Functions
)

# Track failures
FAILED_TESTS=()
PASSED_TESTS=()

# Function to run a single chapter test
run_chapter_test() {
    local ch=$1
    local test_dir="$BOOK_DIR/test/ch$ch"

    if [ ! -d "$test_dir" ]; then
        echo -e "${YELLOW}⚠️  Chapter $ch test directory not found${NC}"
        return 0
    fi

    # Find test script
    local test_script="$test_dir/test_all_ch${ch}.sh"

    if [ ! -f "$test_script" ]; then
        echo -e "${YELLOW}⚠️  Test script not found: $test_script${NC}"
        return 0
    fi

    # Make script executable
    chmod +x "$test_script"

    # Run test script
    local script_name=$(basename "$test_script")
    if timeout 120 bash "$test_script" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Ch$ch: $script_name${NC}"
        PASSED_TESTS+=("Ch$ch:$script_name")
    else
        echo -e "${RED}❌ Ch$ch: $script_name FAILED${NC}"
        FAILED_TESTS+=("Ch$ch:$script_name")
        return 1  # Fail fast
    fi

    return 0
}

# Run tests in parallel with fail-fast
export -f run_chapter_test
export BOOK_DIR
export GREEN RED YELLOW NC
export -a PASSED_TESTS FAILED_TESTS

# Use xargs for parallel execution with fail-fast
printf "%s\n" "${CRITICAL_CHAPTERS[@]}" | \
    xargs -P "$PARALLEL_JOBS" -I {} bash -c 'run_chapter_test "$@"' _ {}

EXIT_CODE=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ QUALITY GATE PASSED: ruchy-book validation${NC}"
    echo -e "${GREEN}   ${#CRITICAL_CHAPTERS[@]} critical chapters validated${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo -e "${RED}❌ QUALITY GATE FAILED: ruchy-book validation${NC}"
    echo -e "${RED}   One or more critical tests failed${NC}"
    echo ""
    echo "To bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
