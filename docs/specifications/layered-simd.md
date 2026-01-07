# Layered SIMD Architecture Specification

**Version**: 1.0.0
**Status**: Draft
**Author**: Ruchy Core Team
**Date**: 2025-01-07

## 1. Executive Summary

This specification defines a layered SIMD architecture for Ruchy that separates concerns between pure SIMD numerics (Trueno) and DataFrame semantics (Polars), following Toyota Production System principles of eliminating waste (Muda) and continuous improvement (Kaizen).

## 2. Problem Statement

Current state:
- Polars dependency adds 60-90s to clean builds
- 15-30MB binary size overhead
- Transitive dependencies increase test surface
- Most Ruchy programs don't need DataFrame semantics

## 3. Architectural Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│              (Ruchy user code / examples)                │
├─────────────────────────────────────────────────────────┤
│                   Abstraction Layer                      │
│         (ruchy::simd - unified SIMD interface)          │
├──────────────────────┬──────────────────────────────────┤
│   Layer 2: DataFrame │   Layer 1: Pure SIMD Numerics    │
│   (Optional Feature) │   (Default, Always Available)    │
│                      │                                   │
│   ┌────────────────┐ │   ┌────────────────────────────┐ │
│   │     Polars     │ │   │         Trueno             │ │
│   │   (polars.rs)  │ │   │   - Vector operations      │ │
│   │                │ │   │   - Matrix operations      │ │
│   │   - DataFrame  │ │   │   - Eigenvalue decomp      │ │
│   │   - Series     │ │   │   - portable_simd          │ │
│   │   - GroupBy    │ │   │   - AVX2/AVX512/NEON       │ │
│   │   - Joins      │ │   │                            │ │
│   └────────────────┘ │   └────────────────────────────┘ │
├──────────────────────┴──────────────────────────────────┤
│                    Hardware Layer                        │
│        (CPU SIMD: SSE4.2, AVX2, AVX512, NEON)           │
└─────────────────────────────────────────────────────────┘
```

## 4. Design Principles (Toyota Way)

### 4.1 Jidoka (Automation with Human Touch)
- Automatic SIMD width detection at compile time
- Runtime fallback for unsupported instructions
- Clear error messages when features unavailable

### 4.2 Just-in-Time (JIT) Compilation
- Only compile what's needed via feature flags
- Default: minimal dependency footprint
- Optional: rich DataFrame semantics

### 4.3 Heijunka (Leveling)
- Consistent API across SIMD backends
- Smooth migration path between layers
- Predictable performance characteristics

### 4.4 Genchi Genbutsu (Go and See)
- Benchmarks prove performance claims
- Real workloads validate architecture
- Continuous measurement via CI

## 5. Peer-Reviewed Citations

### 5.1 SIMD Vectorization

[1] Larsen, S., & Amarasinghe, S. (2000). **Exploiting superword level parallelism with multimedia instruction sets.** ACM SIGPLAN Notices, 35(5), 145-156. https://doi.org/10.1145/358438.349320

*Foundational work on automatic SIMD vectorization. Establishes theoretical basis for portable_simd approach.*

[2] Fog, A. (2023). **Optimizing software in C++: An optimization guide for Windows, Linux, and Mac platforms.** Technical University of Denmark. https://www.agner.org/optimize/

*Industry-standard reference for SIMD optimization. Validates AVX2/AVX512 performance characteristics.*

### 5.2 DataFrame Systems

[3] Abadi, D., et al. (2013). **The design and implementation of modern column-oriented database systems.** Foundations and Trends in Databases, 5(3), 197-280. https://doi.org/10.1561/1900000024

*Columnar storage foundations. Justifies Arrow/Polars memory layout for cache efficiency.*

[4] Kersten, T., et al. (2018). **Everything you always wanted to know about compiled and vectorized queries but were afraid to ask.** VLDB Endowment, 11(13), 2209-2222. https://doi.org/10.14778/3275366.3284966

*Compiled vs interpreted query execution. Supports Ruchy's transpilation approach.*

### 5.3 Memory Hierarchy

[5] Drepper, U. (2007). **What every programmer should know about memory.** Red Hat, Inc. https://people.freebsd.org/~lstewart/articles/cpumemory.pdf

*Cache-aware programming. Validates Trueno's tiled matrix operations.*

[6] Williams, S., Waterman, A., & Patterson, D. (2009). **Roofline: An insightful visual performance model for multicore architectures.** Communications of the ACM, 52(4), 65-76. https://doi.org/10.1145/1498765.1498785

*Roofline model for compute vs memory bound analysis. Guides SIMD optimization strategy.*

### 5.4 Toyota Production System

[7] Liker, J. K. (2004). **The Toyota Way: 14 Management Principles from the World's Greatest Manufacturer.** McGraw-Hill. ISBN: 978-0071392310

*Core TPS principles applied to software architecture.*

[8] Ohno, T. (1988). **Toyota Production System: Beyond Large-Scale Production.** Productivity Press. ISBN: 978-0915299140

*Original TPS source. Muda (waste) elimination drives feature flag design.*

### 5.5 Numerical Computing

[9] Goto, K., & Van De Geijn, R. A. (2008). **Anatomy of high-performance matrix multiplication.** ACM Transactions on Mathematical Software, 34(3), 1-25. https://doi.org/10.1145/1356052.1356053

*GotoBLAS design. Influences Trueno matrix multiplication implementation.*

[10] Anderson, E., et al. (1999). **LAPACK Users' Guide (3rd ed.).** SIAM. ISBN: 978-0898714470

*Linear algebra standards. Trueno eigenvalue decomposition follows LAPACK conventions.*

## 6. Feature Flag Configuration

```toml
[features]
default = ["simd-core"]

