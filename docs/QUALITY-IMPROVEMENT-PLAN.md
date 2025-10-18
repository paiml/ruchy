# Quality Improvement Plan - Ruchy v3.91.0

**Date**: 2025-10-18
**Current Version**: 3.91.0
**Methodology**: Toyota Way + EXTREME TDD + Evidence-Based Prioritization

---

## Executive Summary

**Current Quality Status**: **B+ (Good, Not Excellent)**

**Overall Score**: 87.6/100 (Parser), 70.34% coverage, 3,849 tests

**Goal**: Achieve **A+ Quality** (TDG ≥90/100, Coverage ≥80%, Zero defects)

**Timeline**: 4-6 weeks for A+ quality across all components

---

## Current State Assessment

### ✅ Strengths (Keep Doing)

1. **Parser Quality** - **EXCELLENT** (87.6/100, A-)
   - 26 modular files (91.6% file reduction)
   - EXTREME TDD methodology
   - Property + mutation testing
   - Binary operators module: 97.2/100 (A+) ⭐

2. **Testing Infrastructure** - **EXCELLENT**
   - 3,849 tests passing (100% success rate)
   - Property testing (10K+ iterations)
   - Mutation testing (≥75% target)
   - Book validation (4 chapters on every commit)

3. **Quality Gates** - **EXCELLENT**
   - Pre-commit hooks active
   - ruchy-book validation
   - Coverage regression prevention
   - Toyota Way principles enforced

### ⚠️ Weaknesses (Must Improve)

1. **Test Coverage** - **70.34%** (target: 80%+)
   - Gap: 9.66% to reach target
   - Some modules <60% coverage
   - Need systematic coverage improvement

2. **handlers/mod.rs** - **68.9/100 (C+)**
   - File size: 2,843 lines (too large)
   - Needs modularization (like expressions.rs)
   - Complexity issues

3. **Book Compatibility** - **19%** (49/259 examples)
   - Target: 100% compatibility
   - Gap: 210 examples need fixing
   - Blocking user adoption

4. **Interpreter Complexity** - **HIGH**
   - evaluate_expr: 138 (target <50)
   - Value::fmt: 66 (target <30)
   - Value::format_dataframe: 69 (target <30)

5. **Documentation** - **65% (C+)**
   - No rustdoc (API undocumented)
   - Book compatibility low
   - Missing migration guides

---

## Prioritized Improvement Plan

### Priority Matrix

| Task | Impact | Effort | Priority | Timeline |
|------|--------|--------|----------|----------|
| **Fix Test Coverage →80%** | HIGH | MEDIUM | **P0** | 1-2 weeks |
| **handlers/mod.rs Modularization** | HIGH | HIGH | **P1** | 2-3 weeks |
| **Book Compatibility →100%** | MEDIUM | HIGH | **P1** | 3-4 weeks |
| **Interpreter Complexity** | MEDIUM | HIGH | **P2** | 3-4 weeks |
| **rustdoc API Docs** | MEDIUM | MEDIUM | **P2** | 1-2 weeks |
| **Mutation Coverage ≥75%** | LOW | MEDIUM | **P3** | 2-3 weeks |

---

## P0: Test Coverage →80% (1-2 weeks)

### Current Status
- **Overall**: 70.34%
- **Target**: 80%+
- **Gap**: 9.66%

### Strategy: **Systematic Gap Filling**

**Step 1**: Identify Low-Coverage Modules (Day 1)
```bash
make coverage
# Find modules <70%
cargo llvm-cov report | grep -E " [0-6][0-9]\.[0-9]" | sort -k3
```

**Step 2**: TDD Coverage Improvement (Days 2-7)
For each low-coverage module:
1. **RED**: Write tests for uncovered lines
2. **GREEN**: Achieve ≥80% coverage per module
3. **REFACTOR**: Simplify complex code if needed

**Step 3**: Property Test Addition (Days 8-10)
Add property tests to high-risk modules:
- Type system (inference, checking)
- Code generation (transpiler)
- Runtime (evaluation)

