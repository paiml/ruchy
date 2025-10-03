# BOOK-CH15-003: Reference Operator Parser Fix

**Date**: 2025-10-02
**Ticket**: BOOK-CH15-003
**Status**: ✅ COMPLETE
**Tests**: 5/5 passing + 3383 regression tests passing

## Summary

Fixed critical parser bug where the `&` (reference) operator was not recognized in prefix position, causing parse errors like "Expected RightBrace, found Let" when using expressions like `func(&data)`.

## Root Cause

The parser's `parse_prefix()` function in `src/frontend/parser/expressions.rs` was missing a case for `Token::Ampersand`.

**Existing unary operators handled**:
- `Token::Minus` → `UnaryOp::Negate` (e.g., `-42`)
- `Token::Bang` → `UnaryOp::Not` (e.g., `!flag`)
- `Token::Star` → `UnaryOp::Deref` (e.g., `*ptr`)
- `Token::Tilde` → `UnaryOp::BitwiseNot` (e.g., `~bits`)

**Missing operator**:
- `Token::Ampersand` → `UnaryOp::Reference` (e.g., `&data`)

## The Bug

Without `&` as a unary operator, expressions like `calculate_sum(&data)` would fail to parse. The parser would see `&` and not know how to handle it in prefix position, causing it to misparse the expression and fail with "Expected RightBrace, found Let".

**Failing code before fix**:
```ruchy
fun main() {
    let data = vec![1, 2, 3];
    let sum = calculate_sum(&data);  // Parser error!
    println("Sum: {}", sum);
}

fun calculate_sum(data: &Vec<i32>) -> i32 {
    42
}
```

**Error**: `Expected RightBrace, found Let`

## The Fix

Added `Token::Ampersand` case to `parse_prefix()` function:

```rust
// src/frontend/parser/expressions.rs lines 114-124
Token::Ampersand => {
    state.tokens.advance();
    let expr = super::parse_expr_with_precedence_recursive(state, 13)?; // High precedence for unary
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Reference,
            operand: Box::new(expr),
        },
        span,
    ))
}
```

This change:
1. Recognizes `&` as a valid prefix operator
2. Advances the token stream past the `&`
3. Parses the operand with precedence 13 (same as other unary operators)
4. Creates a `UnaryOp::Reference` AST node

## Impact

### Before Fix
- **Chapter 15 compatibility**: 25% (1/4 examples working)
- **Examples failing**: Calculator, Data Processor
- **Root cause**: Missing `&` unary operator support

### After Fix
- **Chapter 15 compatibility**: 100% (4/4 examples working)
- **All tests passing**: 3383 tests
- **New TDD tests**: 5/5 passing

## Test Coverage

Created comprehensive TDD test suite in `tests/parser_reference_types_tdd.rs`:

```rust
#[test]
fn test_reference_type_after_multilet_function() { /* ... */ }

#[test]
fn test_simple_reference_type_parameter() { /* ... */ }

#[test]
fn test_reference_type_after_simple_function() { /* ... */ }

#[test]
fn test_reference_vec_type() { /* ... */ }

#[test]
fn test_data_processor_example() { /* ... */ }
```

**All tests pass** ✅

## Verified Working Examples

### Example 1: Data Processor
```bash
$ ./target/release/ruchy compile test_data_proc.ruchy -o data_proc
✓ Successfully compiled

$ ./data_proc
Data Analysis Results:
Sum: 55
Average: 5.5
Maximum: 10
```

### Example 2: Math Library
```bash
$ ./target/release/ruchy compile test_math.ruchy -o math
✓ Successfully compiled

$ ./math
Mathematical Functions Demo
Factorial of 10: 3628800
```

## Complexity Analysis

**Function**: `parse_prefix()` in `expressions.rs`
**Change**: Added 10-line case for `Token::Ampersand`
**Cyclomatic Complexity**: +1 (acceptable, follows existing pattern)
**Cognitive Complexity**: Low (identical structure to other unary operators)

**Follows Toyota Way**:
- ✅ Pattern matches existing unary operator handling
- ✅ No code duplication
- ✅ Single responsibility (parse one operator type)
- ✅ TDD approach (tests written first, then fix)

## Related Work

- **BOOK-CH15-001**: Initial audit of Chapter 15 examples
- **BOOK-CH15-002**: Discovered multi-statement functions already work
- **BOOK-CH18-002**: Printf-style formatting for println (separate fix)

## Future Considerations

The `&` operator now works in prefix position for creating references. This enables:
- Borrowing in function calls: `func(&data)`
- Reference expressions: `let r = &value`
- Compatibility with Rust-style reference semantics

**No breaking changes** - this is purely additive functionality that was missing.

## Success Metrics

- ✅ All Chapter 15 examples compile and execute
- ✅ No regression in 3383 existing tests
- ✅ 5 new TDD tests added and passing
- ✅ Parser complexity remains under Toyota Way limits (<10 per function)
- ✅ Code follows existing patterns (no new abstractions needed)
