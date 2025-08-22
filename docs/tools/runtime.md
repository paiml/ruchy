# Performance Analysis with `ruchy runtime`

**World's First**: Automatic BigO algorithmic complexity detection in a programming language.

## Overview

The `ruchy runtime` tool automatically analyzes your code's algorithmic complexity, providing BigO notation analysis, performance profiling, and optimization suggestions. This revolutionary feature helps developers understand and optimize their code's performance characteristics without manual analysis.

## Features

### Automatic BigO Detection
Identifies algorithmic complexity patterns: O(1), O(log n), O(n), O(n log n), O(n²), O(n³), O(2^n).

### Nested Loop Analysis
Detects and analyzes nested loop complexity with worst-case scenarios.

### Recursive Pattern Recognition
Identifies common recursive patterns (divide-and-conquer, linear, tree recursion).

### Performance Profiling
Function-level timing and hot-spot identification.

### Memory Usage Analysis
Tracks allocation patterns and memory complexity.

## Usage

### Basic Performance Metrics

```bash
ruchy runtime script.ruchy
```

Output:
```
⚡ Basic Performance Metrics for script.ruchy
  Total Functions: 12
  Recursive Functions: 3
  Loop Complexity: O(n²)
  Estimated Runtime: O(n²)
  Optimization Score: 72/100
  
  ⚠️ Potential Bottlenecks:
    • Nested loops in process_matrix (line 45)
    • Recursive fibonacci without memoization (line 23)
```

### BigO Complexity Analysis

```bash
ruchy runtime script.ruchy --bigo
```

Output:
```
🔬 BigO Complexity Analysis for script.ruchy

Function Complexities:
  • linear_search: O(n) - Linear time
  • binary_search: O(log n) - Logarithmic time
  • bubble_sort: O(n²) - Quadratic time
  • merge_sort: O(n log n) - Linearithmic time
  • fibonacci: O(2^n) - Exponential time ⚠️

Overall Complexity: O(n²)
Worst Case: O(2^n) in fibonacci function

Optimization Suggestions:
  1. Use memoization for fibonacci function
  2. Replace bubble_sort with merge_sort
  3. Consider using HashMap for linear_search scenarios
```

### Execution Profiling

```bash
ruchy runtime script.ruchy --profile --verbose
```

Output:
```
📊 Execution Profiling for script.ruchy

Execution Time: 45ms
Call Graph Depth: 5
Functions Analyzed: 8

🔥 Hot Spots:
  • process_data: 35ms (77.8%)
  • validate_input: 5ms (11.1%)
  • format_output: 3ms (6.7%)

Function Timing Details:
  🔴 process_data: 35ms - CRITICAL PATH
  🟡 validate_input: 5ms - Moderate
  🟢 format_output: 3ms - Acceptable
  🟢 helper_func: 1ms - Fast
```

### Benchmarking

```bash
ruchy runtime script.ruchy --bench
```

Output:
```
🏁 Benchmark Execution for script.ruchy

Iterations: 1000
Mean Time: 12.5ms
Std Dev: 1.2ms
Min: 10.1ms
Max: 18.3ms
P50: 12.3ms
P95: 14.8ms
P99: 16.2ms

Performance Grade: B+ (Good)
```

### Performance Comparison

```bash
ruchy runtime v1.ruchy --compare v2.ruchy
```

Output:
```
🔀 Performance Comparison: v1.ruchy vs v2.ruchy

Metric          | v1.ruchy | v2.ruchy | Change
----------------|----------|----------|--------
Complexity      | O(n²)    | O(n log n)| ✅ -50%
Execution Time  | 45ms     | 12ms     | ✅ -73%
Memory Usage    | 12MB     | 8MB      | ✅ -33%
Functions       | 15       | 12       | ✅ -20%

Verdict: v2.ruchy is 3.75x faster overall
```

## Examples

### Example 1: Loop Complexity Detection

```ruchy
fun find_duplicates(arr: [i32]) -> [i32] {
    let mut duplicates = []
    
    // O(n²) complexity - nested loops
    for i in 0..arr.len() {
        for j in (i+1)..arr.len() {
            if arr[i] == arr[j] {
                duplicates.push(arr[i])
            }
        }
    }
    
    duplicates
}

fun find_duplicates_optimized(arr: [i32]) -> [i32] {
    let mut seen = HashSet::new()
    let mut duplicates = []
    
    // O(n) complexity - single pass
    for item in arr {
        if seen.contains(item) {
            duplicates.push(item)
        } else {
            seen.insert(item)
        }
    }
    
    duplicates
}
```

Analysis:
```bash
ruchy runtime --bigo example.ruchy
```

Output:
```
find_duplicates: O(n²) - Quadratic
  ⚠️ Nested loops detected (lines 4-10)
  Suggestion: Use HashSet for O(n) solution

find_duplicates_optimized: O(n) - Linear
  ✅ Optimal for this problem
```

### Example 2: Recursive Complexity

