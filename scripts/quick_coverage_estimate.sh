#!/bin/bash
# Quick coverage estimation script that doesn't timeout
# Uses test density and heuristics to estimate coverage

echo "=== Quick Coverage Estimation ==="
echo "Date: $(date)"
echo ""

# Count metrics
TOTAL_LINES=$(find src -name '*.rs' -type f | xargs wc -l | tail -1 | awk '{print $1}')
TOTAL_FILES=$(find src -name '*.rs' -type f | wc -l)
TEST_FUNCTIONS=$(grep -r '#\[test\]' src --include='*.rs' 2>/dev/null | wc -l)
TEST_MODULES=$(find src -name '*.rs' -exec grep -l '#\[test\]' {} \; 2>/dev/null | wc -l)
UNTESTED_MODULES=$(find src -name '*.rs' -type f ! -exec grep -l '#\[test\]' {} \; 2>/dev/null | wc -l)
DOCTESTS=$(grep -r '/// ```' src --include='*.rs' 2>/dev/null | wc -l)
PROPERTY_TESTS=$(grep -r 'proptest!' src --include='*.rs' 2>/dev/null | wc -l)

echo "Source Metrics:"
echo "  Total lines: $TOTAL_LINES"
echo "  Total files: $TOTAL_FILES"
echo ""

echo "Test Metrics:"
echo "  Test functions: $TEST_FUNCTIONS"
echo "  Files with tests: $TEST_MODULES"
echo "  Files without tests: $UNTESTED_MODULES"
echo "  Doctests: $DOCTESTS"
echo "  Property tests: $PROPERTY_TESTS"
echo ""

# Calculate estimates
if [ $TOTAL_FILES -gt 0 ]; then
    TEST_DENSITY=$(echo "scale=2; $TEST_FUNCTIONS / $TOTAL_FILES" | bc)
    MODULE_COVERAGE=$(echo "scale=2; ($TEST_MODULES * 100) / $TOTAL_FILES" | bc)

    # Heuristic: each test covers ~25 lines, each doctest ~10 lines
    ESTIMATED_LINES_COVERED=$(echo "($TEST_FUNCTIONS * 25) + ($DOCTESTS * 10) + ($PROPERTY_TESTS * 100)" | bc)
    ESTIMATED_COVERAGE=$(echo "scale=2; ($ESTIMATED_LINES_COVERED * 100) / $TOTAL_LINES" | bc)

    # Cap at 100%
    if (( $(echo "$ESTIMATED_COVERAGE > 100" | bc -l) )); then
        ESTIMATED_COVERAGE="100.00"
    fi

    echo "Coverage Estimates:"
    echo "  Test density: $TEST_DENSITY tests/file"
    echo "  Module coverage: $MODULE_COVERAGE% of files have tests"
    echo "  Estimated line coverage: $ESTIMATED_COVERAGE%"
    echo ""
fi

# Show top untested files by size
echo "Top 10 untested files (by lines):"
for file in $(find src -name '*.rs' -type f ! -exec grep -l '#\[test\]' {} \; 2>/dev/null); do
    wc -l "$file"
done | sort -rn | head -10

echo ""
echo "=== End Report ==="