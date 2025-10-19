# PARSER-DEFECT-001: Block Followed by Tuple Misparsed as Function Call

## Status
**DISCOVERED**: 2025-10-19
**FIXED**: 2025-10-19
**SEVERITY**: CRITICAL (Blocked BOOTSTRAP-003)
**FIX STATUS**: GREEN PHASE COMPLETE ✅

## Problem Summary

When a block-like expression (loop, if, while, for, match, try, or explicit block) is immediately followed by a tuple literal, the parser incorrectly treats the tuple as function call arguments applied to the block.

### Minimal Reproduction

```ruchy
fn test() -> (i32, i32) {
    let mut x = 0;
    loop {
        if x >= 3 { break; }
        x = x + 1;
    }
    (x, x)  // ← This tuple is misparsed as calling the loop!
}
```

**Expected**: Tuple literal `(x, x)` as separate statement
**Actual**: Parsed as `(loop { ... })(x, x)` - function call!

### Error Message
```
Error: Evaluation error: Type error: Cannot call non-function value: integer
```

## Root Cause Analysis

### Parser Flow

1. Parser encounters `loop { ... }` keyword
2. Calls `parse_labeled_loop()` which calls `parse_expr_recursive()` for body
3. Body parser parses `{ break; }` as `Block([Break])`
4. Returns to Pratt parser's postfix handling
5. Postfix handler sees `(` token
6. Treats it as function call operator via `try_handle_single_postfix()`
7. Creates `Call { func: Block([Break]), args: [(x, x)] }`

### Code Location

**File**: `src/frontend/parser/mod.rs`
**Function**: `try_handle_single_postfix()` line 346-377
**Specific Issue**: Line 351

```rust
Some((Token::LeftParen, _)) => Ok(Some(functions::parse_call(state, left)?)),
```

This unconditionally treats ANY `LeftParen` after ANY expression as a function call.

### AST Structure (Incorrect)

```
Function {
    name: "test",
    body: Block([
        Let {
            name: "x",
            body: Loop {
                body: Call {           ← WRONG: Call inside Loop!
                    func: Block([      ← The loop body block
                        Break
                    ]),
                    args: [            ← The tuple elements
                        Identifier("x"),
                        Identifier("x")
                    ]
                }
            }
        }
    ])
}
```

### AST Structure (Correct - Expected)

```
Function {
    name: "test",
    body: Block([
        Let {
            name: "x",
            body: Loop {
                body: Block([
                    Break
                ])
            }
        },
        Tuple([              ← Separate Tuple statement
            Identifier("x"),
            Identifier("x")
        ])
    ])
}
```

## Impact

### Blocked Work
- **BOOTSTRAP-003**: Core Lexer Implementation in ruchyruchy project
- Pattern `fn tokenize() -> (Token, i32)` cannot be used
- Affects ALL functions returning tuples after control flow

### Test Results
- `test_red_loop_mut_tuple_basic()`: FAILS
- `test_red_loop_mut_tuple_simple()`: FAILS
- `test_baseline_tuple_without_loop()`: PASSES
- `test_baseline_tuple_with_literals()`: PASSES

## Fix Implementation

### Approach
Add check in `try_handle_single_postfix()` to prevent block-like expressions from consuming `(` as postfix call.

### Code Change

**File**: `src/frontend/parser/mod.rs`

```rust
Some((Token::LeftParen, _)) => {
    // PARSER-DEFECT-001 FIX: Block-like control flow expressions should NOT
    // consume `(...)` as function calls in statement position.
    if is_block_like_expression(&left) {
        Ok(None) // Don't treat `(` as postfix call
    } else {
        Ok(Some(functions::parse_call(state, left)?))
    }
}

/// Check if expression is a block-like control flow construct
fn is_block_like_expression(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Block(_)
            | ExprKind::Loop { .. }
            | ExprKind::While { .. }
            | ExprKind::For { .. }
            | ExprKind::If { .. }
            | ExprKind::Match { .. }
            | ExprKind::TryCatch { .. }
    )
}
```

### Rationale
- Block-like expressions ending with `}` should not auto-consume `(...)`
- Matches Rust's behavior where blocks don't participate in postfix calls
- Allows tuples to be parsed as separate statements
- Still permits explicit calls like `(some_func)(args)` where left is not block-like

## Test Coverage

### RED Tests (Created)
- `test_red_loop_mut_tuple_basic()` - Full reproduction case
- `test_red_loop_mut_tuple_simple()` - Minimal case
- `tests/runtime_loop_mut_tuple_return.rs` - Comprehensive suite

### GREEN Target
All RED tests should pass after fix applied.

### Property Tests (TODO)
```rust
proptest! {
    #[test]
    fn prop_block_tuple_not_call(x in any::<i32>()) {
        let code = format!("fn test() {{ loop {{ break; }} ({}, {}) }}", x, x);
        let ast = Parser::new(&code).parse().unwrap();
        // Assert no Call with Block as func
        assert!(!contains_block_call(&ast));
    }
}
```

## Complexity Analysis

### Modified Function
- `try_handle_single_postfix()`: Complexity remains ≤10
- Added helper: `is_block_like_expression()`: Complexity 2 (simple match)
- **Total**: Within Toyota Way limits (≤10)

## References

- Issue discovered during: BOOTSTRAP-003 (Core Lexer Implementation)
- Related: ../ruchyruchy/GITHUB_ISSUE_loop_mut_tuple_return.md
- Test file: tests/runtime_loop_mut_tuple_return.rs

## Verification Steps

1. Build with fix: `cargo build --release`
2. Run RED tests: `cargo test runtime_loop_mut_tuple_return`
3. Verify AST: `ruchy ast tests/loop_tuple.ruchy`
4. Check no Call nodes with Block func
5. Run property tests: `cargo test property_tests -- --ignored`
6. Mutation test: `cargo mutants --file src/frontend/parser/mod.rs`

## Resolution

**Target**: All tests pass, AST shows Tuple not Call ✅
**Quality Gate**: PMAT TDG B (75.1), Complexity ≤10 ✅, Zero SATD ✅
**Status**: FIX COMPLETE

### Test Results
- **Library Tests**: 3987 passed, 0 failed, 149 ignored ✅
- **GREEN Tests**: 5/5 passing ✅
  - test_green_loop_mut_tuple_basic ✅
  - test_green_loop_mut_tuple_simple ✅
  - test_baseline_tuple_return_no_loop ✅
  - test_baseline_loop_mut_no_tuple ✅
  - test_green_phase_summary ✅
- **No Regressions**: All existing tests pass ✅
- **BOOTSTRAP-003**: UNBLOCKED ✅

---

**Generated**: 2025-10-19
**EXTREME TDD**: RED ✅ → GREEN ✅ → REFACTOR ✅ → MUTATION (pending)
