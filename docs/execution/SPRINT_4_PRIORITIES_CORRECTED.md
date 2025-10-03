# Sprint 4 Priorities - CORRECTED

**Date**: 2025-10-03
**Status**: 🎯 Ready to execute
**Previous Version**: SPRINT_4_PRIORITIES.md (INCORRECT - based on false assumptions)

## Critical Process Lesson

**❌ Initial Analysis**: Rushed to conclusions without empirical testing
**✅ Corrected Analysis**: Genchi Genbutsu (現地現物) - Go and See the actual behavior

**Time Saved**: 1.5-2.5 hours (would have debugged non-existent multi-variable bug)
**Method**: Toyota Way - empirical testing revealed truth

---

## Corrected Findings Summary

### FALSE ALARM #1: Multi-Variable Expressions ✅ WORKS

**Claim**: "Returns first variable only"
**Reality**: Works perfectly

```bash
$ ruchy -e 'let price = 99.99; let tax = 0.08; price * (1.0 + tax)'
107.9892  ✅

$ ruchy -e 'let x = 10.0; let y = 20.0; (x * x + y * y).sqrt()'
22.360679774997898  ✅
```

**Status**: ~~P0~~ → **NO ACTION NEEDED**

### FALSE ALARM #2: One-Liner "Bugs" ⚠️ 87.5% COSMETIC

**8 failing one-liners breakdown**:
- **7 failures** (87.5%): Float formatting (`108.0` vs `108`) - COSMETIC
- **1 failure** (12.5%): println format - COSMETIC

**Math correctness**: 100% ✅
**Logic correctness**: 100% ✅
**Display preference**: Differs from expectations

**Status**: ~~P0~~ → **P2 (Cosmetic UX improvement)**

### CONFIRMED ISSUE: DataFrame Transpiler Support ✅ P0

**Finding**: DataFrames work in interpreter, fail in transpiler

**Evidence**:
```bash
# Interpreter mode: ✅ WORKS
$ cat test_df.ruchy
let df = df![
  "name" => ["Alice", "Bob"],
  "age" => [30, 25]
];
df

$ ruchy test_df.ruchy
DataFrame with 2 columns:  ✅ SUCCESS
  name: 2 rows
  age: 2 rows

# Transpiler mode: ❌ FAILS
$ ruchy build test_df.ruchy
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `polars`
```

**Root Cause**: Generated code uses `polars::` but doesn't include dependency

**Status**: ✅ **CONFIRMED P0**

---

## Corrected Sprint 4 Priorities

### P0-1: Document DataFrame as Interpreter-Only (30 min)

**Task**: Update Chapter 18 to clarify DataFrame support

**Changes to ruchy-book**:
```markdown
# Chapter 18: DataFrames

**Current Status**: DataFrames work in **interpreter mode only**.

## Using DataFrames

DataFrames are available when running Ruchy scripts directly:

\`\`\`bash
# ✅ Works: Interpreter mode
ruchy dataframe_example.ruchy

# ❌ Not yet supported: Transpiled binaries
ruchy build dataframe_example.ruchy
\`\`\`

For transpiled Rust programs, use [polars](https://pola.rs) directly.

## Planned: Transpiler Support

DataFrame transpilation is planned for v3.8+. Track progress at #issue-number.
```

**Success Criteria**:
- [ ] Chapter 18 updated with status banner
- [ ] All 4 examples marked as "interpreter mode"
- [ ] User expectations properly set
- [ ] GitHub issue created to track transpiler support

**Effort**: 30 minutes
**Impact**: 🔴 HIGH - Prevents user confusion

---

### P0-2: Create DataFrame Transpiler Support Ticket (15 min)

**Task**: Document transpiler enhancement needed

**GitHub Issue Template**:
```markdown
## Feature: DataFrame Transpiler Support

**Status**: DataFrames work in interpreter, fail in transpiler

**Root Cause**: Generated code uses `polars::` without dependency

**Solution**: When transpiling DataFrame code:
1. Detect DataFrame usage (`df![]` macro)
2. Add `polars = "X.Y"` to generated Cargo.toml
3. Add `use polars::prelude::*;` to generated code
4. Test all Chapter 18 examples compile successfully

**Acceptance Criteria**:
- [ ] `ruchy build dataframe_example.ruchy` succeeds
- [ ] Generated binary includes polars dependency
- [ ] All 4 Chapter 18 examples compile and run
- [ ] Regression tests added

**Effort**: 4-8 hours
**Priority**: P1 (Future Sprint)
```

