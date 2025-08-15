#!/bin/bash

# Code coverage script for Ruchy using cargo-llvm-cov
set -e

echo "🔍 Running code coverage analysis with cargo-llvm-cov..."

# Install cargo-llvm-cov if not present
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Clean previous coverage data
cargo llvm-cov clean --workspace

# Run coverage with all features and generate reports
echo "Running tests and generating coverage..."
cargo llvm-cov \
    --all-features \
    --workspace \
    --lcov \
    --output-path target/coverage/lcov.info \
    --html \
    --output-dir target/coverage/html \
    --ignore-filename-regex "tests/|benches/|examples/" \
    --no-fail-fast

# Get coverage summary
COVERAGE_OUTPUT=$(cargo llvm-cov --summary-only --all-features --workspace 2>&1)
echo "$COVERAGE_OUTPUT"

# Extract coverage percentage
COVERAGE=$(echo "$COVERAGE_OUTPUT" | grep "TOTAL" | awk '{print $10}' | sed 's/%//')

if [ -z "$COVERAGE" ]; then
    # Fallback extraction method
    COVERAGE=$(echo "$COVERAGE_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | head -1 | sed 's/%//')
fi

echo "📊 Total Coverage: ${COVERAGE}%"

# Check if coverage meets minimum threshold (80%)
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
    echo "❌ Coverage is below 80% threshold!"
    echo ""
    echo "To improve coverage:"
    echo "  1. Add more unit tests for uncovered functions"
    echo "  2. Add integration tests for main workflows"
    echo "  3. Add property tests for critical functions"
    echo ""
    echo "View detailed report: open target/coverage/html/index.html"
    exit 1
else
    echo "✅ Coverage meets the 80% threshold!"
fi

echo ""
echo "📄 HTML report: target/coverage/html/index.html"
echo "📄 LCOV report: target/coverage/lcov.info"
echo ""
echo "To view the HTML report:"
echo "  open target/coverage/html/index.html"