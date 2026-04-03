# Sub-spec: Directory Walking — Testing & Quality Gates

**Parent:** [multi-threaded-dir-walk-spec.md](../multi-threaded-dir-walk-spec.md) Testing Section

---

## Testing Requirements (EXTREME TDD)

### Test Suite Structure

```
tests/
└── stdlib_dir_walk_test.rs          # Main test suite
    ├── Basic walk() tests            (10 tests)
    ├── Parallel walk_parallel() tests (8 tests)
    ├── Advanced walk_with_options() (12 tests)
    ├── Utility glob()/find() tests   (6 tests)
    ├── Text search() tests           (8 tests)
    ├── Integration tests             (6 tests)
    ├── Property tests                (4 tests, 40K cases)
    ├── Concurrency tests (NEW)       (3 tests - loom, sanitizer, stress)
    ├── Security tests (NEW)          (5 tests - traversal, symlinks, unicode, injection, TOCTOU)
    ├── Performance benchmarks (NEW)  (2 tests - overhead, speedup)
    └── Error handling tests          (6 tests)

Total: 70 tests + 2 benchmarks (INCREASED from 60)
```

### Test Categories

#### 1. Basic Walk Tests (10 tests)
```rust
#[test]
fn test_stdlib005_walk_basic() {
    // Create test directory structure
    let temp_dir = create_test_tree();

    let code = format!(r#"
        let entries = walk("{}")
        assert(entries.len() > 0)
        println("Found {} entries", entries.len())
    "#, temp_dir.path().display());

    ruchy_cmd().arg("-e").arg(code)
        .assert().success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_walk_filter_files() {
    let code = r#"
        let files = walk("/tmp/test")
            .filter(|e| e.is_file)

        for f in files {
            println("File: {}", f.path)
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_walk_depth() {
    let code = r#"
        let entries = walk("/tmp/test")

        let depths = entries.map(|e| e.depth).unique()
        assert(depths.contains(0))  # Root
        assert(depths.contains(1))  # First level
    "#;
    // ... assertions
}
```

#### 2. Parallel Processing Tests (8 tests)
```rust
#[test]
fn test_stdlib005_walk_parallel_callback() {
    let code = r#"
        let results = walk_parallel("/tmp/test", |entry| {
            if entry.is_file {
                return { path: entry.path, size: entry.size }
            }
        })

        let files = results.filter(|r| r != nil)
        assert(files.len() > 0)
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_walk_parallel_word_count() {
    // Real-world example: count words in parallel
    let code = r#"
        let totals = walk_parallel("/tmp/docs", |entry| {
            if entry.path.ends_with(".txt") {
                let content = read_file(entry.path)
                return content.split(" ").len()
            }
        })

        let total = totals.filter(|t| t != nil).reduce(0, |a, x| a + x)
        println("Total words: {}", total)
    "#;
    // ... assertions
}
```

#### 3. Advanced Options Tests (12 tests)
```rust
#[test]
fn test_stdlib005_walk_max_depth() {
    let code = r#"
        let entries = walk_with_options("/tmp/test", {
            max_depth: 2
        })

        let max_depth = entries.map(|e| e.depth).max()
        assert(max_depth <= 2)
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_walk_follow_symlinks() {
    let code = r#"
        let entries = walk_with_options("/tmp/test", {
            follow_links: true
        })

        let symlinks = entries.filter(|e| e.is_symlink)
        # Should include symlink targets
    "#;
    // ... assertions
}
```

