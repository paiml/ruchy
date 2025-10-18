# expressions.rs Modularization Roadmap

## Executive Summary

**Goal**: Improve TDG score from 71.2/100 (B-) to ≥85/100 (A-) through systematic modularization

**Current Status** (as of 2025-10-18):
- File size: 4,922 lines, 241 functions
- TDG Score: 71.2/100 (B-)
- Progress: 25.7% complete (1,701 lines removed across 14 modules + dead code cleanup)

**Target**: Extract ~5,000 lines (≥75%) to achieve TDG ≥85

---

## ✅ Completed Phases (Commits: 149b3c96, c88d78bb)

### Phase 1: control_flow Module
- **Lines**: 177
- **Functions**: 4 (parse_break_token, parse_continue_token, parse_return_token, parse_throw_token)
- **Tests**: 5 unit tests
- **Quality**: Not measured (small module)

### Phase 2: visibility_modifiers Module ⭐
- **Lines**: 558
- **Functions**: 17 (6 primary + 11 helpers)
- **Tests**: 10 unit tests + 6 property tests
- **Quality**: **TDG 91.1/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)

### Phase 3: literals Module ⭐
- **Lines**: 206
- **Functions**: 1 (parse_literal_token)
- **Tests**: 7 unit tests + 6 property tests
- **Quality**: **TDG 92.9/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Dead Code Removed**: 58 lines (duplicate parse_literal_token function)

### Phase 4: identifiers Module ⭐
- **Lines**: 401
- **Functions**: 5 (parse_identifier_token, parse_module_path_segments, parse_path_segment, parse_turbofish_generics, token_to_keyword_string)
- **Tests**: 7 unit tests + 6 property tests
- **Quality**: **TDG 82.9/100 (B+ grade)** - Near A- target
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Cross-Module**: visibility_modifiers imports identifiers module

### Phase 5: arrays Module ⭐
- **Lines**: 275
- **Functions**: 5 (parse_list_literal, parse_array_element, parse_array_init, parse_regular_list, parse_list_comprehension_body)
- **Tests**: 7 unit tests + 6 property tests
- **Quality**: **TDG 93.5/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Cross-Module**: Delegates to collections module

### Phase 7: tuples Module ⭐
- **Lines**: 225
- **Functions**: 3 (parse_parentheses_token, parse_tuple_elements, maybe_parse_lambda)
- **Tests**: 7 unit tests + 6 property tests
- **Quality**: **TDG 93.1/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: Unit type, grouped expressions, tuples, tuple-to-lambda conversion

### Phase 8: dataframes Module ⭐
- **Lines**: 178
- **Functions**: 1 (parse_dataframe_token - dispatch between df! and df identifier)
- **Tests**: 7 unit tests + 6 property tests
- **Quality**: **TDG 93.0/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: DataFrame literal vs identifier dispatch, delegates to collections module

### Phase 9: unary_operators Module ⭐
- **Lines**: 286
- **Functions**: 9 (parse_unary_prefix + 8 operator functions)
- **Tests**: 6 unit tests + 4 property tests
- **Quality**: **TDG 91.3/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Dead Code Removed**: 79 lines (duplicate parse_unary_operator_token)
- **Operators**: -, !, *, &, **, await, ~, spawn

### Phase 10B: binary_operators Module ⭐
- **Lines**: 393 (including tests)
- **Functions**: 7 (token_to_binary_op, get_precedence, 5 mapping helpers)
- **Tests**: 8 unit tests + 7 property tests
- **Quality**: **TDG 97.2/100 (A+ grade)** - HIGHEST SCORE YET ⭐⭐⭐
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: Token→BinaryOp mapping, operator precedence, mathematical invariants
- **Re-exports**: Public API maintained for external callers

### Phase 11: string_operations Module ⭐
- **Lines**: 317 (including tests)
- **Functions**: 6 (parse_fstring_into_parts + 5 helpers)
- **Tests**: 9 unit tests + 7 property tests
- **Quality**: **TDG 91.4/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: F-string interpolation, escaped braces, nested expressions, format specifiers
- **Integration**: literals.rs imports directly from string_operations module

