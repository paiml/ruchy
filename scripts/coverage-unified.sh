#!/bin/bash
# scripts/coverage-unified.sh
# WebAssembly Extreme Quality Assurance Framework v3.0
# Unified Coverage Collection Script

set -euo pipefail

# Color output for better readability
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Starting unified coverage collection...${NC}"

# Clean previous artifacts
rm -rf target/coverage
mkdir -p target/coverage

# Phase 1: Native branch coverage
echo -e "${YELLOW}Phase 1: Native testing with branch coverage${NC}"
cargo llvm-cov test \
    --all-features \
    --workspace \
    --branch \
    --ignore-filename-regex '(tests?/|benches/|examples/)' \
    --no-report \
    --output-dir target/coverage/native

# Phase 2: WASM unit tests in wasmtime (fast)
echo -e "${YELLOW}Phase 2: WASM unit tests${NC}"
LLVM_PROFILE_FILE="target/coverage/wasm-%p-%m.profraw" \
cargo llvm-cov test \
    --target wasm32-unknown-unknown \
    --no-report \
    --output-dir target/coverage/wasm

# Phase 3: Browser integration tests (comprehensive)
echo -e "${YELLOW}Phase 3: Browser matrix testing${NC}"
if command -v wasm-pack &> /dev/null; then
    wasm-pack test \
        --headless \
        --chrome \
        --firefox \
        -- --all-features
else
    echo -e "${YELLOW}Warning: wasm-pack not found, skipping browser tests${NC}"
fi

# Phase 4: Generate unified report with branch analysis
echo -e "${YELLOW}Phase 4: Generating unified report${NC}"
cargo llvm-cov report \
    --lcov \
    --branch \
    --output-path target/coverage/rust.lcov

# Validate branch coverage threshold
BRANCH_COV=$(cargo llvm-cov report --json | jq -r '.data[0].totals.branches.percent // 0')
if (( $(echo "$BRANCH_COV < 90" | bc -l 2>/dev/null || echo "0") )); then
    echo -e "${RED}ERROR: Branch coverage ${BRANCH_COV}% is below 90% threshold${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Coverage collection complete. Branch coverage: ${BRANCH_COV}%${NC}"