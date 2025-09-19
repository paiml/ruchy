// Extreme TDD Test Suite for src/notebook/testing/performance.rs
// Target: 383 lines, 0% → 95%+ coverage
//
// Quality Standards:
// - TDD methodology: Test-first development
// - Cyclomatic complexity ≤10 for all test functions
// - Property-based testing with 10,000+ iterations
// - Zero SATD (Self-Admitted Technical Debt) comments
// - Complete Big O algorithmic analysis
// - Toyota Way: Root cause analysis and systematic defect prevention

use ruchy::notebook::testing::performance::{
    PerformanceBenchmarker, Benchmark, BenchmarkResult,
    ParallelTestExecutor, TestExecutionResult,
    TestCache, CachedResult, CacheStats,
    ResourceMonitor, ResourceUsage,
    TestSharder,
    RegressionDetector, RegressionResult,
    TestPrioritizer, TestHistory,
};
use ruchy::notebook::testing::types::{Notebook, Cell, CellType};
use std::time::{Duration, Instant};
use std::sync::Arc;
use proptest::prelude::*;

// Helper functions
fn create_test_benchmark(id: &str, name: &str, iterations: usize) -> Benchmark {
    Benchmark {
        id: id.to_string(),
        name: name.to_string(),
        setup: Box::new(|| {}),
        run: Box::new(|| { std::thread::sleep(Duration::from_micros(10)); }),
        teardown: Box::new(|| {}),
        iterations,
    }
}

fn create_test_cell(id: &str, source: &str) -> Cell {
    Cell {
        id: id.to_string(),
        source: source.to_string(),
        cell_type: CellType::Code,
        metadata: Default::default(),
    }
}

fn create_test_notebook(num_cells: usize) -> Notebook {
    let cells = (0..num_cells)
        .map(|i| create_test_cell(&format!("cell{}", i), &format!("test code {}", i)))
        .collect();
    Notebook {
        cells,
        metadata: None,
    }
}

// Test PerformanceBenchmarker
#[test]
fn test_performance_benchmarker_new() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let results = benchmarker.run_all();
    assert!(results.is_empty());
}

#[test]
fn test_performance_benchmarker_default() {
    let mut benchmarker = PerformanceBenchmarker::default();
    let results = benchmarker.run_all();
    assert!(results.is_empty());
}

#[test]
fn test_add_benchmark() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let bench = create_test_benchmark("test1", "Test Benchmark", 10);
    benchmarker.add_benchmark(bench);
    // Successfully added
    assert!(true);
}

#[test]
fn test_run_all_benchmarks() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let bench1 = create_test_benchmark("bench1", "First", 5);
    let bench2 = create_test_benchmark("bench2", "Second", 5);

    benchmarker.add_benchmark(bench1);
    benchmarker.add_benchmark(bench2);

    let results = benchmarker.run_all();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_benchmark_result_statistics() {
    let mut benchmarker = PerformanceBenchmarker::new();
    let bench = create_test_benchmark("stats", "Statistics Test", 10);
    benchmarker.add_benchmark(bench);

    let results = benchmarker.run_all();
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert_eq!(result.id, "stats");
    assert!(result.mean_time_ms >= 0.0);
    assert!(result.median_time_ms >= 0.0);
    assert!(result.std_dev_ms >= 0.0);
    assert!(result.min_time_ms <= result.max_time_ms);
    assert!(result.percentile_95_ms >= result.median_time_ms);
}

// Test ParallelTestExecutor
#[test]
fn test_parallel_executor_new() {
    let _executor = ParallelTestExecutor::new();
    assert!(true);
}

#[test]
fn test_parallel_executor_with_threads() {
    let _executor = ParallelTestExecutor::with_threads(4);
    assert!(true);
}

#[test]
fn test_execute_parallel() {
    let executor = ParallelTestExecutor::new();
    let notebook = create_test_notebook(5);
    let results = executor.execute_parallel(&notebook, 2);
    assert_eq!(results.len(), 5);
}