### Phase 12: loops Module ⭐
- **Lines**: 388 (including tests)
- **Functions**: 6 (parse_loop_label, parse_while_loop, parse_for_loop, parse_loop, parse_labeled_while_loop, parse_labeled_for_loop, parse_labeled_loop, parse_for_pattern)
- **Tests**: 10 unit tests + 7 property tests
- **Quality**: **TDG 93.2/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: For loops with pattern destructuring, while loops, while-let loops, infinite loops, labeled loops
- **expressions.rs reduction**: 147 lines removed (5,355 → 5,208)

### Phase 13: lambdas Module ⭐
- **Lines**: 265 (including tests)
- **Functions**: 3 (parse_lambda_no_params, parse_lambda_from_expr, parse_lambda_expression)
- **Tests**: 7 unit tests + 7 property tests
- **Quality**: **TDG 91.8/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: No-parameter lambdas (||), pipe-delimited lambdas (|x|), arrow syntax (x =>)
- **expressions.rs reduction**: 80 lines removed (5,208 → 5,128)

### Phase 14: variable_declarations Module ⭐
- **Lines**: 568 (including tests)
- **Functions**: 2 main (parse_let_statement, parse_var_statement) + 11 helpers
- **Tests**: 8 unit tests + 7 property tests
- **Quality**: **TDG 87.0/100 (A- grade)** - MEETS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: Let bindings, mutable bindings, type annotations, pattern matching, let-else, let-in, var statements
- **expressions.rs reduction**: 44 lines removed (5,128 → 5,084)
- **Note**: Module includes comprehensive let/var parsing with pattern support

### Phase 15: async_expressions Module ⭐
- **Lines**: 337 (including tests)
- **Functions**: 7 (parse_async_token, parse_async_function, parse_async_block, parse_async_lambda, parse_async_arrow_lambda, parse_async_lambda_params, parse_async_param_list, parse_single_async_param)
- **Tests**: 5 unit tests + 7 property tests (2 unit tests marked TODO for syntax verification)
- **Quality**: **TDG 92.8/100 (A grade)** - EXCEEDS TARGET
- **Methodology**: EXTREME TDD (RED→GREEN→REFACTOR)
- **Features**: Async functions, async blocks, async lambdas, async arrow lambdas
- **expressions.rs reduction**: 162 lines removed (5,084 → 4,922)

### Dead Code Cleanup ⭐
- **Lines Removed**: 142 (actor helper functions)
- **Functions Removed**: 6 (parse_actor_name, parse_actor_body, parse_actor_state_field, parse_actor_receive_block, parse_actor_bare_field, create_actor_expression)
- **Reason**: parse_actor_definition delegates to actors::parse_actor; functions were unreachable
- **Impact**: expressions.rs: 5,497 → 5,355 lines
- **Tests**: All 3,836 tests still passing
- **Commit**: `05052a8b`

**Total Removed**: 4,742 lines (1,701 from expressions.rs: 1,559 extracted + 142 dead code removed)

---

## 🗺️ Remaining Extraction Plan

### Priority Tiers (Based on Impact & Testability)

#### 🔴 **Tier 1: High Impact** (~2,500 lines, 40% of remaining)

**Phase 3: Literal & Primitive Parsing** ✅ COMPLETE
- **Lines**: 206 (expressions.rs: -89 lines including dead code)
- **Functions**: parse_literal_token (integers, floats, strings, chars, bytes, bools, fstrings)
- **Property Tests**: 6 (integers, floats, strings, bools, type suffixes, hex integers)
- **Unit Tests**: 7 (integer, float, string, char, bool, fstring, type suffix)
- **Quality**: TDG 92.9/100 (A grade) ⭐ EXCEEDS TARGET
- **Actual Effort**: 1.5 hours
- **Commit**: 44eab952

**Phase 4: Identifier & Path Resolution** ✅ COMPLETE
- **Lines**: 401 (expressions.rs: -202 lines)
- **Functions**: 5 (parse_identifier_token, parse_module_path_segments, parse_path_segment, parse_turbofish_generics, token_to_keyword_string)
- **Property Tests**: 6 (identifiers, qualified paths, triple paths, keywords, specials, lambdas)
- **Unit Tests**: 7 (simple, qualified, nested, underscore, self, super, lambda)
- **Quality**: TDG 82.9/100 (B+ grade) - Near A- target
- **Actual Effort**: 2 hours
- **Commit**: 5222eb61

