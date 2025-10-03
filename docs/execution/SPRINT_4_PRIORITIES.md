# Sprint 4 Priorities: Ecosystem Compatibility Fixes

**Sprint**: Sprint 4 (Post-WASM Refactoring)
**Date**: 2025-10-03
**Status**: 🚧 Ready to start
**Goal**: Fix P0 bugs discovered in v3.67.0 compatibility testing

## Sprint Context

### What Just Happened (Sprint 3)
- ✅ WASM backend refactoring complete (24 functions extracted)
- ✅ v3.67.0 released to crates.io
- ✅ Comprehensive ecosystem testing completed
- ✅ **Discovery**: v3.67.0 shows +4% improvement over v3.63.0

### Current Ecosystem Health

| Repository | Pass Rate | Change from Previous | Status |
|------------|-----------|---------------------|--------|
| ruchy-book | 97/120 (81%) | +5 examples (+4%) | ⬆️ IMPROVED |
| rosetta-ruchy | 71/105 (67.6%) | +1 example (+0.9%) | ⬆️ IMPROVED |
| ruchy-repl-demos | 3/3 (100%) | No change | ✅ STABLE |

**Key Finding**: v3.67.0 is BETTER than v3.63.0 - no regressions from WASM refactoring.

## Sprint 4 Objectives

### Primary Goal
Fix **2 critical P0 bugs** affecting user-facing functionality

### Success Metrics
- ✅ Multi-variable expressions: 20/20 one-liners passing (from 12/20)
- ✅ DataFrame support: 4/4 examples working (from 0/4) OR documented limitation
- ✅ ruchy-book: >85% pass rate (from 81%)
- ✅ All fixes include regression tests (TDD mandatory)

### Expected Impact
- **+16 examples working** = 113/120 (94%) in ruchy-book
- **User satisfaction**: Fix most-reported bugs
- **Credibility**: DataFrame feature working or honestly documented

## Priority Tasks

### P0-1: Fix Multi-Variable Expressions (HIGHEST PRIORITY)

**Ticket**: Create as INTERP-001
**Why First**: Smallest effort, highest user impact, quick win

#### Problem Statement
```ruchy
// ❌ FAILING - returns 99.99 instead of 107.99
let price = 99.99;
let tax = 0.08;
price * (1.0 + tax)

// ❌ FAILING - 8/20 one-liners affected
```

#### Investigation Steps
1. **Create minimal reproducible test case**:
   ```rust
   #[test]
   fn test_multi_variable_expression() {
       let input = r#"
           let x = 10;
           let y = 20;
           x + y
       "#;
       let result = run_ruchy(input);
       assert_eq!(result, "30");  // Currently returns "10"
   }
   ```

2. **Debug interpreter expression evaluation**:
   - Check `src/runtime/interpreter.rs::evaluate_expr()`
   - Verify symbol table lookup working correctly
   - Test if only first variable being returned

3. **Root cause analysis using 5 Whys**:
   ```
   Q1: Why does it return first variable only?
   A1: Last expression not evaluating correctly

   Q2: Why is last expression not evaluating?
   A2: May be returning after first `let` statement

   Q3: Why returning after first let?
   A3: Block evaluation may not be sequencing correctly

   Q4: Why is block evaluation wrong?
   A4: (To be discovered via debugging)

   Q5: (To be discovered)
   ```

4. **Fix implementation** following TDD:
   - Write failing test first (step 1)
   - Implement fix
   - Verify test passes
   - Run full test suite

5. **Regression prevention**:
   - Add property tests for multi-variable expressions
   - Test with 2, 3, 4+ variables
   - Test with different types (int, float, string)

#### Success Criteria
- [ ] Minimal test case passes
- [ ] All 8 failing one-liners now passing (20/20 total)
- [ ] Regression tests added
- [ ] Zero test failures in existing suite

#### Time Estimate
- Investigation: 30-60 minutes
- Fix implementation: 30-60 minutes
- Testing: 30 minutes
- **Total**: 1.5-2.5 hours

#### Assignee
@claude (immediate start)

---

### P0-2: Fix or Document DataFrame Support

