#!/usr/bin/env bash
# Matrix Test Runner - Native Platform Only
#
# Runs all native matrix tests and reports results.
# WASM tests are deferred pending WASM eval() implementation.
#
# Usage: ./tests/matrix/run_matrix_tests.sh

set -euo pipefail

echo "🧪 Matrix Testing Framework - Native Platform"
echo "=============================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

echo "📋 Test Suite Status:"
echo "  ✅ Native Platform: ENABLED"
echo "  ⏸️  WASM Platform: DEFERRED (see tests/e2e/matrix/README.md)"
echo ""

# Run native matrix tests
echo "🦀 Running Native Matrix Tests..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run all native matrix tests
echo "Running Matrix Test 001: Simple Arithmetic..."
if cargo test --test matrix_001_simple_arithmetic_native -- --test-threads=1 2>&1 | tee /tmp/matrix_001_output.txt; then
    echo -e "${GREEN}✅ Matrix 001 PASSED${NC}"
    TEST_001_PASSED=$(grep -o '[0-9]* passed' /tmp/matrix_001_output.txt | awk '{print $1}' || echo "0")
    PASSED_TESTS=$((PASSED_TESTS + TEST_001_PASSED))
    TOTAL_TESTS=$((TOTAL_TESTS + TEST_001_PASSED))
else
    echo -e "${RED}❌ Matrix 001 FAILED${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo ""
echo "Running Matrix Test 002: CSV Processing Workflow..."
if cargo test --test matrix_002_csv_workflow_native -- --test-threads=1 2>&1 | tee /tmp/matrix_002_output.txt; then
    echo -e "${GREEN}✅ Matrix 002 PASSED${NC}"
    TEST_002_PASSED=$(grep -o '[0-9]* passed' /tmp/matrix_002_output.txt | awk '{print $1}' || echo "0")
    PASSED_TESTS=$((PASSED_TESTS + TEST_002_PASSED))
    TOTAL_TESTS=$((TOTAL_TESTS + TEST_002_PASSED))
else
    echo -e "${RED}❌ Matrix 002 FAILED${NC}"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Summary
echo "📊 Matrix Test Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Native Platform:"
echo "  Tests Run: $TOTAL_TESTS"
echo "  Passed: $PASSED_TESTS"
echo "  Failed: $FAILED_TESTS"
echo ""

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo -e "${GREEN}✅ All matrix tests PASSED${NC}"
    echo ""
    echo "🎯 Next Steps:"
    echo "  1. Add more native matrix test suites"
    echo "  2. Implement WASM eval() for WASM matrix tests"
    echo "  3. Build unified matrix test comparison tool"
    exit 0
else
    echo -e "${RED}❌ Some matrix tests FAILED${NC}"
    echo ""
    echo "🔍 Debugging Tips:"
    echo "  1. Check test output above for specific failures"
    echo "  2. Run individual test: cargo test --test matrix_001_simple_arithmetic_native <test_name>"
    echo "  3. Use --nocapture for detailed output: cargo test -- --nocapture"
    exit 1
fi
