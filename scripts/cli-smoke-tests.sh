#!/bin/bash
# CLI Smoke Tests - Fast validation of CLI invocation patterns
# Part of CLI-UNIFY-004: Pre-commit Hook for CLI Regression Prevention
#
# **Purpose**: Prevent CLI UX regressions by validating all invocation patterns
# **Target**: Complete in <30 seconds total
# **Tests**: 5 critical smoke tests
#
# **bashrs Exception**: DET002 (timestamps) - ACCEPTABLE
# **Rationale**: Performance testing requires timing measurements for validation
#   - Timestamps used to verify CLI speed requirements (<1s eval, <2s interpret)
#   - Testing scripts need timing data for regression detection
#   - Not used in build/deployment pipelines (test-only script)
#
# **bashrs Exception**: SEC001 (eval) - SAFE
# **Rationale**: All test commands are internally controlled (not user input)
#   - Commands defined in this file only, no external input
#   - Used for test execution abstraction, not dynamic code execution
#
# **Reference**: docs/unified-deno-cli-spec.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track test results
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=5

echo "🔍 CLI Smoke Tests (CLI-UNIFY-004)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Helper function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    local expected_pattern="$3"
    local max_duration="$4"

    echo -n "  [$(($TESTS_PASSED + $TESTS_FAILED + 1))/$TOTAL_TESTS] $test_name... "

    # Run test with timeout
    local start_time
    start_time="$(date +%s)"
    local output
    local exit_code

    if output=$(eval "$test_command" 2>&1); then
        exit_code=0
    else
        exit_code=$?
    fi

    local end_time
    end_time="$(date +%s)"
    local duration
    duration=$(($end_time - $start_time))

    # Check if test passed
    if [[ "$exit_code" -eq 0 ]] && echo "$output" | grep -q "$expected_pattern"; then
        if [[ "$duration" -le "$max_duration" ]]; then
            echo -e "${GREEN}✅${NC} (${duration}s)"
            TESTS_PASSED=$(($TESTS_PASSED + 1))
            return 0
        else
            echo -e "${RED}❌${NC} (too slow: ${duration}s > ${max_duration}s)"
            TESTS_FAILED=$(($TESTS_FAILED + 1))
            return 1
        fi
    else
        echo -e "${RED}❌${NC}"
        echo "     Expected: $expected_pattern"
        echo "     Got: $(printf '%s\n' "$output" | head -1)"
        TESTS_FAILED=$(($TESTS_FAILED + 1))
        return 1
    fi
}

# Create temp test file for smoke tests
TEMP_FILE="$(mktemp --suffix=.ruchy)"
echo 'println(42)' > "$TEMP_FILE"
trap "rm -f \"$TEMP_FILE\"" EXIT

# ============================================================================
# SMOKE TEST 1: No args opens REPL (not help)
# ============================================================================
run_test \
    "ruchy (no args) → REPL" \
    "echo ':quit' | cargo run --release --quiet --bin ruchy 2>/dev/null | head -5" \
    "Ruchy\|Welcome\|>>" \
    3

# ============================================================================
# SMOKE TEST 2: Run command interprets (fast, <2s)
# ============================================================================
run_test \
    "ruchy run → interpret" \
    "cargo run --release --quiet --bin ruchy run $TEMP_FILE 2>/dev/null" \
    "42" \
    2

# ============================================================================
# SMOKE TEST 3: Eval flag works (<1s)
# ============================================================================
run_test \
    "ruchy -e → evaluate" \
    "cargo run --release --quiet --bin ruchy -- -e 'println(1 + 1)' 2>/dev/null" \
    "2" \
    1

# ============================================================================
# SMOKE TEST 4: Direct file execution (<2s)
# ============================================================================
run_test \
    "ruchy file → execute" \
    "cargo run --release --quiet --bin ruchy $TEMP_FILE 2>/dev/null" \
    "42" \
    2

# ============================================================================
# SMOKE TEST 5: Compile creates binary (can be slow)
# ============================================================================
COMPILE_OUTPUT="$(mktemp)"
run_test \
    "ruchy compile → binary" \
    "cargo run --release --quiet --bin ruchy compile $TEMP_FILE --output $COMPILE_OUTPUT 2>/dev/null && test -f $COMPILE_OUTPUT" \
    "" \
    60  # Compile can take up to 60s

rm -f "$COMPILE_OUTPUT"

# ============================================================================
# REPORT RESULTS
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ "$TESTS_FAILED" -eq 0 ]]; then
    echo -e "${GREEN}✅ All $TOTAL_TESTS smoke tests passed${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}❌ $TESTS_FAILED/$TOTAL_TESTS smoke tests failed${NC}"
    echo ""
    echo "Fix CLI regressions before committing:"
    echo "  - Check that 'ruchy' opens REPL (not help)"
    echo "  - Verify 'ruchy run' interprets (not compiles)"
    echo "  - Test 'ruchy -e' evaluates quickly"
    echo ""
    echo "To bypass (NOT RECOMMENDED): git commit --no-verify"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
