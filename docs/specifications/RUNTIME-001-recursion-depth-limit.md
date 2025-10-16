# [RUNTIME-001] Recursion Depth Limit Implementation Specification

**Status**: CRITICAL PRODUCTION BLOCKER
**Discovered**: 2025-10-16 via SQLITE-TEST-004 defensive testing
**Priority**: P0 - Must fix before v4.0.0 release
**Estimated Effort**: 4-6 hours

---

## Problem Statement

The Ruchy runtime currently CRASHES with stack overflow (SIGABRT) on infinite recursion instead of returning a graceful error. This is a CRITICAL production blocker discovered through defensive testing.

### Failing Test Cases

1. **Infinite recursion**: `fun infinite() { infinite() }` → SIGABRT
2. **Mutual recursion**: `fun foo() { bar() }; fun bar() { foo() }` → SIGABRT
3. **Deep recursion**: `countdown(10000)` → SIGABRT

### Expected Behavior

Runtime should:
1. **Track recursion depth** for each function call
2. **Check depth limit** before entering function body
3. **Return error** when limit exceeded: `InterpreterError::RecursionLimitExceeded`
4. **Never panic or crash** - always return Result

---

## Root Cause Analysis (Five Whys)

**Why does the runtime crash?**
→ Because Rust stack overflows when recursion is too deep

**Why doesn't the runtime catch this?**
→ Because there's no recursion depth tracking in `eval_function_call`

**Why isn't there depth tracking?**
→ Because the `frames` field in `Interpreter` struct is not actively used

**Why isn't `frames` field used?**
→ Because eval path doesn't update `frames` on function calls

**Why doesn't eval path use `frames`?**
→ **ROOT CAUSE**: The evaluation architecture uses closures (`eval_with_env`) which don't have access to the Interpreter state, so they can't check/update call depth

---

## Solution Design

### Option 1: Thread-Local Depth Counter (RECOMMENDED)

**Pros**:
- Simple to implement
- No signature changes needed
- Works with current closure-based eval
- Thread-safe

**Cons**:
- Uses thread-local storage (small overhead)

**Implementation**:
```rust
// In src/runtime/eval_function.rs

use std::cell::Cell;

thread_local! {
    static CALL_DEPTH: Cell<usize> = Cell::new(0);
    static MAX_DEPTH: Cell<usize> = Cell::new(1000); // Default limit
}

pub fn set_max_recursion_depth(depth: usize) {
    MAX_DEPTH.with(|max| max.set(depth));
}

fn check_recursion_depth() -> Result<(), InterpreterError> {
    CALL_DEPTH.with(|depth| {
        let current = depth.get();
        MAX_DEPTH.with(|max| {
            if current >= max.get() {
                Err(InterpreterError::RecursionLimitExceeded(current, max.get()))
            } else {
                depth.set(current + 1);
                Ok(())
            }
        })
    })
}

fn decrement_depth() {
    CALL_DEPTH.with(|depth| {
        depth.set(depth.get().saturating_sub(1));
    });
}

// Modify eval_closure_call_direct:
fn eval_closure_call_direct<F>(
    params: &[String],
    body: &Expr,
    env: &HashMap<String, Value>,
    args: &[Value],
    mut eval_with_env: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr, &HashMap<String, Value>) -> Result<Value, InterpreterError>,
{
    // CHECK DEPTH BEFORE ENTERING
    check_recursion_depth()?;

    // Ensure depth is decremented on ALL exit paths
    let result = (|| {
        if args.len() != params.len() {
            return Err(InterpreterError::RuntimeError(format!(
                "Function expects {} arguments, got {}",
                params.len(),
                args.len()
            )));
        }

        let mut call_env = env.clone();
        for (param, arg) in params.iter().zip(args.iter()) {
            call_env.insert(param.clone(), arg.clone());
        }

        match eval_with_env(body, &call_env) {
            Err(InterpreterError::Return(val)) => Ok(val),
            other => other,
        }
    })();

    // ALWAYS decrement, even on error
    decrement_depth();

    result
}
```

### Option 2: Pass Depth as Parameter

**Pros**:
- Explicit, no hidden state
- Easy to test

**Cons**:
- Requires changing ALL eval function signatures
- Major refactoring needed

### Option 3: Use Interpreter.frames Field

**Pros**:
- Uses existing infrastructure
- More comprehensive call tracking

