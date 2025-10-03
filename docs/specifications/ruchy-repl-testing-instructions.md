# Ruchy REPL Testing Infrastructure

## Project Description
Build comprehensive test harness for Ruchy REPL using layered testing strategy: `assert_cmd` for non-interactive CLI tests, `rust-cli/rexpect` for PTY-based interaction tests, extended with zero-copy buffer management and sub-100μs pattern matching for high-frequency test scenarios.

## Architecture Decision: Extend vs Rewrite

**DECISION: Hybrid approach**

1. **Use `assert_cmd`** (0.12+) for non-interactive REPL tests
   - Stdin redirection (batch mode)
   - Exit code validation
   - Error message assertions
   - Fast feedback loop (<100ms per test)

2. **Use `rust-cli/rexpect`** (0.5+) for PTY-required features
   - Tab completion testing
   - Multi-line input with continuation prompts
   - History navigation (Up/Down arrows)
   - Ctrl-C/Ctrl-D signal handling

3. **Build `rexpect-extensions`** crate for gaps in rust-cli/rexpect:
   - Async/await support (tokio integration)
   - Zero-copy buffer optimization (current rexpect allocates on every read)
   - Multi-session select (simultaneous REPL testing)
   - Sub-millisecond timeout precision (current: 1ms granularity)

## Gap Analysis: rust-cli/rexpect vs Spec

### What rexpect HAS ✅
- PTY spawning via nix (Unix)
- Basic pattern matching (string/regex)
- timeout support (millisecond precision)
- send/expect/wait API
- bash-specific helpers (`spawn_bash`, `wait_for_prompt`)

### What's MISSING for Ruchy ❌
1. **No async support** - all blocking I/O
2. **No zero-copy buffering** - allocates Vec on every read
3. **No multi-session** - can't test parallel REPLs
4. **No pattern compilation** - regex recompiled per expect()
5. **No structured output** - before/after as tuples, not types
6. **Limited error types** - generic Error enum
7. **No Windows ConPTY** - Unix only

## Implementation Strategy

### Layer 1: Fast Non-Interactive Tests (assert_cmd)

```rust
// tests/cli_tests.rs - Run FIRST (fastest feedback)
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn batch_mode_evaluates_expression() {
    Command::cargo_bin("ruchy-repl")
        .unwrap()
        .write_stdin("2 + 2\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("Int = 4"));
}

#[test]
fn syntax_error_exits_nonzero() {
    Command::cargo_bin("ruchy-repl")
        .unwrap()
        .write_stdin("2 +\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("syntax error"));
}

#[test]
fn help_flag_shows_usage() {
    Command::cargo_bin("ruchy-repl")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE"));
}

// Property test: any valid program exits 0
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn valid_integers_exit_zero(n in -1000i64..1000) {
            Command::cargo_bin("ruchy-repl")
                .unwrap()
                .write_stdin(format!("{}\n", n))
                .assert()
                .success();
        }
    }
}
```

### Layer 2: Interactive Tests (rust-cli/rexpect)

