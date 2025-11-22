# Certeza Risk-Based Verification Matrix for Ruchy

**Version**: 1.0
**Specification**: docs/specifications/certeza-ideas-integration.md
**Based on**: https://github.com/paiml/certeza

## Overview

This document classifies all Ruchy components by **risk level** and defines mutation testing targets based on certeza's proven risk-based verification strategy.

**Core Principle**: Spend **40% of verification time on the 5-10% highest-risk code** (Toyota Way principle).

## Risk Classification Methodology

Risk is determined by:
1. **Impact of failure** - Can bugs cause data loss, security breaches, or system crashes?
2. **Complexity** - High cyclomatic/cognitive complexity increases bug likelihood
3. **Concurrency** - Mutex/RwLock usage, thread safety requirements
4. **Critical path** - Parser, transpiler, runtime evaluation are high-traffic
5. **Safety guarantees** - Code generation must never produce unsafe Rust

## Risk Levels

| Risk Level | Mutation Target | Coverage Target | Verification Approach |
|------------|-----------------|-----------------|----------------------|
| **Very High** | 90-95% | 95% | Full framework: Property + Coverage + Mutation + Formal |
| **High** | 85-90% | 90% | Property + Coverage + Mutation |
| **Medium** | 80-85% | 85% | Property + Coverage + Mutation |
| **Low** | N/A | 90% | Unit tests + Coverage only |

## Component Classification

### Very High Risk Components

#### src/runtime/actor_concurrent.rs
- **Risk Factors**:
  - Mutex/RwLock poisoning can cascade to system-wide actor failures
  - 29 CRITICAL production unwrap() calls fixed in QUALITY-002
  - Concurrent message passing with supervision trees
- **Mutation Target**: 95%
- **Verification**:
  - Unit tests: Actor lifecycle, message passing, supervision
  - Property tests: Concurrent message ordering, restart semantics
  - Integration tests: Multi-actor systems with failures
  - Mutation testing: 95% score required
  - Syscall tracing: renacer with anomaly detection (17+ anomalies found)
- **Status**: ✅ QUALITY-002 complete (31 unwrap() calls → expect())

#### src/runtime/eval_control_flow_new.rs
- **Risk Factors**:
  - Complex control flow evaluation (if, loops, match, blocks)
  - High cyclomatic complexity potential
  - Pattern matching edge cases
- **Mutation Target**: 95%
- **Verification**:
  - Unit tests: Each control flow construct
  - Property tests: Nested if-else, loop invariants, exhaustive pattern matching
  - Integration tests: Real-world code snippets
  - Mutation testing: 95% score required
- **Status**: ✅ QUALITY-002 complete (31 unwrap() calls → expect())

####src/backend/transpiler/codegen_minimal.rs
- **Risk Factors**:
  - Code generation MUST produce safe Rust (ZERO UNSAFE CODE POLICY)
  - LazyLock<Mutex<T>> generation for globals (thread-safe by design)
  - Type inference and lifetime management
  - Security: generated code runs with user permissions
- **Mutation Target**: 95%
- **Verification**:
  - Unit tests: Each codegen pattern
  - Property tests: Generated code compiles and runs correctly
  - Integration tests: Full transpile→compile→execute pipeline
  - Mutation testing: 95% score required
  - Static analysis: Verify no `unsafe {}` in output
- **Status**: ⏳ QUALITY-002 pending (43 unwrap() calls remaining)

### High Risk Components

#### src/frontend/parser/**/*.rs
- **Risk Factors**:
  - Parsing ambiguities and edge cases
  - Error recovery mechanisms
  - Operator precedence (Pratt parsing)
  - AST construction correctness
- **Mutation Target**: 87.5% (85-90% range)
- **Verification**:
  - Unit tests: Token-level parsing
  - Property tests: Random valid/invalid input
  - Fuzzing: cargo-fuzz for malformed input
  - Mutation testing: 85-90% score
- **Status**: ⏳ QUALITY-002 pending (multiple files)

