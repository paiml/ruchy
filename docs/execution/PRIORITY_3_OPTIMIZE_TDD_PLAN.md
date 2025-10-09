# Priority 3: Zero Coverage Module Testing - optimize.rs Extreme TDD Plan

**Target Module**: `src/middleend/mir/optimize.rs`
**Current Coverage**: 1.36% (8/590 regions)
**Target Coverage**: 80%+ (472+ regions)
**Test Strategy**: Extreme TDD (Unit + Property + Mutation)

---

## Current State Analysis

### Coverage Gaps (from cargo llvm-cov)
- **Total Regions**: 590
- **Covered Regions**: 8 (1.36%)
- **Missed Regions**: 582 (98.64%)
- **Functions**: 41 total, 37 completely untested (90%)

### Existing Tests (Quality Issues)
1. **Placeholder Unit Tests** (lines 530-547):
   - `test_dead_code_elimination()` - Empty
   - `test_constant_propagation()` - Empty
   - `test_common_subexpression_elimination()` - Empty
   - Comment: "deferred pending MIR completion" (WRONG - code exists!)

2. **Property Test Template** (lines 550-566):
   - `test_new_never_panics()` - Doesn't test actual optimization logic
   - Just catches panics, provides zero validation

3. **Doctest Defects**:
   - Lines 22-45: Duplicate/incorrect doctests (copy-paste error)
   - Lines 53-76: More duplicate doctests
   - All use `ignore` - none are runnable
   - References non-existent functions like `optimize::new()`

### Root Cause (Five Whys)
1. **Why 1.36% coverage?** - Tests are empty stubs
2. **Why are tests empty?** - Comment says "deferred pending MIR completion"
3. **Why defer?** - Assumption that MIR wasn't complete
4. **Why assume incomplete?** - No verification (violated Genchi Genbutsu)
5. **Why no verification?** - Tests weren't written first (TDD not followed)

**ROOT CAUSE**: TDD not followed. Tests written as placeholders instead of driving implementation.

---

## Extreme TDD Test Plan

### Phase 1: Unit Tests (Target: 80%+ Line Coverage)

#### 1.1 DeadCodeElimination Tests

**Test Function: `DeadCodeElimination::new()`**
- ✅ Test creates empty live_locals set
- ✅ Test creates empty live_blocks set

**Test Function: `DeadCodeElimination::run()`**
- ✅ Test removes dead locals (unused variables)
- ✅ Test removes dead statements (assignments to unused locals)
- ✅ Test removes dead blocks (unreachable code after return)
- ✅ Test preserves live locals (used variables)
- ✅ Test preserves function parameters (always live)
- ✅ Test preserves entry block (always live)
- ✅ Test handles empty function (no statements)

**Test Function: `mark_live()` (via run() behavior)**
- ✅ Test marks entry block as live
- ✅ Test marks function parameters as live
- ✅ Test propagates liveness through control flow
- ✅ Test handles loops (marks all blocks in loop)
- ✅ Test handles unreachable blocks (not marked)

**Test Function: `mark_statement_live()` (via run() behavior)**
- ✅ Test marks locals in Assign statements
- ✅ Test marks locals in StorageLive statements
- ✅ Test marks locals in StorageDead statements
- ✅ Test handles Nop statements

**Test Function: `mark_place_live()` (via run() behavior)**
- ✅ Test marks Place::Local
- ✅ Test marks Place::Field base
- ✅ Test marks Place::Deref base
- ✅ Test marks Place::Index base and index

**Test Function: `mark_rvalue_live()` (via run() behavior)**
- ✅ Test marks Rvalue::Use operand
- ✅ Test marks Rvalue::UnaryOp operand
- ✅ Test marks Rvalue::BinaryOp both operands
- ✅ Test marks Rvalue::Ref place
- ✅ Test marks Rvalue::Aggregate all operands
- ✅ Test marks Rvalue::Call function and arguments

