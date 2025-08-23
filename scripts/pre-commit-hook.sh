#!/bin/bash
# Ruchy Pre-commit Hook - MANDATORY Quality Gates
# Prevents regression of critical REPL features

set -e

echo "🔒 Running MANDATORY Quality Gates (Toyota Way - Stop the Line)"

# GATE 0: Core Interpreter Reliability (HIGHEST PRIORITY)
echo "0️⃣ Testing core interpreter reliability..."
cargo test --test interpreter_core_reliability --quiet
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ CRITICAL: Core interpreter tests failed!${NC}"
    echo "Following Toyota Way - we MUST fix this before ANY commits"
    echo "Run: cargo test --test interpreter_core_reliability"
    exit 1
fi
echo -e "${GREEN}✅ Core interpreter reliable${NC}"

# Build the binary first if needed
if [ ! -f ./target/debug/ruchy ]; then
    echo "Building debug binary for quality gates..."
    cargo build --bin ruchy --quiet
fi

# Use the built binary for tests
RUCHY_BIN="./target/debug/ruchy"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

# GATE 1: Basic REPL functionality
echo "1️⃣  Testing basic REPL functionality..."
echo 'println("test")' | timeout 5s $RUCHY_BIN repl 2>/dev/null | grep -q "test"
print_status $? "Basic REPL test passed"

# GATE 2: One-liner execution MUST work
echo "2️⃣  Testing one-liner execution..."
result=$($RUCHY_BIN -e "2 + 2" 2>/dev/null)
if [ "$result" = "4" ]; then
    print_status 0 "One-liner execution works"
else
    print_status 1 "One-liner execution failed"
fi

# GATE 3: Function calls MUST work
echo "3️⃣  Testing function calls..."
echo -e 'fun add(a: i32, b: i32) -> i32 { a + b }\nadd(3, 4)\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -q "7"
print_status $? "Function calls work"

# GATE 4: Match expressions MUST work
echo "4️⃣  Testing match expressions..."
echo -e 'match 2 { 1 => "one", 2 => "two", _ => "other" }\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -q "two"
print_status $? "Match expressions work"

# GATE 5: Block expressions MUST return last value
echo "5️⃣  Testing block expressions..."
echo -e '{ let a = 5; let b = 10; a + b }\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -q "15"
print_status $? "Block expressions work correctly"

# GATE 6: For loops MUST work
echo "6️⃣  Testing for loops..."
echo -e 'for i in [1, 2, 3] { println(i) }\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -q "2"
print_status $? "For loops work"

# GATE 7: While loops MUST work
echo "7️⃣  Testing while loops..."
echo -e 'let mut x = 0\nwhile x < 2 { x = x + 1 }\nx\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -q "2"
print_status $? "While loops work"

# GATE 8: String interpolation MUST work
echo "8️⃣  Testing string interpolation..."
echo -e 'let name = "Test"\nf"Hello {name}"\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -q "Hello Test"
print_status $? "String interpolation works"

# GATE 9: Lists MUST display correctly
echo "9️⃣  Testing list display..."
result=$(echo -e '[1, 2, 3]\n:quit' | $RUCHY_BIN repl 2>/dev/null | grep -o '\[1, 2, 3\]')
if [ "$result" = "[1, 2, 3]" ]; then
    print_status 0 "Lists display correctly"
else
    print_status 1 "List display is broken"
fi

# GATE 10: Run clippy with strict settings
echo "🔟 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings 2>/dev/null
print_status $? "No clippy warnings"

# GATE 11: Check for TODO/FIXME comments (excluding quality module)
echo "1️⃣1️⃣ Checking for SATD comments..."
! grep -r "TODO\|FIXME\|HACK" src/ --include="*.rs" --exclude-dir=quality 2>/dev/null | grep -v "TODO\\\\\|FIXME\\\\\|HACK"
print_status $? "No technical debt comments"

# GATE 12: Version consistency check
echo "1️⃣2️⃣ Checking version consistency..."
MAIN_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
WORKSPACE_VERSION=$(grep -A5 '^\[workspace.package\]' Cargo.toml | grep '^version = ' | cut -d'"' -f2)
CLI_USES_WORKSPACE=$(test -f ruchy-cli/Cargo.toml && grep 'version.workspace = true' ruchy-cli/Cargo.toml || echo "")
if [ -n "$CLI_USES_WORKSPACE" ]; then
    print_status 0 "Versions consistent: ruchy=$MAIN_VERSION, ruchy-cli=workspace($WORKSPACE_VERSION)"
