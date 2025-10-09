# PRIORITY-3: eval_control_flow_new.rs - Detailed Test Gap Analysis

## Current Status Summary

**Module**: `src/runtime/eval_control_flow_new.rs`
**Baseline Coverage**: 29.06% (145 lines covered, 354 uncovered)
**Target**: 80%+ coverage
**Current Tests**: 10 integration tests (all passing)
**Estimated Remaining Work**: 1-1.5 hours

## Coverage Gap Analysis

### Functions Currently Covered (10 tests)
✅ `eval_if_expr` - Basic true/false conditions
✅ `eval_while_loop` - Basic loop execution
✅ `eval_for_loop` - Range and array iteration
✅ `eval_match` - Literal pattern matching
✅ `eval_block_expr` - Block evaluation
✅ `eval_list_expr` - Array creation
✅ `eval_tuple_expr` - Tuple creation
✅ `handle_loop_control` - Break signal
✅ Basic control flow paths

### Functions NEEDING Coverage (18 functions, ~225 lines)

#### Category 1: Loop Control (High Priority - ~80 lines)
**Functions**:
- `eval_loop_body` - Loop body execution with break/continue handling
- `run_while_loop` - Complete while loop execution
- `eval_loop_condition` - Condition evaluation helper
- `handle_loop_control` - Continue signal handling (partial)

**Missing Test Cases** (Priority: HIGH):
1. While loop with break returning value: `while true { if cond { break 42 } }`
2. While loop with continue skipping iterations
3. Loop condition error propagation
4. Loop body error propagation
5. Infinite loop detection (if implemented)
6. Loop with empty body
7. Loop returning last value

**Estimated**: 7 tests, ~40 lines covered

---

#### Category 2: Pattern Matching (High Priority - ~70 lines)
**Functions**:
- `eval_match_arm` - Individual arm evaluation
- `eval_match_guard` - Guard condition evaluation
- `find_matching_arm` - Pattern matching logic
- `match_literal_pattern` - Literal pattern matching (partial)
- `match_list_pattern` - Array pattern matching (partial)
- `match_tuple_pattern` - Tuple pattern matching (partial)
- `match_identifier_pattern` - Variable binding patterns
- `match_wildcard_pattern` - Wildcard patterns (partial)

**Missing Test Cases** (Priority: HIGH):
1. Match with guard conditions: `match x { n if n > 5 => ... }`
2. Match with no matching arms (error case)
3. Match with nested patterns: `match [1, [2, 3]] { [1, [x, y]] => ... }`
4. Match with mixed patterns: `match (1, 2) { (1, _) => ... }`
5. Match with identifier binding: `match 5 { x => x + 1 }`
6. Match with type mismatches (error cases)
7. Match with empty patterns
8. Pattern matching with structs (if supported)

**Estimated**: 8 tests, ~45 lines covered

---

#### Category 3: Advanced Iteration (Medium Priority - ~50 lines)
**Functions**:
- `eval_array_iteration` - Array element iteration
- `eval_range_iteration` - Range iteration
- `execute_iteration_step` - Single iteration execution
- `extract_range_bounds` - Range bound extraction
- `create_range_iterator` - Range iterator creation
- `value_to_integer` - Value conversion helper

**Missing Test Cases** (Priority: MEDIUM):
1. For loop over empty array: `for x in [] { ... }`
2. For loop over nested arrays: `for row in [[1,2], [3,4]] { ... }`
3. Range with negative bounds: `for i in -5..5 { ... }`
4. Range with equal bounds: `for i in 5..5 { ... }`
5. Range with reversed bounds (error): `for i in 5..0 { ... }`
6. Inclusive range: `for i in 0..=10 { ... }`
7. Range iteration with break/continue
8. Non-integer range values (error cases)

**Estimated**: 8 tests, ~35 lines covered

---

