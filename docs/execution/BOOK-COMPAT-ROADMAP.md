# Ruchy-Book Compatibility Roadmap

**Based on**: ruchy-book compatibility report (2025-10-13)
**Current Status**: 84% complete (113/134 examples passing)
**Target**: 100% complete (134/134 examples passing)
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)

---

## Executive Summary

The ruchy-book contains 134 examples across 13 chapters. Currently 113 examples work (84%), with one critical gap (DataFrame transpilation) and several high-priority missing features.

**Impact of Completion**:
- Phase 1 (DataFrame): 84% → 87% (+3%)
- Phase 2 (Stdlib): 87% → 98% (+11%)  
- Phase 3 (Testing): 98% → 99.5% (+1.5%)
- Phase 4 (Advanced): 99.5% → 100% (+0.5%)

---

## Phase 1: CRITICAL - DataFrame Transpilation 🔴

**Priority**: IMMEDIATE
**Ticket**: DATAFRAME-001
**Examples Affected**: 4 in Ch18 (0% working → 100% target)
**Impact**: +3% compatibility

### Problem

DataFrames work perfectly in interpreter but fail to compile to binaries.

**Error**: `error[E0433]: failed to resolve: use of unresolved module or unlinked crate 'polars'`

### Solution

Auto-generate `Cargo.toml` with polars dependency during compilation.

### Implementation

- **RED**: 10 unit tests for DataFrame compilation
- **GREEN**: Cargo.toml generation, polars injection, df![] transpilation
- **REFACTOR**: 10K+ property tests, mutation testing

### Ticket

See: `docs/execution/DATAFRAME-001-transpilation.md`

---

## Phase 2: HIGH PRIORITY - Standard Library 🟡

**Priority**: SHORT-TERM (1-2 weeks)
**Examples Affected**: ~15 across Ch15, Ch04-01, Ch17
**Impact**: +11% compatibility

### STDLIB-001: String Methods

**Examples Affected**: 5-7 examples
**Methods Needed**:
- `.parse::<T>()` - Parse strings to other types
- `.split(pattern)` - Split strings into Vec<&str>
- `.trim()` - Remove whitespace
- `.to_uppercase()` / `.to_lowercase()` - Case conversion
- `.replace(from, to)` - String replacement

**Implementation Plan**:
- **RED**: 10 tests (one per method + edge cases)
- **GREEN**: Implement each method in stdlib
- **REFACTOR**: Property tests with random strings

### STDLIB-002: Command-Line Arguments

**Examples Affected**: 3-4 examples (Ch15)
**Feature Needed**: `std::env::args()`

**Implementation Plan**:
- **RED**: Tests for arg parsing, arg count, arg access
- **GREEN**: Implement env::args() in stdlib
- **REFACTOR**: Property tests with various arg patterns

### STDLIB-003: Math Functions

**Examples Affected**: 4-6 examples (Ch04-01)
**Functions Needed**:
- `sqrt(x)` - Square root
- `pow(base, exp)` - Power (if not working)
- `abs(x)` - Absolute value
- `sin(x)`, `cos(x)`, `tan(x)` - Trigonometry
- `log(x)`, `ln(x)` - Logarithms
- `floor(x)`, `ceil(x)`, `round(x)` - Rounding

**Implementation Plan**:
- **RED**: Test each math function with known values
- **GREEN**: Delegate to Rust's libm or std::f64
- **REFACTOR**: Property tests for mathematical identities

---

## Phase 3: HIGH PRIORITY - Testing Framework 🟡

**Priority**: SHORT-TERM
**Examples Affected**: 2 in Ch16
**Impact**: +1.5% compatibility

### TESTING-001: Assertion Macros

**Macros Needed**:
- `assert_eq!(left, right)` - Equality assertion
- `assert_eq!(left, right, "message")` - With message
- `assert!(condition)` - Boolean assertion
- `assert!(condition, "message")` - With message

**Implementation Plan**:
- **RED**: Tests for assert failures, success cases, messages
- **GREEN**: Implement macros in parser/interpreter
- **REFACTOR**: Property tests with random values

---

## Phase 4: MEDIUM PRIORITY - Advanced Features 🟢

**Priority**: MEDIUM-TERM (1 month)
**Examples Affected**: ~6 across Ch04, Ch05, Ch19
**Impact**: +4.5% compatibility

### PATTERN-001: Advanced Pattern Matching

**Features Needed**:
- Complex destructuring in match arms
- Pattern guards (`match x if condition`)
- Struct pattern matching
- Exhaustiveness checking

**Examples Affected**: 3-4 examples (Ch04, Ch05)

### TYPES-001: Result and Option Types

**Types Needed**:
- `Result<T, E>` enum
- `Option<T>` enum
- `.unwrap()`, `.expect()` methods
- `?` operator for error propagation

**Examples Affected**: 2-3 examples (Ch17)

### SYNTAX-001: Pipeline Operator

**Feature Needed**: `|>` operator for function chaining

**Example**:
```rust
data |> filter |> map |> collect
```

**Examples Affected**: 1-2 examples (Ch04)

---

## Implementation Priority Order

### Sprint 1: DATAFRAME-001 (Critical)

**Duration**: 2-3 days
**Impact**: 84% → 87%
**Status**: Ticket created

**Tasks**:
1. Create RED phase tests (10 unit tests)
2. Implement Cargo.toml generation
3. Implement DataFrame transpilation
4. REFACTOR with property tests
5. Validate all 4 Ch18 examples pass

### Sprint 2: STDLIB-001, STDLIB-002, STDLIB-003 (High Priority)

**Duration**: 1-2 weeks
**Impact**: 87% → 98%

**Week 1**:
- STDLIB-001: String methods (3 days)
- STDLIB-002: Command-line args (2 days)