**Test Function: `mark_operand_live()` (via run() behavior)**
- ✅ Test marks Operand::Copy place
- ✅ Test marks Operand::Move place
- ✅ Test handles Operand::Constant (no locals to mark)

**Test Function: `mark_terminator_live()` (via run() behavior)**
- ✅ Test marks Terminator::Goto target block
- ✅ Test marks Terminator::If condition and both branch blocks
- ✅ Test marks Terminator::Switch discriminant and all target blocks
- ✅ Test marks Terminator::Return operand (if present)
- ✅ Test marks Terminator::Call function, args, and destination block
- ✅ Test handles Terminator::Unreachable

**Test Function: `remove_dead_statements()`**
- ✅ Test removes assignments to dead locals
- ✅ Test preserves assignments to live locals
- ✅ Test removes all Nop statements
- ✅ Test handles blocks with only dead statements

**Test Function: `remove_dead_blocks()`**
- ✅ Test removes unreachable blocks
- ✅ Test preserves reachable blocks
- ✅ Test handles function with only entry block

**Test Function: `remove_dead_locals()`**
- ✅ Test removes unused local declarations
- ✅ Test preserves used local declarations
- ✅ Test preserves parameter locals

**Test Function: `is_place_live()`**
- ✅ Test returns true for live local
- ✅ Test returns false for dead local
- ✅ Test recursively checks base of Field
- ✅ Test recursively checks base of Deref
- ✅ Test checks both base and index of Index

#### 1.2 ConstantPropagation Tests

**Test Function: `ConstantPropagation::new()`**
- ✅ Test creates empty constants map

**Test Function: `ConstantPropagation::run()`**
- ✅ Test propagates integer constants
- ✅ Test propagates boolean constants
- ✅ Test folds binary operations (2 + 3 = 5)
- ✅ Test folds unary operations (-5, !true)
- ✅ Test handles non-constant values (no propagation)
- ✅ Test clears state between runs

**Test Function: `extract_constant()`**
- ✅ Test extracts Constant from Rvalue::Use
- ✅ Test evaluates BinaryOp with two constants
- ✅ Test evaluates UnaryOp with one constant
- ✅ Test returns None for non-constant Rvalue

**Test Function: `eval_binary_op()`**
- ✅ Test Add: Int(2) + Int(3) = Int(5)
- ✅ Test Sub: Int(5) - Int(2) = Int(3)
- ✅ Test Mul: Int(3) * Int(4) = Int(12)
- ✅ Test Eq: Int(5) == Int(5) = Bool(true)
- ✅ Test Lt: Int(3) < Int(5) = Bool(true)
- ✅ Test And: Bool(true) && Bool(false) = Bool(false)
- ✅ Test Or: Bool(true) || Bool(false) = Bool(true)
- ✅ Test returns None for non-int/bool combinations

**Test Function: `eval_unary_op()`**
- ✅ Test Neg: -Int(5) = Int(-5)
- ✅ Test Not: !Bool(true) = Bool(false)
- ✅ Test returns None for type mismatches

**Test Function: `get_constant_value()`**
- ✅ Test returns constant from Operand::Constant
- ✅ Test looks up local from Operand::Copy
- ✅ Test looks up local from Operand::Move
- ✅ Test returns None for unknown locals

**Test Function: `propagate_in_statement()`**
- ✅ Test replaces constant locals in Assign rvalue
- ✅ Test handles non-Assign statements (no crash)

**Test Function: `propagate_in_rvalue()`**
- ✅ Test propagates in Rvalue::Use
- ✅ Test propagates in Rvalue::UnaryOp
- ✅ Test propagates in Rvalue::BinaryOp (both operands)
- ✅ Test propagates in Rvalue::Call (function + all args)

**Test Function: `propagate_in_operand()`**
- ✅ Test replaces local with constant
- ✅ Test leaves constants unchanged

**Test Function: `propagate_in_terminator()`**
- ✅ Test propagates in Terminator::If condition
- ✅ Test propagates in Terminator::Switch discriminant
- ✅ Test propagates in Terminator::Return operand
- ✅ Test propagates in Terminator::Call (function + args)

