# Multi-Threaded Directory Walking Specification (STDLIB-005)

**Version**: 2.0.0
**Status**: HARDENED (Post-Critical Review)
**Created**: 2025-10-20
**Updated**: 2025-10-20 (Critical review applied)
**Target Release**: v3.100.0
**Ticket**: STDLIB-005

## Critical Review Applied (Toyota Way: Jidoka - Stop the Line)

This specification underwent rigorous critical review using Five Whys and Genchi Genbutsu principles. The following **5 fundamental design flaws** were identified and corrected:

1. ✅ **Memory Scalability Defect**: walk_parallel() eager collection → OOM on large directories
   - **Fix**: Documented defect, proposed iterator-based API for v2.0
2. ✅ **Abstraction Cost Not Measured**: "Zero-cost" claim was unvalidated
   - **Fix**: Renamed to "high-performance", added <1µs overhead benchmark as quality gate
3. ✅ **Concurrency Testing Gap**: No systematic concurrency testing
   - **Fix**: Added loom tests, thread sanitizer, stress tests (3 new test categories)
4. ✅ **Security Testing Gap**: No directory traversal/symlink attack tests
   - **Fix**: Added 5 security test categories (directory traversal, symlink bombs, unicode, path injection, TOCTOU)
5. ✅ **Missing Algorithm Justification**: Design not grounded in CS research
   - **Fix**: Added theoretical foundations section with peer-reviewed algorithm references (Aho-Corasick, Blumofe & Leiserson work-stealing, Thompson NFA)

**Kaizen Result**: Quality gates increased from 7 → 16, mutation coverage 80% → 90%, estimated time 10-14h → 14-18h

## Overview

Multi-threaded directory walking and fast text search for Ruchy, combining:
- **Directory walking**: Python's `os.walk()` ergonomics + Rust's `walkdir` + `rayon` performance
- **Text search**: Ripgrep's speed with Ruchy's simplicity via `grep` crate
- **CLI tools**: 5 production-ready sysadmin utilities (find, tree, du, count, rg)
- **Programmatic API**: 6 functions for data science, data engineering, and automation workflows

## Design Philosophy

1. **Simple by default, powerful when needed**
   - **CLI tools for sysadmins** (no coding required, like `python -m http.server`)
   - Basic walk() for simple cases
   - Advanced options for complex scenarios
   - Parallel processing built-in, not bolted-on

2. **High-performance abstraction** (corrected from "zero-cost")
   - Direct wrapping of `walkdir` crate (proven, battle-tested)
   - Parallel iteration via `rayon` (optimal work-stealing scheduler based on Cilk work-stealing)
   - **⚠️ Reality Check**: Interpreter boundary crossings are NOT zero-cost
   - **Quality Gate**: Boundary overhead must be <1µs per item (measured via benchmarks)
   - **Justification**: "Zero-cost" applies to compiled Rust, not interpreter FFI calls

3. **Data pipeline friendly**
   - Iterator-based API (chainable, composable)
   - Integrates with existing array methods (map, filter, reduce)
   - Natural fit for ETL workflows

4. **Three ways to use** (following Ruchy hybrid pattern from http-server-mvp-spec):
   - **CLI** - Quick sysadmin tasks (like GNU find, tree, du, wc)
   - **Import** - Programmatic usage in .ruchy scripts
   - **Task Runner** - Workflow automation via ruchy.yaml

---


## Sub-spec Index

| Sub-spec | Scope |
|----------|-------|
| [CLI Tools](sub/dirwalk-cli.md) | 5 production-ready CLI utilities (find, tree, du, count, rg) |
| [API Design & Use Cases](sub/dirwalk-api.md) | Programmatic API, implementation architecture, use case examples |
| [Testing & Quality Gates](sub/dirwalk-testing.md) | EXTREME TDD, quality gates, concurrency/security testing |

---

## Theoretical Foundations and Algorithm References

**Purpose**: Ground design decisions in peer-reviewed computer science research (per critical review).

### 1. Work-Stealing Scheduler (Rayon)

**Algorithm**: Cilk-style work-stealing based on **[Blumofe & Leiserson, 1999]**

**Key Properties**:
- **Time Complexity**: T_p ≤ T_1/p + O(T_∞) where:
  - T_1 = sequential execution time
  - T_∞ = critical path length (span)
  - p = number of processors
- **Space Complexity**: O(p × T_∞) (provably efficient)
- **Locality**: LIFO for local tasks (cache-friendly), FIFO for stolen tasks

**Implementation Details**:
```rust
// Rayon's work-stealing deque
// - Each thread has local deque (LIFO for own work)
// - Steals from other threads' tails (FIFO for stolen work)
// - Lock-free chase-lev deque algorithm [Chase & Lev, 2005]
```