```ruchy
// O(2^n) - Exponential
fun fib_naive(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fib_naive(n-1) + fib_naive(n-2)
    }
}

// O(n) - Linear with memoization
fun fib_memo(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
    if n <= 1 {
        return n
    }
    
    if let Some(&result) = cache.get(&n) {
        return result
    }
    
    let result = fib_memo(n-1, cache) + fib_memo(n-2, cache)
    cache.insert(n, result)
    result
}

// O(n) - Iterative
fun fib_iter(n: u64) -> u64 {
    if n <= 1 { return n }
    
    let mut prev = 0
    let mut curr = 1
    
    for _ in 2..=n {
        let next = prev + curr
        prev = curr
        curr = next
    }
    
    curr
}
```

Analysis:
```bash
ruchy runtime --bigo fibonacci.ruchy
```

Output:
```
Complexity Analysis:
  fib_naive: O(2^n) - Exponential ❌
    Pattern: Tree recursion detected
    Warning: Exponential growth - avoid for n > 40
    
  fib_memo: O(n) - Linear ✅
    Pattern: Dynamic programming with memoization
    Space: O(n) for cache
    
  fib_iter: O(n) - Linear ✅
    Pattern: Iterative with constant space
    Space: O(1) - Optimal

Recommendation: Use fib_iter for best performance
```

### Example 3: Sorting Algorithm Comparison

```ruchy
fun bubble_sort(mut arr: [i32]) {
    for i in 0..arr.len() {
        for j in 0..(arr.len() - i - 1) {
            if arr[j] > arr[j + 1] {
                arr.swap(j, j + 1)
            }
        }
    }
}

fun quick_sort(mut arr: [i32]) {
    if arr.len() <= 1 { return }
    
    let pivot = arr[arr.len() / 2]
    let (less, equal, greater) = partition(arr, pivot)
    
    quick_sort(less)
    quick_sort(greater)
    
    arr = less + equal + greater
}
```

Benchmark comparison:
```bash
ruchy runtime sorts.ruchy --bench --compare
```

Output:
```
Algorithm Performance Comparison:

bubble_sort:
  Complexity: O(n²) worst/average, O(n) best
  Time (n=1000): 45ms
  Time (n=10000): 4500ms
  Space: O(1)
  Stable: Yes

quick_sort:
  Complexity: O(n log n) average, O(n²) worst
  Time (n=1000): 2ms
  Time (n=10000): 25ms
  Space: O(log n)
  Stable: No

Winner: quick_sort (22.5x faster on average)
```

## Memory Analysis

```bash
ruchy runtime script.ruchy --memory
```

Output:
```
💾 Memory Usage Analysis for script.ruchy

Allocation Patterns:
  • Stack allocations: 45
  • Heap allocations: 12
  • Total allocated: 256KB
  • Peak usage: 128KB

Memory Complexity:
  • process_array: O(n) space
  • recursive_func: O(log n) stack space
  • build_matrix: O(n²) space ⚠️

Potential Issues:
  ⚠️ Large allocation in build_matrix (line 67)
  ⚠️ Possible memory leak in process_loop (line 89)

Optimization Suggestions:
  1. Use iterative approach instead of recursion
  2. Process matrix in chunks to reduce memory
  3. Clear temporary collections after use
```

## Output Formats

### JSON Output

```bash
ruchy runtime script.ruchy --bigo --json
```

```json
{
  "overall_complexity": "O(n²)",
  "functions": [
    {
      "name": "bubble_sort",
      "complexity": "O(n²)",
      "nested_loops": 2,
      "recursive": false,
      "optimization_score": 40
    }
  ],
  "bottlenecks": ["bubble_sort", "nested_search"],
  "suggestions": [
    "Replace bubble_sort with O(n log n) algorithm",
    "Use HashMap for O(1) lookups"
  ]
}
```

### Performance Report

```bash
ruchy runtime script.ruchy --output perf-report.md
```

Generates comprehensive performance report with:
- Executive summary
- Function-by-function analysis
- Complexity breakdown
- Optimization recommendations
- Benchmark results

## CI/CD Integration

```yaml
# GitHub Actions
- name: Performance Analysis
  run: |
    ruchy runtime src/ --bigo --threshold O(n²)
    if [ $? -ne 0 ]; then
      echo "Performance regression detected"
      exit 1
    fi

- name: Benchmark
  run: |
    ruchy runtime src/ --bench --compare baseline/
    ruchy runtime --output performance.json
```

## Performance Thresholds

Set complexity thresholds for CI/CD:

```bash
# Fail if any function exceeds O(n²)
ruchy runtime --threshold "O(n²)" src/

# Fail if overall complexity exceeds O(n log n)
ruchy runtime --overall-threshold "O(n log n)" src/
```

## Limitations

- Cannot analyze I/O-bound operations
- Approximates complexity for dynamic data structures
- May not detect all amortized complexities
- Limited support for parallel algorithms

## Future Enhancements

- Cache complexity analysis
- Parallel algorithm detection
- Amortized analysis support
- Real-time profiling integration
- Machine learning for optimization suggestions

---

*Revolutionary BigO detection - only in Ruchy v0.10.0*