#### 1.3 CommonSubexpressionElimination Tests

**Test Function: `CommonSubexpressionElimination::new()`**
- ✅ Test creates empty expressions map

**Test Function: `CommonSubexpressionElimination::run()`**
- ✅ Test eliminates duplicate BinaryOp expressions
- ✅ Test eliminates duplicate UnaryOp expressions
- ✅ Test preserves first occurrence of expression
- ✅ Test replaces subsequent occurrences with Copy
- ✅ Test handles expressions with different operands (no elimination)
- ✅ Test clears state between runs

**Test Function: `process_statement()`**
- ✅ Test records new expression
- ✅ Test reuses existing expression
- ✅ Test handles non-Assign statements (no crash)

**Test Function: `rvalue_key()`**
- ✅ Test generates unique key for Rvalue::Use
- ✅ Test generates unique key for Rvalue::BinaryOp
- ✅ Test generates unique key for Rvalue::UnaryOp
- ✅ Test same expressions generate same key
- ✅ Test different expressions generate different keys

**Test Function: `operand_key()`**
- ✅ Test generates key for Operand::Copy
- ✅ Test generates key for Operand::Move
- ✅ Test generates key for Operand::Constant

**Test Function: `place_key()`**
- ✅ Test generates key for Place::Local
- ✅ Test generates key for Place::Field (recursive)
- ✅ Test generates key for Place::Index (recursive)
- ✅ Test generates key for Place::Deref (recursive)

#### 1.4 Integration Tests

**Test Function: `optimize_function()`**
- ✅ Test runs all passes in correct order
- ✅ Test runs 3 iterations for convergence
- ✅ Test combines const prop + CSE + DCE effectively
- ✅ Test handles empty function (no crash)

**Test Function: `optimize_program()`**
- ✅ Test optimizes all functions in program
- ✅ Test handles program with no functions (no crash)
- ✅ Test handles program with multiple functions

---

### Phase 2: Property Tests (Target: 10,000+ Cases Per Property)

#### Property 1: Optimization Preserves Semantics
```rust
proptest! {
    #[test]
    fn prop_optimization_preserves_semantics(func: Function) {
        let original = func.clone();
        let mut optimized = func.clone();
        optimize_function(&mut optimized);

        // Execute both and compare results
        // Property: output(original) == output(optimized)
        assert_eq!(execute_function(&original), execute_function(&optimized));
    }
}
```

#### Property 2: Dead Code Elimination is Idempotent
```rust
proptest! {
    #[test]
    fn prop_dce_idempotent(func: Function) {
        let mut once = func.clone();
        let mut twice = func.clone();

        let mut dce = DeadCodeElimination::new();
        dce.run(&mut once);
        dce.run(&mut twice);
        dce.run(&mut twice);

        // Property: DCE(DCE(x)) == DCE(x)
        assert_eq!(once, twice);
    }
}
```

#### Property 3: Entry Block Always Live
```rust
proptest! {
    #[test]
    fn prop_entry_block_always_live(func: Function) {
        let mut optimized = func.clone();
        let mut dce = DeadCodeElimination::new();
        dce.run(&mut optimized);

        // Property: entry block always present after DCE
        assert!(optimized.blocks.iter().any(|b| b.id == func.entry_block));
    }
}
```

#### Property 4: Parameters Always Live
```rust
proptest! {
    #[test]
    fn prop_parameters_always_live(func: Function) {
        let original_params = func.params.clone();
        let mut optimized = func.clone();
        optimize_function(&mut optimized);

        // Property: all parameters remain after optimization
        for param in &original_params {
            assert!(optimized.locals.iter().any(|l| l.id == *param));
        }
    }
}
```

