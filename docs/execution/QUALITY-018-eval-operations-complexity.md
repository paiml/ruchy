# QUALITY-018: Refactor eval_operations.rs Complexity Violations

**Priority**: Medium
**Estimated Effort**: 11.5 hours (per PMAT analysis)
**Created**: 2025-10-13
**Context**: Discovered during RUNTIME-003 when trying to commit identity comparison code

## Problem

eval_operations.rs has 3 pre-existing complexity violations that block commits:

1. **`modulo_values` (line 336)**: Cyclomatic: 6, **Cognitive: 21** (limit: 10)
2. **`eval_comparison_op` (line ~100)**: **Cyclomatic: 8** (limit: 10), Cognitive: 13
3. **`equal_objects` (line ~700)**: Cyclomatic: 5, **Cognitive: 16** (limit: 10)

These violations exist independently of RUNTIME-003 work. New code (`equal_values` with complexity 2) is within limits.

## Impact

- Blocks committing ANY changes to eval_operations.rs
- identity comparison implementation for classes (8 lines, complexity 2) is uncommitted but WORKING
- Tests prove implementation is correct (10/10 passing)

## Solution

Refactor the 3 functions to reduce complexity:

### 1. modulo_values (Cognitive: 21 → <10)
Extract helper for division-by-zero checks:
```rust
fn check_div_by_zero_int(b: i64) -> Result<(), InterpreterError> {
    if b == 0 { Err(InterpreterError::DivisionByZero) } else { Ok(()) }
}

fn check_div_by_zero_float(b: f64) -> Result<(), InterpreterError> {
    if b == 0.0 { Err(InterpreterError::DivisionByZero) } else { Ok(()) }
}
```

### 2. eval_comparison_op (Cyclomatic: 8 → <10)
Split into smaller functions by operation type (Less, Greater, LessEq, GreaterEq)

### 3. equal_objects (Cognitive: 16 → <10)
Extract nested logic into helper functions

## Acceptance Criteria

- [ ] All 3 functions have complexity ≤10 (both cyclomatic and cognitive)
- [ ] All existing tests still pass
- [ ] PMAT analysis shows 0 violations
- [ ] Can commit eval_operations.rs changes without quality gate blocking

## Related

- RUNTIME-003: Identity comparison code waiting to be committed
- Quality gates: Pre-commit hook blocks on ANY file-level violation
