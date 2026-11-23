#!/bin/bash
#
# WASM Pre-Commit Hook
# Enforces WASM quality gates before allowing commits
#
# Installation: Copy to .git/hooks/pre-commit and make executable
#   cp scripts/wasm-pre-commit.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
#
# Or use symlink:
#   ln -sf ../../scripts/wasm-pre-commit.sh .git/hooks/pre-commit

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🔍 WASM Quality Pre-Commit Checks${NC}"
echo "===================================="

# Check if WASM files were modified
WASM_FILES_CHANGED=$(git diff --cached --name-only | grep -E '(src/backend/wasm/|tests/wasm_)' || true)

if [ -z "$WASM_FILES_CHANGED" ]; then
    echo -e "${GREEN}✅ No WASM files changed, skipping WASM quality checks${NC}"
    exit 0
fi

echo "WASM files modified:"
echo "$WASM_FILES_CHANGED" | sed 's/^/  - /'
echo ""

# Track failures
FAILED=0

# 1. Memory Model Tests (Quick - ~1s)
echo "💾 Running memory model tests..."
if cargo test --test it wasm_memory_model:: --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ Memory model tests passed${NC}"
else
    echo -e "${RED}❌ Memory model tests failed${NC}"
    FAILED=1
fi

# 2. Property Invariant Tests (Quick - ~1s)
echo "🔬 Running property invariant tests..."
if cargo test --test it wasm_memory_property_tests::invariant_tests --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✅ Property invariant tests passed${NC}"
else
    echo -e "${RED}❌ Property invariant tests failed${NC}"
    FAILED=1
fi

# 3. Complexity Check (Quick - ~1s)
echo "📈 Checking code complexity..."
if command -v pmat &> /dev/null; then
    if pmat analyze complexity --max-cyclomatic 10 --max-cognitive 10 src/backend/wasm/mod.rs &> /dev/null; then
        echo -e "${GREEN}✅ Complexity check passed (≤10)${NC}"
    else
        echo -e "${RED}❌ Complexity check failed (>10)${NC}"
        FAILED=1
    fi
else
    echo -e "${YELLOW}⚠️  PMAT not installed, skipping complexity check${NC}"
fi

# 4. WASM Build Verification (Quick - ~2s)
echo "📦 Verifying WASM build..."
if cargo build --target wasm32-unknown-unknown --quiet 2>&1; then
    echo -e "${GREEN}✅ WASM build successful${NC}"
else
    echo -e "${RED}❌ WASM build failed${NC}"
    FAILED=1
fi

# 5. SATD Check (if PMAT available)
if command -v pmat &> /dev/null; then
    echo "🚫 Checking for SATD violations..."
    if pmat analyze satd --fail-on-violation src/backend/wasm/ &> /dev/null; then
        echo -e "${GREEN}✅ No SATD violations (TODO/FIXME/HACK)${NC}"
    else
        echo -e "${RED}❌ SATD violations found${NC}"
        FAILED=1
    fi
fi

echo ""
echo "===================================="

# Summary
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All WASM quality checks passed!${NC}"
    echo "Commit allowed to proceed."
    exit 0
else
    echo -e "${RED}❌ WASM quality checks failed!${NC}"
    echo ""
    echo "Please fix the issues above before committing."
    echo ""
    echo "To bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    echo ""
    echo "To run full test suite:"
    echo "  make test-wasm-all"
    exit 1
fi
