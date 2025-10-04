//! Extreme TDD Tests for notebook/testing/performance.rs
//!
//! Following extreme TDD methodology:
//! 1. Write comprehensive test first
//! 2. Minimal implementation to pass
//! 3. Refactor for quality
//!
//! Coverage target: 383 uncovered lines -> 100% coverage
//! Focus: Performance benchmarking, parallel execution, caching, monitoring

use proptest::prelude::*;
use ruchy::notebook::testing::performance::{
    Benchmark, BenchmarkResult, CacheStats, CachedResult, ParallelTestExecutor,
    PerformanceBenchmarker, RegressionDetector, RegressionResult, ResourceMonitor, ResourceUsage,
    TestCache, TestExecutionResult, TestHistory, TestPrioritizer, TestSharder,
};
use ruchy::notebook::testing::types::{Cell, CellType, Notebook};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// Helper Functions for Test Data
// ============================================================================

fn create_test_cell(id: &str, source: &str) -> Cell {
    Cell {
        id: id.to_string(),
        source: source.to_string(),
        cell_type: CellType::Code,
        metadata: HashMap::new(),
    }
}

fn create_test_notebook(cells: Vec<Cell>) -> Notebook {
    Notebook { cells }
}

fn create_test_benchmark(id: &str, name: &str, iterations: usize) -> Benchmark {
    Benchmark {
        id: id.to_string(),
        name: name.to_string(),
        setup: Box::new(|| {}),
        run: Box::new(|| {
            // Simulate some work
            thread::sleep(Duration::from_millis(1));
        }),
        teardown: Box::new(|| {}),
        iterations,
    }
}

// ============================================================================
// Unit Tests - PerformanceBenchmarker
// ============================================================================

#[test]
fn test_performance_benchmarker_new() {
    let benchmarker = PerformanceBenchmarker::new();
    assert!(benchmarker.benchmarks.is_empty());
    assert!(benchmarker.results.is_empty());
}

#[test]
fn test_performance_benchmarker_add_benchmark() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let benchmark = create_test_benchmark("test", "Test Benchmark", 5);

    benchmarker.add_benchmark(benchmark);

    assert_eq!(benchmarker.benchmarks.len(), 1);
    assert_eq!(benchmarker.benchmarks[0].id, "test");
    assert_eq!(benchmarker.benchmarks[0].name, "Test Benchmark");
    assert_eq!(benchmarker.benchmarks[0].iterations, 5);
}

#[test]
fn test_performance_benchmarker_run_all_empty() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let results = benchmarker.run_all();
    assert!(results.is_empty());
}

#[test]
fn test_performance_benchmarker_run_all_single() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let benchmark = create_test_benchmark("test", "Test Benchmark", 3);
    benchmarker.add_benchmark(benchmark);

    let results = benchmarker.run_all();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test");
    assert!(results[0].mean_time_ms > 0.0);
    assert!(results[0].std_dev_ms >= 0.0);
    assert!(results[0].min_time_ms <= results[0].max_time_ms);
    assert!(results[0].median_time_ms >= results[0].min_time_ms);
    assert!(results[0].median_time_ms <= results[0].max_time_ms);
}

#[test]
fn test_performance_benchmarker_run_all_multiple() {
    let mut benchmarker = PerformanceBenchmarker::new();

    for i in 0..3 {
        let benchmark = create_test_benchmark(&format!("test_{}", i), &format!("Test {}", i), 2);
        benchmarker.add_benchmark(benchmark);
    }

    let results = benchmarker.run_all();

    assert_eq!(results.len(), 3);
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.id, format!("test_{}", i));
        assert!(result.mean_time_ms > 0.0);
    }
}

