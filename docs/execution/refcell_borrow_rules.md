# RefCell Borrow Rules and Panic Prevention Guide

## Executive Summary

RefCell provides **runtime-checked** borrowing. Unlike Rust's compile-time borrow checker, violations cause **panics at runtime**. This guide ensures we NEVER panic in the Ruchy interpreter.

## Core Borrow Rules

### Rule 1: Multiple Immutable Borrows Allowed
```rust
let obj = Rc::new(RefCell::new(hashmap));

// ✅ SAFE: Multiple simultaneous reads
let borrow1 = obj.borrow();
let borrow2 = obj.borrow();
let val1 = borrow1.get("key1");
let val2 = borrow2.get("key2");
// Both borrows alive simultaneously - OK!
```

### Rule 2: Only ONE Mutable Borrow at a Time
```rust
// ✅ SAFE: Single mutable borrow
{
    let mut borrow = obj.borrow_mut();
    borrow.insert("key", value);
} // Borrow dropped here

// ❌ PANIC: Two mutable borrows
let mut borrow1 = obj.borrow_mut();
let mut borrow2 = obj.borrow_mut(); // PANIC!
```

### Rule 3: No Immutable Borrow While Mutable Exists
```rust
// ❌ PANIC: Read while writing
let mut write_borrow = obj.borrow_mut();
let read_borrow = obj.borrow(); // PANIC!

// ✅ SAFE: Release write before read
{
    let mut write_borrow = obj.borrow_mut();
    write_borrow.insert("key", value);
} // Write borrow dropped
let read_borrow = obj.borrow(); // OK!
```

## Panic Scenarios in Ruchy

### Scenario 1: Nested Field Access During Mutation

**UNSAFE CODE:**
```rust
fn eval_assign_nested_bad(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    let borrowed = obj.borrow(); // Immutable borrow
    let key = borrowed.get("key"); // Still borrowed!

    obj.borrow_mut().insert("other", value); // PANIC! Already borrowed
}
```

**SAFE CODE:**
```rust
fn eval_assign_nested_safe(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    // Clone the value while borrowed, then release
    let key = obj.borrow().get("key").cloned();
    // Borrow released here

    obj.borrow_mut().insert("other", value); // OK!
}
```

**Why this works:** `.cloned()` creates an owned value, allowing the borrow to drop before the next borrow.

### Scenario 2: Method Call Accessing Self Fields

**UNSAFE CODE:**
```rust
fn eval_method_call_bad(instance: &Rc<RefCell<HashMap<String, Value>>>) {
    let borrowed = instance.borrow();
    let method = borrowed.get("__method").cloned().unwrap();

    // Still borrowed!
    if let Value::Closure { body, .. } = method {
        // Closure execution might try to borrow_mut instance
        self.eval_expr(&body); // PANIC if body contains self.field = value
    }
}
```

**SAFE CODE:**
```rust
fn eval_method_call_safe(instance: &Rc<RefCell<HashMap<String, Value>>>) {
    // Clone method THEN release borrow
    let method = instance.borrow().get("__method").cloned().unwrap();
    // Borrow released here

    if let Value::Closure { body, .. } = method {
        // Now safe to execute, can borrow_mut if needed
        self.eval_expr(&body); // OK!
    }
}
```

**Pattern:** Always `.cloned()` values from RefCell, never hold borrow across other operations.

### Scenario 3: Actor Message Handler Execution

**UNSAFE CODE:**
```rust
fn process_message_bad(actor: &Rc<RefCell<HashMap<String, Value>>>, msg: &Value) {
    let handlers = actor.borrow().get("__handlers").cloned().unwrap();
    let state_ref = &actor.borrow(); // Immutable borrow held

    // Execute handler which might mutate state
    execute_handler(handler, state_ref); // PANIC if handler does self.field = x
}
```

**SAFE CODE:**
```rust
fn process_message_safe(actor: &Rc<RefCell<HashMap<String, Value>>>, msg: &Value) {
    // Get handler, release borrow
    let handler = actor.borrow().get("__handlers").cloned().unwrap();
    // Borrow released here

    // Execute handler with actor reference (can borrow_mut internally)
    execute_handler(handler, actor); // OK!
}
```

**Pattern:** Pass the `Rc<RefCell<>>` itself, not a borrow of it.

### Scenario 4: Iterating While Mutating

**UNSAFE CODE:**
```rust
fn iterate_and_mutate_bad(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    let borrowed = obj.borrow();
    for (key, value) in borrowed.iter() {
        // Still borrowed!
        if condition(value) {
            obj.borrow_mut().remove(key); // PANIC! Already borrowed
        }
    }
}
```

**SAFE CODE:**
```rust
fn iterate_and_mutate_safe(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    // Collect keys first, release borrow
    let keys_to_remove: Vec<String> = obj.borrow()
        .iter()
        .filter(|(_, v)| condition(v))
        .map(|(k, _)| k.clone())
        .collect();
    // Borrow released here

    // Now mutate
    let mut borrowed_mut = obj.borrow_mut();
    for key in keys_to_remove {
        borrowed_mut.remove(&key); // OK!
    }
}
```

**Pattern:** Separate read and write phases. Collect data during read, mutate during write.

## Safe Patterns for Ruchy Interpreter

### Pattern 1: Clone-Release-Mutate

