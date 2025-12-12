# Performance Baseline Comparisons

This document establishes baseline comparisons for Ruchy performance evaluation.

## Comparison Targets

### 1. Rust (Native Compilation)

Ruchy transpiles to Rust, so the baseline is hand-written Rust code.

| Metric | Rust Baseline | Ruchy Target | Acceptable Overhead |
|--------|--------------|--------------|---------------------|
| Parse time | N/A | < 10ms/1000 LOC | N/A |
| Transpile time | N/A | < 50ms/1000 LOC | N/A |
| Runtime | 1.0x | 1.0x - 1.1x | ≤ 10% |
| Binary size | 1.0x | 1.0x - 1.2x | ≤ 20% |
| Memory usage | 1.0x | 1.0x - 1.1x | ≤ 10% |

### 2. Python (Reference Language)

Ruchy aims for Python-like syntax with Rust performance.

| Metric | Python 3.11 | Ruchy | Expected Speedup |
|--------|-------------|-------|------------------|
| Fibonacci(35) | 1.2s | 0.012s | 100x |
| List comprehension (10M) | 0.8s | 0.08s | 10x |
| String operations (1M) | 0.5s | 0.05s | 10x |
| JSON parsing (10MB) | 0.3s | 0.03s | 10x |

### 3. Other Transpiled Languages

| Language | Compilation Speed | Runtime | Memory |
|----------|------------------|---------|--------|
| TypeScript → JS | Baseline | Baseline | Baseline |
| Ruchy → Rust | 0.8x - 1.2x | 0.9x - 1.1x | 0.9x - 1.1x |

## Benchmark Suite

### Parser Benchmarks

```rust
// Baseline: Parse 1000-line file
// Target: < 10ms
// CI Threshold: 15ms (50% margin)

#[bench]
fn bench_parse_1000_lines(b: &mut Bencher) {
    b.iter(|| parse_file("fixtures/1000_lines.ruchy"));
}
```

### Transpiler Benchmarks

```rust
// Baseline: Transpile 1000-line file
// Target: < 50ms
// CI Threshold: 75ms (50% margin)

#[bench]
fn bench_transpile_1000_lines(b: &mut Bencher) {
    b.iter(|| transpile_file("fixtures/1000_lines.ruchy"));
}
```

### Runtime Benchmarks

```rust
// Baseline: Equivalent Rust code
// Target: Within 10% overhead
// CI Threshold: 20% overhead triggers investigation

#[bench]
fn bench_fibonacci_rust(b: &mut Bencher) {
    b.iter(|| fibonacci_rust(35));
}

#[bench]
fn bench_fibonacci_ruchy(b: &mut Bencher) {
    b.iter(|| fibonacci_ruchy(35));
}
```

## Historical Baselines

### v1.0.0 (2024-01-15)

| Benchmark | Mean | 95% CI | Effect vs v0.9 |
|-----------|------|--------|----------------|
| parse_1000 | 8.2ms | [7.9, 8.5] | d=0.3 (small) |
| transpile_1000 | 42.1ms | [40.5, 43.7] | d=0.5 (medium) |
| runtime_fib35 | 11.8ms | [11.2, 12.4] | d=0.1 (negligible) |

### v1.1.0 (2024-03-01)

| Benchmark | Mean | 95% CI | Effect vs v1.0 |
|-----------|------|--------|----------------|
| parse_1000 | 7.8ms | [7.5, 8.1] | d=-0.2 (improved) |
| transpile_1000 | 38.5ms | [37.0, 40.0] | d=-0.4 (improved) |
| runtime_fib35 | 11.5ms | [11.0, 12.0] | d=-0.1 (stable) |

## Regression Criteria

### Automatic Fail

- Any benchmark > 50% slower than baseline
- Effect size > 0.8 (large regression)
- Memory usage > 30% increase

### Manual Review Required

- Any benchmark > 20% slower than baseline
- Effect size > 0.5 (medium regression)
- New allocation patterns detected

### Acceptable

- Performance within ±10% of baseline
- Effect size < 0.2 (small or negligible)
- No new allocations

## Comparison Methodology

1. **Isolation**: Benchmarks run on dedicated CI runners
2. **Warm-up**: 3 iterations before measurement
3. **Samples**: Minimum 100 iterations
4. **Statistics**: 95% CI via bootstrap, Cohen's d for comparison
5. **Reproducibility**: Fixed seeds, pinned Rust version

## Reporting

CI generates comparison reports:

```json
{
  "benchmark": "parse_1000_lines",
  "current": {
    "mean_ms": 7.8,
    "ci_95": [7.5, 8.1]
  },
  "baseline": {
    "version": "v1.0.0",
    "mean_ms": 8.2,
    "ci_95": [7.9, 8.5]
  },
  "comparison": {
    "change_percent": -4.9,
    "effect_size": -0.2,
    "significance": "not_significant",
    "verdict": "PASS"
  }
}
```