# Layer 1: Pure SIMD (always fast)
simd-core = ["trueno"]

# Layer 2: DataFrame (optional, heavy)
dataframe = ["polars", "arrow", "arrow-array", "arrow-buffer", "arrow-schema"]

# Convenience aliases
full = ["simd-core", "dataframe"]
minimal = []  # No SIMD, pure scalar
```

## 7. API Design

### 7.1 Unified SIMD Trait

```rust
/// Core SIMD operations trait (Layer 1)
pub trait SimdOps<T> {
    fn sum(&self) -> T;
    fn dot(&self, other: &Self) -> T;
    fn scale(&mut self, factor: T);
    fn add(&self, other: &Self) -> Self;
    fn mul(&self, other: &Self) -> Self;
}

/// Matrix operations (Layer 1)
pub trait MatrixOps<T> {
    fn matmul(&self, other: &Self) -> Self;
    fn transpose(&self) -> Self;
    fn eigenvalues(&self) -> Vec<T>;
}

/// DataFrame operations (Layer 2, optional)
#[cfg(feature = "dataframe")]
pub trait DataFrameOps {
    fn select(&self, columns: &[&str]) -> Self;
    fn filter(&self, predicate: Expr) -> Self;
    fn groupby(&self, columns: &[&str]) -> GroupBy;
    fn join(&self, other: &Self, on: &str) -> Self;
}
```

### 7.2 Compile-Time Backend Selection

```rust
// Automatic SIMD width detection
#[cfg(target_feature = "avx512f")]
type SimdWidth = Simd<f32, 16>;

#[cfg(all(target_feature = "avx2", not(target_feature = "avx512f")))]
type SimdWidth = Simd<f32, 8>;

#[cfg(all(target_feature = "neon", not(target_feature = "avx2")))]
type SimdWidth = Simd<f32, 4>;