**Week 2**:
- STDLIB-003: Math functions (3-4 days)
- Integration testing (1 day)

### Sprint 3: TESTING-001 (High Priority)

**Duration**: 2-3 days
**Impact**: 98% → 99.5%

**Tasks**:
1. Implement assert_eq! macro
2. Implement assert! macro
3. Test framework integration
4. Validate Ch16 examples

### Sprint 4: PATTERN-001, TYPES-001, SYNTAX-001 (Medium Priority)

**Duration**: 2-4 weeks
**Impact**: 99.5% → 100%

**Week 1-2**: PATTERN-001 (Advanced patterns)
**Week 3**: TYPES-001 (Result/Option)
**Week 4**: SYNTAX-001 (Pipeline operator)

---

## Success Metrics

### Current Baseline (2025-10-13)

- **Overall**: 113/134 examples (84%)
- **Fully Working**: 84 examples (63%)
- **Partially Working**: 29 examples (22%)
- **Not Working**: 21 examples (16%)

### After Phase 1 (DataFrame)

- **Overall**: 117/134 examples (87%)
- **Critical Gap Resolved**: Data science use cases enabled

### After Phase 2 (Stdlib)

- **Overall**: 132/134 examples (98%)
- **Production Ready**: Most use cases supported

### After Phase 3 (Testing)

- **Overall**: 133/134 examples (99.5%)
- **Complete Testing Framework**: Full test suite capability

### After Phase 4 (Advanced)

- **Overall**: 134/134 examples (100%)
- **Feature Complete**: All documented features working

---

## Testing Strategy (Per Ticket)

All tickets follow EXTREME TDD methodology from RUNTIME-003:

### RED Phase
- Write 10 unit tests FIRST (all failing)
- Document expected behavior
- Create test fixtures
- Verify tests fail for right reasons

### GREEN Phase
- Minimal implementation to pass tests
- One test at a time
- No premature optimization
- Commit after each test passes

### REFACTOR Phase
- Add 6+ property tests (10K+ cases)
- Run mutation tests (target ≥75%)
- Optimize if needed
- Update documentation

### Quality Gates
- Complexity ≤10 per function
- Test coverage 100% for new code
- Property tests validate invariants
- Mutation tests prove test effectiveness

---

## Ticket Creation Checklist

For each ticket, create a document following DATAFRAME-001 template:

- [ ] Problem statement with ruchy-book reference
- [ ] Specification with examples
- [ ] RED phase test plan (10 unit tests)
- [ ] GREEN phase implementation steps
- [ ] REFACTOR phase property/mutation tests
- [ ] Acceptance criteria
- [ ] Known challenges
- [ ] Success metrics

---

## Dependencies and Blockers

### DATAFRAME-001 Dependencies
- None (can start immediately)

### STDLIB-001/002/003 Dependencies
- None (can run in parallel)

### TESTING-001 Dependencies
- Requires parser updates (macro support)

### PATTERN-001/TYPES-001 Dependencies
- May require parser enhancements
- May require type system updates

### SYNTAX-001 Dependencies
- Requires lexer update (|> token)
- Requires parser update (operator precedence)

---

## Resource Allocation

### Estimated Total Time
- Phase 1: 12-16 hours (2-3 days)
- Phase 2: 60-80 hours (1.5-2 weeks)
- Phase 3: 12-16 hours (2-3 days)
- Phase 4: 80-120 hours (2-4 weeks)

**Total**: 164-232 hours (4-6 weeks of focused work)

### Complexity Budget
- All new functions: Complexity ≤10
- Property tests: 10K+ cases per feature
- Mutation tests: ≥75% coverage
- Test execution: <5 minutes per feature

---

## Risks and Mitigation

### Risk 1: DataFrame Transpilation Complexity

**Risk**: Polars API may be complex to transpile correctly

**Mitigation**:
- Start with simple cases (RED phase)
- Incremental complexity (GREEN phase)
- Extensive property testing (REFACTOR phase)

### Risk 2: Stdlib Function Scope Creep

**Risk**: String/math functions may have many edge cases

**Mitigation**:
- Define minimal feature set in ticket
- Property tests find edge cases
- Document known limitations

### Risk 3: Parser Changes for Advanced Features

**Risk**: Pattern matching/macros may require significant parser work

**Mitigation**:
- Defer to Phase 4 (not critical path)
- Can reach 98% without these features
- Consider alternative syntax if needed

---

## Next Actions

### Immediate (This Session)
1. ✅ Create DATAFRAME-001 ticket
2. ⏳ Commit tickets to git
3. ⏳ Update roadmap.yaml with new tickets
4. ⏳ Begin DATAFRAME-001 RED phase

### This Week
- Complete DATAFRAME-001 (all phases)
- Create STDLIB-001, STDLIB-002, STDLIB-003 tickets
- Begin STDLIB-001 RED phase

### This Month
- Complete Phase 1 and Phase 2
- Reach 98% compatibility
- Begin Phase 3 (testing framework)

---

## Conclusion

The ruchy-book compatibility report provides a clear roadmap to 100% feature completion. By following EXTREME TDD methodology (as proven successful in RUNTIME-003), we can systematically address each gap:

1. **Phase 1 (Critical)**: Fix DataFrame transpilation → 87%
2. **Phase 2 (High)**: Implement missing stdlib → 98%
3. **Phase 3 (High)**: Add testing framework → 99.5%
4. **Phase 4 (Medium)**: Advanced features → 100%

**Key Principle**: Stop, write tests, implement minimally, validate with property tests, commit, repeat.

**Expected Outcome**: Production-ready Ruchy language with complete ruchy-book compatibility.

---

**Created**: 2025-10-13
**Source**: ruchy-book compatibility report
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)
**Target Completion**: 4-6 weeks
