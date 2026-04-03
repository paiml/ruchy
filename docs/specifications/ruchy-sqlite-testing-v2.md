# SQLite-Style Testing Specification for Ruchy Language

**Version**: 2.0 (Research-Enhanced)
**Date**: October 15, 2025
**Methodology**: Adapted from SQLite + Peer-Reviewed Research Integration
**Target**: 100% MC/DC Coverage + 80% Mutation Coverage + Zero Regressions
**Status**: Production-Ready Specification for 16-Week Implementation

Sub-specs live in `docs/specifications/sub/` and are linked from this TOC.

---

## Table of Contents

### 1. Component-Specific Testing Standards

| Sub-spec | Lines | Scope |
|----------|-------|-------|
| [Frontend: Parser & AST](sub/sqlite-testing-frontend.md) | 568 | Grammar coverage, property tests, fuzzing |
| [Type System & Inference](sub/sqlite-testing-typesystem.md) | 453 | Type soundness proofs, inference tests |
| [Code Generation & Transpilation](sub/sqlite-testing-codegen.md) | 556 | Metamorphic testing, semantic equivalence |
| [Runtime & Interpreter](sub/sqlite-testing-runtime.md) | 429 | Error path validation, fault injection |

---

## Executive Summary

### Strategic Justification: Why SQLite-Level Testing for Ruchy?

**Target Domain**: Ruchy targets **mission-critical data science infrastructure** where runtime failures cascade catastrophically: financial model execution, scientific simulations, production ML pipelines, and embedded analytics systems. The investment in SQLite-level testing is not overhead—it is the product's primary market differentiator and technical moat.

**Economic Rationale**:
- **Cost of Failure**: A single production bug in financial trading systems averages $4.6M (Tricentis, 2021)
- **Enterprise Trust Barrier**: Fortune 500 companies reject unproven languages; SQLite-level testing provides auditable correctness certificates
- **Competitive Moat**: No existing scripting language can claim 100% MC/DC + 80% mutation coverage + formal type soundness proofs
- **Reduced Time-to-Trust**: SQLite's 20-year reliability reputation compressed into 16-week engineering sprint

### SQLite Testing Philosophy Applied to Language Implementation

SQLite achieves legendary reliability through:
- **608:1 test-to-code ratio**: 92M SLOC test code for 151K SLOC source
- **100% branch coverage**: Every code path executed in tests
- **100% MC/DC coverage**: Every condition proven to independently affect outcomes
- **Four independent test harnesses**: TCL tests, TH3, SLT, dbsqlfuzz
- **Continuous validation**: 300K pre-commit tests + nightly comprehensive suites

Ruchy adapts and extends these principles for complete language implementation across compiler, runtime, tooling, and ecosystem.

### Research Foundation

This specification integrates peer-reviewed research from:
- **NASA** (Modified Condition/Decision Coverage for avionics)
- **MIT Press** (Type system soundness theorems)
- **ACM** (Metamorphic testing methodology)
- **Elsevier** (Mutation testing effectiveness validation)
- **IEEE** (Compiler diagnostic quality frameworks)

### Ruchy Component Mapping to Enhanced Test Harnesses

| SQLite Standard | Ruchy Adaptation | Components | Research Foundation | Target |
|----------------|------------------|------------|---------------------|---------|
| **TCL Tests (21.6K)** | E2E Workflow Tests | All user-facing features | SQLite methodology | 500+ tests |
| **TH3 (1.04M SLOC)** | Property Test Suite | Parser, types, codegen, runtime | Pierce (MIT), QuickCheck | 1M+ iterations |
| **SLT (7.2M queries)** | Metamorphic Testing | Semantic equivalence validation | Chen et al. (ACM 2018) | 100K+ programs |
| **dbsqlfuzz (1B/day)** | Coverage-Guided Fuzzing | Parser security, memory safety | Zalewski (AFL) | 24 hours/release |
| **Anomaly Tests** | Error Path Validation | OOM, I/O failures, malformed input | SQLite standard | 100% error paths |
| **Veryquick (300K)** | Pre-Commit Suite | Critical paths only | SQLite standard | <3 min, 90%+ bugs |
| **New: Benchmarks** | Performance Validation | No regression detection | criterion.rs | <5% tolerance |
| **New: Diagnostics** | Error Quality Testing | Compiler usability | Barik et al. (MSR 2016) | 80%+ quality |
| **New: Corpus** | Real-World Validation | Production code compatibility | Industry practice | 10K+ programs |

**Innovation**: Ruchy employs **eight independent harnesses** versus SQLite's four, adding modern software engineering practices (performance regression, diagnostic quality, corpus validation) while maintaining SQLite's core rigor.

