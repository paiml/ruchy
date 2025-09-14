#!/bin/bash
# Simple and idiomatic coverage script for Rust projects

set -e

echo "🧹 Cleaning previous coverage data..."
cargo llvm-cov clean

echo "📊 Running tests with coverage instrumentation..."
# Run tests and generate coverage report
# --lib: Include library tests
# --tests: Include integration tests
# --ignore-filename-regex: Exclude test files from coverage
cargo llvm-cov \
    --lib \
    --tests \
    --ignore-filename-regex='(test|spec)\.rs$' \
    --html \
    --output-dir ./target/coverage \
    --summary-only \
    2>/dev/null || {
        echo "⚠️  Some tests failed, but coverage was still collected"
    }

echo ""
echo "📈 Coverage Summary:"
cargo llvm-cov report --summary-only 2>/dev/null | grep -E "TOTAL|^\s*src/" | tail -20

echo ""
echo "📁 HTML report generated at: ./target/coverage/index.html"
echo "🔗 Open with: xdg-open ./target/coverage/index.html"