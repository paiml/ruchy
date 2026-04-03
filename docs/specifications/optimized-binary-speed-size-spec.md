# Optimized Binary Speed & Size Specification

**Document ID**: PERF-002
**Version**: 1.1.0
**Status**: Partially Implemented (Phases 2-4 Complete)
**Author**: Ruchy Team
**Date**: 2025-11-10
**Last Updated**: 2025-11-10 (implementation status)

---

## Executive Summary

This specification defines optimal Rust compilation profiles for `ruchy compile` based on empirical benchmarking data from the [compiled-rust-benchmarking](https://github.com/paiml/compiled-rust-benchmarking) project. The study demonstrates **5-51x performance improvements** and **91.7% binary size reductions** through systematic optimization.

**Key Finding**: LTO ("link-time optimization") provides BOTH speed AND size benefits, making it the optimal default for most workloads.

**Binary Analysis Integration**: Complements profile optimization with `ruchy analyze` command ([Issue #145](https://github.com/paiml/ruchy/issues/145)) for comprehensive binary analysis, optimization recommendations, and CI/CD integration. Prototype validated with 6/6 tests passing in [ruchyruchy COMPILED-INST-003](https://github.com/paiml/ruchyruchy).

---

## Implementation Status

| Phase | Feature | Status | Commit | Tests | Quality |
|-------|---------|--------|--------|-------|---------|
| **Phase 1** | Update Documentation | ✅ COMPLETE | N/A | N/A | N/A |
| **Phase 2** | Fix `release-dist` Profile | ✅ COMPLETE | 10d92ad6 | 15/15 | A (92.4) |
| **Phase 3** | `--show-profile-info` Flag | ✅ COMPLETE | f898f243 | 15/15 | A+ (96.8) |
| **Phase 4** | `--pgo` Automation | ✅ COMPLETE | e68bebb1 | 15/15 | ≤10 complexity |

**Overall Status**: All 4 phases implemented and tested ✅

**Released In**:
- Phase 2: v3.212.0
- Phase 3: v3.213.0
- Phase 4: v3.214.0

**Total Test Coverage**: 45 tests (43 automated, 2 require user interaction)

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

---

## Sub-spec Index

| Sub-spec | Sections | Description | Lines |
|----------|----------|-------------|-------|
| [binary-profiles-migration.md](sub/binary-profiles-migration.md) | Implementation, CLI, Workloads, Migration | Profile configurations (release, release-tiny, release-ultra, release-dist), CLI integration, binary analysis command (`ruchy analyze`), workload-specific recommendations, and migration strategy (Phases 1-4) | 500 |
| [binary-testing-future.md](sub/binary-testing-future.md) | Testing, Docs, Future, References | Testing strategy (regression, integration, benchmarks), documentation updates, future work (auto-profile, binary patching, ML recommendations), and references | 254 |

---

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
## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Default speedup | >10x vs baseline | Benchmark suite |
| Tiny binary size | <500 KB | File size |
| User adoption | >80% use default | Telemetry (opt-in) |
| Compile time | <2min for 5K LOC | CI/CD metrics |
| Documentation clarity | >90% understand profiles | User survey |

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
