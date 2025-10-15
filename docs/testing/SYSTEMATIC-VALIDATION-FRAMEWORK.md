# Systematic Validation Framework

**Purpose**: Stop "whack-a-mole" bug fixing by validating ALL Ruchy tools systematically
**Created**: 2025-10-15 (Response to BUG-037 and user request)
**Toyota Way**: Jidoka - Build quality into the process, not bolt it on later

---

## Problem Statement

Prior to this framework, Ruchy development suffered from:
- **Whack-a-mole bug fixing**: Fix one tool, break another
- **No systematic validation**: Each tool tested in isolation
- **Missing integration tests**: Tools work alone but fail together
- **Regression-prone**: No way to catch breakage early

**Critical Example**: BUG-037 revealed that `assert_eq` was completely unimplemented, yet test runner reported SUCCESS. This could only happen without systematic validation.

---

## Solution: Three-Layer Validation

### Layer 1: Systematic Tool Validation (29 tests)

**File**: `tests/systematic_tool_validation.rs`
**Coverage**: ALL 15 Ruchy tools
**Methodology**: assert_cmd for deterministic CLI testing

**Test Structure** (per tool):
1. **Smoke test**: Basic functionality works
2. **Example validation**: `cargo run --example` enforcement
3. **Error handling**: Negative tests catch failures gracefully
4. **Integration**: Tools work together on same program

**Results**:
- ✅ 29 tests passing
- 🟡 3 tests ignored (documented limitations)
- ⏱️ ~13 seconds total runtime
- 📊 Coverage: check, transpile, run, eval, test, lint, compile, ast, wasm, notebook, coverage, runtime, provability, property-tests, mutations

**Key Achievement**: `integration_all_tools_on_single_program` - runs 6 tools on one program to catch interaction bugs

### Layer 2: Interactive CLI Validation (20 tests)

**File**: `tests/cli_interactive_validation.rs`
**Coverage**: REPL, TTY detection, signal handling, pipes, redirection
**Methodology**: rexpect for PTY-based interactive testing

**Test Categories**:
1. **REPL Interactive**: Arithmetic, functions, error recovery
2. **CLI Commands**: run, eval, check, test
3. **Signal Handling**: Ctrl+C graceful exit
4. **TTY Detection**: Interactive vs non-interactive modes
5. **Pipes/Redirection**: stdin, stdout, stderr
6. **Error Messages**: Actionable, clear, with context
7. **Performance**: Long-running scripts, memory intensive

**Why rexpect?**:
- Catches TTY-specific bugs (colors, prompts, readline)
- Tests signal handling (SIGINT, SIGTERM)
- Validates actual user experience

### Layer 3: Unit Test Validation (Existing)

**Files**:
- `tests/bug_037_test_assertions_dont_fail.rs` (6 tests - EXTREME TDD)
- `tests/fifteen_tool_validation.rs` (21 tests)
- Plus 3800+ existing tests

---

## BUG-037 Fix Details (EXTREME TDD)

### Problem
Test runner reported PASS even when assertions failed.

### Root Cause (TWO bugs!)
1. **Test functions not executed**: `run_test_file()` only defined functions, never called them
2. **`assert_eq` not implemented**: Built-in assertion functions missing entirely

### Solution

#### Fix 1: Execute Test Functions
**File**: `src/bin/handlers/handlers_modules/test_helpers.rs`

**Before**:
```rust
pub fn run_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    let test_content = read_file_with_context(test_file)?;
    let mut repl = Repl::new(std::env::temp_dir())?;
    repl.evaluate_expr_str(&test_content, None)?;  // Only defines, never calls!
    Ok(())
}
```

**After**:
```rust
pub fn run_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    let test_content = read_file_with_context(test_file)?;

    // Parse AST to find @test functions
    let mut parser = Parser::new(&test_content);
    let ast = parser.parse()?;
    let test_functions = extract_test_functions(&ast)?;

    // Load file (defines functions)
    let mut repl = Repl::new(std::env::temp_dir())?;
    repl.evaluate_expr_str(&test_content, None)?;

    // Execute each test function
    for test_fn_name in test_functions {
        let call_expr = format!("{}()", test_fn_name);
        repl.evaluate_expr_str(&call_expr, None)?;  // Assertions now execute!
    }

    Ok(())
}
```

**Key**: `extract_test_functions()` handles both single functions and blocks of functions

#### Fix 2: Implement Assertions
**Files**:
- `src/runtime/builtins.rs` (function implementations)
- `src/runtime/builtin_init.rs` (registration)
- `src/runtime/eval_builtin.rs` (dispatcher)
- `src/runtime/interpreter.rs` (new error variant)
- `src/runtime/eval_display.rs` (error display)

**Implementation**:
```rust
// New error type
pub enum InterpreterError {
    // ... existing variants
    AssertionFailed(String),  // BUG-037: Test assertions
}

// Built-in functions
fn eval_assert_eq(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() < 2 {
        return Err(InterpreterError::RuntimeError(
            "assert_eq() expects at least 2 arguments".to_string(),
        ));
    }

    let expected = &args[0];
    let actual = &args[1];
    let message = if args.len() > 2 {
        format!("{}", args[2])
    } else {
        format!("Assertion failed: expected {:?}, got {:?}", expected, actual)
    };

    if expected != actual {
        Err(InterpreterError::AssertionFailed(message))
    } else {
        Ok(Value::Nil)
    }
}
```

**Complexity**: 3 (well within ≤10 limit)

