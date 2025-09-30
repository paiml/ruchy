# RefCell Migration Path

## Overview

This document provides a step-by-step migration plan from `Value::Object(Rc<HashMap>)` to supporting both immutable `Object` and mutable `ObjectMut(Rc<RefCell<HashMap>>)`.

## Migration Principles

1. **Zero Breaking Changes**: All existing code continues to work
2. **Incremental**: Each step is independently testable
3. **Complexity Budget**: Every function ≤10 cognitive complexity
4. **Test Coverage**: Run full test suite after each step
5. **Toyota Way**: Stop on first failure, fix root cause

## Phase 1: Add ObjectMut Variant (30 minutes)

### Step 1.1: Add Variant to Value Enum

**File**: `src/runtime/interpreter.rs`

```rust
pub enum Value {
    // ... existing variants ...

    /// Object/HashMap value for key-value mappings (immutable)
    Object(Rc<HashMap<String, Value>>),

    /// Mutable object with interior mutability (NEW)
    ObjectMut(Rc<RefCell<HashMap<String, Value>>>),

    // ... rest of variants ...
}
```

**Complexity**: 0 (just adding variant)

### Step 1.2: Update Display Implementation

**File**: `src/runtime/interpreter.rs`

Find the `impl Display for Value` and add:

```rust
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing cases ...

            Value::Object(map) => {
                // Existing object formatting
                // ... keep as is ...
            }

            Value::ObjectMut(cell) => {
                // Same as Object, but borrow first
                let map = cell.borrow();
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            }
        }
    }
}
```

**Complexity**: 6 (iteration + formatting)

### Step 1.3: Update Clone Implementation

**File**: `src/runtime/interpreter.rs`

```rust
impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            // ... existing cases ...

            Value::Object(map) => Value::Object(Rc::clone(map)),

            Value::ObjectMut(cell) => {
                // Clone the Rc, shares same RefCell
                Value::ObjectMut(Rc::clone(cell))
            }
        }
    }
}
```

**Complexity**: 2 (simple match)

### Step 1.4: Update PartialEq Implementation

**File**: `src/runtime/interpreter.rs`

```rust
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // ... existing cases ...

            (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b) || **a == **b,

            (Value::ObjectMut(a), Value::ObjectMut(b)) => {
                // Either same RefCell or same contents
                Rc::ptr_eq(a, b) || *a.borrow() == *b.borrow()
            }

            // ObjectMut and Object are not equal (different types)
            (Value::Object(_), Value::ObjectMut(_)) => false,
            (Value::ObjectMut(_), Value::Object(_)) => false,

            // ... rest of cases ...
        }
    }
}
```

**Complexity**: 5 (match with multiple arms)

**Test After Phase 1:**
```bash
cargo test --lib 2>&1 | grep "test result:"
# Expected: 3364 passed (no regressions)
```

## Phase 2: Add Helper Functions (1 hour)

### Step 2.1: Create Utility Module

**File**: `src/runtime/object_helpers.rs` (NEW)

```rust
//! Helper functions for working with Object and ObjectMut values
//! All functions maintain ≤10 complexity budget

use super::{Value, InterpreterError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Check if value is a mutable object
/// Complexity: 1
pub fn is_mutable_object(value: &Value) -> bool {
    matches!(value, Value::ObjectMut(_))
}

/// Check if value is any kind of object (mutable or immutable)
/// Complexity: 2
pub fn is_object(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::ObjectMut(_))
}

/// Get field from object (handles both Object and ObjectMut)
/// Complexity: 5
pub fn get_object_field(value: &Value, field: &str) -> Option<Value> {
    match value {
        Value::Object(map) => map.get(field).cloned(),
        Value::ObjectMut(cell) => cell.borrow().get(field).cloned(),
        _ => None,
    }
}

/// Set field in mutable object (returns error for immutable)
/// Complexity: 7
pub fn set_object_field(
    value: &Value,
    field: &str,
    new_value: Value,
) -> Result<(), InterpreterError> {
    match value {
        Value::Object(_) => Err(InterpreterError::RuntimeError(
            format!("Cannot mutate immutable object field '{}'", field),
        )),
        Value::ObjectMut(cell) => {
            cell.borrow_mut().insert(field.to_string(), new_value);
            Ok(())
        }
        _ => Err(InterpreterError::RuntimeError(
            format!("Cannot access field '{}' on non-object", field),
        )),
    }
}

/// Create new mutable object from HashMap
/// Complexity: 2
pub fn new_mutable_object(map: HashMap<String, Value>) -> Value {
    Value::ObjectMut(Rc::new(RefCell::new(map)))
}

/// Create new immutable object from HashMap
/// Complexity: 2
pub fn new_immutable_object(map: HashMap<String, Value>) -> Value {
    Value::Object(Rc::new(map))
}

/// Convert immutable Object to mutable ObjectMut (copies data)
/// Complexity: 4
pub fn to_mutable(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            Value::ObjectMut(Rc::new(RefCell::new((**map).clone())))
        }
        Value::ObjectMut(_) => value.clone(),
        _ => value.clone(),
    }
}

/// Convert mutable ObjectMut to immutable Object (copies data)
/// Complexity: 4
pub fn to_immutable(value: &Value) -> Value {
    match value {
        Value::ObjectMut(cell) => {
            Value::Object(Rc::new(cell.borrow().clone()))
        }
        Value::Object(_) => value.clone(),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mutable_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        assert!(!is_mutable_object(&immutable));
        assert!(is_mutable_object(&mutable));
    }

    #[test]
    fn test_get_object_field() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        assert_eq!(get_object_field(&immutable, "key"), Some(Value::Integer(42)));
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(42)));
    }

    #[test]
    fn test_set_object_field() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        // Immutable should fail
        assert!(set_object_field(&immutable, "key", Value::Integer(99)).is_err());

        // Mutable should succeed
        assert!(set_object_field(&mutable, "key", Value::Integer(99)).is_ok());
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(99)));
    }

    #[test]
    fn test_to_mutable() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map);
        let mutable = to_mutable(&immutable);

        assert!(is_mutable_object(&mutable));
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(42)));
    }

    #[test]
    fn test_to_immutable() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let mutable = new_mutable_object(map);
        let immutable = to_immutable(&mutable);

        assert!(!is_mutable_object(&immutable));
        assert_eq!(get_object_field(&immutable, "key"), Some(Value::Integer(42)));
    }
}
```

