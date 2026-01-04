# 95% Coverage Strategy: Extreme TDD Pre-Release Goal

**Version**: 1.0.0
**Date**: 2025-12-13
**Status**: DRAFT - Awaiting Review
**Authors**: Claude Code (AI Assistant)
**Related**: `90-percent-coverage-strategy-spec.md`, `improve-testing-quality-using-certeza-concepts.md`

## Executive Summary

This specification defines a rigorous, academically-grounded strategy to achieve 95% test coverage for the Ruchy compiler while maintaining fast feedback loops (<5 minutes for `test-fast`, <10 minutes for `coverage`). The approach synthesizes property-based testing, selective mutation analysis, and tiered test execution inspired by the bashrs project's proven methodology.

**Current State**: 75.26% coverage
**Target State**: 95.00% coverage
**Deadline**: Pre-release gate (FRIDAY release schedule)

## 1. Theoretical Foundation

### 1.1 The Adequacy Hypothesis

Test adequacy criteria provide the theoretical basis for determining when testing is "sufficient." Zhu et al. [1] established that coverage-based criteria (statement, branch, path) serve as necessary but not sufficient conditions for test suite quality. This specification adopts a multi-criteria approach combining:

1. **Line coverage** (primary metric): 95% target
2. **Branch coverage** (secondary): 85% target
3. **Mutation score** (quality validation): 75% on critical modules

### 1.2 Property-Based Testing Rationale

Claessen and Hughes [2] introduced QuickCheck, demonstrating that random testing with algebraic properties discovers edge cases that example-based tests miss. Key insight: a small number of well-chosen properties often achieves higher fault detection than hundreds of example tests [3].

**Ruchy Application**: Rather than exhaustive example tests, we leverage proptest with carefully designed properties that encode invariants of the parser, transpiler, and runtime.

### 1.3 Mutation Testing as Coverage Quality Validator

DeMillo et al. [4] proposed the "Competent Programmer Hypothesis" and "Coupling Effect," establishing mutation testing's theoretical foundation. Jia and Harman [5] surveyed 30 years of research confirming mutation analysis as the most effective technique for evaluating test suite quality.

**Key Finding**: High line coverage with low mutation score indicates superficial tests that execute code without verifying behavior [6].

## 2. Risk-Based Testing Strategy

### 2.1 Module Risk Classification

Following Ammann and Offutt [7], we classify modules by fault probability and impact:

| Risk Level | Modules | Coverage Target | Mutation Target |
|------------|---------|-----------------|-----------------|
| **Very High** | `parser/`, `transpiler/` | 98% line, 95% branch | 90% |
| **High** | `runtime/interpreter.rs`, `eval_*.rs` | 95% line, 90% branch | 80% |
| **Medium** | `cli/`, `quality/`, `reporting/` | 90% line | 70% |
| **Low** | `stdlib/`, utilities | 80% line | Doctest only |

### 2.2 Current Coverage Gaps (Priority Order)

Based on llvm-cov analysis:

| Module | Current | Target | Gap | Priority |
|--------|---------|--------|-----|----------|
| `runtime/interpreter.rs` | 29.68% | 95% | 65.32% | P0-Critical |
| `runtime/eval_builtin.rs` | 33% | 95% | 62% | P0-Critical |
| `quality/formatter.rs` | 29% | 90% | 61% | P1-High |
| `quality/scoring.rs` | 37% | 90% | 53% | P1-High |
| `backend/transpiler/` | ~60% | 98% | 38% | P0-Critical |
| `frontend/parser/` | ~70% | 98% | 28% | P0-Critical |

## 3. Speed-Optimized Testing Architecture

### 3.1 Three-Tier Execution Model

Adapted from bashrs's proven approach [8]:

```
Tier 1: On-Save (<1 second)
├── cargo check
├── clippy (critical lints only)
└── Affected unit tests

Tier 2: On-Commit (<5 minutes)
├── Full unit test suite
├── Property tests (50 cases)
├── Documentation tests
└── Integration smoke tests

Tier 3: Pre-Release (<30 minutes)
├── Comprehensive property tests (500 cases)
├── Mutation testing (critical modules)
├── Full coverage report
└── Cross-platform validation
```

### 3.2 Performance Optimizations