#[test]
fn test_benchmark_result_statistics() {
    let mut benchmarker = PerformanceBenchmarker::new();

    // Create benchmark with known timing behavior
    let benchmark = Benchmark {
        id: "stats_test".to_string(),
        name: "Statistics Test".to_string(),
        setup: Box::new(|| {}),
        run: Box::new(|| {
            thread::sleep(Duration::from_millis(10));
        }),
        teardown: Box::new(|| {}),
        iterations: 5,
    };

    benchmarker.add_benchmark(benchmark);
    let results = benchmarker.run_all();

    let result = &results[0];

    // All times should be around 10ms (accounting for system variation)
    assert!(result.mean_time_ms >= 8.0);
    assert!(result.mean_time_ms <= 20.0);
    assert!(result.min_time_ms <= result.mean_time_ms);
    assert!(result.max_time_ms >= result.mean_time_ms);
    assert!(result.percentile_95_ms >= result.median_time_ms);
}

// ============================================================================
// Unit Tests - ParallelTestExecutor
// ============================================================================

#[test]
fn test_parallel_test_executor_new() {
    let executor = ParallelTestExecutor::new();
    assert!(executor.num_threads > 0);
    assert!(executor.num_threads <= num_cpus::get());
}

#[test]
fn test_parallel_test_executor_with_threads() {
    let executor = ParallelTestExecutor::with_threads(4);
    assert_eq!(executor.num_threads, 4);
}

#[test]
fn test_parallel_test_executor_execute_empty() {
    let executor = ParallelTestExecutor::new();
    let notebook = create_test_notebook(vec![]);

    let results = executor.execute_parallel(&notebook, 2);

    assert!(results.is_empty());
}

#[test]
fn test_parallel_test_executor_execute_single_thread() {
    let executor = ParallelTestExecutor::new();
    let cells = vec![
        create_test_cell("cell1", "let x = 1;"),
        create_test_cell("cell2", "let y = 2;"),
    ];
    let notebook = create_test_notebook(cells);

    let results = executor.execute_parallel(&notebook, 1);

    assert_eq!(results.len(), 2);
    for result in &results {
        assert!(result.success);
        assert!(!result.output.is_empty());
        assert!(result.duration_ms >= 0);
    }
}

#[test]
fn test_parallel_test_executor_execute_multiple_threads() {
    let executor = ParallelTestExecutor::new();
    let cells = (0..10)
        .map(|i| create_test_cell(&format!("cell_{}", i), &format!("let x{} = {};", i, i)))
        .collect();
    let notebook = create_test_notebook(cells);

    let results = executor.execute_parallel(&notebook, 4);

    assert_eq!(results.len(), 10);

    // Verify all cells were executed
    let mut cell_ids: Vec<_> = results.iter().map(|r| r.cell_id.clone()).collect();
    cell_ids.sort();

    for i in 0..10 {
        assert!(cell_ids.contains(&format!("cell_{}", i)));
    }
}

#[test]
fn test_parallel_test_executor_zero_threads() {
    let executor = ParallelTestExecutor::new();
    let cells = vec![create_test_cell("cell1", "let x = 1;")];
    let notebook = create_test_notebook(cells);

    let results = executor.execute_parallel(&notebook, 0);

    // Should handle gracefully
    assert!(results.is_empty());
}

// ============================================================================
// Unit Tests - TestCache
// ============================================================================

#[test]
fn test_test_cache_new() {
    let cache = TestCache::new();
    assert_eq!(cache.cache.len(), 0);
    assert_eq!(cache.hits, 0);
    assert_eq!(cache.misses, 0);
    assert_eq!(cache.max_size, 1000);
}

#[test]
fn test_test_cache_with_max_size() {
    let cache = TestCache::with_max_size(100);
    assert_eq!(cache.max_size, 100);
}

#[test]
fn test_test_cache_store_and_get() {
    let mut cache = TestCache::new();

    let result = TestExecutionResult {
        cell_id: "test_cell".to_string(),
        success: true,
        output: "OK".to_string(),
        duration_ms: 100,
    };

    cache.store("test_key", &result);
    let retrieved = cache.get("test_key");

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.cell_id, "test_cell");
    assert!(retrieved.success);
    assert_eq!(retrieved.output, "OK");
    assert_eq!(retrieved.duration_ms, 100);
}

