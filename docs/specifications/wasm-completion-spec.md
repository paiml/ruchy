# WASM Backend Completion Specification

## CRITICAL: Root Cause Analysis

**Discovery Date**: 2025-10-06
**Discovered By**: 15-tool validation protocol (TOOL-VALIDATION-001,002)
**Impact**: WASM backend advertised but non-functional for real programs

### Five Whys Analysis

1. **Why did WASM validation fail?** - Type mismatch errors in generated WASM bytecode
2. **Why were there type mismatches?** - Built-in functions like println() not properly handled
3. **Why weren't built-ins handled?** - No import section, no host function bindings
4. **Why was this not caught earlier?** - Only tested with simple arithmetic, not real programs
5. **Root Cause**: WASM backend was developed incrementally without complete feature specification

### Genchi Genbutsu (Go and See)

**Current State**:
- ✅ WASM module structure generation
- ✅ Basic arithmetic operations
- ✅ Type inference for primitives
- ❌ Built-in function imports (println, print, etc.)
- ❌ Import section generation
- ❌ Host function bindings
- ❌ String memory management
- ❌ Control flow with void expressions

**Failing Examples**:
- All control flow examples (if, match, for, while) - contain println statements
- All function examples - contain println statements

## Specification: Complete WASM Built-in Support

### Requirements (MANDATORY/BLOCKING)

#### 1. Import Section for Host Functions
```wasm
(import "env" "println" (func $println (param i32)))
(import "env" "print" (func $print (param i32)))
```

#### 2. Built-in Function Registry
```rust
struct BuiltInFunction {
    name: &'static str,
    params: Vec<WasmType>,
    result: Option<WasmType>,
    import_module: &'static str,
    import_name: &'static str,
}

const BUILTINS: &[BuiltInFunction] = &[
    BuiltInFunction {
        name: "println",
        params: vec![WasmType::I32], // String pointer
        result: None, // Void
        import_module: "env",
        import_name: "println",
    },
    BuiltInFunction {
        name: "print",
        params: vec![WasmType::I32],
        result: None,
        import_module: "env",
        import_name: "print",
    },
];
```

#### 3. Function Resolution During Lowering
- Check if function name matches built-in
- If built-in: use import index
- If user-defined: use function index
- Emit correct Call instruction with proper index

#### 4. Type-Correct Expression Handling
- **Void functions** (println, print): Don't produce stack values
- **Value functions** (user-defined): May produce stack values
- **If expressions**: Produce value only if both branches produce values

### Implementation Plan (EXTREME TDD)

#### Phase 1: Import Section (RED→GREEN→REFACTOR)
1. **RED**: Test that WASM module has import section for println
2. **GREEN**: Implement `emit_import_section()` method
3. **REFACTOR**: Ensure complexity ≤10, A- TDG score

#### Phase 2: Built-in Registry (RED→GREEN→REFACTOR)
1. **RED**: Test built-in function lookup by name
2. **GREEN**: Implement BuiltInRegistry with lookup methods
3. **REFACTOR**: Property test: all advertised built-ins are registered

#### Phase 3: Function Resolution (RED→GREEN→REFACTOR)
1. **RED**: Test Call instruction uses correct index for println
2. **GREEN**: Implement resolve_function() in lower_call
3. **REFACTOR**: Mutation test: verify index calculation is correct

#### Phase 4: Void Expression Handling (RED→GREEN→REFACTOR)
1. **RED**: Test that println() doesn't leave stack values
2. **GREEN**: Fix expression_produces_value() for void calls
3. **REFACTOR**: Property test: void functions never produce values

### Testing Requirements

#### Unit Tests
- `test_import_section_generated()`
- `test_builtin_function_lookup()`
- `test_println_resolves_to_import()`
- `test_void_function_no_stack_value()`

#### Property Tests (10,000 iterations)
```rust
proptest! {
    #[test]
    fn test_all_builtins_registered(name: String) {
        if is_builtin(&name) {
            assert!(BuiltInRegistry::lookup(&name).is_some());
        }
    }

    #[test]
    fn test_void_functions_never_produce_values(builtin: BuiltInFunction) {
        if builtin.result.is_none() {
            let expr = create_call_expr(&builtin.name);
            assert!(!emitter.expression_produces_value(&expr));
        }
    }
}
```

#### Mutation Tests (≥75% coverage)
```bash
cargo mutants --file src/backend/wasm/builtins.rs --timeout 300
cargo mutants --file src/backend/wasm/imports.rs --timeout 300
```

**Target Mutations**:
- Import index calculations
- Function resolution logic
- Type checking for void/value functions

### Acceptance Criteria

✅ **ALL 15-tool validation tests pass**:
- LANG-COMP-003 (control flow): 5/5 examples compile to valid WASM
- LANG-COMP-004 (functions): 4/4 examples compile to valid WASM
- LANG-COMP-005 (string interpolation): 4/4 examples compile to valid WASM

✅ **Quality Gates**:
- PMAT TDG: A- minimum (≥85 points)
- Complexity: ≤10 per function
- SATD: 0 (no TODOs/FIXMEs)
- Mutation coverage: ≥75%
- Property test coverage: 80%+ of WASM backend

✅ **Validation**:
```bash
# All examples must pass
cargo run --bin ruchy -- wasm examples/lang_comp/03-control-flow/01_if.ruchy
wasmparser::validate output.wasm  # Must succeed
```

## Toyota Way Principles Applied

1. **Jidoka (Stop the Line)**: Halted all work when 15-tool validation found failures
2. **Genchi Genbutsu (Go and See)**: Investigated actual WASM bytecode, found missing imports
3. **Kaizen (Continuous Improvement)**: Specification prevents future incomplete features
4. **Poka-Yoke (Error Proofing)**: 15-tool validation catches issues automatically
5. **Respect for People**: Complete implementation respects users who depend on advertised features

## Next Steps

1. ✅ Create this specification (COMPLETE)
2. ⏳ Implement Phase 1: Import Section
3. ⏳ Implement Phase 2: Built-in Registry
4. ⏳ Implement Phase 3: Function Resolution
5. ⏳ Implement Phase 4: Void Expression Handling
6. ⏳ Run full LANG-COMP test suite
7. ⏳ Update CLAUDE.md with completion

## Success Metrics

- **Before**: 8/17 LANG-COMP examples pass 15-tool validation (47%)
- **After**: 17/17 LANG-COMP examples pass 15-tool validation (100%)
- **Quality**: All new code ≥A- TDG, ≤10 complexity, ≥75% mutation coverage