---

## 2. Eight-Harness Testing Framework

### Harness Summary

| # | Harness | Purpose | Test Count | Coverage | Research |
|---|---------|---------|-----------|----------|----------|
| **1** | E2E Workflows | User-facing functionality | 500+ | 100% workflows | SQLite TCL |
| **2** | Property Tests | Mathematical correctness | 1M+ iterations | 100% branch | Pierce, QuickCheck |
| **3** | Metamorphic Tests | Semantic equivalence | 100K+ programs | 99.9% match | Chen et al. (ACM) |
| **4** | Mutation Tests | Test effectiveness | Continuous | 80%+ score | Papadakis et al. |
| **5** | Fuzzing | Memory safety | 24 hrs/release | 0 crashes | AFL (Zalewski) |
| **6** | Benchmarks | Performance | Continuous | <5% regression | criterion.rs |
| **7** | Diagnostics | Error quality | 100+ scenarios | 80% quality | Barik et al. (MSR) |
| **8** | Corpus | Real-world | 10K+ programs | >95% success | Industry practice |

### Harness 5: Performance Benchmarking

```rust
// benches/compiler_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn parse_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    for size in [100, 1_000, 10_000, 100_000] {
        group.bench_with_input(
            BenchmarkId::new("expression", size),
            &size,
            |b, &size| {
                let expr = generate_expression_of_size(size);
                b.iter(|| parse(&expr));
            }
        );
    }
    group.finish();
}

criterion_group!(benches, parse_benchmarks);
criterion_main!(benches);
```

### Harness 6: Diagnostic Quality Testing

**Research Foundation**: Barik et al. (2016), "Compiler error messages considered unhelpful"

**Quality Criteria**:
1. **Precision**: Exact error location (line, column)
2. **Context**: Show surrounding code
3. **Actionability**: Suggest concrete fixes

### Harness 7: Corpus Testing

Adapt 10,000+ real-world programs to Ruchy syntax. Target: >95% success rate.

---

## 3. Release Criteria (Enhanced)

### 3.1 Mandatory Requirements (15 Gates)

**No release until ALL criteria met**:

1. **Branch Coverage**: 100%
2. **MC/DC Coverage**: 100% on critical logic
3. **Mutation Coverage**: 80%+
4. **Property Tests**: 1M+ iterations, 100% pass
5. **Metamorphic Tests**: 100K+ programs, <10 divergences
6. **E2E Tests**: 500+ workflows, 100% pass
7. **Fuzzing**: 24 hours, 0 crashes
8. **Performance**: <5% regression
9. **Diagnostic Quality**: 80%+ score
10. **Corpus Success**: >95% on 10K programs
11. **Complexity**: ≤10 per function
12. **Security**: 0 unsafe violations (cargo-geiger)
13. **Vulnerabilities**: 0 known (cargo-audit)
14. **Regression**: 0 known regressions
15. **Cross-Platform**: Linux, macOS, Windows

---

## 4. Risk-Driven Implementation Roadmap

**Philosophy**: Vertical slice first (end-to-end correctness for minimal subset) beats component-by-component development.

### Phase 1: Vertical Slice (Weeks 1-4)
**Scope**: Integers, arithmetic, variables, functions, if/else

### Phase 2: Feature Expansion (Weeks 5-12)
**Features**: Strings → Collections → Pattern Matching → Generics → Standard Library

### Phase 3: Ecosystem (Weeks 13-16)
**Components**: DataFrame → WASM → LSP → Notebook → MCP

---

## 5. Research Foundation

### Primary Citations

1. **Hayhurst et al. (2001)**: MC/DC for avionics (NASA/TM-2001-210876)
2. **Papadakis et al. (2019)**: Mutation testing effectiveness (Elsevier)
3. **Chen et al. (2018)**: Metamorphic testing methodology (ACM)
4. **Pierce (2002)**: Type soundness theorems (MIT Press)
5. **Barik et al. (2016)**: Diagnostic quality framework (IEEE MSR)
6. **Zalewski (2014)**: Coverage-guided fuzzing (AFL)
7. **Hipp (2020)**: SQLite testing methodology

### Standards

- **DO-178B/C**: Avionics software certification
- **ISO 26262**: Automotive functional safety
- **Common Criteria**: IT security evaluation

---

**Version**: 2.0 (Research-Enhanced)
**Date**: October 15, 2025
**Status**: Production-Ready Specification
**Implementation Timeline**: 16 weeks
**Expected Quality Grade**: A+ (SQLite Standard)
