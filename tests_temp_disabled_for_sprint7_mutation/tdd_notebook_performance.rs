//! Notebook Performance & Optimization Tests - Sprint 13
//!
//! Tests for performance optimization and scalability:
//! - Lazy cell evaluation and caching
//! - Parallel cell execution
//! - Memory optimization and garbage collection
//! - Large dataset handling
//! - Incremental computation
//! - Performance profiling and metrics
//! - Resource limits and quotas
//! - Optimization recommendations

use ruchy::wasm::notebook::NotebookRuntime;
use std::time::Instant;

// ============================================================================
// Performance Optimization Tests
// ============================================================================

#[test]
fn test_lazy_cell_evaluation() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Add multiple cells but don't execute them yet
    let cell1 = runtime.add_cell("code", "let expensive_data = compute_large_dataset()");
    let cell2 = runtime.add_cell("code", "let filtered = expensive_data.filter(x => x > 100)");
    let cell3 = runtime.add_cell("code", "let result = filtered.sum()");

    // Enable lazy evaluation
    runtime.set_execution_mode("lazy");

    // Mark cells for execution but don't compute yet
    runtime.mark_for_execution(&cell1).unwrap();
    runtime.mark_for_execution(&cell2).unwrap();

    // Only cell3 execution should trigger computation of dependencies
    let start = Instant::now();
    runtime.execute_cell(&cell3).unwrap();
    let duration = start.elapsed();

    // Check lazy evaluation worked
    let execution_stats = runtime.get_execution_statistics();
    assert!(
        execution_stats.contains("lazy_evaluated"),
        "Should use lazy evaluation"
    );
    assert!(
        execution_stats.contains("3"),
        "Should execute 3 cells total"
    );

    println!("Lazy evaluation completed in {:?}", duration);
}

#[test]
fn test_cell_result_caching() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Add expensive computation cell
    let cell = runtime.add_cell("code", "let cached_result = expensive_computation()");

    // First execution
    let start1 = Instant::now();
    runtime.execute_cell(&cell).unwrap();
    let first_duration = start1.elapsed();

    // Second execution should use cache
    let start2 = Instant::now();
    runtime.execute_cell(&cell).unwrap();
    let second_duration = start2.elapsed();

    // Cache should make second execution much faster
    assert!(
        second_duration < first_duration / 10,
        "Cached execution should be >10x faster"
    );

    // Check cache usage
    let cache_stats = runtime.get_cache_statistics();
    assert!(cache_stats.contains("hits\": 1"), "Should have 1 cache hit");

    println!(
        "Cache speedup: {:.2}x",
        first_duration.as_secs_f64() / second_duration.as_secs_f64()
    );
}

#[test]
fn test_parallel_cell_execution() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Add independent cells that can run in parallel
    let cell1 = runtime.add_cell("code", "let dataset1 = load_data('file1.csv')");
    let cell2 = runtime.add_cell("code", "let dataset2 = load_data('file2.csv')");
    let cell3 = runtime.add_cell("code", "let dataset3 = load_data('file3.csv')");

    // Enable parallel execution
    runtime.set_execution_mode("parallel");
    runtime.set_max_workers(3);

    // Execute cells in parallel
    let start = Instant::now();
    runtime
        .execute_cells_parallel(vec![&cell1, &cell2, &cell3])
        .unwrap();
    let parallel_duration = start.elapsed();

    // Compare with sequential execution
    runtime.set_execution_mode("sequential");
    let start_seq = Instant::now();
    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell2).unwrap();
    runtime.execute_cell(&cell3).unwrap();
    let sequential_duration = start_seq.elapsed();

    // Parallel should be faster
    assert!(
        parallel_duration.as_millis() < (sequential_duration.as_millis() * 6 / 10),
        "Parallel execution should be significantly faster"
    );

    println!(
        "Parallel speedup: {:.2}x",
        sequential_duration.as_secs_f64() / parallel_duration.as_secs_f64()
    );
}