**Ticket**: Create as DF-001
**Why Second**: Critical for credibility, may be quick config fix

#### Problem Statement
```ruchy
// ❌ ALL 4 DataFrame examples in Chapter 18 failing (0%)
// Advertised feature completely non-functional
```

#### Investigation Steps

1. **Check feature flag configuration**:
   ```bash
   # Verify Cargo.toml has dataframe feature
   grep "features.*dataframe" Cargo.toml

   # Test if examples work with feature enabled
   ruchy --features dataframe dataframe_example.ruchy
   ```

2. **Test basic DataFrame operations**:
   ```rust
   #[test]
   fn test_basic_dataframe() {
       let input = r#"
           let df = df![
               "name" => ["Alice", "Bob"],
               "age" => [30, 25]
           ];
           println!(df);
       "#;
       let result = run_ruchy_with_features(input, &["dataframe"]);
       assert!(result.contains("Alice"));
   }
   ```

3. **Decision tree**:
   ```
   IF feature flag missing:
     -> Add feature flag to default features
     -> Test all 4 examples
     -> If passing: DONE

   ELSE IF feature implemented but broken:
     -> Debug implementation
     -> Fix bugs
     -> Add tests

   ELSE IF feature not implemented:
     -> TWO OPTIONS:
        A) Implement DataFrame support (20+ hours)
        B) Update book to mark as "Planned for v3.8+" (30 min)
   ```

4. **Honest assessment**:
   - If feature not implemented: **UPDATE BOOK** to reflect reality
   - Do NOT ship broken/incomplete features as "working"
   - Toyota Way: Stop the line if quality not met

#### Success Criteria (Option A - Feature Works)
- [ ] All 4 DataFrame examples passing (4/4)
- [ ] Feature flag properly configured
- [ ] Comprehensive DataFrame tests added
- [ ] Documentation accurate

#### Success Criteria (Option B - Document Limitation)
- [ ] Chapter 18 updated with "Planned for v3.8+" banner
- [ ] Examples kept for future reference
- [ ] GitHub issue created to track implementation
- [ ] User expectations properly set

#### Time Estimate
- Investigation: 1-2 hours
- Option A (Fix): 4-8 hours
- Option B (Document): 30 minutes
- **Total**: 1.5-8 hours (depends on findings)

#### Assignee
@claude (after P0-1 complete)

---

### P1: Chapter-by-Chapter Analysis (BACKGROUND TASK)

**Ticket**: Create as QUALITY-009
**Why**: Understanding failure patterns informs future fixes

#### Task
Create detailed analysis of all 23 failing examples in ruchy-book:

```bash
# Extract failure details
cd ../ruchy-book
cat test/extracted-examples/failing.log > ../ruchy/docs/execution/CHAPTER_FAILURES_V3_67_0.md

# Categorize by failure type:
# - SyntaxError: Parser bugs
# - RuntimeError: Interpreter bugs
# - NotImplemented: Missing features
# - Timeout: Performance issues
```

#### Deliverable
Document with:
- List of all 23 failing examples
- Root cause categorization
- Recommended fix for each
- Priority ranking

#### Time Estimate
- 2-3 hours analysis

#### Assignee
@claude (parallel with P0 fixes)

---

## Sprint Execution Plan

### Day 1 (Today)
- [x] Complete ecosystem compatibility testing
- [x] Generate comprehensive compatibility report
- [ ] **START P0-1**: Multi-variable expressions (1.5-2.5 hours)
- [ ] Create regression test FIRST (TDD)
- [ ] Debug and fix
- [ ] Verify fix with full test suite

### Day 2
- [ ] **COMPLETE P0-1** if not done
- [ ] **START P0-2**: DataFrame support (1.5-8 hours)
- [ ] Investigation phase
- [ ] Make go/no-go decision on implementation vs documentation
- [ ] Execute chosen path

### Day 3
- [ ] **COMPLETE P0-2**
- [ ] **START P1**: Chapter failure analysis (2-3 hours)
- [ ] Run full ecosystem tests with fixes
- [ ] Update integration reports
- [ ] Generate Sprint 4 completion report