#[cfg(not(any(target_feature = "avx2", target_feature = "neon")))]
type SimdWidth = Simd<f32, 4>;  // SSE fallback
```

## 8. Performance Targets

| Operation | Scalar | SIMD (AVX2) | SIMD (AVX512) | Target |
|-----------|--------|-------------|---------------|--------|
| Vector sum (1M f32) | 1.0ms | 0.15ms | 0.08ms | <0.1ms |
| Matrix mul (1024x1024) | 2.1s | 0.12s | 0.06s | <0.1s |
| Dot product (1M f32) | 2.0ms | 0.25ms | 0.12ms | <0.15ms |
| Eigenvalues (512x512) | 1.5s | 0.8s | 0.5s | <0.6s |

## 9. Build Time Targets

| Configuration | Clean Build | Incremental | Target |
|--------------|-------------|-------------|--------|
| `--no-default-features` | 15s | 2s | <20s |
| `--features simd-core` | 25s | 3s | <30s |
| `--features dataframe` | 90s | 10s | <120s |
| `--all-features` | 120s | 12s | <150s |

## 10. Migration Path

### Phase 1: Abstraction Layer (Week 1)
- Create `ruchy::simd` module
- Define unified traits
- Wrap existing Trueno calls

### Phase 2: Feature Flags (Week 2)
- Move Polars behind `dataframe` feature
- Update Cargo.toml dependencies
- Fix conditional compilation

### Phase 3: Default Change (Week 3)
- Change default to `simd-core` only
- Update documentation
- Release notes

### Phase 4: Optimization (Week 4)
- Benchmark all configurations
- Profile hot paths
- Tune SIMD implementations

## 11. Risk Analysis

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing users | Medium | High | Semver major bump, migration guide |
| Performance regression | Low | High | Continuous benchmarking in CI |
| API instability | Medium | Medium | Feature flag for experimental APIs |
| Trueno bugs | Low | Medium | Extensive property testing |

## 12. Success Criteria

1. Clean build <30s without DataFrame
2. Binary size <5MB without DataFrame
3. 100% API compatibility for existing code
4. Zero performance regressions
5. 95% test coverage maintained

## 13. References

See Section 5 for complete peer-reviewed citations.

## 14. Appendix: Toyota Way Alignment

| TPS Principle | Application in This Spec |
|---------------|-------------------------|
| Eliminate Muda (Waste) | Remove unnecessary Polars compilation |
| Jidoka (Build in Quality) | Type-safe SIMD trait bounds |
| Heijunka (Level Loading) | Consistent API across backends |
| Kaizen (Continuous Improvement) | Incremental migration phases |
| Genchi Genbutsu (Go See) | Benchmark-driven development |
| Respect for People | Clear documentation, migration guides |
| Long-term Thinking | Sustainable architecture, not quick fixes |

---

## 15. 100-Point Falsification Checklist

**Methodology**: Following Popper's falsifiability principle [11], each claim must be testable and rejectable. Tests marked [F] are falsification tests that MUST FAIL if the claim is false.

### Section A: Build Performance (1-20)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 1 | [F] No-default-features builds in <30s | `time cargo build --no-default-features --release` | <30s |
| 2 | [F] simd-core builds in <45s | `time cargo build --features simd-core --release` | <45s |
| 3 | [F] dataframe feature adds >45s | Compare with/without dataframe | Delta >45s |
| 4 | [F] Incremental no-default <5s | Touch lib.rs, rebuild | <5s |
| 5 | [F] Incremental simd-core <5s | Touch lib.rs, rebuild | <5s |
| 6 | [F] Clean build removes all artifacts | `cargo clean && ls target/` | Empty |
| 7 | [F] Parallel build scales | `CARGO_BUILD_JOBS=1 vs 8` | >3x speedup |
| 8 | [F] Check faster than build | `time cargo check` vs build | >2x faster |
| 9 | [F] Test compile <60s no-default | `time cargo test --no-default-features --no-run` | <60s |
| 10 | [F] Doc build <30s no-default | `time cargo doc --no-default-features` | <30s |
| 11 | [F] WASM build works no-default | `cargo build --target wasm32-unknown-unknown --no-default-features` | Success |
| 12 | [F] Release smaller than debug | Compare binary sizes | Release <50% |
| 13 | [F] LTO reduces binary size | Compare with/without LTO | LTO <90% |
| 14 | [F] Strip reduces binary | Compare stripped vs unstripped | <70% |
| 15 | [F] No duplicate dependencies | `cargo tree -d` | No duplicates |
| 16 | [F] Minimal features = minimal deps | Count deps with/without features | Ratio >2x |
| 17 | [F] Polars not in default dep tree | `cargo tree --no-default-features \| grep polars` | Not found |
| 18 | [F] Trueno in simd-core dep tree | `cargo tree --features simd-core \| grep trueno` | Found |
| 19 | [F] Arrow only with dataframe | `cargo tree --no-default-features \| grep arrow` | Not found |
| 20 | [F] Build cache effective | Second build <5s | <5s |

### Section B: Binary Size (21-35)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 21 | [F] No-default binary <5MB | `ls -la target/release/ruchy` | <5MB |
| 22 | [F] simd-core binary <8MB | Build with simd-core, measure | <8MB |
| 23 | [F] dataframe adds >10MB | Compare with/without | Delta >10MB |
| 24 | [F] WASM binary <2MB no-default | Build WASM, measure | <2MB |
| 25 | [F] Debug symbols removable | `strip` works | Success |
| 26 | [F] No embedded test code | Release has no `#[test]` symbols | Not found |
| 27 | [F] Dead code eliminated | `-C link-dead-code=no` effective | Size reduction |
| 28 | [F] Panic=abort reduces size | Compare panic strategies | abort <unwind |
| 29 | [F] opt-level=z smaller | Compare opt-level=3 vs z | z <3 |
| 30 | [F] Single codegen unit smaller | Compare codegen-units=1 vs 16 | 1 <16 |
| 31 | [F] No debug info in release | `readelf -S` shows no .debug | Not found |
| 32 | [F] Minimal .rodata section | Check readonly data size | <1MB |
| 33 | [F] No duplicate strings | Check for repeated literals | Minimal |
| 34 | [F] Compression effective | Compare gzipped sizes | <50% |
| 35 | [F] UPX packing works | `upx` compresses binary | Success |

