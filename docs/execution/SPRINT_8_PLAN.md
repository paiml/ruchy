# Sprint 8: Test Suite Modernization & Parser Unit Tests

**Status**: Planning
**Blockers**: Waiting for Sprint 7 mutation test completion
**Estimated Duration**: 3-4 weeks
**Priority**: P0 - Critical Quality Debt

---

## Executive Summary

Sprint 7 mutation testing revealed **0% catch rate** in parser module, exposing critical test coverage gaps. Sprint 8 will systematically add parser unit tests to achieve 80%+ mutation coverage.

---

## Root Cause Analysis (Toyota Way)

**Problem**: 0% mutation catch rate (43+ mutants, 0 caught)

**Five Whys**:
1. **Why 0% catch rate?** Parser has no unit tests
2. **Why no unit tests?** Only integration tests via transpiler/REPL
3. **Why only integration tests?** Parser development focused on features, not isolated testing
4. **Why no isolated testing?** No TDD discipline enforced during parser implementation
5. **Root Cause**: Technical debt accumulated from feature-first development without test-first discipline

**Solution**: Implement systematic parser unit testing with TDD discipline

---

## Sprint 8 Goals

### Primary Goal
**Achieve 80%+ mutation coverage** on parser module (industry standard)

### Success Criteria
- [ ] Add 200+ parser unit tests (doctests + integration)
- [ ] Mutation catch rate: 0% → 80%+
- [ ] All 43+ identified test gaps covered
- [ ] Zero SATD comments in test files
- [ ] 100% of new tests follow TDD protocol

---

## Sprint 8 Phases

### Phase 1: Critical Test Gaps (Week 1)
**Goal**: Address the 15 highest-priority test gaps identified in Sprint 7

**Test Gaps to Fix** (from mutation testing):
1. Range operator precedence (`try_range_operators`)
2. F-string parsing (`parse_literal_token`)
3. Turbofish generics (`parse_turbofish_generics`)
4. Assignment operators (`try_assignment_operators`)
5. URL import validation (`parse_url_import`)
6. Comprehension detection (`looks_like_comprehension`)
7. Character quote processing (`should_process_char_quote`)
8. Constructor patterns (`parse_constructor_pattern`)
9. Declaration keywords as keys (`declaration_token_to_key`)
10. Actor receive blocks (`parse_actor_receive_block`)
11. Module visibility (`parse_module_item`)
12. Use path parsing (`parse_use_path`)
13. Prefix operators (`is_prefix_operator`)
14. Dataframe syntax (`is_dataframe_legacy_syntax_token`)
15. Method call parsing (`parse_method_call`)

**Deliverables**:
- [ ] 15 new test files (one per gap)
- [ ] Property tests for each function
- [ ] Doctests for all public parser functions
- [ ] Mutation retest: verify 15 mutants now caught

**Estimated Effort**: 40 hours (8 hours/day × 5 days)

### Phase 2: Systematic Parser Coverage (Week 2)
**Goal**: Add comprehensive unit tests for all parser modules

**Modules to Test**:
- `mod.rs` - Core parser logic (86.72% line coverage, 0% mutation)
- `expressions.rs` - Expression parsing (59.48% line, needs unit tests)
- `collections.rs` - Collection literals (45.64% line, critical gaps)
- `utils.rs` - Parser utilities (70.28% line, edge cases missing)
- `operator_precedence.rs` - Precedence logic (64.37% line, 0% mutation)

**Test Strategy**:
1. **Doctests**: Every public function gets runnable example
2. **Unit Tests**: Edge cases, error conditions, boundary values
3. **Property Tests**: Invariants (e.g., `parse(unparse(x)) == x`)
4. **Fuzz Tests**: Random input stress testing

**Deliverables**:
- [ ] 100+ parser unit tests added
- [ ] All parser functions have doctests
- [ ] Property test suite (10,000+ iterations per function)
- [ ] Coverage: 45-70% → 85%+ line coverage

**Estimated Effort**: 60 hours (12 hours/day × 5 days)

### Phase 3: Integration Test Modernization (Week 3)
**Goal**: Fix 21+ disabled integration tests from Sprint 7

**Disabled Tests to Fix** (with `.NEEDS_REWRITE` files):
- REPL tests (7 files) - API refactoring issues
- Interpreter tests (4 files) - eval_binary_op now private
- Value migration tests (3 files) - Rc<str> updates
- Doctest updates (7+ files) - Current API examples

**Modernization Strategy**:
1. Read `.NEEDS_REWRITE` documentation
2. Update to current API (REPL, Interpreter, Value types)
3. Add missing test coverage revealed by mutations
4. Re-enable tests one by one
5. Verify all 450+ integration tests compile

**Deliverables**:
- [ ] All 21+ disabled tests fixed and re-enabled
- [ ] All 450+ integration tests compiling
- [ ] Integration test suite 100% healthy
- [ ] Pre-commit hooks enforcing test compilation

**Estimated Effort**: 50 hours (10 hours/day × 5 days)

### Phase 4: Mutation Coverage Validation (Week 4)
**Goal**: Verify 80%+ mutation coverage achieved

**Tasks**:
1. Re-run full parser mutation tests (cargo-mutants)
2. Analyze remaining test gaps
3. Add targeted tests for survived mutants
4. Document final mutation coverage metrics
5. Add mutation testing to CI/CD pipeline

**Deliverables**:
- [ ] Mutation coverage report (baseline vs final)
- [ ] Test gap remediation complete
- [ ] CI/CD mutation testing integration
- [ ] Sprint 8 completion summary

**Estimated Effort**: 30 hours (6 hours/day × 5 days)

---

## Test Development Protocol (Mandatory)

### TDD Discipline (EXTREME TDD)
**CRITICAL**: Sprint 8 uses **Extreme TDD** - test FIRST, then implementation fixes.