#[test]
fn test_memory_optimization() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Create cells with large data
    runtime.add_cell("code", "let big_data1 = create_array(1000000)");
    runtime.add_cell("code", "let big_data2 = create_array(1000000)");
    runtime.add_cell("code", "let big_data3 = create_array(1000000)");

    // Get initial memory usage
    let initial_memory = runtime.get_memory_usage();
    let initial_bytes: u64 = serde_json::from_str(&initial_memory)
        .ok()
        .and_then(|v: serde_json::Value| v["total_allocated"].as_u64())
        .unwrap_or(0);

    // Enable memory optimization
    runtime.enable_memory_optimization(true);
    runtime.set_memory_limit(10_000_000); // 10MB limit

    // Execute cells with memory management
    runtime.execute_all_cells().unwrap();

    // Trigger garbage collection
    runtime.run_garbage_collection().unwrap();

    // Check memory after GC
    let final_memory = runtime.get_memory_usage();
    let final_bytes: u64 = serde_json::from_str(&final_memory)
        .ok()
        .and_then(|v: serde_json::Value| v["total_allocated"].as_u64())
        .unwrap_or(0);

    // Memory should be managed efficiently
    assert!(
        final_bytes < initial_bytes * 2,
        "Memory usage should be controlled"
    );

    println!(
        "Memory optimization: {:.2}% reduction",
        (1.0 - final_bytes as f64 / initial_bytes as f64) * 100.0
    );
}

#[test]
fn test_large_dataset_handling() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable streaming for large datasets
    runtime.enable_streaming_mode(true);
    runtime.set_chunk_size(1000);

    // Process large dataset in chunks
    let cell = runtime.add_cell(
        "code",
        "let result = process_large_dataset('huge_file.parquet')",
    );

    // Execute with progress tracking
    runtime
        .execute_cell_with_progress(&cell, |progress| {
            println!("Processing: {}%", progress.percentage);
        })
        .unwrap();

    // Check streaming was used
    let execution_info = runtime.get_last_execution_info();
    assert!(execution_info.contains("streaming"), "Should use streaming");
    assert!(
        execution_info.contains("chunks"),
        "Should process in chunks"
    );

    println!("Large dataset processing completed");
}

#[test]
fn test_incremental_computation() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable incremental computation
    runtime.enable_incremental_mode(true);

    // Create computation pipeline
    let cell1 = runtime.add_cell("code", "let base = load_data()");
    let cell2 = runtime.add_cell("code", "let transformed = base.map(transform)");
    let cell3 = runtime.add_cell("code", "let aggregated = transformed.group_by('category')");

    // Execute pipeline
    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell2).unwrap();
    runtime.execute_cell(&cell3).unwrap();

    // Modify only the transform function
    runtime.update_cell(&cell2, "let transformed = base.map(new_transform)");

    // Re-execute should only recompute affected cells
    let start = Instant::now();
    runtime.execute_incremental(&cell3).unwrap();
    let incremental_duration = start.elapsed();

    // Check incremental execution
    let stats = runtime.get_incremental_stats();
    assert!(
        stats.contains("cells_recomputed\": 2"),
        "Should only recompute cell2 and cell3"
    );
    assert!(
        stats.contains("cells_skipped\": 1"),
        "Should skip cell1 (unchanged)"
    );

    println!("Incremental computation saved: {:?}", incremental_duration);
}

#[test]
fn test_performance_profiling() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable profiling
    runtime.enable_profiling(true);

    // Execute various operations
    let cell1 = runtime.add_cell("code", "let data = generate_data(1000)");
    let cell2 = runtime.add_cell("code", "let processed = data.apply(complex_transform)");
    let cell3 = runtime.add_cell("code", "let result = processed.aggregate()");

    runtime.execute_cell(&cell1).unwrap();
    runtime.execute_cell(&cell2).unwrap();
    runtime.execute_cell(&cell3).unwrap();

    // Get profiling report
    let profile = runtime.get_performance_profile();

    // Check profiling data
    assert!(
        profile.contains("execution_times"),
        "Should track execution times"
    );
    assert!(
        profile.contains("memory_peaks"),
        "Should track memory peaks"
    );
    assert!(profile.contains("cpu_usage"), "Should track CPU usage");
    assert!(
        profile.contains("bottlenecks"),
        "Should identify bottlenecks"
    );

    // Get optimization suggestions
    let suggestions = runtime.get_optimization_suggestions();
    assert!(
        !suggestions.is_empty(),
        "Should provide optimization suggestions"
    );

    println!(
        "Performance profile generated with {} suggestions",
        suggestions.lines().count()
    );
}

