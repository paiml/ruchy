// Matrix Data Science Benchmarks - Phase 4 Week 3 Performance Testing
//
// Benchmarks the 42 matrix test workflows to establish performance baselines
// and identify optimization opportunities.
//
// Reference: docs/specs/PHASE4-NOTEBOOK-EXCELLENCE.md - Week 3
//
// Performance Targets (from Phase 4 spec):
// - Array operations: <1ms per operation
// - CSV-style processing (1000 items): <10ms
// - Statistical computations: <5ms
// - Time series analysis: <10ms for 1000 data points

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ruchy::runtime::repl::Repl;
use std::path::PathBuf;

// ============================================================================
// Section 1: Simple Arithmetic Benchmarks (Matrix 001)
// ============================================================================

fn bench_arithmetic_addition(c: &mut Criterion) {
    c.bench_function("matrix_001_arithmetic_addition", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval("10 + 20");
            black_box(result)
        });
    });
}

fn bench_arithmetic_subtraction(c: &mut Criterion) {
    c.bench_function("matrix_001_arithmetic_subtraction", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval("50 - 30");
            black_box(result)
        });
    });
}

fn bench_arithmetic_multiplication(c: &mut Criterion) {
    c.bench_function("matrix_001_arithmetic_multiplication", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval("6 * 7");
            black_box(result)
        });
    });
}

fn bench_arithmetic_division(c: &mut Criterion) {
    c.bench_function("matrix_001_arithmetic_division", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval("100 / 4");
            black_box(result)
        });
    });
}

// ============================================================================
// Section 2: CSV Processing Benchmarks (Matrix 002)
// ============================================================================

fn bench_csv_array_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_002_csv_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("array_creation", size), size, |b, &size| {
            b.iter(|| {
                let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
                // Create array with 'size' elements
                let code = format!("[{}]", (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
                let result = repl.eval(&code);
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_csv_filter_operation(c: &mut Criterion) {
    c.bench_function("matrix_002_csv_filter", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Filter array: keep elements > 25
            let result = repl.eval("[10, 20, 30, 40, 50].filter(|x| x > 25)");
            black_box(result)
        });
    });
}

fn bench_csv_map_operation(c: &mut Criterion) {
    c.bench_function("matrix_002_csv_map", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Map array: double each element
            let result = repl.eval("[1, 2, 3, 4, 5].map(|x| x * 2)");
            black_box(result)
        });
    });
}

fn bench_csv_reduce_operation(c: &mut Criterion) {
    c.bench_function("matrix_002_csv_reduce", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Reduce array: sum all elements
            let result = repl.eval("[1, 2, 3, 4, 5].reduce(|acc, x| acc + x, 0)");
            black_box(result)
        });
    });
}

fn bench_csv_filter_map_reduce_pipeline(c: &mut Criterion) {
    c.bench_function("matrix_002_csv_filter_map_reduce_pipeline", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Complete pipeline: filter > 30, map salary extraction, reduce to sum
            let result = repl.eval(
                "[[1, 25, 50000], [2, 35, 75000], [3, 45, 100000], [4, 32, 80000]]\
                .filter(|row| row[1] > 30)\
                .map(|row| row[2])\
                .reduce(|acc, x| acc + x, 0)"
            );
            black_box(result)
        });
    });
}

// ============================================================================
// Section 3: Statistical Analysis Benchmarks (Matrix 003)
// ============================================================================

fn bench_stats_mean_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_003_statistical_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("mean", size), size, |b, &size| {
            b.iter(|| {
                let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
                // Generate array and calculate mean
                let data = (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                let code = format!("let data = [{}]; data.reduce(|acc, x| acc + x, 0) / data.len()", data);
                let result = repl.eval(&code);
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_stats_sum(c: &mut Criterion) {
    c.bench_function("matrix_003_stats_sum", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval("[1, 2, 3, 4, 5].reduce(|acc, x| acc + x, 0)");
            black_box(result)
        });
    });
}

fn bench_stats_sum_of_squares(c: &mut Criterion) {
    c.bench_function("matrix_003_stats_sum_of_squares", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval("[1, 2, 3, 4, 5].map(|x| x * x).reduce(|acc, x| acc + x, 0)");
            black_box(result)
        });
    });
}

