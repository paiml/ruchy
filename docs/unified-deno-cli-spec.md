# Unified Deno-Style CLI Specification for Ruchy

**Version**: 1.0.0
**Date**: 2025-10-14
**Status**: 🔴 STOP THE LINE - Critical UX Issue
**Methodology**: EXTREME TDD + Property Tests + Mutation Tests + rexpect CLI Tests

---

## Executive Summary

**PROBLEM**: Ruchy's CLI is inconsistent and confusing:
- `ruchy script.ruchy` interprets (fast, correct ✅)
- `ruchy run script.ruchy` compiles (slow, confusing ❌)
- `ruchy` shows help instead of REPL ❌

**SOLUTION**: Adopt Deno's proven CLI design for consistency and speed.

---

## Current State Analysis (v3.81.0)

### What Works ✅

| Command | Behavior | Speed | Status |
|---------|----------|-------|--------|
| `ruchy script.ruchy` | Interprets via REPL | 0.003s (3ms) | ✅ PERFECT |
| `ruchy -e "code"` | Interprets one-liner | <0.01s | ✅ PERFECT |
| `echo "code" \| ruchy` | Interprets stdin | <0.01s | ✅ PERFECT |
| `ruchy repl` | Interactive REPL | Instant | ✅ PERFECT |
| `ruchy compile file -o bin` | Rustc compile | 10-60s | ✅ CORRECT (explicit) |
| `ruchy transpile file` | Show Rust code | <1s | ✅ CORRECT |
| `ruchy check file` | Parse + type check | <1s | ✅ CORRECT |
| `ruchy fmt file` | Format code | <1s | ✅ CORRECT (BUG-031 fixed) |
| `ruchy lint file` | Lint code | <1s | ✅ CORRECT |
| `ruchy test` | Run tests | Variable | ✅ CORRECT |

### What's Broken ❌

| Command | Current Behavior | Expected Behavior | Issue |
|---------|------------------|-------------------|-------|
| `ruchy` | Shows help | Opens REPL | ❌ Wrong default |
| `ruchy run script.ruchy` | Compiles (slow) | Interprets (fast) | ❌ Unnecessary compilation |

### Root Cause

**File**: `src/bin/handlers/mod.rs:handle_run_command()`
```rust
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    // BUG: Compiles instead of interpreting
    compile_to_binary(file, &options)?;
    execute_binary(&binary_path)?;
}
```

**Should be** (like `handle_file_execution()`):
```rust
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    // Use interpreter (like REPL does)
    let source = fs::read_to_string(file)?;
    let mut repl = Repl::new();
    repl.eval(&source)?;
}
```

---

## Deno CLI Design (Gold Standard)

### Deno's Execution Model

```bash
# These are ALL equivalent - interpret immediately:
deno script.ts                    # Direct execution (idiomatic)
deno run script.ts                # Explicit run (same as above)
deno run --allow-net script.ts    # With permissions

# Explicit compilation (slow, for production):
deno compile script.ts            # Creates standalone binary
```

**Key Insight**: Deno makes INTERPRETATION the default, compilation explicit.

### Deno's Command Categories

#### 1. Execution (Fast Path - Interpret)
- `run` - Run a program (interpret)
- `serve` - Run a server (interpret)
- `task` - Run a task (interpret)
- `repl` - Interactive REPL
- `eval` - Evaluate code

#### 2. Dependency Management
- `add` - Add dependencies
- `install` - Install dependencies
- `uninstall` - Remove dependencies
- `outdated` - Check outdated deps
- `remove` - Remove from config

#### 3. Tooling (Fast - No Compilation)
- `bench` - Run benchmarks
- `check` - Type-check only
- `clean` - Remove cache
- **`compile`** - **EXPLICIT** compilation to binary
- `coverage` - Generate coverage
- `doc` - Generate docs
- `fmt` - Format code
- `info` - Show info
- `lint` - Lint code
- `test` - Run tests
- `publish` - Publish package
- `upgrade` - Upgrade Deno

