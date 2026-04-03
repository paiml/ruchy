# Sub-spec: REPL Testing -- Architecture and Implementation Layers

**Parent:** [ruchy-repl-testing-instructions.md](../ruchy-repl-testing-instructions.md) Sections 1-4

---

## Project Description
Build comprehensive test harness for Ruchy REPL using layered testing strategy: `assert_cmd` for non-interactive CLI tests, `rust-cli/rexpect` for PTY-based interaction tests, extended with zero-copy buffer management and sub-100us pattern matching for high-frequency test scenarios.

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

### What rexpect HAS
- PTY spawning via nix (Unix)
- Basic pattern matching (string/regex)
- timeout support (millisecond precision)
- send/expect/wait API
- bash-specific helpers (`spawn_bash`, `wait_for_prompt`)

### What's MISSING for Ruchy
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