**Update mod.rs:**
```rust
// In src/runtime/mod.rs
pub mod object_helpers;
```

**Test After Phase 2:**
```bash
cargo test object_helpers
# Expected: 5 tests pass
cargo test --lib
# Expected: 3369 passed (5 new tests)
```

## Phase 3: Update Constructors (1 hour)

### Step 3.1: Update Actor Instantiation

**File**: `src/runtime/interpreter.rs` (~line 3765)

```rust
fn instantiate_actor_with_args(
    &mut self,
    actor_name: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    // ... existing field setup code ...

    // Store message handlers
    if let Some(Value::Array(handlers)) = actor_info.get("__handlers") {
        instance.insert("__handlers".to_string(), Value::Array(Rc::clone(handlers)));
    }

    // CHANGE: Return ObjectMut instead of Object
    Ok(object_helpers::new_mutable_object(instance))  // NEW
}
```

**Complexity**: No change (just changing return type)

### Step 3.2: Update Class Instantiation

**File**: `src/runtime/interpreter.rs` (~line 3550)

```rust
fn instantiate_class_with_constructor(
    &mut self,
    class_name: &str,
    constructor_name: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    // ... existing constructor execution code ...

    // CHANGE: Return ObjectMut for classes (they may have mutable methods)
    Ok(object_helpers::new_mutable_object(instance))  // NEW
}
```

**Complexity**: No change

**Test After Phase 3:**
```bash
cargo test --test actor_extreme_tdd_tests --test class_runtime_extreme_tdd
# Expected: Some previously ignored tests may start failing (expected)
# They fail because field access doesn't handle ObjectMut yet
```

## Phase 4: Update Field Access (1 hour)

### Step 4.1: Update eval_field_access

**File**: `src/runtime/interpreter.rs` (~line 1800)

```rust
ExprKind::FieldAccess { object, field } => {
    let obj = self.eval_expr(object)?;

    // Use helper function instead of direct match
    object_helpers::get_object_field(&obj, field)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!(
                "Field '{}' not found on object",
                field
            ))
        })
}
```

**Complexity**: 3 (reduced from previous implementation)

**Test After Phase 4:**
```bash
cargo test --test class_runtime_extreme_tdd::test_class_field_access
# Expected: Now passes!
```

## Phase 5: Update Field Assignment (1 hour)

### Step 5.1: Update eval_assign

**File**: `src/runtime/interpreter.rs` (~line 2367)

```rust
ExprKind::FieldAccess { object, field } => {
    match &object.kind {
        ExprKind::Identifier(obj_name) => {
            let obj = self.lookup_variable(obj_name)?;

            // Check if mutable object
            if object_helpers::is_mutable_object(&obj) {
                // Mutable: Update in place
                object_helpers::set_object_field(&obj, field, val.clone())?;
                Ok(val)
            } else {
                // Immutable: Create new object (existing behavior)
                match obj {
                    Value::Object(ref map) => {
                        let mut new_map = (**map).clone();
                        new_map.insert(field.clone(), val.clone());
                        let new_obj = Value::Object(Rc::new(new_map));
                        self.set_variable(obj_name, new_obj);
                        Ok(val)
                    }
                    _ => Err(InterpreterError::RuntimeError(format!(
                        "Cannot access field '{}' on non-object",
                        field
                    ))),
                }
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "Complex field assignment not yet supported".to_string(),
        )),
    }
}
```

