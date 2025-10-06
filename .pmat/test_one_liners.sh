#!/bin/bash
# Test one-liners from ch04-01 to verify actual current state
# Created: 2025-10-06

PASSED=0
FAILED=0

test_one_liner() {
    local desc="$1"
    local code="$2"
    local expected="$3"

    echo -n "Testing: $desc... "

    result=$(echo "$code" | cargo run --quiet --bin ruchy repl 2>&1 | grep -v "Type :help\|Goodbye" | tr -d '\n' | xargs)

    if [[ "$result" == *"$expected"* ]]; then
        echo "✅ PASS"
        ((PASSED++))
        return 0
    else
        echo "❌ FAIL (expected: $expected, got: $result)"
        ((FAILED++))
        return 1
    fi
}

echo "📊 Ruchy One-Liner Compatibility Test"
echo "======================================"
echo ""

# Basic arithmetic
test_one_liner "Simple addition" "2 + 2" "4"
test_one_liner "Percentage calc" "100.0 * 1.08" "108"
test_one_liner "Compound interest" "1000.0 * 1.05 * 1.05" "1102.5"

# Multi-variable
test_one_liner "Multi-variable" "let price = 99.99; let tax = 0.08; price * (1.0 + tax)" "107.98"

# Boolean
test_one_liner "Comparison" "10 > 5" "true"
test_one_liner "Boolean AND" "true && false" "false"
test_one_liner "Boolean OR" "true || false" "true"
test_one_liner "Conditional" 'if 100 > 50 { "expensive" } else { "cheap" }' "expensive"

# String
test_one_liner "String concat" '"Hello " + "World"' "Hello World"
test_one_liner "String interpolation" 'let name = "Ruchy"; "Hello " + name' "Hello Ruchy"

# Math methods
test_one_liner "sqrt method" "let x = 10.0; let y = 20.0; (x * x + y * y).sqrt()" "22.36"

echo ""
echo "======================================"
echo "Results:"
echo "  ✅ PASSED: $PASSED"
echo "  ❌ FAILED: $FAILED"
echo "  Success Rate: $(echo "scale=1; $PASSED * 100 / ($PASSED + $FAILED)" | bc)%"
echo "======================================"

if [ $FAILED -eq 0 ]; then
    echo "✨ All tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi
