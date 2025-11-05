# Feature Request: Production Profiling & Benchmarking Tools for Ruchy

**GitHub Issue**: https://github.com/paiml/ruchy/issues/138
**Date**: 2025-11-05
**Project**: ruchy
**Priority**: High - Blocks production optimization workflows
**Related**: ruchy-lambda performance optimization

## Summary

The Ruchy toolchain currently has excellent **interpreter profiling tools** (`ruchydbg profile`) but lacks **production binary profiling tools** for optimizing transpiled Rust code. This blocks real-world performance analysis, especially for AWS Lambda cold start optimization.

**Current gap**: Can profile interpreter (16.31µs overhead), but can't profile compiled binary (23.93ms execution).

---

## Motivation: Real-World Use Case

**Project**: ruchy-lambda - AWS Lambda custom runtime
- **Goal**: <8ms cold start (currently 8.50ms)
- **Binary**: 400KB (smaller than Rust's 596KB)
- **Performance**: Matches native Rust (23.93ms fibonacci(35))

**Questions we can't answer without profiling tools**:
1. Why is our 400KB binary faster than 596KB Rust (tokio)?
2. What parts of the transpiled code are hotspots?
3. Can we optimize below 8ms cold start?
4. How does memory allocation compare to hand-written Rust?

---

## Current State Analysis

### What Works ✅

**Ruchy toolchain**:
- `ruchy bench` - Compilation time benchmarking (0.095ms)
- `ruchydbg profile --perf` - Interpreter overhead (parse: 14.36µs, eval: 1.95µs)
- `ruchydbg profile --stack` - Call depth analysis (works for small inputs)
- `ruchydbg profile --types` - Type stability tracking

### What's Missing ❌

**Production profiling**:
- ❌ Binary execution profiler (profile transpiled Rust code)
- ❌ Flamegraph generation (visualize hotspots)
- ❌ Memory profiler (heap allocations, leaks)
- ❌ CPU profiler (instruction-level perf)
- ❌ Performance regression tracking
- ❌ PGO (Profile-Guided Optimization) workflow
- ❌ Binary size analyzer (what contributes to binary size)
- ❌ Lambda/serverless-specific profiling

**Current workaround**: Use Rust tools (`cargo-flamegraph`, `perf`, `valgrind`) on transpiled output, but this loses Ruchy source mapping.

---

## Proposed Tools

### 1. `ruchy profile --binary` (HIGH PRIORITY)

Profile **transpiled Rust code execution** with Ruchy source mapping.

**Usage**:
```bash
# Profile compiled binary
ruchy profile --binary fibonacci.ruchy --iterations 1000

# Generate flamegraph
ruchy profile --binary fibonacci.ruchy --format flamegraph --output flame.svg

# Profile with source mapping
ruchy profile --binary fibonacci.ruchy --source-map --verbose

# Profile specific function
ruchy profile --binary fibonacci.ruchy --function fibonacci --trace-calls
```

**Output**:
```
=== Binary Execution Profile ===
File: fibonacci.ruchy
Iterations: 1000

Function-level timings:
  fibonacci()    23.85ms  (99.7%)  [59,000,000 calls]
  main()          0.08ms  ( 0.3%)  [1 call]

Hotspots (>10% total time):
  fibonacci.ruchy:2-6    23.70ms  (99.4%)    [recursive calls]

Memory:
  Allocations:   0 bytes
  Peak RSS:      1.2 MB

Recommendations:
  ✓ No allocations detected (optimal)
  ✓ Stack-only execution (optimal for recursion)
  ⚠️ Consider memoization for n>20
```

**Implementation approach**:
- Integrate with Rust `perf`/`flamegraph` crates
- Maintain source maps from `.ruchy` → `.rs`
- Parse Rust profiling output and map back to Ruchy source
- Support multiple output formats (text, JSON, flamegraph)

---

### 2. `ruchy analyze` (HIGH PRIORITY)

Analyze transpiled binaries for optimization opportunities.

**Usage**:
```bash
# Binary size breakdown
ruchy analyze --binary-size bootstrap.ruchy --output size-report.txt

# Cold start estimation (Lambda-specific)
ruchy analyze --lambda-ready bootstrap.ruchy

# Memory layout analysis
ruchy analyze --memory fibonacci.ruchy

# Optimization suggestions
ruchy analyze --suggest fibonacci.ruchy
```

**Output (binary size)**:
```
=== Binary Size Analysis ===
Binary: bootstrap (400 KB)

Size breakdown:
  Rust stdlib:     250 KB  (62.5%)
  Ruchy runtime:    50 KB  (12.5%)
  User code:        30 KB  ( 7.5%)
  Dependencies:     40 KB  (10.0%)
  Debug info:        0 KB  ( 0.0%)  [stripped]
  Padding/align:    30 KB  ( 7.5%)

Comparison:
  vs Rust (tokio): -196 KB  (-32.9% smaller)
  vs Go:           -3800 KB (-90.5% smaller)
  vs C++:          +313 KB  (+359% larger)

Recommendations:
  ✓ Binary size optimal for Lambda
  ✓ No debug symbols (stripped correctly)
  ⚠️ Consider LTO=fat for additional 10-15% reduction
```

**Output (Lambda-ready)**:
```
=== Lambda Readiness Analysis ===
Binary: bootstrap.ruchy → bootstrap (400 KB)

Cold start estimation:
  Binary load:      3.0ms  (400 KB @ 133 MB/s)
  Init code:        5.5ms  (Runtime::new + setup)
  Total estimate:   8.5ms  ✅ EXCELLENT (<10ms target)

Memory estimate:
  Base RSS:        12 MB
  Peak usage:      14 MB  (under 128 MB Lambda minimum)

Performance:
  Fibonacci(35):   23.93ms  (matches native Rust)
  HTTP GET:        2-3ms    (TcpStream overhead)

Optimizations applied:
  ✅ LTO enabled
  ✅ Symbols stripped
  ✅ opt-level=3
  ✅ codegen-units=1

Recommendations:
  ✓ Ready for Lambda deployment
  ✓ Cold start within target (<10ms)
  ⚠️ Consider PGO for 5-10% improvement
```

---

### 3. `ruchy bench --regression` (MEDIUM PRIORITY)

Track performance regressions across Ruchy versions.

**Usage**:
```bash
# Set baseline
ruchy bench --regression baseline fibonacci.ruchy --version v3.201.0

# Compare against baseline
ruchy bench --regression compare fibonacci.ruchy --baseline v3.201.0

# Continuous benchmarking
ruchy bench --regression ci fibonacci.ruchy --threshold 5%  # Fail if >5% slower
```

**Output**:
```
=== Performance Regression Check ===
Baseline: v3.201.0 (23.93ms)
Current:  v3.202.0 (24.15ms)

Change: +0.22ms (+0.9%)  ✅ PASS (< 5% threshold)

Breakdown:
  Compilation:  0.095ms → 0.097ms  (+2.1%)
  Execution:   23.835ms → 24.053ms  (+0.9%)

Status: ✅ No significant regression detected
```

---

### 4. `ruchy pgo` (MEDIUM PRIORITY)

Profile-Guided Optimization workflow.

**Usage**:
```bash
# Step 1: Collect profiling data
ruchy pgo collect fibonacci.ruchy --runs 10000 --output profile.data

# Step 2: Transpile with PGO hints
ruchy transpile fibonacci.ruchy --pgo-data profile.data --output optimized.rs

# Step 3: Verify improvement
ruchy bench fibonacci.ruchy --with-pgo --compare-baseline
```

**Expected improvement**: 5-10% for hot-path optimization.

---

### 5. `ruchy profile --compare` (LOW PRIORITY)

Side-by-side comparison with other languages.

**Usage**:
```bash
# Compare Ruchy vs Rust vs Python
ruchy profile --compare fibonacci.ruchy fibonacci.rs fibonacci.py

# Compare against baseline implementations
ruchy profile --compare fibonacci.ruchy --baseline-langs rust,python,go
```

**Output**:
```
=== Multi-Language Comparison ===
Test: fibonacci(35) - 59M recursive calls

Performance:
  C (gcc -O3):       12.73ms   54.1x vs Python  [baseline]
  Rust (rustc -O3):  23.86ms   28.9x vs Python
  Ruchy (transpiled) 23.93ms   28.8x vs Python  ✅ MATCHES RUST
  Go:                37.59ms   18.3x vs Python
  Python:           688.89ms   1.0x  vs Python

Binary size:
  Ruchy:   400 KB  ✅
  Rust:    596 KB
  Go:     4200 KB

Cold start (estimated):
  Ruchy:    8.5ms  ✅
  Rust:    14.9ms
  Go:      56.5ms
  Python:  85.7ms
```

---

## Implementation Phases

### Phase 1: Binary Profiling (Weeks 1-2)
- [ ] `ruchy profile --binary` - Basic execution profiling
- [ ] Integration with Rust `perf` crate
- [ ] Source map preservation (.ruchy → .rs mapping)
- [ ] Text output format

### Phase 2: Analysis Tools (Weeks 3-4)
- [ ] `ruchy analyze --binary-size` - Size breakdown
- [ ] `ruchy analyze --lambda-ready` - Serverless analysis
- [ ] JSON output format for CI/CD integration

### Phase 3: Regression Tracking (Weeks 5-6)
- [ ] `ruchy bench --regression baseline` - Baseline creation
- [ ] `ruchy bench --regression compare` - Regression detection
- [ ] CI/CD integration examples

### Phase 4: Advanced Features (Weeks 7-8)
- [ ] `ruchy profile --binary --format flamegraph` - Flamegraph generation
- [ ] `ruchy pgo` - Profile-guided optimization workflow
- [ ] Memory profiling integration

---

## Success Metrics

**For ruchy-lambda project**:
- [ ] Can profile 23.93ms execution and identify hotspots
- [ ] Can explain why 400KB binary is faster than 596KB Rust
- [ ] Can optimize below 8ms cold start with data-driven insights
- [ ] Can track performance regressions in CI/CD

**For Ruchy ecosystem**:
- [ ] Users can profile production binaries, not just interpreter
- [ ] Performance optimization becomes data-driven
- [ ] Ruchy can compete with Rust on observable performance metrics
- [ ] Profiling tools generate actionable recommendations

---

## Related Work

**Existing tools we'd integrate with**:
- Rust `perf` - Linux performance counters
- `cargo-flamegraph` - SVG flamegraph generation
- `valgrind` - Memory profiling
- AWS CloudWatch - Lambda metrics

**Similar features in other languages**:
- Go: `go tool pprof` (CPU/memory profiling)
- Rust: `cargo flamegraph`, `cargo-profiler`
- Python: `cProfile`, `py-spy`
- Julia: `@profile`, `ProfileView.jl`

---

## Questions for Maintainers

1. **Source mapping**: How to preserve `.ruchy` → `.rs` source maps through transpilation?
2. **Profiling backend**: Use Rust `perf` crate, or shell out to `perf`/`flamegraph`?
3. **Output formats**: JSON for CI/CD, text for humans, SVG for flamegraphs?
4. **Integration point**: Add to `ruchy` CLI or separate `ruchy-profile` tool?
5. **Lambda-specific**: Should serverless analysis be in core or plugin?

---

## Alternative: Interim Solutions

While waiting for native Ruchy tools, users can:

1. **Profile transpiled Rust**:
   ```bash
   ruchy transpile fibonacci.ruchy -o fibonacci.rs
   rustc -C opt-level=3 fibonacci.rs -o fibonacci
   perf record -F 99 -g ./fibonacci
   perf script | flamegraph.pl > flame.svg
   ```

2. **Use Rust profiling tools**:
   ```bash
   cargo build --release
   cargo flamegraph --bin bootstrap
   ```

3. **Manual CloudWatch analysis**:
   - Deploy to Lambda
   - Parse CloudWatch logs for "Init Duration"
   - Compare across versions

**Problem**: All of these lose Ruchy source context and require Rust expertise.

---

## Priority Justification

**HIGH**: `ruchy profile --binary`, `ruchy analyze`
- Blocks production optimization
- ruchy-lambda achieving <8ms cold start depends on this
- No current workaround with Ruchy source mapping

**MEDIUM**: `ruchy bench --regression`, `ruchy pgo`
- Nice to have for CI/CD and advanced optimization
- Can use manual benchmarking in interim

**LOW**: `ruchy profile --compare`
- Convenience feature
- Can manually compare with other tools

---

## Files for Reference

All code in `ruchy-lambda` repository:
- `benchmarks/local-fibonacci/` - Benchmark harness (C, Rust, Go, Python, Ruchy, Julia)
- `crates/runtime-pure/` - Pure Ruchy runtime (transpiled)
- `README.md` - Performance results (8.50ms cold start, 400KB binary)

**Proof of need**: We have production Lambda deployment with 8.50ms cold start but can't profile it at the Ruchy source level.

---

**Contact**: Noah (ruchy-lambda project maintainer)
**Related Issues**: #137 (Parser limitations)