```rust
// tests/interactive_tests.rs - For PTY features
use rexpect::{spawn, error::Error};
use std::time::Duration;

#[test]
fn tab_completion_suggests_keywords() -> Result<(), Error> {
    let mut repl = spawn("cargo run --bin ruchy-repl", Some(5000))?;
    
    repl.exp_string("ruchy> ")?;
    
    // Send partial keyword + tab
    repl.send("le\t")?;
    
    // Should complete to "let"
    repl.exp_string("let")?;
    
    // Cleanup
    repl.send_line("exit")?;
    repl.exp_eof()?;
    Ok(())
}

#[test]
fn multiline_function_definition() -> Result<(), Error> {
    let mut repl = spawn("cargo run --bin ruchy-repl", Some(5000))?;
    
    repl.exp_string("ruchy> ")?;
    repl.send_line("fn add(a: Int, b: Int) -> Int {")?;
    
    // Expect continuation prompt
    repl.exp_string("... ")?;
    repl.send_line("  a + b")?;
    
    repl.exp_string("... ")?;
    repl.send_line("}")?;
    
    // Should return to main prompt
    repl.exp_string("ruchy> ")?;
    
    // Test function
    repl.send_line("add(2, 3)")?;
    repl.exp_string("Int = 5")?;
    
    Ok(())
}

#[test]
fn ctrl_c_interrupts_without_exit() -> Result<(), Error> {
    let mut repl = spawn("cargo run --bin ruchy-repl", Some(5000))?;
    
    repl.exp_string("ruchy> ")?;
    
    // Start infinite loop
    repl.send_line("loop { println(\"running\") }")?;
    
    // Let it run briefly
    std::thread::sleep(Duration::from_millis(100));
    
    // Send Ctrl-C
    repl.send_control('c')?;
    
    // Should return to prompt (not exit)
    repl.exp_string("ruchy> ")?;
    
    // Verify still responsive
    repl.send_line("2 + 2")?;
    repl.exp_string("Int = 4")?;
    
    Ok(())
}

#[test]
fn history_navigation_up_arrow() -> Result<(), Error> {
    let mut repl = spawn("cargo run --bin ruchy-repl", Some(5000))?;
    
    repl.exp_string("ruchy> ")?;
    
    // Execute commands to build history
    repl.send_line("let x = 1")?;
    repl.exp_string("ruchy> ")?;
    
    repl.send_line("let y = 2")?;
    repl.exp_string("ruchy> ")?;
    
    // Navigate up (should get "let y = 2")
    repl.send("\x1b[A")?; // Up arrow ANSI code
    repl.exp_string("let y = 2")?;
    
    Ok(())
}
```

### Layer 3: Performance-Critical Extensions (rexpect-extensions)

Only build if profiling shows rexpect is bottleneck:

```rust
// rexpect-extensions/src/lib.rs
use tokio::io::{AsyncReadExt, Interest};
use rexpect::session::PtySession as SyncSession;

/// Async wrapper around rexpect::Session
pub struct AsyncSession {
    inner: SyncSession,
    async_fd: tokio::io::unix::AsyncFd<RawFd>,
}

impl AsyncSession {
    pub async fn spawn(cmd: &str, timeout_ms: u64) -> Result<Self, Error> {
        // Spawn in thread pool to avoid blocking executor
        let inner = tokio::task::spawn_blocking(move || {
            rexpect::spawn(cmd, Some(timeout_ms))
        })
        .await??;
        
        let raw_fd = inner.get_file_handle();
        let async_fd = tokio::io::unix::AsyncFd::new(raw_fd)?;
        
        Ok(Self { inner, async_fd })
    }
    
    pub async fn expect(&mut self, pattern: &str) -> Result<String, Error> {
        // Zero-copy optimization: poll until data available
        loop {
            self.async_fd.readable().await?;
            
            // Try non-blocking expect
            match self.inner.try_read() {
                Ok(Some(data)) => {
                    if data.contains(pattern) {
                        return Ok(data);
                    }
                }
                Ok(None) => continue, // No data yet
                Err(e) => return Err(e),
            }
        }
    }
}

// Parallel REPL testing
pub async fn select_expect(
    sessions: &mut [&mut AsyncSession],
    pattern: &str,
) -> Result<(usize, String), Error> {
    use tokio::select;
    
    // Build futures for each session
    let futures: Vec<_> = sessions.iter_mut()
        .enumerate()
        .map(|(i, s)| async move { 
            s.expect(pattern).await.map(|r| (i, r)) 
        })
        .collect();
    
    // Return first to complete
    select! {
        result = futures[0] => result,
        result = futures[1] => result,
        // ... up to N sessions
    }
}
```

## Extreme TDD Workflow for Ruchy REPL Tests

### Step 1: Write Test Suite Structure (RED)

```bash
# Create test files BEFORE implementing REPL
touch tests/cli_batch_tests.rs        # assert_cmd
touch tests/interactive_pty_tests.rs  # rexpect  
touch tests/completion_tests.rs       # rexpect + tab
touch tests/history_tests.rs          # rexpect + arrows
touch tests/signal_tests.rs           # rexpect + Ctrl-C/D
touch tests/multiline_tests.rs        # rexpect + continuation

# All should fail initially
cargo test
# Expected: "binary `ruchy-repl` not found"
```