**Phase 5: Array & Collection Literals** ✅ COMPLETE
- **Lines**: 275 (expressions.rs: -72 lines)
- **Functions**: 5 (parse_list_literal, parse_array_element, parse_array_init, parse_regular_list, parse_list_comprehension_body)
- **Property Tests**: 6 (empty, single, multi, init, trailing commas, nested)
- **Unit Tests**: 7 (empty, simple, trailing comma, init, spread, nested, mixed)
- **Quality**: TDG 93.5/100 (A grade) ⭐ EXCEEDS TARGET
- **Actual Effort**: 1.5 hours
- **Commit**: bde658a6

**Phase 6: Object & Struct Literals** (~500 lines)
- **Location**: Scattered across file
- **Functions**:
  - Object literal parsing `{ key: value }`
  - Destructuring
  - Shorthand syntax
  - Computed keys
- **Property Tests**:
  - Nested objects
  - Key uniqueness
  - Various value types
- **Estimated Effort**: 4 hours
- **TDG Impact**: Medium (~8% reduction)

**Phase 7: Tuple Parsing** ✅ COMPLETE
- **Lines**: 225 (expressions.rs: -51 lines)
- **Functions**: 3 (parse_parentheses_token, parse_tuple_elements, maybe_parse_lambda)
- **Property Tests**: 6 (empty, pairs, nested, trailing commas, large tuples, lambda conversion)
- **Unit Tests**: 7 (unit type, single element, pairs, triples, nested, grouped, lambda)
- **Quality**: TDG 93.1/100 (A grade) ⭐ EXCEEDS TARGET
- **Actual Effort**: 1.5 hours
- **Commit**: 85bce0dc

**Phase 8: DataFrame Literals** ✅ COMPLETE
- **Lines**: 178 (expressions.rs: -14 lines)
- **Functions**: 1 (parse_dataframe_token - dispatch between df! and df identifier)
- **Property Tests**: 6 (identifier, empty literal, integer columns, method chains, column names, multiple columns)
- **Unit Tests**: 7 (empty, single column, multiple columns, identifier, method call, assignment, integers)
- **Quality**: TDG 93.0/100 (A grade) ⭐ EXCEEDS TARGET
- **Actual Effort**: 0.5 hours
- **Commit**: 55f107c9

**Phase 9: Unary & Prefix Operators** (~300 lines)
- **Location**: Lines 666-740
- **Functions**:
  - `parse_unary_operator_token` (line 666)
  - Negation, not, reference, dereference
  - Increment/decrement (lines 5883-5903)
- **Property Tests**:
  - Operator precedence
  - Nested unary ops
  - Left-associativity
- **Estimated Effort**: 3 hours
- **TDG Impact**: Low-Medium (~5% reduction)

#### 🟡 **Tier 2: Medium Impact** (~1,500 lines, 24% of remaining)

**Phase 10: Pattern Matching Constructs** (~400 lines)
- Pattern destructuring
- Match arms
- Guards
- **Estimated Effort**: 4 hours
- **TDG Impact**: Medium (~6% reduction)

**Phase 11: Function Declarations** (~400 lines)
- Function parsing
- Lambda expressions
- Arrow functions
- **Estimated Effort**: 4 hours
- **TDG Impact**: Medium (~6% reduction)

**Phase 12: Loop Constructs** (~300 lines)
- For loops
- While loops
- Loop labels
- **Estimated Effort**: 3 hours
- **TDG Impact**: Low-Medium (~5% reduction)

**Phase 13: Type Annotations** (~400 lines)
- Type parsing
- Generic constraints
- Trait bounds
- **Estimated Effort**: 4 hours
- **TDG Impact**: Medium (~6% reduction)

#### 🟢 **Tier 3: Low Impact / Cleanup** (~1,000 lines, 16% of remaining)

**Phase 14: String Operations** (~300 lines)
- String concatenation
- Template literals
- **Estimated Effort**: 2 hours

**Phase 15: Async/Await** (~200 lines)
- Async function parsing
- Await expressions
- **Estimated Effort**: 2 hours

**Phase 16: Module System** (~300 lines)
- Import statements
- Export declarations
- **Estimated Effort**: 3 hours

**Phase 17: Miscellaneous** (~200 lines)
- Remaining scattered functionality
- Utility functions
- **Estimated Effort**: 2 hours

---

## 📊 Effort Estimation

