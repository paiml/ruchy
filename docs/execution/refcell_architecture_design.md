# RefCell Architecture Design Document

## Overview

This document describes the architecture for adding interior mutability to Ruchy's runtime using `RefCell`. This enables mutable state in actors and classes while maintaining Rust's memory safety guarantees.

## Problem Statement

Currently, `Value::Object(Rc<HashMap<String, Value>>)` is immutable. This causes problems:

1. **Actor state doesn't persist**: `actor.send(message)` doesn't update actor fields
2. **Class methods can't mutate**: `&mut self` methods create new objects without updating original
3. **Sequential mutations don't accumulate**: Multiple method calls lose intermediate state

## Design Goals

1. **Zero breaking changes**: Existing immutable code continues to work
2. **Opt-in mutability**: Only actors and classes use RefCell
3. **Panic safety**: Document borrow scenarios that could panic
4. **Performance**: Minimal overhead for immutable objects
5. **Complexity budget**: All functions ≤10 cognitive complexity

## Architecture Design

### 1. New Value Variant

Add a new variant to the `Value` enum for mutable objects:

```rust
pub enum Value {
    // ... existing variants ...

    /// Immutable object (existing)
    Object(Rc<HashMap<String, Value>>),

    /// Mutable object with interior mutability (NEW)
    ObjectMut(Rc<RefCell<HashMap<String, Value>>>),

    // ... rest of variants ...
}
```

**Rationale**:
- Separate variant avoids runtime overhead for immutable objects
- Clear distinction in type system between mutable/immutable
- Enables pattern matching to handle both cases

### 2. Usage Guidelines

**When to use ObjectMut:**
- Actor instances (need mutable state for message handling)
- Class instances with `&mut self` methods
- Any object that needs interior mutability

**When to use Object:**
- Immutable structs
- Function returns that don't need mutation
- Configuration objects
- All existing code (backward compatible)

### 3. Borrow Rules and Panic Prevention

**RefCell Runtime Rules:**
1. Multiple immutable borrows allowed simultaneously
2. Only ONE mutable borrow at a time
3. No immutable borrows when mutable borrow exists
4. Violating these rules causes **runtime panic**

**How Ruchy Avoids Panics:**

Since the interpreter is **single-threaded and synchronous**:
- No concurrent borrows possible
- Borrows are scoped to function execution
- Each operation borrows, executes, releases before next operation
- **Therefore: RefCell should NEVER panic in normal execution**

**Example Safe Pattern:**
```rust
// Safe: Sequential borrows
let value = obj_mut.borrow().get("field").cloned();
// Borrow released here
obj_mut.borrow_mut().insert("field", new_value);
// No overlap between borrows
```

**Example Unsafe Pattern (we MUST avoid):**
```rust
// UNSAFE: Nested borrows
let borrowed = obj_mut.borrow();
let key = borrowed.get("key");
obj_mut.borrow_mut().insert("other", value); // PANIC! Already borrowed
```

### 4. Migration Strategy

**Phase A: Add ObjectMut variant**
- Add variant to Value enum
- Update Display impl
- Update Clone impl
- Update PartialEq impl

**Phase B: Update constructors**
- `instantiate_actor_with_args`: Return ObjectMut
- `instantiate_class_with_constructor`: Return ObjectMut (when has mutable methods)
- Add helper: `Value::new_mutable_object(HashMap) -> Value`

**Phase C: Update field access**
- `eval_field_access`: Handle both Object and ObjectMut
- Pattern match on both variants
- Use `.borrow()` for reads

**Phase D: Update field assignment**
- `eval_assign`: Handle ObjectMut with `.borrow_mut()`
- Create new object for Object (existing behavior)
- Mutate in place for ObjectMut (new behavior)

**Phase E: Update method calls**
- `eval_method_call`: Detect `&mut self` methods
- Use `.borrow_mut()` for mutable methods
- Use `.borrow()` for immutable methods

### 5. Implementation Functions (≤10 complexity each)

**Helper Functions to Create:**

```rust
// Complexity: 3
fn is_mutable_object(value: &Value) -> bool {
    matches!(value, Value::ObjectMut(_))
}

// Complexity: 5
fn get_object_field(value: &Value, field: &str) -> Option<Value> {
    match value {
        Value::Object(map) => map.get(field).cloned(),
        Value::ObjectMut(cell) => cell.borrow().get(field).cloned(),
        _ => None,
    }
}

// Complexity: 7
fn set_object_field(value: &Value, field: &str, new_value: Value) -> Result<(), InterpreterError> {
    match value {
        Value::Object(_) => {
            // Immutable: Error or create new object
            Err(InterpreterError::RuntimeError("Cannot mutate immutable object".to_string()))
        }
        Value::ObjectMut(cell) => {
            cell.borrow_mut().insert(field.to_string(), new_value);
            Ok(())
        }
        _ => Err(InterpreterError::RuntimeError("Not an object".to_string()))
    }
}

// Complexity: 8
fn clone_as_mutable(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            Value::ObjectMut(Rc::new(RefCell::new((**map).clone())))
        }
        Value::ObjectMut(cell) => {
            Value::ObjectMut(Rc::new(RefCell::new(cell.borrow().clone())))
        }
        _ => value.clone()
    }
}
```

