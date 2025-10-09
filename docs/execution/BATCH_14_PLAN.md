# Batch 14 Quality Violations Plan

**Sprint**: Quality Violations Elimination (Priority 2)
**Current Status**: 119 violations (136 → 119 from Batch 13)
**Target**: 119 → 109 violations (-10 violations)
**Date**: 2025-10-08

---

## Current Violation Breakdown

| Type | Count | Priority |
|------|-------|----------|
| **Complexity** | 44 | 🔥 HIGH |
| **SATD** | 23 | 🔥 HIGH |
| **Entropy** | 49 | ⚠️ MEDIUM |
| **Minor** | 3 | ✅ LOW |
| **TOTAL** | 119 | |

---

## Critical Complexity Violations

### Top 3 High-Complexity Functions (from CLAUDE.md v1.9.3)

1. **`evaluate_expr()`** - Complexity 138
   - Location: `src/runtime/repl/mod.rs`
   - Target: ≤50 (intermediate), ultimate ≤10
   - Impact: Core interpreter function, touches all expression evaluation
   - Reduction needed: 128 points (93% reduction)
   - Estimated effort: 5-10 sessions (major refactoring)

2. **`Value::fmt()`** - Complexity 66
   - Location: `src/runtime/value.rs` (likely)
   - Target: ≤30 (intermediate), ultimate ≤10
   - Impact: All value display formatting
   - Reduction needed: 36 points (55% reduction)
   - Estimated effort: 2-3 sessions

3. **`Value::format_dataframe()`** - Complexity 69
   - Location: `src/runtime/value.rs` (likely)
   - Target: ≤30 (intermediate), ultimate ≤10
   - Impact: DataFrame display formatting only
   - Reduction needed: 39 points (57% reduction)
   - Estimated effort: 2-3 sessions

**Total complexity reduction needed**: 203 points across 3 functions

---

## Batch 14 Strategy - Incremental Approach

### Phase 1: Low-Hanging Fruit (This Session)
**Target**: -5 to -10 violations from SATD and minor categories

#### SATD Violations (23 total)
- Search for TODO, FIXME, HACK comments
- Convert to proper implementations or remove
- Each SATD fix = -1 violation

**Commands**:
```bash
# Find SATD violations
pmat analyze satd src/ --fail-on-violation

# Or manual search
grep -rn "TODO\|FIXME\|HACK" src/ | wc -l
```

#### Minor Violations (3 total)
- Address all 3 minor violations
- Quick wins, likely formatting or documentation issues

**Expected Result**: 119 → 114 violations (-5 violations minimum)

---

### Phase 2: Medium Complexity Functions (Next Session)
**Target**: Functions with complexity 11-30

#### Approach:
1. Identify all functions with complexity 11-30
2. Select 5-10 functions to refactor
3. Apply systematic decomposition:
   - Extract helper functions
   - Reduce nesting levels
   - Simplify conditionals
4. Target: -5 to -10 violations

**Commands**:
```bash
# Find medium complexity functions
pmat analyze complexity --max-cyclomatic 10 src/ | grep "Violation"
```

**Expected Result**: 114 → 104 violations (-10 violations)

---

### Phase 3: High Complexity Functions (Future Sessions)
**Target**: Functions with complexity >30

#### Approach:
1. **evaluate_expr** (138 → ≤50) - 5 sessions
   - Week 1: Extract match arms to helper functions (138 → 100)
   - Week 2: Decompose complex arms (100 → 70)
   - Week 3: Simplify control flow (70 → 50)
   - Week 4: Further decomposition (50 → 30)
   - Week 5: Final cleanup (30 → ≤10)

2. **Value::fmt** (66 → ≤30) - 2 sessions
   - Session 1: Extract type-specific formatters (66 → 40)
   - Session 2: Simplify formatting logic (40 → ≤30)

3. **Value::format_dataframe** (69 → ≤30) - 2 sessions
   - Session 1: Extract table rendering logic (69 → 40)
   - Session 2: Simplify column formatting (40 → ≤30)

**Expected Result**: 104 → 60 violations over multiple sessions

