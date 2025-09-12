#!/bin/bash
# Build optimized WASM module for Ruchy

set -e

echo "Building optimized WASM module..."

# Build with minimal features and aggressive size optimization
cargo build \
    --target wasm32-unknown-unknown \
    --release \
    --no-default-features \
    --features wasm-target \
    -Z build-std=std,panic_abort \
    -Z build-std-features=panic_immediate_abort

# Get the output path
WASM_OUTPUT="target/wasm32-unknown-unknown/release/ruchy.wasm"

if [ -f "$WASM_OUTPUT" ]; then
    echo "Initial WASM size: $(du -h $WASM_OUTPUT | cut -f1)"
    
    # Use wasm-opt for additional optimization
    if command -v wasm-opt &> /dev/null; then
        echo "Running wasm-opt..."
        wasm-opt -Oz \
            --enable-simd \
            --enable-bulk-memory \
            --enable-sign-ext \
            --strip-debug \
            --strip-producers \
            -o "${WASM_OUTPUT}.opt" \
            "$WASM_OUTPUT"
        
        mv "${WASM_OUTPUT}.opt" "$WASM_OUTPUT"
        echo "Optimized WASM size: $(du -h $WASM_OUTPUT | cut -f1)"
    else
        echo "wasm-opt not found. Install with: cargo install wasm-opt"
    fi
    
    # Use wasm-strip if available
    if command -v wasm-strip &> /dev/null; then
        echo "Stripping WASM..."
        wasm-strip "$WASM_OUTPUT"
        echo "Final WASM size: $(du -h $WASM_OUTPUT | cut -f1)"
    fi
    
    echo "WASM module built: $WASM_OUTPUT"
else
    echo "Build failed - no WASM output found"
    exit 1
fi