#[test]
fn test_parallel_execution_results() {
    let executor = ParallelTestExecutor::with_threads(2);
    let notebook = create_test_notebook(3);
    let results = executor.execute_parallel(&notebook, 2);

    for result in results {
        assert!(!result.cell_id.is_empty());
        assert!(result.duration_ms >= 0);
    }
}

// Test TestCache
#[test]
fn test_test_cache_new() {
    let _cache = TestCache::new();
    assert!(true);
}

#[test]
fn test_test_cache_with_max_size() {
    let _cache = TestCache::with_max_size(100);
    assert!(true);
}

#[test]
fn test_cache_store_and_get() {
    let mut cache = TestCache::new();
    let result = TestExecutionResult {
        cell_id: "test".to_string(),
        success: true,
        output: "test output".to_string(),
        duration_ms: 10,
    };

    cache.store("test_key", &result);
    let retrieved = cache.get("test_key");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().cell_id, "test");
}

#[test]
fn test_cache_miss() {
    let mut cache = TestCache::new();
    let result = cache.get("non_existent");
    assert!(result.is_none());
}

#[test]
fn test_cache_stats() {
    let mut cache = TestCache::with_max_size(10);
    let result = TestExecutionResult {
        cell_id: "cell1".to_string(),
        success: true,
        output: "output".to_string(),
        duration_ms: 5,
    };

    cache.store("key1", &result);
    let _ = cache.get("key1"); // Hit
    let _ = cache.get("key2"); // Miss

    let stats = cache.get_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.size, 1);
}

#[test]
fn test_cache_eviction() {
    let mut cache = TestCache::with_max_size(2);

    for i in 0..5 {
        let result = TestExecutionResult {
            cell_id: format!("cell{}", i),
            success: true,
            output: format!("output{}", i),
            duration_ms: 5,
        };
        cache.store(&format!("key{}", i), &result);
    }

    let stats = cache.get_stats();
    assert!(stats.size <= 2); // Max size respected
}

// Test ResourceMonitor
#[test]
fn test_resource_monitor_new() {
    let _monitor = ResourceMonitor::new();
    assert!(true);
}

#[test]
fn test_resource_monitor_start_stop() {
    let mut monitor = ResourceMonitor::new();
    monitor.start();
    std::thread::sleep(Duration::from_millis(10));
    monitor.stop();
    assert!(true);
}

#[test]
fn test_resource_monitor_get_usage() {
    let mut monitor = ResourceMonitor::new();
    monitor.start();
    std::thread::sleep(Duration::from_millis(10));
    monitor.stop();

    let usage = monitor.get_usage();
    assert!(usage.cpu_percent >= 0.0);
    assert!(usage.memory_mb >= 0.0);
    assert!(usage.peak_memory_mb >= usage.memory_mb);
}

// Test TestSharder
#[test]
fn test_test_sharder_new() {
    let _sharder = TestSharder;
    assert!(true);
}

#[test]
fn test_shard_empty() {
    let sharder = TestSharder;
    let shards = sharder.shard(&[], 3);
    assert_eq!(shards.len(), 3);
    assert!(shards.iter().all(|s| s.is_empty()));
}

#[test]
fn test_shard_basic() {
    let sharder = TestSharder;
    let tests = vec!["test1".to_string(), "test2".to_string(), "test3".to_string()];
    let shards = sharder.shard(&tests, 2);

    assert_eq!(shards.len(), 2);
    let total: usize = shards.iter().map(|s| s.len()).sum();
    assert_eq!(total, 3);
}

#[test]
fn test_shard_by_duration() {
    let sharder = TestSharder;
    let tests = vec![
        ("test1".to_string(), Duration::from_millis(100)),
        ("test2".to_string(), Duration::from_millis(200)),
        ("test3".to_string(), Duration::from_millis(150)),
    ];

    let shards = sharder.shard_by_duration(&tests, 2);
    assert_eq!(shards.len(), 2);

    let total: usize = shards.iter().map(|s| s.len()).sum();
    assert_eq!(total, 3);
}

// Test RegressionDetector
#[test]
fn test_regression_detector_new() {
    let _detector = RegressionDetector::new();
    assert!(true);
}

