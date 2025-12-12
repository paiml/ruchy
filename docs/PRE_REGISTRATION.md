# Pre-Registration Document

This document pre-registers the experimental design and hypotheses for Ruchy's performance claims.

## Study Information

**Title**: Performance Evaluation of Ruchy Programming Language
**Version**: 1.0
**Date**: 2024-01-01
**Authors**: Noah Gift

## Hypotheses

### H1: Transpilation Efficiency

**Hypothesis**: Ruchy transpilation to Rust will produce code with runtime performance within 10% of hand-written Rust.

**Operationalization**:
- Measure: Wall-clock execution time
- Baseline: Equivalent hand-written Rust code
- Threshold: Mean runtime ≤ 1.1x baseline

**Prediction**: Effect size d < 0.2 (small or negligible difference)

### H2: Parse Performance

**Hypothesis**: Ruchy parser will process 1000 lines of code in under 10ms on reference hardware.

**Operationalization**:
- Measure: Parse time (AST generation)
- Input: Standardized 1000-line test file
- Hardware: GitHub Actions runner (2-core, 7GB RAM)

**Prediction**: 95% CI upper bound < 15ms

### H3: Memory Efficiency

**Hypothesis**: Ruchy runtime memory usage will be within 20% of equivalent Rust programs.

**Operationalization**:
- Measure: Peak RSS during execution
- Baseline: Equivalent Rust program
- Threshold: Peak RSS ≤ 1.2x baseline

**Prediction**: Effect size d < 0.3

### H4: Error Classification Accuracy

**Hypothesis**: The Oracle ML classifier will achieve ≥85% accuracy on error categorization.

**Operationalization**:
- Measure: Classification accuracy (correct / total)
- Test set: Held-out 20% of labeled errors
- Cross-validation: 5-fold

**Prediction**: Mean accuracy ≥ 0.85, 95% CI lower bound ≥ 0.80

## Methods

### Sampling Plan

**Inclusion criteria**:
- Valid Ruchy source files
- Parse successfully to AST
- No external dependencies

**Sample size justification**:
- Power analysis: α=0.05, β=0.20, d=0.5
- Required n ≥ 64 per condition
- Actual: n = 100 benchmark iterations

### Variables

**Independent variables**:
- Input size (lines of code)
- Code complexity (cyclomatic)
- Operation type (parse, transpile, execute)

**Dependent variables**:
- Execution time (ms)
- Memory usage (bytes)
- Classification accuracy (proportion)

**Controlled variables**:
- Hardware (CI runner specifications)
- Rust version (1.83.0)
- Compiler flags (release, LTO)
- Random seeds (42)

### Analysis Plan

**Primary analysis**:
- Two-sided t-tests for continuous outcomes
- Chi-square for categorical outcomes
- α = 0.05 (Bonferroni-corrected for multiple comparisons)

**Effect size reporting**:
- Cohen's d for continuous
- Odds ratio for categorical

**Confidence intervals**:
- 95% CI via bootstrap (10,000 resamples)

## Data Exclusion

**Pre-specified exclusion criteria**:
- Benchmark runs with CPU utilization < 90% (interference)
- Outliers > 3.5 MAD from median
- Failed executions (reported separately)

**No post-hoc exclusions** will be performed.

## Deviation Log

Any deviations from this pre-registration will be documented here with justification.

| Date | Deviation | Justification |
|------|-----------|---------------|
| - | None | - |

## Registration

This pre-registration is committed to the repository as of commit hash: [TO BE FILLED AT COMMIT]

Timestamp: 2024-01-01T00:00:00Z
