# QUALITY-002: Systematic unwrap() Replacement Specification

**Ticket**: QUALITY-002
**Priority**: Critical
**Estimated Effort**: 3-5 days (4 days realistic)
**Status**: In Progress (Started 2025-11-21)
**Methodology**: EXTREME TDD + Phased Rollout + Risk-Based Prioritization

---

## Executive Summary

**Problem**: 3,697 `unwrap()` calls in production code (`src/`) present a **Cloudflare-class defect risk**. Unwrap panics caused a 3+ hour Cloudflare outage (2025-11-18).

**Solution**: Systematic replacement of `unwrap()` with `.expect("descriptive message")` across all production code, prioritizing highest-risk modules first.

**Rationale**:
- `unwrap()` panics provide no context (just "called `Option::unwrap()` on a `None` value")
- `.expect()` provides actionable error messages for debugging
- Test code can keep `unwrap()` for cleaner assertions

---

## unwrap() Distribution Analysis

### Total Breakdown (3,697 calls in src/)

| Module | Calls | Percentage | Priority | Est. Time |
|--------|-------|------------|----------|-----------|
| Runtime/Interpreter | 1,112 | 30.1% | HIGH | 1.5 days |
| Other (stdlib, wasm, etc.) | 1,741 | 47.1% | MEDIUM | 1.5 days |
| Notebook | 326 | 8.8% | LOW | 0.5 day |
| CLI/Binary | 222 | 6.0% | **CRITICAL** | 1 day |
| Quality/Linter | 156 | 4.2% | MEDIUM | 0.5 day |
| Frontend (Parser) | 109 | 2.9% | HIGH | 0.5 day |
| LSP | 28 | 0.8% | LOW | 0.1 day |
| Transpiler | 3 | 0.1% | HIGH | 0.1 day |

### Top 20 Files (Highest Concentration)

| Rank | File | Calls | Module | Priority |
|------|------|-------|--------|----------|
| 1 | src/wasm/notebook.rs | 161 | WASM | MEDIUM |
| 2 | src/runtime/builtins.rs | 137 | Runtime | HIGH |
| 3 | src/notebook/engine.rs | 134 | Notebook | MEDIUM |
| 4 | src/runtime/bytecode/vm.rs | 116 | Runtime | **CRITICAL** |
| 5 | src/stdlib/fs.rs | 92 | Stdlib | MEDIUM |
| 6 | src/backend/arrow_integration.rs | 84 | Backend | LOW |
| 7 | src/stdlib/dataframe.rs | 79 | Stdlib | MEDIUM |
| 8 | src/backend/transpiler/statements.rs | 79 | Transpiler | HIGH |
| 9 | src/runtime/observatory.rs | 75 | Runtime | MEDIUM |
| 10 | src/stdlib/regex.rs | 73 | Stdlib | MEDIUM |
| 11 | src/runtime/mod.rs | 66 | Runtime | HIGH |
| 12 | src/notebook/server.rs | 66 | Notebook | LOW |
| 13 | src/backend/transpiler/codegen_minimal.rs | 66 | Transpiler | HIGH |
| 14 | src/bin/handlers/handlers_modules/test_helpers.rs | 61 | CLI | **CRITICAL** |
| 15 | src/middleend/infer.rs | 58 | Type Inference | **CRITICAL** |
| 16 | src/frontend/parser/mod.rs | 56 | Parser | **CRITICAL** |
| 17 | src/stdlib/path.rs | 55 | Stdlib | MEDIUM |
| 18 | src/quality/linter.rs | 54 | Quality | MEDIUM |
| 19 | src/lib.rs | 52 | Library Root | HIGH |
| 20 | src/backend/transpiler/dispatcher_helpers/error_handling.rs | 48 | Transpiler | HIGH |

---

## Risk-Based Prioritization

### CRITICAL Risk (Do First - Day 1-2)

**Rationale**: Core execution paths, user-facing errors, compilation pipeline

| Module | Files | Calls | Why Critical |
|--------|-------|-------|--------------|
| CLI/Binary | All `src/bin/` | 222 | Direct user interaction, error handling crucial |
| Frontend/Parser | `parser/mod.rs`, etc. | 109 | Compilation errors need context |
| Type Inference | `middleend/infer.rs` | 58 | Type errors must be debuggable |
| Runtime VM | `bytecode/vm.rs` | 116 | Core interpreter execution |
| Transpiler | Core files | 3 | Code generation errors critical |

