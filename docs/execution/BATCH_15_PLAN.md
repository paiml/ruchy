# Batch 15 Quality Violations Plan

**Sprint**: Quality Violations Elimination (Priority 2)
**Current Status**: 467 violations (55 complexity, 0 SATD, 55 entropy, 286 duplicates, 2 other)
**Target**: 467 → 462 violations (-5 violations)
**Date**: 2025-10-09

---

## Current Violation Breakdown

| Type | Count | Priority |
|------|-------|----------|
| **Complexity** | 55 (5 errors >10, 82 warnings) | 🔥 HIGH |
| **SATD** | 0 | ✅ COMPLETE |
| **Entropy** | 55 | ⚠️ MEDIUM |
| **Duplicates** | 286 | ⚠️ MEDIUM |
| **Other** | 2 | ✅ LOW |
| **TOTAL** | 467 | |

---

## Top 5 Complexity Errors (>10)

From PMAT analysis:

1. **`handle_mutations_command()`** - Complexity 11
   - Location: `src/bin/handlers/mod.rs`
   - Target: ≤10 (Toyota Way)
   - Reduction needed: 1 point minimum

2. **`parse_parentheses_token()`** - Complexity 11
   - Location: `src/frontend/parser/expressions.rs`
   - Target: ≤10
   - Reduction needed: 1 point minimum

3. **`parse_match_list_pattern()`** - Complexity 11
   - Location: `src/frontend/parser/expressions.rs`
   - Target: ≤10
   - Reduction needed: 1 point minimum

4. **`handle_property_tests_single_file()`** - Complexity 10
   - Note: Already refactored in Batch 14, may be reporting issue
   - Verify actual state

5. **`parse_constructor()`** - Complexity 10
   - Location: `src/frontend/parser/expressions.rs`
   - Target: ≤10 (already at threshold)
   - May need minor adjustment

---

## Batch 15 Strategy

### Phase 1: Handlers Complexity (This Session)
**Target**: Fix handle_mutations_command (11 → ≤10)

#### Approach:
1. Extract helper functions from handle_mutations_command
2. Apply same refactoring pattern as Batch 14
3. Target: -1 to -2 complexity errors

**Expected Result**: 5 errors → 4 errors (-1 violation)

---

### Phase 2: Parser Complexity (Next Session)
**Target**: Functions in parser/expressions.rs

#### Approach:
1. parse_parentheses_token (11 → ≤10)
2. parse_match_list_pattern (11 → ≤10)
3. parse_constructor (10 → minor refactor if needed)

**Expected Result**: 4 errors → 1-2 errors (-2 to -3 violations)

---

## Batch 15 Execution Plan (This Session)

### Step 1: Analyze handle_mutations_command (10 min)
- Read function implementation
- Identify complexity sources
- Plan helper function extraction

### Step 2: Refactor handle_mutations_command (30 min)
- Extract 2-3 helper functions
- Target complexity ≤10
- Maintain all functionality

### Step 3: Build and Test (10 min)
```bash
cargo build --bin ruchy
cargo test --test p0_critical_features
```

### Step 4: Verify Complexity Reduction (5 min)
```bash
pmat analyze complexity --path src/bin/handlers/mod.rs --max-cyclomatic 10
```

### Step 5: Commit and Document (5 min)
- Commit with detailed metrics
- Update roadmap
- Update violation count

**Total Time**: ~60 minutes (1 hour)
**Expected Reduction**: 467 → 466 violations (-1 error)

---

## Toyota Way Principles

### Jidoka (Stop the Line)
- Run full test suite after refactoring
- Never proceed if tests fail
- Zero tolerance for regressions

### Genchi Genbutsu (Go and See)
- Read actual code before refactoring
- Understand why complexity is high
- Verify fix works, measure impact

### Kaizen (Continuous Improvement)
- Small batches (-1 to -5 violations per session)
- Systematic approach (handlers → parser → other)
- Document lessons learned

### Respect for People
- Preserve all existing functionality
- Clear helper function names
- Maintain test coverage

---

## Success Criteria

### Batch 15 (This Session)
- ✅ 467 → 466 violations (-1 minimum)
- ✅ handle_mutations_command ≤10 complexity
- ✅ All tests passing (zero regressions)
- ✅ Progress documented

### Overall Sprint (Quality Violations Elimination)
- 🎯 467 → 0 violations (ZERO TOLERANCE)
- 🎯 All functions ≤10 complexity
- 🎯 Zero SATD comments ✅ (COMPLETE)
- 🎯 All entropy violations resolved

---

## Risk Mitigation

### Medium Risk: Handler Refactoring
- **Risk**: Breaking CLI command functionality
- **Mitigation**:
  - Comprehensive test suite
  - P0 tests validate critical features
  - Incremental changes
  - Manual testing after changes

### Low Risk: Parser Refactoring
- **Risk**: Parser behavior changes
- **Mitigation**: 
  - Extensive parser tests
  - Property tests validate invariants
  - Test all grammar rules

---

## Next Steps After Batch 15

1. **Batch 16**: Parser complexity functions (2-3 functions)
2. **Batch 17**: Entropy violations (code duplication)
3. **Batch 18**: Duplicate code elimination
4. **Final Cleanup**: Coverage and provability violations

---

**Status**: 📋 **PLANNED** - Ready to execute
**Owner**: Ruchy Development Team
**Timeline**: Batch 15 this session, continue systematic reduction
