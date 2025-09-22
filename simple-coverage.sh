#!/bin/bash
set -e

echo "📊 Running Simple Coverage Analysis..."
echo "======================================"

# Clean previous coverage data
cargo llvm-cov clean --workspace

# Run lib tests with coverage
echo "🧪 Running lib tests..."
cargo llvm-cov --lib --no-report

# Generate report
echo "📊 Coverage Report:"
cargo llvm-cov report --ignore-filename-regex "tests/|benches/" | grep -E "TOTAL|src/frontend/parser|src/runtime|src/backend" || echo "No coverage data"

# Summary
echo ""
echo "📈 Summary:"
cargo llvm-cov report --summary-only 2>&1 | tail -5 || echo "Unable to generate summary"