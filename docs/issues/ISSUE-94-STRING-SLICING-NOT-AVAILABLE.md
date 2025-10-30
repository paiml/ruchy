# ISSUE-94: String Slicing Not Available

**Created**: 2025-10-30
**Status**: 🔄 IN PROGRESS
**GitHub Issue**: #94
**Priority**: HIGH (blocks common string manipulation patterns)

## Problem Statement

String slicing with range syntax (`text[0..5]`) is not supported. Current error:
```
Error: Evaluation error: Runtime error: Cannot index string with range
```

## Expected Behavior

String slicing should work like Rust:
```ruchy
let text = "Hello, World!";
let hello = text[0..5];    // "Hello"
let world = text[7..12];   // "World"
let start = text[..5];     // "Hello" (from beginning)
let end = text[7..];       // "World!" (to end)
```

## Root Cause Analysis (Five Whys)

1. **Why does string slicing fail?**
   → Runtime evaluator doesn't handle range indexing for strings

2. **Why doesn't runtime handle range indexing?**
   → `eval_index` function only handles integer indices for strings

3. **Why only integer indices?**
   → String slicing feature was never implemented

4. **Why wasn't it implemented?**
   → Not prioritized in initial language feature set

5. **Root Cause**: Missing feature implementation - needs range handling in `eval_index`

## Impact

- **Severity**: HIGH
- **User Impact**: Blocks common text processing and parsing patterns
- **Workaround**: Manual character iteration or substring methods
- **Affected Files**: src/runtime/eval.rs (eval_index function)

## Implementation Plan (EXTREME TDD)

### Phase 1: RED - Write Failing Tests
- [ ] Basic slicing: `text[start..end]`
- [ ] Open ranges: `text[..end]`, `text[start..]`
- [ ] Full range: `text[..]`
- [ ] Empty slices: `text[5..5]`
- [ ] Out of bounds (should error gracefully)
- [ ] UTF-8 character boundaries

### Phase 2: GREEN - Minimal Implementation
- [ ] Add range pattern matching in `eval_index`
- [ ] Implement string slicing for `Range` values
- [ ] Handle open-ended ranges (`RangeFrom`, `RangeTo`)
- [ ] Add bounds checking with clear error messages

### Phase 3: REFACTOR - Quality & Edge Cases
- [ ] Property tests: `slice.len() == end - start`
- [ ] Fuzz testing for UTF-8 edge cases
- [ ] PMAT complexity check (≤10)
- [ ] Documentation with examples

## Test Coverage Requirements

**Unit Tests** (tests/issue_094_string_slicing.rs):
- 6 basic slicing tests
- 3 edge case tests (empty, out of bounds, UTF-8)

**Property Tests**:
- Length invariant: `text[a..b].len() == b - a`
- Concatenation: `text[0..5] + text[5..]` == `text`
- Bounds: All valid slices produce valid strings

**Integration Tests**:
- Ruchy book examples using string slicing
- REPL interaction

## Files to Modify

1. **src/runtime/eval.rs**
   - Function: `eval_index` (add range handling)
   - Complexity target: ≤10

2. **tests/issue_094_string_slicing.rs** (NEW)
   - Comprehensive test suite

3. **CHANGELOG.md**
   - Document new feature

4. **docs/execution/roadmap.yaml**
   - Add session summary

## Toyota Way Principles

- **Stop the Line**: Discovered missing feature → immediately create issue and implement
- **Genchi Genbutsu**: Verified actual behavior with test case
- **Kaizen**: Incremental improvement (one feature at a time)
- **Jidoka**: Automated testing prevents regression

## References

- GitHub Issue: https://github.com/paiml/ruchy/issues/94
- Rust String Slicing: https://doc.rust-lang.org/book/ch08-02-strings.html#slicing-strings
- Related: Indexing already works for single characters via `.chars()` methods

## Timeline

- **Created**: 2025-10-30
- **Target Completion**: Same day (EXTREME TDD - 1-2 hours)
- **Actual Completion**: TBD

## Notes

- Implementation must handle UTF-8 properly (byte boundaries vs character boundaries)
- Consider both byte slicing (`&text[0..5]` in Rust) vs character slicing
- Ruchy should likely use character-based slicing for consistency with `.len()` method