#### Property 5: Optimization Reduces or Maintains Size
```rust
proptest! {
    #[test]
    fn prop_optimization_reduces_size(func: Function) {
        let original_statements = count_statements(&func);
        let original_blocks = func.blocks.len();
        let original_locals = func.locals.len();

        let mut optimized = func.clone();
        optimize_function(&mut optimized);

        // Property: optimized has <= original statement count
        assert!(count_statements(&optimized) <= original_statements);
        // Property: optimized has <= original block count
        assert!(optimized.blocks.len() <= original_blocks);
        // Property: optimized has <= original local count
        assert!(optimized.locals.len() <= original_locals);
    }
}
```

#### Property 6: Constant Propagation Doesn't Create New Locals
```rust
proptest! {
    #[test]
    fn prop_const_prop_no_new_locals(func: Function) {
        let original_local_count = func.locals.len();
        let mut optimized = func.clone();
        let mut const_prop = ConstantPropagation::new();
        const_prop.run(&mut optimized);

        // Property: const prop only replaces, never adds locals
        assert_eq!(optimized.locals.len(), original_local_count);
    }
}
```

#### Property 7: CSE Doesn't Change Block Count
```rust
proptest! {
    #[test]
    fn prop_cse_preserves_blocks(func: Function) {
        let original_block_count = func.blocks.len();
        let mut optimized = func.clone();
        let mut cse = CommonSubexpressionElimination::new();
        cse.run(&mut optimized);

        // Property: CSE only changes statements, not control flow
        assert_eq!(optimized.blocks.len(), original_block_count);
    }
}
```

#### Property 8: All Optimization Functions Never Panic
```rust
proptest! {
    #[test]
    fn prop_optimizations_never_panic(func: Function) {
        let mut f1 = func.clone();
        let mut f2 = func.clone();
        let mut f3 = func.clone();

        // Property: optimizations never panic on any input
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            DeadCodeElimination::new().run(&mut f1);
            ConstantPropagation::new().run(&mut f2);
            CommonSubexpressionElimination::new().run(&mut f3);
        }));
    }
}
```

#### Property Test Configuration
- **Cases per property**: 10,000+ (proptest default with config override)
- **Max shrink iters**: 1,000 (find minimal failing case)
- **Timeout**: 300s per property

---

### Phase 3: Mutation Testing (Target: ≥75% Kill Rate)

#### Mutation Testing Strategy

**Run Command**:
```bash
cargo mutants --file src/middleend/mir/optimize.rs --timeout 300
```

**Expected Mutation Categories**:

1. **Logical Operators** (most common):
   - `&&` ↔ `||` (should be caught by boolean constant tests)
   - `==` ↔ `!=` (should be caught by comparison tests)
   - `<` ↔ `<=` ↔ `>` ↔ `>=` (should be caught by boundary tests)

2. **Arithmetic Operators**:
   - `+` ↔ `-` ↔ `*` (should be caught by constant folding tests)
   - `-x` ↔ `x` (should be caught by unary op tests)

3. **Function Return Values**:
   - Return `None` instead of `Some(...)` (should be caught by extraction tests)
   - Return empty collections (should be caught by optimization tests)

4. **Match Arm Deletion**:
   - Delete match arms (should be caught by comprehensive case tests)

5. **Loop Mutations**:
   - Skip loop iterations (should be caught by iteration tests)

#### Test Gap Analysis Process

1. **Run mutation tests**: `cargo mutants --file src/middleend/mir/optimize.rs`
2. **Analyze MISSED mutations**: Identify specific mutations that survived
3. **Write targeted tests**: Create tests that explicitly catch each MISSED mutation
4. **Re-run mutation tests**: Verify kill rate increases
5. **Repeat until ≥75%**: Continue until target achieved

#### Expected Mutation Coverage

Based on Sprint 8 empirical data:
- **Initial run**: 50-60% kill rate (common for untested code)
- **After unit tests**: 65-70% kill rate
- **After property tests**: 75-80% kill rate
- **After targeted fixes**: 80-85% kill rate

**Target**: ≥75% mutation coverage (CAUGHT/(CAUGHT+MISSED) ≥ 75%)

---

### Phase 4: Coverage Verification

**Command**:
```bash
cargo llvm-cov --html --open --ignore-filename-regex '(tests/|examples/)'
```

