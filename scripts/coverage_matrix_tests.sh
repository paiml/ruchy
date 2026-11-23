#!/usr/bin/env bash
# Unified Coverage Tracking for Matrix Tests
#
# Generates coverage reports that include matrix test execution.
# Part of Phase 4 Notebook Excellence - Week 1.

set -euo pipefail

echo "📊 Matrix Test Coverage Analysis"
echo "================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "❌ cargo-llvm-cov not found. Installing..."
    cargo install cargo-llvm-cov
fi

echo -e "${BLUE}🧹 Cleaning old coverage data...${NC}"
cargo llvm-cov clean
rm -rf target/coverage/matrix
mkdir -p target/coverage/matrix

echo ""
echo -e "${BLUE}🦀 Running matrix tests with coverage...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run matrix tests with coverage tracking
echo "Running Matrix 001: Simple Arithmetic..."
cargo llvm-cov --no-report test --test it matrix_001_simple_arithmetic_native:: -- --test-threads=1 2>&1 | tee target/coverage/matrix/test-001-output.txt

echo ""
echo "Running Matrix 002: CSV Processing Workflow..."
cargo llvm-cov --no-report test --test it matrix_002_csv_workflow_native:: -- --test-threads=1 2>&1 | tee target/coverage/matrix/test-002-output.txt

echo ""
echo "Running Matrix 003: Statistical Analysis..."
cargo llvm-cov --no-report test --test it matrix_003_statistical_analysis_native:: -- --test-threads=1 2>&1 | tee target/coverage/matrix/test-003-output.txt

echo ""
echo "Running Matrix 004: Time Series Analysis..."
cargo llvm-cov --no-report test --test it matrix_004_time_series_native:: -- --test-threads=1 2>&1 | tee target/coverage/matrix/test-004-output.txt

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}📊 Generating coverage reports...${NC}"

# Generate HTML report
cargo llvm-cov report --html --output-dir target/coverage/matrix/html

# Generate LCOV report
cargo llvm-cov report --lcov --output-path target/coverage/matrix/lcov.info

# Generate text summary
cargo llvm-cov report > target/coverage/matrix/summary.txt

echo ""
echo -e "${GREEN}✅ Coverage reports generated${NC}"
echo ""

# Extract and display coverage summary
echo "📋 Coverage Summary:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f target/coverage/matrix/summary.txt ]; then
    # Show overall coverage percentage
    TOTAL_LINE=$(grep "TOTAL" target/coverage/matrix/summary.txt || echo "")
    if [ -n "$TOTAL_LINE" ]; then
        echo "$TOTAL_LINE"
    fi

    echo ""
    echo "Module Coverage:"
    echo "───────────────────────────────────────────────"

    # Show runtime module coverage (what matrix tests exercise)
    grep -E "src/runtime/" target/coverage/matrix/summary.txt | head -10 || echo "  No runtime coverage data"

    # Show REPL coverage (native tests use REPL)
    grep -E "src/repl/" target/coverage/matrix/summary.txt | head -5 || echo "  No REPL coverage data"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Report locations
echo "📁 Coverage Report Locations:"
echo "  • HTML Report: target/coverage/matrix/html/index.html"
echo "  • LCOV File: target/coverage/matrix/lcov.info"
echo "  • Text Summary: target/coverage/matrix/summary.txt"
echo "  • Test Output: target/coverage/matrix/test-output.txt"
echo ""

# Check if we can open the HTML report
if command -v xdg-open &> /dev/null; then
    echo "💡 Tip: Open HTML report with:"
    echo "   xdg-open target/coverage/matrix/html/index.html"
elif command -v open &> /dev/null; then
    echo "💡 Tip: Open HTML report with:"
    echo "   open target/coverage/matrix/html/index.html"
else
    echo "💡 Tip: Open target/coverage/matrix/html/index.html in your browser"
fi

echo ""
echo -e "${GREEN}✅ Matrix test coverage analysis complete${NC}"