### Section C: SIMD Correctness (36-55)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 36 | [F] Vector sum matches scalar | Compare SIMD vs loop sum | Exact match |
| 37 | [F] Dot product matches scalar | Compare implementations | <1e-6 error |
| 38 | [F] Matrix mul matches naive | Compare against O(n³) | <1e-5 error |
| 39 | [F] Eigenvalues match LAPACK | Compare against reference | <1e-4 error |
| 40 | [F] Transpose is involution | A^T^T == A | Exact match |
| 41 | [F] Zero vector sum is zero | sum([0; N]) | == 0.0 |
| 42 | [F] Identity matrix eigenvalues | All ones for I | == 1.0 |
| 43 | [F] Associativity holds | (a+b)+c == a+(b+c) | <1e-10 error |
| 44 | [F] Commutativity holds | a+b == b+a | Exact match |
| 45 | [F] Distributivity holds | a*(b+c) == a*b + a*c | <1e-10 error |
| 46 | [F] NaN propagates correctly | NaN in -> NaN out | is_nan() |
| 47 | [F] Inf handled correctly | Inf operations defined | No panic |
| 48 | [F] Subnormal numbers work | Very small values | Correct |
| 49 | [F] Negative zero preserved | -0.0 semantics | IEEE 754 |
| 50 | [F] Overflow saturates/wraps | Integer overflow behavior | Defined |
| 51 | [F] Underflow to zero | Very small float results | == 0.0 |
| 52 | [F] Alignment requirements met | SIMD load/store aligned | No SIGBUS |
| 53 | [F] Unaligned fallback works | Unaligned data handled | No crash |
| 54 | [F] Empty vector handled | Operations on [] | Defined result |
| 55 | [F] Single element vector | Operations on [x] | == x |

### Section D: SIMD Performance (56-70)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 56 | [F] SIMD faster than scalar sum | Benchmark 1M elements | SIMD <50% time |
| 57 | [F] SIMD faster than scalar dot | Benchmark 1M elements | SIMD <50% time |
| 58 | [F] AVX2 faster than SSE | Benchmark on AVX2 CPU | AVX2 <80% SSE |
| 59 | [F] AVX512 faster than AVX2 | Benchmark on AVX512 CPU | AVX512 <80% AVX2 |
| 60 | [F] Cache-aware tiling helps | Compare tiled vs naive matmul | Tiled faster |
| 61 | [F] Prefetching improves perf | Compare with/without prefetch | Prefetch faster |
| 62 | [F] Memory bandwidth bound | Roofline analysis | Matches model |
| 63 | [F] Compute bound for small | Small matrices compute bound | Matches model |
| 64 | [F] Parallel scaling linear | 1,2,4,8 threads | Near-linear |
| 65 | [F] No false sharing | Multi-threaded perf | No degradation |
| 66 | [F] NUMA aware allocation | Multi-socket perf | Local faster |
| 67 | [F] Consistent performance | 100 runs, low variance | CV <5% |
| 68 | [F] No warmup required | First run = steady state | <10% difference |
| 69 | [F] Branch prediction stable | Repeated runs same perf | <2% variance |
| 70 | [F] No thermal throttling | Long runs stable | <5% degradation |

