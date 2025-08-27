# STDLIB-002: Advanced Math Functions

## Summary
Implement trigonometric and logarithmic functions for mathematical computations.

## Status
- **Priority**: P1 (Important for data science examples)
- **Complexity**: Low
- **Estimated Effort**: 1 hour

## Current State
- Status: ❌ Not implemented
- Basic math functions (abs, min, max, sqrt) are working

## Implementation Requirements

### Functions to Implement
```rust
sin(x: Float) -> Float
cos(x: Float) -> Float
tan(x: Float) -> Float
log(x: Float) -> Float      // Natural logarithm
log10(x: Float) -> Float    // Base-10 logarithm
random() -> Float           // 0.0-1.0
```

### Implementation Strategy
1. Add to interpreter in `repl.rs` as global functions
2. Map to Rust std library in transpiler:
   - `sin(x)` → `x.sin()`
   - `cos(x)` → `x.cos()`
   - `tan(x)` → `x.tan()`
   - `log(x)` → `x.ln()`
   - `log10(x)` → `x.log10()`
   - `random()` → `rand::random::<f64>()`

## Test Cases
```rust
// Trigonometry
assert!((sin(0.0) - 0.0).abs() < 0.0001)
assert!((cos(0.0) - 1.0).abs() < 0.0001)
assert!((tan(0.0) - 0.0).abs() < 0.0001)

// Logarithms
assert!((log(E) - 1.0).abs() < 0.0001)
assert!((log10(100.0) - 2.0).abs() < 0.0001)

// Random
let r = random()
assert!(r >= 0.0 && r <= 1.0)
```

## Files to Modify
- `src/runtime/repl.rs` - Add global math functions
- `src/backend/transpiler/expressions.rs` - Map function calls to Rust methods
- `tests/stdlib_math_test.rs` - New test file

## Success Criteria
- All math functions available in REPL and transpiled code
- Accuracy within floating-point tolerance
- random() properly seeded and bounded