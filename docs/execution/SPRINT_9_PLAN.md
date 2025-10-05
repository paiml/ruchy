# Sprint 9: Runtime Test Suite Modernization - PLAN

**Date**: 2025-10-05
**Duration**: 4 weeks (based on Sprint 8 success pattern)
**Status**: PLANNING
**Based On**: Sprint 8 Parser Mutation Testing (91% achievement)

---

## Executive Summary

Apply Sprint 8's proven mutation testing methodology to the **runtime module** (58 files, ~30K LOC) to achieve 80%+ mutation coverage across critical runtime components.

### Success Pattern from Sprint 8

Sprint 8 achieved **91% success** (10/11 parser files at 75-100% mutation coverage) using:
- ✅ **Incremental file-by-file testing** (5-30 min feedback loop)
- ✅ **Baseline-driven approach** for large files (>1000 lines)
- ✅ **5 reusable test patterns** (match arms, stubs, negations, boundaries, guards)
- ✅ **70 targeted tests** eliminating 92+ mutation gaps
- ✅ **Zero regressions** (all tests passing)

### Sprint 9 Target

- **Target**: 80%+ mutation coverage on 25-30 critical runtime files
- **Method**: Incremental + baseline-driven mutation testing
- **Pattern**: Reuse Sprint 8's 5 test gap patterns
- **Quality**: Zero test regressions, systematic coverage

---

## Module Inventory (58 runtime files)

### Small Files (<200 lines) - 9 files - **Week 1 Priority**
```
✅ async_runtime.rs (140) - COMPLETE (100% coverage)
   eval_func.rs (104)
   eval_literal.rs (116)
   gc.rs (129)
   validation.rs (184)
   transformation.rs (202)
   eval_string_interpolation.rs (206)
   interpreter_tests.rs (206) - test file, skip
   value_utils.rs (228)
```

### Medium Files (200-500 lines) - 28 files - **Weeks 2-3**
```
   eval_try_catch.rs (234)
   eval_display.rs (268)
   eval_method.rs (282)
   deterministic.rs (290) - 55 mutants, 10+ gaps
   eval_array.rs (291) - 45 mutants, gaps found
   eval_string.rs (296)
   actor_runtime.rs (313)
   object_helpers.rs (321)
   builtin_init.rs (324)
   cache.rs (330)
   eval_expr.rs (382)
   mod.rs (383)
   resource_eval.rs (393)
   safe_arena.rs (405)
   eval_string_methods.rs (418)
   eval_pattern.rs (421)
   eval_loops.rs (424)
   eval_method_dispatch.rs (425)
   inspect.rs (456)
   eval_control_flow.rs (458)
   gc_impl.rs (466)
   lazy.rs (479)
   grammar_coverage.rs (502) - may be test-related
```

### Large Files (500-1000 lines) - 13 files - **Week 4**
```
   eval_pattern_match.rs (514)
   actor_concurrent.rs (519)
   eval_function.rs (531)
   repl_recording.rs (541)
   eval_data_structures.rs (565)
   replay_converter.rs (594)
   eval_operations.rs (615)
   arena.rs (628)
   transaction.rs (639)
   builtins.rs (645)
   eval_control_flow_new.rs (718)
   compilation.rs (771)
   eval_dataframe.rs (835)
```

### Very Large Files (>1000 lines) - 8 files - **Baseline-Driven**
```
   replay.rs (847)
   completion.rs (852)
   pattern_matching.rs (898)
   eval_builtin.rs (1022)
   actor.rs (1108)
   assessment.rs (1157)
   magic.rs (1184)
   dataflow_ui.rs (1355)
   dataflow_debugger.rs (1400)
   observatory_ui.rs (1467)
   eval_dataframe_ops.rs (1949)
   observatory.rs (2368)
   interpreter.rs (5845) - LARGEST, baseline-driven essential
```

---

## 4-Week Phased Plan

### Phase 1 (Week 1): Foundation - Small Files

**Goal**: Complete 8-10 small files (<200 lines) at 80%+ mutation coverage

**Files**:
1. ✅ async_runtime.rs (140) - DONE (100%)
2. eval_func.rs (104)
3. eval_literal.rs (116)
4. gc.rs (129)
5. validation.rs (184)
6. transformation.rs (202)
7. eval_string_interpolation.rs (206)
8. value_utils.rs (228)
9. eval_try_catch.rs (234)
10. eval_display.rs (268)

