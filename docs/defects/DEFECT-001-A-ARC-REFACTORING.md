# DEFECT-001-A: Repl Thread-Safety Refactoring (Rc → Arc)

**Date**: 2025-10-12
**Status**: 🚨 **CRITICAL - BLOCKING SHARED STATE**
**Parent**: DEFECT-001 (UI Cell Execution Broken)
**Severity**: P0 - Architecture blocker

---

## Executive Summary

**Problem**: Ruchy Repl uses `Rc<T>` (not thread-safe) throughout, preventing shared state across async handlers in notebook server.

**Impact**: Variables don't persist across cell executions because each request creates a NEW Repl instance.

**Root Cause**: `Value` enum in `src/runtime/interpreter.rs` uses `Rc<str>`, `Rc<[Value]>`, `Rc<Expr>`, etc.

**Solution**: Refactor entire runtime to use `Arc<T>` (thread-safe) instead of `Rc<T>`.

**Scope**: This is a **MAJOR** refactoring affecting ~20 files in `src/runtime/`.

---

## Evidence of Problem

```bash
# Attempt to compile shared Repl state
$ cargo build --bin ruchy

error[E0277]: `Rc<str>` cannot be sent between threads safely
error[E0277]: `Rc<[interpreter::Value]>` cannot be sent between threads safely
error[E0277]: `Rc<ast::Expr>` cannot be sent between threads safely
error[E0277]: `Rc<HashMap<...>>` cannot be sent between threads safely
```

**Why This Blocks Shared State**:
- Axum handlers run on tokio thread pool
- `Arc<Mutex<Repl>>` requires `Repl: Send`
- `Repl` contains `Value` which contains `Rc<T>`
- `Rc<T>` is NOT `Send` (not thread-safe)
- Therefore: Cannot share Repl across threads

---

## Files Requiring Changes

### Core Runtime (PRIMARY)
1. **src/runtime/interpreter.rs** - `Value` enum definition (CRITICAL)
2. **src/runtime/eval_pattern.rs**
3. **src/runtime/value_utils.rs**
4. **src/runtime/eval_method_dispatch.rs**
5. **src/runtime/eval_control_flow_new.rs**
6. **src/runtime/eval_function.rs**
7. **src/runtime/eval_builtin.rs**
8. **src/runtime/eval_display.rs**
9. **src/runtime/eval_string_methods.rs**
10. **src/runtime/cache.rs**
11. **src/runtime/arena.rs**
12. **src/runtime/safe_arena.rs**
13. **src/runtime/lazy.rs**
14. **src/runtime/gc.rs**
15. **src/runtime/eval_loops.rs**

### REPL Module
16. **src/runtime/repl/formatting.rs**
17. **src/runtime/repl/mod.rs**
18. **src/runtime/repl/state.rs**

### Tests
19. **src/runtime/interpreter_tests.rs**
20. **All other test files using Rc**

---

## Refactoring Tickets (Extreme TDD)

### DEFECT-001-A-TICKET-1: Add Property Tests for Value Cloning

**Priority**: P0 - MUST DO FIRST
**Estimate**: 2 hours

**Objective**: Prove that Arc-based Values behave identically to Rc-based Values.

**TDD Approach**:
```rust
// tests/property_arc_refactor.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_value_clone_equivalence(value in arb_value()) {
        let cloned = value.clone();
        prop_assert_eq!(value, cloned);
    }

    #[test]
    fn test_value_thread_safety(value in arb_value()) {
        // Arc should allow Send
        std::thread::spawn(move || {
            let _ = value.clone();
        }).join().unwrap();
    }

    #[test]
    fn test_closure_capture_semantics(env in arb_env()) {
        // Closures should capture environment correctly
        let closure = create_closure_with_env(env.clone());
        prop_assert!(closure_has_correct_env(&closure, &env));
    }
}
```

**Acceptance Criteria**:
- ✅ 10,000+ property test iterations pass
- ✅ All Value operations preserve semantics
- ✅ Thread-safety verified via property tests

---

### DEFECT-001-A-TICKET-2: Refactor Value Enum (Core Change)

**Priority**: P0 - CRITICAL
**Estimate**: 3 hours
**Dependencies**: TICKET-1 (property tests must exist first)

