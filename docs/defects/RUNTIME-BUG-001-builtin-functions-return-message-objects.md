# RUNTIME-BUG-001: Builtin Functions with Underscores Return Message Objects

**Status**: STOP THE LINE
**Priority**: P0 (Critical)
**Category**: Runtime Bug
**Created**: 2025-11-09
**Discovered by**: EXTREME TDD comprehensive testing

## Problem

Built-in functions with underscores in their names (`is_nil`, `type_of`, `parse_int`, `parse_float`) return Message objects instead of their actual values.

## Evidence

```bash
$ ruchy -e "println(is_nil(()))"
{__type: "Message", data: [nil], type: "is_nil"}  # WRONG - should print "true"

$ ruchy -e "println(len([1,2,3]))"
3  # CORRECT - functions without underscores work

$ ruchy -e "println(sqrt(16.0))"
4.0  # CORRECT - functions without underscores work
```

## Root Cause

**File**: `src/runtime/interpreter.rs:7488-7507`

Built-in functions are registered in `BuiltinRegistry` (src/runtime/builtins.rs) but are **NOT being registered in the interpreter's environment**. When these functions are called:

1. Parser creates: `Call { func: Identifier("is_nil"), args: [...] }`
2. Evaluator calls: `lookup_variable("is_nil")` → **FAILS** (not in env)
3. Falls back to Message constructor: `{ __type: "Message", type: "is_nil", data: [...] }`

This happens because:
- `builtins.rs` defines `BuiltinRegistry::new()` with all 70 functions
- But `BuiltinRegistry` is **never instantiated or used** in interpreter.rs
- Builtin functions are NOT added to interpreter environment on startup

## Affected Functions

All 70 builtin functions in `BuiltinRegistry` are potentially affected, but tested:
- `is_nil()` - ❌ Returns Message object
- `type_of()` - ❌ Returns Message object
- `parse_int()` - ❌ Returns Message object (also doesn't fail on invalid input)
- `parse_float()` - ❌ Returns Message object (also doesn't fail on invalid input)

Functions without underscores work via different code path (likely hardcoded).

## Expected Behavior

```bash
$ ruchy -e "println(is_nil(()))"
true  # Returns boolean

$ ruchy -e "println(type_of(42))"
Integer  # Returns type name string

$ ruchy -e "println(parse_int(\"42\"))"
42  # Returns parsed integer
```

## Fix Strategy

### Option 1: Register Builtins in Environment (Recommended)

Modify `Interpreter::new()` to:
1. Create `BuiltinRegistry`
2. For each builtin, add to environment as `Value::BuiltinFunction`
3. Update `eval_function_call` to handle `Value::BuiltinFunction`

### Option 2: Check Builtins Before Message Fallback

Modify `interpreter.rs:7488` to:
1. Before creating Message object, check `BuiltinRegistry::is_builtin(name)`
2. If true, call `BuiltinRegistry::call(name, args)`
3. Only fall back to Message if not a builtin

## Test Case

**File**: `tests/runtime_builtins_core_comprehensive.rs:123, 134`

```rust
#[test]
#[ignore = "Runtime limitation: is_nil() returns Message object, not boolean - needs runtime fix"]
fn test_type_is_nil_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(is_nil(()))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}
```

Once fixed, remove `#[ignore]` from these tests:
- `test_type_is_nil_true` (line 123)
- `test_type_is_nil_false` (line 134)

## Related Tickets

- RUNTIME-BUG-002: Collection mutations (push/pop/reverse/sort)
- RUNTIME-BUG-003: Parse error handling
- RUNTIME-BUG-004: to_string/parse_int roundtrip

## Five Whys

1. **Why do `is_nil()` calls return Message objects?**
   → Because `lookup_variable("is_nil")` fails

2. **Why does variable lookup fail?**
   → Because `is_nil` is not in the interpreter environment

3. **Why isn't `is_nil` in the environment?**
   → Because builtins are registered in `BuiltinRegistry` but never added to env

4. **Why aren't builtins added to env?**
   → Because `Interpreter::new()` doesn't instantiate `BuiltinRegistry`

5. **Why was this pattern chosen?**
   → Likely incomplete refactoring - `BuiltinRegistry` exists but isn't integrated

## Acceptance Criteria

- [ ] All 70 builtin functions work correctly (return actual values, not Message objects)
- [ ] Tests in `runtime_builtins_core_comprehensive.rs` pass without `#[ignore]`
- [ ] `ruchy -e "println(is_nil(()))"` prints `true`
- [ ] `ruchy -e "println(type_of(42))"` prints type name
- [ ] No regression in existing functionality
- [ ] All tests pass: `cargo test --workspace`
