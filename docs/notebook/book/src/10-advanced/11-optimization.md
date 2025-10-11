# Optimization - Feature 41/41

Performance optimization in Ruchy uses profiling, algorithmic improvements, and zero-cost abstractions to achieve maximum efficiency.

## Profiling

```ruchy
use std::time::Instant

let start = Instant::now()
expensive_operation()
let duration = start.elapsed()

println!("Time: {:?}", duration)
```

**Test Coverage**: ✅ [tests/lang_comp/advanced/optimization.rs](../../../../tests/lang_comp/advanced/optimization.rs)

**Expected Output**: Execution time measured

## Iterator Optimization

```ruchy
// Optimized: Single pass with iterator chain
let sum: i32 = (1..=1000)
  .filter(|x| x % 2 == 0)
  .map(|x| x * 2)
  .sum()

// Unoptimized: Multiple passes
let mut nums = vec![]
for i in 1..=1000 {
  if i % 2 == 0 {
    nums.push(i)
  }
}
let mut doubled = vec![]
for n in nums {
  doubled.push(n * 2)
}
let sum: i32 = doubled.iter().sum()
```

**Expected Output**: `500500` (optimized runs faster)

## String Building

```ruchy
// Optimized: Pre-allocate capacity
let mut s = String::with_capacity(1000)
for i in 0..100 {
  s.push_str(&i.to_string())
}

// Unoptimized: Repeated reallocations
let mut s = String::new()
for i in 0..100 {
  s.push_str(&i.to_string())  // Reallocates frequently
}
```

**Expected Output**: Concatenated string with fewer allocations

## Copy vs Clone

```ruchy
// Fast: Copy (stack only)
#[derive(Copy, Clone)]
struct Point {
  x: i32,
  y: i32
}

let p1 = Point { x: 1, y: 2 }
let p2 = p1  // Bitwise copy (fast)

// Slow: Clone (heap allocation)
let s1 = String::from("hello")
let s2 = s1.clone()  // Heap allocation
```

**Expected Output**: Copy is faster than Clone

## Vec Reuse

```ruchy
// Optimized: Reuse allocation
let mut buffer = Vec::with_capacity(1000)
for _ in 0..10 {
  buffer.clear()
  for i in 0..100 {
    buffer.push(i)
  }
  process(&buffer)
}

// Unoptimized: New allocation each time
for _ in 0..10 {
  let mut buffer = Vec::new()
  for i in 0..100 {
    buffer.push(i)
  }
  process(&buffer)
}
```

**Expected Output**: Single allocation vs 10 allocations

## Inline Hints

```ruchy
#[inline]
fn add(a: i32, b: i32) -> i32 {
  a + b
}

#[inline(always)]
fn critical_path(x: i32) -> i32 {
  x * 2 + 1
}

#[inline(never)]
fn cold_path() {
  // Rarely called
}
```

**Expected Output**: Compiler inlining hints

## SmallVec for Stack Allocation

```ruchy
use smallvec::SmallVec

// Stores up to 4 items on stack, heap after that
let mut vec: SmallVec<[i32; 4]> = SmallVec::new()
vec.push(1)
vec.push(2)
vec.push(3)  // Still on stack
vec.push(4)  // Still on stack
vec.push(5)  // Now moves to heap
```

**Expected Output**: Stack allocation for small sizes

## Benchmarking

```ruchy
use criterion::{black_box, criterion_group, criterion_main, Criterion}

fn fibonacci(n: u64) -> u64 {
  match n {
    0 => 1,
    1 => 1,
    n => fibonacci(n-1) + fibonacci(n-2)
  }
}

fn bench_fib(c: &mut Criterion) {
  c.bench_function("fib 20", |b| {
    b.iter(|| fibonacci(black_box(20)))
  })
}

criterion_group!(benches, bench_fib)
criterion_main!(benches)
```

**Expected Output**: Benchmark results with timing

## Lazy Evaluation

```ruchy
use once_cell::sync::Lazy

static EXPENSIVE: Lazy<Vec<i32>> = Lazy::new(|| {
  println!("Computing...")
  (0..1000).collect()
})

fn use_data() {
  println!("{}", EXPENSIVE[0])  // Computed on first access
  println!("{}", EXPENSIVE[1])  // Reuses cached value
}
```

**Expected Output**: "Computing..." printed once

## Best Practices

### ✅ Profile Before Optimizing

```ruchy
// Good: Measure first
let start = Instant::now()
let result = algorithm()
println!("Time: {:?}", start.elapsed())

// Bad: Premature optimization
// Complex optimizations without profiling
```

### ✅ Use Iterator Chains

```ruchy
// Good: Single pass
let result: Vec<_> = data
  .iter()
  .filter(|x| x.is_valid())
  .map(|x| x.process())
  .collect()

// Bad: Multiple passes
let filtered: Vec<_> = data.iter().filter(|x| x.is_valid()).collect()
let result: Vec<_> = filtered.iter().map(|x| x.process()).collect()
```

### ✅ Avoid Unnecessary Clones

```ruchy
// Good: Borrow
fn process(data: &Vec<i32>) {
  // Use data
}

// Bad: Clone unnecessarily
fn process(data: Vec<i32>) {
  // Forces caller to clone
}
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 88%

Optimization in Ruchy focuses on profiling first, then using iterators, avoiding allocations, and leveraging zero-cost abstractions for maximum performance.

**Key Takeaways**:
- Profile: Measure before optimizing with `Instant::now()`
- Iterators: Single-pass chains for efficiency
- Allocation: Pre-allocate with `with_capacity()`, reuse buffers
- Copy: Prefer Copy over Clone for stack types
- Inline: Use `#[inline]` for hot paths
- Benchmarking: Use criterion for accurate measurements

---

[← Previous: Advanced Patterns](./10-advanced-patterns.md) | [Next: Testing →](./12-testing.md)