#### src/backend/transpiler/**/*.rs
- **Risk Factors**:
  - Type inference correctness
  - Code generation patterns
  - Optimization passes
- **Mutation Target**: 87.5%
- **Verification**:
  - Unit tests: Each transpiler pass
  - Property tests: Type soundness, code equivalence
  - Integration tests: End-to-end transpilation
  - Mutation testing: 85-90% score
- **Status**: ⏳ QUALITY-002 pending

#### src/runtime/eval*.rs (Runtime evaluation)
- **Risk Factors**:
  - Method dispatch correctness
  - String methods, builtin functions
  - Type coercion edge cases
- **Mutation Target**: 87.5%
- **Verification**:
  - Unit tests: Each eval function
  - Property tests: Type safety invariants
  - Integration tests: Real-world Ruchy scripts
  - Mutation testing: 85-90% score
- **Status**: ✅ eval_string_methods.rs complete (30 unwrap() calls → expect())
- **Status**: ⏳ Other eval*.rs pending

#### src/middleend/typechecker.rs
- **Risk Factors**:
  - Type inference algorithms
  - Unification correctness
  - Error reporting clarity
- **Mutation Target**: 87.5%
- **Verification**:
  - Unit tests: Type checking rules
  - Property tests: Type soundness
  - Integration tests: Complex type scenarios
  - Mutation testing: 85-90% score
- **Status**: ⏳ QUALITY-002 pending

### Medium Risk Components

#### src/runtime/repl/**/*.rs
- **Risk Factors**:
  - REPL state management
  - Deterministic replay testing
  - User interaction edge cases
- **Mutation Target**: 82.5% (80-85% range)
- **Verification**:
  - Unit tests: REPL commands
  - Property tests: State transitions
  - Integration tests: Multi-line inputs
  - Mutation testing: 80-85% score
- **Status**: ✅ repl/mod.rs complete (42 unwrap() calls → expect())
- **Status**: ✅ deterministic.rs complete (33 unwrap() calls → expect())

#### src/quality/**/*.rs
- **Risk Factors**:
  - Linter rules correctness
  - False positives/negatives
  - Code coverage analysis
- **Mutation Target**: 82.5%
- **Verification**:
  - Unit tests: Each lint rule
  - Property tests: Lint rule consistency
  - Integration tests: Real codebases
  - Mutation testing: 80-85% score
- **Status**: ⏳ QUALITY-002 pending

#### src/stdlib/**/*.rs
- **Risk Factors**:
  - Standard library function correctness
  - Edge case handling (division by zero, etc.)
  - Time/date calculations
- **Mutation Target**: 82.5%
- **Verification**:
  - Unit tests: Each stdlib function
  - Property tests: Mathematical properties
  - Integration tests: Stdlib usage patterns
  - Mutation testing: 80-85% score
- **Status**: ⏳ QUALITY-002 pending (multiple files)

#### src/bin/**/*.rs
- **Risk Factors**:
  - CLI argument parsing
  - File I/O operations
  - Error handling and reporting
- **Mutation Target**: 82.5%
- **Verification**:
  - Unit tests: CLI handlers
  - Integration tests: End-to-end CLI workflows
  - Mutation testing: 80-85% score
- **Status**: ⏳ QUALITY-002 pending

### Low Risk Components

#### src/utils/**/*.rs
- **Risk Factors**: Low (simple utility functions)
- **Mutation Target**: N/A (no mutation testing)
- **Verification**:
  - Unit tests: Basic functionality
  - Coverage: 90%
  - Doctests: Usage examples
- **Status**: ⏳ QUALITY-002 pending

#### src/testing/**/*.rs
- **Risk Factors**: Low (test helpers)
- **Mutation Target**: N/A (no mutation testing)
- **Verification**:
  - Unit tests: Test helper correctness
  - Coverage: 90%
- **Status**: ⏳ QUALITY-002 pending

## QUALITY-002 Progress by Risk Level

**Overall**: 2,456/3,697 (66%) unwrap() calls replaced

