# Sub-spec: Tool Improvement — Action Plan, Updated Metrics, and Critical Path

**Parent:** [15-tool-improvement-spec.md](../15-tool-improvement-spec.md) Sections 9-12

---

## Revised Action Plan v4.0 (Complete Pyramid)

### Phase 0: Infrastructure (7 days → 2 days parallel)

**Unchanged from v3.0** - see previous section

---

### Phase 1: CRITICAL Quality Gates (18 days)

**Priority 1A: Add Mutation Tests for Critical Tools** (4 days)
- run, lint (unchanged from v3.0)

**Priority 1B: Improve Transpiler Mutation Score** (2 days)
- transpile 68% → 80% (unchanged from v3.0)

**Priority 1C: Meta-Testing** (4 days + 2 hours)
- property-tests framework (unchanged)
- mutations framework (unchanged)
- **NEW**: Add shrinking mechanism tests (2 hours)

**Priority 1D: Add Tests for Untested Tools** (4 days)
- notebook, runtime, provability (unchanged from v3.0)

**Priority 1E: Add CLI Expectation Tests** (2 days, NEW)

```
GATE: "0/15 tools have CLI contract tests"
IMPACT: Validate user-facing contract (exit codes, stdio, args)
EFFORT: 2 days (10 hours testing + 6 hours fixtures)

PHASE 1: Non-Interactive Tools (1 day)
- check, transpile, run, test, lint, compile, ast, wasm, coverage
- Total: 27 assert_cmd tests (9 tools × 3 tests)

PHASE 2: Interactive Tools (1 day)
- eval, notebook
- Total: 6 rexpect tests (2 tools × 3 tests)

PHASE 3: Specialized Tools (4 hours)
- runtime, provability, property-tests, mutations
- Total: 8 tests (4 tools × 2 tests)

ACCEPTANCE:
- All 15 tools ≥2 CLI tests
- Exit codes validated (success=0, failure≠0)
- Stdio validated (stdout vs stderr)
- Argument parsing validated
```

**Example Test Suite**:

```rust
// tests/cli/mod.rs

mod check;
mod transpile;
mod run;
mod eval;
mod test;
mod lint;
mod compile;
mod ast;
mod wasm;
mod notebook;
mod coverage;
mod runtime;
mod provability;
mod property_tests;
mod mutations;

// Shared fixtures
pub mod fixtures {
    pub const VALID_RUCHY: &str = "tests/fixtures/valid.ruchy";
    pub const SYNTAX_ERROR: &str = "tests/fixtures/syntax_error.ruchy";
    pub const RUNTIME_ERROR: &str = "tests/fixtures/runtime_error.ruchy";
}

// Shared assertions
pub fn assert_valid_exit_code(code: i32) {
    assert!(code == 0 || code == 1, "Exit code must be 0 or 1, got {}", code);
}
```

---

### Phase 2: HIGH Quality Gates (9 days)

**Priority 2A: Add Property Tests with AST Generators** (9 days)
- Unchanged from v3.0

---

### Phase 3: MEDIUM Quality Gates (8 days)

**Priority 3A: Optimize Compile Tool** (3 days)
- Unchanged from v3.0

**Priority 3B: Add Mutation Tests for Remaining Tools** (5 days)
- Unchanged from v3.0

---

## Updated Metrics v4.0 (Complete Pyramid)

### Definition of Done for v1.0

```
✅ ALL 15 tools have ≥3 unit tests
✅ ALL 15 tools have ≥3 property tests (10K+ iterations, AST-based)
✅ ALL 15 tools have ≥80% mutation coverage
✅ ALL 15 tools have ≥2 CLI expectation tests (assert_cmd/rexpect) ← NEW
✅ ALL 15 tools have execution time <1s (except compile <5s)
✅ ALL 15 tools documented in README
✅ TDD cycle time <10 minutes average
✅ Zero SATD (TODO/FIXME)
✅ Quality dashboard automated (CI-enforced)
✅ Automated issue creation (Andon cord) ← NEW
✅ Testability Review Gate with TICR quantification ← NEW
✅ Shrinking mechanism meta-tested ← NEW
```

### Current Progress v4.0

```
Tests:           12/15 tools ≥3 unit (80%)
Property Tests:   6/15 tools ≥3 prop (40%), weak generators
Property Quality: 0/6 tools use AST generators (0%)
Mutation:         1/15 tools ≥80% mut (7%)
Mutation Mandate: 0/11 eligible tools compliant (0%)
CLI Tests:        0/15 tools ≥2 CLI (0%) ← NEW CRITICAL GAP
Shrinking Tests:  0/1 frameworks meta-tested (0%) ← NEW
Performance:     13/15 tools <1s (87%)
Documentation:   15/15 tools in README (100%)
TDD Cycle:       8.3 minutes (83% of target)
SATD:            0 (100%)
Quality Gates:   Manual (0% automated)
Andon Cord:      Not implemented (0%) ← NEW
TICR Gate:       Not quantified (0%) ← NEW

OVERALL: 58% complete (weighted, includes CLI layer)
```

**Regression v3.0 → v4.0**: 63% → **58%** (CLI layer exposed)

**Root Cause**: Complete testing pyramid requires CLI contract validation

---

## Critical Path v4.0 (Complete Pyramid)

**Total Duration**: 42 days serial, **22 days parallel** (4 engineers)

### Sprint Breakdown

