# Sub-spec: Language Completeness Book — Quality Gates, Execution & Walkthrough

**Parent:** [language-completeness-documentation-book.md](../language-completeness-documentation-book.md) Sections 7-13

---

## 🔬 QUALITY GATES (BLOCKING)

### Per-Feature Gates (Must Pass Before Merge)

```bash
# GATE 1: PMAT Quality (BLOCKING)
pmat tdg docs/lang-completeness-book/ --min-grade A- --fail-on-violation
pmat tdg examples/lang_comp/ --min-grade B+ --fail-on-violation

# GATE 2: Test Coverage (BLOCKING)
cargo test lang_comp_ --all-features
# Required: 100% of feature tests passing

# GATE 3: Property Tests (BLOCKING)
cargo test property_tests --package ruchy -- --ignored
# Required: ≥10,000 cases per feature, 100% pass

# GATE 4: Mutation Tests (BLOCKING)
cargo mutants --package ruchy --file tests/lang_comp/**
# Required: ≥75% mutation coverage

# GATE 5: Native Tool Validation (BLOCKING)
./docs/lang-completeness-book/test-suite/validate_tooling.sh
# Required: ALL tools pass on ALL examples

# GATE 6: Book Compatibility (BLOCKING)
./.pmat/test_book_compat.sh
# Required: Maintain 100% compatibility

# GATE 7: Zero Regressions (BLOCKING)
cargo test --all-features
# Required: ALL existing tests still pass
```

### Sprint-Level Gates (Before Release)

```bash
# Comprehensive validation
pmat tdg . --format=sarif --output=quality-gate-report.sarif
pmat quality-gate --fail-on-violation --format=detailed

# GitHub Pages build test
mdbook build docs/lang-completeness-book/
# Required: Builds without errors

# End-to-end validation
./docs/lang-completeness-book/test-suite/run_all_tests.sh
# Required: 100% pass rate
```

---

## 📦 DELIVERABLES

### Per-Feature Deliverables (MANDATORY)

For each `LANG-COMP-XXX` ticket:

1. **Documentation**
   - Feature documentation (Markdown)
   - Code examples (`.ruchy` files)
   - Test files (unit, property, mutation)

2. **Quality Reports**
   - PMAT TDG score report
   - Test coverage report
   - Mutation test report
   - Tool validation report

3. **GitHub Pages Content**
   - mdbook-compatible Markdown
   - Rendered HTML (auto-generated)
   - Working code playground links

4. **Roadmap Updates**
   - Ticket status: COMPLETE
   - Metrics updated
   - Known issues documented

### Sprint-Level Deliverables

1. **Quality Dashboard**
   - `docs/quality/LANG_COMP_DASHBOARD.md`
   - Real-time metrics
   - Progress tracking

2. **Test Suite**
   - Master test runner
   - CI/CD integration
   - Performance benchmarks

3. **GitHub Pages Site**
   - https://[org].github.io/ruchy/lang-completeness/
   - Interactive examples
   - Search functionality

---

## 🚀 EXECUTION WORKFLOW

### Starting a New Feature

```bash
# 1. Create roadmap ticket
# Edit: docs/execution/roadmap.md
# Add: LANG-COMP-XXX: [Feature] Documentation & Validation

# 2. Run baseline quality check
pmat tdg . --min-grade A- --fail-on-violation

# 3. Create branch (per CLAUDE.md: work on main)
# NO BRANCHING - work directly on main

# 4. Create directory structure
mkdir -p docs/lang-completeness-book/[category]/tests
mkdir -p examples/lang_comp/[category]

# 5. Start RED phase: Write tests FIRST
# Create: tests/lang_comp/[category]/[feature]_test.rs
```

### During Development

```bash
# RED: Write failing test
cargo test [feature]_test -- --nocapture
# Expected: FAIL (test exists, example doesn't)

# GREEN: Create example
ruchy run examples/lang_comp/[category]/[file].ruchy
# Expected: Works correctly

# REFACTOR: Validate with tools
ruchy lint examples/lang_comp/[category]/[file].ruchy
ruchy test examples/lang_comp/[category]/[file].ruchy
ruchy score examples/lang_comp/[category]/[file].ruchy
ruchy compile examples/lang_comp/[category]/[file].ruchy
ruchy wasm examples/lang_comp/[category]/[file].ruchy

# If ANY tool fails: STOP THE LINE
```

### Completing a Feature

