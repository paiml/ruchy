# REPL Refactoring Plan - Achieving 80% Coverage with Low Complexity

## Current State
- **File**: `src/runtime/repl.rs` 
- **Size**: 10,874 lines (MASSIVE)
- **Functions**: 392 functions in ONE file
- **Cyclomatic Complexity**: 4,932 (should be <500)
- **Cognitive Complexity**: 6,302 (should be <750)
- **Current Coverage**: 35% (with 554 tests)
- **Target**: 80% coverage with complexity <15 per function

## Root Cause Analysis (Toyota Way - 5 Whys)
1. Why is coverage low despite 554 tests? → File is too complex (10,874 lines)
2. Why is the file so complex? → 392 functions in one module
3. Why are all functions in one module? → No separation of concerns
4. Why no separation? → Organic growth without refactoring
5. Why no refactoring? → Focus on features over maintainability

## Refactoring Strategy

### Phase 1: Extract Major Components (IMMEDIATE)

Split `repl.rs` into focused modules:

```rust
// src/runtime/repl/mod.rs (orchestrator - <500 lines)
pub mod commands;      // Command handlers (~1,500 lines)
pub mod evaluation;    // Expression evaluation (~3,000 lines)  
pub mod bindings;      // Variable/binding management (~800 lines)
pub mod completion;    // Tab completion (~600 lines)
pub mod history;       // History management (~400 lines)
pub mod methods;       // Method evaluation (~2,000 lines)
pub mod operators;     // Operator evaluation (~1,000 lines)
pub mod debug;         // Debug/introspection (~800 lines)
pub mod state;         // State management (~500 lines)
pub mod errors;        // Error handling (~400 lines)
```

### Phase 2: Reduce Function Complexity

Target functions with complexity >15:

1. **evaluate_expr** (complexity: 138) → Split into:
   - `evaluate_literal`
   - `evaluate_control_flow`
   - `evaluate_data_structure`
   - `evaluate_function_call`
   - `evaluate_binding`

2. **evaluate_list_methods** (complexity: 55) → Extract each method:
   - `list_map`, `list_filter`, `list_reduce`, etc.

3. **evaluate_string_methods** (complexity: 48) → Extract each method:
   - `string_upper`, `string_split`, `string_replace`, etc.

### Phase 3: Apply SOLID Principles

1. **Single Responsibility**: Each module handles ONE aspect
2. **Open/Closed**: Method handlers use trait dispatch
3. **Liskov Substitution**: Common evaluation interface
4. **Interface Segregation**: Narrow interfaces for each component
5. **Dependency Inversion**: Core REPL depends on abstractions

### Implementation Plan

#### Step 1: Create module structure
```bash
mkdir -p src/runtime/repl
touch src/runtime/repl/{mod,commands,evaluation,bindings,completion,history,methods,operators,debug,state,errors}.rs
```

#### Step 2: Move code systematically
- Start with low-dependency code (commands, history)
- Move evaluation functions next
- Keep tests passing at each step

#### Step 3: Reduce complexity function by function
- Target: No function >15 cognitive complexity
- Use helper functions and early returns
- Extract complex conditions to named functions

### Expected Outcomes

After refactoring:
- **Main REPL module**: <500 lines, complexity <100
- **Each submodule**: <1,500 lines, complexity <200
- **Each function**: <50 lines, complexity <15
- **Total complexity**: <1,500 (from 6,302)
- **Testability**: 80% coverage achievable with existing tests
- **Maintainability**: A+ PMAT grade

### Test Strategy After Refactoring

With smaller modules:
- Unit tests per module (easier to achieve 80%)
- Integration tests for REPL orchestration
- Existing 554 tests provide foundation
- Need ~200 more targeted tests post-refactor

### PMAT Compliance

Post-refactor targets:
- **Cyclomatic Complexity**: <10 per function (from 138)
- **Cognitive Complexity**: <15 per function (from 200+)
- **File Size**: <1,500 lines per module (from 10,874)
- **Documentation**: >70% coverage with doctests
- **TDG Grade**: A+ for each module

## Immediate Action Items

1. **STOP** adding more tests to monolithic REPL
2. **REFACTOR** into modules following this plan
3. **TEST** each module to 80% individually
4. **VERIFY** PMAT compliance for each module
5. **RELEASE** v1.53.0 with refactored REPL

## Toyota Way Implementation

- **Jidoka**: Build quality into each module
- **Poka-yoke**: Type system prevents module coupling
- **Kaizen**: Incremental refactoring with tests
- **Genchi Genbutsu**: Measure complexity at source
- **Respect**: Make code maintainable for future developers

## Success Metrics

- [ ] REPL split into 10+ modules
- [ ] No function >15 complexity
- [ ] Each module >80% coverage
- [ ] PMAT grade A+ for each module
- [ ] Total test count: ~750 (existing 554 + 200 new)
- [ ] Build time: <30 seconds
- [ ] Test time: <5 seconds

This refactoring is **MANDATORY** to achieve both 80% coverage AND low complexity as requested.