**Success Criteria**:
- ✅ Overall coverage ≥80%
- ✅ All critical modules ≥80%
- ✅ Property tests for invariants
- ✅ Mutation coverage ≥75%

**Methodology**: EXTREME TDD
- Write test FIRST
- Prove coverage improvement
- Property tests for mathematical correctness

---

## P1: handlers/mod.rs Modularization (2-3 weeks)

### Current Status
- **File size**: 2,843 lines
- **TDG**: 68.9/100 (C+)
- **Functions**: 100+ functions

### Goal: **TDG ≥85/100 (A-)**

### Strategy: **Follow expressions.rs Pattern**

**Phase 1**: Analysis (Days 1-2)
```bash
# Analyze complexity
pmat analyze complexity src/bin/handlers/mod.rs

# Identify extraction candidates
# Target: 5-10 modules, ~400-600 lines each
```

**Phase 2**: Modularization (Days 3-12)
Extract into focused modules (similar to expressions.rs success):
1. **check_handler** (~300 lines)
2. **transpile_handler** (~400 lines)
3. **lint_handler** (~300 lines)
4. **compile_handler** (~400 lines)
5. **run_handler** (~300 lines)
6. **coverage_handler** (~300 lines)
7. **wasm_handler** (~300 lines)
8. **notebook_handler** (~400 lines)

**Phase 3**: Testing (Days 13-15)
- Extract tests (handlers/tests.rs already exists)
- Add property tests
- Verify TDG ≥85/100

**Success Criteria**:
- ✅ Main file <600 lines
- ✅ TDG ≥85/100 (A-)
- ✅ Each module ≤10 complexity
- ✅ 100% test passing (no regressions)

**Methodology**: EXTREME TDD
- RED→GREEN→REFACTOR for each module
- Property tests for command handlers
- Integration tests for CLI contract

---

## P1: Book Compatibility →100% (3-4 weeks)

### Current Status
- **Compatibility**: 19% (49/259 examples)
- **Gap**: 210 examples broken
- **Blocking**: User adoption

### Goal: **100% Compatibility**

### Strategy: **Systematic Example Fixing**

**Phase 1**: Categorize Failures (Week 1)
```bash
# Run all examples, categorize failures
for file in ../ruchy-book/examples/**/*.ruchy; do
  ruchy run "$file" 2>&1 | tee -a failures.log
done

# Categorize:
# - Parser errors (30-40%)
# - Runtime errors (30-40%)
# - Missing features (20-30%)
```

**Phase 2**: Fix by Category (Weeks 2-3)
1. **Parser Errors** (50-80 examples)
   - Fix grammar issues
   - Add missing syntax support
   - Test with property tests

2. **Runtime Errors** (50-80 examples)
   - Fix evaluation bugs
   - Add missing built-ins
   - Improve error messages

3. **Missing Features** (40-60 examples)
   - Implement missing features
   - Add to language completeness tests
   - Document in LANG-COMP

**Phase 3**: Validation (Week 4)
- Run full compatibility suite
- Add to pre-commit validation
- Update book validation script to test ALL chapters

**Success Criteria**:
- ✅ 100% compatibility (259/259)
- ✅ All examples in pre-commit validation
- ✅ Zero regressions on commits
- ✅ Book updated with working examples

**Methodology**: EXTREME TDD
- Fix bugs with RED→GREEN→REFACTOR
- Add regression tests for each fix
- Property tests for language features

---

## P2: Interpreter Complexity Reduction (3-4 weeks)

### Current Status
- **evaluate_expr**: 138 (target <50)
- **Value::fmt**: 66 (target <30)
- **Value::format_dataframe**: 69 (target <30)

### Goal: **Complexity ≤50 per function**

### Strategy: **Extract Method + Visitor Pattern**

**Phase 1**: evaluate_expr Reduction (Weeks 1-2)
Current pattern:
```rust
// ❌ BAD: Massive match with 138 complexity
fn evaluate_expr(expr: &Expr) -> Result<Value> {
    match &expr.kind {
        ExprKind::Literal => { ... }
        ExprKind::BinaryOp => { ... }
        // ... 80+ more arms
    }
}
```

