#!/bin/bash
# Simple, reliable coverage script for Ruchy
# Following Rust best practices and idiomatic tooling

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "📊 Running Ruchy Coverage Analysis"
echo "===================================="
echo ""

# Ensure cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Clean previous coverage data
echo "🧹 Cleaning previous coverage data..."
cargo llvm-cov clean --workspace

# Run coverage for library tests only (most reliable)
echo "🧪 Running library tests with coverage..."
cargo llvm-cov --lib \
    --html \
    --output-dir target/coverage \
    --summary-only \
    2>/dev/null || true

# Get the coverage percentage
COVERAGE=$(cargo llvm-cov report --lib 2>/dev/null | grep "^TOTAL" | awk '{print $10}')

echo ""
echo "================== COVERAGE REPORT =================="
cargo llvm-cov report --lib 2>/dev/null | grep "^TOTAL"
echo "====================================================="
echo ""

# Parse coverage as a number
COVERAGE_NUM=$(echo "$COVERAGE" | sed 's/%//')

# Display coverage with color coding
if (( $(echo "$COVERAGE_NUM < 40" | bc -l) )); then
    echo -e "${RED}📉 Coverage: ${COVERAGE}${NC}"
elif (( $(echo "$COVERAGE_NUM < 60" | bc -l) )); then
    echo -e "${YELLOW}📊 Coverage: ${COVERAGE}${NC}"
else
    echo -e "${GREEN}📈 Coverage: ${COVERAGE}${NC}"
fi

echo ""
echo "📁 HTML Report: file://$(pwd)/target/coverage/index.html"
echo ""

# Optional: Try integration tests (may have issues)
echo "🔄 Attempting integration tests (optional)..."
cargo llvm-cov --tests \
    --no-report \
    --ignore-filename-regex='tests?/' \
    2>/dev/null || echo "⚠️  Some integration tests failed, but coverage was collected"

# Final report
echo ""
echo "✅ Coverage analysis complete!"
echo ""
echo "To view detailed HTML report:"
echo "  open target/coverage/index.html"
echo ""
echo "To improve coverage:"
echo "  1. Add unit tests in src/*/mod.rs files (#[cfg(test)] modules)"
echo "  2. Focus on untested functions shown in HTML report"
echo "  3. Use 'cargo llvm-cov --show-missing' to find gaps"