**Complexity**: 8 (if-else + pattern matching)

**Test After Phase 5:**
```bash
cargo test --test class_runtime_extreme_tdd::test_class_instantiation_with_new
# Expected: Now passes!
```

## Phase 6: Update Method Calls (2 hours)

### Step 6.1: Update eval_method_call

**File**: `src/runtime/interpreter.rs` (~line 2200)

This is the most complex change. Extract helper function:

```rust
fn eval_method_call_on_object(
    &mut self,
    obj: &Value,
    method_name: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    // Get method closure
    let method = object_helpers::get_object_field(obj, method_name)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!(
                "Method '{}' not found",
                method_name
            ))
        })?;

    if let Value::Closure { params, body, env: _ } = method {
        // Create method environment
        let mut method_env = HashMap::new();

        // Bind self (pass ObjectMut directly, not a borrow)
        method_env.insert("self".to_string(), obj.clone());

        // Bind parameters
        for (param, arg) in params.iter().zip(args) {
            method_env.insert(param.clone(), arg.clone());
        }

        // Push environment
        self.env_stack.push(method_env);

        // Execute method body
        let result = self.eval_expr(&body)?;

        // Pop environment
        self.env_stack.pop();

        Ok(result)
    } else {
        Err(InterpreterError::RuntimeError(format!(
            "'{}' is not a method",
            method_name
        )))
    }
}
```

**Complexity**: 9 (within budget)

**Test After Phase 6:**
```bash
cargo test --test class_runtime_extreme_tdd::test_class_method_with_parameters
# Expected: Now passes!
```

## Phase 7: Update Actor Message Handling (Already Done!)

The actor message handling in `process_actor_message_sync` already passes the actor instance correctly. It should work with ObjectMut without changes because:

1. It passes the full `Value` to handler execution
2. Handler can call `set_object_field` via field assignment
3. Changes persist because ObjectMut uses RefCell

**Test After Phase 7:**
```bash
cargo test --test actor_extreme_tdd_tests
# Expected: Previously ignored tests now pass!
```

## Phase 8: Remove #[ignore] Tags (15 minutes)

### Step 8.1: Update Actor Tests

**File**: `tests/actor_extreme_tdd_tests.rs`

Remove `#[ignore]` from:
- `test_send_message_to_actor`
- `test_ping_pong_actors`
- `test_actor_state_modification`
- `test_actor_type_safety`
- `test_actor_message_ordering`

### Step 8.2: Update Class Tests

**File**: `tests/class_runtime_extreme_tdd.rs`

Remove `#[ignore]` from:
- `test_bank_account_class`

Keep `#[ignore]` on (still require additional features):
- `test_accessing_parent_fields` (requires super() parsing)
- `test_class_with_undefined_field_type` (requires type checking)

**Test After Phase 8:**
```bash
cargo test
# Expected: 3372 passed (8 tests no longer ignored, 6 new from ObjectMut)
```

## Final Verification Checklist

- [ ] All 3364 original tests still pass
- [ ] 6 new tests from object_helpers pass
- [ ] 6 previously ignored tests now pass (2 still ignored for other reasons)
- [ ] Total: 3372 tests passing
- [ ] Zero clippy warnings
- [ ] All property tests pass (110,000 iterations)
- [ ] No RefCell panics in test suite
- [ ] All functions ≤10 complexity

## Rollback Plan

If issues arise:

1. **Phase 1-2**: Just remove ObjectMut variant and helpers (no other code uses it yet)
2. **Phase 3+**: Revert git commits for specific phase
3. **Nuclear option**: `git revert` entire branch, start over with lessons learned

## Timeline Summary

| Phase | Time | Cumulative |
|-------|------|------------|
| Phase 1: Add variant | 30 min | 30 min |
| Phase 2: Helpers | 1 hour | 1.5 hours |
| Phase 3: Constructors | 1 hour | 2.5 hours |
| Phase 4: Field access | 1 hour | 3.5 hours |
| Phase 5: Field assign | 1 hour | 4.5 hours |
| Phase 6: Method calls | 2 hours | 6.5 hours |
| Phase 7: Verify actors | 0 min | 6.5 hours |
| Phase 8: Remove ignores | 15 min | 6.75 hours |

**Total: ~7 hours**

## Success Criteria

- ✅ Zero regressions
- ✅ 8 previously failing tests now pass
- ✅ All functions ≤10 complexity
- ✅ Comprehensive test coverage
- ✅ Clean clippy
- ✅ Documentation complete

This migration path ensures a safe, incremental transition to RefCell-based mutable state while maintaining Toyota Way quality standards.