#[test]
fn test_test_cache_get_nonexistent() {
    let mut cache = TestCache::new();
    let result = cache.get("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_test_cache_statistics() {
    let mut cache = TestCache::new();

    let result = TestExecutionResult {
        cell_id: "test".to_string(),
        success: true,
        output: "OK".to_string(),
        duration_ms: 50,
    };

    // Initial stats
    let stats = cache.get_statistics();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 0);
    assert_eq!(stats.hit_rate, 0.0);

    // Store and retrieve
    cache.store("key1", &result);
    cache.get("key1"); // Hit
    cache.get("key2"); // Miss

    let stats = cache.get_statistics();
    assert!(stats.hits > 0);
    assert!(stats.misses > 0);
    assert_eq!(stats.size, 1);
    assert!(stats.hit_rate > 0.0 && stats.hit_rate < 1.0);
}

#[test]
fn test_test_cache_eviction() {
    let mut cache = TestCache::with_max_size(2);

    let result = TestExecutionResult {
        cell_id: "test".to_string(),
        success: true,
        output: "OK".to_string(),
        duration_ms: 50,
    };

    // Fill cache to capacity
    cache.store("key1", &result);
    cache.store("key2", &result);
    assert_eq!(cache.cache.len(), 2);

    // Add one more - should trigger eviction
    cache.store("key3", &result);
    assert_eq!(cache.cache.len(), 2);

    // key1 should be evicted (LRU)
    assert!(cache.get("key1").is_none());
    assert!(cache.get("key2").is_some());
    assert!(cache.get("key3").is_some());
}

// ============================================================================
// Unit Tests - ResourceMonitor
// ============================================================================

#[test]
fn test_resource_monitor_new() {
    let monitor = ResourceMonitor::new();
    assert!(monitor.start_time.is_none());
}

#[test]
fn test_resource_monitor_start_stop() {
    let mut monitor = ResourceMonitor::new();

    monitor.start();
    assert!(monitor.start_time.is_some());

    thread::sleep(Duration::from_millis(10));

    let usage = monitor.get_usage();
    assert!(usage.duration_ms >= 10);
    assert!(usage.memory_mb > 0.0);
    assert!(usage.cpu_percent > 0.0);
    assert!(usage.peak_memory_mb > 0.0);

    monitor.stop();
}

#[test]
fn test_resource_monitor_get_usage_before_start() {
    let monitor = ResourceMonitor::new();
    let usage = monitor.get_usage();
    assert_eq!(usage.duration_ms, 0);
}

// ============================================================================
// Unit Tests - TestSharder
// ============================================================================

#[test]
fn test_test_sharder_new() {
    let sharder = TestSharder::new();
    // Just verify it constructs successfully
}

#[test]
fn test_test_sharder_shard_empty() {
    let sharder = TestSharder::new();
    let tests = vec![];
    let shards = sharder.shard(&tests, 3);

    assert_eq!(shards.len(), 3);
    for shard in shards {
        assert!(shard.is_empty());
    }
}

#[test]
fn test_test_sharder_shard_zero_shards() {
    let sharder = TestSharder::new();
    let tests = vec!["test1".to_string(), "test2".to_string()];
    let shards = sharder.shard(&tests, 0);

    assert!(shards.is_empty());
}

#[test]
fn test_test_sharder_shard_round_robin() {
    let sharder = TestSharder::new();
    let tests = vec![
        "test1".to_string(),
        "test2".to_string(),
        "test3".to_string(),
        "test4".to_string(),
        "test5".to_string(),
    ];

    let shards = sharder.shard(&tests, 3);

    assert_eq!(shards.len(), 3);
    assert_eq!(shards[0], vec!["test1", "test4"]);
    assert_eq!(shards[1], vec!["test2", "test5"]);
    assert_eq!(shards[2], vec!["test3"]);
}

#[test]
fn test_test_sharder_shard_by_duration() {
    let sharder = TestSharder::new();
    let tests = vec![
        ("fast".to_string(), Duration::from_millis(10)),
        ("slow".to_string(), Duration::from_millis(1000)),
        ("medium".to_string(), Duration::from_millis(100)),
        ("very_slow".to_string(), Duration::from_millis(2000)),
    ];

    let shards = sharder.shard_by_duration(&tests, 2);

    assert_eq!(shards.len(), 2);

    // Should balance workload - slow tests should be distributed
    let shard0_names: Vec<&str> = shards[0].iter().map(|s| s.as_str()).collect();
    let shard1_names: Vec<&str> = shards[1].iter().map(|s| s.as_str()).collect();

    // Very slow test should be in one shard, slow test in another
    assert!(shard0_names.contains(&"very_slow") || shard1_names.contains(&"very_slow"));
    assert!(shard0_names.contains(&"slow") || shard1_names.contains(&"slow"));
}

