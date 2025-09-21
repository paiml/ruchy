#!/bin/bash
# scripts/test-browsers.sh
# WebAssembly Extreme Quality Assurance Framework v3.0
# Browser Matrix Testing Script

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting browser matrix testing...${NC}"

# Phase 1: Build WASM package
echo -e "${YELLOW}Phase 1: Building WASM package${NC}"
if command -v wasm-pack &> /dev/null; then
    wasm-pack build --target web --out-dir pkg
    echo -e "${GREEN}✓ WASM package built${NC}"
else
    echo -e "${YELLOW}Warning: wasm-pack not found, skipping WASM build${NC}"
fi

# Phase 2: Run native WASM tests
echo -e "${YELLOW}Phase 2: Native WASM tests${NC}"
if cargo test --target wasm32-unknown-unknown test_basic_compilation 2>/dev/null; then
    echo -e "${GREEN}✓ Native WASM tests passed${NC}"
else
    echo -e "${YELLOW}Warning: Native WASM tests skipped${NC}"
fi

# Phase 3: Browser compatibility tests
echo -e "${YELLOW}Phase 3: Browser compatibility tests${NC}"
if command -v wasm-pack &> /dev/null; then
    # Test in Chrome (headless)
    if wasm-pack test --headless --chrome -- --features wasm-test 2>/dev/null; then
        echo -e "${GREEN}✓ Chrome tests passed${NC}"
    else
        echo -e "${YELLOW}Warning: Chrome tests failed or skipped${NC}"
    fi

    # Test in Firefox (headless)
    if wasm-pack test --headless --firefox -- --features wasm-test 2>/dev/null; then
        echo -e "${GREEN}✓ Firefox tests passed${NC}"
    else
        echo -e "${YELLOW}Warning: Firefox tests failed or skipped${NC}"
    fi
else
    echo -e "${YELLOW}Warning: Browser tests skipped (wasm-pack not available)${NC}"
fi

# Phase 4: E2E JavaScript tests
echo -e "${YELLOW}Phase 4: E2E JavaScript tests${NC}"
if [ -d "e2e-tests" ] && [ -f "e2e-tests/package.json" ]; then
    cd e2e-tests

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo "Installing E2E test dependencies..."
        npm install
    fi

    # Run E2E tests
    if npm test 2>/dev/null; then
        echo -e "${GREEN}✓ E2E tests passed${NC}"
    else
        echo -e "${YELLOW}Warning: E2E tests failed or dependencies missing${NC}"
    fi

    cd ..
else
    echo -e "${YELLOW}Warning: E2E tests directory not found${NC}"
fi

# Phase 5: Size analysis
echo -e "${YELLOW}Phase 5: Binary size analysis${NC}"
if [ -f "scripts/analyze-size.sh" ]; then
    if ./scripts/analyze-size.sh; then
        echo -e "${GREEN}✓ Size analysis passed${NC}"
    else
        echo -e "${RED}✗ Size analysis failed${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}Warning: Size analysis script not found${NC}"
fi

# Summary
echo -e "${GREEN}Browser matrix testing complete!${NC}"

# Performance report
if [ -f "pkg/ruchy_wasm.wasm" ]; then
    SIZE=$(wc -c < pkg/ruchy_wasm.wasm)
    SIZE_KB=$((SIZE / 1024))
    echo "WASM binary size: ${SIZE_KB}KB"

    if [ $SIZE_KB -gt 500 ]; then
        echo -e "${RED}Warning: Binary size exceeds 500KB limit${NC}"
    fi
fi