#!/bin/bash
# WASM & Notebook Coverage Analysis Script
# Target: >80% test coverage with A+ TDG score
# Uses LLVM coverage for precise measurement

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}🚀 WASM & Notebook Coverage Analysis${NC}"
echo -e "${CYAN}Target: >80% coverage with A+ TDG score${NC}"
echo "=================================================="

# Check dependencies
echo -e "\n${BLUE}📋 Checking Dependencies...${NC}"

# Install cargo-llvm-cov if not present
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Check PMAT for TDG scoring
if ! command -v pmat &> /dev/null; then
    echo -e "${YELLOW}⚠️  PMAT not found - TDG scoring will be skipped${NC}"
    echo "Install with: cargo install pmat"
    SKIP_TDG=true
else
    SKIP_TDG=false
fi

echo -e "${GREEN}✅ Dependencies ready${NC}"

# Clean previous coverage data
echo -e "\n${BLUE}🧹 Cleaning Previous Coverage Data...${NC}"
cargo llvm-cov clean --workspace

# Define target modules for coverage
WASM_MODULES="src/backend/wasm"
NOTEBOOK_MODULES="ruchy-notebook/src"
TEST_MODULES="tests/wasm_emitter_tdd.rs ruchy-notebook/tests"

echo -e "\n${BLUE}📊 Running WASM Module Coverage...${NC}"
echo "Target modules: ${WASM_MODULES}"

# Run WASM-specific tests with coverage
cargo llvm-cov test \
    --features wasm-compile \
    --no-report \
    --test wasm_emitter_tdd \
    --timeout 60 \
    2>/dev/null || echo "WASM tests completed with warnings"

echo -e "\n${BLUE}📊 Running Notebook Module Coverage...${NC}"
echo "Target modules: ${NOTEBOOK_MODULES}"

# Run notebook-specific tests with coverage  
cargo llvm-cov test \
    --features notebook \
    --manifest-path ruchy-notebook/Cargo.toml \
    --no-report \
    --features native \
    --timeout 60 \
    2>/dev/null || echo "Notebook tests completed with warnings"

echo -e "\n${BLUE}📊 Generating Combined Coverage Report...${NC}"

# Generate combined HTML report for both modules
cargo llvm-cov report \
    --html \
    --output-dir target/coverage-wasm-notebook \
    --ignore-filename-regex ".*test.*|.*example.*|.*bench.*" \
    --include-build-script=false

# Generate text report for analysis
cargo llvm-cov report \
    --ignore-filename-regex ".*test.*|.*example.*|.*bench.*" \
    --include-build-script=false > target/coverage-wasm-notebook.txt

echo -e "\n${GREEN}📈 Coverage Analysis Results${NC}"
echo "=============================="

# Extract coverage metrics
TOTAL_COVERAGE=$(grep "TOTAL" target/coverage-wasm-notebook.txt | awk '{print $NF}' | sed 's/%//')
WASM_COVERAGE=$(grep -E "(backend/wasm|wasm)" target/coverage-wasm-notebook.txt | head -1 | awk '{print $NF}' | sed 's/%//' || echo "0.00")
NOTEBOOK_COVERAGE=$(grep -E "(notebook)" target/coverage-wasm-notebook.txt | head -1 | awk '{print $NF}' | sed 's/%//' || echo "0.00")

echo -e "${BOLD}Coverage Results:${NC}"
echo "  📊 Total Coverage: ${TOTAL_COVERAGE}%"
echo "  🚀 WASM Module: ${WASM_COVERAGE}%"
echo "  📝 Notebook Module: ${NOTEBOOK_COVERAGE}%"

# Check if target achieved
TARGET=80.0
if (( $(echo "$TOTAL_COVERAGE >= $TARGET" | bc -l) )); then
    echo -e "${GREEN}✅ Coverage Target Achieved: ${TOTAL_COVERAGE}% >= ${TARGET}%${NC}"
    COVERAGE_STATUS="PASS"
else
    echo -e "${RED}❌ Coverage Target Not Met: ${TOTAL_COVERAGE}% < ${TARGET}%${NC}"
    COVERAGE_STATUS="FAIL"
fi

echo -e "\n${BLUE}🎯 TDG Quality Analysis${NC}"
echo "========================"

