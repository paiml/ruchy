# Reaper Project Compilation Errors - Root Cause Analysis

**Date**: 2025-11-01
**Issue**: #111
**Current State**: 10 errors remaining (63 → 42 → 10 via v3.161.0-v3.164.0)
**Methodology**: Toyota Way (Five Whys, Genchi Genbutsu, Extreme TDD)

## Executive Summary

Despite 4 releases (v3.164.0 through v3.167.0) targeting Issue #111 errors, **7 E0308 errors persist** in the reaper project. This document applies **Five Whys root cause analysis** to understand why fixes that work in isolation fail in real-world code.

**Critical Discovery**: v3.165.0/v3.166.0 String fixes **DID NOT reduce error count** in reaper (stayed at 10 errors), indicating **pattern mismatch** between our test cases and real-world code.

## Error Breakdown (Current State)

| Error Type | Count | Status | Fixed By |
|------------|-------|--------|----------|
| E0308 (type mismatch) | 7 | ❌ UNSOLVED | v3.165.0/v3.166.0 attempted, failed |
| E0382 (use of moved value) | 1 | ❌ UNSOLVED | Not yet attempted |
| E0507 (cannot move out of Vec) | 2 | ⏳ PENDING | v3.167.0 (needs validation) |
| **TOTAL** | **10** | **90% unsolved** | |

## Code Churn Analysis (PMAT)

**Highest Churn Files** (since 2025-10-31):
1. `src/backend/transpiler/statements.rs` - **8 changes** (highest risk)
2. `src/backend/transpiler/mod.rs` - 3 changes
3. `src/backend/transpiler/expressions_helpers/field_access.rs` - 3 changes
4. `src/backend/transpiler/types.rs` - 2 changes

**Complexity Hotspots** (statements.rs):
- `transpile_call()` - **Complexity 31** (3× over limit)
- `transpile_let()` - **Complexity 30** (3× over limit)

**SATD Analysis**: Zero technical debt markers found ✅

**Interpretation**: High churn + high complexity in `statements.rs` indicates this is a **bug attractor zone**. The fact that it's been modified 8 times recently suggests repeated attempts to fix symptoms rather than root causes.

## Toyota Way Root Cause Analysis

### Problem Statement

**Symptom**: v3.165.0 and v3.166.0 were designed to fix E0308 String return type errors, but **error count remained at 10** in reaper project.

**Evidence**:
- v3.165.0 tested: String return type fixes work in isolation (3 tests passing)
- v3.166.0 tested: Vec indexing String return works in isolation (3 tests passing)
- Reaper v3.164.0: 10 errors
- Reaper v3.165.0: **Still 10 errors** (expected 3)
- Reaper v3.166.0: **Still 10 errors** (expected 3)

**Conclusion**: Our fixes target the wrong patterns.

---

## Five Whys Analysis: E0308 Errors

### Why #1: Why do 7 E0308 errors remain despite String fixes?

**Answer**: The E0308 errors in reaper are NOT caused by String return types.

**Evidence**:
- v3.165.0 fixed: `fun get() -> String { "text" }` ❌ Not the reaper pattern
- v3.166.0 fixed: `fun get() -> String { items[0] }` ❌ Not the reaper pattern
- Reaper errors: **Unknown patterns** (we never examined actual errors)

**Toyota Way Violation**: We violated **Genchi Genbutsu** (Go and See). We assumed the error patterns without examining actual reaper code.

### Why #2: Why did we assume the wrong patterns?

**Answer**: We created test cases based on HYPOTHETICAL patterns, not ACTUAL reaper errors.

**Evidence**:
- DEFECT-012 tests: Created synthetic String return examples
- DEFECT-013 tests: Created synthetic Vec indexing examples
- Never ran: `cargo build 2>&1` on actual reaper project to see REAL errors

**Toyota Way Violation**: **Stop and Fix** - we should have examined real errors before creating fixes.

### Why #3: Why didn't we examine real reaper errors?

