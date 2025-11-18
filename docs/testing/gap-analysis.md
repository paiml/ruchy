# Certeza Phase 2: Testing Gap Analysis
**Document ID**: CERTEZA-002-GAP-ANALYSIS
**Version**: 1.0.0
**Date**: 2025-11-18
**Status**: Active

## Executive Summary

This document identifies critical gaps in test coverage for High-Risk and Very High-Risk modules based on the risk stratification performed in Phase 2 of the Certeza framework implementation.

**Key Finding**: High-Risk modules (parser, type checker, transpiler) have **INSUFFICIENT property-based testing coverage**, which is critical for compiler correctness.

---

## Gap Analysis Methodology

1. **Risk Classification**: Classified all 305 modules into risk levels (see `risk-stratification.yaml`)
2. **Test Inventory**: Counted unit tests, property tests, and integration tests per module
3. **Coverage Assessment**: Identified modules below target metrics
4. **Priority Ranking**: Ordered gaps by risk level and impact

---

## Critical Gaps (BLOCKING for Phase 3)

### 1. Parser Property Tests (CRITICAL GAP)

**Module**: `src/frontend/parser/` (46 files)
**Risk Level**: HIGH
**Current State**:
- Unit test modules: 9
- Property tests: **0** ❌
- Integration tests: ~50+ in tests/

**Target State** (Certeza High-Risk):
- Property test coverage: 80%+
- Required properties:
  - `parse_always_produces_valid_ast` - Parser never panics
  - `parse_roundtrip_preserves_semantics` - Parse → AST → String → Parse = same AST
  - `parse_never_panics_on_invalid_input` - Fuzzing-style resilience
  - `parse_error_recovery_produces_partial_ast` - Error recovery works

**Impact**: Parser bugs cause **syntax errors**, **incorrect AST**, **cascading failures** in transpiler.

**Action Required**: Create `tests/properties/parser_properties.rs` with comprehensive property tests.

**Priority**: 🔴 **P0 - CRITICAL** (Phase 3 Sprint 5)

---

### 2. Type Checker Property Tests (CRITICAL GAP)

**Module**: `src/middleend/` (10 files: infer.rs, unify.rs, types.rs, etc.)
**Risk Level**: HIGH
**Current State**:
- Unit test modules: 4
- Property tests: **4** (MINIMAL) ⚠️
- Integration tests: Limited

**Target State** (Certeza High-Risk):
- Property test coverage: 80%+
- Required properties:
  - `type_inference_is_deterministic` - Same input → same type
  - `unification_is_idempotent` - unify(T, T) = T
  - `unification_is_commutative` - unify(A, B) = unify(B, A)
  - `type_inference_never_panics` - Resilience
  - `type_soundness_preservation` - Well-typed → well-typed after unification

**Impact**: Type checker bugs cause **wrong types**, **runtime type errors**, **unsafe code generation**.

**Action Required**: Expand property tests in `src/middleend/infer.rs` and create dedicated test suite.

**Priority**: 🔴 **P0 - CRITICAL** (Phase 3 Sprint 5)

---

### 3. Transpiler Property Tests (MODERATE GAP)

**Module**: `src/backend/transpiler/` (30 files)
**Risk Level**: HIGH
**Current State**:
- Unit test modules: 19
- Property tests: **7** (SOME) ⚠️
- Integration tests: ~60+ in tests/

**Target State** (Certeza High-Risk):
- Property test coverage: 80%+
- Required properties:
  - `generated_rust_always_compiles` - Transpiled code compiles with rustc
  - `no_unsafe_in_generated_code` - **GitHub #132 enforcement**
  - `no_static_mut_in_generated_code` - Thread safety
  - `generated_code_preserves_semantics` - Semantic equivalence
  - `generated_code_is_idiomatic` - Follows Rust best practices

