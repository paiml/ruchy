# Next Sprint Options - 2025-10-06

**Context**: HYBRID C sprint complete, 2 legacy bugs fixed, book compatibility at 86.9%

**Current State**:
- Book Compatibility: 86.9% (20/23 testable, +4.3% from baseline)
- Quality: TDG A-, 3554 tests passing, zero regressions
- Sprint 9 Phase 3: Paused (3/10 large runtime files complete)
- Technical Debt: 119 violations (44 complexity, 23 SATD, 49 entropy)

---

## 🎯 Option 1: Complete Book Compatibility (90%+ Target)

**Goal**: Close final 3.1% gap to reach 90% book compatibility

### Remaining Gaps (3/23 examples)
Based on book analysis, likely missing:
1. **Advanced dataframe operations** (Chapter 18) - may need method implementations
2. **Complex pattern matching edge cases** - potentially decorator patterns or advanced guards
3. **Module system features** - import/export edge cases

### Approach
1. Run book compatibility suite to identify exact 3 failing examples
2. Implement missing features using EXTREME TDD
3. Add comprehensive test coverage (property + mutation)
4. Update book integration documentation

### Effort Estimate
- **Time**: 3-5 hours (based on HYBRID C efficiency)
- **Risk**: LOW (small scope, clear target)
- **Tests**: +15-25 tests expected

### Benefits
- ✅ Achieve 90%+ book compatibility milestone
- ✅ User-facing value (more book examples work)
- ✅ Marketing benefit (90% compatibility claim)
- ✅ Builds on HYBRID C momentum

### Risks
- Missing features might be complex (async/await, advanced macros)
- May require parser or runtime changes
- Could uncover deeper architectural issues

### Recommendation
**HIGH PRIORITY** - Natural continuation of HYBRID C success, clear value

---

## 🎯 Option 2: Resume Sprint 9 Phase 3 (Runtime Quality)

**Goal**: Complete mutation testing for 7 remaining large runtime files

### Scope
Resume overnight mutation testing infrastructure for:
- `eval_for_loop.rs` (442 lines)
- `eval_match.rs` (462 lines)
- `eval_operators.rs` (489 lines)
- `eval_variables.rs` (512 lines)
- `eval_functions.rs` (583 lines)
- `eval_control_flow.rs` (627 lines)
- `interpreter.rs` (700+ lines) - largest file

### Approach
1. Run overnight mutation tests (10-15h runtime)
2. Analyze MISSED mutations (expect 50-100 gaps)
3. Create targeted test suites using Sprint 9 patterns
4. Achieve 75-90% mutation coverage per file

### Effort Estimate
- **Setup Time**: 1 hour (scripts ready)
- **Overnight Run**: 10-15 hours (automated)
- **Analysis + Fix**: 8-12 hours
- **Total Active Time**: 9-13 hours

### Benefits
- ✅ Completes Sprint 9 mission (runtime quality)
- ✅ Systematic test gap elimination
- ✅ High mutation coverage foundation
- ✅ Infrastructure already built

### Risks
- Large time investment (9-13h active work)
- May find 100+ mutations to fix
- Risk of diminishing returns (test oracle limitations)
- Could take 2-3 sessions

### Recommendation
**MEDIUM PRIORITY** - Important but significant time investment

---

## 🎯 Option 3: Address Technical Debt (Quality Sprint)

**Goal**: Reduce 119 quality violations, improve TDG grade

### Current Violations
- **44 Complexity**: Functions >10 cyclomatic complexity
- **23 SATD**: TODO/FIXME/HACK comments
- **49 Entropy**: Duplicate/repetitive patterns
- **3 Minor**: Dead code, unused imports

### Approach (Toyota Way Kaizen)
**Phase 1: SATD Elimination** (2-3h)
- Remove all 23 TODO/FIXME/HACK comments
- Either implement or document why deferred
- Zero tolerance policy going forward

**Phase 2: Complexity Reduction** (4-6h)
- Target top 10 most complex functions
- Extract helper functions (complexity ≤10)
- Add comprehensive tests for refactored code
- Use PMAT to verify improvements

**Phase 3: Entropy Reduction** (3-4h)
- Identify duplicate patterns
- Extract common functions
- Apply DRY principle systematically

### Effort Estimate
- **Total Time**: 9-13 hours
- **Risk**: MEDIUM (refactoring always risky)
- **Tests**: Comprehensive coverage required

### Benefits
- ✅ Improved code maintainability
- ✅ Easier onboarding for contributors
- ✅ Reduced bug surface area
- ✅ Better TDG grade (A- → A or A+)

### Risks
- High risk of introducing regressions
- Requires extensive testing
- May not provide immediate user value
- Could take multiple sessions

