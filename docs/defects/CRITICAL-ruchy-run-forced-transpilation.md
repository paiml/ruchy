# CRITICAL DEFECT: `ruchy run` Uses Transpilation Instead of Interpreter

**Date**: 2025-10-14
**Severity**: 🔴 CRITICAL - Violates Core Design Principles
**Status**: 🔴 STOP THE LINE

---

## Problem Statement

`ruchy run` **compiles code with rustc** instead of **interpreting** it, despite having a working interpreter (used by REPL).

## Evidence

### Code Proof

**File**: `src/bin/handlers/mod.rs:handle_run_command()`
```rust
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    use ruchy::backend::{compile_to_binary, CompileOptions};

    // BUG: This transpiles + compiles instead of interpreting!
    compile_to_binary(file, &options)?;
    execute_binary(&binary_path)?;
}
```

**File**: `src/runtime/repl/mod.rs`
```rust
pub fn eval(&mut self, line: &str) -> Result<String> {
    // REPL uses evaluator (interpreter)
    self.evaluator.evaluate_line(expr, &mut self.state)?
}
```

**Conclusion**: Interpreter EXISTS but `ruchy run` doesn't use it!

---

## User Impact

### Expected Behavior (Like Python/Ruby/Node)
```bash
$ python script.py      # Interprets immediately (<1s)
$ ruby script.rb        # Interprets immediately (<1s)
$ node script.js        # Interprets immediately (<1s)
$ ruchy run script.ruchy  # Should interpret immediately!
```

### Actual Behavior
```bash
$ ruchy run script.ruchy
# Step 1: Parse (fast)
# Step 2: Transpile to Rust (slow)
# Step 3: rustc compile (10-60 seconds!)
# Step 4: Execute (fast)
```

**Time Comparison**:
- Expected: <1 second (interpret)
- Actual: 10-60 seconds (compile)
- **100x slower than necessary!**

---

## Real User Feedback

From `../ruchy-book/docs/bugs/dataframe-transpilation-complete-failure.md`:

> "ruchy run dataframe_test.ruchy hangs for >90 seconds"

**Root Cause**: Not a DataFrame bug - it's compiling with cargo+rustc instead of interpreting!

---

## Why This Violates Core Principles

### 1. Toyota Way - Jidoka (Quality at Source)
- Users expect `run` = interpret (standard across all scripting languages)
- Forcing compilation breaks this expectation
- Creates false bug reports ("code hangs") when it's just compiling

### 2. Consistency (Interpreter/Transpiler Parity)
- REPL interprets ✅
- `ruchy run` should interpret ✅
- But `ruchy run` actually compiles ❌

**Inconsistency**: Same code behaves differently in REPL vs `run`

### 3. Development Speed
- Interpret: <1s feedback
- Compile: 10-60s feedback
- **Development cycle 100x slower**

---

## The Fix

### BEFORE (Current - Broken)
```rust
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    compile_to_binary(file, &options)?;  // ❌ Compiles!
    execute_binary(&binary_path)?;
}
```

### AFTER (Correct)
```rust
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    // Read file
    let code = fs::read_to_string(file)?;

    // Parse
    let mut parser = Parser::new(&code);
    let ast = parser.parse()?;

    // Interpret (like REPL does)
    let mut evaluator = Evaluator::new();
    let mut state = State::new();
    evaluator.evaluate(&ast, &mut state)?;

    Ok(())
}
```

---

## Related Bugs

This design flaw causes or contributes to:

1. **DataFrame failures**: Compile-time issues instead of interpret-time
2. **"Hanging" reports**: Users think code hangs, but it's rustc compiling
3. **Slow feedback**: 100x slower than necessary development cycle
4. **False negatives**: Code works in REPL but "fails" in `run` (compilation errors)

---

## Action Items

**IMMEDIATE** (Stop The Line):
1. ✅ Document this critical defect
2. ⏸️ Halt features until interpreter parity restored
3. 🔧 Refactor `ruchy run` to use interpreter (like REPL)
4. ✅ File GitHub issue
5. 📝 Update documentation to clarify `run` vs `compile`

**Commands Should Be**:
- `ruchy run` → Interpret immediately (fast feedback)
- `ruchy compile` → Transpile + rustc (production binary)
- `ruchy repl` → Interactive interpret (current behavior ✅)

---

## Test Case

```rust
// File: tests/critical_run_uses_interpreter.rs

#[test]
fn test_ruchy_run_interprets_not_compiles() {
    use std::time::Instant;

    // Simple Ruchy program
    let code = r#"
        fun main() {
            println("Hello from interpreter!")
        }
    "#;

    fs::write("/tmp/test_run_speed.ruchy", code)?;

    // Time execution
    let start = Instant::now();
    Command::new("ruchy")
        .args(&["run", "/tmp/test_run_speed.ruchy"])
        .output()?;
    let duration = start.elapsed();

    // ASSERT: Should complete in <2 seconds (interpret)
    // NOT 10-60 seconds (compile)
    assert!(
        duration.as_secs() < 2,
        "ruchy run took {}s - it's compiling instead of interpreting!",
        duration.as_secs()
    );
}
```

---

## Severity Justification

**Why CRITICAL**:
1. Violates user expectations (run = interpret everywhere else)
2. Makes development 100x slower than necessary
3. Causes false bug reports ("code hangs")
4. Interpreter exists but isn't used (wasted functionality)
5. Creates interpreter/transpiler inconsistency

**Toyota Way**: This is a **Jidoka violation** - quality defect in core design.

---

## References

- User Report: `../ruchy-book/docs/bugs/dataframe-transpilation-complete-failure.md`
- Interpreter Code: `src/runtime/repl/mod.rs`
- Broken Handler: `src/bin/handlers/mod.rs:handle_run_command()`
- GitHub Issue: https://github.com/paiml/ruchy/issues/[TBD]

---

**Status**: 🚨 STOP THE LINE - Fix before continuing feature work