### Day 4 (Sprint Review)
- [ ] Verify all success criteria met
- [ ] Publish updated integration reports
- [ ] Version bump and release v3.68.0
- [ ] Sprint retrospective
- [ ] Plan Sprint 5

## Quality Gates (MANDATORY)

### Before Starting Each Task
```bash
# MANDATORY: TDG baseline
pmat tdg . --min-grade A- --fail-on-violation

# Traditional quality gates
cargo test --all
make coverage
cargo clippy -- -D warnings
```

### During Development
```bash
# After each function/module:
pmat tdg <modified_file.rs> --include-components --min-grade B+

# Real-time monitoring:
pmat tdg dashboard --port 8080 &
```

### Before Commit
```bash
# MANDATORY: TDG A- grade verification
pmat tdg . --min-grade A- --fail-on-violation || {
    echo "❌ COMMIT BLOCKED: TDG grade below A-"
    exit 1
}

# Full test suite:
cargo test --all
make compatibility  # Test against all 3 repos
```

## Toyota Way Compliance

### Jidoka (Quality Built-In)
- ✅ All fixes MUST include regression tests
- ✅ TDD mandatory: test written BEFORE fix
- ✅ No fix committed without proof via passing test

### Genchi Genbutsu (Go and See)
- ✅ Debug actual failing examples (not synthetic tests)
- ✅ Use failing.log from ruchy-book for real cases
- ✅ Verify fixes against actual user code

### Kaizen (Continuous Improvement)
- ✅ Document root causes in sprint retrospective
- ✅ Update development process to prevent similar bugs
- ✅ Share learnings across sprints

### Hansei (Reflection)
- ✅ 5 Whys root cause analysis for each bug
- ✅ Sprint retrospective: What went well? What to improve?
- ✅ Process improvements documented

## Success Metrics Review

### Sprint 4 Goals
- [ ] Multi-variable expressions: 20/20 one-liners (from 12/20)
- [ ] DataFrame support: 4/4 working OR documented
- [ ] ruchy-book: >85% pass rate (target: 113/120 = 94%)
- [ ] All fixes include TDD tests
- [ ] Zero regressions in existing tests

### Ecosystem Impact
- **Before Sprint 4**: 97/120 (81%)
- **After Sprint 4 (estimated)**: 113/120 (94%)
- **Improvement**: +16 examples (+13%)

### Release Criteria for v3.68.0
- [ ] All P0 fixes complete
- [ ] All quality gates passing
- [ ] Integration reports updated
- [ ] CHANGELOG.md updated
- [ ] Compatibility verified across all 3 repos

## Risks and Mitigations

### Risk 1: Multi-variable fix harder than expected
**Probability**: Low (seems like simple bug)
**Impact**: Medium (delays DataFrame work)
**Mitigation**: Time-box to 4 hours, escalate if blocked

### Risk 2: DataFrame feature not implemented
**Probability**: Medium
**Impact**: High (user expectations not met)
**Mitigation**: Honest documentation update (Option B)

### Risk 3: Fixes introduce regressions
**Probability**: Low (comprehensive test suite)
**Impact**: High (break existing functionality)
**Mitigation**: Mandatory pre-commit testing against all 3 repos

## Dependencies

### Blockers
- None - can start immediately

### External Dependencies
- ruchy-book, rosetta-ruchy, ruchy-repl-demos repos available
- PMAT v2.112.0+ installed
- Deno available for ruchy-book tests

## Sprint Retrospective Template (For Day 4)

### What Went Well
- [ ] (To be filled at sprint end)

### What Didn't Go Well
- [ ] (To be filled at sprint end)

### What We Learned
- [ ] (To be filled at sprint end)

### Process Improvements
- [ ] (To be filled at sprint end)

### Kaizen Actions for Sprint 5
- [ ] (To be filled at sprint end)

---

**Status**: 🚧 Ready to start
**Next Action**: Create INTERP-001 ticket and start P0-1 (multi-variable expressions)
**Expected Completion**: Day 3 (2025-10-06)
**Release Target**: v3.68.0 after Sprint 4 complete
