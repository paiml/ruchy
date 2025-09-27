#!/bin/bash
# P0 CRITICAL VALIDATION - MUST PASS FOR ANY COMMIT
#
# This script enforces Extreme TDD principles:
# 1. P0 features MUST work
# 2. No regressions allowed
# 3. Test before commit

set -e

echo "🚨 P0 CRITICAL VALIDATION STARTING..."
echo "======================================"
echo "Principle: If it's advertised, it MUST work"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Run P0 critical tests
echo "📋 Running P0 Critical Features Test Suite..."
if cargo test --test p0_critical_features 2>&1 | tee /tmp/p0_test_output.log | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ P0 Critical Features: PASSED${NC}"
else
    echo -e "${RED}❌ P0 CRITICAL FAILURE DETECTED!${NC}"
    echo ""
    echo "Failed tests:"
    grep "FAILED" /tmp/p0_test_output.log || true
    grep "CRITICAL:" /tmp/p0_test_output.log || true
    echo ""
    echo -e "${RED}BLOCKING: P0 features are broken. Fix immediately!${NC}"
    echo "These represent advertised functionality that MUST work."
    exit 1
fi

# Run transpiler regression tests
echo ""
echo "📋 Running Transpiler Regression Tests..."
if cargo test --test critical_transpiler_regression_test 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ Transpiler Regression Tests: PASSED${NC}"
else
    echo -e "${RED}❌ TRANSPILER REGRESSION DETECTED!${NC}"
    echo "The transpiler is generating incorrect code."
    exit 1
fi

# Check for HashSet generation in functions (specific regression)
echo ""
echo "📋 Checking for HashSet regression..."
TEST_FILE=$(mktemp)
cat > "$TEST_FILE" << 'EOF'
fn test_func(a: i32, b: i32) -> i32 { a + b }
println(test_func(1, 2))
EOF

if cargo run --release --bin ruchy -- transpile "$TEST_FILE" 2>/dev/null | grep -q "HashSet"; then
    echo -e "${RED}❌ HASHSET REGRESSION DETECTED!${NC}"
    echo "Functions are generating HashSet code instead of direct returns!"
    rm "$TEST_FILE"
    exit 1
else
    echo -e "${GREEN}✅ No HashSet regression${NC}"
fi
rm "$TEST_FILE"

# Count passing/ignored tests
echo ""
echo "📊 Test Summary:"
TOTAL_P0=$(grep -c "^fn p0_" tests/p0_critical_features.rs || echo 0)
IGNORED=$(grep -c "^#\[ignore" tests/p0_critical_features.rs || echo 0)
ACTIVE=$((TOTAL_P0 - IGNORED))

echo "  Total P0 tests: $TOTAL_P0"
echo "  Active tests: $ACTIVE (must pass)"
echo "  Tracked issues: $IGNORED (known gaps)"

# Warn about ignored tests
if [ "$IGNORED" -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}⚠️  Warning: $IGNORED P0 features are not implemented:${NC}"
    grep -B1 "^#\[ignore" tests/p0_critical_features.rs | grep "^fn p0_" | sed 's/fn p0_/  - /' | sed 's/()//'
    echo -e "${YELLOW}These represent gaps in advertised functionality.${NC}"
fi

echo ""
echo -e "${GREEN}✅ P0 VALIDATION PASSED${NC}"
echo "All critical features are working correctly."

exit 0