---

## Proposed Ruchy CLI (Deno-Inspired)

### Execution Commands (Interpret - <1s)

```bash
# No arguments - default to REPL (like Python, Ruby, Node)
ruchy                                    # Opens REPL

# Direct execution - interpret immediately (like Deno)
ruchy script.ruchy                       # Interpret (0.003s)
ruchy run script.ruchy                   # Same as above (consistency)

# One-liner execution
ruchy -e "println('hello')"              # Interpret one-liner
ruchy eval "println('hello')"            # Explicit eval command

# Stdin execution
echo "println('hello')" | ruchy          # Interpret stdin

# Interactive REPL
ruchy repl                               # Explicit REPL (same as no args)

# Server mode (future)
ruchy serve main.ruchy                   # Run HTTP server
```

### Tooling Commands (Fast - No Compilation)

```bash
# Development tools
ruchy check script.ruchy                 # Parse + type check only
ruchy fmt script.ruchy                   # Format code
ruchy lint script.ruchy                  # Lint code
ruchy test                               # Run tests (interpret)
ruchy bench                              # Run benchmarks

# Analysis tools
ruchy ast script.ruchy                   # Show AST
ruchy doc script.ruchy                   # Generate docs
ruchy coverage test.ruchy                # Coverage analysis
ruchy runtime --bigo script.ruchy        # Performance analysis
```

### Compilation Commands (Explicit - Slow)

```bash
# EXPLICIT compilation (only when needed)
ruchy compile script.ruchy -o binary     # Compile to standalone binary
ruchy build                              # Build project (cargo wrapper)
ruchy transpile script.ruchy             # Show Rust code
```

### Quality Tools

```bash
# Extreme quality (PMAT-powered)
ruchy quality-gate                       # Run quality gates
ruchy property-tests test.ruchy          # Property-based testing
ruchy mutations test.ruchy               # Mutation testing
ruchy fuzz script.ruchy                  # Fuzz testing
ruchy provability script.ruchy           # Formal verification
```

### Package Management (Future)

```bash
ruchy add jsr:@std/assert                # Add dependency
ruchy remove @std/assert                 # Remove dependency
ruchy install                            # Install dependencies
ruchy publish                            # Publish package
```

---

## Implementation Plan (EXTREME TDD)

### Phase 1: Fix Critical UX Issues (STOP THE LINE)

#### TASK 1.1: Make `ruchy` (no args) open REPL
**Current**: Shows help
**Expected**: Opens REPL (like Python, Ruby, Node, Deno)

**TDD Steps**:
1. **RED**: Write failing test
```rust
// tests/cli/test_default_command.rs
#[test]
fn test_ruchy_no_args_opens_repl() {
    use rexpect::spawn;

    let mut p = spawn("ruchy", Some(5000)).expect("Failed to spawn");

    // REPL prompt should appear
    p.exp_string("Welcome to Ruchy REPL").expect("No REPL prompt");
    p.exp_regex("ruchy>|>>>").expect("No prompt");

    // Should accept input
    p.send_line("2 + 2").expect("Failed to send");
    p.exp_string("4").expect("No output");

    p.send_line(":quit").expect("Failed to quit");
}
```

2. **GREEN**: Fix implementation
```rust
// src/bin/ruchy.rs:main()
fn main() -> Result<()> {
    let cli = Cli::parse();

    // NEW: If no arguments, open REPL
    if cli.eval.is_none() && cli.file.is_none() && cli.command.is_none() {
        return handle_repl_command(None, false);
    }

    // ... rest of logic
}
```

3. **REFACTOR**: Check complexity ≤10

**Property Test**:
```rust
#[test]
fn property_repl_never_panics_on_valid_input() {
    proptest! {
        #[test]
        fn test(code in "[a-z0-9 +\\-*/()]+") {
            let mut p = spawn("ruchy", Some(2000)).unwrap();
            p.exp_string("ruchy>").unwrap();
            let _ = p.send_line(&code); // Should not panic
        }
    }
}
```

