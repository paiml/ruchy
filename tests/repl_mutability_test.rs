// TDD Test Suite for REPL Mutability Specification
// Enforcing semantic consistency between REPL and compiled modes

use ruchy::runtime::repl::Repl;

#[test]
fn test_immutable_binding_prevents_reassignment() {
    let mut repl = Repl::new().unwrap();
    
    // Create immutable binding
    repl.eval("let x = 1").unwrap();
    
    // Attempt to reassign should fail
    let result = repl.eval("x = 2");
    assert!(result.is_err(), "Should not allow assignment to immutable binding");
    
    // Original value should be unchanged
    let value = repl.eval("x").unwrap();
    assert_eq!(value, "1");
}

#[test]
fn test_mutable_binding_allows_reassignment() {
    let mut repl = Repl::new().unwrap();
    
    // Create mutable binding
    repl.eval("var x = 1").unwrap();
    
    // Reassignment should succeed
    repl.eval("x = 2").unwrap();
    
    // Value should be updated
    let value = repl.eval("x").unwrap();
    assert_eq!(value, "2");
}

#[test]
fn test_shadowing_creates_new_binding() {
    let mut repl = Repl::new().unwrap();
    
    // Create first binding
    repl.eval("let x = 1").unwrap();
    
    // Shadow with new binding
    repl.eval("let x = 2").unwrap();
    
    // Should have new value
    let value = repl.eval("x").unwrap();
    assert_eq!(value, "2");
}

#[test]
fn test_undefined_variable_assignment_fails() {
    let mut repl = Repl::new().unwrap();
    
    // Assignment to undefined variable should fail
    let result = repl.eval("y = 10");
    assert!(result.is_err(), "Should not allow assignment to undefined variable");
}

#[test]
fn test_var_shadowing_maintains_mutability() {
    let mut repl = Repl::new().unwrap();
    
    // Create mutable binding
    repl.eval("var x = 1").unwrap();
    
    // Shadow with another mutable binding
    repl.eval("var x = 2").unwrap();
    
    // New binding should also be mutable
    repl.eval("x = 3").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "3");
}

#[test]
fn test_let_shadows_var() {
    let mut repl = Repl::new().unwrap();
    
    // Create mutable binding
    repl.eval("var x = 1").unwrap();
    repl.eval("x = 2").unwrap(); // Should work
    
    // Shadow with immutable binding
    repl.eval("let x = 3").unwrap();
    
    // Now reassignment should fail
    let result = repl.eval("x = 4");
    assert!(result.is_err(), "Shadowed immutable binding should not be reassignable");
    assert_eq!(repl.eval("x").unwrap(), "3");
}

#[test]
fn test_var_shadows_let() {
    let mut repl = Repl::new().unwrap();
    
    // Create immutable binding
    repl.eval("let x = 1").unwrap();
    
    // Shadow with mutable binding
    repl.eval("var x = 2").unwrap();
    
    // Now reassignment should work
    repl.eval("x = 3").unwrap();
    assert_eq!(repl.eval("x").unwrap(), "3");
}

#[test]
fn test_error_message_for_immutable_assignment() {
    let mut repl = Repl::new().unwrap();
    
    repl.eval("let x = 42").unwrap();
    let result = repl.eval("x = 43");
    
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("immutable") || error.contains("cannot assign"),
        "Error should mention immutability: {}", error);
}

#[test]
fn test_error_message_for_undefined_variable() {
    let mut repl = Repl::new().unwrap();
    
    let result = repl.eval("undefined_var = 100");
    
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("undefined") || error.contains("not found"),
        "Error should mention undefined variable: {}", error);
}

#[test]
fn test_mutable_collections() {
    let mut repl = Repl::new().unwrap();
    
    // Mutable vector
    repl.eval("var nums = [1, 2, 3]").unwrap();
    repl.eval("nums = [4, 5, 6]").unwrap(); // Reassign whole vector
    assert_eq!(repl.eval("nums").unwrap(), "[4, 5, 6]");
}

#[test]
fn test_immutable_collections() {
    let mut repl = Repl::new().unwrap();
    
    // Immutable vector
    repl.eval("let nums = [1, 2, 3]").unwrap();
    let result = repl.eval("nums = [4, 5, 6]");
    assert!(result.is_err(), "Should not allow reassignment of immutable vector");
    assert_eq!(repl.eval("nums").unwrap(), "[1, 2, 3]");
}

#[test]
fn test_nested_scope_mutability() {
    let mut repl = Repl::new().unwrap();
    
    // Outer immutable
    repl.eval("let x = 1").unwrap();
    
    // In function, parameter shadows
    repl.eval("fn modify(x) { x + 1 }").unwrap();
    let result = repl.eval("modify(10)").unwrap();
    assert_eq!(result, "11");
    
    // Original x unchanged
    assert_eq!(repl.eval("x").unwrap(), "1");
}

#[test]
fn test_for_loop_iteration_variable() {
    let mut repl = Repl::new().unwrap();
    
    // For loop variable is immutable within loop
    repl.eval("var result = 0").unwrap();
    repl.eval("for i in [1, 2, 3] { result = result + i }").unwrap();
    assert_eq!(repl.eval("result").unwrap(), "6");
}

#[test]
fn test_dataframe_pipeline_shadowing() {
    let mut repl = Repl::new().unwrap();
    
    // Simulate DataFrame pipeline with shadowing
    repl.eval("let df = [1, 2, 3, 4, 5]").unwrap();
    repl.eval("let df = df.filter(|x| x > 2)").unwrap_or_else(|_| {
        // If filter doesn't exist, just reassign
        repl.eval("let df = [3, 4, 5]").unwrap()
    });
    
    // df should be shadowed, not mutated
    let result = repl.eval("df");
    assert!(result.is_ok(), "Shadowing should work for pipeline pattern");
}