**Success Criteria**:
- [ ] Issue created with detailed technical analysis
- [ ] Added to Sprint 5 backlog
- [ ] Linked from Chapter 18 documentation

**Effort**: 15 minutes
**Impact**: 🟡 MEDIUM - Tracks future work

---

### P1: Categorize Remaining 19 Book Failures (2-3 hours)

**Task**: Reanalyze 23 total failures minus 4 DataFrame = 19 remaining

**Method**: For each failure:
1. Read error message from `errors.log`
2. Test example manually in interpreter
3. Categorize:
   - **Logic Bug**: Incorrect computation → P0/P1 fix
   - **Cosmetic**: Display format preference → P2 backlog
   - **Not Implemented**: Missing feature → Document
   - **Transpiler Only**: Works in interpreter → Document

**Expected Breakdown**:
Based on one-liner analysis, likely:
- 2-5 logic bugs (actual P0/P1 issues)
- 8-10 cosmetic issues (float/string formatting)
- 2-4 not implemented (missing features)
- 2-4 transpiler-only (like DataFrame)

**Deliverable**: `CHAPTER_FAILURES_CATEGORIZED.md` with:
```markdown
## Logic Bugs (Fix in Sprint 4)
1. [Chapter X, Example Y] - Description - Estimated effort
2. ...

## Cosmetic Issues (Defer to Sprint 6+)
1. [Chapter X, Example Y] - Float formatting - P2
2. ...

## Not Implemented (Document)
1. [Chapter X, Example Y] - Feature Z not available - Add note
2. ...

## Transpiler-Only Issues (Document like DataFrame)
1. [Chapter X, Example Y] - Works in interpreter, fails transpiler
2. ...
```

**Success Criteria**:
- [ ] All 19 failures categorized
- [ ] 2-5 logic bugs identified for Sprint 4 fixes
- [ ] Cosmetic issues moved to backlog
- [ ] Not implemented features documented

**Effort**: 2-3 hours
**Impact**: 🔴 HIGH - Identifies real bugs to fix

---

### P1: Fix Confirmed Logic Bugs (Variable effort)

**Task**: Fix 2-5 actual logic bugs identified in P1 analysis

**Approach** (per bug):
1. Create failing test case (TDD)
2. Debug root cause (5 Whys)
3. Implement fix
4. Verify test passes
5. Run full test suite
6. Commit with regression test

**Example**:
```bash
# If "Control flow bug" found:
1. Create test: tests/control_flow_edge_case.rs
2. Debug: Why does it fail?
3. Fix: src/runtime/eval_expr.rs
4. Verify: cargo test control_flow_edge_case
5. Full suite: cargo test --all
6. Commit: [CONTROL-XXX] Fix edge case in if-else evaluation
```

**Success Criteria**:
- [ ] All identified logic bugs fixed
- [ ] Each fix has regression test
- [ ] Zero new test failures
- [ ] TDG A- grade maintained

**Effort**: 1-3 hours per bug (2-15 hours total)
**Impact**: 🔴 HIGH - Fixes real user-facing bugs

---

### P2: Float Display Formatting (Backlog)

**Task**: Auto-format `108.0` → `108` when appropriate

**Decision**: **DEFER TO SPRINT 6+**

**Rationale**:
- Math is correct (higher priority)
- Type-preserving behavior is defensible
- Cosmetic improvement, not a bug
- Focus on logic bugs first

**Effort**: 2-4 hours (when scheduled)
**Impact**: 🟡 LOW - UX polish

---

### P2: println Formatting (Backlog)

**Task**: Adjust println output conventions

**Decision**: **DEFER TO SPRINT 6+**

**Effort**: 1-2 hours (when scheduled)
**Impact**: 🟡 LOW - Display convention

---

## Sprint 4 Execution Plan

### Day 1 (Today - Completed)
- [x] Ecosystem compatibility testing
- [x] Initial analysis (INCORRECT)
- [x] Empirical verification (CORRECTED)
- [x] Root cause analysis
- [ ] **P0-1**: Document DataFrame as interpreter-only (30 min)
- [ ] **P0-2**: Create DataFrame transpiler ticket (15 min)

### Day 2
- [ ] **P1 (Part 1)**: Categorize 19 book failures (2-3 hours)
- [ ] Start fixing identified logic bugs

