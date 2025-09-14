# Ruchy Test Coverage Roadmap 2025
## Systematic 5% Sprint Plan to 80% Coverage

**Start Date**: 2025-01-14  
**Current Coverage**: 41.68% (29,071 / 49,818 lines)  
**Target Coverage**: 80.00%  
**Sprint Size**: 5% coverage increase per sprint  
**Total Sprints**: 8 sprints  
**Estimated Completion**: March 11, 2025 (8 weeks)  

## 🎯 Quality Requirements (MANDATORY - ALL SPRINTS)

### PMAT TDG A+ Standards
- **Cyclomatic Complexity**: ≤10 per function (HARD LIMIT)
- **Cognitive Complexity**: ≤10 per function  
- **Function Size**: ≤30 lines
- **TDG Score**: A+ (≥95 points) for all new code
- **Zero SATD**: No TODO/FIXME/HACK comments
- **Documentation**: 100% for new public APIs

### TDD Protocol (Toyota Way)
1. **RED**: Write failing test FIRST
2. **GREEN**: Minimal code to pass test
3. **REFACTOR**: Improve while maintaining ≤10 complexity
4. **VERIFY**: Run `pmat tdg` to ensure A+ grade
5. **COMMIT**: Only if all quality gates pass

### Pre-Sprint Checklist
```bash
# MANDATORY before starting any sprint
pmat tdg . --min-grade A- --fail-on-violation
cargo test --lib  # Must be 100% passing
cargo clippy -- -D warnings  # Zero warnings
```

---

## 📊 Sprint Overview

| Sprint | Target Coverage | Lines to Cover | Focus Areas | Status |
|--------|----------------|----------------|-------------|--------|
| Sprint 1 | 45% → 47% | +995 lines | Runtime/REPL core functions | 🔴 Not Started |
| Sprint 2 | 47% → 50% | +1,493 lines | Parser/Lexer fundamentals | 🔴 Not Started |
| Sprint 3 | 50% → 55% | +2,491 lines | Transpiler expressions | 🔴 Not Started |
| Sprint 4 | 55% → 60% | +2,491 lines | Type inference/checking | 🔴 Not Started |
| Sprint 5 | 60% → 65% | +2,491 lines | Actor system/concurrency | 🔴 Not Started |
| Sprint 6 | 65% → 70% | +2,491 lines | WASM/Notebook runtime | 🔴 Not Started |
| Sprint 7 | 70% → 75% | +2,491 lines | Error handling/recovery | 🔴 Not Started |
| Sprint 8 | 75% → 80% | +2,491 lines | Integration/E2E tests | 🔴 Not Started |

---

## 🚀 SPRINT 1: Runtime/REPL Core (41.68% → 47%)
**Target**: +2,633 lines covered  
**Deadline**: 2025-01-21 (1 week)

### Priority Modules (Current → Target)
```
runtime/repl.rs: 0.00% → 60% (+4,268 lines) [CRITICAL]
runtime/interpreter.rs: 11.84% → 80% (+1,725 lines)
runtime/magic.rs: 0.00% → 70% (+288 lines)
runtime/completion.rs: 0.00% → 60% (+251 lines)
```

### Test Requirements
- [ ] **REPL-001**: Core REPL loop with 50+ test cases
- [ ] **REPL-002**: Command parsing and execution
- [ ] **REPL-003**: State management and recovery
- [ ] **REPL-004**: Tab completion functionality
- [ ] **REPL-005**: Magic commands (!help, !clear, etc.)
- [ ] **INTERP-001**: Expression evaluation (all types)
- [ ] **INTERP-002**: Stack operations
- [ ] **INTERP-003**: Error handling paths
- [ ] **PROP-001**: Property tests with 10,000+ iterations

### Sprint Execution Plan
```bash
# Day 1-2: REPL Core
cd tests/
touch repl_core_tdd.rs
# Write 50+ tests for REPL state machine

# Day 3-4: Interpreter Coverage
touch interpreter_tdd.rs
# Focus on uncovered evaluation paths

# Day 5: Property Testing
# Add proptest for all new functions

# Day 6: Quality Verification
pmat tdg runtime/ --min-grade A+
cargo llvm-cov --lib | grep runtime/

# Day 7: Documentation & Review
# Add doctests for all public APIs
```

---

## 🚀 SPRINT 2: Parser/Lexer Core (47% → 50%)
**Target**: +1,493 lines covered  
**Deadline**: 2025-01-28 (1 week)

### Priority Modules
```
frontend/lexer.rs: 38.40% → 90% (+64 lines)
frontend/parser/expressions.rs: 10.51% → 70% (+1,002 lines)
frontend/parser/mod.rs: 42.50% → 85% (+170 lines)
frontend/parser/utils.rs: 1.21% → 60% (+387 lines)
```

### Test Requirements
- [ ] **LEX-001**: All token types with edge cases
- [ ] **LEX-002**: String/char literal parsing
- [ ] **LEX-003**: Number parsing (int/float/scientific)
- [ ] **PARSE-001**: All expression types
- [ ] **PARSE-002**: Operator precedence
- [ ] **PARSE-003**: Error recovery mechanisms
- [ ] **PROP-002**: Fuzz testing with random input

---

## 🚀 SPRINT 3: Transpiler Core (50% → 55%)
**Target**: +2,491 lines covered  
**Deadline**: 2025-02-04 (1 week)

### Priority Modules
```
backend/transpiler/statements.rs: 2.20% → 70% (+959 lines)
backend/transpiler/expressions.rs: 21.99% → 80% (+222 lines)
backend/transpiler/codegen_minimal.rs: 0.00% → 70% (+174 lines)
backend/transpiler/mod.rs: 5.35% → 70% (+315 lines)
```