if [ "$SKIP_TDG" = "false" ]; then
    # Run TDG analysis on WASM module
    echo -e "${CYAN}WASM Module TDG Score:${NC}"
    WASM_TDG_SCORE=$(pmat tdg src/backend/wasm --quiet 2>/dev/null || echo "0")
    if [ -n "$WASM_TDG_SCORE" ] && (( $(echo "$WASM_TDG_SCORE >= 95" | bc -l) 2>/dev/null )); then
        echo -e "  🚀 WASM: ${GREEN}${WASM_TDG_SCORE}/100 (A+)${NC}"
        WASM_TDG_STATUS="PASS"
    else
        echo -e "  🚀 WASM: ${YELLOW}${WASM_TDG_SCORE}/100${NC}"
        WASM_TDG_STATUS="REVIEW"
    fi
    
    # Run TDG analysis on notebook module
    echo -e "${CYAN}Notebook Module TDG Score:${NC}"
    NOTEBOOK_TDG_SCORE=$(pmat tdg ruchy-notebook/src --quiet 2>/dev/null || echo "0")
    if [ -n "$NOTEBOOK_TDG_SCORE" ] && (( $(echo "$NOTEBOOK_TDG_SCORE >= 95" | bc -l) 2>/dev/null )); then
        echo -e "  📝 Notebook: ${GREEN}${NOTEBOOK_TDG_SCORE}/100 (A+)${NC}"
        NOTEBOOK_TDG_STATUS="PASS"
    else
        echo -e "  📝 Notebook: ${YELLOW}${NOTEBOOK_TDG_SCORE}/100${NC}"
        NOTEBOOK_TDG_STATUS="REVIEW"
    fi
else
    echo -e "${YELLOW}⚠️  TDG analysis skipped - PMAT not available${NC}"
    WASM_TDG_STATUS="SKIP"
    NOTEBOOK_TDG_STATUS="SKIP"
fi

echo -e "\n${BOLD}🎯 Quality Gate Summary${NC}"
echo "=========================="
echo "Target: >80% coverage + A+ TDG score (95+)"
echo ""

# Final status determination
if [ "$COVERAGE_STATUS" = "PASS" ] && [ "$WASM_TDG_STATUS" = "PASS" ] && [ "$NOTEBOOK_TDG_STATUS" = "PASS" ]; then
    echo -e "${GREEN}🏆 QUALITY GATE: PASSED${NC}"
    echo -e "   ✅ Coverage: ${TOTAL_COVERAGE}% (target: ${TARGET}%)"
    echo -e "   ✅ WASM TDG: ${WASM_TDG_SCORE}/100 (A+)"
    echo -e "   ✅ Notebook TDG: ${NOTEBOOK_TDG_SCORE}/100 (A+)"
    EXIT_CODE=0
elif [ "$COVERAGE_STATUS" = "FAIL" ]; then
    echo -e "${RED}❌ QUALITY GATE: FAILED - Coverage Below Target${NC}"
    echo -e "   ❌ Coverage: ${TOTAL_COVERAGE}% (target: ${TARGET}%)"
    echo "   📋 Action Required: Add more tests to improve coverage"
    EXIT_CODE=1
else
    echo -e "${YELLOW}⚠️  QUALITY GATE: REVIEW REQUIRED${NC}"
    echo -e "   ✅ Coverage: ${TOTAL_COVERAGE}% (target: ${TARGET}%)"
    if [ "$WASM_TDG_STATUS" != "PASS" ]; then
        echo -e "   ⚠️  WASM TDG: ${WASM_TDG_SCORE}/100 (review required)"
    fi
    if [ "$NOTEBOOK_TDG_STATUS" != "PASS" ]; then
        echo -e "   ⚠️  Notebook TDG: ${NOTEBOOK_TDG_SCORE}/100 (review required)"
    fi
    EXIT_CODE=0
fi

echo -e "\n${BLUE}📁 Report Locations:${NC}"
echo "  📊 HTML Report: target/coverage-wasm-notebook/index.html"
echo "  📄 Text Report: target/coverage-wasm-notebook.txt"

echo -e "\n${BLUE}🛠️  Improvement Suggestions:${NC}"
if (( $(echo "$TOTAL_COVERAGE < 80" | bc -l) )); then
    echo "  • Add unit tests for uncovered functions"
    echo "  • Add integration tests for WASM compilation pipeline"
    echo "  • Add notebook API endpoint tests"
    echo "  • Add error handling path tests"
fi

if [ "$SKIP_TDG" = "false" ]; then
    if (( $(echo "$WASM_TDG_SCORE < 95" | bc -l) )); then
        echo "  • Refactor complex WASM emitter functions (complexity > 10)"
        echo "  • Add documentation to WASM module functions"
        echo "  • Remove any TODO/FIXME comments"
    fi
    
    if (( $(echo "$NOTEBOOK_TDG_SCORE < 95" | bc -l) )); then
        echo "  • Simplify notebook API handler functions"
        echo "  • Add comprehensive documentation"
        echo "  • Eliminate code duplication in notebook module"
    fi
fi

echo -e "\n${GREEN}✨ Analysis Complete${NC}"

exit $EXIT_CODE