```rust
// ✅ ALWAYS SAFE
fn safe_pattern_1(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    let value = obj.borrow().get("key").cloned(); // Clone and release
    obj.borrow_mut().insert("other", new_value);  // OK!
}
```

### Pattern 2: Scoped Borrows

```rust
// ✅ ALWAYS SAFE
fn safe_pattern_2(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    {
        let borrowed = obj.borrow();
        process(borrowed.get("key"));
    } // Borrow explicitly dropped

    obj.borrow_mut().insert("key", value); // OK!
}
```

### Pattern 3: Early Drop

```rust
// ✅ ALWAYS SAFE
fn safe_pattern_3(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    let borrowed = obj.borrow();
    let value = borrowed.get("key").cloned();
    drop(borrowed); // Explicitly drop borrow

    obj.borrow_mut().insert("other", value); // OK!
}
```

### Pattern 4: Separate Functions

```rust
// ✅ ALWAYS SAFE
fn read_field(obj: &Rc<RefCell<HashMap<String, Value>>>) -> Option<Value> {
    obj.borrow().get("key").cloned()
} // Borrow released on function return

fn write_field(obj: &Rc<RefCell<HashMap<String, Value>>>, val: Value) {
    obj.borrow_mut().insert("key", val); // OK, separate function
}

fn process(obj: &Rc<RefCell<HashMap<String, Value>>>) {
    let val = read_field(obj);  // Borrow released
    write_field(obj, new_val);  // OK!
}
```

**Pattern:** Separate read and write into different functions. Borrows released at function boundaries.

## Defensive Programming with try_borrow

For production code where we want to catch errors gracefully:

```rust
fn defensive_field_access(obj: &Rc<RefCell<HashMap<String, Value>>>) -> Result<Value, InterpreterError> {
    obj.try_borrow()
        .map_err(|_| InterpreterError::RuntimeError("Borrow check failed".to_string()))?
        .get("field")
        .cloned()
        .ok_or_else(|| InterpreterError::RuntimeError("Field not found".to_string()))
}
```

**When to use:**
- User-facing operations that shouldn't crash
- External API boundaries
- Debugging mode

**When NOT to use:**
- Internal interpreter operations (panic = bug, should fix)
- Performance-critical paths (adds overhead)

## Why Ruchy Won't Panic

**Single-threaded interpreter guarantees:**

1. **No concurrent access**: Only one thread executing interpreter code
2. **Sequential operations**: Each operation completes before next starts
3. **Scoped borrows**: Borrows released before next operation
4. **No recursion holding borrows**: Functions release borrows before recursion

**Example Safe Execution:**
```
1. eval_expr(actor ! message)
   ├─ Get actor instance: actor.borrow().get("__handlers").cloned() [RELEASED]
   ├─ Execute handler body: eval_expr(handler.body)
   │  └─ Field assignment: actor.borrow_mut().insert("field", value) [OK]
   └─ Return result
```

Each step releases borrows before the next step.

## Testing Strategy

### Unit Tests for Borrow Safety

```rust
#[test]
fn test_no_panic_sequential_access() {
    let obj = Rc::new(RefCell::new(HashMap::new()));

    // Sequential reads - should not panic
    for _ in 0..1000 {
        let _ = obj.borrow().get("key");
    }
}

#[test]
fn test_no_panic_sequential_writes() {
    let obj = Rc::new(RefCell::new(HashMap::new()));

    // Sequential writes - should not panic
    for i in 0..1000 {
        obj.borrow_mut().insert("key", Value::Integer(i));
    }
}

#[test]
fn test_no_panic_read_write_alternating() {
    let obj = Rc::new(RefCell::new(HashMap::new()));

    // Alternating read/write - should not panic
    for i in 0..1000 {
        obj.borrow_mut().insert("key", Value::Integer(i));
        let _ = obj.borrow().get("key").cloned();
    }
}

#[test]
#[should_panic]
fn test_panic_nested_borrow() {
    let obj = Rc::new(RefCell::new(HashMap::new()));

    let _borrow = obj.borrow(); // Hold borrow
    let _mut_borrow = obj.borrow_mut(); // PANIC!
}
```

### Property Tests

Already created in Phase 2:
- `prop_refcell_never_panics_on_borrow`: Verifies 10,000 operations don't panic

## Code Review Checklist

When reviewing code with RefCell:

- [ ] All `.borrow()` calls are followed by `.cloned()` or scoped
- [ ] No borrows held across function calls that might borrow
- [ ] Iterators don't hold borrows while mutating
- [ ] Methods on ObjectMut pass `Rc<RefCell<>>`, not `&RefCell<>`
- [ ] No borrows held across `eval_expr` calls
- [ ] Separate read and write phases clearly
- [ ] Comment any complex borrow patterns

## Summary

**Golden Rule:** Never hold a borrow across another borrow operation.

**Safe patterns:**
1. Clone then release: `.borrow().get().cloned()`
2. Scoped borrows: `{ let b = obj.borrow(); ... }`
3. Separate functions: Read in one function, write in another

**Unsafe patterns to avoid:**
1. Nested borrows: Holding borrow while trying another
2. Iterating while mutating: `for` loop with borrow_mut inside
3. Passing borrowed refs: Pass `&Rc<RefCell<>>` not `&HashMap<>`

Following these rules, Ruchy's RefCell usage will be **panic-free**.