---

#### TASK 1.2: Make `ruchy run` use interpreter (not compile)
**Current**: Compiles with rustc (slow)
**Expected**: Interprets like `ruchy script.ruchy` (fast)

**TDD Steps**:
1. **RED**: Write failing test
```rust
// tests/cli/test_run_command_speed.rs
#[test]
fn test_ruchy_run_interprets_not_compiles() {
    use std::time::Instant;
    use tempfile::NamedTempFile;

    // Create test file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fun main() {{ println(\"Hello\") }}").unwrap();

    // Time execution
    let start = Instant::now();
    Command::new("ruchy")
        .args(&["run", file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute");
    let duration = start.elapsed();

    // ASSERT: Should be <2s (interpret), NOT 10-60s (compile)
    assert!(
        duration.as_secs() < 2,
        "ruchy run took {}s - it's compiling instead of interpreting!",
        duration.as_secs()
    );
}

#[test]
fn test_ruchy_run_produces_no_binary() {
    // Should NOT create binary artifacts
    let file = NamedTempFile::new().unwrap();
    writeln!(file, "fun main() {{ println(42) }}").unwrap();

    Command::new("ruchy")
        .args(&["run", file.path().to_str().unwrap()])
        .output()
        .unwrap();

    // Check no binary created
    assert!(
        !Path::new("/tmp/ruchy_temp_run").exists(),
        "Binary artifact created - not interpreting!"
    );
}
```

2. **GREEN**: Fix implementation
```rust
// src/bin/handlers/mod.rs
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    // NEW: Use interpreter (same as handle_file_execution)
    let source = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let mut repl = Repl::new();
    if verbose {
        eprintln!("Running file: {}", file.display());
    }

    match repl.eval(&source) {
        Ok(result) => {
            if should_print_result(&result) {
                println!("{result}");
            }
            // Try calling main() if it exists
            if let Ok(main_result) = repl.eval("main()") {
                if should_print_result(&main_result) {
                    println!("{main_result}");
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

3. **REFACTOR**: Deduplicate with `handle_file_execution()` (DRY principle)

**Property Test**:
```rust
proptest! {
    #[test]
    fn property_run_and_direct_exec_equivalent(code in "[a-z0-9 ]+") {
        let file = NamedTempFile::new().unwrap();
        writeln!(file, "fun main() {{ println(\"{}\") }}", code).unwrap();

        // Both should produce identical output
        let direct = Command::new("ruchy")
            .arg(file.path())
            .output().unwrap().stdout;

        let run = Command::new("ruchy")
            .args(&["run", file.path().to_str().unwrap()])
            .output().unwrap().stdout;

        prop_assert_eq!(direct, run);
    }
}
```

**Mutation Test Target**:
- Verify tests catch if someone accidentally re-adds `compile_to_binary()` call
- Run: `cargo mutants --file src/bin/handlers/mod.rs --timeout 300`
- Target: ≥80% mutation coverage

---

### Phase 2: Comprehensive CLI Testing (E2E)

#### Test Matrix (All Must Pass)

| Test Case | Command | Expected Behavior | Test Type |
|-----------|---------|-------------------|-----------|
| **Default REPL** | `ruchy` | Opens REPL | rexpect |
| **Direct exec** | `ruchy file.ruchy` | Interprets | assert_cmd |
| **Run exec** | `ruchy run file.ruchy` | Interprets | assert_cmd |
| **One-liner** | `ruchy -e "code"` | Interprets | assert_cmd |
| **Stdin** | `echo "code" \| ruchy` | Interprets | assert_cmd |
| **Speed test** | `ruchy run file.ruchy` | <2s | Instant::now() |
| **No artifacts** | `ruchy run file.ruchy` | No binary | Path::exists |
| **Equivalence** | `ruchy file` vs `ruchy run file` | Same output | Property test |

#### Test File: `tests/cli/extreme_tdd_cli_suite.rs`

```rust
//! EXTREME TDD CLI Test Suite
//!
//! Testing Strategy:
//! 1. Unit tests: Individual command behavior
//! 2. Integration tests: Command interactions
//! 3. Property tests: Invariants across all inputs
//! 4. Mutation tests: Verify test quality
//! 5. Performance tests: Speed requirements
//! 6. rexpect tests: Interactive CLI behavior