| Technique | Impact | Implementation |
|-----------|--------|----------------|
| **nextest** | 2-4x faster test execution | `cargo install cargo-nextest` |
| **mold linker** | 3-5x faster linking | `~/.cargo/config.toml` |
| **Parallel property tests** | Linear scaling with cores | `RUST_TEST_THREADS=$(nproc)` |
| **Reduced proptest cases** | 10x fewer cases in fast mode | `PROPTEST_CASES=50` |
| **Incremental compilation** | Skip unchanged crates | Default cargo behavior |
| **Test filtering** | Run only affected tests | `--test-threads` optimization |

### 3.3 Makefile Targets

```makefile
# Fast feedback (<5 min) - 50 property cases
test-fast:
    PROPTEST_CASES=50 cargo nextest run --workspace

# Coverage with reasonable speed (<10 min) - 100 cases
coverage:
    PROPTEST_CASES=100 cargo llvm-cov nextest --workspace

# Comprehensive (nightly) - 500 cases + mutations
test-comprehensive:
    PROPTEST_CASES=500 cargo nextest run --workspace
    cargo mutants --file src/frontend/parser/mod.rs --timeout 300
```

## 4. Property-Based Testing Strategy

### 4.1 Property Design Principles

Following Fink and Bishop [9], effective properties satisfy:

1. **Completeness**: Captures essential behavior
2. **Minimality**: No redundant constraints
3. **Independence**: Tests orthogonal concerns
4. **Generatability**: Valid inputs can be efficiently generated

### 4.2 Property Categories for Ruchy

| Category | Example Property | Target Module |
|----------|------------------|---------------|
| **Idempotence** | `format(format(code)) == format(code)` | `formatter.rs` |
| **Roundtrip** | `parse(tokenize(code)).is_ok()` implies `code` is valid | `parser/` |
| **Invariant** | `eval(expr).type == infer_type(expr)` | `interpreter.rs` |
| **Metamorphic** | `eval(a + 0) == eval(a)` | `eval_builtin.rs` |
| **Oracle** | `transpile(code)` compiles with rustc | `transpiler/` |

### 4.3 Property Test Template

```rust
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_arithmetic_identity(x in any::<i64>()) {
            let code = format!("{} + 0", x);
            let result = eval_code(&code).expect("valid arithmetic");
            prop_assert_eq!(result, Value::Integer(x));
        }
    }
}
```

## 5. Mutation Testing Strategy

### 5.1 Selective Mutation Approach

Full mutation testing is prohibitively expensive [10]. We adopt selective mutation:

1. **Critical-Path Mutation**: Only mutate parser, transpiler, type inference
2. **Sampling**: Random 30% sample of mutants for medium-risk modules
3. **Incremental**: Only test mutations in changed files

### 5.2 Mutation Operators (Prioritized)

| Operator | Description | Priority |
|----------|-------------|----------|
| AOR | Arithmetic Operator Replacement | High |
| ROR | Relational Operator Replacement | High |
| LCR | Logical Connector Replacement | High |
| SDL | Statement Deletion | Medium |
| ABS | Absolute Value Insertion | Low |

### 5.3 Execution Protocol

```bash
# P0-Critical modules (must achieve 80%+ killed)
cargo mutants --file src/frontend/parser/mod.rs --timeout 300
cargo mutants --file src/backend/transpiler/mod.rs --timeout 300

# P1-High modules (target 70%+ killed)
cargo mutants --file src/runtime/interpreter.rs --timeout 300

# Skip low-risk modules to maintain speed
```

## 6. Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [ ] Configure nextest for parallel execution
- [ ] Implement PROPTEST_CASES environment variable support
- [ ] Add tiered Makefile targets (test-fast, coverage, test-comprehensive)
- [ ] Establish mutation baseline for parser/transpiler

### Phase 2: High-Impact Coverage (Week 2-3)
- [ ] interpreter.rs: 29.68% → 80% via comprehensive unit tests
- [ ] eval_builtin.rs: 33% → 80% via property tests for all eval_* functions
- [ ] transpiler/: 60% → 90% via roundtrip properties
- [ ] parser/: 70% → 90% via grammar-aware fuzzing

### Phase 3: Quality Validation (Week 4)
- [ ] Mutation testing on P0 modules (target: 80% killed)
- [ ] Property test hardening (catch edge cases mutations reveal)
- [ ] Branch coverage analysis and gap filling