### Total Remaining Work
- **Lines to Extract**: ~5,000 lines (75% of current file)
- **Modules to Create**: 15 additional modules
- **Total Estimated Time**: 50-60 hours
- **Average per Module**: 3-4 hours

### Suggested Sprint Plan (2-week iterations)

**Sprint 1: Tier 1 Phases 3-5** (9 hours)
- Literals, Identifiers, Arrays
- Target: 1,200 lines extracted (~19% reduction)

**Sprint 2: Tier 1 Phases 6-9** (12 hours)
- Objects, Tuples, DataFrames, Unary Ops
- Target: 1,300 lines extracted (~21% reduction)

**Sprint 3: Tier 2 Phases 10-11** (8 hours)
- Patterns, Functions
- Target: 800 lines extracted (~13% reduction)

**Sprint 4: Tier 2 Phases 12-13** (7 hours)
- Loops, Types
- Target: 700 lines extracted (~11% reduction)

**Sprint 5: Tier 3 Cleanup** (6 hours)
- Strings, Async, Modules, Misc
- Target: 1,000 lines extracted (~16% reduction)

**Total: 5 sprints, ~42 hours**

---

## 🎯 Success Metrics

### Primary Goal: TDG Score
- **Current**: 71.2/100 (B-)
- **Target**: ≥85/100 (A-)
- **Stretch**: ≥90/100 (A)

### Secondary Goals
1. **File Size**: expressions.rs ≤1,000 lines (84% reduction)
2. **Function Count**: ≤50 functions per file
3. **Module Quality**: All modules achieve TDG ≥85 (A-)
4. **Test Coverage**:
   - Unit tests: All existing tests migrated
   - Property tests: ≥3 per module (EXTREME TDD)
   - Mutation coverage: ≥75% (cargo-mutants)

### Code Quality Standards (Per CLAUDE.md)
- **Cyclomatic Complexity**: ≤10 per function
- **Cognitive Complexity**: ≤10 per function
- **SATD Comments**: Zero (no TODO/FIXME/HACK)
- **Documentation**: Comprehensive with examples

---

## 🔧 Technical Guidelines

### Module Naming Convention
```
src/frontend/parser/expressions_helpers/
├── arrays.rs               ✅ DONE (275 lines, TDG: 93.5/100 A)
├── binary_operators.rs     ✅ DONE (393 lines, TDG: 97.2/100 A+) ⭐
├── control_flow.rs         ✅ DONE (177 lines, TDG: N/A)
├── dataframes.rs           ✅ DONE (178 lines, TDG: 93.0/100 A)
├── identifiers.rs          ✅ DONE (401 lines, TDG: 82.9/100 B+)
├── literals.rs             ✅ DONE (206 lines, TDG: 92.9/100 A)
├── string_operations.rs    ✅ DONE (317 lines, TDG: 91.4/100 A)
├── tuples.rs               ✅ DONE (225 lines, TDG: 93.1/100 A)
├── unary_operators.rs      ✅ DONE (286 lines, TDG: 91.3/100 A)
├── visibility_modifiers.rs ✅ DONE (558 lines, TDG: 91.1/100 A)
├── objects.rs              ⏳ PLANNED (500 lines)
├── patterns.rs             ⏳ PLANNED (400 lines)
├── functions.rs            ⏳ PLANNED (400 lines)
├── loops.rs                ⏳ PLANNED (300 lines)
├── types.rs                ⏳ PLANNED (400 lines)
├── async_await.rs          ⏳ PLANNED (200 lines)
├── modules.rs              ⏳ PLANNED (300 lines)
└── mod.rs                  ✅ UPDATED
```

### EXTREME TDD Process (Mandatory)
1. **RED**: Write property tests FIRST (using proptest)
2. **GREEN**: Extract module, ensure all tests pass
3. **REFACTOR**: Apply PMAT quality gates (≤10 complexity, A- grade)
4. **MUTATION**: Run cargo-mutants (≥75% coverage)
5. **COMMIT**: Document with metrics

### Property Test Template
```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        #[ignore] // Run with: cargo test property_tests -- --ignored
        fn prop_module_never_panics(input: ArbitraryInput) {
            let _ = module_function(input); // Should not panic
        }

        #[test]
        #[ignore]
        fn prop_module_invariant_holds(a: Type, b: Type) {
            let result = module_function(a, b);
            prop_assert!(invariant_check(result));
        }
    }
}
```