### Step 2: Minimal REPL Implementation (GREEN)

```rust
// src/bin/ruchy-repl.rs - Minimal to pass first test
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    // Check if TTY (interactive) or redirected (batch)
    let is_tty = atty::is(atty::Stream::Stdin);
    
    loop {
        if is_tty {
            print!("ruchy> ");
            stdout.flush()?;
        }
        
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break; // EOF
        }
        
        // Minimal eval: just echo for now
        println!("{}", line.trim());
    }
    
    Ok(())
}
```

### Step 3: Incremental Feature Addition (RED → GREEN → REFACTOR)

**Iteration 1**: Expression evaluation
```bash
# RED: Write test
echo '#[test] fn evaluates_arithmetic() { ... }' >> tests/cli_batch_tests.rs
cargo test evaluates_arithmetic # FAIL

# GREEN: Implement parser + eval (minimal)
# ... add parser.rs, eval.rs

cargo test evaluates_arithmetic # PASS

# REFACTOR: Extract functions, add types
cargo test # Still PASS
```

**Iteration 2**: Tab completion
```bash
# RED: Write test
echo '#[test] fn tab_completes_keywords() { ... }' >> tests/completion_tests.rs
cargo test tab_completes_keywords # FAIL

# GREEN: Add rustyline with completion
# ... integrate rustyline::Editor, CompletionHelper

cargo test tab_completes_keywords # PASS

# REFACTOR: Extract completion logic
cargo test # Still PASS
```

## pmat Integration - Mutation Testing Strategy

### Configuration

```toml
# .pmat.toml
[mutate]
targets = [
    "src/repl.rs",          # Main REPL loop
    "src/parser.rs",        # Parser critical paths
    "src/eval.rs",          # Evaluator boundary conditions
    "src/completion.rs",    # Tab completion logic
]

[test]
# Fast tests only for mutation runs
filter = "not(integration) and not(slow)"
timeout = "10s"

[mutations]
operators = [
    "replace_operator",     # < → <=, == → !=
    "replace_constant",     # prompt strings, timeouts
    "negate_condition",     # if → if !
    "remove_statement",     # skip critical checks
]

# High-value targets for REPL
[[mutations.custom]]
pattern = 'if is_tty'
replacements = ['if true', 'if false']  # Should break prompt display

[[mutations.custom]]
pattern = 'readline.read_line'
replacements = ['Ok(0)', 'Err(...)']    # Should break EOF handling
```

### Mutation Test Targets

```rust
// src/repl.rs - Critical sections to mutate
pub fn run_repl() -> io::Result<()> {
    let mut rl = Editor::<RuchyHelper>::new()?;
    
    // MUTATE: Should test handles Ctrl-C correctly
    rl.set_signal_handler(SignalBehavior::Signal);
    //                                      ^^^^^^ mutate to ::Interrupt
    //                                             test must catch REPL exit
    
    loop {
        let prompt = if in_multiline { "... " } else { "ruchy> " };
        //                             ^^^^^^^          ^^^^^^^^^ 
        //                             mutate these to same value
        //                             test must fail on incorrect prompt
        
        match rl.readline(prompt) {
            Ok(line) => {
                if line.trim().is_empty() {
                //            ^^^^^^^^^ mutate to !is_empty()
                //                     test must catch skipped lines
                    continue;
                }
                
                // Parse and eval
                match parse_and_eval(&line) {
                    Ok(val) => println!("{}", val),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        // MUTATE: Should test error doesn't exit REPL
                        // return Err(e); // mutant adds this
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                // MUTATE: Should test Ctrl-C doesn't exit
                // break; // mutant adds this
                continue;
            }
            Err(ReadlineError::Eof) => {
                // MUTATE: Should test Ctrl-D exits cleanly
                break;
                // continue; // mutant replaces with this
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    Ok(())
}
```

### Run Mutation Tests

