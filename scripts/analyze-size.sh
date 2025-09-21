#!/bin/bash
# scripts/analyze-size.sh
# WebAssembly Extreme Quality Assurance Framework v3.0
# WASM Binary Size Analysis

set -euo pipefail

echo "Building optimized WASM binary..."
cargo build --release --target wasm32-unknown-unknown

# Find the WASM file
WASM_FILE=$(find target/wasm32-unknown-unknown/release -name "*.wasm" | head -1)

if [ -z "$WASM_FILE" ]; then
    echo "ERROR: No WASM file found"
    exit 1
fi

echo "Found WASM file: $WASM_FILE"

# Run wasm-opt optimization if available
OPTIMIZED_FILE="target/optimized.wasm"
if command -v wasm-opt &> /dev/null; then
    echo "Running wasm-opt optimization..."
    wasm-opt -Oz "$WASM_FILE" -o "$OPTIMIZED_FILE"
else
    echo "wasm-opt not found, using original file"
    cp "$WASM_FILE" "$OPTIMIZED_FILE"
fi

echo "Size analysis:"
echo "==============="

# Original size
ORIGINAL=$(wc -c < "$WASM_FILE")
echo "Original: $(numfmt --to=iec-i --suffix=B $ORIGINAL)"

# Optimized size
OPTIMIZED=$(wc -c < "$OPTIMIZED_FILE")
echo "Optimized: $(numfmt --to=iec-i --suffix=B $OPTIMIZED)"

# Reduction calculation
if [ $ORIGINAL -gt 0 ]; then
    REDUCTION=$((ORIGINAL - OPTIMIZED))
    PERCENT=$((REDUCTION * 100 / ORIGINAL))
    echo "Reduction: $(numfmt --to=iec-i --suffix=B $REDUCTION) ($PERCENT%)"
fi

# Detailed analysis with twiggy if available
if command -v twiggy &> /dev/null; then
    echo -e "\nTop 10 largest functions:"
    twiggy top -n 10 "$OPTIMIZED_FILE" 2>/dev/null || echo "Twiggy analysis failed"

    echo -e "\nMonomorphization bloat analysis:"
    twiggy monos "$OPTIMIZED_FILE" 2>/dev/null || echo "Monomorphization analysis failed"
else
    echo -e "\ntwiggy not available for detailed analysis"
fi

# Fail if binary exceeds size limit
MAX_SIZE=$((500 * 1024))  # 500KB
if [ $OPTIMIZED -gt $MAX_SIZE ]; then
    echo "ERROR: Optimized WASM size ($OPTIMIZED bytes) exceeds limit ($MAX_SIZE bytes)"
    exit 1
fi

echo -e "\n✅ Size analysis complete"