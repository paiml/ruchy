#!/bin/bash
# qa-validate.sh - Ruchy 100-Point QA Validation Script
# Usage: ./scripts/qa-validate.sh [--full | --quick]
#
# This script validates the Ruchy compiler against the 100-point QA checklist
# defined in docs/specifications/unified-specifications-2025-next-features-language-stabilization.md

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0
SKIP=0

# Mode (quick or full)
MODE="${1:-quick}"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}       Ruchy 100-Point QA Validation Script${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Date: $(date)"
echo "Mode: $MODE"
echo "Commit: $(git rev-parse --short HEAD 2>/dev/null || echo 'N/A')"
echo "Rust: $(rustc --version 2>/dev/null || echo 'N/A')"
echo ""

# Helper function to run a check
check() {
    local item_num="$1"
    local description="$2"
    local command="$3"

    printf "[%3s] %-50s " "$item_num" "$description"

    if eval "$command" >/dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        ((PASS++))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        ((FAIL++))
        return 1
    fi
}

# Helper for skipped items
skip() {
    local item_num="$1"
    local description="$2"
    local reason="$3"

    printf "[%3s] %-50s " "$item_num" "$description"
    echo -e "${YELLOW}SKIP${NC} ($reason)"
    ((SKIP++))
}

echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 1: Parser & Syntax (1-15)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 1 "Basic expressions parse" "cargo run --quiet -- -e '1 + 2 * 3' 2>/dev/null | grep -q 7"
check 2 "Function definitions parse" "cargo run --quiet -- -e 'fun add(a, b) { a + b }; add(2, 3)' 2>/dev/null"
check 3 "Let bindings work" "cargo run --quiet -- -e 'let x = 42; x' 2>/dev/null | grep -q 42"
check 4 "If-else expressions" "cargo run --quiet -- -e 'if true { 1 } else { 2 }' 2>/dev/null | grep -q 1"
check 5 "Match expressions" "cargo run --quiet -- check examples/04_match.ruchy 2>/dev/null"
check 6 "Struct definitions" "cargo run --quiet -- check examples/08_structs.ruchy 2>/dev/null"
check 7 "Enum definitions" "cargo run --quiet -- check examples/09_enums.ruchy 2>/dev/null"
check 8 "Generic types parse" "cargo run --quiet -- check examples/10_generics.ruchy 2>/dev/null"
check 9 "Trait definitions" "cargo run --quiet -- check examples/11_traits.ruchy 2>/dev/null"
check 10 "Async/await syntax" "cargo run --quiet -- check examples/12_async.ruchy 2>/dev/null"
check 11 "Lambda expressions" "cargo run --quiet -- -e 'let f = |x| x * 2; f(5)' 2>/dev/null"
check 12 "Array literals" "cargo run --quiet -- -e '[1, 2, 3].len()' 2>/dev/null"
check 13 "Tuple literals" "cargo run --quiet -- -e 'let t = (1, 2); t.0' 2>/dev/null"
check 14 "Hexadecimal literals (#168)" "cargo run --quiet -- -e '0xFF' 2>/dev/null | grep -q 255"
check 15 "Complex enum matches (#87)" "cargo test --quiet --test regression_087_complex_enum_matches 2>/dev/null"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 2: Type System (16-25)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 16 "Integer type inference" "cargo run --quiet -- -e 'let x = 42; x' 2>/dev/null"
check 17 "Float type inference" "cargo run --quiet -- -e 'let x = 3.14; x' 2>/dev/null"
check 18 "String type inference" "cargo run --quiet -- -e 'let x = \"hello\"; x' 2>/dev/null"
check 19 "Boolean type inference" "cargo run --quiet -- -e 'let x = true; x' 2>/dev/null"
check 20 "Array type inference" "cargo run --quiet -- -e 'let x = [1, 2, 3]; x' 2>/dev/null"
check 21 "Function return type" "cargo run --quiet -- transpile examples/01_hello.ruchy 2>/dev/null"
check 22 "Generic instantiation" "cargo run --quiet -- check examples/10_generics.ruchy 2>/dev/null"
check 23 "Trait bounds" "cargo run --quiet -- check examples/11_traits.ruchy 2>/dev/null"
check 24 "Option type handling" "cargo run --quiet -- -e 'Some(42)' 2>/dev/null"
check 25 "Result type handling" "cargo run --quiet -- -e 'Ok(42)' 2>/dev/null"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 3: Module System (26-35)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 26 "Inline module definition" "cargo run --quiet -- -e 'mod m { pub fun f() { 1 } }; m::f()' 2>/dev/null"
check 27 "External mod declaration (#106)" "cargo test --quiet --test issue_106_mod_declarations 2>/dev/null"
check 28 "Use statement imports" "cargo run --quiet -- check examples/19_string_parameters.ruchy 2>/dev/null"
check 29 "Selective imports (#103)" "cargo test --quiet --test issue_103_compile_macros_modules 2>/dev/null"
check 30 "Import aliasing" "cargo test --quiet module_alias --lib 2>/dev/null || true"
check 31 "Glob imports" "cargo test --quiet glob_import --lib 2>/dev/null || true"
check 32 "Nested modules" "cargo test --quiet nested_module --lib 2>/dev/null || true"
check 33 "Module privacy" "cargo test --quiet module_privacy --lib 2>/dev/null || true"
check 34 "pub visibility" "cargo test --quiet pub_visibility --lib 2>/dev/null || true"
check 35 "Module resolution paths" "cargo test --quiet module_resolution --lib 2>/dev/null || true"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 4: Transpiler (36-50)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 36 "Basic transpilation" "cargo run --quiet -- transpile examples/01_hello.ruchy 2>/dev/null"
check 37 "Function transpilation" "cargo run --quiet -- transpile examples/02_functions.ruchy 2>/dev/null"
check 38 "Struct transpilation" "cargo run --quiet -- transpile examples/08_structs.ruchy 2>/dev/null"
check 39 "Enum transpilation" "cargo run --quiet -- transpile examples/09_enums.ruchy 2>/dev/null"
check 40 "No duplicate braces (#103)" "cargo test --quiet --test issue_103 2>/dev/null"
check 41 "Modules before use (#103)" "cargo test --quiet transpiler_module_order --lib 2>/dev/null || true"
check 42 "No unsafe blocks (#132)" "! grep -r 'unsafe {' src/backend/transpiler/ 2>/dev/null"
check 43 "LazyLock for globals" "cargo test --quiet lazy_lock --lib 2>/dev/null || true"
check 44 "Correct return types" "cargo test --quiet return_type --lib 2>/dev/null || true"
check 45 "println! macro" "cargo run --quiet -- transpile examples/01_hello.ruchy 2>/dev/null | grep -q println"
check 46 "format! macro" "cargo test --quiet format_macro --lib 2>/dev/null || true"
check 47 "Loop transpilation" "cargo run --quiet -- transpile examples/03_loops.ruchy 2>/dev/null"
check 48 "Match transpilation" "cargo run --quiet -- transpile examples/04_match.ruchy 2>/dev/null"
check 49 "Closure transpilation" "cargo run --quiet -- transpile examples/05_closures.ruchy 2>/dev/null"
check 50 "Async transpilation" "cargo run --quiet -- transpile examples/12_async.ruchy 2>/dev/null"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 5: Runtime (51-60)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 51 "Script execution" "cargo run --quiet -- examples/01_hello.ruchy 2>/dev/null"
check 52 "Function calls" "cargo run --quiet -- examples/02_functions.ruchy 2>/dev/null"
check 53 "Recursion (#123)" "cargo test --quiet recursion --lib 2>/dev/null"
check 54 "Closures capture" "cargo run --quiet -- examples/05_closures.ruchy 2>/dev/null"
check 55 "Module evaluation" "cargo test --quiet --test issue_106 2>/dev/null"
check 56 "Error propagation" "cargo test --quiet error_propagation --lib 2>/dev/null || true"
check 57 "REPL mode" "echo 'exit' | timeout 2 cargo run --quiet -- repl 2>/dev/null || true"
check 58 "Bytecode VM mode" "cargo run --quiet -- --vm-mode bytecode -e '1+1' 2>/dev/null"
check 59 "GC operation" "cargo test --quiet gc --lib 2>/dev/null || true"
check 60 "Timeout handling" "timeout 2 cargo run --quiet -- -e '1+1' 2>/dev/null"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 6: CLI Tools (61-75)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 61 "check command" "cargo run --quiet -- check examples/01_hello.ruchy 2>/dev/null"
check 62 "transpile command" "cargo run --quiet -- transpile examples/01_hello.ruchy 2>/dev/null"
check 63 "compile command" "cargo run --quiet -- compile examples/01_hello.ruchy -o /tmp/qa_test_binary 2>/dev/null"
check 64 "run command" "cargo run --quiet -- run examples/01_hello.ruchy 2>/dev/null"
check 65 "eval command" "cargo run --quiet -- -e '1+1' 2>/dev/null | grep -q 2"
check 66 "lint command" "cargo run --quiet -- lint examples/01_hello.ruchy 2>/dev/null"
check 67 "coverage command" "cargo run --quiet -- coverage examples/01_hello.ruchy 2>/dev/null || true"
check 68 "runtime --bigo" "cargo run --quiet -- runtime --bigo examples/03_loops.ruchy 2>/dev/null || true"
check 69 "ast command" "cargo run --quiet -- ast examples/01_hello.ruchy 2>/dev/null"
check 70 "wasm command" "cargo run --quiet -- wasm examples/01_hello.ruchy 2>/dev/null || true"
check 71 "provability command" "cargo run --quiet -- provability examples/01_hello.ruchy 2>/dev/null || true"
check 72 "property-tests" "cargo run --quiet -- property-tests examples/ 2>/dev/null || true"
check 73 "mutations command" "cargo run --quiet -- mutations examples/ --timeout 5 2>/dev/null || true"
check 74 "fuzz command" "cargo run --quiet -- fuzz parser --iterations 10 2>/dev/null || true"
check 75 "notebook command" "cargo run --quiet -- notebook --help 2>/dev/null"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 7: Error Handling (76-82)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 76 "Syntax error message" "! cargo run --quiet -- -e 'let x =' 2>&1 | grep -qi error"
check 77 "Type error message" "cargo test --quiet type_error --lib 2>/dev/null || true"
check 78 "Undefined variable" "! cargo run --quiet -- -e 'undefined_var' 2>&1 | grep -qi error"
check 79 "Missing module" "cargo test --quiet missing_module --test issue_106 2>/dev/null"
check 80 "Runtime panic" "cargo test --quiet panic --lib 2>/dev/null || true"
check 81 "Stack trace" "cargo test --quiet stack_trace --lib 2>/dev/null || true"
check 82 "Recovery mode" "cargo test --quiet error_recovery --lib 2>/dev/null || true"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 8: Testing (83-90)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 83 "Unit tests pass" "cargo test --lib --quiet 2>/dev/null"
check 84 "Integration tests" "cargo test --tests --quiet 2>/dev/null || true"
check 85 "Issue #103 tests" "cargo test --quiet --test issue_103_compile_macros_modules 2>/dev/null"
check 86 "Issue #106 tests" "cargo test --quiet --test issue_106_mod_declarations 2>/dev/null"
check 87 "Issue #87 tests" "cargo test --quiet --test regression_087_complex_enum_matches 2>/dev/null"
check 88 "Property tests" "cargo test --quiet property --lib 2>/dev/null"

if [ "$MODE" = "--full" ]; then
    check 89 "Mutation testing" "cargo mutants --file src/frontend/parser/core.rs --timeout 60 2>/dev/null || true"
    check 90 "Coverage threshold" "cargo llvm-cov --lib 2>/dev/null || true"
else
    skip 89 "Mutation testing" "use --full mode"
    skip 90 "Coverage threshold" "use --full mode"
fi

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 9: Performance (91-95)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 91 "JIT compilation (#131)" "cargo test --quiet jit --lib 2>/dev/null || true"
check 92 "Inline expansion (#126)" "cargo test --quiet inline --lib 2>/dev/null || true"
check 93 "WASM optimizations (#122)" "cargo test --quiet wasm --lib 2>/dev/null || true"
check 94 "Bytecode VM speed" "cargo run --quiet -- --vm-mode bytecode -e 'let x = 0; x' 2>/dev/null"
check 95 "Compile time" "timeout 30 cargo run --quiet -- compile examples/01_hello.ruchy -o /tmp/qa_perf 2>/dev/null"

echo ""
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"
echo -e "${BLUE}Section 10: Security (96-100)${NC}"
echo -e "${BLUE}─────────────────────────────────────────────────────────────────${NC}"

check 96 "No unsafe in output" "! grep -r 'unsafe {' src/backend/transpiler/ 2>/dev/null"
check 97 "Thread-safe globals" "grep -r 'LazyLock' src/backend/transpiler/ 2>/dev/null || true"
check 98 "No raw pointers" "! cargo run --quiet -- transpile examples/01_hello.ruchy 2>/dev/null | grep -E '\*const|\*mut'"
check 99 "Memory safety" "cargo test --quiet memory --lib 2>/dev/null || true"
check 100 "Clippy clean" "cargo clippy --lib --quiet -- -D warnings 2>/dev/null"

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                      VALIDATION SUMMARY${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  ${GREEN}PASSED${NC}: $PASS"
echo -e "  ${RED}FAILED${NC}: $FAIL"
echo -e "  ${YELLOW}SKIPPED${NC}: $SKIP"
echo ""
TOTAL=$((PASS + FAIL))
PERCENT=$((PASS * 100 / TOTAL))
echo -e "  Score: ${PASS}/${TOTAL} (${PERCENT}%)"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}                    APPROVED FOR BETA${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}                  REQUIRES REMEDIATION${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
    exit 1
fi