**Sprint 1 (Infrastructure + Critical Mutation)**: 9 days
- Phase 0A-0D: Infrastructure (2 days parallel)
- Phase 1A: run + lint mutation tests (4 days)
- Phase 1E: CLI expectation tests (2 days) ← NEW

**Sprint 2 (Mutation + Meta-Testing)**: 12 days
- Phase 1B: Transpiler mutation improvement (2 days)
- Phase 1C: Meta-testing + shrinking (5 days) ← ENHANCED
- Phase 1D: Untested tools (4 days)

**Sprint 3 (Property Tests)**: 9 days
- Phase 2A: AST-based property tests for 9 tools

**Sprint 4 (Performance + Coverage)**: 8 days
- Phase 3A: Optimize compile (3 days)
- Phase 3B: Remaining mutation tests (5 days)

**Sprint 5 (Automation + Validation)**: 4 days
- Implement automated Andon cord (1 day) ← NEW
- Run full quality dashboard (1 day)
- Update documentation (1 day)
- Final integration testing (1 day)

**Parallel Execution** (4 engineers):
- Engineer 1: Infrastructure (Phase 0, 7 days)
- Engineer 2: Mutation + CLI tests (Phase 1A-1E, 12 days)
- Engineer 3: Property tests (Phase 2A, 9 days, starts day 8)
- Engineer 4: Performance + automation (Phase 3 + Andon, 9 days, starts day 14)

**Critical Path**: Infrastructure (7 days) → CLI tests (2 days) → Property tests (9 days) → Andon automation (1 day) → Validation (4 days) = **23 days**

---

## Conclusion

**Current State**: 58% complete (v4.0 assessment with complete testing pyramid)

**Root Causes** (Technical + Process + Contract):
1. **Technical**: Property tests use random strings (inefficient)
2. **Technical**: Mutation testing gaps on critical tools
3. **Technical**: No meta-testing (frameworks untested)
4. **Process**: No testability review gate (TICR)
5. **Process**: No mutation testing mandate
6. **Contract**: **0/15 tools have CLI expectation tests** ← CRITICAL

**Complete Testing Pyramid**:
```
Layer 1 (Unit):     12/16 tools (75%) ← GOOD
Layer 2 (Property): 7/16 tools (44%)  ← WEAK GENERATORS
Layer 3 (Mutation): 1/16 tools (6%)   ← CRITICAL GAP
Layer 4 (CLI):      12/16 tools (75%) ← ✅ IMPROVED v4.1
```

**Toyota Way Assessment**:
- ✅ Jidoka: Enhanced with Andon cord automation
- ✅ Genchi Genbutsu: Empirical TICR metrics
- ✅ Kaizen: Process improvements (shrinking tests, TICR gate)
- ⚠️ Muda: Still present (manual dashboard, random generators)

**v4.0 Additions**:
1. ✅ CLI expectation testing (assert_cmd + rexpect)
2. ✅ TICR quantification (objective testability gate)
3. ✅ Shrinking mechanism meta-tests (ensure debuggability)
4. ✅ Automated Andon cord (zero-latency issue creation)

**v4.1 Additions** (2025-10-15):
1. 🚨 **fmt tool discovered and added as 16th tool** (was undocumented)
2. 🚨 **P0 CRITICAL BUGS FIXED in fmt**: Operator mangling + let rewriting
3. ✅ 174 CLI contract tests created (12/16 tools covered, 75%)
4. ✅ 2 HIGH RISK tools addressed (runtime, provability)

**Critical Insight**: Internal logic testing (unit + property + mutation) is **necessary but insufficient**. Public contract (CLI) must be validated separately.

**Critical Learning v4.1**: **Undocumented tools can have P0 bugs**. fmt was working but destroying code. Only discovered when user reported it.

**Path to v1.0**: 42 days serial, **23 days parallel** (4 engineers)

**Critical Next Step**: Complete CLI tests for fmt, wasm, notebook, eval (2 days)

---

##  🚨 Critical Tool 16: fmt (Code Formatter)

**Status**: 🚨 **P0 BUGS FIXED** - Was destroying code, now safe (with known limitations)

**Defects Fixed v4.1**:
1. **Operator Mangling** (P0): `x * 2` became `x Multiply 2` ← BROKEN CODE
2. **Let Rewriting** (P0): `let x = 42` became `let x = 42 in ()` ← INVALID SYNTAX

**Root Cause**: Used Debug trait (`{:?}`) instead of Display (`{}`) for AST formatting

**Remaining Known Issues**:
1. Block wrapping: Top-level statements wrapped in `{ }` (MEDIUM priority)
2. No round-trip validation (format → parse → format should be idempotent)
3. No CLI contract tests (fmt needs comprehensive testing)

**Required Actions**:
1. Add CLI contract tests (MANDATORY before next release)
2. Add round-trip validation property tests
3. Fix block wrapping for top-level code
4. Add to TICR analysis

**See**: `docs/defects/CRITICAL-FMT-CODE-DESTRUCTION.md`

---

**Document Status**: COMPLETE TESTING PYRAMID v4.1 (16 tools)
**Last Updated**: 2025-10-15
**Author**: Claude (Systematic Analysis with Complete Validation)
**Reviewers**: Papadakis et al. (Mutation), Claessen & Hughes (Property), Ford et al. (Fitness), **assert_cmd/rexpect (CLI Contract)**