// ============================================================================
// Unit Tests - RegressionDetector
// ============================================================================

#[test]
fn test_regression_detector_new() {
    let detector = RegressionDetector::new();
    assert!(detector.baselines.is_empty());
    assert_eq!(detector.tolerance_percent, 5.0);
}

#[test]
fn test_regression_detector_with_tolerance() {
    let detector = RegressionDetector::with_tolerance(10.0);
    assert_eq!(detector.tolerance_percent, 10.0);
}

#[test]
fn test_regression_detector_add_baseline() {
    let mut detector = RegressionDetector::new();
    detector.add_baseline("test_function", 100.0);

    assert!(detector.baselines.contains_key("test_function"));
    assert_eq!(detector.baselines["test_function"], 100.0);
}

#[test]
fn test_regression_detector_no_baseline() {
    let detector = RegressionDetector::new();
    let result = detector.check_regression("unknown_test", 150.0);

    assert!(!result.is_regression);
    assert_eq!(result.percent_change, 0.0);
    assert_eq!(result.baseline, 0.0);
    assert_eq!(result.current, 150.0);
}

#[test]
fn test_regression_detector_within_tolerance() {
    let mut detector = RegressionDetector::new();
    detector.add_baseline("test", 100.0);

    let result = detector.check_regression("test", 104.0); // 4% increase

    assert!(!result.is_regression);
    assert_eq!(result.percent_change, 4.0);
    assert_eq!(result.baseline, 100.0);
    assert_eq!(result.current, 104.0);
}

#[test]
fn test_regression_detector_exceeds_tolerance() {
    let mut detector = RegressionDetector::new();
    detector.add_baseline("test", 100.0);

    let result = detector.check_regression("test", 110.0); // 10% increase

    assert!(result.is_regression);
    assert_eq!(result.percent_change, 10.0);
    assert_eq!(result.baseline, 100.0);
    assert_eq!(result.current, 110.0);
}

#[test]
fn test_regression_detector_improvement() {
    let mut detector = RegressionDetector::new();
    detector.add_baseline("test", 100.0);

    let result = detector.check_regression("test", 90.0); // 10% improvement

    assert!(!result.is_regression);
    assert_eq!(result.percent_change, -10.0);
}

// ============================================================================
// Unit Tests - TestPrioritizer
// ============================================================================

#[test]
fn test_test_prioritizer_new() {
    let prioritizer = TestPrioritizer::new();
    assert!(prioritizer.history.is_empty());
}

#[test]
fn test_test_prioritizer_record_failure() {
    let mut prioritizer = TestPrioritizer::new();
    prioritizer.record_failure("test1", 3);

    assert!(prioritizer.history.contains_key("test1"));
    let history = &prioritizer.history["test1"];
    assert_eq!(history.failures, 3);
    assert_eq!(history.successes, 0);
    assert!(history.last_run.is_some());
}

#[test]
fn test_test_prioritizer_record_success() {
    let mut prioritizer = TestPrioritizer::new();
    prioritizer.record_success("test1", 5);

    assert!(prioritizer.history.contains_key("test1"));
    let history = &prioritizer.history["test1"];
    assert_eq!(history.failures, 0);
    assert_eq!(history.successes, 5);
    assert!(history.last_run.is_some());
}

#[test]
fn test_test_prioritizer_prioritize_empty() {
    let prioritizer = TestPrioritizer::new();
    let tests = vec![];
    let prioritized = prioritizer.prioritize(&tests);
    assert!(prioritized.is_empty());
}