### Phase 4: Polish (Pre-Release)
- [ ] Achieve 95% line coverage
- [ ] All mutation scores meet targets
- [ ] Performance validation (<5 min test-fast, <10 min coverage)
- [ ] Document test strategy in CONTRIBUTING.md

## 7. Success Metrics

| Metric | Current | Target | Validation |
|--------|---------|--------|------------|
| Line Coverage | 75.26% | 95.00% | `cargo llvm-cov` |
| Branch Coverage | ~50% | 85.00% | `cargo llvm-cov --branch` |
| Mutation Score (parser) | TBD | 80%+ | `cargo mutants` |
| Mutation Score (transpiler) | TBD | 80%+ | `cargo mutants` |
| test-fast Duration | ~150s | <300s | `time make test-fast` |
| coverage Duration | ~600s | <600s | `time make coverage` |

## 8. Academic References

[1] Zhu, H., Hall, P. A., & May, J. H. (1997). Software unit test coverage and adequacy. *ACM Computing Surveys*, 29(4), 366-427. https://doi.org/10.1145/267580.267590

[2] Claessen, K., & Hughes, J. (2000). QuickCheck: A lightweight tool for random testing of Haskell programs. *Proceedings of the Fifth ACM SIGPLAN International Conference on Functional Programming (ICFP)*, 268-279. https://doi.org/10.1145/351240.351266

[3] Hamlet, R. (1994). Random testing. In J. Marciniak (Ed.), *Encyclopedia of Software Engineering* (pp. 970-978). Wiley.

[4] DeMillo, R. A., Lipton, R. J., & Sayward, F. G. (1978). Hints on test data selection: Help for the practicing programmer. *Computer*, 11(4), 34-41. https://doi.org/10.1109/C-M.1978.218136

[5] Jia, Y., & Harman, M. (2011). An analysis and survey of the development of mutation testing. *IEEE Transactions on Software Engineering*, 37(5), 649-678. https://doi.org/10.1109/TSE.2010.62

[6] Andrews, J. H., Briand, L. C., & Labiche, Y. (2006). Is mutation an appropriate tool for testing experiments? *Proceedings of the 28th International Conference on Software Engineering (ICSE)*, 402-411. https://doi.org/10.1145/1134285.1134344

[7] Ammann, P., & Offutt, J. (2017). *Introduction to Software Testing* (2nd ed.). Cambridge University Press. ISBN: 978-1107172012

[8] Papadakis, M., Kintis, M., Zhang, J., Jia, Y., Le Traon, Y., & Harman, M. (2019). Mutation testing advances: An analysis and survey. *Advances in Computers*, 112, 275-378. https://doi.org/10.1016/bs.adcom.2018.03.015

[9] Fink, G., & Bishop, M. (1997). Property-based testing: A new approach to testing for assurance. *ACM SIGSOFT Software Engineering Notes*, 22(4), 74-80. https://doi.org/10.1145/263244.263267

[10] Fraser, G., & Arcuri, A. (2012). Whole test suite generation. *IEEE Transactions on Software Engineering*, 39(2), 276-291. https://doi.org/10.1109/TSE.2012.14

## 9. Appendix: bashrs Speed Techniques Reference

The bashrs project achieves <5 minute test-fast with these techniques:

```makefile
# bashrs/Makefile (reference implementation)
test-fast:
    @if command -v cargo-nextest >/dev/null 2>&1; then \
        PROPTEST_CASES=50 RUST_TEST_THREADS=$$(nproc) cargo nextest run \
            --workspace \
            --status-level skip \
            --failure-output immediate; \
    else \
        PROPTEST_CASES=50 cargo test --workspace; \
    fi
```

Key insights:
1. **PROPTEST_CASES=50**: 10x fewer cases than comprehensive mode
2. **RUST_TEST_THREADS=$(nproc)**: Maximum parallelism
3. **cargo-nextest**: Modern test runner with better parallelism
4. **--status-level skip**: Reduce output noise
5. **--failure-output immediate**: Fast feedback on failures

## 10. Approval Checklist

Before implementation, confirm:

- [ ] Coverage targets are achievable given codebase complexity
- [ ] Speed targets align with CI/CD constraints
- [ ] Mutation testing scope is appropriately scoped
- [ ] Property test design covers critical invariants
- [ ] Resource allocation (time, compute) is approved

---

**Awaiting Review**: Please confirm this strategy before implementation begins.
