# GitHub Issues - ruchy-book Compatibility (2025-10-21)

**Filed**: October 21, 2025
**Context**: Book compatibility testing revealed 32% failure rate (127/359 examples failing)
**Source**: E2E testing of actual ruchy-book chapter content
**Status**: Issues filed upstream for tracking

---

## Critical Issues

### Issue #45: Multi-line Code Blocks with Inline Comments (CRITICAL)

**URL**: https://github.com/paiml/ruchy/issues/45
**Severity**: CRITICAL - Blocks major compatibility improvement
**Impact**: 200+ broken examples
**Ruchy Ticket**: PARSER-053
**Sprint Plan**: `docs/execution/PARSER-053-SPRINT-PLAN.md`

#### Description
Parser fails when comments appear between continued lines in multi-line expressions, method chains, or function calls.

#### Reproduction
```ruchy
let result = some_function()
    # This comment breaks parsing
    .method_call()
    .another_method()
```

**Error**: Parser fails to handle comment tokens in expression continuations.

#### Root Cause
- Parser doesn't skip comment tokens when parsing multi-line expressions
- Tokenizer includes comments in token stream, breaking operator precedence
- No test coverage for comments in expression continuations

#### Expected Fix
- Modify parser to transparently skip comment tokens
- Maintain correct AST structure despite inline comments
- Add comprehensive test coverage (unit + property + mutation)

#### Success Metrics
- Before: 65% book compatibility (233/359)
- After: ≥90% book compatibility (324+/359)
- Improvement: +25% compatibility, +91 examples working

#### Implementation Timeline
- Estimated: 6-9 hours (1 focused workday)
- Complexity: Medium (parser modification + extensive testing)
- Risk: Low (well-scoped change with clear test cases)

---

## Medium Priority Issues

### Issue #46: Negative Array Indexing Not Supported

**URL**: https://github.com/paiml/ruchy/issues/46
**Severity**: MEDIUM - Nice-to-have feature
**Impact**: ~5 broken examples
**Ruchy Ticket**: FEATURE-042

#### Description
Python-style negative array indexing (`arr[-1]` for last element) is not implemented.

#### Current Behavior
```ruchy
let arr = [1, 2, 3, 4, 5]
let last = arr[-1]  # Error: Index out of bounds
```

#### Expected Behavior
```ruchy
let arr = [1, 2, 3, 4, 5]
let last = arr[-1]  # Returns: 5
let second_last = arr[-2]  # Returns: 4
```

#### Implementation Notes
- Modify array indexing evaluation to handle negative indices
- Formula: `negative_index → length + index`
- Edge case: `arr[-6]` on 5-element array should error
- Add bounds checking for negative indices

#### Testing Requirements
- Unit tests: Negative indices on arrays of different sizes
- Property tests: `arr[-i] == arr[len - i]` for valid indices
- Mutation tests: Verify bounds checking is tested

---

### Issue #47: Missing array.append() and string.format()

**URL**: https://github.com/paiml/ruchy/issues/47
**Severity**: MEDIUM - Completeness feature
**Impact**: ~5 broken examples
**Ruchy Ticket**: STDLIB-007

#### Description
Common array and string methods referenced in book examples are not implemented.

#### Missing Functions

**1. array.append(value)**
```ruchy
let mut arr = [1, 2, 3]
arr.append(4)  # Error: Unknown method 'append'
# Expected: arr == [1, 2, 3, 4]
```

**2. string.format(...args)**
```ruchy
let name = "Alice"
let age = 30
let message = "Name: {}, Age: {}".format(name, age)
# Error: Unknown method 'format'
# Expected: "Name: Alice, Age: 30"
```

#### Implementation Notes

**array.append()**
- Add to `src/runtime/methods.rs` or `src/stdlib/array.rs`
- Signature: `fn append(&mut self, value: T)`
- Behavior: Mutates array in-place, returns unit
- Alternative: Consider `push()` for Rust consistency

**string.format()**
- Add to `src/runtime/methods.rs` or `src/stdlib/string.rs`
- Signature: `fn format(&self, args: Vec<Value>) -> String`
- Behavior: Replace `{}` placeholders with args in order
- Alternative: Already have `f"..."` string interpolation (prefer that?)

#### Testing Requirements
- Unit tests: Basic append/format functionality
- Property tests:
  - `arr.append(x).len() == original_len + 1`
  - `s.format(args).len() >= s.len()`
- Mutation tests: ≥75% coverage

---

## Summary

| Issue # | Title | Impact | Priority | Ticket | Estimated |
|---------|-------|--------|----------|--------|-----------|
| #45 | Multi-line Comments | 200+ examples | CRITICAL | PARSER-053 | 6-9h |
| #47 | array.append() + string.format() | ~5 examples | MEDIUM | STDLIB-007 | 2-3h |
| #46 | Negative Array Indexing | ~5 examples | MEDIUM | FEATURE-042 | 3-4h |

**Total Impact**: 210+ broken examples (58% of failures)
**Total Effort**: 11-16 hours estimated

---

## Recommended Sprint Order

### Sprint 1: PARSER-053 (CRITICAL)
**Why First**: Unblocks 200+ examples, highest impact
**Risk**: Low (well-scoped parser change)
**Value**: +25% book compatibility

### Sprint 2: STDLIB-007 (Quick Win)
**Why Second**: Easy implementation, small impact
**Risk**: Very low (standard library additions)
**Value**: +1.4% book compatibility

### Sprint 3: FEATURE-042 (Enhancement)
**Why Last**: Nice-to-have feature, lowest impact
**Risk**: Low (well-defined behavior)
**Value**: +1.4% book compatibility

**Combined Impact**: +91 examples → 90%+ book compatibility achieved

---

## Quality Standards (MANDATORY)

All issues MUST follow EXTREME TDD + PMAT quality gates:

1. **Tests First**: Write failing tests before implementation
2. **Complexity**: All functions ≤10 cyclomatic complexity
3. **TDG Score**: A- minimum (≥85 points)
4. **Unit Tests**: 100% coverage of new code
5. **Property Tests**: 10K+ random inputs
6. **Mutation Tests**: ≥75% mutation coverage (BLOCKING)
7. **15-Tool Validation**: All 15 native tools must work

**NO EXCEPTIONS** to quality gates. Quality is non-negotiable.

---

## References

- **Sprint Plan**: `docs/execution/PARSER-053-SPRINT-PLAN.md`
- **Roadmap**: `docs/execution/roadmap.yaml` (version 3.13)
- **GitHub Repo**: https://github.com/paiml/ruchy
- **Book Repo**: https://github.com/paiml/ruchy-book

---

*This document tracks GitHub issues filed for ruchy-book compatibility improvements. All issues are real bugs discovered through E2E testing, not synthetic/theoretical problems.*