**Cons**:
- Requires refactoring eval to pass Interpreter through closure chain
- Complex changes

---

## Recommended Implementation Plan

**Choose Option 1** (Thread-Local) for minimal disruption and fastest fix.

### Step 1: Add Error Type (5 min)

```rust
// In src/runtime/interpreter.rs (InterpreterError enum)

#[derive(Debug, Clone)]
pub enum InterpreterError {
    // ... existing variants ...

    /// Recursion depth limit exceeded
    RecursionLimitExceeded(usize, usize), // (current_depth, max_depth)
}
```

### Step 2: Implement Depth Tracking (30 min)

- Add thread-local depth counter to `eval_function.rs`
- Add `check_recursion_depth()` and `decrement_depth()` helpers
- Modify `eval_closure_call_direct()` to check/decrement depth

### Step 3: Integrate with REPL Config (15 min)

```rust
// In src/runtime/repl/mod.rs

impl Repl {
    pub fn new(work_dir: PathBuf) -> Result<Self> {
        let config = ReplConfig::default();

        // SET MAX DEPTH FROM CONFIG
        crate::runtime::eval_function::set_max_recursion_depth(config.maxdepth);

        // ... rest of initialization
    }
}
```

### Step 4: Error Message (10 min)

```rust
// In InterpreterError Display implementation

InterpreterError::RecursionLimitExceeded(depth, max) => {
    write!(f, "Recursion limit exceeded: depth {} exceeds maximum {}\n\
               Hint: Possible infinite recursion detected. Check for:\n\
               - Functions calling themselves without base case\n\
               - Mutual recursion between functions\n\
               - Very deep call chains",
           depth, max)
}
```

### Step 5: Testing (30 min)

Un-ignore tests in `sqlite_004_runtime_anomalies.rs`:
- test_sqlite_001_stack_overflow_infinite_recursion
- test_sqlite_002_stack_overflow_mutual_recursion
- test_sqlite_003_deep_call_stack

All should now PASS with clear error messages.

### Step 6: Add Configuration API (15 min)

```rust
// Public API for setting limit
pub fn set_recursion_limit(limit: usize) {
    crate::runtime::eval_function::set_max_recursion_depth(limit);
}

// Public API for getting current depth (debugging)
pub fn current_recursion_depth() -> usize {
    crate::runtime::eval_function::get_current_depth()
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_recursion_limit_enforced() {
    set_max_recursion_depth(10);

    let prog = "fun countdown(n) { if n > 0 { countdown(n - 1) } else { 0 } }; countdown(20)";

    let result = execute_program(prog);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Recursion limit exceeded"));
}

#[test]
fn test_recursion_within_limit() {
    set_max_recursion_depth(100);

    let prog = "fun countdown(n) { if n > 0 { countdown(n - 1) } else { 0 } }; countdown(50)";

    let result = execute_program(prog);
    assert!(result.is_ok());
}
```

### Integration Tests

All tests in `sqlite_004_runtime_anomalies.rs` should pass.

---

## Performance Impact

**Overhead per function call**: ~50ns (thread-local access + integer comparison)
**Impact on benchmarks**: <1% (negligible)
**Memory**: 2 × usize per thread (~16 bytes)

---

## Rollout Plan

1. **Implement fix** (2-3 hours)
2. **Run full test suite** (5 minutes)
3. **Verify sqlite_004 tests pass** (1 minute)
4. **Update CHANGELOG** (5 minutes)
5. **Create PR with [RUNTIME-001] tag** (5 minutes)
6. **Merge immediately** (CRITICAL fix, no review delay needed for P0)

---

## Success Criteria

✅ All 3 stack overflow tests in sqlite_004 PASS
✅ Runtime never crashes on recursion
✅ Clear error message with helpful hints
✅ Configurable limit via ReplConfig
✅ <1% performance impact
✅ Zero new compiler warnings

---

## Related Issues

- **Found by**: SQLITE-TEST-004 (Runtime Anomaly Validation)
- **Blocks**: v4.0.0 release
- **Toyota Way**: Jidoka - Stop the line, fix the root cause

---

## References

- **SQLite Approach**: "Test what can go wrong, not just what should work"
- **Rust std::panic**: Cannot reliably catch stack overflow
- **Industry Standard**: Python (RecursionError after 1000 calls), Ruby (SystemStackError after ~10K)
- **Recommended Default**: 1000 calls (matches Python, sufficient for most use cases)
