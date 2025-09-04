# Backend Transpiler Refactoring Plan

## Critical Issue: statements.rs is 2,694 lines with 46 public functions

### MANDATORY: Split into 5 modules with complexity ≤10 per function

## Proposed Module Structure

### 1. `statements/control_flow.rs` (~500 lines)
**Functions to move**:
- `transpile_if` (complexity: reduce from ~15 to ≤10)
- `transpile_while`
- `transpile_for` 
- `transpile_loop`
- `transpile_if_let`
- `transpile_while_let`
- `transpile_break`
- `transpile_continue`

### 2. `statements/variables.rs` (~400 lines)
**Functions to move**:
- `transpile_let` (complexity: reduce from ~20 to ≤10)
- `transpile_let_pattern`
- `transpile_const`
- `transpile_assignment`
- Variable mutability analysis helpers

### 3. `statements/functions.rs` (~500 lines)
**Functions to move**:
- `transpile_function` (complexity: reduce from ~30 to ≤10)
- `transpile_lambda`
- `transpile_call`
- `transpile_method_call`
- Parameter type inference helpers

### 4. `statements/modules.rs` (~400 lines)
**Functions to move**:
- `transpile_module`
- `transpile_import`
- `transpile_import_inline`
- `transpile_export`
- `transpile_use`

### 5. `statements/blocks.rs` (~300 lines)
**Functions to move**:
- `transpile_block`
- `transpile_pipeline`
- `transpile_list_comprehension`
- Expression sequencing helpers

## Complexity Reduction Strategy

### Current High-Complexity Functions

1. **`transpile_function`** (complexity: ~30)
   - Extract: parameter processing (≤5)
   - Extract: return type inference (≤5)
   - Extract: body transpilation (≤5)
   - Extract: async handling (≤5)
   - Main dispatcher: ≤10

2. **`transpile_let`** (complexity: ~20)
   - Extract: pattern matching (≤5)
   - Extract: type annotation (≤5)
   - Extract: mutability detection (≤5)
   - Main dispatcher: ≤5

3. **`transpile_if`** (complexity: ~15)
   - Extract: condition processing (≤5)
   - Extract: branch handling (≤5)
   - Main dispatcher: ≤5

## TDD Implementation Plan

### Phase 1: Create Module Structure
```rust
// src/backend/transpiler/statements/mod.rs
pub mod control_flow;
pub mod variables;
pub mod functions;
pub mod modules;
pub mod blocks;

// Re-export for compatibility
pub use control_flow::*;
pub use variables::*;
pub use functions::*;
pub use modules::*;
pub use blocks::*;
```

### Phase 2: Move Functions (RED → GREEN)
1. Create failing tests for each module
2. Move functions to new modules
3. Ensure all tests pass
4. Verify complexity ≤10

### Phase 3: Refactor High Complexity (REFACTOR)
1. Identify functions >10 complexity
2. Extract helper functions
3. Re-run tests to ensure no regression
4. Measure coverage improvement

## Success Metrics
- [ ] No function exceeds complexity 10
- [ ] Backend coverage increases from 52.9% to 70%+
- [ ] All existing tests continue to pass
- [ ] Each new module has 80%+ coverage

## Estimated Impact
- **Lines per module**: ~300-500 (manageable)
- **Functions per module**: 8-10 (focused)
- **Complexity per function**: ≤10 (testable)
- **Coverage improvement**: +20-30%