### Section E: API Compatibility (71-85)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 71 | [F] Existing code compiles | All examples/ compile | Zero errors |
| 72 | [F] No API breakage | semver-checks pass | No breaking |
| 73 | [F] Trait bounds satisfied | All impls compile | Zero errors |
| 74 | [F] Generic code works | Parameterized types work | Compiles |
| 75 | [F] Error types compatible | Error handling unchanged | Same types |
| 76 | [F] Debug impl exists | println!("{:?}", x) works | Compiles |
| 77 | [F] Clone impl exists | x.clone() works | Compiles |
| 78 | [F] Send + Sync where needed | Thread-safe types | Compiles |
| 79 | [F] Serde optional | Serialize without serde | Compiles |
| 80 | [F] No_std compatible | Core-only build | Compiles |
| 81 | [F] WASM compatible | wasm32 target | Compiles |
| 82 | [F] FFI stable | C ABI functions work | Linkable |
| 83 | [F] Async compatible | Async code compiles | Compiles |
| 84 | [F] Const fn where possible | Compile-time eval | Works |
| 85 | [F] Inline hints respected | #[inline] effective | Inlined |

### Section F: Feature Flags (86-95)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 86 | [F] Default features work | `cargo build` | Success |
| 87 | [F] No-default works | `cargo build --no-default-features` | Success |
| 88 | [F] Each feature independent | Build each alone | All succeed |
| 89 | [F] All features together | `cargo build --all-features` | Success |
| 90 | [F] Feature combinations work | Pairwise combinations | All succeed |
| 91 | [F] Cfg attributes correct | `#[cfg(feature)]` logic | Correct |
| 92 | [F] Optional deps optional | Not in default tree | Verified |
| 93 | [F] Feature docs accurate | Docs match behavior | Consistent |
| 94 | [F] Feature examples work | Each feature has example | Runnable |
| 95 | [F] Feature tests exist | Each feature tested | Pass |

### Section G: Documentation & Quality (96-100)

| # | Claim | Falsification Test | Pass Criteria |
|---|-------|-------------------|---------------|
| 96 | [F] All public items documented | `#![deny(missing_docs)]` | Compiles |
| 97 | [F] Examples in docs run | `cargo test --doc` | Pass |
| 98 | [F] No clippy warnings | `cargo clippy -- -D warnings` | Zero |
| 99 | [F] No unsafe without audit | `#![forbid(unsafe_code)]` or audited | Verified |
| 100 | [F] MSRV documented and tested | CI tests minimum version | Pass |

---

### Falsification Test Runner

```bash
#!/bin/bash
# Run all 100 falsification tests
# Usage: ./scripts/falsification-tests.sh

set -e
PASS=0
FAIL=0

run_test() {
    local num=$1
    local desc=$2
    local cmd=$3
    local expected=$4

    echo -n "[$num] $desc... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo "PASS"
        ((PASS++))
    else
        echo "FAIL"
        ((FAIL++))
    fi
}

# Section A: Build Performance
run_test 1 "No-default builds" "cargo build --no-default-features --release"
run_test 17 "Polars not in default" "! cargo tree --no-default-features | grep -q polars"
# ... (remaining tests)

echo "================================"
echo "PASS: $PASS / 100"
echo "FAIL: $FAIL / 100"
echo "================================"
```

### Citation

[11] Popper, K. (1959). **The Logic of Scientific Discovery.** Basic Books. ISBN: 978-0415278447

*Falsifiability as demarcation criterion. Each claim must be testable and rejectable.*