**Success Criteria**:
- ✅ optimize.rs: ≥80% line coverage (472+ regions)
- ✅ All public functions: 100% coverage
- ✅ All key private functions: ≥80% coverage
- ✅ All property tests: passing 10,000+ cases
- ✅ Mutation tests: ≥75% kill rate

**Current → Target**:
- **Line Coverage**: 1.36% → 80%+ (59x improvement)
- **Region Coverage**: 8/590 → 472/590
- **Function Coverage**: 10% → 90%+
- **Property Tests**: 0 → 8+ properties × 10K cases
- **Mutation Coverage**: 0% → 75%+

---

## Implementation Timeline

### Estimated Effort

**Phase 1 - Unit Tests**: ~3-4 hours
- DeadCodeElimination: 12 test functions × 5-10 test cases = ~60 tests
- ConstantPropagation: 10 test functions × 5-10 test cases = ~50 tests
- CommonSubexpressionElimination: 6 test functions × 5-10 test cases = ~30 tests
- Integration: 3 test functions × 5 test cases = ~15 tests
- **Total**: ~155 unit tests

**Phase 2 - Property Tests**: ~1-2 hours
- 8 property tests × 10,000 cases = 80,000 test executions
- Each property needs helper functions and generators
- **Total**: ~8 property test functions

**Phase 3 - Mutation Testing**: ~2-3 hours
- Initial run: ~30 min
- Gap analysis: ~30 min
- Targeted test writing: ~1-2 hours
- Re-verification: ~30 min
- **Total**: Iterative until ≥75% achieved

**Phase 4 - Coverage Verification**: ~30 min
- Generate report
- Analyze gaps
- Write final tests if needed

**Total Estimated Time**: 6-9 hours

---

## Success Metrics

### Quantitative
- ✅ **Coverage**: 1.36% → 80%+ (59x improvement)
- ✅ **Unit Tests**: 3 placeholders → 155+ real tests
- ✅ **Property Tests**: 1 template → 8 properties × 10K cases
- ✅ **Mutation Coverage**: 0% → 75%+
- ✅ **Functions Tested**: 4 → 41 (10x improvement)

### Qualitative
- ✅ **Confidence**: Empirical proof that optimizations work correctly
- ✅ **Maintainability**: Comprehensive test suite prevents regressions
- ✅ **Documentation**: Tests serve as executable specification
- ✅ **Quality**: A+ code standard (≤10 complexity, 100% coverage)

---

## Risk Mitigation

### Risk 1: MIR Types Not Fully Implemented
**Mitigation**: Use builder pattern for test fixtures, mock incomplete features
**Fallback**: Mark tests as `#[ignore]` with tracking issue

### Risk 2: Property Tests Take Too Long
**Mitigation**: Use smaller case counts for fast feedback, full 10K for CI
**Fallback**: Reduce to 1,000 cases per property (still valuable)

### Risk 3: Mutation Testing Timeout
**Mitigation**: Increase timeout to 600s, optimize slow tests
**Fallback**: Test specific modules, not entire file at once

### Risk 4: Coverage Below 80%
**Mitigation**: Identify untestable code (e.g., debug logging), document why
**Fallback**: 75% acceptable if remaining 5% is documented as untestable

---

## References

- **CLAUDE.md**: Extreme TDD Protocol (lines 18-138)
- **CLAUDE.md**: Mutation Testing Protocol (lines 90-129)
- **CLAUDE.md**: A+ Code Standard (lines 6-49)
- **Sprint 8 Mutation Data**: paiml-mcp-agent-toolkit empirical results
- **Coverage Baseline**: /tmp/p0_test_output.log (15/15 P0 tests passing)

---

**READY TO EXECUTE**: This plan follows Toyota Way (Genchi Genbutsu - go see the actual code) and Extreme TDD (RED→GREEN→REFACTOR with comprehensive validation).

**Next Step**: Begin Phase 1 - Unit Tests (DeadCodeElimination::new)
