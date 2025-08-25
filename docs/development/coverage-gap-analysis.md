# Coverage Gap Analysis - Transpiler Module

## Current Status: 54.85% Coverage (Target: 70%)

### Module Breakdown

| Module | Coverage | Lines | Gap to Good | Priority |
|--------|----------|-------|-------------|----------|
| actors.rs | 80.00% | 135 | ✅ Achieved | Low |
| mod.rs | 65.26% | 285 | 4.74% | Medium |
| dataframe.rs | 57.58% | 165 | 12.42% | Medium |
| expressions.rs | 54.40% | 511 | 15.60% | High |
| statements.rs | 50.19% | 1070 | 19.81% | Critical |
| dispatcher.rs | 40.25% | 708 | 29.75% | Critical |
| types.rs | 36.36% | 495 | 33.64% | High |
| codegen_minimal.rs | 34.78% | 437 | 35.22% | High |
| patterns.rs | 14.14% | 198 | 55.86% | Critical |
| result_type.rs | 12.39% | 113 | 57.61% | Critical |
| dataframe_helpers.rs | 0.00% | 9 | 70.00% | Low (only 9 lines) |

### Critical Gaps (Highest Impact)

#### 1. patterns.rs (14.14% → 70%)
- **Lines to cover**: 111 of 198
- **Key uncovered areas**:
  - Complex pattern matching (struct patterns, guards)
  - Nested patterns
  - Or patterns
  - Range patterns
- **Strategy**: Property-based tests with all pattern variants

#### 2. result_type.rs (12.39% → 70%)
- **Lines to cover**: 65 of 113
- **Key uncovered areas**:
  - Result match transpilation
  - Result chaining with ?
  - Result unwrap_or variants
  - Result mapping functions
- **Strategy**: Integration tests for error handling paths

#### 3. statements.rs (50.19% → 70%)
- **Lines to cover**: 212 of 1070
- **Key uncovered areas**:
  - Complex statement blocks
  - Loop statements (for, while)
  - Break/continue with labels
  - Import/export statements
- **Strategy**: Full program transpilation tests

#### 4. dispatcher.rs (40.25% → 70%)
- **Lines to cover**: 211 of 708
- **Key uncovered areas**:
  - Expression dispatch paths
  - Type dispatch logic
  - Pattern dispatch
  - Error handling paths
- **Strategy**: Integration tests covering all AST node types

### Path to 70% Coverage

#### Quick Wins (High Impact, Low Effort)
1. **patterns.rs**: Add 10 comprehensive pattern tests (+111 lines) → +3.5%
2. **result_type.rs**: Add 5 Result handling tests (+65 lines) → +2.0%
3. **types.rs**: Add struct/enum/trait tests (+166 lines) → +5.2%

**Total Quick Wins**: +10.7% → **65.55% coverage**

#### Medium Effort (Requires Integration Tests)
1. **statements.rs**: Add 15 statement tests (+212 lines) → +6.6%
2. **dispatcher.rs**: Add dispatch integration tests (+211 lines) → +6.6%

**Total with Medium**: +13.2% → **78.75% coverage**

### Recommended Test Strategy

#### Phase 1: Quick Wins (1 day)
```rust
// patterns.rs tests needed:
- test_match_all_pattern_types()
- test_nested_patterns()
- test_pattern_guards()
- test_or_patterns()
- test_range_patterns()

// result_type.rs tests needed:
- test_result_matching()
- test_result_chaining()
- test_error_propagation()
- test_result_combinators()

// types.rs tests needed:
- test_struct_transpilation()
- test_enum_transpilation()
- test_trait_transpilation()
- test_generic_types()
```

#### Phase 2: Integration Tests (1-2 days)
```rust
// Full program tests:
- test_complete_program_transpilation()
- test_all_statement_types()
- test_all_expression_types()
- test_error_handling_programs()
- test_async_programs()
```

### Test Infrastructure Needed

1. **Test Data Generator**: Create AST builders for complex scenarios
2. **Golden Tests**: Store expected Rust output for comparison
3. **Property Tests**: Use proptest for invariant checking
4. **Fuzzing**: Target parser → transpiler pipeline

### Coverage Tracking

```bash
# Quick coverage check
cargo llvm-cov report --ignore-filename-regex "tests/" | grep backend/transpiler

# Detailed HTML report
cargo llvm-cov --html --open --ignore-filename-regex "tests/"

# Module-specific coverage
cargo llvm-cov show src/backend/transpiler/patterns.rs --ignore-filename-regex "tests/"
```

### Conclusion

Reaching 70% transpiler coverage is achievable with focused effort on the lowest-coverage modules. The critical path involves:

1. **Immediate**: Add tests to patterns.rs and result_type.rs (gain ~5.5%)
2. **Short-term**: Expand types.rs tests (gain ~5.2%)
3. **Medium-term**: Integration tests for statements and dispatcher (gain ~13.2%)

This would achieve **~78% coverage**, exceeding the 70% target.

**Estimated Time**: 2-3 days of focused testing
**Risk**: Some code paths may be unreachable or dead code
**Mitigation**: Remove dead code or mark as unreachable!()