**Strategy**: Incremental file-by-file (5-30 min per file)

**Success Criteria**:
- ✅ 8-10 files at 80%+ mutation coverage
- ✅ Zero test regressions
- ✅ Reusable test patterns documented

**Expected Tests Added**: ~15-25 tests

---

### Phase 2 (Week 2): Core Evaluation - Medium Files

**Goal**: Complete 12-15 medium files (200-400 lines) at 80%+ mutation coverage

**Files Priority**:
1. eval_method.rs (282) - core functionality
2. deterministic.rs (290) - 10+ gaps identified
3. eval_array.rs (291) - gaps identified
4. eval_string.rs (296)
5. actor_runtime.rs (313)
6. object_helpers.rs (321)
7. builtin_init.rs (324)
8. cache.rs (330)
9. eval_expr.rs (382)
10. resource_eval.rs (393)
11. safe_arena.rs (405)
12. eval_string_methods.rs (418)
13. eval_pattern.rs (421)
14. eval_loops.rs (424)
15. eval_method_dispatch.rs (425)

**Strategy**: Incremental for files <350 lines, baseline-driven for 350-425

**Success Criteria**:
- ✅ 12-15 files at 80%+ mutation coverage
- ✅ deterministic.rs gaps eliminated (10+ mutations)
- ✅ eval_array.rs gaps eliminated

**Expected Tests Added**: ~30-40 tests

---

### Phase 3 (Week 3): Advanced Evaluation - Medium-Large Files

**Goal**: Complete 10-12 files (400-600 lines) at 80%+ mutation coverage

**Files Priority**:
1. inspect.rs (456)
2. eval_control_flow.rs (458)
3. gc_impl.rs (466)
4. lazy.rs (479)
5. eval_pattern_match.rs (514)
6. actor_concurrent.rs (519)
7. eval_function.rs (531)
8. repl_recording.rs (541)
9. eval_data_structures.rs (565)
10. replay_converter.rs (594)
11. eval_operations.rs (615)
12. arena.rs (628)

**Strategy**: Baseline-driven approach (files >500 lines timeout on incremental)

**Success Criteria**:
- ✅ 10-12 files at 80%+ mutation coverage
- ✅ Baseline-driven strategy validated
- ✅ Pattern library expanded

**Expected Tests Added**: ~25-35 tests

---

### Phase 4 (Week 4): Large Files & Completion

**Goal**: Complete 5-8 large files at 75%+ mutation coverage

**Files Priority** (baseline-driven essential):
1. transaction.rs (639)
2. builtins.rs (645)
3. eval_control_flow_new.rs (718)
4. compilation.rs (771)
5. eval_dataframe.rs (835)
6. replay.rs (847)
7. completion.rs (852)
8. pattern_matching.rs (898)

**Deferred** (investigate separately):
- eval_builtin.rs (1022) - complex builtins
- actor.rs (1108) - actor system complexity
- interpreter.rs (5845) - LARGEST file, needs dedicated sprint
- observatory*.rs - UI/debugging tools

**Strategy**: Baseline-driven only (extract gaps from baseline, write targeted tests)

**Success Criteria**:
- ✅ 5-8 large files at 75%+ mutation coverage
- ✅ Overall runtime mutation coverage 70%+ across tested files
- ✅ Comprehensive test patterns documented

**Expected Tests Added**: ~20-30 tests

---

## Test Gap Patterns (from Sprint 8)

### Pattern 1: Match Arm Deletions (35% of gaps)
**Example from deterministic.rs**:
```
MISSED: delete match arm Value::Object(map) in estimate_heap_usage
MISSED: delete match arm Value::String(s) in estimate_heap_usage
```
**Solution**: Test ALL match arms with assertions

### Pattern 2: Function Stubs (30% of gaps)
**Example from async_runtime.rs**:
```
MISSED: replace AsyncRuntime::sleep with ()
```
**Solution**: Validate actual behavior (timing, side effects, not just "doesn't panic")

### Pattern 3: Negation Operators (20% of gaps)
**Solution**: Test both true AND false branches explicitly

