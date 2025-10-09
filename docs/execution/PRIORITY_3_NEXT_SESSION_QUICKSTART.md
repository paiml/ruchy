# PRIORITY-3: Quick Start Guide for Next Session

## Session Objective
**Increase `src/runtime/eval_control_flow_new.rs` coverage from 29% to 80%+**

## Current Status (Commit a443e4c1)
- ✅ 10 integration tests created (all passing)
- ✅ Ticket documented (PRIORITY_3_EVAL_CONTROL_FLOW.md)
- ✅ Detailed analysis completed (PRIORITY_3_EVAL_CONTROL_FLOW_ANALYSIS.md)
- ⏳ Coverage: 29.06% (need +51% to reach 80%)

## Immediate Actions (First 15 Minutes)

### 1. Verify Baseline
```bash
cd /home/noah/src/ruchy
cargo llvm-cov --all-targets 2>&1 | grep "eval_control_flow_new.rs"
```
Expected: ~29% coverage

### 2. Open Analysis Document
```bash
cat docs/execution/PRIORITY_3_EVAL_CONTROL_FLOW_ANALYSIS.md
```
Review: Categories 1-4 for test priorities

### 3. Run Existing Tests
```bash
cargo test --test priority_3_eval_control_flow
```
Expected: 10/10 passing

## High-Priority Tests to Add (Next 30 Minutes)

Copy-paste ready tests to add to `tests/priority_3_eval_control_flow.rs`:

```rust
// === CATEGORY 1: Loop Control (7 tests) ===

#[test]
fn test_while_break_with_value() {
    let v = eval_code("while true { break 42 }").unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn test_while_continue() {
    let code = "let mut x = 0; let mut c = 0; while x < 5 { x = x + 1; if x == 3 { continue }; c = c + 1 }; c";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(4)); // Skips increment when x==3
}

#[test]
fn test_loop_empty_body() {
    let v = eval_code("let mut x = 0; while x < 3 { x = x + 1 }; x").unwrap();
    assert_eq!(v, Value::Integer(3));
}

#[test]
fn test_for_with_break() {
    let code = "let mut s = 0; for i in 0..10 { if i > 3 { break }; s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6)); // 0+1+2+3
}

#[test]
fn test_for_with_continue() {
    let code = "let mut s = 0; for i in 0..5 { if i == 2 { continue }; s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(8)); // 0+1+3+4
}

#[test]
fn test_nested_loops_break() {
    let code = r#"
        let mut found = false
        for i in 0..5 {
            for j in 0..5 {
                if i == 2 && j == 3 {
                    found = true
                    break
                }
            }
        }
        found
    "#;
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Bool(true));
}

#[test]
fn test_while_false_never_executes() {
    let code = "let mut x = 0; while false { x = x + 1 }; x";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(0));
}

// === CATEGORY 2: Pattern Matching (8 tests) ===

#[test]
fn test_match_with_guard_true() {
    let code = "match 10 { x if x > 5 => 100, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(100));
}

#[test]
fn test_match_with_guard_false() {
    let code = "match 3 { x if x > 5 => 100, _ => 99 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(99));
}

#[test]
fn test_match_identifier_binding() {
    let code = "match 42 { x => x + 1 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(43));
}

#[test]
fn test_match_tuple_destructure() {
    let code = "match (1, 2, 3) { (a, b, c) => a + b + c, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_match_array_destructure() {
    let code = "match [1, 2, 3] { [a, b, c] => a + b + c, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_match_nested_pattern() {
    let code = "match (1, (2, 3)) { (a, (b, c)) => a + b + c, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_match_wildcard_in_pattern() {
    let code = "match (1, 2, 3) { (1, _, 3) => 100, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(100));
}

#[test]
fn test_match_multiple_arms() {
    let code = r#"
        match 2 {
            0 => 10,
            1 => 20,
            2 => 30,
            3 => 40,
            _ => 50,
        }
    "#;
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(30));
}
```

## After Adding Tests (Check Progress)

```bash
# Add tests to tests/priority_3_eval_control_flow.rs
# Run tests
cargo test --test priority_3_eval_control_flow

# Check coverage
cargo llvm-cov --all-targets 2>&1 | grep "eval_control_flow_new.rs"
```

**Expected Progress**: 29% → 45-50% with these 15 tests

## Remaining Work (Ordered by Priority)

1. **Category 3: Iteration** - 8 tests (30 min)
   - Empty arrays, negative ranges, inclusive ranges

2. **Category 4: Expressions** - 9 tests (30 min)
   - Error cases, array init, return outside function

3. **Property Tests** - 8 properties × 10K cases (30 min)
   - Use proptest crate
   - See analysis doc for examples

4. **Mutation Testing** - Iterate until ≥75% (30 min)
   - Run cargo-mutants
   - Add tests for MISSED mutations

## Commands Reference

```bash
# Run all library tests
cargo test --lib

# Run specific test file
cargo test --test priority_3_eval_control_flow

# Check coverage (all targets)
cargo llvm-cov --all-targets 2>&1 | grep "eval_control_flow_new.rs"

# Generate HTML coverage report
cargo llvm-cov --all-targets --html
# Open: target/llvm-cov/html/src/runtime/eval_control_flow_new.rs.html

# Run mutation testing
cargo mutants --file src/runtime/eval_control_flow_new.rs --timeout 300

# Check P0 tests
cargo test p0_critical_features

# Format code
cargo fmt

# Commit
git add docs/execution/PRIORITY_3_*.md tests/priority_3_eval_control_flow.rs
git commit -m "[PRIORITY-3] Continue eval_control_flow coverage (X% → Y%)"
```

## Success Criteria

- [ ] Coverage: 29% → 80%+ (**need +51%**)
- [ ] Tests: 10 → 40+ passing (**need +30**)
- [ ] Property tests: 8 × 10K cases (**80,000 executions**)
- [ ] Mutation kill rate: ≥75%
- [ ] P0 tests: 15/15 passing (no regressions)
- [ ] PMAT: A- grade maintained

## Time Budget

- Loop Control: 30 min
- Pattern Matching: 30 min
- Iteration: 30 min
- Expressions: 30 min
- Property Tests: 30 min
- Mutation Testing: 30 min
- **Total**: 3 hours (with buffer)

## Files to Edit

1. `tests/priority_3_eval_control_flow.rs` - Add tests here
2. `docs/execution/PRIORITY_3_EVAL_CONTROL_FLOW.md` - Update status
3. `docs/execution/roadmap.md` - Update progress

## Quick Win Strategy

If time-constrained, focus on:
1. Loop control tests (7 tests = +40 lines)
2. Pattern matching tests (8 tests = +45 lines)
3. Skip property/mutation if 80% reached with unit tests

This gets 85% of benefit in 50% of time.

---

**Last Updated**: 2025-10-09 (Commit a443e4c1)
**Module**: src/runtime/eval_control_flow_new.rs (499 lines)
**Sprint**: Priority-3 (Module 3/N)