Target pattern:
```rust
// ✅ GOOD: Visitor pattern with delegation
fn evaluate_expr(expr: &Expr) -> Result<Value> {
    match &expr.kind {
        ExprKind::Literal(lit) => evaluate_literal(lit),
        ExprKind::BinaryOp(op, l, r) => evaluate_binary_op(op, l, r),
        // Delegate to focused functions (complexity ≤10 each)
    }
}

fn evaluate_literal(lit: &Literal) -> Result<Value> { ... }
fn evaluate_binary_op(op: &BinaryOp, l: &Expr, r: &Expr) -> Result<Value> { ... }
```

**Phase 2**: Value::fmt Reduction (Week 3)
Extract formatting logic into:
- format_number()
- format_string()
- format_collection()
- format_dataframe()
- format_object()

**Phase 3**: Validation (Week 4)
- Run PMAT complexity analysis
- Verify all functions ≤50 complexity
- Run full test suite
- Check mutation coverage

**Success Criteria**:
- ✅ evaluate_expr ≤50 complexity
- ✅ Value::fmt ≤30 complexity
- ✅ All evaluation functions ≤10 complexity
- ✅ 100% tests passing
- ✅ Property tests for evaluation invariants

---

## P2: rustdoc API Documentation (1-2 weeks)

### Current Status
- **rustdoc**: 0% (no docs)
- **Impact**: Hard to use as library

### Goal: **100% Public API Documented**

### Strategy: **Document-First Development**

**Phase 1**: Public API Identification (Days 1-2)
```bash
# Find all public items
grep -r "^pub fn\|^pub struct\|^pub enum" src/lib.rs src/**/*.rs | wc -l

# Categorize:
# - Core types (Expr, Type, Value)
# - Parser API
# - Transpiler API
# - Runtime API
# - WASM bindings
```

**Phase 2**: Documentation Writing (Days 3-10)
For each public API:
```rust
/// Evaluates a Ruchy expression and returns a value.
///
/// # Examples
///
/// ```
/// use ruchy::evaluate;
/// let result = evaluate("1 + 2").unwrap();
/// assert_eq!(result, Value::Integer(3));
/// ```
///
/// # Errors
///
/// Returns `Err` if:
/// - Expression is syntactically invalid
/// - Runtime error occurs during evaluation
/// - Type mismatch detected
///
/// # Panics
///
/// Never panics. All errors returned via `Result`.
pub fn evaluate(code: &str) -> Result<Value> { ... }
```

**Phase 3**: Validation (Days 11-14)
```bash
# Generate docs
cargo doc --no-deps --all-features

# Run doc tests
cargo test --doc

# Check coverage
cargo rustdoc -- -Z unstable-options --show-coverage
```

**Success Criteria**:
- ✅ 100% public API documented
- ✅ Examples in every doc comment
- ✅ All doc tests passing
- ✅ cargo doc generates clean docs

---

## P3: Mutation Coverage ≥75% (2-3 weeks)

### Current Status
- **Mutation testing**: Ad-hoc (Sprint 8)
- **Coverage**: Unknown overall
- **Target**: ≥75% mutation score

### Goal: **Systematic Mutation Testing**

### Strategy: **Incremental File-by-File**

**Phase 1**: Baseline (Week 1)
```bash
# Run mutation tests on critical modules
cargo mutants --file src/frontend/parser/expressions.rs --timeout 300
cargo mutants --file src/interpreter/eval.rs --timeout 300
cargo mutants --file src/transpiler/core.rs --timeout 300

# Record baseline scores
```

**Phase 2**: Gap Filling (Week 2)
For modules <75%:
1. Analyze MISSED mutations
2. Add targeted tests
3. Re-run mutation tests
4. Repeat until ≥75%

**Phase 3**: Automation (Week 3)
```bash
# Add to Makefile
make mutation-test-all:
    cargo mutants --workspace --timeout 300