fn bench_stats_weighted_average(c: &mut Criterion) {
    c.bench_function("matrix_003_stats_weighted_average", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            let result = repl.eval(
                "let data = [[80, 2], [90, 3], [85, 1]];\
                data.map(|pair| pair[0] * pair[1]).reduce(|acc, x| acc + x, 0)"
            );
            black_box(result)
        });
    });
}

fn bench_stats_normalization(c: &mut Criterion) {
    c.bench_function("matrix_003_stats_normalization", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Min-max normalization
            let result = repl.eval("((50 - 0) * 10) / (100 - 0)");
            black_box(result)
        });
    });
}

// ============================================================================
// Section 4: Time Series Analysis Benchmarks (Matrix 004)
// ============================================================================

fn bench_timeseries_moving_average(c: &mut Criterion) {
    c.bench_function("matrix_004_timeseries_sma", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Simple moving average calculation
            let result = repl.eval(
                "let window = [10, 20, 30];\
                window.reduce(|acc, x| acc + x, 0) / window.len()"
            );
            black_box(result)
        });
    });
}

fn bench_timeseries_percent_change(c: &mut Criterion) {
    c.bench_function("matrix_004_timeseries_percent_change", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Percent change calculation
            let result = repl.eval("((120 - 100) * 100) / 100");
            black_box(result)
        });
    });
}

fn bench_timeseries_cumulative_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_004_timeseries_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("cumulative_sum", size), size, |b, &size| {
            b.iter(|| {
                let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
                let data = (1..=size).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                let code = format!("let data = [{}]; data.reduce(|acc, x| acc + x, 0)", data);
                let result = repl.eval(&code);
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_timeseries_momentum(c: &mut Criterion) {
    c.bench_function("matrix_004_timeseries_momentum", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Momentum calculation: current - past
            let result = repl.eval("150 - 120");
            black_box(result)
        });
    });
}

fn bench_timeseries_rate_of_change(c: &mut Criterion) {
    c.bench_function("matrix_004_timeseries_roc", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Rate of Change (ROC)
            let result = repl.eval("((110 - 100) * 100) / 100");
            black_box(result)
        });
    });
}

fn bench_timeseries_exponential_weighting(c: &mut Criterion) {
    c.bench_function("matrix_004_timeseries_exp_weighting", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Exponential weighting calculation
            let result = repl.eval("(100 * 7 + 80 * 3) / 10");
            black_box(result)
        });
    });
}

fn bench_timeseries_anomaly_detection(c: &mut Criterion) {
    c.bench_function("matrix_004_timeseries_anomaly_detection", |b| {
        b.iter(|| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            // Anomaly detection: deviation from mean
            let result = repl.eval("150 - 100");
            black_box(result)
        });
    });
}

// ============================================================================
// Criterion Benchmark Groups
// ============================================================================

criterion_group!(
    arithmetic_benches,
    bench_arithmetic_addition,
    bench_arithmetic_subtraction,
    bench_arithmetic_multiplication,
    bench_arithmetic_division
);

criterion_group!(
    csv_benches,
    bench_csv_array_creation,
    bench_csv_filter_operation,
    bench_csv_map_operation,
    bench_csv_reduce_operation,
    bench_csv_filter_map_reduce_pipeline
);

criterion_group!(
    stats_benches,
    bench_stats_mean_calculation,
    bench_stats_sum,
    bench_stats_sum_of_squares,
    bench_stats_weighted_average,
    bench_stats_normalization
);

criterion_group!(
    timeseries_benches,
    bench_timeseries_moving_average,
    bench_timeseries_percent_change,
    bench_timeseries_cumulative_sum,
    bench_timeseries_momentum,
    bench_timeseries_rate_of_change,
    bench_timeseries_exponential_weighting,
    bench_timeseries_anomaly_detection
);

criterion_main!(
    arithmetic_benches,
    csv_benches,
    stats_benches,
    timeseries_benches
);
