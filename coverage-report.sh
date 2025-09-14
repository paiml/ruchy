#!/bin/bash
# Idiomatic Rust coverage script - simple and reliable

echo "📊 Ruchy Coverage Report"
echo "========================"
echo ""

# Only run lib tests (most stable)
cargo llvm-cov --lib --summary-only 2>/dev/null | grep "^TOTAL" || {
    echo "Coverage tool not found. Installing..."
    cargo install cargo-llvm-cov
    cargo llvm-cov --lib --summary-only 2>/dev/null | grep "^TOTAL"
}

echo ""
echo "Run 'cargo llvm-cov --lib --html' for detailed HTML report"