---

## 📈 Progress Tracking

### Completion Checklist

**Phase 1-2**: ✅ COMPLETE
- [x] control_flow extracted
- [x] visibility_modifiers extracted (A grade)

**Tier 1 (High Impact)**:
- [x] Phase 3: Literals (206 lines, TDG 92.9/100 A) ✅
- [x] Phase 4: Identifiers (401 lines, TDG 82.9/100 B+) ✅
- [x] Phase 5: Arrays (275 lines, TDG 93.5/100 A) ✅
- [ ] Phase 6: Objects (~500 lines)
- [x] Phase 7: Tuples (225 lines, TDG 93.1/100 A) ✅
- [x] Phase 8: DataFrames (178 lines, TDG 93.0/100 A) ✅
- [x] Phase 9: Unary Ops (286 lines, TDG 91.3/100 A) ✅

**Tier 2 (Medium Impact)**:
- [ ] Phase 10: Patterns (~400 lines)
- [ ] Phase 11: Functions (~400 lines)
- [ ] Phase 12: Loops (~300 lines)
- [ ] Phase 13: Types (~400 lines)

**Tier 3 (Cleanup)**:
- [ ] Phase 14: Strings (~300 lines)
- [ ] Phase 15: Async (~200 lines)
- [ ] Phase 16: Modules (~300 lines)
- [ ] Phase 17: Misc (~200 lines)

### TDG Score Progression (Target)
```
Current:  71.2/100 (B-)  [0% extracted]
Sprint 1: 74.0/100 (B)   [20% extracted]
Sprint 2: 78.0/100 (B+)  [40% extracted]
Sprint 3: 82.0/100 (B+)  [53% extracted]
Sprint 4: 86.0/100 (A-)  [66% extracted] ← TARGET MET
Sprint 5: 91.0/100 (A)   [80% extracted] ← STRETCH GOAL
```

---

## 🚧 Known Challenges & Mitigations

### Challenge 1: Interdependencies
**Problem**: Functions call each other across modules
**Mitigation**: Use `pub(in crate::frontend::parser)` visibility for cross-module calls

### Challenge 2: Shared Helper Functions
**Problem**: Utility functions used by multiple modules
**Mitigation**: Create `helpers.rs` module for shared utilities

### Challenge 3: Large Extraction Effort
**Problem**: 50+ hours of work across 15 modules
**Mitigation**: Incremental sprints, commit after each module

### Challenge 4: Test Migration
**Problem**: Existing tests reference old function locations
**Mitigation**: Tests at module level auto-migrate with functions

### Challenge 5: Maintaining Compatibility
**Problem**: Risk of breaking existing functionality
**Mitigation**: Full test suite run after each extraction

---

## 📚 References

- **CLAUDE.md**: EXTREME TDD protocol, quality standards
- **PMAT Documentation**: TDG scoring, complexity analysis
- **Existing Modules**: control_flow.rs, visibility_modifiers.rs (templates)
- **Property Testing**: proptest crate documentation

---

## 🎓 Lessons Learned (From Phases 1-2)

1. **Small modules (<200 lines) have minimal TDG impact**
   - Need 400+ lines for measurable improvement

2. **Property tests provide superior coverage**
   - 6 property tests > dozens of unit tests
   - Catch edge cases unit tests miss

3. **A-grade modules are achievable**
   - visibility_modifiers.rs: 91.1/100
   - Proves TDG ≥85 is realistic target

4. **EXTREME TDD prevents regressions**
   - RED→GREEN→REFACTOR cycle catches issues early
   - All 10 unit tests + 6 property tests passing

5. **Incremental approach is sustainable**
   - 1 module per session prevents burnout
   - Each module is independently valuable

---

## 🔮 Future Enhancements

Post-modularization improvements:
1. **Mutation Testing**: Run cargo-mutants on each module
2. **Fuzz Testing**: Add AFL/cargo-fuzz for edge case discovery
3. **Benchmark Suite**: Performance regression testing
4. **Documentation**: Comprehensive rustdoc with examples
5. **Integration Tests**: End-to-end parsing scenarios

---

**Last Updated**: 2025-10-18
**Next Review**: After Sprint 1 completion
**Owner**: Modularization team
**Status**: 5% complete, on track for 5-sprint completion