```bash
# 1. Run all quality gates
pmat tdg . --min-grade A- --fail-on-violation
cargo test lang_comp_ --all-features
cargo test property_tests --package ruchy -- --ignored

# 2. Run mutation tests
cargo mutants --package ruchy --file tests/lang_comp/[category]/

# 3. Generate reports
pmat tdg docs/lang-completeness-book/[category]/ --format=markdown --output=docs/quality/[category]_report.md

# 4. Update roadmap
# Mark: LANG-COMP-XXX: ✅ COMPLETE

# 5. Commit with quality metrics
git add .
git commit -m "[LANG-COMP-XXX] [Feature] Documentation & Validation

TDG Score: [score]
Property Tests: [count] cases, 100% pass
Mutation Coverage: [%]
Tool Validation: 5/5 tools passing

Closes: LANG-COMP-XXX
"
```

---

## 🎓 EXAMPLE: LANG-COMP-001 Walkthrough

### Ticket: LANG-COMP-001: Basic Syntax Documentation & Validation

#### Step 1: RED Phase - Write Tests

```rust
// File: tests/lang_comp/basic_syntax/variables_test.rs

use std::process::Command;

#[test]
fn test_variable_let_binding() {
    let output = Command::new("ruchy")
        .args(["run", "examples/lang_comp/01-basic-syntax/01_variables.ruchy"])
        .output()
        .expect("Failed to run example");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use std::process::Command;

    proptest! {
        #[test]
        fn variable_binding_never_crashes(value in any::<i64>()) {
            let code = format!("let x = {}\nx", value);
            let output = Command::new("ruchy")
                .args(["eval", &code])
                .output()
                .expect("Failed to run ruchy");

            prop_assert!(
                output.status.success() ||
                String::from_utf8_lossy(&output.stderr).contains("error")
            );
        }
    }
}
```

