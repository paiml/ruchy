# QUALITY-RECOVERY Sprint: Test-Driven BUG-002 Fix

**Sprint Goal**: Fix higher-order function support through comprehensive testing
**Duration**: 2-3 days
**Priority**: P0 - CRITICAL
**Approach**: Test-First, Toyota Way, Zero Defects

## 📋 Sprint Backlog

### RECOVERY-001: Property Testing Infrastructure [8 hours]
**Objective**: Implement property tests for type inference invariants

**Acceptance Criteria**:
- [ ] 50+ property tests for parameter type inference
- [ ] 50+ property tests for return type inference  
- [ ] Property: main() never has return type
- [ ] Property: function parameters preserve type consistency
- [ ] Property: higher-order functions type correctly

**Implementation**:
```rust
// tests/properties/type_inference.rs
proptest! {
    #[test]
    fn main_never_has_return_type(code in valid_main_function()) {
        let transpiled = transpile(code);
        assert!(!transpiled.contains("fn main() ->"));
    }
    
    #[test]
    fn function_params_consistent(func in function_with_params()) {
        // If param used as function, must be Fn type
        let transpiled = transpile(func);
        verify_param_type_consistency(transpiled);
    }
}
```

### RECOVERY-002: Fuzz Testing Campaign [6 hours]
**Objective**: Discover edge cases in transpiler with fuzzing

**Acceptance Criteria**:
- [ ] 10+ fuzz targets for transpiler
- [ ] 1M+ iterations without panic
- [ ] Corpus of 1000+ test cases
- [ ] Zero crashes or hangs
- [ ] Coverage-guided fuzzing enabled

**Targets**:
1. `fuzz_function_transpilation`
2. `fuzz_parameter_inference`
3. `fuzz_return_type_inference`
4. `fuzz_higher_order_functions`
5. `fuzz_main_generation`

### RECOVERY-003: Unit Test Coverage to 80% [12 hours]
**Objective**: Comprehensive unit test coverage of transpiler

**Acceptance Criteria**:
- [ ] Transpiler coverage >= 80%
- [ ] Type inference coverage >= 90%
- [ ] Statement transpilation >= 85%
- [ ] All edge cases covered
- [ ] Doctests for all public functions

**Focus Areas**:
```rust
// Priority coverage targets:
- transpile_function() - Currently 45%
- infer_param_types() - Currently 0% (NEW)
- transpile_main() - Currently 0% (CRITICAL)
- type_inference module - Currently 20%
```

### RECOVERY-004: Example Programs Suite [4 hours]
**Objective**: Create comprehensive examples demonstrating all patterns

**Acceptance Criteria**:
- [ ] 20+ example programs in examples/
- [ ] All examples compile and run
- [ ] Higher-order function examples
- [ ] CI runs "cargo run --examples"
- [ ] Examples match book patterns

**Examples to Create**:
```
examples/
├── higher_order/
│   ├── apply.ruchy
│   ├── map.ruchy
│   ├── filter.ruchy
│   ├── reduce.ruchy
│   └── compose.ruchy
├── basics/
│   ├── hello_world.ruchy
│   ├── functions.ruchy
│   └── variables.ruchy
└── advanced/
    ├── currying.ruchy
    ├── closures.ruchy
    └── recursion.ruchy
```

### RECOVERY-005: PMAT Complexity Analysis [2 hours]
**Objective**: Measure and reduce code complexity

**Acceptance Criteria**:
- [ ] Run PMAT on entire codebase
- [ ] Document complexity hotspots
- [ ] Refactor functions > 10 complexity
- [ ] Achieve average complexity < 5
- [ ] Zero functions > 15 complexity

**Commands**:
```bash
pmat agent analyze --max-complexity 10
pmat agent auto-fix --target src/backend/transpiler/
```

### RECOVERY-006: Integration Test Suite [4 hours]
**Objective**: End-to-end testing with compilation

**Acceptance Criteria**:
- [ ] Test complete compilation pipeline
- [ ] Test all ruchy-book examples
- [ ] Test with ruchy-repl-demos
- [ ] Performance benchmarks
- [ ] Memory usage validation

### RECOVERY-007: Fix Implementation (Test-First) [6 hours]
**Objective**: Implement BUG-002 fix with test-first approach

**Acceptance Criteria**:
- [ ] Write failing tests first
- [ ] Implement minimal fix
- [ ] All tests pass
- [ ] No regression in existing tests
- [ ] Code review completed

**Test-First Steps**:
1. Write test: `test_higher_order_function_compilation()`
2. Write test: `test_main_has_no_return_type()`
3. Write test: `test_string_params_work()`
4. Implement fix to pass tests
5. Verify no regressions

### RECOVERY-008: Quality Gates Optimization [2 hours]
**Objective**: Make quality gates fast enough to never bypass

**Acceptance Criteria**:
- [ ] Pre-commit hooks < 30 seconds
- [ ] Parallel test execution
- [ ] Incremental testing
- [ ] Skip unchanged code
- [ ] Clear progress indicators

### RECOVERY-009: Release Automation [3 hours]
**Objective**: Automated release pipeline preventing bad releases

**Acceptance Criteria**:
- [ ] GitHub Actions CI/CD pipeline
- [ ] Automated testing before publish
- [ ] Canary deployment to test crate
- [ ] Rollback mechanism
- [ ] Release notes generation

### RECOVERY-010: Documentation & Postmortem [2 hours]
**Objective**: Document lessons learned and prevent recurrence

**Acceptance Criteria**:
- [ ] Postmortem document published
- [ ] CLAUDE.md updated with new rules
- [ ] Testing guide created
- [ ] Release checklist documented
- [ ] Team training on Toyota Way

## 📊 Sprint Metrics

### Quality Metrics
- **Current Coverage**: 33.52%
- **Target Coverage**: 80%
- **Current Complexity**: Avg 12, Max 138
- **Target Complexity**: Avg 5, Max 10

### Testing Metrics  
- **Property Tests**: 0 → 100+
- **Fuzz Targets**: 0 → 10+
- **Examples**: 0 → 20+
- **Integration Tests**: 5 → 50+

### Process Metrics
- **Quality Gate Time**: 2+ min → <30 sec
- **Release Confidence**: Low → High
- **Defect Escape Rate**: 100% → <5%

## 🚀 Definition of Done

1. ✅ All acceptance criteria met
2. ✅ 80% test coverage achieved
3. ✅ All property tests passing
4. ✅ 1M+ fuzz iterations without failure
5. ✅ All examples compile and run
6. ✅ PMAT complexity targets met
7. ✅ Quality gates < 30 seconds
8. ✅ Code review completed
9. ✅ Documentation updated
10. ✅ No regression in ruchy-book

## 📅 Sprint Schedule

**Day 1**:
- Morning: RECOVERY-001 (Property Tests)
- Afternoon: RECOVERY-002 (Fuzz Tests)

**Day 2**:
- Morning: RECOVERY-003 (Unit Coverage)
- Afternoon: RECOVERY-004 (Examples) + RECOVERY-005 (PMAT)

**Day 3**:
- Morning: RECOVERY-006 (Integration) + RECOVERY-007 (Fix)
- Afternoon: RECOVERY-008 (Gates) + RECOVERY-009 (Automation)
- Evening: RECOVERY-010 (Documentation)

## 🎯 Success Criteria

**Sprint succeeds when**:
1. Higher-order functions work correctly
2. No regression in existing functionality  
3. 80% test coverage achieved
4. All quality gates passing
5. Automated release pipeline working
6. Team trained on new process

---

**Toyota Way Commitment**: We will NEVER again bypass quality gates. Quality is built IN, not bolted ON.