**Total CRITICAL**: ~508 calls (14% of total)

### HIGH Risk (Do Second - Day 3)

**Rationale**: Core runtime functionality, standard library essentials

| Module | Files | Calls | Why High Risk |
|--------|-------|-------|---------------|
| Runtime Builtins | `runtime/builtins.rs` | 137 | Built-in function failures |
| Runtime Core | `runtime/mod.rs` | 66 | Core runtime logic |
| Transpiler Statements | `backend/transpiler/statements.rs` | 79 | Statement translation |
| Transpiler Codegen | `backend/transpiler/codegen_minimal.rs` | 66 | Code generation |
| Library Root | `src/lib.rs` | 52 | Public API entry points |

**Total HIGH**: ~400 calls (11% of total)

### MEDIUM Risk (Do Third - Day 4)

**Rationale**: Standard library, utilities, non-critical paths

| Module | Files | Calls | Why Medium Risk |
|--------|-------|-------|-----------------|
| Stdlib | `stdlib/*.rs` | ~280 | User-facing APIs but not critical path |
| Quality/Linter | `quality/linter.rs` | 156 | Development-time tooling |
| Runtime Observatory | `runtime/observatory.rs` | 75 | Debugging/monitoring features |
| WASM Notebook | `wasm/notebook.rs` | 161 | Notebook features (not core compiler) |
| Notebook Engine | `notebook/engine.rs` | 134 | Notebook infrastructure |

**Total MEDIUM**: ~806 calls (22% of total)

### LOW Risk (Do Last - Optional)

**Rationale**: Ancillary features, LSP, optional modules

| Module | Files | Calls | Why Low Risk |
|--------|-------|-------|--------------|
| LSP | `lsp/mod.rs` | 28 | Editor integration (not core) |
| Notebook Server | `notebook/server.rs` | 66 | Web server features |
| Backend Arrow | `backend/arrow_integration.rs` | 84 | Data integration (optional) |

**Total LOW**: ~178 calls (5% of total)

---

## Phased Implementation Plan

### Phase 1: CRITICAL Modules (Day 1-2, ~508 calls)

**Time**: 2 days
**Scope**: CLI, Parser, Type Inference, Runtime VM, Transpiler core

#### Phase 1A: CLI & Binary (Day 1, 222 calls)
- `src/bin/` - All CLI handlers and entry points
- **Focus**: User-facing errors, command-line argument parsing, file I/O
- **Test Strategy**: Run all CLI tests after each module

#### Phase 1B: Core Compilation Pipeline (Day 2, 286 calls)
- `src/frontend/parser/mod.rs` (56 calls) - Parser errors
- `src/middleend/infer.rs` (58 calls) - Type inference errors
- `src/runtime/bytecode/vm.rs` (116 calls) - VM execution
- `src/backend/transpiler/` core files (3 calls) - Code generation
- **Focus**: Compilation error messages, type error reporting
- **Test Strategy**: Run full test suite, verify error messages improved

**Acceptance Criteria**:
- ✅ All 508 CRITICAL unwrap() calls replaced
- ✅ All tests passing
- ✅ Error messages more descriptive
- ✅ Commit: `[QUALITY-002] Phase 1: Replace CRITICAL unwrap() calls (508/3697)`

### Phase 2: HIGH Risk Modules (Day 3, ~400 calls)

**Time**: 1 day
**Scope**: Runtime core, Transpiler statements, Library root

#### Day 3 Tasks:
- `src/runtime/builtins.rs` (137 calls)
- `src/runtime/mod.rs` (66 calls)
- `src/backend/transpiler/statements.rs` (79 calls)
- `src/backend/transpiler/codegen_minimal.rs` (66 calls)
- `src/lib.rs` (52 calls)

**Focus**: Built-in functions, runtime errors, transpilation quality