```bash
# Full mutation analysis (slow - run nightly)
cargo pmat --package ruchy-repl --bin ruchy-repl

# Target specific module (fast feedback)
cargo pmat --package ruchy-repl --target src/repl.rs

# Check threshold
cargo pmat --threshold 80
# Fail if <80% of mutants killed
```

## Property-Based Testing with Hypothesis-style Strategies

```rust
// tests/proptests.rs
use proptest::prelude::*;
use assert_cmd::Command;

proptest! {
    // Property: All valid integers roundtrip
    #[test]
    fn integer_literal_roundtrip(n in -1000000i64..1000000) {
        Command::cargo_bin("ruchy-repl")
            .unwrap()
            .write_stdin(format!("{}\n", n))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("Int = {}", n)));
    }
    
    // Property: Parser never panics on arbitrary input
    #[test]
    fn parser_never_panics(input in "\\PC{0,1000}") {
        let result = Command::cargo_bin("ruchy-repl")
            .unwrap()
            .write_stdin(&input)
            .timeout(std::time::Duration::from_secs(1))
            .assert();
        
        // Should exit (success or error), not timeout/panic
        assert!(result.get_output().status.code().is_some());
    }
    
    // Property: Interactive mode always prints prompt
    #[test]
    fn interactive_always_shows_prompt(
        inputs in prop::collection::vec(".*", 1..10)
    ) {
        let mut repl = rexpect::spawn("cargo run --bin ruchy-repl", Some(5000))?;
        
        for input in inputs {
            repl.exp_string("ruchy> ")?;
            repl.send_line(&input)?;
        }
        
        // Should still show prompt after arbitrary inputs
        repl.exp_string("ruchy> ")?;
    }
}
```

## Test Organization

```
ruchy/
├── Cargo.toml
├── .pmat.toml
│
├── src/
│   ├── bin/
│   │   └── ruchy-repl.rs    # Main REPL binary
│   ├── parser.rs
│   ├── eval.rs
│   └── completion.rs
│
├── tests/
│   ├── cli_batch_tests.rs         # assert_cmd (fast)
│   ├── interactive_pty_tests.rs   # rexpect (medium)
│   ├── completion_tests.rs        # rexpect + tab (slow)
│   ├── history_tests.rs           # rexpect + navigation
│   ├── signal_tests.rs            # rexpect + Ctrl-C/D
│   ├── multiline_tests.rs         # rexpect + continuation
│   ├── proptests.rs               # proptest integration
│   │
│   └── common/
│       └── mod.rs                 # Shared test utilities
│
├── benches/
│   ├── repl_startup.rs            # Startup latency
│   └── eval_throughput.rs         # Expressions/sec
│
└── rexpect-extensions/            # Optional performance layer
    ├── Cargo.toml
    └── src/
        ├── lib.rs                 # Async session
        └── zero_copy.rs           # Buffer optimization
```

## Dependencies

```toml
# Cargo.toml
[dependencies]
# REPL implementation
rustyline = "14.0"       # Readline with completion
atty = "0.2"             # TTY detection
thiserror = "1.0"        # Error handling

[dev-dependencies]
# Testing layers
assert_cmd = "2.0"       # Non-interactive CLI tests
predicates = "3.0"       # Assertions for assert_cmd
rexpect = "0.5"          # Interactive PTY tests
proptest = "1.4"         # Property-based testing

# Benchmarking
criterion = "0.5"        # Performance regression detection

# Optional performance layer
[dev-dependencies.rexpect-extensions]
path = "./rexpect-extensions"
optional = true

[features]
async-tests = ["rexpect-extensions"]
```

## Performance Targets

| Test Type | Count | Total Runtime | Feedback Loop |
|-----------|-------|---------------|---------------|
| CLI (assert_cmd) | 50+ | <2s | RED→GREEN: <10s |
| PTY (rexpect) | 20+ | <5s | RED→GREEN: <30s |
| Property | 10+ | <30s | Nightly CI |
| Mutation (pmat) | - | <5min | Pre-commit hook |

## Pre-Commit Checklist

