# PARSER-009: Implement Impl Block Parsing

**Status**: Open
**Priority**: High (BLOCKER for Issue #147)
**Complexity**: Medium (8-10 functions, module refactoring)
**Estimated Time**: 4-6 hours
**Created**: 2025-11-11

## Executive Summary

Implement full impl block parsing support to unblock Issue #147. The transpiler is already correct - only parser implementation is needed.

## Root Cause Analysis (Five Whys)

**Problem**: Users cannot use `impl` blocks, getting "impl blocks are not supported" error

1. **Why?** Parser bails immediately at `src/frontend/parser/expressions_helpers/impls.rs:45`
2. **Why?** Implementation was stubbed out during initial development
3. **Why?** Ruchy originally preferred methods-in-struct syntax
4. **Why?** Simpler to parse and transpile initially
5. **Why?** Incremental development - impl blocks deferred

**Root Cause**: Parser stub was never completed, though transpiler support exists

## Current State

### ✅ Working (Transpiler)
- `src/backend/transpiler/types.rs:937-1032` - Full impl block transpilation
- Correctly generates `pub fn` (NOT `pub pub fn` as reported in Issue #147)
- Supports:
  - Type implementations: `impl TypeName { methods }`
  - Trait implementations: `impl Trait for Type { methods }`
  - Generic implementations: `impl<T> Trait for Type<T> { methods }`
  - pub/private method visibility

### ❌ Broken (Parser)
- `src/frontend/parser/expressions_helpers/impls.rs:42-56` - Hard-coded `bail!()`
- No actual parsing logic
- Module visibility issues for helper functions

### ✅ Tests Ready (RED Step Complete)
- `tests/transpiler_147_impl_blocks.rs` - 4 failing tests (0/4 passing)
- Tests cover:
  1. Basic impl with pub fun
  2. Multiple pub methods
  3. Mixed pub/private visibility
  4. Self receivers (&self, &mut self)

## Implementation Plan (EXTREME TDD)

### Phase 1: Module Refactoring (Foundation)
**Goal**: Fix import visibility issues

**Files to Modify**:
1. `src/frontend/parser/mod.rs`
   - Declare `mod utils_helpers;` (currently missing)
   - Re-export needed functions

2. `src/frontend/parser/utils_helpers/mod.rs`
   - Already exists with correct sub-modules
   - Ensure `pub use` exports for external access

**Validation**: `cargo build --lib` compiles without errors

### Phase 2: Parser Implementation (GREEN Step)
**Goal**: Make 4/4 tests pass

**File**: `src/frontend/parser/expressions_helpers/impls.rs`

**Functions to Implement** (Complexity ≤10):

1. **`parse_impl_block()`** - Main entry (complexity: 8)
   - Parse `impl` keyword
   - Optional type parameters: `impl<T, U>`
   - Trait name + for_type OR just for_type
   - Methods block `{ methods }`
   - Return `ExprKind::Impl`

2. **`expect_identifier()`** - Helper (complexity: 2)
   - Match `Token::Identifier(name)`
   - Clone and advance
   - Error if not identifier

3. **`parse_impl_method()`** - Method parser (complexity: 6)
   - Optional `pub` keyword
   - `fun` keyword
   - Method name (identifier)
   - Parameters `(params)`
   - Optional return type `-> Type`
   - Body block `{ body }`
   - Return `ImplMethod`

**Dependencies**:
- `parse_type_parameters()` from `utils_helpers::types`
- `parse_params()` from `utils_helpers::params`
- `parse_type()` from `utils_helpers::types`
- `parse_block()` from `collections`

**Token Matching**:
- Use `matches!(state.tokens.peek(), Some((Token::X, _)))`
- Use `state.tokens.expect(&Token::X)?` for consumption
- Check Token enum for correct names (NOT `Token::Lt`, find actual name)

### Phase 3: REFACTOR
**Goal**: Apply quality gates

**Quality Checks**:
1. ✅ Complexity ≤10 per function
2. ✅ Zero clippy warnings
3. ✅ Zero SATD comments
4. ✅ All functions documented
5. ✅ `pmat tdg` ≥ A- grade

### Phase 4: VALIDATE (End-to-End)
**Goal**: Prove it works

**Validation Steps**:
1. **Unit Tests**: `cargo test --test transpiler_147_impl_blocks` (4/4 passing)
2. **Rustc Compilation**: Generated code compiles to .rlib
3. **Manual Test**: Create impl block, transpile, verify output
4. **Integration**: Test with existing codebase examples
5. **Book Validation**: `make validate-book` still passes

## Test Metrics (Success Criteria)

- **Unit Tests**: 4/4 passing (currently 0/4)
- **Test Execution**: <5 seconds
- **Zero Regressions**: All existing tests still pass
- **Rustc Validation**: All 4 generated .rs files compile
- **Coverage**: Parser module coverage increases by ~5%

## Files Modified

### Parser
- `src/frontend/parser/mod.rs` - Add `mod utils_helpers;`
- `src/frontend/parser/expressions_helpers/impls.rs` - Full implementation (~140 lines)

### Tests
- `tests/transpiler_147_impl_blocks.rs` - Already created (4 tests, 194 lines)

### Documentation
- `docs/tickets/PARSER-009-impl-blocks.md` - This ticket
- `CHANGELOG.md` - Add PARSER-009 entry

### Roadmap
- `docs/execution/roadmap.yaml` - Add PARSER-009 task

## Risks and Mitigations

### Risk 1: Token Enum Unknown Names
**Impact**: Compilation failures
**Likelihood**: High
**Mitigation**: Grep Token enum first, verify < and > token names

### Risk 2: Module Visibility Issues
**Impact**: Import errors
**Likelihood**: High (already encountered)
**Mitigation**: Follow existing patterns from `actors.rs` and `structs.rs`

### Risk 3: Parser State Management
**Impact**: Incorrect AST construction
**Likelihood**: Medium
**Mitigation**: Copy patterns from working parsers (actors, structs, traits)

### Risk 4: Test Failures After Implementation
**Impact**: Need additional fixes
**Likelihood**: Medium
**Mitigation**: TDD approach ensures tests guide implementation

## Dependencies

**Blocks**:
- None (transpiler already works)

**Blocked By**:
- None

**Enables**:
- Issue #147 resolution
- User adoption of impl blocks
- Closer Rust syntax parity

## Success Metrics

1. ✅ All 4 tests pass: `cargo test --test transpiler_147_impl_blocks`
2. ✅ No regressions: `cargo test` (all existing tests pass)
3. ✅ Quality gates: `pmat tdg . --min-grade A-`
4. ✅ Zero clippy warnings: `make lint`
5. ✅ Rustc validation: All 4 .rlib files compile
6. ✅ Issue #147 closed

## References

- **Issue #147**: https://github.com/paiml/ruchy/issues/147
- **Transpiler Code**: `src/backend/transpiler/types.rs:937-1032`
- **Parser Stub**: `src/frontend/parser/expressions_helpers/impls.rs:42-56`
- **Test Suite**: `tests/transpiler_147_impl_blocks.rs`
- **AST Definition**: `src/frontend/ast.rs:587-593` (`ExprKind::Impl`)

## Timeline Estimate

- **Phase 1** (Module Refactoring): 1 hour
- **Phase 2** (Parser Implementation): 2-3 hours
- **Phase 3** (REFACTOR): 1 hour
- **Phase 4** (VALIDATE): 1 hour

**Total**: 4-6 hours (single focused session)

## Notes

- Transpiler is CORRECT - verified no "pub pub fn" bug
- Issue #147 is a phantom bug (parser rejection, not transpiler)
- Workaround exists: methods-in-struct syntax (Ruchy preferred style)
- Implementation is straightforward - follow existing parser patterns