**Acceptance Criteria**:
- ✅ All 400 HIGH-risk unwrap() calls replaced
- ✅ All tests passing
- ✅ Built-in function errors clearer
- ✅ Commit: `[QUALITY-002] Phase 2: Replace HIGH-risk unwrap() calls (908/3697)`

### Phase 3: MEDIUM Risk Modules (Day 4, ~806 calls)

**Time**: 1 day
**Scope**: Stdlib, Quality tools, Notebooks

#### Day 4 Tasks:
- `src/stdlib/` - fs, dataframe, regex, path (~280 calls)
- `src/quality/linter.rs` (156 calls)
- `src/runtime/observatory.rs` (75 calls)
- `src/wasm/notebook.rs` (161 calls)
- `src/notebook/engine.rs` (134 calls)

**Focus**: Standard library robustness, development tooling

**Acceptance Criteria**:
- ✅ All 806 MEDIUM-risk unwrap() calls replaced
- ✅ All tests passing
- ✅ Stdlib errors user-friendly
- ✅ Commit: `[QUALITY-002] Phase 3: Replace MEDIUM-risk unwrap() calls (1714/3697)`

### Phase 4: LOW Risk & Remaining (Optional, ~178 calls)

**Time**: 0.5 day (optional)
**Scope**: LSP, Notebook server, Arrow integration

**Can be deferred if time-constrained**

**Acceptance Criteria**:
- ✅ All remaining unwrap() calls replaced
- ✅ Final commit: `[QUALITY-002] Phase 4: Complete unwrap() replacement (3697/3697) - DONE`

---

## Replacement Patterns

### Pattern 1: File Operations

```rust
// ❌ BEFORE (no context)
let contents = fs::read_to_string(&path).unwrap();

// ✅ AFTER (descriptive)
let contents = fs::read_to_string(&path)
    .expect(&format!("Failed to read file: {}", path.display()));
```

### Pattern 2: Collection Access

```rust
// ❌ BEFORE
let first = items.first().unwrap();

// ✅ AFTER
let first = items.first()
    .expect("Expected at least one item in collection");
```

### Pattern 3: Option Unwrapping

```rust
// ❌ BEFORE
let value = map.get(&key).unwrap();

// ✅ AFTER
let value = map.get(&key)
    .expect(&format!("Expected key '{}' to exist in map", key));
```

### Pattern 4: Result Unwrapping

```rust
// ❌ BEFORE
let parsed = value.parse::<i32>().unwrap();

// ✅ AFTER
let parsed = value.parse::<i32>()
    .expect(&format!("Failed to parse '{}' as integer", value));
```

### Pattern 5: Lock Operations

```rust
// ❌ BEFORE
let data = mutex.lock().unwrap();

// ✅ AFTER
let data = mutex.lock()
    .expect("Failed to acquire mutex lock (poisoned?)");
```

---

## EXTREME TDD Protocol

### RED Phase (Before Each Module)

1. **Identify unwrap() calls**: `grep unwrap() src/module/file.rs`
2. **Create failing test**: Test that current error message is unhelpful
3. **Document expected behavior**: What error message SHOULD appear

### GREEN Phase (During Replacement)

1. **Replace unwrap() with expect()**: Add descriptive message
2. **Run tests**: `cargo test --package ruchy --lib module::tests`
3. **Verify error messages**: Intentionally trigger error, verify message quality

### REFACTOR Phase (After Module)

1. **Review expect() messages**: Ensure consistency, clarity
2. **Check for similar patterns**: Group related unwrap() calls
3. **Run full suite**: `cargo test --all-targets`

### VALIDATE Phase (After Phase)

1. **PMAT TDG check**: `pmat tdg src/ --min-grade A-`
2. **Clippy**: `cargo clippy --all-targets -- -D warnings`
3. **Coverage**: Ensure no regressions
4. **Commit**: Atomic commit with phase summary

---

## Quality Gates (Pre-Commit)

**MANDATORY before each phase commit**:

```bash
# 1. Run all tests
cargo test --all-targets
echo "✅ Tests: $?"

# 2. Check for remaining unwrap() in modified files
git diff --name-only | xargs grep -n "unwrap()" || echo "✅ No unwrap() in changed files"

# 3. Run PMAT TDG
pmat tdg src/ --min-grade A-
echo "✅ TDG: $?"

# 4. Run clippy
cargo clippy --all-targets -- -D warnings
echo "✅ Clippy: $?"

# 5. Verify error messages improved (manual check)
# Intentionally trigger errors, verify messages are descriptive
```

