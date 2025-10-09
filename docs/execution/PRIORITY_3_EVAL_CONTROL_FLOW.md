# PRIORITY-3: Zero Coverage Module Testing - eval_control_flow_new.rs

## Status
**Status**: 🟡 IN PROGRESS (Partial Integration Complete)
**Module**: `src/runtime/eval_control_flow_new.rs`
**Baseline Coverage**: 0.00% (dead code - not integrated)
**Current Coverage**: 22.34% (306/394 lines uncovered)
**Realistic Ceiling**: ~40% (interpreter uses different loop/match implementations)
**Target Coverage**: 80%+ (requires major interpreter refactoring)
**Sprint**: Priority 3 (Module 3/N)
**Integration**: ✅ 7/29 functions integrated, 41/41 tests passing

## Objective
Apply EXTREME TDD methodology to increase coverage of control flow evaluation module from 29.06% to 80%+, following the proven pattern from optimize.rs (61x improvement) and wasm/mod.rs (41x improvement).

## Module Analysis

### File Statistics
- **Lines of Code**: 499
- **Current Coverage**: 29.06%
- **Uncovered Lines**: 354 (70.94%)
- **Functions**: ~12-15 (estimated)

### Module Purpose
Control flow evaluation module handles:
- If expressions (with optional else)
- Let expressions (variable binding)
- While loops
- For loops
- Match statements with pattern matching
- Loop control (break/continue)

### Quality Baseline
- ✅ **Toyota Way Compliant**: All functions documented with ≤10 complexity
- ✅ **Well-Structured**: Extracted for maintainability
- ✅ **Clear Interfaces**: Standalone functions with callback patterns
- ⚠️ **Low Coverage**: Only 29.06% tested

## Test Strategy

### Phase 1: Unit Tests (Target: 20-25 tests)
Test each public function with:
- Happy path scenarios
- Edge cases (empty, nil, errors)
- Error conditions
- Boundary values

**Functions to Test**:
1. `eval_if_expr` - if/else evaluation
2. `eval_let_expr` - variable binding
3. `eval_while_loop` - while loop execution
4. `eval_for_loop` - for loop iteration
5. `eval_match` - pattern matching
6. `eval_loop_with_break` - loop control
7. Helper functions for pattern matching

### Phase 2: Property Tests (Target: 8-10 properties, 10K cases each)
**Property Categories**:
1. **Control Flow Invariants**:
   - If true condition always evaluates then-branch
   - If false condition with else always evaluates else-branch
   - While false condition never executes body

2. **Loop Invariants**:
   - For loop executes exactly N times for range 0..N
   - Break always terminates loop immediately
   - Continue skips to next iteration

3. **Pattern Matching Laws**:
   - Match always evaluates exactly one arm
   - Wildcard pattern matches anything
   - Literal patterns match only exact values

4. **Error Resilience**:
   - Invalid conditions produce appropriate errors
   - Type mismatches are caught
   - Functions never panic on valid AST

### Phase 3: Mutation Testing (Target: ≥75% kill rate)
Run `cargo mutants --file src/runtime/eval_control_flow_new.rs --timeout 300`

**Expected Mutation Categories**:
- Condition negations (!condition)
- Loop boundary changes (< vs <=)
- Pattern match arm deletions
- Return value replacements

## Success Criteria
- ✅ Line coverage: 29.06% → 80%+
- ✅ Function coverage: ? → 100%
- ✅ Unit tests: 0 → 20-25 passing
- ✅ Property tests: 0 → 8-10 properties (80K+ executions)
- ✅ Mutation coverage: ? → 75%+ kill rate
- ✅ All P0 tests: 15/15 passing (zero regressions)
- ✅ Quality gates: PMAT A-, ≤10 complexity, 0 SATD

## Timeline
**Estimated**: 1.5-2 hours (based on previous Priority 3 sprints)
**Phases**:
- Analysis & Test Planning: 15 min
- Unit Test Development: 30-45 min
- Property Test Development: 30 min
- Mutation Testing: 15-30 min
- Documentation & Commit: 15 min

## Toyota Way Principles
- **Jidoka**: Stop on any test failure, fix immediately
- **Genchi Genbutsu**: Read code, understand actual behavior
- **Kaizen**: Incremental improvement, one function at a time
- **Zero Defects**: Every function must be fully tested

## Integration Notes

### Successfully Integrated (7 functions)
✅ `eval_if_expr` - If/else expression evaluation
✅ `eval_return_expr` - Return statement handling
✅ `eval_list_expr` - Array literal creation
✅ `eval_array_init_expr` - Array initialization `[value; size]`
✅ `eval_block_expr` - Block statement evaluation
✅ `eval_tuple_expr` - Tuple literal creation
✅ `eval_range_expr` - Range expression `start..end`

### NOT Integrated (22 functions - ~60% of module)
❌ While loop helpers (eval_while_loop, eval_loop_condition, eval_loop_body, run_while_loop)
❌ For loop helpers (eval_for_loop, eval_array_iteration, eval_range_iteration)
❌ Match expression helpers (eval_match, eval_match_arm, eval_match_guard, find_matching_arm)
❌ Pattern matching functions (match_literal_pattern, match_list_pattern, match_tuple_pattern, etc.)
❌ Loop control helpers (handle_loop_control, extract_range_bounds, create_range_iterator)

**Blocker**: Interpreter evolved with labeled loop support and different control flow handling. Integrating these functions would require major interpreter refactoring (high risk, low benefit).

### Test Coverage
- **Tests Created**: 41 tests (25 initial + 16 added)
- **All Passing**: 41/41 ✅
- **P0 Tests**: 15/15 ✅ (zero regressions)
- **Test Categories**:
  - Basic control flow (10 tests)
  - Loop control (7 tests)
  - Pattern matching (8 tests)
  - Advanced iteration (8 tests)
  - Error cases & edge cases (8 tests)

### Coverage Ceiling Analysis
- **Integrated Functions**: ~40% of module (7/29 functions, ~158/394 lines)
- **Maximum Achievable**: ~40% without interpreter refactoring
- **Current Achievement**: 22.34% (88/394 lines)
- **Gap to Ceiling**: ~17.66% (~70 more lines possible with exhaustive edge case testing)
- **Gap to 80% Target**: 57.66% (~227 lines - requires integrating blocked functions)

## Notes
- Module already follows Toyota Way (≤10 complexity)
- Clear function separation makes testing straightforward
- Callback pattern requires careful test setup
- May need mock/stub functions for eval_expr closures
- **Recommendation**: Mark as "Partial Integration Complete" and move to next Priority-3 module