```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "🧪 Running fast tests..."
cargo test --lib --bins -- --test-threads=4

echo "🔍 Running clippy..."
cargo clippy --all-targets -- -D warnings

echo "🎯 Running mutation tests on changed files..."
CHANGED=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)
if [ -n "$CHANGED" ]; then
    for file in $CHANGED; do
        cargo pmat --target "src/${file#src/}" --threshold 75 || {
            echo "❌ Mutation score too low for $file"
            exit 1
        }
    done
fi

echo "✅ All checks passed"
```

## Critical Test Cases (Must Pass Before v0.1)

### CLI Layer (assert_cmd)
- [ ] Batch mode evaluates expressions
- [ ] Syntax errors exit non-zero
- [ ] --help shows usage
- [ ] --version shows version
- [ ] Stdin EOF exits cleanly

### Interactive Layer (rexpect)
- [ ] Prompt appears on start
- [ ] Tab completes keywords
- [ ] Multi-line function definition
- [ ] History navigation (up/down arrows)
- [ ] Ctrl-C interrupts without exit
- [ ] Ctrl-D exits cleanly

### Property Layer (proptest)
- [ ] All valid integers roundtrip
- [ ] Parser never panics
- [ ] Interactive always shows prompt

### Mutation Layer (pmat)
- [ ] Mutation score >75% on repl.rs
- [ ] Mutation score >80% on parser.rs
- [ ] All timeout mutants killed

## Development Workflow

```bash
# 1. Start with failing CLI test (fastest)
cargo test cli_evaluates_basic_arithmetic
# RED: binary not found

# 2. Implement minimal REPL
vim src/bin/ruchy-repl.rs

# 3. Verify test passes
cargo test cli_evaluates_basic_arithmetic
# GREEN

# 4. Add interactive test
cargo test interactive_shows_prompt
# RED: no prompt displayed

# 5. Add TTY detection + prompt
vim src/bin/ruchy-repl.rs

# 6. Verify both tests pass
cargo test
# GREEN

# 7. Run mutation tests on new code
cargo pmat --target src/bin/ruchy-repl.rs
# Target: >75% score

# 8. Add missing test cases to kill mutants
# Iterate until score acceptable
```

## When to Use Each Testing Layer

**Use assert_cmd when:**
- Testing non-interactive CLI behavior
- Batch mode / stdin redirection
- Exit codes and error messages
- Fast feedback needed (<100ms)

**Use rexpect when:**
- Testing interactive features (prompts, completion)
- PTY-specific behavior (terminal size, echo)
- Signal handling (Ctrl-C, Ctrl-D)
- Multi-line input with continuation

**Build rexpect-extensions when:**
- Profiling shows rexpect is bottleneck (>10% of test time)
- Need parallel REPL testing (>2 concurrent sessions)
- Async integration tests required

**Use property tests for:**
- Input fuzzing (arbitrary strings, integers)
- Invariant checking (prompt always appears)
- Regression prevention (parser robustness)

**Run pmat mutation tests:**
- On every commit (pre-commit hook)
- When >75% score drops below threshold
- To identify missing test cases

## Success Metrics

- **Test Coverage**: >85% line coverage (cargo-tarpaulin)
- **Mutation Score**: >75% on core REPL logic (pmat)
- **Test Runtime**: <10s for full suite
- **False Positives**: <5% flaky test rate
- **Property Tests**: >1000 cases per property

## Anti-Patterns to Avoid

❌ Mock the PTY → Use real PTY with `cat`/`echo`  
❌ Sleep for synchronization → Use `expect()` with timeout  
❌ Exact string matching → Use predicates with contains/regex  
❌ Single assertion per test → Group related assertions  
❌ Ignore timing failures → Investigate, don't increase timeout  
❌ Skip mutation tests → They find real bugs

## Resources

- **assert_cmd docs**: https://docs.rs/assert_cmd
- **rexpect docs**: https://docs.rs/rexpect  
- **pmat guide**: https://github.com/facebook/pmat
- **proptest book**: https://proptest-rs.github.io/proptest/
- **Testing CLIs blog**: https://blog.rust-lang.org/inside-rust/2020/11/23/testing-cli-tools.html