#[test]
fn test_resource_limits() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Set resource limits
    runtime.set_memory_limit(50_000_000); // 50MB
    runtime.set_cpu_time_limit(5000); // 5 seconds
    runtime.set_max_output_size(1_000_000); // 1MB output

    // Try to exceed memory limit
    let cell = runtime.add_cell("code", "let huge = allocate_memory(100_000_000)");
    let result = runtime.execute_cell(&cell);

    assert!(result.is_err(), "Should fail when exceeding memory limit");
    assert!(
        result.unwrap_err().contains("memory limit"),
        "Should report memory limit exceeded"
    );

    // Check resource tracking
    let resources = runtime.get_resource_usage();
    assert!(
        resources.contains("memory_limit"),
        "Should track memory limit"
    );
    assert!(resources.contains("cpu_time"), "Should track CPU time");

    println!("Resource limits enforced successfully");
}

#[test]
fn test_smart_dependency_tracking() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable smart dependency tracking
    runtime.enable_smart_dependencies(true);

    // Create complex dependency graph
    runtime.add_cell("code", "let a = 1");
    runtime.add_cell("code", "let b = a + 2");
    runtime.add_cell("code", "let c = a + 3");
    runtime.add_cell("code", "let d = b + c");
    runtime.add_cell("code", "let e = d * 2");

    // Analyze dependencies
    let deps = runtime.analyze_dependencies();

    // Check dependency analysis
    assert!(
        deps.contains("execution_order"),
        "Should determine execution order"
    );
    assert!(
        deps.contains("parallel_groups"),
        "Should identify parallel groups"
    );
    assert!(deps.contains("critical_path"), "Should find critical path");

    // Get optimal execution plan
    let plan = runtime.get_optimal_execution_plan();
    assert!(
        plan.contains("parallel"),
        "Should suggest parallel execution"
    );

    println!(
        "Smart dependency tracking identified {} parallel groups",
        plan.matches("parallel").count()
    );
}

#[test]
fn test_notebook_compilation() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Add cells to compile
    runtime.add_cell("code", "let x = 42");
    runtime.add_cell("code", "let y = x * 2");
    runtime.add_cell("code", "println(y)");

    // Compile notebook to optimized format
    let compiled = runtime.compile_notebook().unwrap();

    // Check compilation
    assert!(
        compiled.contains("compiled_version"),
        "Should have compiled version"
    );
    assert!(
        compiled.contains("optimizations"),
        "Should list optimizations"
    );
    assert!(compiled.contains("bytecode"), "Should generate bytecode");

    // Execute compiled version should be faster
    let start_interpreted = Instant::now();
    runtime.execute_all_cells().unwrap();
    let interpreted_time = start_interpreted.elapsed();

    let start_compiled = Instant::now();
    runtime.execute_compiled(&compiled).unwrap();
    let compiled_time = start_compiled.elapsed();

    assert!(
        compiled_time < interpreted_time,
        "Compiled execution should be faster"
    );

    println!(
        "Compilation speedup: {:.2}x",
        interpreted_time.as_secs_f64() / compiled_time.as_secs_f64()
    );
}

// ============================================================================
// Advanced Optimization Tests
// ============================================================================

#[test]
fn test_query_optimization() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Add data query cells
    runtime.add_cell("code", "let df = load_dataframe('data.csv')");
    runtime.add_cell("code", "let filtered = df.filter(col('value') > 100)");
    runtime.add_cell("code", "let grouped = filtered.group_by('category')");
    runtime.add_cell("code", "let result = grouped.agg(sum('value'))");

    // Enable query optimization
    runtime.enable_query_optimization(true);

    // Optimize execution plan
    let optimized_plan = runtime.optimize_query_plan().unwrap();

    // Check optimizations
    assert!(
        optimized_plan.contains("predicate_pushdown"),
        "Should push filters down"
    );
    assert!(
        optimized_plan.contains("projection_pruning"),
        "Should prune unnecessary columns"
    );
    assert!(
        optimized_plan.contains("join_reordering"),
        "Should optimize join order"
    );

    println!(
        "Query optimization applied {} transformations",
        optimized_plan.matches("optimization").count()
    );
}

