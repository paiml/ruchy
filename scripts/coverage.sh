#!/bin/bash

# Code coverage script for Ruchy
set -e

echo "🔍 Running code coverage analysis..."

# Install tarpaulin if not present
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run coverage
cargo tarpaulin \
    --out Html \
    --out Lcov \
    --output-dir target/coverage \
    --exclude-files "*/tests/*" \
    --exclude-files "*/benches/*" \
    --exclude-files "*/examples/*" \
    --ignore-panics \
    --timeout 120 \
    --skip-clean

# Generate coverage badge
COVERAGE=$(cargo tarpaulin --print-summary | grep "Coverage" | awk '{print $2}' | sed 's/%//')
echo "📊 Coverage: ${COVERAGE}%"

# Check if coverage meets minimum threshold (80%)
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
    echo "❌ Coverage is below 80% threshold!"
    exit 1
else
    echo "✅ Coverage meets the 80% threshold!"
fi

echo "📄 HTML report: target/coverage/tarpaulin-report.html"
echo "📄 LCOV report: target/coverage/lcov.info"