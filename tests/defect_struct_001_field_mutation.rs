//! DEFECT-STRUCT-001: Struct Field Mutation Broken
//!
//! **Problem**: Struct field mutation fails with "Cannot access field 'count' on non-object"
//! **Root Cause**: TBD (to be determined during investigation)
//! **Book Examples Affected**: ch19-00-structs-oop.md (examples 3, 7)
//!
//! **Reproduction**:
//! ```ruchy
//! struct Counter { count: i32 }
//! let mut c = Counter { count: 0 }
//! println(c.count)  // ✅ Works: prints 0
//! c.count = 5       // ❌ FAILS: "Cannot access field 'count' on non-object"
//! ```
//!
//! Run with: cargo test --test `defect_struct_001_field_mutation`

use ruchy::frontend::parser::Parser as RuchyParser;
use ruchy::runtime::interpreter::{Interpreter, Value};

/// Helper: Execute Ruchy code and return result
fn execute_ruchy(source: &str) -> Result<Value, String> {
    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {e:?}"))?;
    let mut interpreter = Interpreter::new();
    interpreter
        .eval_expr(&ast)
        .map_err(|e| format!("Runtime error: {e:?}"))
}

// ============================================================================
// RED PHASE: Failing Tests (Reproduce Defect)
// ============================================================================

#[test]
fn test_defect_struct_001_simple_field_mutation() {
    // This test should PASS once the defect is fixed
    let code = r"
        struct Counter { count: i32 }
        let mut c = Counter { count: 0 }
        c.count = 5
        c.count
    ";

    let result = execute_ruchy(code);
    assert!(result.is_ok(), "Struct field mutation should work: {result:?}");
    assert_eq!(result.unwrap(), Value::Integer(5), "Field should be mutated to 5");
}

#[test]
fn test_defect_struct_001_field_increment() {
    // Book example from ch19-00-structs-oop.md
    let code = r"
        struct Counter { count: i32 }
        let mut c = Counter { count: 0 }
        c.count = c.count + 1
        c.count
    ";

    let result = execute_ruchy(code);
    assert!(result.is_ok(), "Field increment should work: {result:?}");
    assert_eq!(result.unwrap(), Value::Integer(1), "Field should be incremented to 1");
}

#[test]
fn test_defect_struct_001_multiple_mutations() {
    // Book example from ch19-00-structs-oop.md
    let code = r"
        struct Counter { count: i32 }
        let mut c = Counter { count: 0 }
        c.count = 5
        c.count = c.count + 1
        c.count
    ";

    let result = execute_ruchy(code);
    assert!(result.is_ok(), "Multiple mutations should work: {result:?}");
    assert_eq!(result.unwrap(), Value::Integer(6), "Final value should be 6");
}

#[test]
fn test_defect_struct_001_field_access_still_works() {
    // Verify field ACCESS works (not broken by the fix)
    let code = r"
        struct Counter { count: i32 }
        let c = Counter { count: 42 }
        c.count
    ";

    let result = execute_ruchy(code);
    assert!(result.is_ok(), "Field access should still work: {result:?}");
    assert_eq!(result.unwrap(), Value::Integer(42), "Field access should return 42");
}

#[test]
fn test_defect_struct_001_multiple_fields() {
    // Test mutation with multiple fields
    let code = r"
        struct Point { x: i32, y: i32 }
        let mut p = Point { x: 10, y: 20 }
        p.x = 15
        p.y = 25
        p.x + p.y
    ";

    let result = execute_ruchy(code);
    assert!(result.is_ok(), "Multiple field mutations should work: {result:?}");
    assert_eq!(result.unwrap(), Value::Integer(40), "x + y should be 40");
}

// ============================================================================
// GREEN PHASE: Implementation (To Be Added)
// ============================================================================
// Implementation changes will be made in:
// - src/runtime/interpreter.rs (likely fix location)
// - src/frontend/parser.rs (if parser issue)

// ============================================================================
// REFACTOR PHASE: Quality Validation (After Fix)
// ============================================================================
// After fix:
// - Run cargo test --test defect_struct_001_field_mutation
// - Verify all 5 tests pass
// - Check complexity: pmat analyze complexity src/runtime/interpreter.rs
// - Run mutation tests: cargo mutants --file src/runtime/interpreter.rs --timeout 300