**RED Phase**:
```rust
// Property tests will FAIL because Repl is not Send
#[test]
fn test_repl_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Repl>(); // Currently FAILS
}
```

**GREEN Phase**:
```rust
// src/runtime/interpreter.rs
use std::sync::Arc; // Changed from std::rc::Rc

pub enum Value {
    String(Arc<str>),           // Was: Rc<str>
    Array(Arc<[Value]>),        // Was: Rc<[Value]>
    Tuple(Arc<[Value]>),        // Was: Rc<[Value]>
    Closure {
        params: Vec<String>,
        body: Arc<Expr>,        // Was: Rc<Expr>
        env: Arc<HashMap<String, Value>>, // Was: Rc<HashMap<...>>
    },
    Object(Arc<HashMap<String, Value>>), // Was: Rc<HashMap<...>>
    ObjectMut(Arc<std::cell::RefCell<HashMap<String, Value>>>), // Was: Rc<RefCell<...>>
    // ... rest unchanged
}
```

**REFACTOR Phase**:
- Extract Arc creation to helper functions
- Ensure complexity ≤10 for all functions
- Add doctests showing Arc usage

**Acceptance Criteria**:
- ✅ `Repl: Send` test passes
- ✅ All property tests pass (10,000+ iterations)
- ✅ All existing unit tests pass
- ✅ Mutation coverage ≥75% on changed code

---

### DEFECT-001-A-TICKET-3: Update All Rc::new() → Arc::new()

**Priority**: P0 - CRITICAL
**Estimate**: 2 hours
**Dependencies**: TICKET-2

**Approach**: Mechanical find-replace with verification

```bash
# Find all Rc::new usages
grep -r "Rc::new" src/runtime/

# Replace systematically
sed -i 's/Rc::new/Arc::new/g' src/runtime/**/*.rs

# Verify compilation
cargo build --bin ruchy
```

**Acceptance Criteria**:
- ✅ Zero `Rc::new` calls in src/runtime/
- ✅ `cargo build --bin ruchy` succeeds
- ✅ All tests pass

---

### DEFECT-001-A-TICKET-4: Update All use std::rc::Rc

**Priority**: P0 - CRITICAL
**Estimate**: 1 hour
**Dependencies**: TICKET-3

```bash
# Replace import statements
sed -i 's/use std::rc::Rc;/use std::sync::Arc;/g' src/runtime/**/*.rs

# Verify
grep -r "std::rc::Rc" src/runtime/ # Should be empty
```

**Acceptance Criteria**:
- ✅ Zero `std::rc::Rc` imports in src/runtime/
- ✅ `cargo build --bin ruchy` succeeds

---

### DEFECT-001-A-TICKET-5: Run Mutation Tests on Refactored Code

**Priority**: P0 - CRITICAL
**Estimate**: 2 hours
**Dependencies**: TICKET-4

```bash
# Run mutation tests on interpreter.rs
cargo mutants --file src/runtime/interpreter.rs --timeout 300

# Target: ≥75% mutation coverage
```

**Acceptance Criteria**:
- ✅ Mutation coverage ≥75% (CAUGHT/(CAUGHT+MISSED) ≥ 0.75)
- ✅ No new uncaught mutations introduced
- ✅ Property tests catch mutations

---

### DEFECT-001-A-TICKET-6: Implement Shared REPL in Notebook Server

**Priority**: P0 - CRITICAL
**Estimate**: 2 hours
**Dependencies**: TICKET-5

**TDD Approach**:

**RED**:
```rust
// tests/e2e/notebook/00-smoke-test.spec.ts
test('should maintain shared state across cells', async ({ page }) => {
  // Cell 1: x = 10
  await executeCell(page, 'x = 10');
  expect(await getCellOutput(page, 0)).toContain('10');

  // Cell 2: x * 2
  await executeCell(page, 'x * 2');
  expect(await getCellOutput(page, 1)).toContain('20'); // Currently FAILS
});
```

**GREEN**:
```rust
// src/notebook/server.rs
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref SESSIONS: Mutex<HashMap<String, Repl>> = {
        Mutex::new(HashMap::new())
    };
}

async fn execute_handler(Json(request): Json<ExecuteRequest>) -> Json<ExecuteResponse> {
    tokio::task::spawn_blocking(move || {
        let mut sessions = SESSIONS.lock().unwrap();
        let repl = sessions.entry(request.session_id).or_insert_with(|| {
            Repl::new(std::env::current_dir().unwrap()).unwrap()
        });
        // Now Repl is Send, so this works!
        repl.eval(&request.source)
    }).await
}
```