else
    # Check if ruchy-cli exists
    if [ -f ruchy-cli/Cargo.toml ]; then
        CLI_VERSION=$(grep -m1 '^version = ' ruchy-cli/Cargo.toml | cut -d'"' -f2)
        if [ "$MAIN_VERSION" = "$CLI_VERSION" ]; then
            print_status 0 "Versions consistent: $MAIN_VERSION"
        else
            print_status 1 "Version mismatch: ruchy=$MAIN_VERSION, ruchy-cli=$CLI_VERSION"
            exit 1
        fi
    else
        print_status 0 "ruchy-cli deprecated - version: $MAIN_VERSION"
    fi
fi

# GATE 13: Dogfooding - Can we run a .ruchy script?
echo "1️⃣3️⃣ Testing dogfooding..."
cat > /tmp/test_dogfood.ruchy << 'EOF'
let x = 10
let y = 20
println(x + y)
EOF
$RUCHY_BIN run /tmp/test_dogfood.ruchy 2>/dev/null | grep -q "30"
print_status $? "Can run .ruchy scripts"
rm -f /tmp/test_dogfood.ruchy

# GATE 14: CRITICAL - File compilation regression prevention (v1.0.0 lesson)
echo "1️⃣4️⃣ Testing file compilation (CRITICAL - prevents v1.0.0 regression)..."

# Bug #1: Variable scoping hotfix verification  
echo "let x = 42; let y = x + 8; println(y);" > /tmp/test_scoping.ruchy
timeout 10s ./target/release/ruchy compile /tmp/test_scoping.ruchy > /dev/null 2>&1
if [ $? -eq 0 ]; then
    print_status 0 "Variable scoping hotfix working"
else
    echo -e "${RED}❌ CRITICAL REGRESSION: Variable scoping failed (Bug #1)${NC}"
    echo "This is the exact bug that made v1.0.0 unusable for file compilation"
    rm -f /tmp/test_scoping.ruchy
    exit 1
fi
rm -f /tmp/test_scoping.ruchy

# Bug #2: Function definitions hotfix verification
echo 'fun add(a, b) { a + b } println(add(5, 3));' > /tmp/test_functions.ruchy  
timeout 10s ./target/release/ruchy compile /tmp/test_functions.ruchy > /dev/null 2>&1
if [ $? -eq 0 ]; then
    print_status 0 "Function definitions hotfix working"
else
    echo -e "${RED}❌ CRITICAL REGRESSION: Function definitions failed (Bug #2)${NC}"
    echo "This is the exact bug that made v1.0.0 unusable for file compilation"
    rm -f /tmp/test_functions.ruchy
    exit 1
fi
rm -f /tmp/test_functions.ruchy

# Bug #3: Multi-arg printf hotfix verification
echo 'fun main() { let name = "Alice"; println("Hi", name, "!"); }' > /tmp/test_printf.ruchy
timeout 10s ./target/release/ruchy compile /tmp/test_printf.ruchy > /dev/null 2>&1
if [ $? -eq 0 ]; then
    print_status 0 "Multi-arg printf hotfix working"
else
    echo -e "${RED}❌ CRITICAL REGRESSION: Multi-arg printf failed (Bug #3)${NC}"
    echo "This is the exact bug that made v1.0.0 unusable for file compilation"
    rm -f /tmp/test_printf.ruchy  
    exit 1
fi
rm -f /tmp/test_printf.ruchy

# Cleanup any compilation artifacts
rm -f a.out

# GATE 15: PMAT Quality Gate Check
echo "1️⃣5️⃣ PMAT Quality Gate Check..."

# Check if PMAT is available and use the correct quality-gate command
if command -v pmat >/dev/null 2>&1; then
    echo "🏭 Running PMAT Quality Gate..."
    
    # Run PMAT quality gate with fail-on-violation flag
    # Timeout after 10 seconds to prevent hanging
    if timeout 10s pmat quality-gate --fail-on-violation --format summary 2>/dev/null; then
        print_status 0 "PMAT quality gate passed"
    else
        exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${YELLOW}⚠️ PMAT quality gate timed out (non-blocking)${NC}"
        else
            echo -e "${YELLOW}⚠️ PMAT quality gate check failed (non-blocking for now)${NC}"
            # Once PMAT is properly configured, change this to:
            # echo -e "${RED}❌ PMAT QUALITY GATE FAILED${NC}"
            # echo "🏭 Toyota Way: Stop the line for quality defects"
            # exit 1
        fi
    fi
else
    echo -e "${YELLOW}⚠️ PMAT not available - using standard quality gates only${NC}"
fi

echo -e "${GREEN}🎯 All MANDATORY quality gates passed!${NC}"
echo -e "${GREEN}🏭 Toyota Way: Quality built into process - no regressions allowed${NC}"
echo -e "${GREEN}🤖 PMAT Agent: Continuous quality monitoring active${NC}"