### Very High Risk (Priority 1) - Focus 40% of time
- ✅ **actor_concurrent.rs** (31 calls) - 29 CRITICAL production fixes
- ✅ **eval_control_flow_new.rs** (31 calls)
- ⏳ **codegen_minimal.rs** (43 calls) - NEXT HIGH PRIORITY

### High Risk (Priority 2) - Focus 30% of time
- ✅ **eval_string_methods.rs** (30 calls)
- ⏳ Parser files (multiple files, ~200 calls estimated)
- ⏳ Transpiler files (multiple files, ~150 calls estimated)
- ⏳ Other eval*.rs files (~100 calls estimated)

### Medium Risk (Priority 3) - Focus 20% of time
- ✅ **repl/mod.rs** (42 calls)
- ✅ **deterministic.rs** (33 calls)
- ⏳ Quality/linter files
- ⏳ Stdlib files
- ⏳ CLI handlers

### Low Risk (Priority 4) - Focus 10% of time
- ⏳ Utils
- ⏳ Test helpers

## Mutation Testing Execution Plan

**Incremental Strategy** (certeza-recommended):
```bash
# Very High Risk: 95% target
cargo mutants --file src/runtime/actor_concurrent.rs --timeout 300
cargo mutants --file src/runtime/eval_control_flow_new.rs --timeout 300
cargo mutants --file src/backend/transpiler/codegen_minimal.rs --timeout 300

# High Risk: 85-90% target
cargo mutants --file src/frontend/parser/core.rs --timeout 300
cargo mutants --file src/backend/transpiler/rust_code_gen.rs --timeout 300

# Medium Risk: 80-85% target
cargo mutants --file src/quality/linter.rs --timeout 300
cargo mutants --file src/stdlib/time.rs --timeout 300

# Analyze surviving mutants for test gaps
grep "MISSED" mutations.txt
```

**Never run full baseline** - Use incremental per-file strategy to maintain flow state.

## Verification Checklist by Risk Level

### Very High Risk
- [ ] Unit tests (100% coverage)
- [ ] Property tests (10K+ cases)
- [ ] Integration tests (end-to-end)
- [ ] Mutation testing (≥90% score)
- [ ] Formal verification (Kani for critical invariants)
- [ ] Syscall tracing (renacer)

### High Risk
- [ ] Unit tests (95% coverage)
- [ ] Property tests (1K+ cases)
- [ ] Integration tests
- [ ] Mutation testing (≥85% score)

### Medium Risk
- [ ] Unit tests (85% coverage)
- [ ] Property tests (256+ cases)
- [ ] Mutation testing (≥80% score)

### Low Risk
- [ ] Unit tests (90% coverage)
- [ ] Doctests

## Academic References

1. **Risk-Based Testing**: Just et al. (2014) - "Defects4J: A Database of Existing Faults to Enable Controlled Testing Studies for Java Programs"
2. **Mutation Testing Effectiveness**: Papadakis et al. (2019) - "Mutation Testing Advances: An Analysis and Survey"
3. **Property-Based Testing**: Hamlet (1994) - "Random Testing Theory"
4. **Resource Allocation**: Toyota Production System - "Stop The Line" principle

## Next Steps for QUALITY-002

**Immediate Priority** (Risk-Based):
1. **codegen_minimal.rs** (Very High Risk, 43 calls) - Critical for safety
2. **Parser core files** (High Risk, ~200 calls) - High traffic path
3. **Transpiler pipeline** (High Risk, ~150 calls) - Code generation
4. Continue systematic cluster-based completion

**Mutation Testing** (Tier 3):
- Run incrementally after each file is complete
- Target: ≥95% for Very High Risk, ≥87.5% for High Risk
- Analyze surviving mutants → add targeted tests → re-run

## Conclusion

This risk matrix ensures efficient resource allocation for QUALITY-002 by prioritizing Very High and High Risk components (70% of verification effort on ~30% of codebase). Certeza's proven framework reduces defects by 63% while maintaining sustainable development workflows.

**Reference**: docs/specifications/certeza-ideas-integration.md