**Answer**: No access to reaper codebase during development; worked from error count reports only.

**Evidence**:
- Only had: "10 errors" count from user reports
- Never saw: Actual error messages, file names, line numbers, code context
- Relied on: Issue #111 description mentioning "type mismatches"

**Root Cause**: **Insufficient problem definition** - we started fixing before fully understanding the problem.

### Why #4: Why did we start fixing without full understanding?

**Answer**: Pressure to reduce error count led to incremental symptom fixes rather than systematic root cause analysis.

**Evidence**:
- v3.164.0: Fixed 3 specific errors (42 → 10)
- v3.165.0: Assumed remaining 7 were String returns
- v3.166.0: Extended v3.165.0 with Vec indexing
- Pattern: **Incremental guessing** instead of **systematic investigation**

**Toyota Way Violation**: **Five Whys** - we stopped at "type mismatch" instead of drilling down to actual code patterns.

### Why #5: Why did we guess instead of investigate?

**Answer**: Lack of **scientific method** - no hypothesis testing, no verification with real data.

**Root Cause**: **Process failure** - we need a **mandatory investigation phase** before any fix:
1. ✅ Obtain actual error messages
2. ✅ Obtain code snippets causing errors
3. ✅ Reproduce errors in minimal test cases
4. ✅ Verify fix works on ACTUAL error, not synthetic example
5. ✅ Validate on real codebase before declaring "fixed"

---

## Five Whys Analysis: Why Fixes Don't Transfer to Real Code

### Why #1: Why do our fixes work in tests but not in reaper?

**Answer**: Test cases are **too simple** - they don't match the complexity of real-world code patterns.

**Evidence**:
```rust
// Our DEFECT-012 test (PASSES):
fun get() -> String { "text" }

// Possible real reaper pattern (UNKNOWN):
fun get_detection_rule(rules: Vec<Rule>) -> String {
    let rule = rules[0];
    match rule.pattern {
        Some(p) => p,  // ← E0308: expected String, found Option<&str>?
        None => "default"
    }
}
```

**Hypothesis**: Real errors involve **complex combinations**:
- Nested match expressions
- Option/Result unwrapping
- Struct field access
- Lifetime issues
- Generic type parameters

### Why #2: Why don't we test complex combinations?

**Answer**: **Extreme TDD** focuses on minimal reproducible cases, which may miss interaction patterns.

**Evidence**:
- DEFECT-012 test: Direct `return "text"`
- DEFECT-013 test: Direct `return items[0]`
- Missing: Match arms, nested expressions, field access, etc.

**Insight**: We need **property tests** that generate complex expression trees to find interaction bugs.

### Why #3: Why don't we have property tests for type conversions?

**Answer**: Property tests exist for parser, not for transpiler type conversions.

**Gap**: No systematic testing of:
- All expression types × String return type
- All expression types × Vec indexing
- Nested expressions × type coercion

### Why #4: Why no systematic transpiler testing?

**Answer**: Transpiler has **588 modules** but tests focus on isolated features, not integration.

**Evidence**:
- 4,031 library tests passing
- But: Complex real-world patterns fail
- Conclusion: **Coverage is not correctness**

### Why #5 (ROOT CAUSE): Why does coverage not equal correctness?

**Answer**: We test **what we built** instead of **what users need**.

**ROOT CAUSE**: **Missing feedback loop** from real-world usage to test suite.

---

## Genchi Genbutsu: What We Need to See

### Mandatory Investigation Checklist (Before Next Sprint)

To apply **Genchi Genbutsu** (Go and See), we MUST obtain:

#### 1. Actual Error Messages ✅ REQUIRED
```bash
cd ~/reaper
cargo build 2>&1 | grep "^error\[E" > /tmp/reaper_errors.txt
cargo build 2>&1 | grep -A 10 "^error\[E0308\]" > /tmp/reaper_e0308_errors.txt
```