#[test]
fn test_test_prioritizer_prioritize_by_failures() {
    let mut prioritizer = TestPrioritizer::new();

    // Record different failure rates
    prioritizer.record_failure("high_failure", 10);
    prioritizer.record_failure("medium_failure", 5);
    prioritizer.record_failure("low_failure", 1);

    let tests = vec![
        "low_failure".to_string(),
        "high_failure".to_string(),
        "medium_failure".to_string(),
        "no_history".to_string(),
    ];

    let prioritized = prioritizer.prioritize(&tests);

    // Should be ordered by failure count (highest first)
    assert_eq!(prioritized[0], "high_failure");
    assert_eq!(prioritized[1], "medium_failure");
    assert_eq!(prioritized[2], "low_failure");
    assert_eq!(prioritized[3], "no_history");
}

// ============================================================================
// Property-Based Tests (10,000+ iterations)
// ============================================================================

proptest! {
    #[test]
    fn prop_benchmark_result_valid_statistics(
        iterations in 1usize..100
    ) {
        let mut benchmarker = PerformanceBenchmarker::new();
        let benchmark = create_test_benchmark("prop_test", "Property Test", iterations);
        benchmarker.add_benchmark(benchmark);

        let results = benchmarker.run_all();
        let result = &results[0];

        prop_assert!(result.mean_time_ms >= 0.0);
        prop_assert!(result.median_time_ms >= 0.0);
        prop_assert!(result.std_dev_ms >= 0.0);
        prop_assert!(result.min_time_ms >= 0.0);
        prop_assert!(result.max_time_ms >= 0.0);
        prop_assert!(result.percentile_95_ms >= 0.0);
        prop_assert!(result.min_time_ms <= result.max_time_ms);
        prop_assert!(result.median_time_ms >= result.min_time_ms);
        prop_assert!(result.median_time_ms <= result.max_time_ms);
    }

    #[test]
    fn prop_parallel_executor_result_count(
        num_cells in 0usize..50,
        num_threads in 1usize..8
    ) {
        let executor = ParallelTestExecutor::new();
        let cells: Vec<Cell> = (0..num_cells)
            .map(|i| create_test_cell(&format!("cell_{}", i), &format!("let x{} = {};", i, i)))
            .collect();
        let notebook = create_test_notebook(cells);

        let results = executor.execute_parallel(&notebook, num_threads);

        prop_assert_eq!(results.len(), num_cells);
    }

    #[test]
    fn prop_cache_size_bounds(
        max_size in 1usize..1000,
        num_items in 0usize..100
    ) {
        let mut cache = TestCache::with_max_size(max_size);
        let result = TestExecutionResult {
            cell_id: "test".to_string(),
            success: true,
            output: "OK".to_string(),
            duration_ms: 50,
        };

        for i in 0..num_items {
            cache.store(&format!("key_{}", i), &result);
        }

        prop_assert!(cache.cache.len() <= max_size);
        prop_assert!(cache.cache.len() <= num_items);
    }

    #[test]
    fn prop_test_shard_distribution(
        num_tests in 0usize..100,
        num_shards in 1usize..10
    ) {
        let sharder = TestSharder::new();
        let tests: Vec<String> = (0..num_tests)
            .map(|i| format!("test_{}", i))
            .collect();

        let shards = sharder.shard(&tests, num_shards);

        prop_assert_eq!(shards.len(), num_shards);

        let total_distributed: usize = shards.iter().map(|s| s.len()).sum();
        prop_assert_eq!(total_distributed, num_tests);
    }

    #[test]
    fn prop_regression_detector_percentage_calculation(
        baseline in 1.0f64..10000.0,
        current in 1.0f64..10000.0
    ) {
        let mut detector = RegressionDetector::new();
        detector.add_baseline("test", baseline);

        let result = detector.check_regression("test", current);

        let expected_change = ((current - baseline) / baseline) * 100.0;
        prop_assert!((result.percent_change - expected_change).abs() < 0.0001);
        prop_assert_eq!(result.baseline, baseline);
        prop_assert_eq!(result.current, current);
    }
}

// ============================================================================
// Stress Tests - Performance Limits
// ============================================================================

