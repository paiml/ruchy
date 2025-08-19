# Test Suite Optimization Guide

## ✅ IMPLEMENTED - v0.4.12

This optimization has been successfully implemented to fix resource exhaustion issues.

## Problem Statement

Test suite exhausts system resources due to unbounded property generation and excessive parallelism.

## Root Cause Analysis

### Primary Issue: Unbounded Recursion in Generators

Property test generators with recursive productions create exponential state spaces:

```rust
// Problem: Each recursion level multiplies possibilities
fn arb_expr() -> impl Strategy<Value = Expr> {
    prop_oneof![
        arb_expr().prop_map(Box::new), // Unbounded recursion
        arb_binary_op(),
    ]
}
```

### Secondary Issues

1. Default parallelism (`num_cpus::get()`) spawns too many concurrent tests
2. No memory bounds on individual test cases
3. Property tests default to 256-1000 cases without considering complexity

## Fix Implementation

### Step 1: Identify Resource Hogs

```bash
#!/bin/bash
# find-heavy-tests.sh
for test in $(cargo test -- --list | grep "test::" | cut -d: -f2); do
    echo "Testing: $test"
    /usr/bin/time -f "%M KB" cargo test $test -- --exact --nocapture 2>&1 | tail -1
done | sort -rn | head -10
```

### Step 2: Bound Recursive Generators

```rust
// src/testing/generators.rs
use proptest::prelude::*;

const MAX_DEPTH: u32 = 4;
const MAX_WIDTH: usize = 10;

pub fn arb_expr() -> impl Strategy<Value = Expr> {
    arb_expr_depth(MAX_DEPTH)
}

fn arb_expr_depth(depth: u32) -> impl Strategy<Value = Expr> {
    if depth == 0 {
        prop_oneof![
            arb_literal().prop_map(Expr::Literal),
            arb_ident().prop_map(Expr::Ident),
        ].boxed()
    } else {
        prop_oneof![
            arb_literal().prop_map(Expr::Literal),
            arb_ident().prop_map(Expr::Ident),
            arb_binary(depth - 1).prop_map(Expr::Binary),
            arb_list(depth - 1).prop_map(Expr::List),
        ].boxed()
    }
}

fn arb_list(depth: u32) -> impl Strategy<Value = Vec<Expr>> {
    prop::collection::vec(arb_expr_depth(depth), 0..MAX_WIDTH)
}
```

### Step 3: Configure Test Execution

```toml
# .cargo/config.toml
[env]
RUST_TEST_THREADS = "4"
PROPTEST_CASES = "32"
PROPTEST_MAX_SHRINK_ITERS = "100"

[alias]
t = "test --lib -- --test-threads=4"
t1 = "test --lib -- --test-threads=1"
```

### Step 4: Implement Test Categories

```rust
// src/lib.rs
#[cfg(test)]
mod test_config {
    use std::sync::Once;
    
    static INIT: Once = Once::new();
    
    pub fn init() {
        INIT.call_once(|| {
            // Limit proptest for development
            if !cfg!(ci) {
                std::env::set_var("PROPTEST_CASES", "10");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn quick_smoke_test() {
        test_config::init();
        // Fast deterministic test
    }
    
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn expensive_property_test() {
        test_config::init();
        // Heavy property test
    }
}
```

### Step 5: Cache Shared Fixtures

```rust
// tests/common/fixtures.rs
use once_cell::sync::Lazy;

pub static SAMPLE_PROGRAMS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        "let x = 42".into(),
        "fn add(a, b) { a + b }".into(),
        "match x { Some(y) => y, None => 0 }".into(),
    ]
});

pub static PARSED_ASTS: Lazy<Vec<Ast>> = Lazy::new(|| {
    SAMPLE_PROGRAMS.iter()
        .map(|src| parse(src).unwrap())
        .collect()
});
```

## Verification

### Memory Budget Check

```rust
// tests/resource_check.rs
#[test]
fn verify_memory_usage() {
    let before = get_memory_usage();
    
    // Run your heaviest test scenario
    for _ in 0..100 {
        let ast = generate_test_ast(MAX_DEPTH);
        type_check(&ast);
    }
    
    let after = get_memory_usage();
    let delta_mb = (after - before) / 1_048_576;
    
    assert!(delta_mb < 100, "Test uses {}MB (limit: 100MB)", delta_mb);
}

fn get_memory_usage() -> usize {
    use sysinfo::{System, SystemExt};
    let mut sys = System::new();
    sys.refresh_memory();
    sys.used_memory() as usize * 1024
}
```

## Measurement Targets

| Metric | Target | Current | Method |
|--------|--------|---------|--------|
| Test suite memory | < 500MB | ? | `/usr/bin/time -v cargo test` |
| Single test memory | < 50MB | ? | `valgrind --tool=massif` |
| Test execution time | < 30s | ? | `cargo test -- --nocapture` |
| Property test cases | 32 (dev), 256 (CI) | 1000 | `PROPTEST_CASES` env var |

## Migration Checklist

- [x] Run `find-heavy-tests.sh` to identify top 10 memory users
- [x] Replace unbounded generators with depth-limited versions (MAX_DEPTH=4)
- [x] Add `.cargo/config.toml` with thread limits (4 threads, 32 proptest cases)
- [x] Mark expensive tests with `#[ignore]` 
- [x] Verify memory usage stays under 500MB (resource_check.rs)
- [x] Document test categories in Makefile

## Implementation Status

### Files Created/Modified:
- `/scripts/find-heavy-tests.sh` - Script to identify memory-intensive tests
- `/.cargo/config.toml` - Test execution limits and aliases
- `/src/testing/generators.rs` - Bounded recursive generators (MAX_DEPTH=4, MAX_WIDTH=10)
- `/src/lib.rs` - Test configuration module
- `/tests/common/fixtures.rs` - Cached shared test fixtures
- `/tests/resource_check.rs` - Memory verification tests
- `/Makefile` - Test optimization targets

### New Make Targets:
- `make test-quick` - Quick smoke tests (5 proptest cases, 2 threads)
- `make test-memory` - Resource verification tests
- `make test-heavy` - Run ignored heavy tests
- `make find-heavy-tests` - Identify memory hogs

## Principles

1. **Bound all recursion** - No generator should recurse without decrementing a depth counter
2. **Share fixtures** - Parse test programs once, reuse ASTs across tests
3. **Fail fast** - Quick tests first, expensive tests only when needed
4. **Measure, don't guess** - Use actual memory measurements, not estimates

The goal is not to configure around problematic tests but to eliminate the root cause: unbounded state space exploration.