### Test Requirements
- [ ] **TRANS-001**: Statement transpilation (all types)
- [ ] **TRANS-002**: Expression transpilation
- [ ] **TRANS-003**: Type conversions
- [ ] **TRANS-004**: Pattern matching codegen
- [ ] **PROP-003**: Roundtrip transpilation tests

---

## 🚀 SPRINT 4: Type System (55% → 60%)
**Target**: +2,491 lines covered  
**Deadline**: 2025-02-11 (1 week)

### Priority Modules
```
middleend/infer.rs: 0.00% → 60% (+586 lines)
middleend/types.rs: 0.00% → 70% (+95 lines)
middleend/unify.rs: 0.00% → 70% (+59 lines)
backend/transpiler/type_inference.rs: 0.00% → 60% (+86 lines)
```

### Test Requirements
- [ ] **TYPE-001**: Type inference for all expressions
- [ ] **TYPE-002**: Generic type handling
- [ ] **TYPE-003**: Type unification
- [ ] **TYPE-004**: Error messages for type mismatches

---

## 🚀 SPRINT 5: Actor System (60% → 65%)
**Target**: +2,491 lines covered  
**Deadline**: 2025-02-18 (1 week)

### Priority Modules
```
runtime/actor.rs: 0.00% → 70% (+181 lines)
actors.rs: 0.00% → 60% (if feature enabled)
runtime/transaction.rs: 0.00% → 60% (+100 lines)
```

### Test Requirements
- [ ] **ACTOR-001**: Message passing
- [ ] **ACTOR-002**: Supervision trees
- [ ] **ACTOR-003**: Fault tolerance
- [ ] **ACTOR-004**: Concurrent execution

---

## 🚀 SPRINT 6: WASM/Notebook (65% → 70%)
**Target**: +2,491 lines covered  
**Deadline**: 2025-02-25 (1 week)

### Priority Modules
```
wasm/notebook.rs: 0.00% → 50% (+961 lines)
wasm/shared_session.rs: 0.00% → 60% (+266 lines)
wasm/repl.rs: 0.00% → 70% (+71 lines)
```

### Test Requirements
- [ ] **WASM-001**: Cell execution
- [ ] **WASM-002**: State management
- [ ] **WASM-003**: Output formatting
- [ ] **WASM-004**: Error handling

---

## 🚀 SPRINT 7: Error Recovery (70% → 75%)
**Target**: +2,491 lines covered  
**Deadline**: 2025-03-04 (1 week)

### Priority Modules
```
frontend/error_recovery.rs: 0.00% → 70% (+362 lines)
parser/error_recovery.rs: 10.88% → 70% (+114 lines)
frontend/diagnostics.rs: 0.00% → 60% (+101 lines)
```

### Test Requirements
- [ ] **ERROR-001**: Parse error recovery
- [ ] **ERROR-002**: Runtime error handling
- [ ] **ERROR-003**: Diagnostic messages
- [ ] **ERROR-004**: Error propagation

---

## 🚀 SPRINT 8: Integration/E2E (75% → 80%)
**Target**: +2,491 lines covered  
**Deadline**: 2025-03-11 (1 week)

### Priority Modules
```
Full pipeline integration tests
End-to-end compilation tests
Cross-module interaction tests
Performance benchmarks
```

### Test Requirements
- [ ] **E2E-001**: Full compilation pipeline
- [ ] **E2E-002**: REPL session workflows
- [ ] **E2E-003**: Module system
- [ ] **E2E-004**: Real-world programs

---

## 📈 Tracking & Metrics

### Weekly Metrics Collection
```bash
# Run every Friday
echo "Week $(date +%V) Coverage Report" >> coverage-log.md
cargo llvm-cov --lib | grep "^TOTAL" >> coverage-log.md
pmat tdg . --format=json > tdg-week-$(date +%V).json
git commit -m "[COVERAGE] Week $(date +%V): $(cargo llvm-cov --lib | grep '^TOTAL' | awk '{print $10}')"
```

### Success Criteria
- ✅ Coverage increases by ≥5% each sprint
- ✅ All tests pass (zero failures)
- ✅ TDG score A+ maintained
- ✅ Zero complexity violations
- ✅ 100% documentation for new code

### Risk Mitigation
- **Risk**: Complex modules harder to test
  - **Mitigation**: Break into smaller functions first
- **Risk**: Integration test failures
  - **Mitigation**: Fix before adding new tests
- **Risk**: Performance regression
  - **Mitigation**: Benchmark before/after each sprint

---

## 🏁 Final Validation (Sprint 8 Completion)

### Acceptance Criteria
```bash
# All must pass for project completion
cargo llvm-cov --lib | grep "^TOTAL" # Must show ≥80%
pmat tdg . --min-grade A+ --fail-on-violation # Must pass
cargo test --lib # 100% passing
cargo clippy -- -D warnings # Zero warnings
cargo bench # No performance regression >5%
```

### Celebration Milestone 🎉
Upon reaching 80% coverage with A+ quality:
1. Tag release as `v4.0.0-quality-milestone`
2. Update README with coverage badge
3. Publish comprehensive quality report
4. Share achievement with team

---

## 📝 Notes

- Each sprint is exactly 1 week (5 working days + 2 review days)
- Daily standup: Report lines covered and blockers
- Use `cargo llvm-cov --show-missing` to find untested code
- Pair programming encouraged for complex modules
- Review PR must include coverage delta report

**Remember**: Quality over quantity. Better to cover 4% with excellent tests than 6% with poor tests.