**What we need**:
- Full error text
- File names and line numbers
- Expected vs found types
- Code snippets showing context

#### 2. Code Context ✅ REQUIRED
For EACH of the 7 E0308 errors:
```bash
# Extract code around each error
cargo build 2>&1 | grep -B 5 -A 10 "error\[E0308\]"
```

**What we need**:
- Function signatures
- Variable declarations
- Expression structure
- Type annotations (if any)

#### 3. Minimal Reproducible Examples ✅ REQUIRED
For EACH error, create:
```rust
// tests/reaper_e0308_error_1.rs
#[test]
fn test_reaper_e0308_actual_pattern_1() {
    // EXACT code from reaper that triggers E0308
    // NOT a synthetic example
}
```

#### 4. Pattern Analysis ✅ REQUIRED
Categorize errors by ROOT CAUSE:
- String vs &str conversions
- Option/Result unwrapping
- Reference vs owned types
- Generic type mismatches
- Lifetime issues
- Numeric type conversions

#### 5. Verification Protocol ✅ REQUIRED
```bash
# After implementing fix:
1. Fix works on reaper_e0308_error_1 test ✅
2. Fix works on reaper_e0308_error_2 test ✅
3. ... (all 7 errors)
4. cargo build in reaper project ✅
5. Error count reduced: 10 → 3 (7 E0308 fixed) ✅
```

---

## Hypothesis: What Might Be Causing E0308 Errors

Based on transpiler complexity analysis and common Rust patterns:

### Hypothesis 1: Option/Result Type Confusion
```rust
// Ruchy code:
fun get_name(opt: Option<String>) -> String {
    match opt {
        Some(s) => s,      // ← E0308: expected String, found &String?
        None => "default"  // ← E0308: expected String, found &str?
    }
}
```

**Five Whys**:
- Why? Match arms return different types
- Why? Option<String> returns &String when matched
- Why? Rust borrows by default in match
- Why? Our transpiler doesn't add .clone() or .to_string()
- ROOT CAUSE: Missing type coercion in match arms

### Hypothesis 2: Struct Field Access Returns References
```rust
// Ruchy code:
struct Config { name: String }
fun get_config_name(cfg: &Config) -> String {
    cfg.name  // ← E0308: expected String, found &String
}
```

**Five Whys**:
- Why? Field access on &Config returns &String
- Why? Rust borrows struct fields by default
- Why? Our transpiler doesn't clone field access
- Why? We only added clone to Vec indexing
- ROOT CAUSE: Field access needs .clone() for String return

### Hypothesis 3: Method Call Return Types
```rust
// Ruchy code:
fun process() -> String {
    let s = get_data();
    s.trim()  // ← E0308: expected String, found &str (trim returns &str)
}
```

**Five Whys**:
- Why? String methods return &str slices
- Why? Rust optimizes to avoid copying
- Why? Our transpiler doesn't convert &str to String
- Why? We only handle literals, not method returns
- ROOT CAUSE: Method call returns need .to_string()

### Hypothesis 4: Complex Expression Nesting
```rust
// Ruchy code:
fun complex() -> String {
    if condition {
        data[0].field.trim()  // ← E0308: Multiple conversion issues
    } else {
        "default"
    }
}
```

**Five Whys**:
- Why? Vec index + field access + method call + if expression
- Why? Each layer can introduce type issues
- Why? Our fixes are isolated (Vec, String, Field)
- Why? We don't handle combinations
- ROOT CAUSE: Need compositional type conversion

---

## E0382 Analysis: Use of Moved Value

**Count**: 1 error

### Five Whys

**Why #1**: Why does reaper have "use of moved value" error?

**Answer**: A value is used after being moved (e.g., passed to function, returned, moved into closure).

**Why #2**: Why doesn't Ruchy prevent this?

**Answer**: Ruchy allows writing code that Rust's borrow checker rejects.

**Why #3**: Why does transpiler generate invalid ownership patterns?