**References**:
- Blumofe, R. D., & Leiserson, C. E. (1999). "Scheduling multithreaded computations by work stealing." *Journal of the ACM*, 46(5), 720-748.
- Chase, D., & Lev, Y. (2005). "Dynamic circular work-stealing deque." *SPAA '05*.

### 2. Fast Text Search (grep crate / ripgrep)

**Algorithms**:

**2.1 Multi-Pattern Matching: Aho-Corasick [1975]**
- **Purpose**: Find multiple patterns simultaneously in O(n + m + z) time
  - n = text length
  - m = total pattern length
  - z = number of matches
- **Used For**: Searching for multiple keywords (e.g., "error", "warning", "fatal")

**2.2 SIMD String Searching**
- **Vectorized memchr**: Uses SSE2/AVX2 instructions for byte scanning
- **Performance**: Up to 16x speedup on modern CPUs
- **Implementation**: `memchr` crate (used by ripgrep)

**2.3 Regular Expression Engines**
- **Thompson NFA Construction**: Linear time regex matching (no catastrophic backtracking)
- **DFA Caching**: Lazy DFA construction for frequently-used patterns
- **Hybrid Approach**: Switch between NFA and DFA based on pattern complexity

**References**:
- Aho, A. V., & Corasick, M. J. (1975). "Efficient string matching: an aid to bibliographic search." *Communications of the ACM*, 18(6), 333-340.
- Cox, R. (2007). "Regular Expression Matching Can Be Simple And Fast." https://swtch.com/~rsc/regexp/
- Thompson, K. (1968). "Programming Techniques: Regular expression search algorithm." *Communications of the ACM*, 11(6), 419-422.

### 3. Parallel Iterator Abstraction

**Design Pattern**: Iterator-based parallelism [Dean & Ghemawat, 2004 (MapReduce)]

**Key Insight**: Lazy evaluation + parallel execution = composable high-performance pipelines

```rust
// Composable pipeline (no intermediate collections)
walk("/data")
    .par_iter()           // Parallel iterator
    .filter(|e| test(e))  // Parallel filter (no sync needed)
    .map(|e| process(e))  // Parallel map (pure function)
    .reduce(|| 0, |a, b| a + b)  // Parallel reduce (associative)
```

**Performance**: O(n/p) with p processors (optimal speedup for embarrassingly parallel workloads)

**References**:
- Dean, J., & Ghemawat, S. (2004). "MapReduce: Simplified data processing on large clusters." *OSDI '04*.
- Rayon documentation: https://docs.rs/rayon/latest/rayon/

### 4. Concurrency Correctness

**Testing Strategy**: Systematic Exploration via Loom [Kokologiannakis et al., 2019]

**Problem**: Standard tests explore O(1) thread interleavings out of exponentially many possible schedules

**Solution**: Loom systematically explores all possible schedules under a bounded context switch model

**Theoretical Foundation**: DPOR (Dynamic Partial Order Reduction) [Flanagan & Godefroid, 2005]

**References**:
- Flanagan, C., & Godefroid, P. (2005). "Dynamic partial-order reduction for model checking software." *POPL '05*.
- Kokologiannakis, M., Raad, A., & Vafeiadis, V. (2019). "Model checking for weakly consistent libraries." *PLDI '19*.

---

## Implementation Strategy

### Phase 1: Basic Walk (Single-threaded)
**Pattern**: Zero-cost abstraction over `walkdir` crate

```rust
// src/runtime/eval_builtin.rs
fn eval_walk(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk", args, 1)?;

    match &args[0] {
        Value::String(path) => {
            use walkdir::WalkDir;

            let entries: Vec<Value> = WalkDir::new(path.as_ref())
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|entry| {
                    create_file_entry(&entry)
                })
                .collect();

            Ok(Value::from_array(entries))
        }
        _ => Err(InterpreterError::TypeError(
            "walk() expects string path".to_string()
        ))
    }
}

fn create_file_entry(entry: &walkdir::DirEntry) -> Value {
    let metadata = entry.metadata().unwrap();

    Value::Object(Rc::new(HashMap::from([
        ("path".to_string(), Value::from_string(entry.path().display().to_string())),
        ("name".to_string(), Value::from_string(entry.file_name().to_string_lossy().to_string())),
        ("is_file".to_string(), Value::Bool(metadata.is_file())),
        ("is_dir".to_string(), Value::Bool(metadata.is_dir())),
        ("is_symlink".to_string(), Value::Bool(metadata.file_type().is_symlink())),
        ("size".to_string(), Value::Integer(metadata.len() as i64)),
        ("depth".to_string(), Value::Integer(entry.depth() as i64)),
    ])))
}
```

### Phase 2: Parallel Processing
**Pattern**: Rayon's `par_bridge()` for automatic parallelism