Run: `cargo test test_variable_let_binding`
Expected: **FAIL** (example doesn't exist yet)

#### Step 2: GREEN Phase - Create Example

```ruchy
// File: examples/lang_comp/01-basic-syntax/01_variables.ruchy
// Description: Variable declaration with let binding
// Status: VALIDATED
// Quality: TDG A-, Property 10K cases, Mutation 85%

let x = 42
x
```

Run: `ruchy run examples/lang_comp/01-basic-syntax/01_variables.ruchy`
Expected: Output `42`

Run: `cargo test test_variable_let_binding`
Expected: **PASS**

#### Step 3: REFACTOR Phase - Validate Tools

```bash
# Lint
$ ruchy lint examples/lang_comp/01-basic-syntax/01_variables.ruchy
✅ No issues found

# Test
$ ruchy test examples/lang_comp/01-basic-syntax/01_variables.ruchy
✅ All tests passed

# Score
$ ruchy score examples/lang_comp/01-basic-syntax/01_variables.ruchy
✅ Quality Score: 0.95/1.0

# Compile
$ ruchy compile examples/lang_comp/01-basic-syntax/01_variables.ruchy
✅ Compiled successfully to Rust

# WASM
$ ruchy wasm examples/lang_comp/01-basic-syntax/01_variables.ruchy
✅ Compiled successfully to WASM
```

**ALL TOOLS PASS** ✅

#### Step 4: Property Tests

```bash
$ cargo test property_tests::variable_binding_never_crashes -- --ignored --nocapture
running 1 test
test property_tests::variable_binding_never_crashes ... ok (10000 cases)

test result: ok. 1 passed; 0 failed
```

**10,000 CASES PASS** ✅

#### Step 5: Mutation Tests

```bash
$ cargo mutants --package ruchy --file tests/lang_comp/basic_syntax/variables_test.rs
Found 12 mutants, tested 12, caught 10, missed 2
Mutation coverage: 83.3%
```

**≥75% COVERAGE ACHIEVED** ✅

#### Step 6: Documentation

```markdown
# Variables

**Status**: ✅ VALIDATED
**Tested**: 2025-10-06
**Ruchy Version**: 3.68.0
**Quality Score**: 0.95/1.0

## Overview

Variables in Ruchy are declared using `let` keyword with immutable bindings by default.

## Syntax

\`\`\`ruchy
let identifier = expression
\`\`\`

## Examples

### Example 1: Basic Variable Declaration

**File**: `examples/lang_comp/01-basic-syntax/01_variables.ruchy`

\`\`\`ruchy
let x = 42
x  // Output: 42
\`\`\`

**Validation**:
```bash
# Unit test: ✅ PASS
# Property test: ✅ PASS (10,000 cases)
# Mutation test: ✅ PASS (10/12 mutants caught, 83.3%)
# ruchy lint: ✅ PASS
# ruchy test: ✅ PASS
# ruchy score: ✅ 0.95/1.0
# ruchy compile: ✅ PASS
# ruchy wasm: ✅ PASS
```

[... rest of documentation ...]
```

#### Step 7: Commit

```bash
$ git add .
$ git commit -m "[LANG-COMP-001] Basic Syntax: Variables Documentation & Validation

Feature: Variable declaration with let bindings
Examples: 1 working example created
Tests: Unit (1) + Property (10K cases) + Mutation (12 mutants)

TDG Score: A- (85.2)
Property Tests: 10,000 cases, 100% pass
Mutation Coverage: 83.3%
Tool Validation: 5/5 tools passing

Quality Metrics:
- ruchy lint: ✅ PASS
- ruchy test: ✅ PASS
- ruchy score: ✅ 0.95/1.0
- ruchy compile: ✅ PASS
- ruchy wasm: ✅ PASS

Closes: LANG-COMP-001
"
```

---

## 📈 SUCCESS METRICS

### Sprint-Level KPIs

- **Features Documented**: Target 5-10 per sprint
- **PMAT TDG Score**: Maintain A- or higher
- **Property Test Coverage**: ≥80% of all features
- **Mutation Coverage**: ≥75% per feature
- **Tool Pass Rate**: 100% (all tools on all examples)
- **Bug Discovery Rate**: Document and fix immediately
- **Documentation Quality**: mdbook builds without errors

### Project-Level Goals

- **Complete Core Language**: 100% of Phase 1 (10 features)
- **Complete Advanced Features**: 100% of Phase 2 (10 features)
- **Complete Tooling**: 100% of Phase 3 (5 features)
- **GitHub Pages Deployment**: Live and maintained
- **Zero Ambiguity**: Every feature fully documented
- **Production Ready Marker**: All features validated

---

## 🚨 CRITICAL RULES (MANDATORY)

### STOP THE LINE Conditions

**IMMEDIATELY HALT ALL WORK IF:**

1. **Tooling Bug Discovered**
   - ANY `ruchy` command fails on valid code
   - Create `TOOL-BUG-XXX` ticket immediately
   - Fix before resuming documentation

2. **Language Bug Discovered**
   - Feature doesn't work as documented
   - Create `LANG-BUG-XXX` ticket immediately
   - Apply EXTREME TDD to fix

3. **Quality Gate Failure**
   - PMAT TDG drops below A-
   - Any test regression detected
   - Mutation coverage drops below 75%

4. **Breaking Change Detected**
   - Existing examples stop working
   - Tool output changes unexpectedly
   - REPL behavior changes

### Zero Tolerance Policies

1. **No Guessing**: Only document what's empirically validated
2. **No Regression**: ALL existing tests must pass
3. **No Shortcuts**: ALL quality gates must pass
4. **No SATD**: No TODO/FIXME in production code
5. **No Low Quality**: ALL code ≤10 complexity

---

## 🎯 PRIORITY ORDER (THIS SPRINT)

### Immediate Actions (Week 1)

1. **Setup Infrastructure**
   - Create directory structure
   - Setup test framework
   - Configure mdbook
   - Create master test runner

2. **LANG-COMP-001: Basic Syntax**
   - Variables (let bindings)
   - Literals (numbers, strings, booleans)
   - Comments

3. **LANG-COMP-002: Operators**
   - Arithmetic operators
   - Comparison operators
   - Logical operators

### Near-Term (Week 2-3)

4. **LANG-COMP-003: Control Flow**
5. **LANG-COMP-004: Functions**
6. **LANG-COMP-005: Closures**

### Medium-Term (Week 4+)

Continue with remaining Phase 1 features...

---

## 📚 REFERENCES

### External Documentation

- **paiml-mcp-agent-toolkit**: `../paiml-mcp-agent-toolkit/docs/toyota-way-tdd-pattern-library.md`
- **Property Test Guide**: `../paiml-mcp-agent-toolkit/docs/testing/property-based.md`
- **Quality Standards**: `../paiml-mcp-agent-toolkit/docs/sprint-91-property-test-perfection.md`

### Internal Documentation

- **Ruchy CLAUDE.md**: Project development guidelines
- **Roadmap**: `docs/execution/roadmap.md`
- **SPECIFICATION.md**: Language specification

---

**END OF SPECIFICATION**

**Next Action**: Begin LANG-COMP-001 implementation following this spec exactly.
