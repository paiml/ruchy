// TDD tests for second wave of entropy reduction refactoring
// Validates that with_saved_bindings and helper patterns work correctly

use ruchy::runtime::repl::Repl;

#[test]
fn test_map_with_saved_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Set a binding before map
    repl.eval("let x = 100").unwrap();
    
    // Map should not affect outer binding
    let result = repl.eval("[1, 2, 3].map(|x| x * 2)").unwrap();
    assert_eq!(result, "[2, 4, 6]");
    
    // Verify x is still 100
    let result = repl.eval("x").unwrap();
    assert_eq!(result, "100");
}

#[test]
fn test_filter_with_saved_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Set a binding before filter
    repl.eval("let n = 5").unwrap();
    
    // Filter should not affect outer binding
    let result = repl.eval("[1, 2, 3, 4, 5, 6].filter(|n| n > 3)").unwrap();
    assert_eq!(result, "[4, 5, 6]");
    
    // Verify n is still 5
    let result = repl.eval("n").unwrap();
    assert_eq!(result, "5");
}

#[test]
fn test_reduce_with_saved_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Set bindings before reduce
    repl.eval("let acc = 999").unwrap();
    repl.eval("let item = 888").unwrap();
    
    // Reduce should not affect outer bindings
    let result = repl.eval("[1, 2, 3].reduce(|acc, item| acc + item, 0)").unwrap();
    assert_eq!(result, "6");
    
    // Verify original bindings unchanged
    let result = repl.eval("acc").unwrap();
    assert_eq!(result, "999");
    let result = repl.eval("item").unwrap();
    assert_eq!(result, "888");
}

#[test]
fn test_resource_limit_helpers() {
    let mut repl = Repl::new().unwrap();
    
    // Deep recursion should trigger depth limit
    let result = repl.eval("fn deep(n) { if n == 0 { 0 } else { deep(n - 1) } } deep(10000)");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Maximum recursion depth"));
}

#[test]
fn test_evaluate_arg_helpers() {
    let mut repl = Repl::new().unwrap();
    
    // Functions that evaluate arguments should work
    let result = repl.eval("sqrt(4 + 5)").unwrap();
    assert_eq!(result, "3");
    
    let result = repl.eval("pow(2 + 1, 3)").unwrap();
    assert_eq!(result, "27");
}

#[test]
fn test_value_constructor_helpers() {
    let mut repl = Repl::new().unwrap();
    
    // String values
    let result = repl.eval(r#""hello".to_upper()"#).unwrap();
    assert_eq!(result, r#""HELLO""#);
    
    // Integer values
    let result = repl.eval("42.abs()").unwrap();
    assert_eq!(result, "42");
    
    // Float values
    let result = repl.eval("3.14.round()").unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_nested_lambda_bindings() {
    let mut repl = Repl::new().unwrap();
    
    // Nested lambdas should have proper binding isolation
    repl.eval("let outer = 100").unwrap();
    
    let result = repl.eval(
        "[1, 2, 3].map(|x| [10, 20].map(|y| x + y))"
    ).unwrap();
    assert_eq!(result, "[[11, 21], [12, 22], [13, 23]]");
    
    // Outer binding should be unchanged
    let result = repl.eval("outer").unwrap();
    assert_eq!(result, "100");
}

#[test]
fn test_lambda_closure_behavior() {
    let mut repl = Repl::new().unwrap();
    
    // Lambda should capture outer scope
    repl.eval("let multiplier = 10").unwrap();
    
    let result = repl.eval("[1, 2, 3].map(|x| x * multiplier)").unwrap();
    assert_eq!(result, "[10, 20, 30]");
    
    // But not modify it
    let result = repl.eval("multiplier").unwrap();
    assert_eq!(result, "10");
}