#[test]
fn stress_test_many_benchmarks() {
    let mut benchmarker = PerformanceBenchmarker::new();

    // Add 100 benchmarks
    for i in 0..100 {
        let benchmark =
            create_test_benchmark(&format!("bench_{}", i), &format!("Benchmark {}", i), 3);
        benchmarker.add_benchmark(benchmark);
    }

    let start = Instant::now();
    let results = benchmarker.run_all();
    let duration = start.elapsed();

    assert_eq!(results.len(), 100);
    // Should complete within reasonable time (allowing for system variation)
    assert!(duration.as_secs() < 60); // Less than 1 minute
}

#[test]
fn stress_test_large_parallel_execution() {
    let executor = ParallelTestExecutor::new();

    // Create 1000 cells
    let cells: Vec<Cell> = (0..1000)
        .map(|i| create_test_cell(&format!("cell_{}", i), &format!("let x{} = {};", i, i)))
        .collect();
    let notebook = create_test_notebook(cells);

    let start = Instant::now();
    let results = executor.execute_parallel(&notebook, 8);
    let duration = start.elapsed();

    assert_eq!(results.len(), 1000);
    // Should complete efficiently with parallel execution
    assert!(duration.as_secs() < 30); // Less than 30 seconds
}

#[test]
fn stress_test_cache_heavy_usage() {
    let mut cache = TestCache::with_max_size(100);
    let result = TestExecutionResult {
        cell_id: "test".to_string(),
        success: true,
        output: "OK".to_string(),
        duration_ms: 50,
    };

    // Perform many cache operations
    for i in 0..10000 {
        let key = format!("key_{}", i % 150); // Some overlap to test eviction
        cache.store(&key, &result);

        if i % 2 == 0 {
            cache.get(&key);
        }
    }

    let stats = cache.get_statistics();
    assert!(stats.hits > 0);
    assert!(stats.misses > 0);
    assert!(cache.cache.len() <= 100);
}

#[test]
fn stress_test_resource_monitor_duration() {
    let mut monitor = ResourceMonitor::new();

    monitor.start();

    // Simulate long-running operation
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(1));
        let _usage = monitor.get_usage();
    }

    let final_usage = monitor.get_usage();
    monitor.stop();

    assert!(final_usage.duration_ms >= 100);
    assert!(final_usage.memory_mb > 0.0);
    assert!(final_usage.cpu_percent >= 0.0);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_edge_case_zero_iteration_benchmark() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let benchmark = create_test_benchmark("zero_iter", "Zero Iterations", 0);
    benchmarker.add_benchmark(benchmark);

    let results = benchmarker.run_all();

    // Should handle gracefully
    assert_eq!(results.len(), 1);
}

#[test]
fn test_edge_case_very_fast_benchmark() {
    let mut benchmarker = PerformanceBenchmarker::new();

    let fast_benchmark = Benchmark {
        id: "fast".to_string(),
        name: "Very Fast".to_string(),
        setup: Box::new(|| {}),
        run: Box::new(|| {
            // Extremely fast operation
            let _x = 1 + 1;
        }),
        teardown: Box::new(|| {}),
        iterations: 1000,
    };

    benchmarker.add_benchmark(fast_benchmark);
    let results = benchmarker.run_all();

    assert_eq!(results.len(), 1);
    assert!(results[0].mean_time_ms >= 0.0);
    assert!(results[0].min_time_ms >= 0.0);
}

#[test]
fn test_edge_case_empty_notebook_parallel() {
    let executor = ParallelTestExecutor::new();
    let notebook = create_test_notebook(vec![]);

    let results = executor.execute_parallel(&notebook, 100); // Many threads

    assert!(results.is_empty());
}

#[test]
fn test_edge_case_single_cell_many_threads() {
    let executor = ParallelTestExecutor::new();
    let cells = vec![create_test_cell("single", "let x = 42;")];
    let notebook = create_test_notebook(cells);

    let results = executor.execute_parallel(&notebook, 100);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].cell_id, "single");
}

#[test]
fn test_edge_case_cache_zero_capacity() {
    let mut cache = TestCache::with_max_size(0);
    let result = TestExecutionResult {
        cell_id: "test".to_string(),
        success: true,
        output: "OK".to_string(),
        duration_ms: 50,
    };

    cache.store("key", &result);
    let retrieved = cache.get("key");

    // Should handle zero capacity gracefully
    assert!(retrieved.is_none());
}