**Answer**: Transpiler doesn't track value lifetimes or ownership transfers.

**Why #4**: Why no ownership tracking?

**Answer**: Ruchy is designed as a high-level language without explicit ownership annotations.

**Why #5 (ROOT CAUSE)**: Why no automatic borrow/clone insertion?

**ROOT CAUSE**: Transpiler lacks **ownership analysis pass** to insert .clone() or borrow when needed.

**Solution**: Either:
1. **Conservative**: Auto-clone on first move
2. **Smart**: Analyze usage and only clone when reused
3. **Explicit**: Require Ruchy programmers to use .clone()

**Recommendation**: Start with Conservative (#1) for next sprint.

---

## E0507 Analysis: Cannot Move Out of Vec

**Count**: 2 errors

**Status**: Should be fixed by v3.167.0 (pending validation)

### Verification Needed

```bash
cd ~/reaper
cargo build 2>&1 | grep "E0507"
```

**Expected**: 0 E0507 errors (down from 2)

**If still failing**: Apply Five Whys to understand pattern mismatch.

---

## Code Quality Analysis

### High-Risk Areas (Complexity × Churn)

| File | Complexity | Churn | Risk Score | Issue |
|------|-----------|-------|------------|-------|
| statements.rs | 31 (transpile_call) | 8 changes | **248 (CRITICAL)** | Bug attractor |
| statements.rs | 30 (transpile_let) | 8 changes | **240 (CRITICAL)** | Bug attractor |
| types.rs | 28 (transpile_struct) | 2 changes | 56 (HIGH) | Needs refactor |
| field_access.rs | 10 | 3 changes | 30 (MEDIUM) | Acceptable |

**Risk Score** = Complexity × Churn

**Interpretation**:
- **statements.rs** is a **critical bug attractor** (248 risk score)
- Functions with complexity >10 violate Toyota Way standards
- 8 changes suggest **repeated symptomatic fixes** rather than root cause solutions

### Refactoring Priority

**MUST DO Before Next Sprint**:
1. **transpile_call()** - Complexity 31 → ≤10 (decompose into helper functions)
2. **transpile_let()** - Complexity 30 → ≤10 (extract pattern matching logic)
3. **transpile_struct()** - Complexity 28 → ≤10 (separate concerns)

**Toyota Way**: **Jidoka** (Built-in Quality) - complexity limits prevent bugs at design time.

---

## Sprint Plan: Solve ALL Remaining Errors

### Phase 1: Investigation (GENCHI GENBUTSU) - 2 hours

**Deliverables**:
1. ✅ Obtain all 10 error messages from reaper project
2. ✅ Extract code context for each error
3. ✅ Create minimal reproducible test for EACH error
4. ✅ Categorize errors by root cause pattern
5. ✅ Document findings in this file

**Tools**:
- `cargo build 2>&1 | tee /tmp/reaper_errors.log`
- `ruchydbg validate` - Check for hangs/crashes
- `pmat analyze complexity` - Find complexity hotspots in reaper .ruchy files

**Success Criteria**: Can reproduce all 10 errors in isolated tests

---

### Phase 2: RED Tests (EXTREME TDD) - 1 hour

**Deliverables**:
1. `tests/reaper_e0308_errors_RED.rs` - 7 tests, all marked #[ignore]
2. `tests/reaper_e0382_error_RED.rs` - 1 test, marked #[ignore]
3. `tests/reaper_e0507_errors_RED.rs` - 2 tests (verify v3.167.0 fix)

**Pattern**: EXACT code from reaper, not synthetic examples

**Example**:
```rust
#[test]
#[ignore] // RED: Will fail until fix implemented
fn test_reaper_e0308_option_match_arm() {
    let code = r#"
    // ACTUAL code from reaper that causes E0308
    fun get_name(opt: Option<String>) -> String {
        match opt {
            Some(s) => s,
            None => "default"
        }
    }
    "#;

    ruchy_cmd()
        .arg("compile")
        .arg(&temp_file)
        .assert()
        .success(); // RED: Will fail with E0308
}
```

**Success Criteria**: All tests fail with EXACT same errors as reaper

---

### Phase 3: ROOT CAUSE Fix (GREEN) - 4 hours

Based on Phase 1 findings, implement fixes for ROOT CAUSES:

#### Likely Root Cause 1: Match Arm Type Unification
**If**: E0308 errors in match expressions
**Then**: Implement match arm type coercion

```rust
// Add to src/backend/transpiler/statements.rs
fn unify_match_arm_types(&self, arms: &[MatchArm], return_type: &Type) -> Result<Vec<TokenStream>> {
    arms.iter().map(|arm| {
        let arm_tokens = self.transpile_expr(&arm.body)?;
        if self.needs_string_conversion(&arm.body, return_type) {
            Ok(quote! { (#arm_tokens).to_string() })
        } else {
            Ok(arm_tokens)
        }
    }).collect()
}
```

#### Likely Root Cause 2: Field Access Ownership
**If**: E0308 errors on struct field access
**Then**: Auto-clone field access when needed

```rust
// Add to src/backend/transpiler/expressions_helpers/field_access.rs
fn transpile_field_access(&self, object: &Expr, field: &str) -> Result<TokenStream> {
    let obj_tokens = self.transpile_expr(object)?;
    let field_ident = format_ident!("{}", field);

    // If field is String and we're in a move context, clone it
    if self.is_move_context() && self.field_type_is_string(object, field) {
        Ok(quote! { #obj_tokens.#field_ident.clone() })
    } else {
        Ok(quote! { #obj_tokens.#field_ident })
    }
}
```

#### Likely Root Cause 3: Method Call Return Coercion
**If**: E0308 errors on method calls
**Then**: Auto-convert &str method returns to String

```rust
// Known methods that return &str
const STR_RETURNING_METHODS: &[&str] = &["trim", "trim_start", "trim_end", "as_str"];

if STR_RETURNING_METHODS.contains(&method_name) && return_type_is_string() {
    Ok(quote! { (#method_tokens).to_string() })
}
```

#### Likely Root Cause 4: E0382 Ownership
**If**: Use of moved value
**Then**: Auto-clone on first move (conservative approach)

```rust
// Track variable moves in transpiler state
fn transpile_identifier(&self, name: &str) -> Result<TokenStream> {
    let ident = format_ident!("{}", name);

    if self.variable_was_moved(name) && self.variable_is_used_again(name) {
        // Auto-clone to prevent E0382
        Ok(quote! { #ident.clone() })
    } else {
        Ok(quote! { #ident })
    }
}
```

**Constraints**:
- Each fix MUST pass its corresponding RED test
- Complexity ≤10 for all new functions
- ZERO regressions (4,031 library tests must pass)
- PMAT A- minimum grade

---

### Phase 4: REFACTOR (Complexity Reduction) - 3 hours

**Mandatory Refactoring** (Technical Debt Paydown):

#### 1. Decompose transpile_call() (Complexity 31 → ≤10)
```rust
// BEFORE: 31 complexity
fn transpile_call(...) -> Result<TokenStream> {
    match ... {
        // 31 branches
    }
}

// AFTER: ≤10 complexity each
fn transpile_call(...) -> Result<TokenStream> {
    match self.categorize_call(callee) {
        CallCategory::Method => self.transpile_method_call(...),
        CallCategory::Function => self.transpile_function_call(...),
        CallCategory::Constructor => self.transpile_constructor_call(...),
    }
}

fn transpile_method_call(...) -> Result<TokenStream> { ... }  // Complexity ≤10
fn transpile_function_call(...) -> Result<TokenStream> { ... }  // Complexity ≤10
fn transpile_constructor_call(...) -> Result<TokenStream> { ... }  // Complexity ≤10
```

#### 2. Decompose transpile_let() (Complexity 30 → ≤10)
Extract pattern matching, destructuring, type inference into separate functions.

#### 3. Decompose transpile_struct() (Complexity 28 → ≤10)
Separate lifetime handling, derive attribute generation, field processing.

**Success Criteria**:
- All functions ≤10 complexity
- PMAT TDG A- or better on all modified files
- 4,031 tests still passing

---

### Phase 5: VALIDATION (Property + Mutation Tests) - 2 hours

#### Property Tests
```rust
// tests/property_tests/transpiler_type_conversions.rs
proptest! {
    #[test]
    fn test_all_expressions_with_string_return_compile(
        expr in arbitrary_expression(),
        func_name in "[a-z]{3,10}"
    ) {
        let code = format!(r#"
            fun {func_name}() -> String {{
                {expr}
            }}
        "#);

        // Property: All expressions should compile when function returns String
        let result = transpile_and_compile(code);
        prop_assert!(result.is_ok());
    }
}
```

#### Mutation Tests
```bash
cargo mutants --file src/backend/transpiler/statements.rs --timeout 300
cargo mutants --file src/backend/transpiler/expressions_helpers/field_access.rs --timeout 300
```

**Target**: ≥75% mutation coverage (mutants caught by tests)

---

### Phase 6: REAL-WORLD VALIDATION - 1 hour

**Critical Validation**:
```bash
cd ~/reaper
cargo build 2>&1 | grep "^error\[E" | wc -l
```

**Expected Progression**:
- Before: 10 errors
- After Phase 3: 0 errors (or close to 0)

**If errors remain**: STOP and apply Five Whys to each remaining error. Repeat Phase 1-3.

**Success Criteria**:
- Reaper project compiles successfully (0 errors)
- OR: Error count reduced to ≤2 with documented root causes

---

## Success Metrics

### Must-Have (Sprint Success Criteria)
- ✅ All 10 reaper errors reproduced in isolated tests
- ✅ All 10 RED tests created (marked #[ignore])
- ✅ All 10 GREEN fixes implemented
- ✅ All 10 tests passing (no #[ignore])
- ✅ Reaper project: 10 → 0 errors (100% fixed)
- ✅ 4,031 library tests: All passing (ZERO regressions)
- ✅ Complexity: All functions ≤10
- ✅ PMAT Grade: A- or better on all files
- ✅ Mutation Coverage: ≥75% on modified files

### Nice-to-Have (Quality Bonuses)
- ✅ Property tests for type conversions (10K+ random inputs)
- ✅ Mutation testing on all transpiler files
- ✅ Documentation of patterns in SPECIFICATION.md
- ✅ Benchmark performance impact of auto-cloning

---

## Toyota Way Principles Application

### 1. Genchi Genbutsu (Go and See)
- ❌ **VIOLATED**: We didn't examine actual reaper errors before fixing
- ✅ **FIX**: Phase 1 mandatory investigation of REAL errors

### 2. Five Whys (Root Cause Analysis)
- ❌ **VIOLATED**: Stopped at "type mismatch" without drilling deeper
- ✅ **FIX**: Applied Five Whys to each error category above

### 3. Stop the Line (Jidoka)
- ❌ **VIOLATED**: Kept releasing fixes that didn't reduce error count
- ✅ **FIX**: Phase 6 validation BLOCKS release if errors remain

### 4. Built-in Quality (Jidoka)
- ❌ **VIOLATED**: Complexity 31/30/28 functions (3× over limit)
- ✅ **FIX**: Phase 4 mandatory refactoring to ≤10 complexity

### 5. Kaizen (Continuous Improvement)
- ⚠️ **PARTIAL**: Incremental fixes (v3.164.0-v3.167.0) didn't address root causes
- ✅ **FIX**: Systematic investigation → root cause fix → refactor cycle

### 6. Respect for People
- ❌ **VIOLATED**: Wasted user time with fixes that didn't work
- ✅ **FIX**: Validate on real code BEFORE releasing

---

## Lessons Learned

### What Went Wrong

1. **Assumed patterns instead of investigating** (violated Genchi Genbutsu)
2. **Created synthetic tests instead of reproducing real errors** (violated scientific method)
3. **Released fixes without real-world validation** (violated Stop the Line)
4. **Allowed complexity >30 to persist** (violated Built-in Quality)
5. **Fixed symptoms (type mismatches) instead of root causes** (violated Five Whys)

### What We'll Do Differently

1. **Mandatory investigation phase** before any fix (Genchi Genbutsu)
2. **Tests based on REAL code**, not hypotheticals (Scientific Method)
3. **Validation on target codebase** before release (Stop the Line)
4. **Refactor complexity** as part of fix, not deferred (Jidoka)
5. **Five Whys to root cause** before writing code (Five Whys)

### How to Prevent Recurrence

**Process Change**:
```yaml
fix_workflow:
  1_investigate:
    - Obtain actual error messages
    - Extract code context
    - Create reproducible test from REAL code
    - Apply Five Whys to root cause
    - Document in docs/bugs/

  2_red_test:
    - Test based on actual error
    - Verify test reproduces exact error
    - Mark as #[ignore]

  3_green_fix:
    - Implement minimal fix
    - Complexity ≤10 (decompose if needed)
    - Pass RED test

  4_refactor:
    - PMAT quality gates
    - Mutation testing
    - Property testing

  5_validate:
    - Test on ACTUAL codebase
    - Error count MUST decrease
    - If not: STOP and return to step 1

  6_release:
    - Only if validation passed
    - Document error count reduction
    - Publish with verification data
```

---

## Next Sprint Action Items

### Before Starting Any Work

1. [ ] Obtain reaper project error log: `cargo build 2>&1 > /tmp/reaper_errors_v3.167.0.log`
2. [ ] Extract all E0308 errors with context
3. [ ] Extract all E0382 errors with context
4. [ ] Extract all E0507 errors with context
5. [ ] Create minimal reproducible test for EACH error
6. [ ] Apply Five Whys to each error pattern
7. [ ] Document root causes in this file

### Sprint Execution

**Day 1: Investigation** (Genchi Genbutsu)
- Hours 1-2: Obtain and analyze real errors
- Hours 3-4: Create RED tests from real patterns
- **Deliverable**: 10 RED tests reproducing all errors

**Day 2: Implementation** (GREEN)
- Hours 1-4: Implement root cause fixes
- Hours 5-6: Refactor complexity hotspots
- **Deliverable**: All tests passing, complexity ≤10

**Day 3: Validation** (Quality)
- Hours 1-2: Property tests + Mutation tests
- Hours 3-4: Validate on reaper project
- **Deliverable**: 0 errors in reaper, release v3.168.0

### Definition of Done

- [ ] All 10 reaper errors reproduced in tests
- [ ] All 10 tests passing (GREEN)
- [ ] Reaper project: 0 compilation errors
- [ ] 4,031 library tests: All passing
- [ ] Complexity: All functions ≤10
- [ ] PMAT Grade: A- or better
- [ ] Mutation Coverage: ≥75%
- [ ] Documentation: Patterns added to SPECIFICATION.md
- [ ] Released: v3.168.0 published to crates.io
- [ ] Verified: User confirms 0 errors in reaper

---

## References

- **Issue**: https://github.com/paiml/ruchy/issues/111
- **Previous Releases**: v3.161.0 (enum), v3.162.0 (format), v3.164.0 (pattern), v3.165.0 (String), v3.166.0 (Vec String), v3.167.0 (ownership)
- **Tools**: ruchydbg, pmat, cargo-mutants, proptest
- **Methodology**: Toyota Way (Genchi Genbutsu, Five Whys, Jidoka, Kaizen)

---

**Status**: Ready for next sprint execution following Toyota Way principles
**Owner**: Ruchy Development Team
**Priority**: CRITICAL (blocks reaper project deployment)