---

## Batch 14 Execution Plan (This Session)

### Step 1: Analyze SATD Violations (15 min)
```bash
# Find all SATD comments
grep -rn "TODO\|FIXME\|HACK" src/ > /tmp/satd_violations.txt

# Categorize:
# - Can be removed (obsolete comments)
# - Can be fixed quickly (<5 min each)
# - Need separate ticket (defer)
```

### Step 2: Fix Quick SATD Violations (30 min)
- Target: 5-10 SATD comments
- Remove obsolete TODOs
- Implement simple FIXMEs
- Document complex items in roadmap

### Step 3: Address Minor Violations (15 min)
- Run `pmat analyze` to identify minor issues
- Fix all 3 minor violations
- Likely: missing docs, formatting, simple warnings

### Step 4: Verify Zero Regressions (10 min)
```bash
# Run full test suite
cargo test --all

# Verify P0 tests
cargo test --test p0_critical_features

# Check WASM tests
make test-wasm-all
```

### Step 5: Document Progress (10 min)
- Update violation count
- Create commit with metrics
- Update roadmap

**Total Time**: ~80 minutes (1.3 hours)
**Expected Reduction**: 119 → 114 violations (-5 minimum, -10 target)

---

## Toyota Way Principles

### Jidoka (Stop the Line)
- Run full test suite after each batch of fixes
- Never proceed if tests fail
- Zero tolerance for regressions

### Genchi Genbutsu (Go and See)
- Read actual code, don't assume SATD location
- Understand why TODO was added before removing
- Verify fix works, don't just delete comments

### Kaizen (Continuous Improvement)
- Small batches (-10 violations per session)
- Systematic approach (SATD → Minor → Medium → High)
- Document lessons learned

### Respect for People
- TODOs represent developer knowledge
- Don't delete without understanding context
- Convert to proper tickets if work remains

---

## Success Criteria

### Batch 14 (This Session)
- ✅ 119 → ≤114 violations (-5 minimum)
- ✅ All tests passing (zero regressions)
- ✅ SATD violations reduced by 5-10
- ✅ All 3 minor violations fixed
- ✅ Progress documented

### Overall Sprint (Quality Violations Elimination)
- 🎯 119 → 0 violations (ZERO TOLERANCE)
- 🎯 All functions ≤10 complexity
- 🎯 Zero SATD comments
- 🎯 All entropy violations resolved

---

## Risk Mitigation

### High Risk: evaluate_expr Refactoring
- **Risk**: Breaking core interpreter functionality
- **Mitigation**:
  - Extensive test suite (3580+ tests)
  - Incremental changes (5 sessions)
  - Property tests for invariants
  - REPL manual testing after each change

### Medium Risk: SATD Removal
- **Risk**: Removing TODO that documents real issue
- **Mitigation**:
  - Create roadmap tickets for complex TODOs
  - Git history preserves original comments
  - Review each TODO individually

### Low Risk: Minor Violations
- **Risk**: Minimal, usually formatting
- **Mitigation**: Automated tools (cargo fmt, clippy)

---

## Next Steps After Batch 14

1. **Batch 15**: Medium complexity functions (11-30)
   - Target: -10 violations
   - Focus: 5-10 functions in 11-20 complexity range

2. **Batch 16-20**: High complexity functions
   - evaluate_expr decomposition (5 sessions)
   - Value::fmt simplification (2 sessions)
   - Value::format_dataframe cleanup (2 sessions)

3. **Final Cleanup**: Entropy violations
   - Address code duplication
   - Improve code organization
   - Final polish to 0 violations

---

## References

- **Current Status**: docs/execution/roadmap.md (line 626)
- **Quality Standards**: CLAUDE.md (line 420)
- **Previous Work**: Sprint 6 (Batch 1-13 completed, 136 → 119)
- **PMAT Documentation**: https://github.com/paiml/pmat

---

**Status**: 📋 **PLANNED** - Ready to execute
**Owner**: Ruchy Development Team
**Timeline**: Batch 14 this session, Batches 15-20 over next 2-3 weeks
