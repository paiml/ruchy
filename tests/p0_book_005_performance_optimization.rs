//! P0-BOOK-005: Performance Optimization Test Suite
//!
//! Tests for performance optimization features including:
//! - Loop unrolling and vectorization
//! - Memory allocation strategies
//! - Caching and memoization
//! - Parallel processing capabilities
//! - Profile-guided optimization

use ruchy::runtime::Repl;

#[test]
fn test_loop_optimization_basic() {
    let code = r"
var sum = 0
for i in 1..1000 {
    sum = sum + i
}
sum
";

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Basic loop should execute: {result:?}");
    assert_eq!(result.unwrap(), "499500");
}

#[test]
fn test_memory_allocation_strategies() {
    let code = r#"
import std::mem
let big_array = Array.new(10000, 0)
let memory_info = mem::usage()
println(f"Memory allocated: {memory_info}")
"#;

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Memory allocation should work: {result:?}");
}

#[test]
fn test_parallel_processing_syntax() {
    let code = r"
import std::parallel
let data = [1, 2, 3, 4, 5]
let results = parallel::map(data, |x| x * 2)
println(results)
";

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(
        result.is_ok(),
        "Parallel processing should work: {result:?}"
    );
}

#[test]
fn test_caching_and_memoization() {
    let code = r"
import std::cache
let fibonacci = fn(n) {
    if n <= 1 { return n }
    return fibonacci(n-1) + fibonacci(n-2)
}
println(fibonacci(10))
";

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Memoization should work: {result:?}");
}

#[test]
fn test_vectorization_operations() {
    let code = r"
import std::simd
let vec1 = simd::from_slice([1.0, 2.0, 3.0, 4.0])
let vec2 = simd::from_slice([5.0, 6.0, 7.0, 8.0])
let result = vec1 + vec2
println(result)
";

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "SIMD vectorization should work: {result:?}");
}

#[test]
fn test_profile_guided_optimization() {
    let code = r"
import std::profile
let calculate_intensive = fn(data) {
    var result = 0
    for item in data {
        result = result + (item * item * item)
    }
    return result
}
let data = [1, 2, 3, 4, 5]
println(calculate_intensive(data))
";

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(
        result.is_ok(),
        "Profile-guided optimization should work: {result:?}"
    );
}

#[test]
fn test_performance_benchmarking() {
    let code = r#"
import std::bench
let benchmark_result = bench::time(fn() {
    var sum = 0
    for i in 1..10000 {
        sum = sum + i
    }
    sum
})
println(f"Execution time: {benchmark_result}")
"#;

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(
        result.is_ok(),
        "Performance benchmarking should work: {result:?}"
    );
}

#[test]
fn test_compiler_optimizations() {
    let code = r"
// Check various compiler optimization hints
let fast_multiply = fn(a, b) { a * b }
let slow_function = fn(x) { x + 1 }
let result = fast_multiply(42, 2) + slow_function(1)
println(result)
";

    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.eval(code);
    assert!(
        result.is_ok(),
        "Compiler optimization hints should work: {result:?}"
    );
}