#[test]
fn test_auto_scaling() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable auto-scaling
    runtime.enable_auto_scaling(true);
    runtime.set_scaling_policy("adaptive");

    // Start with minimal resources
    runtime.set_initial_workers(1);

    // Add workload that triggers scaling
    for i in 0..10 {
        runtime.add_cell("code", &format!("let data{} = process_batch({})", i, i));
    }

    // Execute with auto-scaling
    runtime.execute_all_cells().unwrap();

    // Check scaling occurred
    let scaling_metrics = runtime.get_scaling_metrics();
    assert!(scaling_metrics.contains("scaled_up"), "Should scale up");
    assert!(
        scaling_metrics.contains("max_workers"),
        "Should track max workers"
    );
    assert!(
        scaling_metrics.contains("efficiency"),
        "Should measure efficiency"
    );

    println!(
        "Auto-scaling achieved {}% efficiency",
        scaling_metrics
            .split("efficiency\":")
            .nth(1)
            .and_then(|s| s.split(',').next())
            .unwrap_or("0")
    );
}

#[test]
fn test_intelligent_caching() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable intelligent caching with LRU policy
    runtime.enable_intelligent_caching(true);
    runtime.set_cache_policy("lru");
    runtime.set_cache_size(100_000_000); // 100MB cache

    // Create cells with different access patterns
    let frequently_used = runtime.add_cell("code", "let freq = expensive_calc1()");
    let rarely_used = runtime.add_cell("code", "let rare = expensive_calc2()");
    let never_reused = runtime.add_cell("code", "let temp = expensive_calc3()");

    // Execute multiple times with different patterns
    for _ in 0..5 {
        runtime.execute_cell(&frequently_used).unwrap();
    }
    runtime.execute_cell(&rarely_used).unwrap();
    runtime.execute_cell(&never_reused).unwrap();

    // Check cache effectiveness
    let cache_stats = runtime.get_cache_statistics();
    let hit_rate: f64 = serde_json::from_str(&cache_stats)
        .ok()
        .and_then(|v: serde_json::Value| v["hit_rate"].as_f64())
        .unwrap_or(0.0);

    assert!(hit_rate > 0.7, "Cache hit rate should be >70%");

    // Check intelligent eviction
    assert!(cache_stats.contains("evicted"), "Should track evictions");
    assert!(
        cache_stats.contains("freq"),
        "Frequently used should be in cache"
    );

    println!(
        "Intelligent caching achieved {:.1}% hit rate",
        hit_rate * 100.0
    );
}

#[test]
fn test_distributed_execution() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Configure distributed execution
    runtime.enable_distributed_mode(true);
    runtime.add_worker_node("worker1", "ws://localhost:8081");
    runtime.add_worker_node("worker2", "ws://localhost:8082");

    // Create distributed workload
    let cells = (0..20)
        .map(|i| {
            runtime.add_cell(
                "code",
                &format!("let partition{} = process_partition({})", i, i),
            )
        })
        .collect::<Vec<_>>();

    // Execute across workers
    let start = Instant::now();
    runtime.execute_distributed(&cells).unwrap();
    let distributed_time = start.elapsed();

    // Get distribution metrics
    let metrics = runtime.get_distribution_metrics();
    assert!(metrics.contains("worker1"), "Should use worker1");
    assert!(metrics.contains("worker2"), "Should use worker2");
    assert!(metrics.contains("load_balance"), "Should balance load");

    println!(
        "Distributed execution across {} workers in {:?}",
        2, distributed_time
    );
}

#[test]
fn test_predictive_prefetching() {
    let mut runtime = NotebookRuntime::new().unwrap();

    // Enable predictive prefetching
    runtime.enable_predictive_prefetch(true);
    runtime.train_prediction_model().unwrap();

    // Create predictable execution pattern
    runtime.add_cell("code", "let dataset = load('data1.csv')");
    runtime.add_cell("code", "let processed = dataset.transform()");
    runtime.add_cell("code", "let next_data = load('data2.csv')");

    // Execute with prefetching
    runtime.execute_all_cells().unwrap();

    // Check prefetching occurred
    let prefetch_stats = runtime.get_prefetch_statistics();
    assert!(
        prefetch_stats.contains("prefetched"),
        "Should prefetch data"
    );
    assert!(prefetch_stats.contains("accuracy"), "Should track accuracy");

    let accuracy: f64 = serde_json::from_str(&prefetch_stats)
        .ok()
        .and_then(|v: serde_json::Value| v["accuracy"].as_f64())
        .unwrap_or(0.0);

    println!("Predictive prefetching accuracy: {:.1}%", accuracy * 100.0);
}
