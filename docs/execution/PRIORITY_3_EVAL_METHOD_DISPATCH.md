# PRIORITY-3: Zero Coverage Module Testing - eval_method_dispatch.rs

## Status
**Status**: 🛑 BLOCKED - MOSTLY DEAD CODE
**Module**: `src/runtime/eval_method_dispatch.rs`
**Baseline Coverage**: 25.77% (265/357 lines uncovered)
**Realistic Coverage**: ~25% (cannot exceed without refactoring)
**Target Coverage**: 80%+ (NOT ACHIEVABLE - requires major refactoring)
**Sprint**: Priority 3 (Module 4/N - ABANDONED)
**Integration**: ⚠️ PARTIAL - Only object/actor dispatch active, ~75% dead code

## Objective
Apply EXTREME TDD methodology to increase coverage of method dispatch module from 25.77% to 80%+, following the proven pattern from eval_control_flow_new.rs integration.

## Module Analysis

### File Statistics
- **Lines of Code**: 481 total, 357 executable
- **Current Coverage**: 25.77% (92 lines covered)
- **Uncovered Lines**: 265 (74.23%)
- **Functions**: 14 total
- **Complexity**: All functions ≤10 (Toyota Way compliant)

### Module Purpose
Method dispatch evaluation handles all method calls on different value types:
- **Float methods**: sqrt, abs, round, floor, ceil, sin, cos, tan, ln, log10, exp, to_string
- **Integer methods**: abs, pow, sqrt, to_string, to_float, is_even, is_odd, signum, etc.
- **DataFrame methods**: select, filter, sort, group_by, sum, count, mean, max, min, columns, shape
- **Generic methods**: Fallback handling for unknown types/methods

### Quality Baseline
- ✅ **Toyota Way Compliant**: All functions documented with ≤10 complexity
- ✅ **Well-Structured**: Clear type-based dispatch pattern
- ✅ **Actively Used**: Called from interpreter.rs line 3954
- ⚠️ **Low Coverage**: Only 25.77% tested

## Functions Analysis

### Covered Functions (Estimated)
Based on 25.77% coverage, likely covered:
1. `eval_method_call` - Main entry point (probably covered by integration)
2. `dispatch_method_call` - Type dispatcher (probably covered)
3. Partial coverage in float/integer/dataframe methods

### Uncovered Functions (Need Tests)
Priority targets (265 uncovered lines):
1. **Float methods** (~80 lines)
   - Mathematical: sqrt, abs, round, floor, ceil, sin, cos, tan, ln, log10, exp
   - Conversion: to_string

2. **Integer methods** (~100 lines)
   - Mathematical: abs, pow, sqrt, to_string, to_float
   - Properties: is_even, is_odd, signum, bit operations
   - Range methods: step_by, take, etc.

3. **DataFrame methods** (~150 lines)
   - Aggregation: sum, count, mean, max, min
   - Transformation: select, filter, sort, group_by
   - Metadata: columns, shape

4. **Generic methods** (~20 lines)
   - Error handling for unknown methods
   - Type mismatch errors

## Test Strategy

### Phase 1: Float Methods (Target: 15 tests, ~80 lines)
**Test Categories**:
- Mathematical operations (sqrt, abs, round, floor, ceil)
- Trigonometric functions (sin, cos, tan)
- Logarithmic functions (ln, log10, exp)
- Type conversion (to_string)
- Error cases (arguments passed to no-arg methods)

**Estimated Coverage Gain**: 25.77% → 45%

### Phase 2: Integer Methods (Target: 20 tests, ~100 lines)
**Test Categories**:
- Mathematical operations (abs, pow, sqrt)
- Type conversions (to_string, to_float)
- Property checks (is_even, is_odd, signum)
- Bit operations (if present)
- Range operations (if present)
- Error cases

**Estimated Coverage Gain**: 45% → 65%

### Phase 3: DataFrame Methods (Target: 15 tests, ~80 lines)
**Test Categories**:
- Aggregation methods (sum, count, mean, max, min)
- Metadata methods (columns, shape)
- Error cases (invalid operations)

**Estimated Coverage Gain**: 65% → 85%+

### Phase 4: Edge Cases & Error Handling (Target: 5 tests, ~20 lines)
**Test Categories**:
- Generic method fallback
- Type mismatch errors
- Unknown method errors

