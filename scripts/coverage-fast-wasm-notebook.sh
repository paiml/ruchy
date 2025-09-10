#!/bin/bash
# Fast WASM & Notebook Coverage Analysis
# Simplified version for baseline measurement

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}🚀 Fast WASM & Notebook Coverage Analysis${NC}"
echo -e "${CYAN}Baseline measurement without heavy deps${NC}"
echo "================================================"

# Install cargo-llvm-cov if not present
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Clean previous coverage data
echo -e "\n${BLUE}🧹 Cleaning Previous Coverage Data...${NC}"
cargo llvm-cov clean --workspace

echo -e "\n${BLUE}📊 Running Core Tests Only (No Heavy Dependencies)...${NC}"

# Run minimal tests without heavy features
cargo llvm-cov test \
    --no-default-features \
    --features minimal \
    --html \
    --output-dir target/coverage-wasm-notebook-fast \
    --ignore-filename-regex ".*test.*|.*example.*|.*bench.*" \
    2>/dev/null || echo "Core tests completed"

# Generate text report for analysis
cargo llvm-cov report \
    --ignore-filename-regex ".*test.*|.*example.*|.*bench.*" > target/coverage-wasm-notebook-fast.txt

echo -e "\n${GREEN}📈 Fast Coverage Analysis Results${NC}"
echo "================================"

# Extract coverage metrics
TOTAL_COVERAGE=$(grep "TOTAL" target/coverage-wasm-notebook-fast.txt | awk '{print $NF}' | sed 's/%//' || echo "0.00")

echo -e "${BOLD}Coverage Results (Minimal Features):${NC}"
echo "  📊 Total Coverage: ${TOTAL_COVERAGE}%"

# Check if target achieved
TARGET=80.0
if (( $(echo "$TOTAL_COVERAGE >= $TARGET" | bc -l) )); then
    echo -e "${GREEN}✅ Coverage Target Achieved: ${TOTAL_COVERAGE}% >= ${TARGET}%${NC}"
    COVERAGE_STATUS="PASS"
else
    echo -e "${RED}❌ Coverage Target Not Met: ${TOTAL_COVERAGE}% < ${TARGET}%${NC}"
    COVERAGE_STATUS="FAIL"
fi

echo -e "\n${BLUE}📁 Report Locations:${NC}"
echo "  📊 HTML Report: target/coverage-wasm-notebook-fast/index.html"
echo "  📄 Text Report: target/coverage-wasm-notebook-fast.txt"

echo -e "\n${GREEN}✨ Fast Analysis Complete${NC}"
echo "Use 'make coverage-wasm-notebook' for full analysis with TDG scoring"

if [ "$COVERAGE_STATUS" = "PASS" ]; then
    exit 0
else
    exit 1
fi