```rust
fn eval_walk_parallel(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk_parallel", args, 2)?;

    use rayon::prelude::*;
    use walkdir::WalkDir;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::Closure { .. }) => {
            let results: Vec<Value> = WalkDir::new(path.as_ref())
                .into_iter()
                .filter_map(|e| e.ok())
                .par_bridge()  // Enable parallel processing
                .map(|entry| {
                    let file_entry = create_file_entry(&entry);

                    // Call user's closure with entry
                    eval_closure(&args[1], &[file_entry])
                        .unwrap_or(Value::Nil)
                })
                .collect();

            Ok(Value::from_array(results))
        }
        _ => Err(InterpreterError::TypeError(
            "walk_parallel(path, callback) expects string and closure".to_string()
        ))
    }
}
```

### Phase 3: Advanced Options
**Pattern**: Object destructuring for optional parameters

```rust
fn eval_walk_with_options(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("walk_with_options", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::Object(opts)) => {
            use walkdir::WalkDir;

            let mut walker = WalkDir::new(path.as_ref());

            // Apply options
            if let Some(Value::Integer(max)) = opts.get("max_depth") {
                walker = walker.max_depth(*max as usize);
            }

            if let Some(Value::Integer(min)) = opts.get("min_depth") {
                walker = walker.min_depth(*min as usize);
            }

            if let Some(Value::Bool(follow)) = opts.get("follow_links") {
                walker = walker.follow_links(*follow);
            }

            let use_parallel = opts.get("parallel")
                .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                .unwrap_or(false);

            let entries = walker.into_iter()
                .filter_map(|e| e.ok());

            if use_parallel {
                use rayon::prelude::*;
                let results: Vec<Value> = entries
                    .par_bridge()
                    .map(|e| create_file_entry(&e))
                    .collect();
                Ok(Value::from_array(results))
            } else {
                let results: Vec<Value> = entries
                    .map(|e| create_file_entry(&e))
                    .collect();
                Ok(Value::from_array(results))
            }
        }
        _ => Err(InterpreterError::TypeError(
            "walk_with_options(path, options) expects string and object".to_string()
        ))
    }
}
```

---

## Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
walkdir = "2.5"      # Recursive directory traversal
rayon = "1.10"       # Data parallelism
glob = "0.3"         # Glob pattern matching (for glob() function)
num_cpus = "1.16"    # CPU core detection
grep = "0.3"         # Fast text search (ripgrep library)
regex = "1.10"       # Regex pattern matching (for search() function)
```

---


## Roadmap Entry (YAML Format)

```yaml
- id: "STDLIB-005"
  title: "Multi-Threaded Directory Walking + Text Search (HARDENED)"
  status: "PLANNED (Post-Critical Review)"
  priority: "🔴 HIGH"
  estimated_time: "14-18h (INCREASED from 10-14h due to security/concurrency hardening)"
  dependencies:
    - "STDLIB-004"  # Array methods needed for chaining
  functions: 6
  cli_tools: 5
  module: "Directory traversal with parallel processing + fast text search (security-hardened)"
  tests:
    unit: 70  # INCREASED from 60
    concurrency: 3  # NEW: loom, thread sanitizer, stress
    security: 5     # NEW: traversal, symlinks, unicode, injection, TOCTOU
    benchmarks: 2   # NEW: abstraction overhead, parallel speedup
    interpreter: 45 # INCREASED from 38
    transpiler: 12  # INCREASED from 10
    integration: 6
    property: 4
    property_cases: 40000
    mutation_target: "≥90%"  # INCREASED from 80%
  quality:
    complexity_max: 10
    tdg_target: "A-"
    quality_gates: 16  # INCREASED from 7
  dependencies_crates:
    - "walkdir = \"2.5\""
    - "rayon = \"1.10\""
    - "glob = \"0.3\""
    - "num_cpus = \"1.16\""
    - "grep = \"0.3\""
    - "regex = \"1.10\""
    - "loom = \"0.7\""      # NEW: Systematic concurrency testing
  theoretical_foundations:
    - "Blumofe & Leiserson (1999): Work-stealing scheduler"
    - "Aho & Corasick (1975): Multi-pattern string matching"
    - "Thompson (1968): NFA-based regex matching"
    - "Flanagan & Godefroid (2005): DPOR for model checking"
  api:
    - "walk(path) -> Array<FileEntry> - Basic recursive walk"
    - "walk_parallel(path, callback) -> Array<Any> - Parallel processing"
    - "walk_with_options(path, options) -> Array<FileEntry> - Advanced options"
    - "glob(pattern) -> Array<String> - Glob pattern matching"
    - "find(path, predicate) -> Array<FileEntry> - Find with predicate"
    - "search(pattern, path, options?) -> Array<SearchMatch> - Fast text search"
  cli:
    - "ruchy find - Smart file finder (simpler than GNU find)"
    - "ruchy tree - Visual directory tree with stats"
    - "ruchy du - Disk usage with visual charts"
    - "ruchy count - File statistics with language detection"
    - "ruchy rg - Fast parallel text search (like ripgrep)"
  implementation_phases:
    - phase: "RED"
      tasks:
        - "Create tests/stdlib_dir_walk_test.rs with 70 unit tests"
        - "Add 3 concurrency tests (loom, thread sanitizer, stress)"
        - "Add 5 security tests (traversal, symlinks, unicode, injection, TOCTOU)"
        - "Add 2 performance benchmarks (abstraction overhead, parallel speedup)"
        - "All tests fail initially (no implementation)"
        - "Property tests defined (4 tests × 10K cases)"
    - phase: "GREEN"
      tasks:
        - "Implement walk() - basic recursive traversal"
        - "Implement walk_parallel() - rayon parallel processing (with memory defect documentation)"
        - "Implement walk_with_options() - advanced configuration"
        - "Implement glob() and find() utilities"
        - "Implement search() - fast text search with grep crate"
        - "All 70/70 unit tests passing"
        - "All 3 concurrency tests passing (loom + thread sanitizer clean)"
        - "All 5 security tests passing (attacks blocked)"
    - phase: "REFACTOR"
      tasks:
        - "Verify complexity ≤10 for all functions"
        - "Run mutation tests (target ≥90%)"
        - "Performance benchmarks: abstraction overhead <1µs, parallel speedup ≥2x"
        - "Security audit: penetration testing against documented attack vectors"
        - "Code review with algorithm justification (theoretical foundations)"
  use_cases:
    - "ETL pipelines: Process thousands of CSV files in parallel"
    - "Log analysis: Search errors across directory trees"
    - "Data science: Build training datasets from image directories"
    - "Code analysis: Count lines of code, find patterns"
    - "Security audits: Find sensitive data patterns in codebases"
  impact: "Enables high-performance data processing + text search for data engineering and sysadmin workflows"
