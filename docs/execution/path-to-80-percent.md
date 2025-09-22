# Path to 80% Test Coverage

## Current Status
- **Current Coverage**: 63.6% (119/187 tests passing)
- **Target Coverage**: 80% (150/187 tests)
- **Tests Needed**: 31 more passing tests

## Quick Wins Available (No Parser Changes)
1. **Import Test Fixes** (2 tests in transpiler_statements)
   - Fix assertions for `transpile_import_wildcard`
   - Fix assertions for `transpile_import_aliased`

2. **Quality Attribute Tests** (Estimate: 5-10 tests)
   - Some may just need assertion adjustments
   - Check which attributes are partially supported

3. **Fun Keyword Tests** (Estimate: 3-5 tests)
   - Some variations might work with minor fixes

## Parser Work Required (Blocked Tests)

### Set Comprehensions (15 tests blocked)
```rust
// Not parsed: {x for x in items}
// Needs: Distinguish set comprehension from object literal
```

### Dict Comprehensions (10 tests blocked)
```rust
// Not parsed: {k: v for (k, v) in pairs}
// Needs: New comprehension syntax in parser
```

### DataFrame Literals (25 tests blocked)
```rust
// Not parsed: df![...]
// Needs: Custom macro parsing support
```

### Advanced Keywords (10 tests blocked)
```rust
// Not parsed: const fun, unsafe fun, pub use
// Needs: Keyword combinations in parser
```

## Implementation Priority

### Phase 1: Quick Wins (5-10 tests)
- Fix import test assertions
- Review quality attribute tests for easy fixes
- Check fun keyword variations

### Phase 2: Set/Dict Comprehensions (25 tests)
- Add parser support for `{...}` comprehension syntax
- Distinguish from object literals by `for` keyword
- Generate appropriate HashSet/HashMap code

### Phase 3: DataFrame Support (25 tests)
- Implement `df!` macro parsing
- Add DataFrame literal AST node
- Generate Polars DataFrame construction code

### Phase 4: Keyword Combinations (10 tests)
- Support `const fun`, `unsafe fun`
- Fix `use X as Y` aliasing (conflicts with cast)
- Add `pub use` re-export syntax

## Estimated Timeline
- Quick wins: 1-2 hours (5-10 tests)
- Set/Dict comprehensions: 4-6 hours (25 tests)
- DataFrame literals: 3-4 hours (25 tests)
- Keyword combinations: 2-3 hours (10 tests)

**Total estimate**: 10-15 hours to reach 80% coverage

## Alternative: Focus on Working Features
Instead of implementing new parser features, we could:
1. Write more tests for working features
2. Add integration tests for existing functionality
3. Create end-to-end test scenarios
4. Add property-based tests for parser/transpiler

This would increase coverage without blocked tests.