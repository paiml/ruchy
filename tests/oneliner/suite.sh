#!/usr/bin/env bash
# One-liner Test Suite for Ruchy
# Tests common Unix pipeline and data transformation use cases

set -uo pipefail

# Use debug build by default, release if available
RUCHY="${CARGO_TARGET_DIR:-target}/debug/ruchy"
if [[ -f "${CARGO_TARGET_DIR:-target}/release/ruchy" ]]; then
    RUCHY="${CARGO_TARGET_DIR:-target}/release/ruchy"
fi

FAILURES=0
PASSES=0

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Test helper function
test_oneliner() {
    local name="$1"
    local input="$2"
    local expected="$3"
    local cmd="$4"
    
    echo -n "Testing $name... "
    
    # Run the command and capture output
    if result=$(echo "$input" | $RUCHY -e "$cmd" 2>&1); then
        if [[ "$result" == "$expected" ]]; then
            echo -e "${GREEN}✓${NC}"
            ((PASSES++))
        else
            echo -e "${RED}✗${NC}"
            echo "  Command:  $cmd"
            echo "  Input:    $input"
            echo "  Expected: $expected"
            echo "  Got:      $result"
            ((FAILURES++))
        fi
    else
        echo -e "${RED}✗ (command failed)${NC}"
        echo "  Command: $cmd"
        echo "  Error:   $result"
        ((FAILURES++))
    fi
}

# Test without input (pure evaluation)
test_eval() {
    local name="$1"
    local cmd="$2"
    local expected="$3"
    
    echo -n "Testing $name... "
    
    if result=$($RUCHY -e "$cmd" 2>&1); then
        if [[ "$result" == "$expected" ]]; then
            echo -e "${GREEN}✓${NC}"
            ((PASSES++))
        else
            echo -e "${RED}✗${NC}"
            echo "  Command:  $cmd"
            echo "  Expected: $expected"
            echo "  Got:      $result"
            ((FAILURES++))
        fi
    else
        echo -e "${RED}✗ (command failed)${NC}"
        echo "  Command: $cmd"
        echo "  Error:   $result"
        ((FAILURES++))
    fi
}

echo "=== Ruchy One-liner Test Suite ==="
echo

# Basic arithmetic
echo "--- Basic Arithmetic ---"
test_eval "addition" "2 + 2" "4"
test_eval "multiplication" "6 * 7" "42"
test_eval "division" "100 / 4" "25"
test_eval "complex expression" "(10 + 5) * 3" "45"

# String operations
echo
echo "--- String Operations ---"
test_eval "string concat" '"Hello, " + "World!"' '"Hello, World!"'
test_eval "string uppercase" '"hello".to_upper()' '"HELLO"'
test_eval "string lowercase" '"WORLD".to_lower()' '"world"'
test_eval "string length" '"Ruchy".len()' "5"
test_eval "string trim" '"  spaces  ".trim()' '"spaces"'

# String interpolation
echo
echo "--- String Interpolation ---"
test_eval "basic interpolation" 'f"2 + 2 = {2 + 2}"' '"2 + 2 = 4"'
test_eval "expression interpolation" 'let x = 10; f"x * 2 = {x * 2}"' '"x * 2 = 20"'

# List operations
echo
echo "--- List Operations ---"
test_eval "list literal" "[1, 2, 3]" "[1, 2, 3]"
test_eval "list map" "[1, 2, 3].map(|x| x * 2)" "[2, 4, 6]"
test_eval "list filter" "[1, 2, 3, 4, 5].filter(|x| x > 3)" "[4, 5]"
test_eval "list reduce" "[1, 2, 3, 4].reduce(0, |acc, x| acc + x)" "10"
test_eval "list length" "[1, 2, 3, 4, 5].len()" "5"

# Mathematical functions
echo
echo "--- Mathematical Functions ---"
test_eval "square root" "16.0.sqrt()" "4"
test_eval "absolute value" "(-42).abs()" "42"
test_eval "power" "2.0.pow(3.0)" "8"
test_eval "floor" "3.7.floor()" "3"
test_eval "ceiling" "3.2.ceil()" "4"

# Boolean logic
echo
echo "--- Boolean Logic ---"
test_eval "and operation" "true && false" "false"
test_eval "or operation" "true || false" "true"
test_eval "not operation" "!true" "false"
test_eval "comparison gt" "10 > 5" "true"
test_eval "comparison eq" "42 == 42" "true"

# Control flow
echo
echo "--- Control Flow ---"
test_eval "if expression" 'if 10 > 5 { "yes" } else { "no" }' '"yes"'
test_eval "match expression" 'match 2 { 1 => "one", 2 => "two", _ => "other" }' '"two"'

# Lambdas and functions
echo
echo "--- Lambdas and Functions ---"
test_eval "lambda definition" 'let f = |x| x * 2; f(21)' "42"
test_eval "lambda with fat arrow" 'let f = |x| => x + 1; f(41)' "42"
test_eval "nested lambda" 'let add = |x| |y| x + y; add(30)(12)' "42"

# Variables and let bindings
echo
echo "--- Variables ---"
test_eval "let binding" 'let x = 42; x' "42"
test_eval "multiple bindings" 'let x = 10; let y = 32; x + y' "42"
test_eval "shadowing" 'let x = 10; let x = x * 2; x' "20"

# Print summary
echo
echo "==================================="
echo "Results: $PASSES passed, $FAILURES failed"

if [[ $FAILURES -eq 0 ]]; then
    echo -e "${GREEN}All one-liner tests passed!${NC}"
    exit 0
else
    echo -e "${RED}$FAILURES one-liner tests failed${NC}"
    exit 1
fi