**Acceptance Criteria**:
- ✅ E2E test passes (variables persist across cells)
- ✅ All 21/21 E2E tests pass (100%)
- ✅ `cargo build --bin ruchy` succeeds
- ✅ Server starts without errors

---

## Testing Strategy (EXTREME TDD)

### Property Tests (10,000+ iterations)
- `test_value_clone_equivalence`: Arc clones work like Rc clones
- `test_value_thread_safety`: Values can cross thread boundaries
- `test_closure_semantics`: Closures capture environment correctly
- `test_array_operations`: Arrays work identically with Arc
- `test_object_operations`: Objects work identically with Arc

### Mutation Tests (≥75% coverage)
- Run on src/runtime/interpreter.rs
- Run on src/runtime/eval_*.rs (all modified files)
- Verify property tests catch mutations

### Integration Tests
- All existing Ruchy tests must pass
- All REPL tests must pass
- All interpreter tests must pass

### E2E Tests
- 21/21 notebook E2E tests must pass
- Shared state tests must pass

---

## Success Criteria

### Technical
- ✅ `Repl: Send` (thread-safe)
- ✅ All Rc → Arc replacements complete
- ✅ `cargo build --bin ruchy` succeeds
- ✅ All tests pass (unit + property + mutation + E2E)
- ✅ Mutation coverage ≥75%
- ✅ Property tests: 10,000+ iterations passing

### Functional
- ✅ Variables persist across notebook cells
- ✅ E2E tests: 21/21 passing (100%)
- ✅ No performance regression (Arc overhead minimal)
- ✅ No semantic changes (Arc and Rc have same API)

### Quality
- ✅ All functions complexity ≤10
- ✅ Zero SATD (no TODO/FIXME/HACK)
- ✅ Comprehensive documentation
- ✅ Pre-commit hooks pass

---

## Risk Assessment

### Low Risk
- **Mechanical refactoring**: Rc and Arc have identical APIs
- **No semantic changes**: Just thread-safety added
- **Well-tested**: Property + mutation tests prove correctness

### Medium Risk
- **Performance**: Arc uses atomic operations (slightly slower than Rc)
  - Mitigation: Profile before/after, expect <5% overhead
- **Test coverage**: Need comprehensive property tests
  - Mitigation: 10,000+ iterations + mutation testing

### High Risk (Mitigated)
- **Breaking existing code**: Large refactoring touches many files
  - Mitigation: Extreme TDD with property tests BEFORE changes
  - Mitigation: Mutation tests verify test quality
  - Mitigation: All existing tests must pass

---

## Timeline

| Ticket | Priority | Estimate | Cumulative |
|--------|----------|----------|------------|
| TICKET-1 | P0 | 2h | 2h |
| TICKET-2 | P0 | 3h | 5h |
| TICKET-3 | P0 | 2h | 7h |
| TICKET-4 | P0 | 1h | 8h |
| TICKET-5 | P0 | 2h | 10h |
| TICKET-6 | P0 | 2h | 12h |

**Total**: 12 hours (1.5 days)

---

## Toyota Way Principles

### Jidoka (Stop The Line)
- Pre-commit hooks BLOCK if tests fail
- Mutation coverage <75% → BLOCKED
- Any E2E test failing → BLOCKED

### Genchi Genbutsu (Go And See)
- Property tests PROVE Arc behaves like Rc
- Mutation tests PROVE tests are effective
- E2E tests PROVE shared state works

### Poka-yoke (Error Proofing)
- Property tests prevent regressions
- Mutation tests prevent weak tests
- Type system enforces Send trait

### Kaizen (Continuous Improvement)
- This refactoring enables proper notebook sessions
- Future features can use thread-safe REPL
- Architecture improved for scalability

---

**Created**: 2025-10-12
**Status**: 🚨 READY TO START (Tickets Defined)
**Owner**: Development Team
**Next Action**: Begin TICKET-1 (Property Tests)

**This is the ROOT CAUSE FIX. No shortcuts. Toyota Way.**