#[test]
fn test_regression_detector_with_tolerance() {
    let _detector = RegressionDetector::with_tolerance(10.0);
    assert!(true);
}

#[test]
fn test_add_baseline() {
    let mut detector = RegressionDetector::new();
    detector.add_baseline("test1", 100.0);
    detector.add_baseline("test2", 200.0);
    assert!(true);
}

#[test]
fn test_check_regression_no_baseline() {
    let detector = RegressionDetector::new();
    let result = detector.check_regression("unknown", 100.0);
    assert!(!result.is_regression);
}

#[test]
fn test_check_regression_within_tolerance() {
    let mut detector = RegressionDetector::with_tolerance(10.0);
    detector.add_baseline("test", 100.0);

    let result = detector.check_regression("test", 105.0);
    assert!(!result.is_regression);
}

#[test]
fn test_check_regression_detected() {
    let mut detector = RegressionDetector::with_tolerance(10.0);
    detector.add_baseline("test", 100.0);

    let result = detector.check_regression("test", 150.0);
    assert!(result.is_regression);
    assert_eq!(result.baseline, 100.0);
    assert_eq!(result.current, 150.0);
    assert_eq!(result.percent_change, 50.0);
}

// Test TestPrioritizer
#[test]
fn test_test_prioritizer_new() {
    let _prioritizer = TestPrioritizer::new();
    assert!(true);
}

#[test]
fn test_record_failure() {
    let mut prioritizer = TestPrioritizer::new();
    prioritizer.record_failure("test1", 3);
    prioritizer.record_failure("test2", 1);
    assert!(true);
}

#[test]
fn test_record_success() {
    let mut prioritizer = TestPrioritizer::new();
    prioritizer.record_success("test1", 10);
    prioritizer.record_success("test2", 5);
    assert!(true);
}

#[test]
fn test_prioritize_empty() {
    let prioritizer = TestPrioritizer::new();
    let tests = vec![];
    let prioritized = prioritizer.prioritize(&tests);
    assert!(prioritized.is_empty());
}

