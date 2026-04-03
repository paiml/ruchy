# Ruchy Test: Native Notebook Testing Framework

## Overview

`ruchy test` executes notebook cells as test cases, validating outputs against expectations with property-based and deterministic verification. The framework supports multiple validation strategies (deterministic, property-based, differential, formal), state management across cells, mutation testing, WASM sandboxed educational labs, and runtime acceptance testing for compiler backends.

```bash
ruchy test                                  # Test all notebooks
ruchy test notebooks/analysis.ruchynb       # Test specific notebook
ruchy test --coverage                       # Test with coverage
ruchy test --mutate                         # Test with mutation
ruchy test --update-golden                  # Regenerate golden outputs
```

---

## Sub-spec Index

| Sub-spec | Sections | Description | Lines |
|----------|----------|-------------|-------|
| [Core Testing Architecture](sub/test-notebook-core-architecture.md) | 1-4 | Command architecture, cell test annotations, output validation engine (deterministic and property-based), state management across cells | 205 |
| [Verification, Coverage, and Quality](sub/test-notebook-verification-quality.md) | 5-11 | Formal verification (SMT/Z3), complexity analysis, runtime acceptance testing, canary notebooks, coverage tracking, mutation testing, golden output management, configuration, CLI output, CI integration | 472 |
| [WASM Educational Labs and Unified Architecture](sub/test-notebook-wasm-labs-unified.md) | 12-15 | WASM sandbox execution, automated feedback generation, progressive hints, anti-cheating, lab authoring DSL, learning analytics, unified test runner, pragmatic SMT integration, incremental complexity analysis, WASM deployment, implementation strategy, performance targets | 499 |

---

## Key Design Decisions

1. **Test metadata lives in notebook** -- No separate test files to maintain
2. **Multiple validation strategies** -- Deterministic, property, differential, formal
3. **State preservation** -- Tests see accumulated notebook state
4. **Efficient golden storage** -- Format-appropriate serialization (Parquet for DataFrames, perceptual hash for plots)
5. **Incremental testing** -- Only re-run affected cells on change
6. **WASM sandboxing** -- Safe execution of untrusted student code
7. **Progressive disclosure** -- Hints and feedback adapt to student progress
8. **Anti-cheating** -- Parameterized problems and plagiarism detection
9. **Unified pipeline** -- Single framework serves users and compiler team
10. **Pragmatic verification** -- Bounded proofs with tractability checks

## Performance Targets

- Simple assertion: <1ms overhead per cell
- Property test: <100ms for 100 iterations
- SMT verification: <5s timeout, cached results
- Complexity analysis: <500ms static, <2s empirical
- WASM compilation: <200ms for typical exercise
- Differential test: <10s for 4 backends

## Implementation Phases

| Phase | Timeframe | Scope |
|-------|-----------|-------|
| 1 | Weeks 1-4 | Core testing, snapshots, state, coverage |
| 2 | Weeks 5-8 | Property-based, differential, mutation, baselines |
| 3 | Weeks 9-12 | SMT integration (Z3), complexity, bounded verification |
| 4 | Weeks 13-16 | WASM sandbox, feedback engine, anti-cheating, analytics |
| 5 | Weeks 17-20 | CI/CD integration, performance, documentation, migration |
