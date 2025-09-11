//! Performance benchmarks for SharedSession operations
//! 
//! Measures critical SharedSession performance characteristics:
//! - Cell execution latency
//! - DataFrame operation throughput  
//! - Memory allocation patterns
//! - Concurrent execution scalability

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ruchy::wasm::shared_session::{SharedSession, ExecutionMode};
use ruchy::wasm::notebook::NotebookRuntime;
use std::time::Duration;

fn benchmark_cell_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell_execution");
    
    // Test different expression complexities
    let test_cases = vec![
        ("simple_arithmetic", "42 + 58"),
        ("variable_assignment", "let x = 100; x * 2"),
        ("string_operations", r#"let name = "Ruchy"; f"Hello {name}!""#),
        ("array_operations", "[1, 2, 3, 4, 5].map(x => x * 2).sum()"),
        ("dataframe_creation", "DataFrame([[1, 2], [3, 4], [5, 6]])"),
    ];
    
    for (name, code) in test_cases {
        group.bench_with_input(BenchmarkId::new("execute", name), code, |b, code| {
            b.iter_batched(
                || SharedSession::new(),
                |mut session| {
                    black_box(session.execute("bench_cell", code))
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

fn benchmark_dataframe_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("dataframe_operations");
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark different DataFrame sizes
    let sizes = vec![100, 1000, 10000];
    
    for size in sizes {
        // DataFrame creation benchmark
        group.bench_with_input(BenchmarkId::new("creation", size), &size, |b, &size| {
            b.iter_batched(
                || SharedSession::new(),
                |mut session| {
                    let code = format!("DataFrame::from_range(0, {})", size);
                    black_box(session.execute("bench_creation", &code))
                },
                criterion::BatchSize::SmallInput,
            );
        });
        
        // DataFrame filtering benchmark
        group.bench_with_input(BenchmarkId::new("filter", size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let mut session = SharedSession::new();
                    let create_code = format!("let df = DataFrame::from_range(0, {})", size);
                    session.execute("setup", &create_code).unwrap();
                    session
                },
                |mut session| {
                    black_box(session.execute("bench_filter", r#"df.filter(col("value") % 2 == 0)"#))
                },
                criterion::BatchSize::SmallInput,
            );
        });
        
        // DataFrame aggregation benchmark
        group.bench_with_input(BenchmarkId::new("sum", size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let mut session = SharedSession::new();
                    let create_code = format!("let df = DataFrame::from_range(0, {})", size);
                    session.execute("setup", &create_code).unwrap();
                    session
                },
                |mut session| {
                    black_box(session.execute("bench_sum", "df.sum()"))
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    
    group.finish();
}

fn benchmark_reactive_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("reactive_execution");
    
    // Test reactive dependency chains
    group.bench_function("linear_dependency_chain", |b| {
        b.iter_batched(
            || {
                let mut session = SharedSession::new();
                session.set_execution_mode(ExecutionMode::Reactive);
                // Set up dependency chain: a -> b -> c -> d
                session.execute("cell_a", "let a = 10").unwrap();
                session.execute("cell_b", "let b = a * 2").unwrap();
                session.execute("cell_c", "let c = b + 5").unwrap();
                session.execute("cell_d", "let d = c * c").unwrap();
                session
            },
            |mut session| {
                // Modify root - should trigger reactive execution of entire chain
                black_box(session.execute_reactive("cell_a", "let a = 20"))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    // Test diamond dependency pattern
    group.bench_function("diamond_dependency", |b| {
        b.iter_batched(
            || {
                let mut session = SharedSession::new();
                session.set_execution_mode(ExecutionMode::Reactive);
                // Set up diamond: root -> left/right -> merge
                session.execute("root", "let x = 5").unwrap();
                session.execute("left", "let left = x * 2").unwrap();
                session.execute("right", "let right = x + 10").unwrap();
                session.execute("merge", "let result = left + right").unwrap();
                session
            },
            |mut session| {
                // Modify root - should trigger all dependent cells
                black_box(session.execute_reactive("root", "let x = 15"))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // Test memory usage with large DataFrames
    group.bench_function("large_dataframe_memory", |b| {
        b.iter_batched(
            || NotebookRuntime::new().unwrap(),
            |mut runtime| {
                // Create large DataFrame and measure memory impact
                let initial_memory = runtime.get_memory_usage();
                runtime.execute_cell_with_session("large_df", "let df = DataFrame::from_range(0, 50000)").unwrap();
                let after_memory = runtime.get_memory_usage();
                
                // Return memory difference as performance metric
                black_box((initial_memory, after_memory))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    // Test checkpoint memory overhead
    group.bench_function("checkpoint_overhead", |b| {
        b.iter_batched(
            || {
                let mut session = SharedSession::new();
                // Create some state to checkpoint
                session.execute("setup1", "let x = 42").unwrap();
                session.execute("setup2", "let y = DataFrame([[1, 2], [3, 4]])").unwrap();
                session
            },
            |mut session| {
                // Create checkpoint and measure performance
                black_box(session.create_checkpoint("perf_test"))
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

fn benchmark_concurrent_safety(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_safety");
    
    // Test thread-safe SharedSession operations
    group.bench_function("concurrent_reads", |b| {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        b.iter_batched(
            || {
                let mut session = SharedSession::new();
                session.execute("shared_data", "let shared = DataFrame::from_range(0, 1000)").unwrap();
                Arc::new(Mutex::new(session))
            },
            |session| {
                let handles: Vec<_> = (0..4).map(|i| {
                    let session = Arc::clone(&session);
                    thread::spawn(move || {
                        let session = session.lock().unwrap();
                        // Read shared data concurrently
                        let globals = session.globals.serialize_for_inspection();
                        black_box(globals)
                    })
                }).collect();
                
                // Wait for all threads to complete
                for handle in handles {
                    handle.join().unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_cell_execution,
    benchmark_dataframe_operations,
    benchmark_reactive_execution,
    benchmark_memory_efficiency,
    benchmark_concurrent_safety
);
criterion_main!(benches);