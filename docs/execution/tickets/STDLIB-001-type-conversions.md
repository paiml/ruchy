# STDLIB-001: Type Conversion Functions

## Summary
Implement type conversion functions (str, int, float, bool) for both interpreter and transpiler.

## Status
- **Priority**: P0 (Critical for book examples)
- **Complexity**: Medium
- **Estimated Effort**: 2 hours

## Current State
- Status: 🟡 REPL only
- These functions exist in interpreter but need transpiler support

## Implementation Requirements

### Functions to Implement
```rust
str(x: Any) -> String      // Convert to string
int(x: Any) -> Int         // Convert to integer  
float(x: Any) -> Float     // Convert to float
bool(x: Any) -> Bool       // Convert to boolean
```

### Transpiler Mapping
- `str(x)` → `x.to_string()`
- `int(x)` → `x.parse::<i32>().unwrap()` or `x as i32`
- `float(x)` → `x.parse::<f64>().unwrap()` or `x as f64`
- `bool(x)` → Type-dependent conversion

## Test Cases
```rust
// String conversions
assert_eq!(str(42), "42")
assert_eq!(str(3.14), "3.14")
assert_eq!(str(true), "true")

// Integer conversions
assert_eq!(int("42"), 42)
assert_eq!(int(3.14), 3)
assert_eq!(int(true), 1)

// Float conversions
assert_eq!(float("3.14"), 3.14)
assert_eq!(float(42), 42.0)

// Boolean conversions
assert_eq!(bool(1), true)
assert_eq!(bool(0), false)
assert_eq!(bool(""), false)
assert_eq!(bool("hello"), true)
```

## Files to Modify
- `src/backend/transpiler/expressions.rs` - Add conversion function mappings
- `src/runtime/repl.rs` - Ensure interpreter implementation exists
- `tests/stdlib_type_conversions_test.rs` - New test file

## Success Criteria
- All type conversion functions work in both REPL and transpiled code
- Tests pass for all conversion combinations
- No performance regression