#### Category 4: Expression Types (Medium Priority - ~40 lines)
**Functions**:
- `eval_range_expr` - Range expression creation
- `eval_array_init_expr` - Array initialization
- `eval_return_expr` - Return statement handling
- `eval_let_expr` - Let binding (needs more coverage)

**Missing Test Cases** (Priority: MEDIUM):
1. Let with error in value: `let x = undefined_var`
2. Let with error in body: `let x = 1; undefined_var`
3. Range expression edge cases (covered above in iteration)
4. Array init with size and default: `[0; 10]`
5. Array init with negative size (error)
6. Array init with non-integer size (error)
7. Return from nested function
8. Return with no value
9. Return outside function (error)

**Estimated**: 9 tests, ~30 lines covered

---

## Property Tests Required (10,000 cases each)

### Property 1: If Expression Determinism
**Invariant**: `eval_if(true, then, else) always equals eval(then)`
**Test**: 10,000 random expressions
**Code**:
```rust
proptest! {
    #[test]
    fn prop_if_true_evaluates_then(then_val: i64, else_val: i64) {
        let code = format!("if true {{ {} }} else {{ {} }}", then_val, else_val);
        let result = eval_code(&code).unwrap();
        assert_eq!(result, Value::Integer(then_val));
    }
}
```

### Property 2: While Loop Termination Count
**Invariant**: `while x < N` executes exactly N times starting from 0
**Test**: 10,000 random N values (0-100)

### Property 3: For Range Iteration Count
**Invariant**: `for i in 0..N` executes exactly N iterations
**Test**: 10,000 random N values

### Property 4: Match Exhaustiveness
**Invariant**: Match with wildcard always succeeds
**Test**: 10,000 random values

### Property 5: Pattern Matching Consistency
**Invariant**: If pattern matches, arm evaluates
**Test**: 10,000 random pattern/value pairs

### Property 6: Loop Control Never Panics
**Invariant**: Break/continue in valid contexts don't panic
**Test**: 10,000 random loop constructs

### Property 7: Block Returns Last Value
**Invariant**: Block with N statements returns Nth value
**Test**: 10,000 random blocks (1-10 statements)

### Property 8: List Construction Order
**Invariant**: `[a, b, c]` preserves element order
**Test**: 10,000 random arrays (1-20 elements)

**Estimated Total**: 80,000 property test executions

---

## Mutation Testing Targets

### Expected Mutations (from cargo-mutants):
1. **Condition Negations**: `if cond` → `if !cond`
2. **Loop Boundaries**: `while x < N` → `while x <= N`
3. **Pattern Match Arms**: Delete individual match arms
4. **Return Value Changes**: `Ok(value)` → `Ok(Value::Nil)`
5. **Error Propagation**: `result?` → `result.unwrap_or(Value::Nil)`
6. **Continue/Break**: Remove break/continue statements

### Mutation Kill Rate Target: ≥75%
**Strategy**:
- Each test should assert SPECIFIC values (not just `is_ok()`)
- Test boundary conditions explicitly (N-1, N, N+1)
- Test error cases to catch unwrap → default mutations
- Test each match arm individually

**Command**:
```bash
cargo mutants --file src/runtime/eval_control_flow_new.rs --timeout 300
```

---

## Test Implementation Plan

### Phase 1: Core Loop Control (30 min)
**Add 7 tests** for loop body, while loop, continue/break
**Target**: +40 lines coverage (29% → 37%)

### Phase 2: Pattern Matching Deep Dive (30 min)
**Add 8 tests** for match guards, nested patterns, bindings
**Target**: +45 lines coverage (37% → 46%)

### Phase 3: Iteration Edge Cases (20 min)
**Add 8 tests** for array/range iteration edge cases
**Target**: +35 lines coverage (46% → 53%)

### Phase 4: Expression Error Cases (20 min)
**Add 9 tests** for error propagation in let, return, array init
**Target**: +30 lines coverage (53% → 59%)