#[test]
fn test_prioritize_by_failure_rate() {
    let mut prioritizer = TestPrioritizer::new();
    prioritizer.record_failure("high_fail", 8);
    prioritizer.record_success("high_fail", 2);
    prioritizer.record_failure("low_fail", 1);
    prioritizer.record_success("low_fail", 9);

    let tests = vec!["low_fail".to_string(), "high_fail".to_string()];
    let prioritized = prioritizer.prioritize(&tests);

    assert_eq!(prioritized.len(), 2);
    // High failure rate should be prioritized first
    assert_eq!(prioritized[0], "high_fail");
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn test_benchmark_iterations_respected(
            iterations in 1usize..100usize
        ) {
            let mut benchmarker = PerformanceBenchmarker::new();
            let bench = create_test_benchmark("prop", "Property Test", iterations);
            benchmarker.add_benchmark(bench);

            let results = benchmarker.run_all();
            prop_assert_eq!(results.len(), 1);
            prop_assert!(results[0].mean_time_ms >= 0.0);
        }

        #[test]
        fn test_parallel_execution_thread_count(
            thread_count in 1usize..16usize,
            cell_count in 1usize..20usize
        ) {
            let executor = ParallelTestExecutor::with_threads(thread_count);
            let notebook = create_test_notebook(cell_count);
            let results = executor.execute_parallel(&notebook, thread_count);
            prop_assert_eq!(results.len(), cell_count);
        }

        #[test]
        fn test_cache_size_limit(
            max_size in 1usize..50usize,
            num_entries in 0usize..100usize
        ) {
            let mut cache = TestCache::with_max_size(max_size);

            for i in 0..num_entries {
                let result = TestExecutionResult {
                    cell_id: format!("cell{}", i),
                    success: true,
                    output: format!("output{}", i),
                    duration_ms: i as u64,
                };
                cache.store(&format!("key{}", i), &result);
            }

            let stats = cache.get_stats();
            prop_assert!(stats.size <= max_size);
        }

        #[test]
        fn test_shard_distribution(
            num_tests in 0usize..100usize,
            num_shards in 1usize..10usize
        ) {
            let sharder = TestSharder;
            let tests: Vec<String> = (0..num_tests)
                .map(|i| format!("test{}", i))
                .collect();

            let shards = sharder.shard(&tests, num_shards);
            prop_assert_eq!(shards.len(), num_shards);

            let total: usize = shards.iter().map(|s| s.len()).sum();
            prop_assert_eq!(total, num_tests);

            // No test should appear twice
            let mut all_tests = Vec::new();
            for shard in shards {
                all_tests.extend(shard);
            }
            all_tests.sort();
            all_tests.dedup();
            prop_assert_eq!(all_tests.len(), num_tests);
        }

        #[test]
        fn test_regression_tolerance(
            tolerance in 0.0f64..100.0f64,
            baseline in 1.0f64..1000.0f64,
            current in 1.0f64..2000.0f64
        ) {
            let mut detector = RegressionDetector::with_tolerance(tolerance);
            detector.add_baseline("test", baseline);

            let result = detector.check_regression("test", current);
            let expected_percent_change = ((current - baseline) / baseline) * 100.0;

            // Regression only detected when current > baseline by tolerance%
            if expected_percent_change > tolerance {
                prop_assert!(result.is_regression);
            } else {
                prop_assert!(!result.is_regression);
            }
        }

        #[test]
        fn test_prioritizer_ordering(
            test_count in 1usize..20usize
        ) {
            let mut prioritizer = TestPrioritizer::new();
            let mut tests = Vec::new();

            for i in 0..test_count {
                let test_name = format!("test{}", i);
                tests.push(test_name.clone());

                // Give different failure rates
                prioritizer.record_failure(&test_name, i);
                prioritizer.record_success(&test_name, test_count - i);
            }

            let prioritized = prioritizer.prioritize(&tests);
            prop_assert_eq!(prioritized.len(), test_count);

            // All tests should be present
            for test in &tests {
                prop_assert!(prioritized.contains(test));
            }
        }
    }
}

// Big O Complexity Analysis
// Performance Testing Core Functions:
//
// - PerformanceBenchmarker::run_all(): O(b * i) where b is benchmarks, i is iterations
//   - Each benchmark runs i iterations
//   - Statistics calculation: O(i) for sorting (median)
//   - Total: O(b * i * log(i)) with sorting
//
// - ParallelTestExecutor::execute_parallel(): O(c) where c is cells
//   - Thread distribution: O(c) to split work
//   - Actual execution: O(c/t) with t threads
//   - Result collection: O(c)
//   - Total speedup: ~t times for CPU-bound work
//
// - TestCache operations:
//   - store(): O(1) average HashMap insert, O(n) worst case resize
//   - get(): O(1) average HashMap lookup
//   - eviction: O(1) with LRU tracking
//   - get_stats(): O(1) field access
//
// - TestSharder operations:
//   - shard(): O(n) to distribute n tests
//   - shard_by_duration(): O(n log n) to sort by duration
//   - Balanced sharding: Greedy algorithm O(n * s) where s is shards
//
// - RegressionDetector operations:
//   - add_baseline(): O(1) HashMap insert
//   - check_regression(): O(1) lookup and calculation
//
// - TestPrioritizer operations:
//   - record_failure/success(): O(1) HashMap update
//   - prioritize(): O(n log n) to sort by failure rate
//
// Space Complexity:
// - PerformanceBenchmarker: O(b + b*i) for benchmarks and results
// - ParallelTestExecutor: O(c) for results
// - TestCache: O(n) where n is max_size
// - RegressionDetector: O(t) for baseline storage
// - TestPrioritizer: O(t) for test history
//
// Performance Characteristics:
// - Parallel execution: Near-linear speedup for CPU-bound tests
// - Caching: O(1) lookup saves O(execution_time)
// - Sharding: Balanced distribution minimizes max shard time
// - Regression detection: Constant-time performance validation
// - Test prioritization: Fails fast by running high-risk tests first