use assert_cmd::Command;
use predicates::prelude::*;
use rexpect::spawn;
use std::time::Instant;
use tempfile::NamedTempFile;
use proptest::prelude::*;

// === UNIT TESTS ===

#[test]
fn test_no_args_opens_repl() {
    let mut p = spawn("ruchy", Some(5000)).expect("Failed to spawn");
    p.exp_string("Welcome to Ruchy REPL").expect("No REPL welcome");
    p.exp_regex(r"ruchy>|>>>").expect("No REPL prompt");
    p.send_line(":quit").expect("Failed to quit");
}

#[test]
fn test_direct_execution_interprets() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"fun main() {{ println("test") }}"#).unwrap();

    Command::cargo_bin("ruchy").unwrap()
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test"));
}

#[test]
fn test_run_command_interprets() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"fun main() {{ println("run test") }}"#).unwrap();

    Command::cargo_bin("ruchy").unwrap()
        .args(&["run", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("run test"));
}

#[test]
fn test_eval_flag_interprets() {
    Command::cargo_bin("ruchy").unwrap()
        .args(&["-e", r#"println("eval test")"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("eval test"));
}

#[test]
fn test_stdin_interprets() {
    Command::cargo_bin("ruchy").unwrap()
        .write_stdin(r#"println("stdin test")"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("stdin test"));
}

// === PERFORMANCE TESTS ===

#[test]
fn test_run_is_fast_not_compiled() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fun main() {{ println(42) }}").unwrap();

    let start = Instant::now();
    Command::cargo_bin("ruchy").unwrap()
        .args(&["run", file.path().to_str().unwrap()])
        .output()
        .unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 2,
        "ruchy run took {}s - compiling instead of interpreting!",
        duration.as_secs()
    );
}

#[test]
fn test_direct_exec_is_fast() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fun main() {{ println(42) }}").unwrap();

    let start = Instant::now();
    Command::cargo_bin("ruchy").unwrap()
        .arg(file.path())
        .output()
        .unwrap();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 100,
        "Direct exec took {}ms - should be <100ms",
        duration.as_millis()
    );
}

// === ARTIFACT TESTS ===

#[test]
fn test_run_creates_no_binary_artifacts() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fun main() {{ println(42) }}").unwrap();

    // Remove any existing artifacts
    let _ = std::fs::remove_file("/tmp/ruchy_temp_run");

    Command::cargo_bin("ruchy").unwrap()
        .args(&["run", file.path().to_str().unwrap()])
        .output()
        .unwrap();

    // Verify no binary created
    assert!(
        !Path::new("/tmp/ruchy_temp_run").exists(),
        "Binary artifact created at /tmp/ruchy_temp_run - not interpreting!"
    );
}

#[test]
fn test_compile_command_creates_binary() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "fun main() {{ println(42) }}").unwrap();

    let output = NamedTempFile::new().unwrap();

    Command::cargo_bin("ruchy").unwrap()
        .args(&[
            "compile",
            file.path().to_str().unwrap(),
            "-o",
            output.path().to_str().unwrap()
        ])
        .assert()
        .success();

    // Binary SHOULD exist for compile command
    assert!(
        output.path().exists(),
        "compile command didn't create binary!"
    );
}

// === PROPERTY TESTS ===

