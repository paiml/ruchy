# Test Coverage Gap Analysis - P0 Priority

**Date**: 2025-10-18
**Current Overall Coverage**: 70.34%
**Target**: 80%+
**Gap**: 9.66% improvement needed

## Executive Summary

Analysis identified **95 modules with <70% coverage**. The top 5 modules account for **~10,000 uncovered lines** and represent the highest ROI for coverage improvement:

1. **runtime/interpreter.rs** (24.33%) - 4,470 uncovered lines
2. **runtime/eval_builtin.rs** (16.83%) - 2,071 uncovered lines
3. **runtime/builtins.rs** (27.95%) - 1,253 uncovered lines
4. **quality/formatter.rs** (29.96%) - 1,709 uncovered lines
5. **quality/scoring.rs** (37.34%) - 1,242 uncovered lines

**Strategy**: Focus on Top 5 + Quick Wins to achieve 80%+ coverage in 1 week.

---

## Critical Low-Coverage Modules (Prioritized by Impact)

### TOP 5 HIGHEST-IMPACT TARGETS (Start Here)

#### 1. **runtime/interpreter.rs** - 24.33% coverage (5,907 lines)
   - **Impact**: CRITICAL - Core interpreter, largest file
   - **Current**: 24.33% line coverage, 23.61% branch coverage
   - **Uncovered Lines**: 4,470 of 5,907 lines
   - **Why Priority**: Interpreter is the heart of the runtime system
   - **Effort**: HIGH (large file, complex logic)
   - **ROI**: HIGHEST - Improving this alone could add ~3-5% to overall coverage

#### 2. **runtime/eval_builtin.rs** - 16.83% coverage (2,490 lines)
   - **Impact**: CRITICAL - Built-in function implementations
   - **Current**: 16.83% line coverage, 17.56% branch coverage
   - **Uncovered Lines**: 2,071 of 2,490 lines
   - **Why Priority**: User-facing built-in functions (println, len, etc.)
   - **Effort**: MEDIUM (well-structured, testable functions)
   - **ROI**: VERY HIGH - Each built-in function is independently testable

#### 3. **runtime/builtins.rs** - 27.95% coverage (1,739 lines)
   - **Impact**: HIGH - Built-in types and helpers
   - **Current**: 27.95% line coverage, 17.82% branch coverage
   - **Uncovered Lines**: 1,253 of 1,739 lines
   - **Why Priority**: Core built-in functionality
   - **Effort**: MEDIUM (testable functions)
   - **ROI**: HIGH - ~2% overall coverage gain possible

#### 4. **quality/formatter.rs** - 29.96% coverage (2,440 lines)
   - **Impact**: HIGH - Code formatting tool
   - **Current**: 29.96% line coverage, 37.28% branch coverage
   - **Uncovered Lines**: 1,709 of 2,440 lines
   - **Why Priority**: User-facing tool, quality infrastructure
   - **Effort**: MEDIUM (needs formatting test fixtures)
   - **ROI**: HIGH - ~1.5% overall coverage gain

#### 5. **quality/scoring.rs** - 37.34% coverage (1,982 lines)
   - **Impact**: MEDIUM-HIGH - Quality scoring infrastructure
   - **Current**: 37.34% line coverage, 42.02% branch coverage
   - **Uncovered Lines**: 1,242 of 1,982 lines
   - **Why Priority**: Quality gates, PMAT integration
   - **Effort**: MEDIUM (algorithmic, property-testable)
   - **ROI**: MEDIUM-HIGH - ~1% overall coverage gain

---

## Secondary Targets (After Top 5)

### Runtime System (Additional)
- `runtime/eval_control_flow_new.rs` - 34.27% (499 lines)
- `runtime/eval_function.rs` - 35.63% (508 lines)
- `runtime/eval_method_dispatch.rs` - 32.84% (405 lines)
- `runtime/eval_data_structures.rs` - 41.25% (480 lines)
- `runtime/eval_dataframe.rs` - 51.50% (800 lines)
- `runtime/compilation.rs` - 37.30% (622 lines)

### Standard Library (High User Impact)
- `stdlib/http.rs` - 19.67% (122 lines) - **EASY WIN**
- `stdlib/path.rs` - 20.41% (147 lines) - **EASY WIN**
- `stdlib/fs.rs` - 27.59% (87 lines) - **EASY WIN**
- `stdlib/json.rs` - 51.16% (86 lines)

### LSP & Tooling
- `lsp/analyzer.rs` - 3.14% (159 lines) - **EASY WIN**

---

## Systematic Coverage Improvement Strategy

### Week 1: Focus on Top 5 (Days 1-7)

**Day 1-2**: runtime/interpreter.rs (24.33% → 60%+)
- Create test fixtures for all interpreter paths
- Property tests for expression evaluation
- Test error handling paths
- **Expected Gain**: +3% overall coverage