### Recommendation
**LOW-MEDIUM PRIORITY** - Important but not urgent, high risk

---

## 🎯 Option 4: WASM Quality Assurance (Strategic)

**Goal**: Establish world-class WASM quality gates (following wasm-labs pattern)

### Background
Roadmap line 16 states:
> "Based on wasm-labs success pattern (87% coverage, 99.4% mutation, 39 E2E tests), we are implementing world-class WASM quality assurance as the EXCLUSIVE priority until complete."

### Current WASM State
- E2E Tests: 39/39 passing (100%)
- Coverage: Unknown (need to measure)
- Mutation: Not tested for WASM-specific code
- Performance: 6.5s execution (35% better than 10s target)

### Approach
**Phase 1: Baseline Measurement** (2-3h)
- Measure current WASM code coverage
- Run mutation tests on WASM backend
- Identify test gaps

**Phase 2: Coverage Enhancement** (4-6h)
- Add unit tests for WASM codegen
- Add integration tests for WASM execution
- Property tests for WASM correctness

**Phase 3: Mutation Testing** (4-6h)
- Run cargo-mutants on WASM backend
- Fix test gaps systematically
- Target 90%+ mutation coverage

### Effort Estimate
- **Total Time**: 10-15 hours
- **Risk**: HIGH (WASM complexity)
- **Tests**: +50-100 tests expected

### Benefits
- ✅ Aligns with stated strategic priority
- ✅ Critical for production readiness
- ✅ Differentiator vs other languages
- ✅ Foundation for WASM-first features

### Risks
- WASM testing is complex
- May require new infrastructure
- Long time investment (10-15h)
- May uncover deep architectural issues

### Recommendation
**STRATEGIC PRIORITY** - Aligns with roadmap, but significant investment

---

## 📊 Comparison Matrix

| Option | Time | User Value | Risk | Strategic Fit | ROI |
|--------|------|------------|------|---------------|-----|
| **1. Book Compat 90%** | 3-5h | ⭐⭐⭐⭐⭐ | LOW | Medium | **HIGHEST** |
| **2. Sprint 9 Phase 3** | 9-13h | ⭐⭐⭐ | Medium | Medium | Medium |
| **3. Technical Debt** | 9-13h | ⭐⭐ | Medium-High | Low | Low-Medium |
| **4. WASM Quality** | 10-15h | ⭐⭐⭐⭐ | High | **HIGHEST** | High |

---

## 🎯 Recommendation: HYBRID Approach

### Primary: Option 1 (Book Compatibility 90%)
**Reasoning**:
- Shortest path to measurable success
- Builds on HYBRID C momentum
- Clear user value
- Low risk, high ROI
- **Estimated**: 3-5 hours

### Secondary: Option 4 (WASM Quality) - IF time allows
**Reasoning**:
- Strategic priority per roadmap
- Foundation for production readiness
- Can be done incrementally
- **Estimated**: Start with Phase 1 (2-3h)

### Defer: Options 2 & 3
- **Sprint 9 Phase 3**: Good work but can wait, significant time investment
- **Technical Debt**: Important but not urgent, high risk/reward ratio

---

## 🚀 Recommended Next Session Plan

### Session Goal: "Book Compatibility 90% + WASM Baseline"

**Phase 1: Book Compatibility (3-5h)**
1. Run book compatibility suite
2. Identify 3 failing examples
3. Implement missing features (EXTREME TDD)
4. Achieve 90%+ compatibility

**Phase 2: WASM Baseline (Optional, 2-3h if time)**
1. Measure WASM code coverage
2. Run mutation tests on WASM backend
3. Document gaps for future sprint

**Expected Outcomes**:
- ✅ 90%+ book compatibility (marketing milestone)
- ✅ WASM quality baseline established
- ✅ 5-8 hours total (manageable single session)
- ✅ Clear next steps for future work

---

## 📋 Decision Criteria

**Choose Option 1 (Book Compat) if**:
- Want immediate user-facing value
- Prefer low-risk, high-ROI work
- Want to maintain momentum from HYBRID C

**Choose Option 4 (WASM) if**:
- Strategic priority is critical
- Production readiness is goal
- Willing to invest 10-15 hours

**Choose Option 2 (Sprint 9) if**:
- Internal quality is top priority
- Have 9-13 hours available
- Want systematic test coverage

**Choose Option 3 (Tech Debt) if**:
- Code maintainability is critical
- Have experienced team for refactoring
- Can afford 9-13 hours + regression risk

---

**Generated**: 2025-10-06
**Status**: READY FOR DECISION
**Recommended**: Option 1 (Primary) + Option 4 Phase 1 (Secondary)