### Pattern 4: Arithmetic Operators (10% of gaps)
**Example from deterministic.rs**:
```
MISSED: replace * with / in estimate_heap_usage
MISSED: replace - with + in execute_with_seed
MISSED: replace += with -= in estimate_heap_usage
```
**Solution**: Test arithmetic operations return correct values

### Pattern 5: Boundary Conditions (5% of gaps)
**Solution**: Test <, <=, ==, >, >= explicitly

---

## Sprint 9 Success Metrics

### Target Metrics

| Metric | Target | Sprint 8 Baseline |
|--------|--------|-------------------|
| Files with 80%+ Coverage | 35-40 / 58 (60-70%) | 10/11 (91%) |
| Mutation Catch Rate | 75-90% average | 75-100% |
| Test Gaps Eliminated | 100+ mutations | 92+ |
| Tests Added | 90-130 tests | 70 |
| Test Regressions | 0 | 0 |
| Schedule Performance | 4 weeks | 4 weeks |

### Quality Gates

1. ✅ **Zero Test Regressions**: All existing tests must pass
2. ✅ **Incremental Progress**: At least 8-10 files per week
3. ✅ **Mutation Coverage**: 75-90% per tested file
4. ✅ **Pattern Documentation**: Test patterns documented for reuse
5. ✅ **Baseline Validation**: Full mutation baseline before Phase 4

---

## Key Decisions from Sprint 8

### What Worked

1. **Incremental File-by-File** (5-30 min) - Ideal for files <1000 lines
2. **Baseline-Driven** (use empirical data) - Essential for large files
3. **Documentation First** - Updated README.md, CLAUDE.md, Makefile early
4. **Tooling Automation** - 4 Makefile targets created
5. **Pragmatic Deferrals** - actors.rs timeout = acceptable (91% success)

### What to Improve

1. **Disk Space Management**: Clean target/ directory between large files
2. **Parallel Processing**: Consider testing multiple small files in parallel
3. **Test Organization**: Group runtime tests by functionality (eval, actor, gc, etc.)
4. **Coverage Tracking**: Maintain mutation coverage spreadsheet

---

## Risk Mitigation

### Risk 1: Large File Timeouts
**Mitigation**: Use baseline-driven approach (proven in Sprint 8 Phase 2-4)

### Risk 2: Disk Space Constraints
**Mitigation**: Run `cargo clean` between large files, monitor with `df -h`

### Risk 3: Complex Runtime Logic
**Mitigation**: Focus on core eval modules first, defer UI/debugging tools

### Risk 4: Test Maintenance Burden
**Mitigation**: Create reusable test helpers, document patterns clearly

---

## Documentation Deliverables

1. **SPRINT_9_COMPLETE.md** - Comprehensive completion report (like Sprint 8)
2. **Runtime Mutation Patterns** - Specific to eval/actor/gc modules
3. **Test Helper Library** - Reusable test utilities for runtime
4. **Makefile Updates** - Runtime-specific mutation testing targets
5. **CLAUDE.md Updates** - Runtime mutation testing protocols

---

## Dependencies

- ✅ cargo-mutants v25.3.1 installed
- ✅ Sprint 8 learnings documented
- ✅ Baseline mutation testing infrastructure
- ✅ Test patterns identified
- ⚠️ Disk space monitoring (100% usage risk)

---

## Timeline

**Week 1** (Days 1-5): Phase 1 - Small files (8-10 files)
**Week 2** (Days 6-10): Phase 2 - Core eval modules (12-15 files)
**Week 3** (Days 11-15): Phase 3 - Advanced eval (10-12 files)
**Week 4** (Days 16-20): Phase 4 - Large files + completion (5-8 files)

**Total**: 35-45 files tested, 70-90% mutation coverage achieved

---

## Next Steps

1. ✅ Review and approve Sprint 9 plan
2. 🔄 Start Phase 1 with eval_func.rs (104 lines)
3. 🔄 Document baseline mutation results
4. 🔄 Create runtime test helper library
5. 🔄 Update roadmap with Sprint 9 start

---

**Status**: ✅ SPRINT 9 PLAN COMPLETE - Ready to begin Phase 1!
**Approval Needed**: Proceed with Sprint 9 execution?
