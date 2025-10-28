# Ruchy Performance Benchmarks

Performance benchmarking infrastructure for Ruchy data science workflows.

## Matrix Data Science Benchmarks

Located in: `benches/matrix_data_science_benchmarks.rs`

Benchmarks the 42 matrix test workflows from Phase 4 Week 2 to establish performance baselines.

### Performance Targets

Based on Phase 4 Notebook Excellence specification:

| Operation Category | Target Performance | Notes |
|-------------------|-------------------|-------|
| Simple Arithmetic | <1ms per operation | Addition, subtraction, multiplication, division |
| Array Operations | <1ms per operation | Filter, map, reduce on small arrays (<100 items) |
| CSV Processing (1000 items) | <10ms | Filter-map-reduce pipelines |
| Statistical Computations | <5ms | Mean, sum of squares, weighted averages |
| Time Series Analysis (1000 points) | <10ms | Moving averages, momentum, ROC |

### Running Benchmarks

```bash
# Run all matrix benchmarks
cargo bench --bench matrix_data_science_benchmarks

# Run specific benchmark group
cargo bench --bench matrix_data_science_benchmarks arithmetic_benches
cargo bench --bench matrix_data_science_benchmarks csv_benches
cargo bench --bench matrix_data_science_benchmarks stats_benches
cargo bench --bench matrix_data_science_benchmarks timeseries_benches

# Run with Criterion HTML reports
cargo bench --bench matrix_data_science_benchmarks -- --save-baseline baseline_v1
```

### Benchmark Categories

#### 1. Arithmetic Benchmarks (Matrix 001)
- `matrix_001_arithmetic_addition` - Simple addition (10 + 20)
- `matrix_001_arithmetic_subtraction` - Simple subtraction (50 - 30)
- `matrix_001_arithmetic_multiplication` - Simple multiplication (6 * 7)
- `matrix_001_arithmetic_division` - Simple division (100 / 4)

#### 2. CSV Processing Benchmarks (Matrix 002)
- `matrix_002_csv_array_creation` - Parametric: 10, 100, 1000 elements
- `matrix_002_csv_filter` - Filter array (keep > 25)
- `matrix_002_csv_map` - Map array (double each element)
- `matrix_002_csv_reduce` - Reduce array (sum all elements)
- `matrix_002_csv_filter_map_reduce_pipeline` - Complete pipeline

#### 3. Statistical Analysis Benchmarks (Matrix 003)
- `matrix_003_statistical_mean` - Parametric: 10, 100, 1000 elements
- `matrix_003_stats_sum` - Sum of array
- `matrix_003_stats_sum_of_squares` - Sum of squares calculation
- `matrix_003_stats_weighted_average` - Weighted average computation
- `matrix_003_stats_normalization` - Min-max normalization

#### 4. Time Series Analysis Benchmarks (Matrix 004)
- `matrix_004_timeseries_sma` - Simple moving average
- `matrix_004_timeseries_percent_change` - Percent change calculation
- `matrix_004_timeseries_cumulative_sum` - Parametric: 10, 100, 1000 elements
- `matrix_004_timeseries_momentum` - Momentum indicator
- `matrix_004_timeseries_roc` - Rate of change
- `matrix_004_timeseries_exp_weighting` - Exponential weighting
- `matrix_004_timeseries_anomaly_detection` - Anomaly detection threshold

### Parametric Benchmarks

Some benchmarks test performance at different scales:
- **Small**: 10 elements (baseline)
- **Medium**: 100 elements (typical use case)
- **Large**: 1000 elements (stress test)

This helps identify O(n) vs O(n²) complexity issues.

### Viewing Results

Criterion generates HTML reports in:
- `target/criterion/report/index.html` - Summary of all benchmarks
- `target/criterion/<benchmark_name>/report/index.html` - Individual benchmark details

### Adding New Benchmarks

When adding new matrix tests, follow this pattern:

```rust
fn bench_new_operation(c: &mut Criterion) {
    c.bench_function("matrix_00X_operation_name", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from("."))
                .expect("Failed to create REPL");
            let result = repl.eval("your_code_here");
            black_box(result)
        });
    });
}
```

Add to appropriate criterion_group at bottom of file.

### Regression Detection

Criterion automatically detects performance regressions:
- **Green**: <5% change (noise)
- **Yellow**: 5-10% change (investigate)
- **Red**: >10% change (regression - requires fix)

### CI Integration

Benchmarks run in CI on:
- Pull requests (compare against main branch baseline)
- Main branch commits (establish new baseline)
- Nightly builds (track long-term trends)

## Other Benchmarks

- `bytecode_vm_performance.rs` - Bytecode VM vs AST interpreter comparison
- `bytecode_vs_ast.rs` - Execution mode performance
- `compilation_bench.rs` - Compile-time performance
- `execution_bench.rs` - REPL evaluation performance
- `interpreter_benchmarks.rs` - Core interpreter operations
- `parser_benchmarks.rs` - Parser performance
- `parser.rs` - Low-level parsing benchmarks

## References

- Phase 4 Specification: `docs/specs/PHASE4-NOTEBOOK-EXCELLENCE.md`
- Criterion.rs Documentation: https://bheisler.github.io/criterion.rs/book/
- Matrix Test Suite: `tests/matrix_*_native.rs`