# Add to CI (optional, slow)
```

**Success Criteria**:
- ✅ ≥75% mutation score overall
- ✅ Critical modules ≥80%
- ✅ Mutation testing in CI/CD
- ✅ Zero uncaught regressions

---

## Implementation Timeline

### Week 1-2: P0 Coverage Improvement
- **Goal**: 70.34% → 80%+
- **Methodology**: TDD gap filling
- **Deliverable**: 80%+ coverage report

### Week 3-5: P1 handlers/mod.rs Modularization
- **Goal**: 68.9 → 85+ TDG
- **Methodology**: expressions.rs pattern
- **Deliverable**: Modularized handlers with A- grade

### Week 4-7: P1 Book Compatibility
- **Goal**: 19% → 100%
- **Methodology**: Systematic example fixing
- **Deliverable**: All 259 examples working

### Week 6-9: P2 Interpreter Complexity
- **Goal**: evaluate_expr 138 → <50
- **Methodology**: Extract method + visitor
- **Deliverable**: Simplified interpreter

### Week 8-9: P2 rustdoc Documentation
- **Goal**: 0% → 100% public API
- **Methodology**: Document-first
- **Deliverable**: Complete API docs

### Week 10-12: P3 Mutation Coverage
- **Goal**: Unknown → ≥75%
- **Methodology**: Incremental testing
- **Deliverable**: Mutation test suite

**Total**: 12 weeks to A+ quality

---

## Success Metrics

### Target State (12 weeks)

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| **TDG Score** | 87.6/100 | ≥90/100 | ⚠️ Need 2.4 points |
| **Test Coverage** | 70.34% | ≥80% | ⚠️ Need 9.66% |
| **handlers/mod.rs** | 68.9/100 | ≥85/100 | ❌ Need 16.1 points |
| **Book Compat** | 19% | 100% | ❌ Need 210 examples |
| **evaluate_expr** | 138 | <50 | ❌ Need 88 reduction |
| **rustdoc** | 0% | 100% | ❌ Need all docs |
| **Mutation Score** | Unknown | ≥75% | ⚠️ Need baseline |

### Quality Gates (Enforced)

1. **Pre-commit**:
   - ✅ Coverage ≥80% (no regressions)
   - ✅ TDG ≥85/100 for new code
   - ✅ Complexity ≤10 for new functions
   - ✅ Book validation passes (all chapters)
   - ✅ All tests passing

2. **Pre-release**:
   - ✅ Mutation score ≥75%
   - ✅ All examples working
   - ✅ rustdoc complete
   - ✅ Zero high-severity bugs

---

## Next Steps (Immediate Actions)

### This Week (P0 - Coverage)

**Day 1-2**: Identify Gaps
```bash
make coverage
cargo llvm-cov report > coverage-analysis.txt
grep -E " [0-6][0-9]\.[0-9]" coverage-analysis.txt > low-coverage.txt
```

**Day 3-5**: Fix Top 5 Modules
- Write tests for uncovered lines
- Add property tests
- Verify coverage improvement

**Weekend**: Validation
- Run full coverage
- Verify ≥80% overall
- Commit improvements

### Next Week (P1 - handlers/mod.rs)

**Monday-Tuesday**: Analysis
- PMAT complexity analysis
- Identify extraction candidates
- Design module structure

**Wednesday-Friday**: Extraction Phase 1
- Extract check_handler module
- Extract transpile_handler module
- Add tests, verify TDG improvement

---

## Conclusion

**Ruchy is at B+ quality (Good), targeting A+ quality (Excellent).**

**Key Focus Areas**:
1. ✅ **P0**: Test coverage 70.34% → 80%+ (1-2 weeks)
2. ✅ **P1**: handlers/mod.rs modularization (2-3 weeks)
3. ✅ **P1**: Book compatibility 19% → 100% (3-4 weeks)
4. ⚠️ **P2**: Interpreter complexity reduction (3-4 weeks)
5. ⚠️ **P2**: rustdoc API documentation (1-2 weeks)
6. ⚠️ **P3**: Mutation coverage ≥75% (2-3 weeks)

**Timeline**: 12 weeks to A+ quality

**Methodology**: Toyota Way + EXTREME TDD + Evidence-Based Prioritization

---

**Plan Owner**: Development Team
**Approval Date**: 2025-10-18
**Review Cadence**: Weekly (every Monday)
**Next Review**: 2025-10-25