---

## Commit Protocol

### Per-Phase Commits

**Message Format**:
```
[QUALITY-002] Phase X: Replace {RISK}-risk unwrap() calls ({DONE}/{TOTAL})

- Replaced {N} unwrap() calls with expect() in:
  - {module1}: {count1} calls
  - {module2}: {count2} calls

Impact:
- Improved error messages for {feature}
- Added context to {specific_errors}

Tests: {PASSING}/{TOTAL} passing
TDG: {SCORE}/100
Complexity: ≤10 maintained

Closes: Part of QUALITY-002
```

**Example**:
```
[QUALITY-002] Phase 1A: Replace CRITICAL unwrap() calls in CLI (222/3697)

- Replaced 222 unwrap() calls with expect() in:
  - src/bin/handlers/: 180 calls
  - src/bin/: 42 calls

Impact:
- CLI errors now show file paths and operation context
- Argument parsing errors include actual argument values

Tests: 4036/4036 passing
TDG: 94.9/100 (A)
Complexity: All functions ≤10

Closes: Part of QUALITY-002
```

---

## Toyota Way Principles

### Jidoka (Built-in Quality)
- **Before**: Silent panics, no debugging context
- **After**: Descriptive errors, immediate understanding

### Andon Cord (Stop the Line)
- If ANY test fails → STOP, fix before proceeding
- If TDG drops → STOP, refactor before continuing

### Genchi Genbutsu (Go and See)
- Intentionally trigger each error path
- Verify error messages are actually helpful
- Test with real-world scenarios

### Kaizen (Continuous Improvement)
- Learn patterns from Phase 1 → Apply to Phase 2
- Identify common expect() messages → Create helpers
- Document lessons learned

### Zero Defects
- All tests must pass after EVERY module
- No "temporary" unwrap() replacements
- No partial commits

---

## Progress Tracking

### Daily Standup Format

**What I did**:
- Replaced {N} unwrap() calls in {modules}
- Tests: {PASSING}/{TOTAL}

**What I'm doing**:
- Phase {X}: {module_name}
- Target: {N} calls today

**Blockers**:
- [List any issues]

**Progress**: {DONE}/{TOTAL} calls ({PERCENTAGE}%)

### Roadmap Updates

Update `docs/execution/roadmap.yaml` daily:

```yaml
- id: "QUALITY-002"
  status: "In Progress"
  progress: "{DONE}/{TOTAL} calls replaced ({PERCENTAGE}%)"
  current_phase: "Phase {X}: {description}"
  last_updated: "{DATE}"
```

---

## Success Criteria

### Definition of Done

- ✅ All 3,697 unwrap() calls in `src/` replaced with expect()
- ✅ All expect() messages are descriptive and actionable
- ✅ All tests passing (4036/4036 or more)
- ✅ TDG score maintained (≥94.9/100, Grade A)
- ✅ No clippy warnings introduced
- ✅ PMAT rust-project-score Code Quality improved (7.0 → 15.0+)
- ✅ Error messages verified manually for top 20 modules
- ✅ Roadmap updated to "Complete"
- ✅ CHANGELOG.md updated with summary

### PMAT Score Impact (Expected)

**Before**:
- Code Quality: 7.0/26 (26.9%)
- rust-project-score: 136.5/134 (A+)

**After**:
- Code Quality: **15.0+/26** (~60%+) - 8-point improvement
- rust-project-score: **140+/134** (A+) - Improved

---

## References

- **Cloudflare Outage (2025-11-18)**: unwrap() panic caused 3+ hour network outage
- **Rust Best Practices**: [Rust Book - Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- **PMAT Documentation**: `docs/PMAT-INTEGRATION-STATUS.md`
- **Roadmap**: `docs/execution/roadmap.yaml` - QUALITY-002

---

**Status**: 🚀 **IN PROGRESS** - Phase 1A starting

**Last Updated**: 2025-11-21

**Next Action**: Begin Phase 1A - CLI/Binary unwrap() replacement (222 calls)