#### 4. Text Search Tests (8 tests)
```rust
#[test]
fn test_stdlib005_search_basic() {
    // Create test files with known content
    let temp_dir = create_test_files_with_content();

    let code = format!(r#"
        let matches = search("error", "{}")

        assert(matches.len() > 0)
        for match in matches {{
            println("{{}}:{{}}: {{}}", match.path, match.line_num, match.line)
        }}
    "#, temp_dir.path().display());

    ruchy_cmd().arg("-e").arg(code)
        .assert().success()
        .stdout(predicate::str::contains("error"));
}

#[test]
fn test_stdlib005_search_case_insensitive() {
    let code = r#"
        let matches = search("ERROR", "/tmp/test", {
            case_insensitive: true
        })

        # Should match "error", "Error", "ERROR"
        assert(matches.len() > 0)
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_with_context() {
    let code = r#"
        let matches = search("target", "/tmp/test", {
            context_lines: 2
        })

        for match in matches {
            # Should have before and after context
            assert(match.before != nil)
            assert(match.after != nil)
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_file_types() {
    let code = r#"
        let matches = search("fn main", "/project", {
            file_types: ["rs", "rust"]
        })

        # Should only search Rust files
        for match in matches {
            assert(match.path.ends_with(".rs"))
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_regex_pattern() {
    let code = r#"
        # Search for phone numbers
        let matches = search(r"\d{3}-\d{3}-\d{4}", "/data")

        for match in matches {
            println("Found phone: {}", match.match_text)
        }
    "#;
    // ... assertions
}

#[test]
fn test_stdlib005_search_count_only() {
    let code = r#"
        let counts = search("TODO", "/project", {
            count_only: true
        })

        for result in counts {
            println("{}: {} matches", result.path, result.count)
        }
    "#;
    // ... assertions
}
```

#### 5. Property Tests (4 tests, 10K cases each)
```rust
proptest! {
    #[test]
    fn prop_walk_never_panics(depth in 0..5usize, file_count in 0..20usize) {
        let temp = create_random_tree(depth, file_count);
        let code = format!(r#"
            let entries = walk("{}")
            assert(entries.len() >= 0)
        "#, temp.path().display());

        ruchy_cmd().arg("-e").arg(code)
            .assert().success();
    }

    #[test]
    fn prop_parallel_same_results(depth in 1..4usize) {
        // Verify walk() and walk_parallel() return same entries
        let temp = create_test_tree(depth);

        let code = format!(r#"
            let serial = walk("{}").map(|e| e.path).sort()

            let parallel = walk_parallel("{}", |e| e.path).sort()

            assert_eq(serial, parallel)
        "#, temp.path().display(), temp.path().display());

        ruchy_cmd().arg("-e").arg(code)
            .assert().success();
    }
}
```

#### 6. Concurrency Safety Tests (CRITICAL - Identified in Review)

**Problem**: Standard tests don't reliably expose race conditions due to non-deterministic thread interleavings.

**Required Testing Tools**:

```rust
// 1. Loom - Systematic concurrency testing (explores all interleavings)
#[cfg(test)]
#[cfg(loom)]
mod concurrency_tests {
    use loom::thread;
    use loom::sync::Arc;

    #[test]
    fn test_walk_parallel_no_data_races() {
        loom::model(|| {
            // Test that walk_parallel doesn't cause data races
            // when closures access shared state (even though they shouldn't)
            let shared = Arc::new(loom::sync::Mutex::new(0));

            // This should FAIL if walk_parallel allows unsafe access
            walk_parallel("/test", |entry| {
                let mut val = shared.lock().unwrap();
                *val += 1;
            });
        });
    }
}

// 2. Thread Sanitizer - Dynamic race detection
// Run ALL tests with: RUSTFLAGS="-Zsanitizer=thread" cargo +nightly test

// 3. Stress Testing - High-contention scenarios
#[test]
fn stress_test_parallel_walk_contention() {
    // Create large directory tree (10K files)
    let temp = create_large_tree(10_000);

    // Run parallel walk with high contention
    for _ in 0..100 {
        let _ = walk_parallel(temp.path(), |entry| {
            // Simulate work
            thread::sleep(Duration::from_micros(1));
            entry.path
        });
    }
}
```

**Quality Gate Requirements**:
1. ✅ **Loom tests**: All concurrency patterns verified under systematic exploration
2. ✅ **Thread Sanitizer**: Zero data races detected in test suite
3. ✅ **Stress tests**: No panics or deadlocks under high contention (100 iterations)