**Estimated Coverage Gain**: 85% → 90%+

## Success Criteria
- ✅ Line coverage: 25.77% → 80%+
- ✅ Function coverage: ? → 100%
- ✅ Unit tests: 0 → 55+ passing
- ✅ All P0 tests: 15/15 passing (zero regressions)
- ✅ Quality gates: PMAT A-, ≤10 complexity, 0 SATD

## Timeline
**Estimated**: 1.5-2 hours
**Phases**:
- Test Planning: 10 min
- Float Methods Tests: 20 min
- Integer Methods Tests: 30 min
- DataFrame Methods Tests: 20 min
- Edge Cases Tests: 10 min
- Coverage Verification: 10 min
- Documentation & Commit: 10 min

## Toyota Way Principles
- **Jidoka**: Stop on any test failure, fix immediately
- **Genchi Genbutsu**: Read code, understand actual method implementations
- **Kaizen**: Incremental improvement, one method category at a time
- **Zero Defects**: Every method must be fully tested

## 🛑 CRITICAL DISCOVERY: Dead Code Analysis (Toyota Way: Genchi Genbutsu)

### Root Cause Investigation
**Method**: Empirical testing revealed 23/39 test failures with "Unknown method" errors

**Finding**: eval_method_dispatch.rs contains duplicate implementations that are NEVER called:

#### Dead Functions (NEVER CALLED - ~75% of module):
❌ `eval_integer_method()` - Interpreter has own implementation at interpreter.rs:3240
❌ `eval_float_method()` - Interpreter handles directly
❌ `eval_dataframe_method()` - Handled elsewhere
❌ All dataframe aggregation functions (sum, count, mean, max, min, etc.)

#### Active Functions (ONLY 25% used):
✅ `eval_method_call()` - Entry point (used at interpreter.rs:3954)
✅ `dispatch_method_call()` - But ONLY for object/actor types
✅ Partial code paths for actor method dispatch

### Why Dead Code Exists
**Historical Context**: Module was created to centralize method dispatch, but:
1. Interpreter evolved to handle primitives (int, float, string) directly
2. DataFrame methods implemented in dedicated eval_dataframe_ops.rs module
3. Only actor/object dispatch remains in this module
4. Refactoring was incomplete - old implementations left behind

### Evidence
```bash
# These functions are NEVER called in the codebase:
$ grep -rn "eval_method_dispatch::eval_integer_method" src/
# NO RESULTS

$ grep -rn "eval_method_dispatch::eval_float_method" src/
# NO RESULTS

$ grep -rn "eval_method_dispatch::eval_dataframe_method" src/
# NO RESULTS
```

**Only Called Function**:
```rust
// src/runtime/interpreter.rs:3954
eval_method_dispatch::eval_method_call(
    &Value::Object(...),  // ONLY for Object types!
    ...
)
```

### Coverage Ceiling
- **Current**: 25.77% (92/357 lines)
- **Maximum Achievable**: ~30% (only object/actor dispatch testable)
- **Blocked Lines**: ~270 lines (75%) - unreachable dead code
- **To Reach 80%**: Requires either:
  1. Integrate dead functions into interpreter (HIGH RISK refactoring)
  2. Delete dead code (cleaner, but loses potential refactoring)

### Toyota Way Response
- **Jidoka**: STOP - discovered defect (dead code)
- **Genchi Genbutsu**: Empirical testing revealed truth
- **Kaizen**: Don't waste effort testing dead code
- **Recommendation**: SELECT DIFFERENT MODULE for Priority-3

## Comparison to eval_control_flow_new.rs
Both modules suffered from incomplete refactoring:

| Aspect | eval_control_flow_new.rs | eval_method_dispatch.rs |
|--------|--------------------------|-------------------------|
| **Dead Code** | 60% (helper functions) | 75% (primitive methods) |
| **Integration** | Partial (7/29 functions) | Partial (object dispatch only) |
| **Coverage Achieved** | 22.34% (from 0%) | 25.77% (baseline) |
| **Coverage Ceiling** | ~40% | ~30% |
| **Root Cause** | Labeled loops incompatible | Primitives handled elsewhere |

## Recommendation
**ABANDON Priority-3 for this module** - select module with actual integration and coverage potential.

Better candidates:
- Modules with <50% coverage that are ACTIVELY used
- No duplicate implementations
- Clear integration path