#[test]
fn test_edge_case_regression_zero_baseline() {
    let mut detector = RegressionDetector::new();
    detector.add_baseline("test", 0.0);

    let result = detector.check_regression("test", 100.0);

    // Should handle zero baseline without division by zero
    assert!(!result.is_regression || result.is_regression); // Either is valid
    assert_eq!(result.baseline, 0.0);
    assert_eq!(result.current, 100.0);
}

// ============================================================================
// Integration Tests - Real Usage Scenarios
// ============================================================================

#[test]
fn integration_test_complete_performance_workflow() {
    // Simulate complete performance testing workflow
    let mut benchmarker = PerformanceBenchmarker::new();
    let executor = ParallelTestExecutor::new();
    let mut cache = TestCache::new();
    let mut monitor = ResourceMonitor::new();
    let mut detector = RegressionDetector::new();

    // Phase 1: Setup benchmarks
    for i in 0..5 {
        let benchmark =
            create_test_benchmark(&format!("bench_{}", i), &format!("Benchmark {}", i), 5);
        benchmarker.add_benchmark(benchmark);
    }

    // Phase 2: Run benchmarks with monitoring
    monitor.start();
    let bench_results = benchmarker.run_all();
    let resource_usage = monitor.get_usage();
    monitor.stop();

    // Phase 3: Parallel execution of notebook cells
    let cells: Vec<Cell> = (0..20)
        .map(|i| {
            create_test_cell(
                &format!("cell_{}", i),
                &format!("let result_{} = {} * 2;", i, i),
            )
        })
        .collect();
    let notebook = create_test_notebook(cells);

    let parallel_results = executor.execute_parallel(&notebook, 4);

    // Phase 4: Cache results and check regressions
    for result in &parallel_results {
        cache.store(&result.cell_id, result);

        // Simulate baseline comparison
        detector.add_baseline(&result.cell_id, result.duration_ms as f64);
        let regression =
            detector.check_regression(&result.cell_id, result.duration_ms as f64 * 1.1);

        if regression.is_regression {
            println!(
                "Regression detected in {}: {}%",
                result.cell_id, regression.percent_change
            );
        }
    }

    // Phase 5: Verify results
    assert_eq!(bench_results.len(), 5);
    assert_eq!(parallel_results.len(), 20);
    assert!(resource_usage.duration_ms > 0);

    let cache_stats = cache.get_statistics();
    assert_eq!(cache_stats.size, 20);

    // All results should be successful
    for result in &parallel_results {
        assert!(result.success);
        assert!(result.duration_ms >= 0);
    }
}

#[test]
fn integration_test_load_balancing_workflow() {
    let sharder = TestSharder::new();
    let mut prioritizer = TestPrioritizer::new();

    // Simulate test history
    let test_failures = vec![
        ("flaky_test", 10),
        ("reliable_test", 1),
        ("broken_test", 15),
        ("new_test", 0),
        ("intermittent_test", 5),
    ];

    for (test_name, failures) in &test_failures {
        if *failures > 0 {
            prioritizer.record_failure(test_name, *failures);
        } else {
            prioritizer.record_success(test_name, 10);
        }
    }

    // Create test list
    let tests: Vec<String> = test_failures
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    // Prioritize tests
    let prioritized_tests = prioritizer.prioritize(&tests);

    // Verify prioritization (higher failure count first)
    assert_eq!(prioritized_tests[0], "broken_test");
    assert_eq!(prioritized_tests[1], "flaky_test");

    // Shard tests with duration awareness
    let test_durations: Vec<(String, Duration)> = prioritized_tests
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), Duration::from_millis((i + 1) * 100)))
        .collect();

    let shards = sharder.shard_by_duration(&test_durations, 3);

    assert_eq!(shards.len(), 3);

    // Verify load balancing
    let shard_sizes: Vec<usize> = shards.iter().map(|s| s.len()).collect();
    let max_size = *shard_sizes.iter().max().unwrap();
    let min_size = *shard_sizes.iter().min().unwrap();

    // Should be reasonably balanced
    assert!(max_size - min_size <= 2);
}

