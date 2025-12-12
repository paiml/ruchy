# Explicit Claims and Falsifiable Hypotheses

This document states explicit, measurable, falsifiable claims about Ruchy's capabilities and performance.

## Performance Claims

### C1: Parse Performance

**Claim**: Ruchy parses 1,000 lines of code in under 10ms on reference hardware.

**Measurement**:
- Metric: Wall-clock time for `parse()` function
- Input: Standardized 1000-line test file (`benches/fixtures/1000_lines.ruchy`)
- Hardware: GitHub Actions runner (2-core, 7GB RAM, Ubuntu 22.04)

**Threshold**: Mean < 10ms with 95% CI upper bound < 15ms

**Falsification**: If benchmark shows mean > 10ms, claim is false.

### C2: Transpilation Overhead

**Claim**: Transpiled Ruchy code executes within 10% of equivalent hand-written Rust.

**Measurement**:
- Metric: Runtime ratio (Ruchy / Rust)
- Benchmark suite: Fibonacci, sorting, string operations
- Conditions: Release build, LTO enabled

**Threshold**: Ratio ≤ 1.10 for all benchmarks

**Falsification**: If any benchmark shows ratio > 1.10, claim is false.

### C3: Memory Efficiency

**Claim**: Ruchy runtime memory usage is within 20% of equivalent Rust programs.

**Measurement**:
- Metric: Peak RSS during execution
- Benchmark: Memory-intensive operations (large arrays, deep recursion)

**Threshold**: Peak RSS ratio ≤ 1.20

**Falsification**: If ratio > 1.20, claim is false.

## Correctness Claims

### C4: Type Safety

**Claim**: Well-typed Ruchy programs do not produce runtime type errors.

**Measurement**:
- Test suite: 10,000+ property-based tests
- Input: Randomly generated well-typed programs

**Threshold**: Zero type errors in test suite

**Falsification**: Any runtime type error in well-typed program falsifies claim.

### C5: Transpilation Correctness

**Claim**: Transpiled programs produce identical output to interpreted programs.

**Measurement**:
- Test suite: All examples in `examples/` directory
- Comparison: stdout, stderr, exit code

**Threshold**: 100% match

**Falsification**: Any divergence between modes falsifies claim.

## ML/Oracle Claims

### C6: Error Classification Accuracy

**Claim**: Oracle classifier achieves ≥85% accuracy on error categorization.

**Measurement**:
- Dataset: Held-out 20% test set (n=500+)
- Metric: Classification accuracy
- Cross-validation: 5-fold

**Threshold**: Mean accuracy ≥ 0.85, 95% CI lower bound ≥ 0.80

**Falsification**: If accuracy < 0.85 on test set, claim is false.

### C7: Suggestion Relevance

**Claim**: ≥70% of Oracle fix suggestions are relevant to the error.

**Measurement**:
- Evaluation: Manual review of random sample (n=100)
- Criteria: Suggestion addresses root cause

**Threshold**: Relevance rate ≥ 0.70

**Falsification**: If relevance < 0.70, claim is false.

## Reproducibility Claims

### C8: Deterministic Execution

**Claim**: Same input with same seed produces identical output.

**Measurement**:
- Test: Run program twice with RUCHY_SEED=42
- Comparison: Full output comparison

**Threshold**: Byte-identical output

**Falsification**: Any difference in output falsifies claim.

### C9: Build Reproducibility

**Claim**: Same source at same commit produces identical binary (with Nix).

**Measurement**:
- Build: `nix build` twice from clean state
- Comparison: SHA256 of output binary

**Threshold**: Identical hash

**Falsification**: Different hashes falsify claim.

## Verification Status

| Claim | Status | Last Verified | Evidence |
|-------|--------|---------------|----------|
| C1 | ✅ Verified | 2024-03-01 | `benches/PERFORMANCE_BASELINE_v1.md` |
| C2 | ✅ Verified | 2024-03-01 | `benches/transpiler_benchmarks.rs` |
| C3 | ⏳ Pending | - | Benchmark suite in development |
| C4 | ✅ Verified | 2024-03-01 | Property tests passing |
| C5 | ✅ Verified | 2024-03-01 | Integration tests passing |
| C6 | ✅ Verified | 2024-03-01 | `tests/oracle_integration.rs` |
| C7 | ⏳ Pending | - | Manual evaluation planned |
| C8 | ✅ Verified | 2024-03-01 | Determinism tests passing |
| C9 | ⏳ Pending | - | Nix build reproducibility testing |

## How to Falsify

To falsify any claim:

1. Identify the claim and its threshold
2. Run the specified measurement
3. If result exceeds threshold, claim is false
4. File issue with reproduction steps

## References

- Popper, K. (1959). The Logic of Scientific Discovery
- docs/BENCHMARK_METHODOLOGY.md
- benches/README.md