**Impact**: Transpiler bugs cause **wrong Rust code**, **compilation failures**, **runtime crashes**, **unsafe code** (GitHub #132).

**Action Required**: Expand property tests to cover all transpiler modules.

**Priority**: 🟡 **P1 - HIGH** (Phase 3 Sprint 6)

---

### 4. Unsafe Code Verification (CRITICAL GAP)

**Module**: Very High-Risk modules with unsafe blocks
**Risk Level**: VERY HIGH
**Current State**:
- `src/jit/compiler.rs` - Manual testing only
- `src/runtime/arena.rs` - Manual testing only
- `src/runtime/bytecode/vm.rs` - Manual testing only

**Target State** (Certeza Very High-Risk):
- Line coverage: 100%
- Branch coverage: 100%
- Mutation score: 95%+
- **Formal verification**: Kani proofs for memory safety invariants

**Impact**: Unsafe bugs cause **memory corruption**, **security vulnerabilities**, **undefined behavior**.

**Action Required**:
1. Add Kani verification harnesses
2. Property tests for invariants
3. Comprehensive unit tests for all unsafe paths

**Priority**: 🔴 **P0 - CRITICAL** (Phase 5 Sprint 9)

---

### 5. static mut Refactoring (BLOCKING)

**Module**: `src/backend/transpiler/mod.rs`
**Risk Level**: VERY HIGH
**Current State**: Contains `static mut` (violates ZERO UNSAFE CODE POLICY)

**Target State**: Refactored to `LazyLock<Mutex<T>>` or `LazyLock<RwLock<T>>`

**Impact**: Thread safety violations, data races, undefined behavior.

**Action Required**: Refactor per GitHub #132 and CLAUDE.md guidelines.

**Priority**: 🔴 **P0 - BLOCKING** (Immediate)

**Reference**: GitHub Issue #132 - [CRITICAL] Transpiler generates invalid Rust code

---

## Testing Metrics Summary

| Risk Level | Modules | Unit Tests | Property Tests | Integration Tests | Gap Status |
|------------|---------|------------|----------------|-------------------|------------|
| Very High | 14 | Limited | **0** ❌ | Limited | CRITICAL |
| High | 87 | Good | **11** ⚠️ | Good | CRITICAL |
| Medium | ~120 | Good | Limited | Good | ACCEPTABLE |
| Low | ~40 | Basic | N/A | Basic | ACCEPTABLE |

### Property Test Coverage by Module

| Module | Files | Property Tests | Target | Gap | Priority |
|--------|-------|----------------|--------|-----|----------|
| **Parser** | 46 | **0** | 80%+ | 100% | P0 |
| **Type Checker** | 10 | **4** | 80%+ | 75% | P0 |
| **Transpiler** | 30 | **7** | 80%+ | 50% | P1 |
| Runtime | 72 | Some | 60% | 40% | P2 |
| Utils | 40 | N/A | Doctests | 0% | P3 |

---

## Root Cause Analysis (Five Whys)

**Problem**: Parser has zero property tests despite being High-Risk.

1. **Why?** Property tests were not prioritized during initial development.
2. **Why?** Testing strategy focused on integration tests over property tests.
3. **Why?** Property testing patterns were not institutionalized in project.
4. **Why?** Certeza-style risk-based allocation was not implemented.
5. **Why?** No systematic framework for test prioritization existed.

**Root Cause**: Lack of risk-based testing framework → Ad-hoc testing strategy → Under-testing of critical paths.

**Solution**: Implement Certeza Phase 3 (Property Testing Expansion) systematically.

---

## Recommended Actions (Prioritized)

### Immediate (Sprint 3)
1. ✅ **DONE**: Risk stratification and gap analysis
2. 🔴 **BLOCKING**: Refactor `static mut` in transpiler to `LazyLock<Mutex<T>>` (GitHub #132)

### Phase 3 (Sprint 5-6): Property Testing Expansion
3. 🔴 **P0**: Create parser property test suite (`tests/properties/parser_properties.rs`)
4. 🔴 **P0**: Expand type checker property tests (`tests/properties/typechecker_properties.rs`)
5. 🟡 **P1**: Expand transpiler property tests (existing + new cases)

### Phase 4 (Sprint 7-8): Mutation Testing
6. 🟡 **P1**: Run mutation tests on High-Risk modules (≥85% score target)
7. 🟡 **P1**: Document mutation test results and gaps

### Phase 5 (Sprint 9-10): Formal Verification
8. 🟠 **P2**: Add Kani verification for unsafe blocks
9. 🟠 **P2**: Formal proofs for memory safety invariants

---

## Success Criteria (Phase 2 Complete)

✅ **ACHIEVED**:
- Risk stratification document created (`risk-stratification.yaml`)
- Gap analysis completed (this document)
- Testing priorities established
- Action plan defined

✅ **METRICS**:
- 305 modules classified by risk level
- 14 Very High-Risk modules identified
- 87 High-Risk modules identified
- Critical gaps documented with priorities

🔴 **BLOCKING ISSUES**:
- GitHub #132: `static mut` refactoring required
- Parser property tests: 0 → target 80%+
- Type checker property tests: minimal → target 80%+

---

## References

- **Specification**: `docs/specifications/improve-testing-quality-using-certeza-concepts.md`
- **Risk Stratification**: `docs/testing/risk-stratification.yaml`
- **CLAUDE.md**: Certeza Three-Tiered Testing Framework section
- **GitHub #132**: [CRITICAL] Transpiler generates invalid Rust code - must use RefCell/Mutex not unsafe
- **Certeza Framework**: https://github.com/paiml/certeza/

---

## Appendix: Test Counts

```bash
# Parser
grep -r "mod tests" src/frontend/parser/*.rs | wc -l
# Output: 9

grep -r "proptest!" src/frontend/parser/*.rs | wc -l
# Output: 0 ❌

# Type Checker
grep -r "mod tests" src/middleend/*.rs | wc -l
# Output: 4

grep -r "proptest!" src/middleend/*.rs | wc -l
# Output: 4 ⚠️

# Transpiler
grep -r "mod tests" src/backend/transpiler/*.rs | wc -l
# Output: 19

grep -r "proptest!" src/backend/transpiler/*.rs | wc -l
# Output: 7 ⚠️
```

---

**Next Phase**: Phase 3 - Property Testing Expansion (Sprint 5-6)
**Ticket**: CERTEZA-003
**Goal**: Achieve 80% property test coverage for High-Risk modules
