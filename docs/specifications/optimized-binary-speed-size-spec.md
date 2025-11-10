# Optimized Binary Speed & Size Specification

**Document ID**: PERF-002
**Version**: 1.0.0
**Status**: Proposed
**Author**: Ruchy Team
**Date**: 2025-11-10

---

## Executive Summary

This specification defines optimal Rust compilation profiles for `ruchy compile` based on empirical benchmarking data from the [compiled-rust-benchmarking](https://github.com/paiml/compiled-rust-benchmarking) project. The study demonstrates **5-51x performance improvements** and **91.7% binary size reductions** through systematic optimization.

**Key Finding**: LTO ("link-time optimization") provides BOTH speed AND size benefits, making it the optimal default for most workloads.

---

## Background

### Motivation

Ruchy transpiles to Rust, then compiles to native binaries. The compilation profile significantly impacts:
- **Execution speed**: 5-51x variance depending on profile
- **Binary size**: Up to 91.7% reduction possible
- **User experience**: Faster startup, smaller downloads, better CI/CD performance

### Research Data

From 150 benchmark jobs across 10 diverse workloads and 15 compilation profiles:

| Metric | Value | Configuration |
|--------|-------|---------------|
| **Maximum speedup** | 51.33x | lto-fat (vs baseline) |
| **Best average** | 15.06x | lto-fat across all workloads |
| **Smallest binary** | 314 KB | size-ultra (91.7% reduction) |
| **Balanced winner** | lto-fat | 15x speed + 53% size reduction |

**Statistical Validation**: ANOVA F=19.87, η²=0.986 (workload type explains 98.6% of variance)

---

## Goals

1. **Default to Fast**: Optimize for speed by default (aligns with [PERF-001])
2. **Size Option**: Provide explicit flag for size-constrained environments
3. **Transparency**: Clear documentation of tradeoffs
4. **Evidence-Based**: Use empirical data from benchmarking project
5. **Predictability**: Consistent behavior across workload types

---

## Design Principles

### 1. LTO is Essential

**Finding**: LTO provides dramatic benefits with minimal downside:
- **Speed**: 15x average improvement
- **Size**: 53% reduction vs baseline
- **Cost**: Longer compile time (acceptable for production builds)

**Decision**: Enable `lto = "fat"` by default for all `ruchy compile` profiles.

### 2. Workload-Specific Optimization

| Workload Type | Best Profile | Speedup | Use Case |
|---------------|--------------|---------|----------|
| Memory random access | lto-fat | 51.33x | Data processing, algorithms |
| CPU iterative | perf-ultra | 25.81x | Numerical computing |
| Memory cache-sensitive | opt-s | 22.64x | High-frequency operations |
| CPU recursive | opt-s | 4.32x | Tree traversal, parsers |
| I/O bound | size-z-native | 1.99x | File operations, network |

**Decision**: Default to lto-fat (best average), allow override via `--profile`.

### 3. Size vs Speed Tradeoff

**Pareto Frontier Analysis**:
- **Pure speed**: codegen-1 (2.68x, 3.7 MB)
- **Balanced**: lto-fat (2.25x, 1.7 MB) ⭐ **DEFAULT**
- **Pure size**: size-ultra (2.16x, 314 KB)

**Key Insight**: The "balanced" option (lto-fat) is only 2% slower than maximum speed but 54% smaller, making it optimal for most users.

---

## Implementation

### Default Profile: `--profile release` (Speed-First)

**Current Cargo.toml** (already optimal as of v3.174.0):

```toml
[profile.release]
opt-level = 3              # Maximum speed
lto = "fat"                # Full link-time optimization ⭐
codegen-units = 1          # Single codegen unit (best optimization)
strip = true               # Remove debug symbols
panic = "abort"            # No unwinding overhead
overflow-checks = false    # No runtime overflow checks
debug-assertions = false   # No debug assertions
incremental = false        # Disable incremental compilation
```

**Characteristics**:
- ⚡ **15x average speedup** vs baseline
- 📦 **~1.7 MB binary** (53% smaller than baseline)
- 🎯 **Best for**: General-purpose production binaries

**Usage**:
```bash
# Implicit (default)
ruchy compile script.ruchy -o myapp

# Explicit
ruchy compile script.ruchy -o myapp --profile release
```

---

### Size-Optimized Profile: `--profile release-tiny`

**Cargo.toml Configuration**:

```toml
[profile.release-tiny]
inherits = "release"
opt-level = "z"            # Optimize for SIZE ⭐
lto = "fat"                # Still use LTO (size + speed)
codegen-units = 1          # Single unit
panic = "abort"            # Minimal panic handler
strip = true               # Remove symbols
```

**Characteristics**:
- 📦 **314 KB binary** (91.7% smaller than baseline)
- ⚡ **2.16x speedup** (only 14% slower than speed-optimized)
- 🎯 **Best for**: Embedded systems, mobile apps, size-constrained deployments

**Usage**:
```bash
ruchy compile script.ruchy -o myapp --profile release-tiny
```

---

### Ultra-Performance Profile: `--profile release-ultra` (with PGO)

**Cargo.toml Configuration** (already defined):

```toml
[profile.release-ultra]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1
```

**Additional Compilation Flags**:
```bash
RUSTFLAGS="-C target-cpu=native -C embed-bitcode=yes"
```

**Two-Step PGO Build**:

```bash
# Step 1: Build with profile generation
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" \
  ruchy compile script.ruchy -o myapp-profiled --profile release-ultra

# Step 2: Run workload to collect profile data
./myapp-profiled <typical-workload>

# Step 3: Build with profile-guided optimization
RUSTFLAGS="-C profile-use=/tmp/pgo-data -C target-cpu=native" \
  ruchy compile script.ruchy -o myapp --profile release-ultra
```

**Characteristics**:
- ⚡ **25-50x speedup** for CPU-intensive workloads
- 📦 **~520 KB binary** (moderate size)
- 🎯 **Best for**: Performance-critical production systems, long-running services

**Tradeoffs**:
- ⏱️ Requires two-step build process
- 💾 Profile data collection overhead
- 🖥️ Hardware-specific (not portable across CPUs)

---

### Distribution Profile: `--profile release-dist`

**Current Configuration** (needs update):

```toml
[profile.release-dist]
inherits = "release"
strip = true
lto = "fat"
codegen-units = 1
panic = "abort"
opt-level = "z"       # ❌ PROBLEM: Size-optimized, not speed
```

**Recommended Update** (align with findings):

```toml
[profile.release-dist]
inherits = "release"
opt-level = 3         # ✅ FIX: Use speed optimization
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
overflow-checks = false
debug-assertions = false
incremental = false
```

**Rationale**: Distribution binaries should prioritize performance (15x speedup) over minimal size savings. Users needing tiny binaries can use `release-tiny` explicitly.

---

## CLI Integration

### Updated `ruchy compile` Command

```
USAGE:
    ruchy compile [OPTIONS] <INPUT> -o <OUTPUT>

OPTIONS:
    -o, --output <PATH>           Output binary path
    --profile <PROFILE>           Build profile [default: release]
                                  Options: release, release-tiny, release-ultra, release-dist
    --target-cpu <CPU>            Target CPU (e.g., native, x86-64-v3) [default: generic]
    --pgo                         Enable profile-guided optimization (two-step build)
    --show-profile-info           Print profile characteristics and exit
    --rustflags <FLAGS>           Additional RUSTFLAGS to pass to rustc

EXAMPLES:
    # Default: Fast binary (~1.7 MB, 15x speedup)
    ruchy compile main.ruchy -o myapp

    # Tiny binary for embedded (314 KB, 2x speedup)
    ruchy compile main.ruchy -o myapp --profile release-tiny

    # Maximum performance with native CPU optimization
    ruchy compile main.ruchy -o myapp --profile release-ultra --target-cpu native

    # Profile-guided optimization (two-step)
    ruchy compile main.ruchy -o myapp --profile release-ultra --pgo
    # (automatically handles PGO workflow)

PROFILE INFO:
    release         15x speed, 1.7 MB  [DEFAULT]
    release-tiny    2x speed, 314 KB   [EMBEDDED]
    release-ultra   25-50x speed       [MAXIMUM PERFORMANCE]
    release-dist    Same as release    [DISTRIBUTION]
```

---

## Benchmark Data Reference

### Speed Rankings (Top 5 Profiles)

| Rank | Profile | Average Speedup | 95% CI | Binary Size |
|------|---------|-----------------|--------|-------------|
| 1 | lto-fat | 15.06x | [13.1, 17.1] | 1.76 MB |
| 2 | lto-thin | 14.52x | [12.6, 16.5] | 1.85 MB |
| 3 | codegen-1 | 14.12x | [12.2, 16.1] | 3.78 MB |
| 4 | standard-release | 13.87x | [12.0, 15.8] | 2.12 MB |
| 5 | perf-ultra | 13.64x | [11.8, 15.5] | 2.34 MB |

### Size Rankings (Top 5 Profiles)

| Rank | Profile | Binary Size | Speedup | Size Reduction |
|------|---------|-------------|---------|----------------|
| 1 | size-ultra | 314 KB | 2.16x | 91.7% |
| 2 | size-z | 512 KB | 2.24x | 86.5% |
| 3 | opt-s | 698 KB | 2.43x | 81.5% |
| 4 | lto-fat | 1.76 MB | 15.06x | 53.4% ⭐ |
| 5 | lto-thin | 1.85 MB | 14.52x | 51.0% |

**Key Insight**: lto-fat is 4th best for size but 1st for speed, making it the optimal balanced choice.

---

## Workload-Specific Recommendations

### Data Science / Scientific Computing

**Characteristics**: Memory-intensive, large datasets, numerical operations

**Recommendation**: `--profile release` (default)
- **Speedup**: 15-51x for memory operations
- **Rationale**: Data processing benefits dramatically from LTO optimizations

```bash
ruchy compile analyze.ruchy -o analyze --profile release
```

### Web Servers / APIs

**Characteristics**: I/O bound, concurrent, long-running

**Recommendation**: `--profile release-ultra --target-cpu native`
- **Speedup**: 10-15x average, better latency
- **Rationale**: Performance-critical, runs on known hardware

```bash
ruchy compile server.ruchy -o server --profile release-ultra --target-cpu native
```

### CLI Tools

**Characteristics**: Fast startup, moderate workload, wide distribution

**Recommendation**: `--profile release` (default)
- **Binary Size**: 1-2 MB (reasonable for download)
- **Performance**: 15x speedup
- **Portability**: Works on all x86-64 CPUs

```bash
ruchy compile tool.ruchy -o tool
```

### Embedded / Edge Devices

**Characteristics**: Size-constrained, ARM processors, limited storage

**Recommendation**: `--profile release-tiny`
- **Binary Size**: 300-500 KB
- **Performance**: Still 2x faster than baseline
- **Deployment**: Fits in constrained environments

```bash
ruchy compile embedded.ruchy -o embedded --profile release-tiny
```

### Serverless / Lambda Functions

**Characteristics**: Cold start sensitive, ephemeral, network-bound

**Recommendation**: `--profile release-tiny`
- **Rationale**: Smaller binaries load faster (cold start optimization)
- **Tradeoff**: Slight performance loss acceptable for I/O-bound workloads

```bash
ruchy compile lambda.ruchy -o bootstrap --profile release-tiny
```

---

## Migration Strategy

### Phase 1: Update Documentation (Immediate)

- [ ] Update README with profile recommendations
- [ ] Add `--profile` examples to CLI documentation
- [ ] Create migration guide for users on old `release-dist`

### Phase 2: Fix `release-dist` Profile (v3.212.0)

**Current Issue**: `release-dist` uses `opt-level = "z"` (size), inconsistent with research findings.

**Fix**:
```toml
[profile.release-dist]
inherits = "release"
opt-level = 3         # Change from "z" to 3
# ... keep other settings
```

**Impact**: Distribution binaries will be ~15x faster (currently only ~2x due to size optimization)

**Breaking Change**: No (profile name stays same, just gets faster)

### Phase 3: Add `--show-profile-info` Flag (v3.213.0)

**Feature**: Print profile characteristics before compilation

```bash
$ ruchy compile main.ruchy -o main --show-profile-info

Profile: release (default)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Optimization:     opt-level = 3 (speed)
  LTO:              fat (maximum)
  Codegen units:    1
  Expected speedup: 15x average
  Expected size:    1-2 MB
  Best for:         General-purpose production binaries
  Compile time:     ~30-60s for 1000 LOC
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Alternative profiles:
  --profile release-tiny    (314 KB, 2x speed, embedded)
  --profile release-ultra   (25-50x speed, PGO, maximum performance)

Continue? [Y/n]
```

### Phase 4: Implement `--pgo` Automation (v3.214.0)

**Feature**: Automate two-step PGO build process

```bash
# Single command replaces 3-step manual process
ruchy compile server.ruchy -o server --profile release-ultra --pgo

# Automatically:
# 1. Builds with profile-generate
# 2. Prompts user to run workload
# 3. Rebuilds with profile-use
```

**UX Flow**:
```
Building with profile generation...
✓ Built: server-profiled

Run your typical workload now to collect profile data:
  ./server-profiled <args>

Press Enter when done...

Building with profile-guided optimization...
✓ Built: server (optimized)

Profile data: /tmp/ruchy-pgo-xxxxx (can be reused)
```

---

## Testing Strategy

### Regression Tests

1. **Profile Parity**: Verify all profiles produce working binaries
2. **Size Bounds**: Ensure `release-tiny` produces <500 KB binaries
3. **Performance Bounds**: Verify `release` achieves >10x speedup on standard benchmarks
4. **PGO Sanity**: Confirm PGO builds are faster than non-PGO

### Integration Tests

```rust
#[test]
fn test_profile_release_default() {
    let output = compile_with_profile("test.ruchy", Profile::Release);
    assert!(output.size_kb() < 2000);  // < 2 MB
    assert!(output.runtime_ms() < baseline.runtime_ms() / 10);  // >10x
}

#[test]
fn test_profile_tiny_size() {
    let output = compile_with_profile("test.ruchy", Profile::ReleaseTiny);
    assert!(output.size_kb() < 500);  // < 500 KB
}
```

### Benchmark Suite

Run full pathfinder study on each Ruchy release:

```bash
# Generate benchmark workload in Ruchy
ruchy benchmark gen-workload --output bench.ruchy

# Compile with each profile
for profile in release release-tiny release-ultra; do
  ruchy compile bench.ruchy -o bench-$profile --profile $profile
  time ./bench-$profile
  ls -lh bench-$profile
done
```

**Acceptance Criteria**:
- `release`: >10x speedup, <2 MB
- `release-tiny`: >2x speedup, <500 KB
- `release-ultra`: >15x speedup (with PGO)

---

## Documentation Updates

### README.md

```markdown
## Compilation Profiles

Ruchy uses research-backed compilation profiles for optimal binaries:

- **Default** (`ruchy compile`): 15x faster, 1-2 MB (best for most use cases)
- **Tiny** (`--profile release-tiny`): 2x faster, 300 KB (embedded systems)
- **Ultra** (`--profile release-ultra`): 25-50x faster (maximum performance)

Based on empirical data: [compiled-rust-benchmarking](https://github.com/paiml/compiled-rust-benchmarking)
```

### CLI Help Text

```
ABOUT PROFILES:
  Research shows LTO provides dramatic benefits (15x speed + 53% size reduction).
  All profiles use LTO by default. See our benchmarking study:
  https://github.com/paiml/compiled-rust-benchmarking
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Default speedup | >10x vs baseline | Benchmark suite |
| Tiny binary size | <500 KB | File size |
| User adoption | >80% use default | Telemetry (opt-in) |
| Compile time | <2min for 5K LOC | CI/CD metrics |
| Documentation clarity | >90% understand profiles | User survey |

---

## Future Work

### Advanced Optimization Features

1. **Auto-Profile Selection**: Analyze workload and recommend optimal profile
2. **Cross-Language PGO**: Collect profiles from Ruchy execution, apply to Rust
3. **Lazy Compilation**: JIT-compile hot paths discovered at runtime
4. **Binary Patching**: Update optimizations without full recompilation

### Research Extensions

1. **Workload Detection**: ML model to classify Ruchy program workload type
2. **Adaptive Optimization**: Runtime feedback to compiler for iterative improvement
3. **Cloud Profiling**: Collect PGO data from production deployments
4. **Size Budget**: Automatic profile selection to meet size constraints

---

## References

### Primary Research

- [Compiled Rust Benchmarking Infrastructure](https://github.com/paiml/compiled-rust-benchmarking)
  - 580 measurements across 10 workloads
  - 15 compilation profiles tested
  - Statistical validation (ANOVA, confidence intervals)

### Rust Documentation

- [Profile Settings](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Link-Time Optimization](https://doc.rust-lang.org/rustc/linker-plugin-lto.html)
- [Profile-Guided Optimization](https://doc.rust-lang.org/rustc/profile-guided-optimization.html)

### Related Specifications

- [PERF-001] - Speed-first optimization (v3.174.0)
- [Binary Build Story](binary-build-story.md) - Distribution strategy
- [Cargo Integration](cargo-integration-ruchy.md) - Rust toolchain integration

---

## Approval

- [ ] Engineering Lead Review
- [ ] Performance Team Approval
- [ ] Documentation Team Approval
- [ ] User Experience Review
- [ ] Security Audit (PGO data handling)

**Estimated Implementation**: 2 sprints (v3.212.0 - v3.214.0)

---

**Document Status**: Ready for Review
**Next Steps**: Team review, approve Phase 1 (documentation updates)
