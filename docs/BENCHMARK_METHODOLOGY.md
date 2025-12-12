# Benchmark Methodology

This document describes the statistical methodology used for Ruchy's performance benchmarks, ensuring reproducible and scientifically rigorous results.

## Hardware Specifications

All benchmarks are run on standardized hardware configurations documented in CI:

| Component | Specification |
|-----------|--------------|
| CPU | AMD EPYC 7763 (GitHub Actions) or equivalent |
| RAM | 16GB minimum |
| OS | Ubuntu 22.04 LTS |
| Rust | 1.83.0 (pinned) |

## Statistical Framework

### Sample Size Determination

We use power analysis to determine minimum sample sizes:

- **Effect size (d)**: 0.5 (medium effect, Cohen's convention)
- **Alpha (α)**: 0.05 (5% false positive rate)
- **Power (1-β)**: 0.80 (80% chance of detecting true effects)
- **Minimum samples**: n ≥ 64 per condition

### Confidence Intervals

All benchmarks report 95% confidence intervals using bootstrap resampling:

```
Mean ± 1.96 × (Standard Error)
```

Example output:
```
parse_large_file: 12.34ms ± 0.56ms (95% CI: [11.78ms, 12.90ms])
```

### Effect Sizes

We report Cohen's d for all comparisons:

| d value | Interpretation |
|---------|---------------|
| 0.2 | Small effect |
| 0.5 | Medium effect |
| 0.8 | Large effect |

Formula: `d = (M1 - M2) / pooled_SD`

### Statistical Tests

- **Within-subject comparisons**: Paired t-test or Wilcoxon signed-rank
- **Between-subject comparisons**: Independent t-test or Mann-Whitney U
- **Multiple comparisons**: Bonferroni correction applied

## Benchmark Configuration

### Criterion.rs Settings

```toml
[profile.bench]
opt-level = 3
lto = true
codegen-units = 1

[[bench]]
name = "parser_benchmarks"
harness = false
```

### Warm-up and Measurement

- **Warm-up iterations**: 3 (JIT compilation, cache warming)
- **Measurement iterations**: 100 minimum
- **Outlier detection**: Modified Z-score (threshold: 3.5)
- **Outlier handling**: Report separately, do not exclude

### Reproducibility Requirements

1. **Fixed random seeds**: All randomized benchmarks use `RUCHY_BENCH_SEED=42`
2. **Isolated execution**: No other CPU-intensive processes
3. **Cool-down period**: 1 second between benchmark groups
4. **Memory baseline**: Record RSS before and after

## Reporting Standards

### Required Metrics

Every benchmark report must include:

1. **Central tendency**: Mean and median
2. **Dispersion**: Standard deviation and IQR
3. **Confidence interval**: 95% CI via bootstrap (10,000 resamples)
4. **Sample size**: Total iterations after warm-up
5. **Effect size**: Cohen's d for comparisons

### Comparison Baselines

Performance is tracked against:

1. **Previous release**: Regression detection
2. **Rust equivalent**: Transpilation overhead measurement
3. **Python equivalent**: Speedup factor calculation

### Example Report Format

```json
{
  "benchmark": "parse_1000_line_file",
  "version": "1.2.0",
  "timestamp": "2024-03-01T12:00:00Z",
  "hardware": "GitHub Actions runner (2-core)",
  "results": {
    "mean_ms": 12.34,
    "median_ms": 12.01,
    "std_dev_ms": 1.23,
    "ci_95_lower_ms": 11.78,
    "ci_95_upper_ms": 12.90,
    "sample_size": 100,
    "outliers_detected": 2,
    "effect_size_vs_previous": 0.15,
    "interpretation": "No significant change (d < 0.2)"
  }
}
```

## Regression Detection

### Automated Alerts

CI triggers alerts when:

- Performance degrades > 10% (mean)
- Effect size > 0.5 (medium) compared to baseline
- Confidence intervals don't overlap with baseline

### Manual Review Required

- Any effect size > 0.8 (large)
- Memory usage increase > 20%
- New allocation patterns detected

## References

- Cohen, J. (1988). Statistical Power Analysis for the Behavioral Sciences
- Criterion.rs documentation: https://bheisler.github.io/criterion.rs/book/
- Rigorous Benchmarking in Reasonable Time: https://dl.acm.org/doi/10.1145/2568821