**Day 3**: runtime/eval_builtin.rs (16.83% → 70%+)
- Test each built-in function individually
- Property tests for mathematical functions
- Boundary condition tests
- **Expected Gain**: +2% overall coverage

**Day 4**: runtime/builtins.rs (27.95% → 70%+)
- Test built-in type implementations
- Test type coercion paths
- **Expected Gain**: +1.5% overall coverage

**Day 5**: quality/formatter.rs (29.96% → 70%+)
- Create formatting test fixtures
- Test all AST node formatting
- **Expected Gain**: +1.5% overall coverage

**Day 6**: quality/scoring.rs (37.34% → 70%+)
- Test scoring algorithms
- Property tests for metrics
- **Expected Gain**: +1% overall coverage

**Day 7**: Validation & Commit
- Run full test suite
- Verify 80%+ overall coverage achieved
- Commit improvements

---

### Quick Wins (Parallel Track)

**Easy Targets** (can be done in parallel):
1. stdlib/http.rs (19.67% → 80%+) - 30 min
2. stdlib/path.rs (20.41% → 80%+) - 30 min
3. stdlib/fs.rs (27.59% → 80%+) - 30 min
4. lsp/analyzer.rs (3.14% → 70%+) - 1 hour

**Total Quick Wins**: ~0.5% overall coverage in 2-3 hours

---

## Test Strategy (EXTREME TDD)

### For Each Module:

1. **RED Phase**: Write failing tests first
   - Unit tests for each function
   - Property tests for invariants
   - Boundary condition tests
   - Error path tests

2. **GREEN Phase**: Run tests, verify they pass
   - Already have implementation
   - Just need to exercise existing code paths

3. **REFACTOR Phase**: Simplify complex code if needed
   - Extract functions with >10 complexity
   - Add helper functions
   - Improve testability

### Property Test Targets:
- Mathematical functions: `pow(a, pow(b, c)) == pow(a, b*c)`
- String operations: `len(s1 + s2) == len(s1) + len(s2)`
- Collection operations: `filter(xs, p).all(p)`
- Formatting: `format(parse(code)) == format(code)` (idempotent)

---

## Success Metrics

### Target State (End of Week 1):
- ✅ Overall coverage: 70.34% → 80%+
- ✅ runtime/interpreter.rs: 24.33% → 60%+
- ✅ runtime/eval_builtin.rs: 16.83% → 70%+
- ✅ runtime/builtins.rs: 27.95% → 70%+
- ✅ quality/formatter.rs: 29.96% → 70%+
- ✅ quality/scoring.rs: 37.34% → 70%+
- ✅ All stdlib modules: >70%
- ✅ lsp/analyzer.rs: >70%

### Verification:
```bash
make coverage
cargo llvm-cov report | grep "TOTAL"
# Should show ≥80% overall coverage
```

---

## Next Steps (Immediate Actions)

**START WITH QUICK WINS** (2-3 hours):
1. stdlib/http.rs - Add HTTP client tests
2. stdlib/path.rs - Add path manipulation tests
3. stdlib/fs.rs - Add file I/O tests
4. lsp/analyzer.rs - Add LSP analysis tests

**THEN TACKLE TOP 5** (Days 1-6):
1. runtime/interpreter.rs - Core interpreter testing
2. runtime/eval_builtin.rs - Built-in function testing
3. runtime/builtins.rs - Built-in type testing
4. quality/formatter.rs - Formatter testing
5. quality/scoring.rs - Scoring algorithm testing

---

## All Low-Coverage Modules (Complete List)

| Module | Coverage | Lines | Uncovered | Priority |
|--------|----------|-------|-----------|----------|
| runtime/interpreter.rs | 24.33% | 5,907 | 4,470 | P0 |
| runtime/eval_builtin.rs | 16.83% | 2,490 | 2,071 | P0 |
| runtime/builtins.rs | 27.95% | 1,739 | 1,253 | P0 |
| quality/formatter.rs | 29.96% | 2,440 | 1,709 | P0 |
| quality/scoring.rs | 37.34% | 1,982 | 1,242 | P0 |
| backend/transpiler/statements.rs | 62.07% | 6,555 | 2,486 | P1 |
| backend/wasm/mod.rs | 65.49% | 2,927 | 1,010 | P1 |
| frontend/parser/collections.rs | 54.49% | 2,028 | 923 | P1 |
| middleend/infer.rs | 50.14% | 3,175 | 1,583 | P1 |
| stdlib/http.rs | 19.67% | 122 | 98 | QUICK WIN |
| stdlib/path.rs | 20.41% | 147 | 117 | QUICK WIN |
| stdlib/fs.rs | 27.59% | 87 | 63 | QUICK WIN |
| lsp/analyzer.rs | 3.14% | 159 | 154 | QUICK WIN |

(95 modules total - see coverage report for complete list)

---

**Analysis Prepared By**: Claude Code (AI Assistant)
**Methodology**: EXTREME TDD + Evidence-Based Prioritization
**Date**: 2025-10-18
**Ruchy Version**: v3.91.0