### Phase 5: Property Tests (30 min)
**Add 8 properties** with 10K cases each
**Target**: +60 lines coverage (59% → 71%)

### Phase 6: Mutation Testing (20 min)
**Run mutations, add targeted tests** for missed mutations
**Target**: +45 lines coverage (71% → 80%+)

**Total Estimated Time**: 2 hours 30 minutes
**Buffer**: 30 minutes for unexpected issues

---

## Quick Start for Next Session

### 1. Run Coverage Baseline
```bash
cargo llvm-cov --lib --html
# Open target/llvm-cov/html/index.html
# Navigate to runtime/eval_control_flow_new.rs
# Identify uncovered lines (red)
```

### 2. Add High-Priority Tests First
Focus on Category 1 (Loop Control) - highest impact:
```rust
#[test]
fn test_while_with_break_value() {
    let v = eval_code("while true { break 42 }").unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn test_continue_skips_iteration() {
    let code = "let mut s = 0; for i in 0..5 { if i == 2 { continue }; s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(8)); // 0+1+3+4, skips 2
}
```

### 3. Verify Coverage Improvement
```bash
cargo llvm-cov --lib
# Check new percentage
```

### 4. Add Property Tests
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_for_range_executes_n_times(n in 0u32..100) {
        let code = format!("let mut c = 0; for i in 0..{} {{ c = c + 1 }}; c", n);
        let v = eval_code(&code).unwrap();
        assert_eq!(v, Value::Integer(n as i64));
    }
}
```

### 5. Run Mutation Testing
```bash
cargo mutants --file src/runtime/eval_control_flow_new.rs --timeout 300 > mutations.txt
grep "MISSED" mutations.txt
# Add tests targeting missed mutations
```

---

## Success Criteria Checklist

- [ ] Line coverage: 29.06% → 80%+ (need +51%)
- [ ] Function coverage: 34.21% → 100% (need +66%)
- [ ] Unit tests: 10 → 40+ passing (need +30)
- [ ] Property tests: 0 → 8 properties (80K executions)
- [ ] Mutation coverage: ? → 75%+ kill rate
- [ ] P0 tests: 15/15 passing (zero regressions)
- [ ] PMAT gates: A- grade, ≤10 complexity, 0 SATD
- [ ] Commit message: References PRIORITY-3 ticket
- [ ] Documentation: Update ticket with final metrics

---

## Notes for Continuation

1. **Integration vs Unit**: Current tests use integration approach (eval_code helper). This is maintainable but may miss low-level edge cases. Consider adding a few direct function tests for complex callback signatures.

2. **Coverage Tool Limitation**: `cargo llvm-cov --lib` doesn't count integration tests. Use `cargo llvm-cov --all-targets` for accurate coverage.

3. **Test Naming**: Use descriptive names that map to coverage goals:
   - `test_while_break_with_value` (covers line 121)
   - `test_match_guard_false` (covers lines 192-195)

4. **Incremental Verification**: Run coverage after every 5-10 tests to ensure progress toward 80% goal.

5. **Time Management**: If approaching 2 hours without 80%, focus on high-value tests (loop control, pattern matching) over perfectionism.

---

## Appendix: Function Signature Reference

Quick reference for complex callback signatures:

```rust
// eval_let_expr signature
pub fn eval_let_expr<F1, F2>(
    name: &str,
    value: &Expr,
    _body: &Expr,
    eval_expr: F1,        // Evaluates expressions
    with_variable: F2,    // Scoped variable binding
) -> Result<Value, InterpreterError>

// eval_for_loop signature
pub fn eval_for_loop<F1, F2>(
    var: &str,
    iterable: &Expr,
    body: &Expr,
    eval_expr: &mut F1,   // Evaluates expressions
    with_variable: F2,    // Scoped variable binding
) -> Result<Value, InterpreterError>
```

Use integration tests (`eval_code` helper) to avoid signature complexity.
