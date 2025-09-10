#!/bin/bash
# Module-Specific Coverage Analysis for WASM + Notebooks
# Uses existing compiled artifacts when possible

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}🎯 Module-Specific Coverage Analysis${NC}"
echo "Target: WASM backend + Notebook modules only"
echo "============================================"

# Check for WASM backend module
if [ -d "src/backend/wasm" ]; then
    echo -e "\n${BLUE}📊 WASM Backend Module Analysis${NC}"
    WASM_FILES=$(find src/backend/wasm -name "*.rs" | wc -l)
    WASM_LINES=$(find src/backend/wasm -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
    echo "  📁 Files: $WASM_FILES"
    echo "  📝 Lines: $WASM_LINES"
else
    echo "  ⚠️  WASM backend module not found at src/backend/wasm"
fi

# Check for notebook module
if [ -d "ruchy-notebook/src" ]; then
    echo -e "\n${BLUE}📊 Notebook Module Analysis${NC}"
    NOTEBOOK_FILES=$(find ruchy-notebook/src -name "*.rs" | wc -l)
    NOTEBOOK_LINES=$(find ruchy-notebook/src -name "*.rs" -exec wc -l {} + | tail -1 | awk '{print $1}')
    echo "  📁 Files: $NOTEBOOK_FILES"
    echo "  📝 Lines: $NOTEBOOK_LINES"
else
    echo "  ⚠️  Notebook module not found at ruchy-notebook/src"
fi

# Check existing tests
echo -e "\n${BLUE}🧪 Test Coverage Analysis${NC}"
if [ -f "tests/wasm_emitter_tdd.rs" ]; then
    WASM_TEST_LINES=$(wc -l < tests/wasm_emitter_tdd.rs)
    echo "  🚀 WASM TDD Tests: $WASM_TEST_LINES lines"
else
    echo "  ❌ No WASM TDD tests found"
fi

if [ -f "ruchy-notebook/tests/notebook_acceptance_tests.rs" ]; then
    NOTEBOOK_TEST_LINES=$(wc -l < ruchy-notebook/tests/notebook_acceptance_tests.rs)
    echo "  📝 Notebook Tests: $NOTEBOOK_TEST_LINES lines"
else
    echo "  ❌ No notebook acceptance tests found"
fi

# Calculate basic metrics
echo -e "\n${GREEN}📈 Module Coverage Estimation${NC}"
echo "=============================="

# Estimate based on test/source ratio (rough heuristic)
if [ -n "$WASM_LINES" ] && [ -n "$WASM_TEST_LINES" ]; then
    WASM_TEST_RATIO=$(echo "scale=2; $WASM_TEST_LINES / $WASM_LINES * 100" | bc)
    echo "  🚀 WASM Test/Source Ratio: ${WASM_TEST_RATIO}%"
fi

if [ -n "$NOTEBOOK_LINES" ] && [ -n "$NOTEBOOK_TEST_LINES" ]; then
    NOTEBOOK_TEST_RATIO=$(echo "scale=2; $NOTEBOOK_TEST_LINES / $NOTEBOOK_LINES * 100" | bc)
    echo "  📝 Notebook Test/Source Ratio: ${NOTEBOOK_TEST_RATIO}%"
fi

# Quality recommendations
echo -e "\n${BLUE}🎯 Coverage Enhancement Priorities${NC}"
echo "=================================="
echo "To reach >80% coverage with A+ TDG score:"
echo ""
echo "1. 🚀 WASM Backend Module:"
echo "   - Add unit tests for each WASM instruction generator"
echo "   - Add integration tests for full AST→WASM compilation"
echo "   - Add property tests for WASM validation"
echo ""
echo "2. 📝 Notebook Module:" 
echo "   - Add unit tests for server endpoints"
echo "   - Add integration tests for code execution pipeline"
echo "   - Add acceptance tests for full notebook sessions"
echo ""
echo "3. 🎯 Cross-module Integration:"
echo "   - Add tests for WASM compilation in notebook environment"
echo "   - Add tests for notebook-specific WASM features"

echo -e "\n${GREEN}✨ Analysis Complete${NC}"
echo "Next: Run 'make coverage-wasm-notebook' after adding tests"