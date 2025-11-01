# Ruchy Performance Analysis - BENCH-008 (Prime Generation)

**Date**: 2025-11-01
**Benchmark**: First 10,000 primes (expected: 10,000th prime = 104,729)
**Tool**: ruchydbg v1.16.0 + manual timing

## Execution Modes Tested

| Mode | Time (ms) | vs Python | Status |
|------|----------:|----------:|--------|
| Python (baseline) | 90 | 1.0x | ✅ Reference |
| Ruchy AST (release, inline) | 1,588 | 18x slower | ⚠️  Slow |
| Ruchy AST (release, main) | 3,300 | 37x slower | ⚠️  Very Slow |  
| Ruchy Bytecode (debug) | 11,800 | 131x slower | ❌ Critical |
| Ruchy Bytecode (release) | 3,300 | 37x slower | ⚠️  Slow |

## Key Findings

### 1. Inline vs main() Performance Difference
- **Inline code** (top-level execution): 1,588ms
- **With main()**: 3,300ms
- **Overhead**: 2x slower for main() wrapper
- **Cause**: Function call overhead + scope management

### 2. Debug vs Release Build
- **Debug bytecode**: 11,800ms
- **Release bytecode**: 3,300ms
- **Improvement**: 3.6x faster in release mode
- **Conclusion**: Always use release builds for benchmarking

### 3. Pathological Input Detection
```bash
$ ruchydbg detect bench-008-primes.ruchy --threshold 15
Performance:
  Baseline: 5.60 µs
  Actual: 42.00 µs
  Slowdown: 7.50x
✅ Performance within acceptable bounds (< 15x threshold)
```

## Performance Bottlenecks

Based on profiling and analysis, the main bottlenecks are:

1. **Function call overhead** (~2x penalty for main() calls)
2. **Arithmetic operations** (while loop with i * i, modulo)
3. **Variable lookups** (scope chain traversal)
4. **Vector operations** (.len(), .push())

## Comparison with Other Languages

| Language | Time (ms) | Relative |
|----------|----------:|---------:|
| Rust (native) | ~5 | 1x |
| Python | 90 | 18x |
| Ruchy (AST) | 1,588 | 318x |

**Note**: Ruchy is 18x slower than Python, which is 18x slower than Rust.

## Recommendations

### Short-term (v3.172.0)
- ✅ Document performance characteristics
- ✅ Ensure release builds are used for benchmarks
- ⚠️  Consider function inlining for hot paths

### Medium-term  
- JIT compilation for hot loops
- Optimize variable lookup (caching)
- Reduce function call overhead
- Implement tail call optimization

### Long-term
- LLVM backend for native code generation
- Profile-guided optimizations
- Specialized bytecode ops for common patterns

## Testing Commands

```bash
# Baseline Python
time python3 bench-008-primes.py

# Ruchy AST (inline)
time ruchy -e "$(cat bench-008-primes-inline.ruchy)"

# Ruchy Bytecode (release)
time ruchy --vm-mode bytecode run bench-008-primes.ruchy

# With profiling
ruchydbg profile --stack bench-008-primes.ruchy
ruchydbg detect bench-008-primes.ruchy --threshold 15
```

## Conclusion

Ruchy's interpreter performance is **acceptable for a prototype language** but has significant room for optimization. The 18x slowdown vs Python is expected for an AST-walking interpreter. Future work should focus on:

1. Bytecode VM optimization (currently experimental)
2. JIT compilation for hot paths
3. Function inlining
4. Specialized ops for common patterns

**Status**: Performance is acceptable for development/prototyping, not production workloads.