```

---

## Success Criteria (Updated per Critical Review)

### Functional
- ✅ All 70 unit tests passing (100%) (INCREASED from 60)
- ✅ Property tests validate 40K random scenarios
- ✅ Real-world examples work (ETL, log analysis, dataset building, security audit)
- ✅ Parallel processing utilizes all CPU cores
- ✅ Text search matches ripgrep performance
- ✅ All 5 CLI tools functional and production-ready

### Concurrency (NEW)
- ✅ All 3 concurrency tests passing:
  - Loom: Systematic exploration of thread interleavings (no data races)
  - Thread Sanitizer: Dynamic race detection (zero violations)
  - Stress test: 100 iterations with high contention (no panics/deadlocks)

### Security (NEW)
- ✅ All 5 security tests passing:
  - Directory traversal: `../` escapes blocked/sanitized
  - Symlink bombs: Circular symlinks detected and terminated
  - Unicode attacks: Homograph attacks prevented via normalization
  - Path injection: Malicious filenames don't execute commands
  - TOCTOU races: Filesystem changes handled gracefully

### Quality
- ✅ Mutation coverage ≥90% (INCREASED from 80% per critical review)
- ✅ Complexity ≤10 for all functions
- ✅ Zero SATD (no TODO/FIXME/HACK)
- ✅ TDG grade A- minimum

### Performance (Updated)
- ✅ Parallel ≥2x faster than serial (4+ core systems)
- ✅ Abstraction overhead <1µs per item (NEW benchmark - was "zero-cost" claim)
- ✅ Memory efficient (iterator-based API planned for v2.0)
- ✅ Theoretical foundation validated (work-stealing, Aho-Corasick, Thompson NFA)

### Usability
- ✅ Simpler than Python's os.walk (fewer lines of code)
- ✅ More powerful than Python (built-in parallelism + text search)
- ✅ CLI tools require zero coding (like Python's -m modules)
- ✅ Examples in documentation (10+ real-world scenarios)
- ✅ Text search simpler than ripgrep (sensible defaults)

---

## Future Enhancements (Not in v1.0)

1. **Async I/O Support** (v2.0)
   - `walk_async()` for I/O-bound workloads
   - Tokio integration for network filesystems

2. **Progress Reporting** (v2.0)
   - Callback for progress updates
   - Estimated time remaining

3. **Caching** (v2.0)
   - Cache directory structure for repeated walks
   - Invalidation on filesystem changes

4. **Filter DSL** (v3.0)
   - SQL-like syntax: `walk("/data").where("size > 1MB AND ext = '.csv'")`

---

## References

- Rust `walkdir` crate: https://docs.rs/walkdir
- Rust `rayon` crate: https://docs.rs/rayon
- Python `os.walk`: https://docs.python.org/3/library/os.html#os.walk
- Ruchy stdlib quality gates: `docs/execution/roadmap.yaml`