### Day 3
- [ ] **P1 (Part 2)**: Continue fixing logic bugs
- [ ] Run comprehensive ecosystem tests
- [ ] Update integration reports

### Day 4 (Sprint Review)
- [ ] Verify all success criteria met
- [ ] Generate Sprint 4 completion report
- [ ] Sprint retrospective (Hansei)
- [ ] Plan Sprint 5

---

## Success Metrics

### Sprint 4 Goals (Revised)

**Primary Goals**:
- ✅ DataFrame status clarified (interpreter-only documented)
- ✅ 2-5 logic bugs identified and fixed
- ✅ False alarms eliminated from roadmap

**Expected Impact on Metrics**:
- **ruchy-book**: 97/120 → 100-105/120 (83-87%) by fixing 3-8 logic bugs
- **Clarity**: Users understand what works vs limitations
- **Process**: Improved verification methodology

**Quality Goals**:
- ✅ TDG A- grade maintained
- ✅ Zero regressions
- ✅ All fixes include regression tests

---

## Quality Gates (MANDATORY)

### Before Starting Work
```bash
pmat tdg . --min-grade A- --fail-on-violation
cargo test --all
make coverage
```

### Before Each Commit
```bash
pmat tdg <modified_file.rs> --include-components --min-grade B+
cargo test --all
```

### Before Sprint Complete
```bash
pmat tdg . --min-grade A- --fail-on-violation
cd ../ruchy-book && make test-comprehensive
cd ../rosetta-ruchy && make test-all-examples
cd ../ruchy-repl-demos && make test
```

---

## Lessons Applied (Kaizen)

### Process Improvements

**New Rule #1: Empirical Verification Required**

Before labeling anything as a "bug":
1. ✅ Test manually with actual examples
2. ✅ Verify expected vs actual behavior
3. ✅ Distinguish logic bugs from cosmetic issues
4. ✅ Confirm with multiple test cases

**New Rule #2: Categorize All Failures**

ALL test failures MUST be categorized:
- **Logic Bug**: Wrong computation → Fix immediately
- **Cosmetic**: Display preference → Backlog
- **Not Implemented**: Missing feature → Document
- **Works Elsewhere**: Interpreter vs transpiler → Document

**New Rule #3: Toyota Way - Genchi Genbutsu**

**GO AND SEE** the actual behavior:
- Don't assume based on test names
- Don't trust initial analysis without verification
- Test it yourself before claiming it's broken

### Time Saved

**False Alarm #1**: Would have spent 1.5-2.5 hours debugging multi-variable bug that doesn't exist ✅ SAVED
**False Alarm #2**: Would have spent 2-4 hours on cosmetic float formatting ✅ DEFERRED

**Total Time Saved**: 3.5-6.5 hours redirected to ACTUAL bugs

---

## Risks and Mitigations

### Risk 1: More logic bugs than expected
**Probability**: Medium
**Impact**: High (delays sprint)
**Mitigation**: Time-box to 2 hours per bug, defer complex ones to Sprint 5

### Risk 2: Categorization takes longer than estimated
**Probability**: Low
**Impact**: Medium
**Mitigation**: Use simple categorization script, parallelize analysis

### Risk 3: Ecosystem tests reveal new issues
**Probability**: Low
**Impact**: Medium
**Mitigation**: Comprehensive pre-commit testing catches issues early

---

## Conclusion

### Summary

- ✅ **Corrected false assumptions** via empirical testing
- ✅ **Prevented wasted effort** on non-existent bugs
- ✅ **Identified real issue** (DataFrame transpiler)
- 🎯 **Clear path forward** with realistic priorities

### Toyota Way Achievement

**Genchi Genbutsu** (現地現物): Went and saw actual behavior
**Hansei** (反省): Reflected on initial mistakes
**Kaizen** (改善): Improved verification process

### Next Immediate Action

**START WITH P0-1** (30 minutes):
1. Update Chapter 18 with interpreter-only banner
2. Set user expectations correctly
3. Quick win to build momentum

**THEN P0-2** (15 minutes):
1. Create DataFrame transpiler enhancement ticket
2. Add to Sprint 5 backlog

**THEN P1** (2-3 hours):
1. Categorize remaining 19 failures
2. Identify real logic bugs
3. Fix systematically with TDD

---

**Generated**: 2025-10-03
**Status**: ✅ Ready to execute
**Method**: Empirical testing + Toyota Way principles
**Confidence**: HIGH - Based on actual testing, not assumptions
