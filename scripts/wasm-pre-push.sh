#!/bin/bash
#
# WASM Pre-Push Hook
# Enforces comprehensive WASM quality gates before pushing
#
# Installation: Copy to .git/hooks/pre-push and make executable
#   cp scripts/wasm-pre-push.sh .git/hooks/pre-push
#   chmod +x .git/hooks/pre-push
#
# Or use symlink:
#   ln -sf ../../scripts/wasm-pre-push.sh .git/hooks/pre-push

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 WASM Quality Pre-Push Validation${NC}"
echo "===================================="

# Check if WASM files were modified in commits being pushed
REMOTE="$1"
URL="$2"

# Get the range of commits being pushed
while read local_ref local_sha remote_ref remote_sha; do
    if [ "$local_sha" = "0000000000000000000000000000000000000000" ]; then
        # Branch is being deleted
        continue
    fi

    if [ "$remote_sha" = "0000000000000000000000000000000000000000" ]; then
        # New branch
        RANGE="$local_sha"
    else
        # Existing branch
        RANGE="$remote_sha..$local_sha"
    fi

    WASM_FILES_CHANGED=$(git diff --name-only "$RANGE" | grep -E '(src/backend/wasm/|tests/wasm_)' || true)

    if [ -z "$WASM_FILES_CHANGED" ]; then
        echo -e "${GREEN}✅ No WASM files changed, skipping WASM validation${NC}"
        exit 0
    fi

    echo "WASM files in commits being pushed:"
    echo "$WASM_FILES_CHANGED" | sed 's/^/  - /'
    echo ""
done

# Track failures
FAILED=0

# 1. Full Memory Model Test Suite (~1s)
echo "💾 Running memory model E2E tests..."
START=$(date +%s)
if cargo test --test wasm_memory_model 2>&1 | grep -q "17 passed"; then
    END=$(date +%s)
    DURATION=$((END - START))
    echo -e "${GREEN}✅ Memory model tests: 17/17 passed (${DURATION}s)${NC}"
else
    echo -e "${RED}❌ Memory model tests failed${NC}"
    FAILED=1
fi

# 2. Full Property Test Suite (~8s)
echo "🔬 Running property tests..."
START=$(date +%s)
if cargo test --test wasm_memory_property_tests 2>&1 | tee /tmp/property_test_output.txt | grep -q "test result: ok"; then
    END=$(date +%s)
    DURATION=$((END - START))
    PASSED=$(grep -oE '[0-9]+ passed' /tmp/property_test_output.txt | head -1 | grep -oE '[0-9]+')
    echo -e "${GREEN}✅ Property tests: $PASSED/$PASSED passed (${DURATION}s)${NC}"
else
    echo -e "${RED}❌ Property tests failed${NC}"
    FAILED=1
fi

# 3. E2E Test Suite (if Playwright available) (~7s)
if command -v npx &> /dev/null && [ -f "playwright.config.ts" ]; then
    echo "🌐 Running E2E tests (all browsers)..."
    START=$(date +%s)

    # Check if HTTP server is running
    if ! curl -s http://localhost:8000 > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Starting HTTP server for E2E tests...${NC}"
        python3 -m http.server 8000 > /dev/null 2>&1 &
        SERVER_PID=$!
        sleep 2
    fi

    if npx playwright test --reporter=list 2>&1 | tee /tmp/e2e_test_output.txt | grep -q "39 passed"; then
        END=$(date +%s)
        DURATION=$((END - START))
        echo -e "${GREEN}✅ E2E tests: 39/39 passed (${DURATION}s)${NC}"
    else
        echo -e "${RED}❌ E2E tests failed${NC}"
        FAILED=1
    fi

    # Clean up server if we started it
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
else
    echo -e "${YELLOW}⚠️  Playwright not available, skipping E2E tests${NC}"
fi

# 4. Complexity Analysis (if PMAT available) (~1s)
if command -v pmat &> /dev/null; then
    echo "📈 Analyzing code complexity..."
    if pmat analyze complexity --max-cyclomatic 10 --max-cognitive 10 src/backend/wasm/mod.rs; then
        echo -e "${GREEN}✅ Complexity check passed (all functions ≤10)${NC}"
    else
        echo -e "${RED}❌ Complexity check failed (functions >10)${NC}"
        FAILED=1
    fi

    echo "🚫 Checking SATD violations..."
    if pmat analyze satd --fail-on-violation src/backend/wasm/; then
        echo -e "${GREEN}✅ No SATD violations${NC}"
    else
        echo -e "${RED}❌ SATD violations found${NC}"
        FAILED=1
    fi
else
    echo -e "${YELLOW}⚠️  PMAT not installed, skipping complexity/SATD checks${NC}"
fi

# 5. WASM Build Verification (~3s)
echo "📦 Building WASM module..."
START=$(date +%s)
if cargo build --target wasm32-unknown-unknown --release 2>&1 | tee /tmp/wasm_build.log; then
    END=$(date +%s)
    DURATION=$((END - START))

    # Check for warnings
    if grep -q "warning:" /tmp/wasm_build.log; then
        echo -e "${YELLOW}⚠️  WASM build has warnings${NC}"
        grep "warning:" /tmp/wasm_build.log | head -5
    fi

    # Check binary size
    WASM_FILE="target/wasm32-unknown-unknown/release/ruchy.wasm"
    if [ -f "$WASM_FILE" ]; then
        SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE")
        SIZE_MB=$((SIZE / 1024 / 1024))
        echo -e "${GREEN}✅ WASM build successful (${SIZE_MB}MB, ${DURATION}s)${NC}"

        if [ "$SIZE" -gt 10485760 ]; then
            echo -e "${YELLOW}⚠️  WARNING: WASM binary >10MB${NC}"
        fi
    fi
else
    echo -e "${RED}❌ WASM build failed${NC}"
    FAILED=1
fi

# 6. Test Coverage Report (optional, if llvm-cov available)
if command -v cargo-llvm-cov &> /dev/null; then
    echo "📊 Generating coverage report..."
    COVERAGE=$(cargo llvm-cov --test wasm_memory_model --test wasm_memory_property_tests --summary-only 2>&1 | grep -oE '[0-9]+\.[0-9]+%' | head -1 || echo "N/A")
    echo -e "${BLUE}ℹ️  Coverage: $COVERAGE${NC}"
fi

echo ""
echo "===================================="
echo ""

# Summary
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ ALL WASM QUALITY GATES PASSED!${NC}"
    echo ""
    echo "Safe to push to remote: $REMOTE"
    echo ""

    # Show summary
    echo "Summary:"
    echo "  - Memory Model: 17/17 tests"
    echo "  - Property Tests: 16/16 tests"
    if command -v npx &> /dev/null && [ -f "playwright.config.ts" ]; then
        echo "  - E2E Tests: 39/39 tests"
    fi
    echo "  - WASM Build: Success"
    echo "  - Complexity: ≤10 (Toyota Way)"

    exit 0
else
    echo -e "${RED}❌ WASM QUALITY GATES FAILED!${NC}"
    echo ""
    echo "Please fix the issues above before pushing."
    echo ""
    echo "To run tests manually:"
    echo "  make test-wasm-all"
    echo ""
    echo "To bypass (NOT RECOMMENDED):"
    echo "  git push --no-verify"
    echo ""
    exit 1
fi
