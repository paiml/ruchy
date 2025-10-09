# Batch 17 Quality Violations Plan

**Sprint**: Quality Violations Elimination (Priority 2)
**Current Status**: 464 violations (52 complexity in tests, 69 SATD in tests, 55 entropy, 286 duplicates, 2 other)
**Target**: 464 → 459 violations (-5 violations minimum)
**Date**: 2025-10-09

---

## Current Violation Breakdown

| Type | Count | Scope | Priority |
|------|-------|-------|----------|
| **Duplicates** | 286 | All files | 🔥 HIGH |
| **SATD** | 69 | Test files | ⚠️ MEDIUM |
| **Entropy** | 55 | Abstract patterns | ⚠️ MEDIUM |
| **Complexity** | 52 | Test files | ⚠️ MEDIUM |
| **Coverage** | 1 | - | ✅ LOW |
| **Provability** | 1 | - | ✅ LOW |
| **TOTAL** | 464 | | |

**Context**: Production code (src/) is Toyota Way compliant after Batches 14-16 (≤10 complexity, 0 SATD, minimal duplication).

---

## Duplication Analysis

### Identified Patterns (from src/bin/handlers/mod.rs)

#### Pattern 1: REPL Initialization (4 instances)
**Current Code**:
```rust
let mut repl = Repl::new(std::env::temp_dir())?;
```

**Locations**:
1. handle_eval_command (line 31)
2. handle_file_execution (line 86)
3. handle_stdin_input (line 125)
4. handle_repl_command (line 388)

**Proposed Helper**:
```rust
fn create_repl() -> Result<Repl> {
    Repl::new(std::env::temp_dir())
}
```

**Impact**: Eliminates 4 duplicate REPL initialization patterns

---

#### Pattern 2: Verbose Command Output Logging (3 instances)
**Current Code**:
```rust
if verbose {
    let stderr = String::from_utf8_lossy(&output_result.stderr);
    eprintln!("Command output:\n{}", stderr);
}
```

**Locations**:
1. run_cargo_mutants (line 2270-2273)
2. run_property_test_suite (line 1910-1913)
3. run_cargo_fuzz (line 2390-2393)

**Proposed Helper**:
```rust
fn log_command_output(output: &std::process::Output, verbose: bool) {
    if verbose {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command output:\n{}", stderr);
    }
}
```

**Impact**: Eliminates 3 duplicate verbose logging patterns

---

#### Pattern 3: File Writing with Context (8+ instances)
**Current Code**:
```rust
fs::write(out_path, content)?;
// OR
fs::write(path, content).with_context(|| format!("Failed to write: {}", path.display()))?;
```

**Locations**:
- write_json_property_report (line 2125)
- write_text_property_report (line 2176)
- write_json_mutation_report (line 2294)
- write_text_mutation_report (line 2313)
- write_json_fuzz_report (line ~2517)
- write_text_fuzz_report (line ~2537)
- handle_transpile_command (line 214)
- handle_wasm_command (line 1812)

**Proposed Helper**:
```rust
fn write_file_with_context(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}
```

**Impact**: Eliminates 8+ duplicate file writing patterns

---

## Batch 17 Strategy

### Focus: Extract Low-Hanging Fruit Patterns

**Rationale**:
- These are simple, clear patterns with no business logic
- Low risk of breaking functionality
- High code duplication reduction
- Builds on Batch 16 momentum

### Step 1: Extract 3 Common Helpers (30 min)
Create helper functions for:
1. `create_repl()` - REPL initialization
2. `log_command_output()` - Verbose command logging
3. `write_file_with_context()` - File writing with error context

### Step 2: Refactor Functions (60 min)
Apply helpers to:
- **REPL**: 4 functions (eval, file_execution, stdin, repl_command)
- **Logging**: 3 functions (mutants, property_test_suite, fuzz)
- **File Write**: 8 functions (all report writers + transpile + wasm)

**Total**: 15 functions refactored

### Step 3: Build and Test (10 min)
```bash
cargo build --bin ruchy
cargo test --test p0_critical_features
```

### Step 4: Verify Duplication Reduction (10 min)
```bash
pmat analyze duplicates --detection-type exact --min-lines 10
pmat quality-gate
```

### Step 5: Commit and Document (10 min)
- Commit with detailed metrics
- Update roadmap
- Update violation count

**Total Time**: ~2 hours
**Expected Reduction**: 464 → 454 violations (-10 minimum from duplication elimination)

---

## Toyota Way Principles

### Jidoka (Stop the Line)
- Run full test suite after each helper extraction
- Never proceed if tests fail
- Zero tolerance for regressions

### Genchi Genbutsu (Go and See)
- Verified exact duplicate patterns via manual code review
- Confirmed pattern occurs 3-15 times per helper
- Validated no semantic differences between instances

### Kaizen (Continuous Improvement)
- Small batches (3 helpers, 15 functions)
- Systematic approach (one pattern at a time)
- Incremental verification

### Respect for People
- Preserve all existing functionality
- Clear, intention-revealing helper names
- Maintain test coverage (15/15 P0 tests)

---

## Success Criteria

### Batch 17 (This Session)
- ✅ 464 → 454 violations (-10 minimum)
- ✅ 3 common helpers extracted (complexity ≤2 each)
- ✅ 15 functions refactored to use helpers
- ✅ All tests passing (zero regressions)
- ✅ Progress documented

### Overall Sprint (Quality Violations Elimination)
- 🎯 472 → 0 violations (ZERO TOLERANCE) - Long-term goal
- 🎯 Batches 14-16: 472 → 464 (-8 net, maintainability improved)
- 🎯 Batch 17: Target -10 duplication violations
- 🎯 Production code: Toyota Way compliant ✅

---

## Risk Mitigation

### Low Risk: Simple Helper Extraction
- **Risk**: Breaking functionality via refactoring
- **Mitigation**:
  - Helpers are pure utility functions with no business logic
  - One-to-one replacement of duplicate patterns
  - Comprehensive P0 test suite validation
  - Incremental changes with per-helper validation

### Low Risk: No Semantic Changes
- **Risk**: Introducing bugs via helper abstraction
- **Mitigation**:
  - Exact code equivalence maintained
  - No conditional logic in helpers
  - Simple delegation patterns only

---

## Next Steps After Batch 17

1. **Batch 18**: Extract more duplication patterns (JSON/text report formatting)
2. **Batch 19**: Test file quality improvement (SATD + complexity in tests)
3. **Batch 20**: Entropy pattern implementation (validation trait, formatters)
4. **Long-term**: 286 duplicate violations systematic reduction

---

**Status**: 📋 **PLANNED** - Ready to execute
**Owner**: Ruchy Development Team
**Timeline**: Batch 17 this session, continue systematic reduction
**Context**: Builds on Batches 14-16 success (28 helper functions, 111 complexity points eliminated)