#### 7. Security Tests (CRITICAL - Identified in Review)

**Problem**: Directory walking is a common attack vector. Must test for malicious inputs.

**Attack Vectors to Test**:

```rust
#[test]
fn security_test_directory_traversal_attack() {
    // Attempt to escape via ../ sequences
    let result = ruchy_cmd()
        .arg("-e")
        .arg(r#"walk("../../../../../../etc/passwd")"#)
        .assert();

    // Should either sanitize path or reject with clear error
    // MUST NOT expose system files outside project directory
}

#[test]
fn security_test_symlink_bomb() {
    // Create circular symlink structure
    let temp = TempDir::new().unwrap();
    let a = temp.path().join("a");
    let b = temp.path().join("b");

    std::os::unix::fs::symlink(&b, &a).unwrap();
    std::os::unix::fs::symlink(&a, &b).unwrap();

    let code = format!(r#"
        walk_with_options("{}", {{
            follow_links: true,
            max_depth: 100
        }})
    "#, temp.path().display());

    // MUST detect cycle and terminate (not infinite loop)
    ruchy_cmd().arg("-e").arg(code)
        .assert()
        .success(); // Or controlled failure
}

#[test]
fn security_test_unicode_normalization() {
    // Test Unicode homograph attacks (e.g., Cyrillic 'а' vs Latin 'a')
    // Create files with visually similar but different Unicode paths
    // Ensure walk() reports them as distinct files
}

#[test]
fn security_test_path_injection() {
    // Test for command injection via malicious filenames
    let temp = TempDir::new().unwrap();
    let evil_file = temp.path().join("; rm -rf /");
    File::create(&evil_file).unwrap();

    let code = format!(r#"
        walk("{}").for_each(|e| {{
            println("File: {{}}", e.path)
        }})
    "#, temp.path().display());

    // MUST NOT execute injected commands
    ruchy_cmd().arg("-e").arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("; rm -rf /"));
}

#[test]
fn security_test_time_of_check_time_of_use() {
    // Test TOCTOU vulnerabilities
    // Create file, walk() finds it, delete it, callback accesses it
    // MUST handle gracefully (not panic or expose undefined behavior)
}
```

**Security Quality Gates**:
1. ✅ **Directory traversal**: All `../` escape attempts blocked or sanitized
2. ✅ **Symlink loops**: Detected and terminated within max_depth limit
3. ✅ **Unicode attacks**: Filenames properly validated and normalized
4. ✅ **Path injection**: No command execution via malicious filenames
5. ✅ **TOCTOU races**: Graceful handling of filesystem changes during walk

### Mutation Testing Requirements

**Target**: ≥90% mutation coverage (INCREASED from 80% per critical review)

**Rationale**: For foundational libraries dealing with concurrency and filesystem operations, subtle bugs have severe consequences. Higher mutation coverage provides confidence that boundary conditions and concurrent interactions are thoroughly tested.

**Strategy**: File-level mutation testing
```bash
# Test walk() implementation
cargo mutants --file src/runtime/eval_dir_walk.rs \
    --test stdlib_dir_walk_test \
    --timeout 300

# Expected: ≥80% coverage
```

**Critical Mutations to Catch**:
1. **Match arm deletions**: All filter conditions must be tested
2. **Boolean negations**: Test is_file vs is_dir boundaries
3. **Boundary conditions**: Test depth limits (max_depth, min_depth)
4. **Iterator transformations**: Verify parallel vs serial behavior
5. **Option handling**: Test all walk_with_options parameters

---

## Quality Gates (MANDATORY)

Following stdlib quality protocol from `docs/execution/roadmap.yaml`:

### Gates (ALL BLOCKING - Updated per Critical Review):

**Functional Correctness**:
1. ✅ **Unit Tests**: ≥60 tests passing (100% pass rate)
2. ✅ **Property Tests**: 4 tests × 10K cases = 40K total validations
3. ✅ **Integration Tests**: 6 real-world scenario tests passing (including security audit)

