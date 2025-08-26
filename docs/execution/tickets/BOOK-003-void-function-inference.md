# BOOK-003: Void Function Return Type Inference (BUG-007)

**Priority**: P0 - CRITICAL  
**Impact**: Would fix ~40+ book examples  
**Duration**: 1 day  
**Coverage Target**: 80%  
**Complexity Target**: All functions < 10 (PMAT enforced)

## Problem Statement

Functions with side effects like `println` are incorrectly inferred to return `i32` instead of void/unit type, causing compilation failures. This affects configuration management, logging, and output examples.

## Root Cause Analysis (5 Whys)

1. **Why do void functions get wrong return type?** `has_non_unit_expression()` incorrectly identifies println as non-unit
2. **Why is println considered non-unit?** Current logic treats all Call expressions as value-producing
3. **Why all calls value-producing?** Simplistic assumption that functions return values
4. **Why simplistic assumption?** Initial implementation didn't distinguish side-effect functions
5. **Why not distinguished?** Missing catalog of void/unit functions

## Current Buggy Behavior

```ruchy
fun apply_config(value: i32) {
    println("Applying configuration...");
    if validate_config(value) {
        println("Configuration applied successfully");
    } else {
        println("Invalid configuration");
    }
}
```

Generates:
```rust
fn apply_config(value: i32) -> i32 {  // WRONG! Should be no return type
    println!("Applying configuration...");
    // ...
}
```

## Solution Design  

### Phase 1: Improve Void Detection (Morning)
```rust
// Enhanced is_print_like_call detection
fn is_void_expression(&self, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, .. } => {
            if let ExprKind::Identifier(name) = &func.kind {
                matches!(name.as_str(), 
                    "println" | "print" | "eprintln" | "eprint" |
                    "debug" | "trace" | "info" | "warn" | "error" |
                    "panic" | "assert" | "assert_eq" | "assert_ne" |
                    "todo" | "unimplemented" | "unreachable"
                )
            } else {
                false
            }
        }
        ExprKind::Block(exprs) => {
            exprs.last().map_or(true, |e| self.is_void_expression(e))
        }
        ExprKind::If { then_branch, else_branch, .. } => {
            self.is_void_expression(then_branch) && 
            else_branch.as_ref().map_or(true, |e| self.is_void_expression(e))
        }
        ExprKind::Assignment { .. } => true,
        ExprKind::While { .. } | ExprKind::For { .. } => true,
        _ => false
    }
}
```

### Phase 2: Refactor Return Type Logic (Afternoon)
```rust
fn infer_return_type(&self, name: &str, body: &Expr) -> TokenStream {
    // Explicit return type provided
    if let Some(ty) = self.explicit_return_type {
        return self.transpile_type(ty);
    }
    
    // Special cases
    if name == "main" {
        return quote! {};  // main never has return type
    }
    
    // Check if body is purely void/side-effects
    if self.is_void_expression(body) {
        return quote! {};  // No return type
    }
    
    // Check if function looks numeric
    if self.looks_like_numeric_function(name) {
        return quote! { -> i32 };
    }
    
    // Check if body has value-producing expression
    if self.has_value_expression(body) {
        return quote! { -> i32 };  // Default to i32 for now
    }
    
    // Default: no return type (void)
    quote! {}
}
```

## Test-Driven Development Plan

### RED Phase - Write Failing Tests
```rust
#[test]
fn test_void_function_no_return() {
    let code = "fun log_message(msg) { println(msg); }";
    let transpiled = transpile(code).unwrap();
    assert!(!transpiled.contains("->"));
}

#[test]
fn test_config_function_void() {
    let code = r#"
        fun apply_config(value) {
            println("Applying config");
            if value > 0 {
                println("Valid");
            } else {
                println("Invalid");
            }
        }
    "#;
    let transpiled = transpile(code).unwrap();
    assert!(!transpiled.contains("-> i32"));
}

#[test]
fn test_mixed_void_value_detection() {
    let code = r#"
        fun process(x) {
            println("Processing");
            x * 2  // This SHOULD have return type
        }
    "#;
    let transpiled = transpile(code).unwrap();
    assert!(transpiled.contains("-> i32"));
}
```

### GREEN Phase - Fix Implementation
- Implement `is_void_expression()` with comprehensive checks
- Update `has_non_unit_expression()` to use new logic
- Fix return type inference to handle void properly

### REFACTOR Phase - Ensure Quality
- Extract void detection to separate module
- Create comprehensive list of void functions
- Ensure complexity < 10 for all functions
- Add performance optimizations

## Success Metrics

1. **Primary**: 40+ book examples with void functions now compile
2. **Secondary**: test_03_config_management.ruchy passes
3. **Tertiary**: 80% test coverage on return type inference
4. **Quaternary**: All functions maintain complexity < 10

## Risk Mitigation

- **Risk**: Breaking existing value-returning functions
- **Mitigation**: Conservative approach - only remove return type when certain

- **Risk**: User-defined functions named "println"
- **Mitigation**: Check if function is builtin vs user-defined

## Quality Gates

- [ ] All inference functions have complexity < 10
- [ ] 80% test coverage on return type inference
- [ ] config_management test passes
- [ ] No regression in existing function tests
- [ ] Performance impact negligible

## Example Code That Should Work After Fix

```ruchy
// Pure side-effect functions - no return type
fun log_startup() {
    println("System starting...");
    println("Loading configuration...");
    println("Ready!");
}

// Assignment is void
fun set_global(value) {
    global_state = value;
}

// Loops are void
fun print_numbers() {
    for i in [1, 2, 3] {
        println(i);
    }
}

// Mixed - last expression is void
fun validate_and_log(x) {
    if x > 0 {
        println("Valid");
    } else {
        println("Invalid");
    }
}

// Value-returning (should keep return type)
fun calculate(x) {
    println("Calculating...");  // Side effect
    x * 2                       // Value returned
}
```

## Implementation Notes

### Current Bug Location
- File: `src/backend/transpiler/statements.rs`
- Function: `transpile_function`
- Lines: ~230-245 (return type inference logic)

### Key Functions to Update
1. `has_non_unit_expression()` - Currently too aggressive
2. `is_print_like_call()` - Needs expansion
3. Return type inference block - Needs void detection

## Toyota Way Principles Applied

- **Jidoka**: Detect void functions automatically, prevent incorrect types
- **Genchi Genbutsu**: Test with actual failing book examples
- **Kaizen**: Incremental improvement to type inference
- **Respect for People**: Reduce user confusion with correct types
- **Long-term Philosophy**: Foundation for effect system