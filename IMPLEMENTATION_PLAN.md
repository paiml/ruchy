# Ruchy Implementation Plan - Next Steps

## Current State (2025-09-25)

### Coverage Achievement
- **Overall Coverage**: 75.88% (up from 33.34%)
- **Lines**: 75.88%
- **Functions**: 79.22%
- **Regions**: 75.38%

### Test Status
- **Total Tests**: 3,372 passing, 64 failing
- **UNIFIED SPEC**: 59/121 tests passing (48.8%)

## Immediate Priorities

### 1. Complete UNIFIED SPEC Implementation (62 tests remaining)

#### 1.1 Fun Keyword Modifiers (10 tests)
**Issue**: Parser doesn't recognize `const`, `unsafe` modifiers before `fun`
**Solution**:
- Add `Token::Const` and `Token::Unsafe` to prefix parser
- Create parse_const_token() and parse_unsafe_token() handlers
- Forward to parse_function() with appropriate flags

#### 1.2 Use Imports Enhancement (4 tests)
**Status**: 6/10 tests passing
**Failing Tests**:
- test_use_crate
- test_pub_use
- test_use_grouped
- test_use_self

**Solution**: Enhance parser to handle:
- `use crate::module`
- `pub use` statements
- Grouped imports: `use std::{vec, collections}`
- `use self::` patterns

#### 1.3 List Comprehensions (15 tests failing)
**Not Implemented**: Parser doesn't recognize comprehension syntax
**Required Syntax**: `[expr for var in iter if condition]`
**Solution**:
- Add comprehension parsing to parse_list()
- Create ExprKind::Comprehension variant
- Transpile to iterator chains

#### 1.4 DataFrame Operations (20+ tests)
**Partially Working**: Basic DataFrame support exists
**Missing**: Method chaining, SQL macro
**Solution**:
- Enhance DataFrame method parsing
- Add SQL macro support
- Implement filter/groupby/agg chains

## Technical Debt to Address

### High Priority
1. **Complexity Violations**:
   - Value::fmt: 66 (target <30)
   - Value::format_dataframe: 69 (target <30)
   - evaluate_expr: 138 (target <50)

2. **Parser Completeness**:
   - Character literals ✅
   - Tuple destructuring ✅
   - Rest patterns ✅
   - Comprehensions ❌
   - Const/unsafe modifiers ❌

### Medium Priority
1. **Error Messages**: Improve parser error recovery
2. **Test Organization**: Consolidate duplicate test files
3. **Documentation**: Update SPECIFICATION.md with implemented features

## Implementation Strategy

### Phase 1: Parser Enhancements (1-2 days)
1. Add const/unsafe modifier support
2. Fix grouped use imports
3. Implement basic comprehension parsing

### Phase 2: Transpiler Updates (2-3 days)
1. Generate correct Rust code for comprehensions
2. Fix DataFrame method chaining
3. Handle const/unsafe function generation

### Phase 3: Quality & Testing (1 day)
1. Run full test suite
2. Fix remaining failures
3. Update documentation

## Success Metrics
- [ ] UNIFIED SPEC: 100% tests passing (121/121)
- [ ] Overall coverage: >80%
- [ ] All functions: Complexity ≤10
- [ ] Zero SATD comments
- [ ] Clean clippy output

## Risk Mitigation
1. **Comprehensions**: Complex feature - implement incrementally
2. **DataFrame**: May need Arrow integration fixes
3. **Modifiers**: Ensure transpiler generates valid Rust

## Next Session Recommendations
1. Start with const/unsafe parser support (quick win)
2. Fix grouped use imports (moderate complexity)
3. Tackle comprehensions if time permits (high complexity)
4. Leave DataFrame for dedicated sprint

## Long-term Roadmap
1. **Q4 2025**: Complete language specification
2. **Q1 2026**: WASM compilation
3. **Q2 2026**: LSP implementation
4. **Q3 2026**: Production readiness

## Notes
- Maintain Toyota Way principles
- Every change must have tests
- Complexity ≤10 for all new functions
- Use property testing for parser reliability
- Document all design decisions