### 6. Test Strategy

**Unit Tests:**
- `test_object_mut_creation`
- `test_object_mut_field_access`
- `test_object_mut_field_mutation`
- `test_object_mut_multiple_reads` (verify no panic)
- `test_object_mut_sequential_writes` (verify no panic)

**Integration Tests:**
- `test_actor_state_persists_after_send`
- `test_class_mutable_method_updates_field`
- `test_nested_object_mutation`

**Property Tests:**
- Already created in Phase 2
- Will verify behavior with 10,000 iterations

### 7. Error Handling

**Borrow Panic Prevention:**
```rust
// Use try_borrow/try_borrow_mut for defensive code
fn safe_get_field(obj: &Rc<RefCell<HashMap<String, Value>>>, field: &str) -> Result<Option<Value>, InterpreterError> {
    obj.try_borrow()
        .map(|borrowed| borrowed.get(field).cloned())
        .map_err(|_| InterpreterError::RuntimeError("Borrow check failed".to_string()))
}
```

**For production use:**
- Use `.borrow()` and `.borrow_mut()` directly (will panic if violated)
- Panics indicate bugs in interpreter logic (good for debugging)
- Single-threaded execution guarantees no concurrent access

### 8. Performance Considerations

**RefCell Overhead:**
- Small runtime cost: one atomic counter per RefCell
- Negligible for interpreter (already slower than native code)
- Trade-off: Correctness > Performance for language features

**Memory:**
- ObjectMut: `Rc<RefCell<HashMap>>` = pointer + refcount + borrow counter
- Object: `Rc<HashMap>` = pointer + refcount
- Difference: One extra usize for borrow counter (~8 bytes on 64-bit)

### 9. Backward Compatibility

**Existing Code:**
- All current Object usage continues to work
- No changes needed to existing tests
- ObjectMut only used for new actor/class instances

**Migration Path:**
- Add ObjectMut alongside Object
- Update pattern matches to handle both
- Existing Object remains immutable
- New code uses ObjectMut when needed

### 10. Documentation Requirements

**User-Facing:**
- Document that actors have mutable state
- Document that `&mut self` methods modify in place
- Explain difference between `struct` (immutable) and `class` (mutable)

**Developer-Facing:**
- Doctest for ObjectMut usage
- Comment all `.borrow()` and `.borrow_mut()` calls
- Document borrow scope in complex functions

## Implementation Checklist

- [ ] Add `ObjectMut` variant to `Value` enum
- [ ] Update `Display` for `Value`
- [ ] Update `Clone` for `Value`
- [ ] Update `PartialEq` for `Value`
- [ ] Add helper functions (4 functions, all ≤10 complexity)
- [ ] Update `instantiate_actor_with_args` to return `ObjectMut`
- [ ] Update `instantiate_class_with_constructor` to return `ObjectMut`
- [ ] Update `eval_field_access` to handle `ObjectMut`
- [ ] Update `eval_assign` to handle `ObjectMut`
- [ ] Update `eval_method_call` to handle `&mut self` with `ObjectMut`
- [ ] Add unit tests (5 tests)
- [ ] Run integration tests (all property tests)
- [ ] Verify all 3364 library tests still pass
- [ ] Remove `#[ignore]` from 8 tests that now work
- [ ] Run `cargo test` - expect 3372/3372 passing

## Success Criteria

- ✅ All 8 previously ignored tests now pass
- ✅ Zero regressions (3364 tests remain passing)
- ✅ All new functions ≤10 complexity
- ✅ Property tests pass (110,000 iterations)
- ✅ Clippy clean
- ✅ No RefCell panics in test suite

## Risk Assessment

**Low Risk:**
- Architecture is well-understood (standard Rust pattern)
- Single-threaded execution prevents race conditions
- Comprehensive test suite catches regressions

**Medium Risk:**
- Complexity in pattern matching (must handle both Object/ObjectMut)
- Potential for accidental nested borrows (mitigated by code review)

**Mitigation:**
- Extensive testing (110,000 property test cases)
- Toyota Way: Stop at first failure, find root cause
- Code review of all `.borrow()` and `.borrow_mut()` calls

## Timeline Estimate

- Phase A (Add variant): 30 minutes
- Phase B (Update constructors): 1 hour
- Phase C (Field access): 1 hour
- Phase D (Field assignment): 1 hour
- Phase E (Method calls): 2 hours
- Testing & Validation: 1 hour

**Total: ~6.5 hours**

## Conclusion

This architecture provides a clean, safe solution for mutable state in Ruchy while maintaining backward compatibility and Rust's memory safety guarantees. The single-threaded interpreter execution model ensures RefCell will never panic in normal operation.