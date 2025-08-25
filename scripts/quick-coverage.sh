#!/bin/bash
# Quick Coverage Check for Ruchy
# Toyota Way: Fast feedback for development workflow

set -e

echo "⚡ Quick Coverage Check"
echo "======================"

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Quick coverage report (no HTML generation for speed)
echo "Running quick coverage analysis..."

# Clean and run
cargo llvm-cov clean --workspace > /dev/null 2>&1

# Get coverage summary only
COVERAGE_OUTPUT=$(cargo llvm-cov --summary-only --all-features --workspace --ignore-filename-regex "tests/|benches/|examples/" 2>/dev/null)

echo "$COVERAGE_OUTPUT"

# Extract coverage percentage  
COVERAGE=$(echo "$COVERAGE_OUTPUT" | grep -oE '[0-9]+\.[0-9]+%' | head -1 | sed 's/%//' | cut -d'.' -f1)

if [ -z "$COVERAGE" ]; then
    COVERAGE=0
fi

echo ""
printf "Coverage: "
if [ "$COVERAGE" -ge 90 ]; then
    echo -e "\033[0;32m${COVERAGE}%\033[0m ✅ Excellent"
elif [ "$COVERAGE" -ge 80 ]; then
    echo -e "\033[1;33m${COVERAGE}%\033[0m ⚠️  Good" 
else
    echo -e "\033[0;31m${COVERAGE}%\033[0m ❌ Needs Improvement"
fi

echo ""
echo "For detailed analysis: ./scripts/coverage.sh --open"