**Protocol**:
1. **RED**: Write failing test for identified mutation gap
2. **GREEN**: Verify test catches the mutation
3. **REFACTOR**: Improve test quality (readability, property tests)
4. **VERIFY**: Re-run mutation test to confirm catch

**Example** (Range operator precedence):
```rust
// STEP 1: RED - Write failing test
#[test]
fn test_range_operator_precedence() {
    let source = "1..5 + 2"; // Should parse as (1..5) + 2, not 1..(5+2)
    let result = parse(source);
    assert_eq!(result, /* expected AST */);
}

// STEP 2: GREEN - Verify mutation caught
// Run: cargo mutants --file src/frontend/parser/mod.rs -F try_range_operators
// Expected: CAUGHT (not MISSED)

// STEP 3: REFACTOR - Add property test
#[cfg(test)]
proptest! {
    #[test]
    fn test_range_precedence_never_panics(a: i32, b: i32, op in "[+\\-*/]") {
        let source = format!("{a}..{b} {op} 2");
        let _ = parse(&source); // Should not panic
    }
}
```

### Quality Standards (A+ Code)
- **Cyclomatic Complexity**: ≤10 per test function
- **Cognitive Complexity**: ≤10 per test function
- **SATD**: Zero TODO/FIXME comments
- **Coverage**: 100% for test gaps
- **PMAT TDG**: A- grade minimum

---

## Metrics & Tracking

### Baseline (Sprint 7 End)
- **Parser Unit Tests**: 0
- **Mutation Catch Rate**: 0% (0/43+ mutants)
- **Line Coverage**: 45-99% (misleading)
- **Disabled Integration Tests**: 21+
- **Test Gaps Identified**: 43+

### Target (Sprint 8 End)
- **Parser Unit Tests**: 200+
- **Mutation Catch Rate**: 80%+ (industry standard)
- **Line Coverage**: 85%+ (meaningful)
- **Disabled Integration Tests**: 0
- **Test Gaps Remaining**: <20% (acceptable for non-critical paths)

### Weekly Tracking
- **Week 1**: Critical gaps → 15/15 fixed, mutation rate 0% → 35%
- **Week 2**: Systematic coverage → 100+ tests added, mutation rate 35% → 60%
- **Week 3**: Integration tests → 21+ re-enabled, all tests compiling
- **Week 4**: Validation → mutation rate 60% → 80%+, CI/CD integrated

---

## Risk Mitigation

### Risk 1: Mutation Testing Takes Too Long
**Mitigation**: Use incremental mutation testing on changed files only
```bash
cargo mutants --file src/frontend/parser/mod.rs  # Test one file at a time
```

### Risk 2: Test Gaps Too Complex
**Mitigation**: Start with simplest gaps (precedence, match arms), defer complex ones
**Priority Order**: Deletions > Replacements > Complex logic

### Risk 3: API Instability
**Mitigation**: Fix integration tests early (Week 3) to catch API changes
**Protocol**: Test compilation before ANY API changes

### Risk 4: Scope Creep
**Mitigation**: Strict focus on parser module only, defer other modules to Sprint 9+
**Rule**: No new features, only test coverage

---

## Dependencies & Blockers

### Current Blockers
- ⏳ Sprint 7 mutation testing in progress (2.7% complete, ~10 hours remaining)
- ⏳ Final test gap count unknown (43+ identified so far)

### Prerequisites
- ✅ Mutation testing framework working (cargo-mutants v25.3.1)
- ✅ Test gaps documented (SPRINT_7_SUMMARY.md)
- ✅ Quality gates enforced (PMAT, pre-commit hooks)

---

## Success Indicators

### Sprint 8 Success = ALL of:
1. ✅ 80%+ mutation coverage on parser module
2. ✅ All 21+ disabled integration tests re-enabled
3. ✅ Zero SATD comments in test files
4. ✅ Mutation testing in CI/CD pipeline
5. ✅ All pre-commit hooks passing

### Sprint 8 Failure = ANY of:
1. ❌ Mutation coverage <60%
2. ❌ Integration tests still disabled
3. ❌ SATD comments in test code
4. ❌ Manual mutation testing only (no CI/CD)
5. ❌ Pre-commit hooks bypassed

---

## Lessons from Sprint 7 (Applied to Sprint 8)

### ✅ What Worked (Continue)
1. **Toyota Way root cause analysis** - Applied to test gap prioritization
2. **Systematic documentation** - Every test gap gets detailed explanation
3. **Empirical validation** - cargo-mutants over PMAT predictions
4. **Property testing** - Will be primary test strategy

### ⚠️ What Didn't Work (Avoid)
1. **Feature-first development** - Sprint 8 is TEST-first only
2. **Deferred test updates** - Fix tests atomically with code changes
3. **Line coverage metrics** - Focus on mutation coverage instead
4. **Integration-only testing** - Add unit tests for fast feedback

---

## Commit Strategy

**Sprint 8 Commit Pattern**:
```
[SPRINT8-TEST-XXX] Add parser unit tests for <function>

- Test gap: <mutation that survived>
- Added: <number> unit tests, <number> property tests
- Mutation coverage: <before>% → <after>%
- PMAT TDG: <grade>

Mutation retest: CAUGHT (was MISSED)
```

---

## Next Steps (Immediate)

1. **Wait for Sprint 7 completion** (~10 hours)
2. **Analyze final mutation results** (identify all test gaps)
3. **Prioritize test gaps** (complexity, impact, effort)
4. **Create Sprint 8 tickets** (one per test gap + integration test)
5. **Begin Phase 1** (critical test gaps)

---

**Last Updated**: 2025-10-05
**Status**: DRAFT - Pending Sprint 7 completion