### Test Results

**RED Phase** (before fix):
```
test test_bug_037_red_failing_assertion_should_fail ... FAILED  ❌
test test_bug_037_baseline_passing_assertion_passes ... FAILED  ❌
test test_bug_037_red_mixed_results ... FAILED  ❌
test test_bug_037_baseline_no_assertions ... PASSED  ✅
test test_bug_037_red_test_functions_execute ... FAILED  ❌
test test_bug_037_red_phase_summary ... PASSED  ✅
```

**GREEN Phase** (after fix):
```
test test_bug_037_red_failing_assertion_should_fail ... PASSED  ✅
test test_bug_037_baseline_passing_assertion_passes ... PASSED  ✅
test test_bug_037_red_mixed_results ... PASSED  ✅
test test_bug_037_baseline_no_assertions ... PASSED  ✅
test test_bug_037_red_test_functions_execute ... PASSED  ✅
test test_bug_037_red_phase_summary ... PASSED  ✅
```

**All 6 tests passing!** ✅

---

## Usage

### Running Systematic Validation

```bash
# Quick validation (29 tests, ~13s)
cargo test --test systematic_tool_validation

# Full validation including interactive tests
cargo test --test systematic_tool_validation -- --ignored --nocapture

# Interactive CLI validation (requires PTY)
cargo test --test cli_interactive_validation -- --ignored

# BUG-037 regression tests
cargo test --test bug_037_test_assertions_dont_fail
```

### Adding New Tools

When adding a new Ruchy tool:

1. **Add to systematic_tool_validation.rs**:
```rust
#[test]
fn tool_XX_new_tool_smoke_test() {
    // Basic functionality
}

#[test]
fn tool_XX_new_tool_error_handling() {
    // Negative test
}

#[test]
fn tool_XX_new_tool_example_validation() {
    // cargo run --example verification
}
```

2. **Add interactive test if applicable**:
```rust
#[test]
#[ignore]
fn new_tool_interactive_usage() {
    // rexpect-based test
}
```

3. **Update integration test**:
Add to `integration_all_tools_on_single_program` if tool should work with others

---

## Known Limitations (Documented)

### Multiple Test Functions Not Detected

**Issue**: When multiple `@test` functions are at top level, only first is detected
**Root Cause**: Parser doesn't wrap multiple top-level items in Block
**Workaround**: Put test functions inside a module or use single test per file
**Tracked**: Ignored test `tool_05_test_runs_multiple_tests`

**Example of Problem**:
```ruchy
@test("first")
fun test_one() { assert_eq(1, 1) }

@test("second")
fun test_two() { assert_eq(2, 2) }  // NOT DETECTED
```

**Workaround**:
```ruchy
// Put in separate files
// OR wrap in module (when module system complete)
```

### Notebook Requires Server Setup

**Issue**: Notebook acceptance tests need async server
**Solution**: Separate integration test suite
**Tracked**: Ignored test `tool_10_notebook_example_validation`

### Interactive Tests Require PTY

**Issue**: rexpect tests need pseudo-terminal
**Solution**: Tests are `#[ignore]` by default, run explicitly in CI with PTY support
**Command**: `cargo test -- --ignored` (only in environments with PTY)

---

## Success Metrics

### Before Framework
- ❌ No systematic tool validation
- ❌ Bugs caught late in development
- ❌ Regression detection: manual testing only
- ❌ Example validation: none

### After Framework
- ✅ 29 systematic tests covering all 15 tools
- ✅ 20 interactive CLI tests with rexpect
- ✅ `cargo run --example` enforcement
- ✅ Integration test for tool interaction
- ✅ ~13 second validation suite
- ✅ CI-friendly (assert_cmd deterministic)
- ✅ PTY tests optional but available

### Impact
- **BUG-037 Fixed**: Assertions now work correctly
- **Regression Prevention**: All tools validated on every commit
- **Faster Development**: Catch bugs in seconds, not days
- **Higher Confidence**: Know immediately if change breaks anything

---

## Toyota Way Principles Applied

1. **Jidoka (Autonomation)**: Build quality into the process
   - Automated validation runs on every commit
   - Tests fail fast with clear error messages

2. **Genchi Genbutsu (Go and See)**: Direct observation
   - rexpect tests run actual CLI in real environment
   - Integration tests verify real user workflows

3. **Kaizen (Continuous Improvement)**: Systematic problem-solving
   - Each bug gets comprehensive test suite
   - Framework prevents similar bugs in future

4. **Stop the Line**: No defect is too small
   - BUG-037 treated as CRITICAL despite being "just assertions"
   - Created comprehensive framework to prevent recurrence

---

## Future Enhancements

### Property Testing (Planned)
- Random input generation for robustness
- Invariant checking across all tools
- Fuzz testing integration

### Mutation Testing (Planned)
- Verify tests actually catch bugs
- Target ≥75% mutation coverage
- Integrate with cargo-mutants

### Performance Regression Detection
- Benchmark suite for all tools
- Automatic regression detection
- Performance budgets per tool

---

## References

- **EXTREME TDD**: tests/bug_037_test_assertions_dont_fail.rs
- **Systematic Validation**: tests/systematic_tool_validation.rs
- **Interactive Tests**: tests/cli_interactive_validation.rs
- **CLAUDE.md**: Project development protocol
- **Toyota Way**: docs/execution/roadmap.md

---

**Last Updated**: 2025-10-15
**Status**: ✅ COMPLETE - All validation frameworks operational
