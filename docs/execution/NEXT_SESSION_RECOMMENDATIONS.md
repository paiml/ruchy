# Next Session Recommendations

**Date**: 2025-10-05
**Current Status**: Sprint 9 Phase 1 COMPLETE, Phase 2 IN PROGRESS
**Session Type**: Continuation

---

## Priority 1: Continue Sprint 9 Phase 2 (Runtime Modules)

### Immediate Tasks

1. **Fix deterministic.rs gaps** (290 lines)
   - 12 MISSED mutations identified
   - Patterns: 3 stubs, 6 arithmetic, 2 comparison, 1 boolean
   - Estimated effort: 6-8 targeted tests
   - File: `src/runtime/deterministic.rs`
   - Baseline: `deterministic_mutation_baseline.txt` (already generated)

2. **Test eval_array.rs** (291 lines)
   - Known gaps from earlier analysis
   - Baseline-driven approach recommended
   - Estimated effort: 5-10 tests

3. **Test eval_string.rs** (296 lines)
   - Expected patterns: match arms, negation operators
   - Baseline-driven approach required
   - Estimated effort: 5-10 tests

4. **Test actor_runtime.rs** (313 lines)
   - Core async functionality
   - May have timing-related mutations
   - Estimated effort: 5-10 tests

### Week 2 Target

**Goal**: Complete 12-15 medium files (200-400 lines) at 80%+ mutation coverage

**Remaining Files (after deterministic.rs, eval_array.rs, eval_string.rs, actor_runtime.rs)**:
- object_helpers.rs (321 lines)
- builtin_init.rs (324 lines)
- cache.rs (330 lines)
- eval_expr.rs (382 lines)
- resource_eval.rs (393 lines)
- safe_arena.rs (405 lines)
- eval_string_methods.rs (418 lines)
- eval_pattern.rs (421 lines)
- eval_loops.rs (424 lines)
- eval_method_dispatch.rs (425 lines)
- inspect.rs (456 lines)

**Strategy**:
- Use baseline-driven approach for all medium files
- Target 5-10 tests per file
- Focus on Sprint 8 patterns (especially Pattern #3 - negation operators)
- Maintain zero test regressions

---

## Priority 2: Parser Regression Analysis (Optional/Secondary)

### Finding

Background mutation test on parser modules revealed **29 MISSED mutations** across parser files:

**Affected Files**:
- `src/frontend/parser/mod.rs` (5 MISSED)
- `src/frontend/parser/expressions.rs` (11 MISSED)
- `src/frontend/parser/collections.rs` (5 MISSED)
- `src/frontend/parser/utils.rs` (3 MISSED)
- `src/frontend/parser/operator_precedence.rs` (4 MISSED)
- `src/frontend/parser/imports.rs` (1 MISSED)

**Patterns**:
- Match arm deletions (9 instances)
- Negation operators (5 instances)
- Function stubs (4 instances)
- Arithmetic operators (3 instances)
- Comparison operators (3 instances)
- Match guard mutations (2 instances)
- Function replacement with () (3 instances)

### Recommendation

**Option A - Immediate (if time permits)**:
- Address these 29 gaps to achieve 100% parser mutation coverage
- Estimated effort: 15-20 tests
- Would complete Sprint 8's deferred items

**Option B - Deferred (recommended)**:
- Continue Sprint 9 focus on runtime modules
- Create "Sprint 8.5" ticket for parser regression fixes
- Address after Sprint 9 completion

**Rationale**: Sprint 9 is the current priority. Parser gaps can be addressed in a dedicated follow-up sprint.

---

## Priority 3: Sprint 9 Documentation Maintenance

### Ongoing Tasks

1. **Update SPRINT_9_PHASE2_PROGRESS.md** after each file completion
2. **Track mutation coverage** per file in progress document
3. **Document new patterns** if any emerge beyond Sprint 8's 5 patterns
4. **Update roadmap** weekly with Sprint 9 progress

### Quality Gates

- ✅ Zero test regressions (3491/3491 tests passing)
- ✅ 80%+ mutation coverage per file
- ✅ Systematic documentation of all patterns
- ✅ Comprehensive baseline tracking

---

## Session Workflow Recommendation

### Start of Session

1. **Review SPRINT_9_PHASE2_PROGRESS.md** for current status
2. **Check roadmap** for latest priorities
3. **Verify all tests passing**: `cargo test --lib --quiet`
4. **Review baseline files** already generated

### During Session

1. **Fix deterministic.rs gaps** (12 mutations, baseline exists)
2. **Test eval_array.rs** (generate baseline, identify gaps, fix)
3. **Test eval_string.rs** (generate baseline, identify gaps, fix)
4. **Test actor_runtime.rs** (generate baseline, identify gaps, fix)
5. **Update documentation** after each file

### End of Session

1. **Run full test suite**: `cargo test --lib`
2. **Update SPRINT_9_PHASE2_PROGRESS.md** with files completed
3. **Update roadmap** with current status
4. **Create session summary** if significant progress made

---

## Estimated Timeline

**Next Session (2-3 hours)**:
- Fix deterministic.rs (12 gaps) - 1 hour
- Test eval_array.rs - 45 minutes
- Test eval_string.rs - 45 minutes
- Documentation updates - 30 minutes

**Week 2 Completion (3-4 sessions)**:
- Complete 12-15 medium files
- Add 60-100 tests
- Fix 80-120 mutation gaps
- Achieve 80%+ coverage across tested files

**Sprint 9 Completion (4 weeks)**:
- Phase 1: COMPLETE (8 files)
- Phase 2: 12-15 files (target)
- Phase 3: 10-12 files (400-600 lines)
- Phase 4: 5-8 files (600-1000 lines)
- Total: 35-40 files at 80%+ coverage

---

## Success Criteria for Next Session

- ✅ deterministic.rs: 12 gaps fixed
- ✅ eval_array.rs: Tested and gaps fixed
- ✅ eval_string.rs: Tested and gaps fixed
- ✅ Zero test regressions maintained
- ✅ Documentation updated
- ✅ Phase 2 progress: 4/15 files (26.7%)

---

## Files and Commands Ready for Next Session

### Pre-Generated Baselines
- `deterministic_mutation_baseline.txt` - Ready to use for gap analysis

### Commands to Run
```bash
# Fix deterministic.rs (baseline already exists)
# Review: deterministic_mutation_baseline.txt
# Add tests to: src/runtime/deterministic.rs

# Test eval_array.rs
cargo mutants --file src/runtime/eval_array.rs --timeout 180 --no-times 2>&1 | tee eval_array_mutation_baseline.txt

# Test eval_string.rs
cargo mutants --file src/runtime/eval_string.rs --timeout 180 --no-times 2>&1 | tee eval_string_mutation_baseline.txt

# Test actor_runtime.rs
cargo mutants --file src/runtime/actor_runtime.rs --timeout 180 --no-times 2>&1 | tee actor_runtime_mutation_baseline.txt

# Verify tests
cargo test --lib --quiet
```

---

**Status**: Sprint 9 progressing excellently - Phase 1 complete, Phase 2 underway
**Next Focus**: Complete deterministic.rs, then continue systematically through medium files
**Deferred**: Parser regression analysis (29 gaps identified, can address in Sprint 8.5)