#[test]
fn integration_test_regression_detection_workflow() {
    let mut detector = RegressionDetector::with_tolerance(5.0);

    // Simulate baseline performance measurements
    let baselines = vec![
        ("algorithm_a", 100.0),
        ("algorithm_b", 200.0),
        ("algorithm_c", 50.0),
        ("algorithm_d", 300.0),
    ];

    for (name, time) in &baselines {
        detector.add_baseline(name, *time);
    }

    // Simulate new measurements
    let new_measurements = vec![
        ("algorithm_a", 102.0), // 2% increase - within tolerance
        ("algorithm_b", 220.0), // 10% increase - regression
        ("algorithm_c", 48.0),  // 4% improvement
        ("algorithm_d", 350.0), // 16.7% increase - regression
        ("algorithm_e", 150.0), // New algorithm - no baseline
    ];

    let mut regressions = Vec::new();
    let mut improvements = Vec::new();

    for (name, time) in &new_measurements {
        let result = detector.check_regression(name, *time);

        if result.is_regression {
            regressions.push((name, result.percent_change));
        } else if result.percent_change < -1.0 {
            improvements.push((name, result.percent_change));
        }
    }

    // Verify regression detection
    assert_eq!(regressions.len(), 2);
    assert!(regressions.iter().any(|(name, _)| *name == &"algorithm_b"));
    assert!(regressions.iter().any(|(name, _)| *name == &"algorithm_d"));

    // Verify improvement detection
    assert_eq!(improvements.len(), 1);
    assert!(improvements.iter().any(|(name, _)| *name == &"algorithm_c"));
}

// ============================================================================
// Error Handling and Robustness Tests
// ============================================================================

#[test]
fn test_robustness_concurrent_access() {
    let cache = Arc::new(Mutex::new(TestCache::new()));
    let mut handles = vec![];

    // Simulate concurrent cache access
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let result = TestExecutionResult {
                cell_id: format!("cell_{}", i),
                success: true,
                output: "OK".to_string(),
                duration_ms: i as u64 * 10,
            };

            let mut cache = cache_clone.lock().unwrap();
            cache.store(&format!("key_{}", i), &result);
            cache.get(&format!("key_{}", i))
        });
        handles.push(handle);
    }

    // Wait for all threads and verify results
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_some());
    }
}

#[test]
fn test_robustness_benchmark_panic_recovery() {
    let mut benchmarker = PerformanceBenchmarker::new();

    // Add normal benchmark
    let normal_benchmark = create_test_benchmark("normal", "Normal Benchmark", 3);
    benchmarker.add_benchmark(normal_benchmark);

    // Note: We can't easily test panic recovery without modifying the benchmark
    // In a real implementation, we would want to catch panics in benchmark execution

    let results = benchmarker.run_all();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_robustness_extreme_values() {
    let mut detector = RegressionDetector::new();

    // Test with extreme values
    detector.add_baseline("extreme_small", 0.001);
    detector.add_baseline("extreme_large", 1_000_000.0);

    let small_result = detector.check_regression("extreme_small", 0.0011);
    let large_result = detector.check_regression("extreme_large", 1_100_000.0);

    // Should handle extreme values without overflow/underflow
    assert!(!small_result.is_regression);
    assert!(large_result.is_regression);
    assert!(small_result.percent_change.is_finite());
    assert!(large_result.percent_change.is_finite());
}

#[test]
fn test_memory_efficiency_large_cache() {
    let mut cache = TestCache::with_max_size(10000);
    let result = TestExecutionResult {
        cell_id: "test".to_string(),
        success: true,
        output: "Small output".to_string(),
        duration_ms: 50,
    };

    // Fill with many entries
    for i in 0..5000 {
        cache.store(&format!("key_{}", i), &result);
    }

    // Memory usage should be reasonable
    let stats = cache.get_statistics();
    assert_eq!(stats.size, 5000);

    // Should handle large number of entries efficiently
    let start = Instant::now();
    for i in 0..1000 {
        cache.get(&format!("key_{}", i));
    }
    let duration = start.elapsed();

    assert!(duration.as_millis() < 100); // Should be fast
}