**Concurrency Correctness** (NEW - Critical):
4. ✅ **Loom Tests**: All concurrency patterns verified under systematic exploration
5. ✅ **Thread Sanitizer**: Zero data races detected (`RUSTFLAGS="-Zsanitizer=thread"`)
6. ✅ **Stress Tests**: No panics/deadlocks under high contention (100 iterations)

**Security** (NEW - Critical):
7. ✅ **Directory Traversal**: All `../` escape attempts blocked/sanitized
8. ✅ **Symlink Loops**: Detected and terminated within max_depth
9. ✅ **Unicode Attacks**: Filenames properly validated and normalized
10. ✅ **Path Injection**: No command execution via malicious filenames
11. ✅ **TOCTOU Races**: Graceful handling of filesystem changes during walk

**Quality & Performance**:
12. ✅ **Mutation Tests**: ≥90% coverage (INCREASED from 80% per review)
13. ✅ **Complexity**: All functions ≤10 cyclomatic complexity
14. ✅ **Abstraction Overhead**: <1µs per item (measured via benchmarks) (NEW)
15. ✅ **Parallel Speedup**: ≥2x faster than serial (4+ cores)
16. ✅ **Documentation**: All 6 functions have doctests + examples + algorithm references

### Performance Benchmarks

**CRITICAL (Per Review)**: Benchmark abstraction overhead to validate "high-performance" claim.

```rust
#[test]
#[ignore]
fn bench_abstraction_overhead() {
    // Measure Ruchy-to-Rust boundary crossing cost per item
    use std::time::Instant;

    let temp = create_large_tree(10_000); // 10K files

    // Benchmark: Pure Rust (baseline)
    let start = Instant::now();
    let rust_count = walkdir::WalkDir::new(temp.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .count();
    let rust_time = start.elapsed();

    // Benchmark: Ruchy wrapper
    let start = Instant::now();
    let code = format!(r#"
        let entries = walk("{}")
        println("Count: {{}}", entries.len())
    "#, temp.path().display());
    ruchy_cmd().arg("-e").arg(code).assert().success();
    let ruchy_time = start.elapsed();

    // Calculate per-item overhead
    let overhead_per_item = (ruchy_time - rust_time).as_micros() as f64 / rust_count as f64;

    println!("Pure Rust: {:?} ({} files)", rust_time, rust_count);
    println!("Ruchy wrapper: {:?}", ruchy_time);
    println!("Overhead per item: {:.3}µs", overhead_per_item);

    // QUALITY GATE: Overhead must be <1µs per item
    assert!(
        overhead_per_item < 1.0,
        "Abstraction overhead {:.3}µs exceeds 1µs budget",
        overhead_per_item
    );
}

#[test]
#[ignore]  // Run with: cargo test --test stdlib_dir_walk_test -- --ignored
fn bench_parallel_speedup() {
    use std::time::Instant;

    let temp = create_large_tree(1000); // 1000 files

    // Serial walk
    let start = Instant::now();
    let code = format!(r#"
        let results = walk("{}")
            .filter(|e| e.is_file)
            .map(|e| {{ path: e.path, size: e.size }})
    "#, temp.path().display());
    ruchy_cmd().arg("-e").arg(code).assert().success();
    let serial_time = start.elapsed();

    // Parallel walk
    let start = Instant::now();
    let code = format!(r#"
        let results = walk_parallel("{}", |e| {{
            if e.is_file {{ path: e.path, size: e.size }}
        }})
    "#, temp.path().display());
    ruchy_cmd().arg("-e").arg(code).assert().success();
    let parallel_time = start.elapsed();

    let speedup = serial_time.as_secs_f64() / parallel_time.as_secs_f64();
    println!("Speedup: {:.2}x", speedup);

    // Verify ≥2x speedup on 4+ core systems
    if num_cpus::get() >= 4 {
        assert!(speedup >= 2.0, "Expected ≥2x speedup, got {:.2}x", speedup);
    }
}
```

---

