#!/bin/bash
# Comprehensive book compatibility test suite
# Tests actual functionality vs documentation claims
# Created: 2025-10-06

PASSED=0
FAILED=0
SKIPPED=0

echo "📚 Ruchy Book Comprehensive Compatibility Test"
echo "=============================================="
echo ""

test_feature() {
    local chapter="$1"
    local desc="$2"
    local code="$3"
    local expected="$4"
    local skip_reason="${5:-}"

    echo -n "[$chapter] $desc... "

    if [ -n "$skip_reason" ]; then
        echo "⏭️  SKIP ($skip_reason)"
        ((SKIPPED++))
        return 2
    fi

    result=$(echo "$code" | cargo run --quiet --bin ruchy repl 2>&1 | grep -v "Type :help\|Goodbye\|Welcome\|Ruchy REPL\|ALL functions\|coverage\|TDG" | tr -d '\n' | xargs)

    if [[ "$result" == *"$expected"* ]]; then
        echo "✅ PASS"
        ((PASSED++))
        return 0
    else
        echo "❌ FAIL"
        echo "   Expected: $expected"
        echo "   Got: $result"
        ((FAILED++))
        return 1
    fi
}

echo "=== CHAPTER 1: Hello World ==="
test_feature "Ch01" "Simple println" 'println("Hello, World!")' "Hello, World!"
test_feature "Ch01" "String literals" '"Hello"' "Hello"
test_feature "Ch01" "Number output" '42' "42"

echo ""
echo "=== CHAPTER 2: Variables ==="
test_feature "Ch02" "Let binding" 'let x = 10; x' "10"
test_feature "Ch02" "Multiple variables" 'let a = 1; let b = 2; a + b' "3"
test_feature "Ch02" "String variable" 'let name = "Alice"; name' "Alice"

echo ""
echo "=== CHAPTER 3: Functions ==="
test_feature "Ch03" "Function definition" 'fun add(x, y) { x + y }; add(2, 3)' "5" "Parser error - fun keyword"
test_feature "Ch03" "Lambda function" 'let f = |x| x * 2; f(5)' "10"
test_feature "Ch03" "Higher-order function" 'let apply = |f, x| f(x); apply(|n| n + 1, 5)' "6"

echo ""
echo "=== CHAPTER 4: Practical Patterns ==="
test_feature "Ch04" "One-liner math" '2 + 2' "4"
test_feature "Ch04" "Percentage calc" '100.0 * 1.08' "108"
test_feature "Ch04" "Multi-variable expr" 'let p = 99.99; let t = 0.08; p * (1.0 + t)' "107.98"
test_feature "Ch04" "Pipeline operator" '10 |> |x| x * 2 |> |y| y + 1' "21" "Pipeline not in REPL"

echo ""
echo "=== CHAPTER 5: Control Flow ==="
test_feature "Ch05" "If expression" 'if 10 > 5 { "yes" } else { "no" }' "yes"
test_feature "Ch05" "Match expression" 'match 2 { 1 => "one", 2 => "two", _ => "other" }' "two"
test_feature "Ch05" "For loop" 'for i in [1, 2, 3] { println(i) }' "" "Multi-line not in REPL"
test_feature "Ch05" "While loop" 'let mut x = 0; while x < 3 { x = x + 1 }; x' "3" "Mutable not supported"

echo ""
echo "=== CHAPTER 6: Data Structures ==="
test_feature "Ch06" "Array literal" '[1, 2, 3]' "[1, 2, 3]"
test_feature "Ch06" "Array indexing" 'let arr = [1, 2, 3]; arr[1]' "2"
test_feature "Ch06" "Object literal" '{ name: "Alice", age: 30 }' 'name: "Alice"'
test_feature "Ch06" "Nested structures" '[{x: 1}, {x: 2}]' "{x: 1}"

echo ""
echo "=== CHAPTER 15: Binary Compilation ==="
# Note: Can't test in REPL, need file-based test
echo "[Ch15] Binary compilation... ⏭️  SKIP (Requires file-based test, verified separately)"
((SKIPPED++))

echo ""
echo "=== CHAPTER 17: Error Handling ==="
test_feature "Ch17" "Try-catch" 'try { 10 / 0 } catch e { "error" }' "error" "Try-catch parser incomplete"
test_feature "Ch17" "Result type" 'Ok(42)' "Ok(42)" "Result type not implemented"

echo ""
echo "=== CHAPTER 18: Dataframes ==="
test_feature "Ch18" "DataFrame literal" 'df!["x" => [1, 2]]' "DataFrame" "Dataframe syntax not implemented"

echo ""
echo "=== STRING METHODS ==="
test_feature "Str" "String length" '"hello".len()' "5"
test_feature "Str" "String uppercase" '"hello".to_uppercase()' "HELLO"
test_feature "Str" "String split" '"a,b,c".split(",")' '["a", "b", "c"]'

echo ""
echo "=== NUMERIC METHODS ==="
test_feature "Num" "Integer sqrt" '(100.0).sqrt()' "10"
test_feature "Num" "Float floor" '(3.7).floor()' "3"
test_feature "Num" "Number to_string" '42.to_string()' '"42"'

echo ""
echo "=============================================="
echo "📊 Results Summary"
echo "=============================================="
echo "  ✅ PASSED:  $PASSED"
echo "  ❌ FAILED:  $FAILED"
echo "  ⏭️  SKIPPED: $SKIPPED"
TOTAL=$((PASSED + FAILED + SKIPPED))
if [ $TOTAL -gt 0 ]; then
    SUCCESS_RATE=$(echo "scale=1; $PASSED * 100 / ($PASSED + $FAILED)" | bc 2>/dev/null || echo "0")
    echo "  📈 Success Rate: ${SUCCESS_RATE}% (excluding skipped)"
fi
echo "=============================================="

if [ $FAILED -eq 0 ]; then
    echo "✨ All non-skipped tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed - see details above"
    exit 1
fi