proptest! {
    /// Property: Direct execution and `run` produce identical output
    #[test]
    fn property_run_equals_direct_exec(code in "[a-zA-Z0-9 ]+") {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"fun main() {{ println("{}") }}"#, code).unwrap();

        let direct_output = Command::cargo_bin("ruchy").unwrap()
            .arg(file.path())
            .output().unwrap().stdout;

        let run_output = Command::cargo_bin("ruchy").unwrap()
            .args(&["run", file.path().to_str().unwrap()])
            .output().unwrap().stdout;

        prop_assert_eq!(direct_output, run_output);
    }

    /// Property: All execution modes should complete quickly (<2s)
    #[test]
    fn property_all_exec_modes_fast(code in "[a-z]+") {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"fun main() {{ println("{}") }}"#, code).unwrap();

        // Test direct exec
        let start = Instant::now();
        let _ = Command::cargo_bin("ruchy").unwrap()
            .arg(file.path())
            .output();
        prop_assert!(start.elapsed().as_secs() < 2);

        // Test run command
        let start = Instant::now();
        let _ = Command::cargo_bin("ruchy").unwrap()
            .args(&["run", file.path().to_str().unwrap()])
            .output();
        prop_assert!(start.elapsed().as_secs() < 2);

        // Test eval
        let start = Instant::now();
        let _ = Command::cargo_bin("ruchy").unwrap()
            .args(&["-e", &format!(r#"println("{}")"#, code)])
            .output();
        prop_assert!(start.elapsed().as_secs() < 2);
    }

    /// Property: REPL should never panic on syntactically valid input
    #[test]
    fn property_repl_never_panics(expr in "[a-z0-9 +\\-*/()]+") {
        // Skip if spawning fails
        if let Ok(mut p) = spawn("ruchy", Some(3000)) {
            if p.exp_regex(r"ruchy>|>>>").is_ok() {
                let _ = p.send_line(&expr);
                // Should not crash - just succeed or show error
                let _ = p.exp_regex(r"ruchy>|>>>|Error");
            }
        }
    }
}

// === REGRESSION TESTS ===

#[test]
fn regression_bug_031_fmt_no_corruption() {
    // BUG-031: fmt corrupted files
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"fun test() {{ println("hello") }}"#).unwrap();

    Command::cargo_bin("ruchy").unwrap()
        .args(&["fmt", file.path().to_str().unwrap()])
        .assert()
        .success();

    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(
        !content.contains("Call {"),
        "BUG-031 regression: fmt corrupted file!"
    );
}

#[test]
fn regression_dataframe_timeout() {
    // User reported: DataFrame code hangs for 90s
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"
        fun main() {{
            let df = DataFrame::new()
                .column("x", [1, 2, 3])
                .build()
            println(df)
        }}
    "#).unwrap();

    let start = Instant::now();
    let _ = Command::cargo_bin("ruchy").unwrap()
        .arg(file.path())
        .timeout(std::time::Duration::from_secs(10))
        .output();

    // Should complete quickly even if DataFrame fails
    assert!(
        start.elapsed().as_secs() < 10,
        "DataFrame execution took too long - still compiling?"
    );
}

// === EQUIVALENCE TESTS ===

#[test]
fn test_all_exec_modes_produce_same_output() {
    let code = r#"fun main() { println("test output") }"#;
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", code).unwrap();

    // Direct execution
    let direct = Command::cargo_bin("ruchy").unwrap()
        .arg(file.path())
        .output().unwrap().stdout;

    // Run command
    let run = Command::cargo_bin("ruchy").unwrap()
        .args(&["run", file.path().to_str().unwrap()])
        .output().unwrap().stdout;

    // Stdin
    let stdin = Command::cargo_bin("ruchy").unwrap()
        .write_stdin(code)
        .output().unwrap().stdout;

    // All should be identical
    assert_eq!(direct, run, "Direct exec != run command");
    assert_eq!(direct, stdin, "Direct exec != stdin");
}
```

---

### Phase 3: Pre-Commit Hook (Prevent Regressions)

#### New Hook: `pre-commit-cli-consistency`

**File**: `.git/hooks/pre-commit` (updated by `pmat hooks install`)

```bash
#!/bin/bash
# Pre-commit hook: CLI Consistency Checks

echo "🔍 Checking CLI consistency..."

# GATE 1: No transpilation in run command
if git diff --cached | grep -q "compile_to_binary.*handle_run_command"; then
    echo "❌ BLOCKED: handle_run_command must use interpreter, not compile_to_binary!"
    echo "   File: src/bin/handlers/mod.rs"
    echo "   Fix: Use Repl::eval() instead of compile_to_binary()"
    exit 1
fi

# GATE 2: Direct exec must use interpreter
if git diff --cached src/bin/handlers/mod.rs | grep -q "compile.*handle_file_execution"; then
    echo "❌ BLOCKED: handle_file_execution must use interpreter!"
    exit 1
fi

# GATE 3: Run CLI consistency tests
echo "Running CLI consistency tests..."
if ! cargo test --test extreme_tdd_cli_suite property_ -- --ignored 2>&1 | grep -q "test result: ok"; then
    echo "❌ BLOCKED: Property tests failed!"
    echo "   Run: cargo test --test extreme_tdd_cli_suite -- --nocapture"
    exit 1
fi

# GATE 4: Run speed tests
echo "Checking execution speed..."
if ! cargo test test_run_is_fast_not_compiled --test extreme_tdd_cli_suite 2>&1 | grep -q "test result: ok"; then
    echo "❌ BLOCKED: Speed test failed - run command is compiling!"
    exit 1
fi

# GATE 5: Verify no binary artifacts
echo "Checking for binary artifacts..."
if cargo test test_run_creates_no_binary_artifacts --test extreme_tdd_cli_suite 2>&1 | grep -q "FAILED"; then
    echo "❌ BLOCKED: Run command creates binary artifacts - not interpreting!"
    exit 1
fi

echo "✅ All CLI consistency checks passed!"
```

---

### Phase 4: Examples Validation

#### Create 10 Working Examples

**File**: `examples/cli_usage/`

1. `01_hello_world.ruchy` - Basic execution
2. `02_one_liner.sh` - Using -e flag
3. `03_stdin_pipe.sh` - Piping to ruchy
4. `04_interactive_repl.sh` - REPL session
5. `05_run_command.sh` - Using run explicitly
6. `06_compile_binary.sh` - Compile to standalone
7. `07_format_code.sh` - Formatting workflow
8. `08_test_suite.sh` - Running tests
9. `09_property_tests.sh` - Property-based testing
10. `10_full_workflow.sh` - Complete dev workflow

**All examples must**:
- Execute successfully with `bash examples/cli_usage/XX_*.sh`
- Complete in <2s (except compile)
- Be documented in `examples/cli_usage/README.md`

---

### Phase 5: Documentation Updates

#### Update Files

1. **README.md** - Add quick start with Deno-style examples
2. **docs/cli-reference.md** - Complete CLI documentation
3. **docs/migration-guide.md** - For existing users
4. **CHANGELOG.md** - Document breaking changes
5. **Book Chapter** - CLI usage patterns

---

## Testing Requirements (EXTREME TDD)

### Test Coverage Matrix

| Category | Target | Verification |
|----------|--------|--------------|
| **Unit Tests** | 100% of CLI handlers | `cargo test` |
| **Property Tests** | 10,000+ iterations | `cargo test property_` |
| **Mutation Tests** | ≥80% coverage | `cargo mutants --file src/bin/handlers/mod.rs` |
| **Integration Tests** | All commands | `cargo test --test extreme_tdd_cli_suite` |
| **rexpect Tests** | REPL interactions | `cargo test --test extreme_tdd_cli_suite` |
| **Performance Tests** | <2s exec, <100ms direct | `cargo test test_.*_fast` |
| **Examples** | 10/10 passing | `bash examples/cli_usage/test_all.sh` |

### Mutation Testing Targets

```bash
# Critical files for mutation testing
cargo mutants --file src/bin/ruchy.rs --timeout 300
cargo mutants --file src/bin/handlers/mod.rs --timeout 300

# Target: ≥80% mutation coverage
# Focus: Verify tests catch if someone re-adds compilation logic
```

### Property Test Invariants

1. **Equivalence**: `ruchy file` ≡ `ruchy run file` ≡ `cat file | ruchy`
2. **Speed**: All interpret modes complete <2s
3. **Determinism**: Same input → same output (no binary artifacts)
4. **Safety**: REPL never panics on syntactically valid input

---

## Breaking Changes & Migration

### For Users

**BREAKING CHANGE**: `ruchy run` now interprets (fast) instead of compiling.

**Migration**:
```bash
# OLD (v3.81.0 and earlier)
ruchy run script.ruchy     # Compiled (slow)

# NEW (v3.82.0+)
ruchy run script.ruchy     # Interprets (fast) ✅
ruchy compile script.ruchy # Compile explicitly
```

**Compatibility**:
- No breaking changes for `ruchy script.ruchy` (already correct)
- No breaking changes for `ruchy -e` (already correct)
- `ruchy compile` unchanged (already correct)

### For Developers

**Code Changes**:
- `handle_run_command()` refactored to use interpreter
- New pre-commit hook prevents regression
- 100+ new tests ensure correctness

---

## Success Criteria

### Must Pass Before Release

- [ ] All unit tests pass (100% of CLI handlers)
- [ ] All property tests pass (10,000+ iterations)
- [ ] Mutation coverage ≥80%
- [ ] All 10 examples execute successfully
- [ ] Pre-commit hook prevents regression
- [ ] Performance tests: `ruchy run` <2s, direct <100ms
- [ ] No binary artifacts from `ruchy run`
- [ ] Equivalence: `ruchy file` ≡ `ruchy run file`
- [ ] REPL opens with `ruchy` (no args)
- [ ] Documentation updated (README, CLI reference, migration guide)

### Release Checklist

- [ ] All tests passing
- [ ] Mutation tests ≥80%
- [ ] Examples validated
- [ ] CHANGELOG.md updated
- [ ] Version bumped to v3.82.0
- [ ] Git tag created
- [ ] Published to crates.io
- [ ] GitHub release created with migration notes
- [ ] Roadmap updated

---

## Timeline

| Phase | Tasks | Time Estimate | Status |
|-------|-------|---------------|--------|
| **Phase 1** | Fix critical UX (2 tasks) | 4h | ⏳ TODO |
| **Phase 2** | CLI test suite | 6h | ⏳ TODO |
| **Phase 3** | Pre-commit hook | 2h | ⏳ TODO |
| **Phase 4** | Examples | 3h | ⏳ TODO |
| **Phase 5** | Documentation | 3h | ⏳ TODO |
| **Phase 6** | Testing & validation | 4h | ⏳ TODO |
| **Phase 7** | Release | 2h | ⏳ TODO |
| **TOTAL** | | **24h** | |

---

## Risks & Mitigation

### Risk 1: Breaking User Scripts
**Mitigation**:
- Only affects `ruchy run` (rarely used due to slowness)
- Most users use `ruchy script.ruchy` (unchanged)
- Clear migration guide
- Version bump to v3.82.0 (signals breaking change)

### Risk 2: Test Suite Complexity
**Mitigation**:
- Incremental implementation (EXTREME TDD)
- Property tests catch edge cases
- Mutation tests verify test quality

### Risk 3: Performance Regression
**Mitigation**:
- Performance tests in CI
- Pre-commit hook blocks slow changes
- Benchmarks track speed over time

---

## References

- Deno CLI: https://docs.deno.com/
- BUG-031: docs/defects/CRITICAL-ruchy-run-forced-transpilation.md
- User Report: ../ruchy-book/docs/bugs/dataframe-transpilation-complete-failure.md
- Property Testing: https://proptest-rs.github.io/proptest/
- Mutation Testing: https://github.com/sourcefrog/cargo-mutants
- rexpect: https://docs.rs/rexpect/

---

**Status**: 📋 Ready for Implementation
**Next Step**: Update roadmap, then implement Phase 1
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR) with mutation & property tests
