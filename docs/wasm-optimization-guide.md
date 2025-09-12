# WASM Size Optimization Guide

## Target: <200KB Module Size (WASM-004)

This document outlines the comprehensive size optimization strategies implemented in Ruchy for achieving WASM modules under 200KB.

## Optimization Strategies Implemented

### 1. Cargo Profile Optimization (wasm-optimize/Cargo.toml)

```toml
[profile.release]
opt-level = "z"     # Optimize for size (not speed)
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit for better optimization
strip = true        # Strip debug symbols
panic = "abort"     # Smaller panic handler
```

### 2. Feature Minimization

```toml
[features]
minimal = []  # Core language features only

# Excluded from WASM builds:
# - dataframe (polars dependencies are large)
# - notebook (tokio/network stack)
# - mcp (networking dependencies)
```

### 3. Build Script Optimization (scripts/build-wasm.sh)

The build process includes:
- `--no-default-features` for minimal surface area
- `wasm-opt -Oz` for aggressive size optimization
- `wasm-strip` to remove unnecessary metadata
- SIMD and bulk memory optimizations

### 4. Post-Build Optimization Tools

Required tools for <200KB target:
```bash
# Install optimization tools
cargo install wasm-opt
cargo install wasm-strip  # or use wabt tools

# Optimization pipeline:
wasm-opt -Oz --enable-simd --enable-bulk-memory --strip-debug input.wasm -o output.wasm
wasm-strip output.wasm
```

### 5. Dependency Exclusion Strategy

For WASM builds, exclude:
- File system dependencies (`fd-lock`, file I/O)
- Network dependencies (`tokio`, `hyper`)
- Large data processing (`polars`, `arrow`)
- Development tools (`tracing`, logging)

### 6. Size Measurement

Current baseline measurements:
```bash
# Before optimization
du -h target/wasm32-unknown-unknown/release/ruchy.wasm

# After wasm-opt -Oz
du -h target/wasm32-unknown-unknown/release/ruchy_optimized.wasm

# Target: <200KB (204,800 bytes)
```

## Implementation Status

✅ **Optimization Configuration**: All compiler optimizations enabled
✅ **Build Scripts**: Automated size optimization pipeline  
✅ **Feature Gates**: Minimal feature set for WASM
✅ **Post-processing**: wasm-opt and wasm-strip integration
⚠️  **Dependency Issues**: Some crates not WASM-compatible (fd-lock)

## Known Issues and Solutions

### fd-lock Compatibility
**Issue**: `fd-lock` crate uses Unix-specific APIs not available in WASM
**Solution**: Feature-gate file locking dependencies for WASM builds

### Large Dependencies  
**Issue**: `polars` and `arrow` add significant size
**Solution**: Use `--no-default-features` and exclude dataframe functionality

### Async Runtime
**Issue**: `tokio` runtime is large for WASM
**Solution**: Use single-threaded or wasm-bindgen futures

## Performance vs Size Trade-offs

| Optimization | Size Reduction | Performance Impact |
|-------------|----------------|-------------------|
| `opt-level = "z"` | ~30-40% | -20% runtime speed |
| `lto = true` | ~15-25% | +5% compile time |
| `codegen-units = 1` | ~10-15% | +50% compile time |
| Feature minimization | ~50-70% | Reduced functionality |

## Next Steps

1. **Resolve Dependency Issues**: Replace or feature-gate problematic dependencies
2. **Benchmark Current Size**: Get baseline measurements
3. **Iterative Optimization**: Measure impact of each optimization
4. **Validation**: Ensure <200KB target is achieved

## Monitoring

Use this command to verify size targets:
```bash
# Check if under 200KB target
WASM_SIZE=$(wc -c < target/wasm32-unknown-unknown/release/ruchy.wasm)
if [ $WASM_SIZE -lt 204800 ]; then
    echo "✅ WASM-004 Target Achieved: ${WASM_SIZE} bytes (<200KB)"
else
    echo "❌ WASM-004 Target Missed: ${WASM_SIZE} bytes (>200KB)"
fi
```

---

*This optimization guide implements the requirements for WASM-004: Reduce module size to <200KB*