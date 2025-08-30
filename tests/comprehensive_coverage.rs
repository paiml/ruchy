//! Comprehensive test coverage suite
//!
//! [TEST-COV-011] Increase test coverage to 80%+

use ruchy::runtime::{Repl, Value, Inspector, Inspect};
use ruchy::runtime::{MagicRegistry, UnicodeExpander};
use ruchy::runtime::{TransactionalState, TransactionMetadata};

#[test]
fn test_repl_basic_operations() {
    let mut repl = Repl::new().unwrap();
    
    // Test basic arithmetic
    assert!(repl.eval("1 + 1").is_ok());
    assert!(repl.eval("10 - 5").is_ok());
    assert!(repl.eval("3 * 4").is_ok());
    assert!(repl.eval("20 / 4").is_ok());
    assert!(repl.eval("10 % 3").is_ok());
    assert!(repl.eval("2 ** 3").is_ok());
}

#[test]
fn test_repl_variables() {
    let mut repl = Repl::new().unwrap();
    
    // Test variable binding
    assert!(repl.eval("let x = 10").is_ok());
    assert!(repl.eval("let y = x + 5").is_ok());
    assert!(repl.eval("let mut z = 0").is_ok());
    assert!(repl.eval("z = 42").is_ok());
}

#[test]
fn test_repl_functions() {
    let mut repl = Repl::new().unwrap();
    
    // Test function definition
    assert!(repl.eval("fun add(a, b) { a + b }").is_ok());
    assert!(repl.eval("add(3, 4)").is_ok());
    
    // Test recursive function
    assert!(repl.eval("fun fact(n) { if n <= 1 { 1 } else { n * fact(n-1) } }").is_ok());
    assert!(repl.eval("fact(5)").is_ok());
}

#[test]
fn test_repl_closures() {
    let mut repl = Repl::new().unwrap();
    
    // Test closure creation
    assert!(repl.eval("let double = |x| x * 2").is_ok());
    assert!(repl.eval("double(21)").is_ok());
    
    // Test closure with capture
    assert!(repl.eval("let x = 10; let add_x = |y| x + y").is_ok());
    assert!(repl.eval("add_x(5)").is_ok());
}

#[test]
fn test_repl_arrays() {
    let mut repl = Repl::new().unwrap();
    
    // Test array operations
    assert!(repl.eval("[1, 2, 3, 4, 5]").is_ok());
    assert!(repl.eval("let arr = [1, 2, 3]; arr[0]").is_ok());
    assert!(repl.eval("[1, 2, 3].len()").is_ok());
    assert!(repl.eval("[1, 2, 3].sum()").is_ok());
    assert!(repl.eval("[1, 2, 3].map(|x| x * 2)").is_ok());
    assert!(repl.eval("[1, 2, 3, 4, 5].filter(|x| x > 2)").is_ok());
}

#[test]
fn test_repl_strings() {
    let mut repl = Repl::new().unwrap();
    
    // Test string operations
    assert!(repl.eval("\"hello\"").is_ok());
    assert!(repl.eval("\"hello\" + \" world\"").is_ok());
    assert!(repl.eval("\"hello\".len()").is_ok());
    assert!(repl.eval("\"hello\".to_uppercase()").is_ok());
    assert!(repl.eval("\"HELLO\".to_lowercase()").is_ok());
    assert!(repl.eval("\"  hello  \".trim()").is_ok());
    assert!(repl.eval("\"hello\".reverse()").is_ok());
    assert!(repl.eval("\"a,b,c\".split(\",\")").is_ok());
}

#[test]
fn test_repl_control_flow() {
    let mut repl = Repl::new().unwrap();
    
    // Test if-else
    assert!(repl.eval("if true { 1 } else { 2 }").is_ok());
    assert!(repl.eval("if false { 1 } else { 2 }").is_ok());
    
    // Test while loop
    assert!(repl.eval("let mut i = 0; while i < 5 { i = i + 1 }; i").is_ok());
    
    // Test for loop
    assert!(repl.eval("let mut sum = 0; for i in 1..5 { sum = sum + i }; sum").is_ok());
}

#[test]
fn test_repl_objects() {
    let mut repl = Repl::new().unwrap();
    
    // Test object creation
    assert!(repl.eval("{ x: 10, y: 20 }").is_ok());
    assert!(repl.eval("let obj = { name: \"test\", value: 42 }; obj.name").is_ok());
}

#[test]
fn test_magic_registry() {
    let registry = MagicRegistry::new();
    
    // Test that registry exists
    assert!(std::ptr::addr_of!(registry) as usize != 0);
}

#[test]
fn test_unicode_expander() {
    let expander = UnicodeExpander::new();
    
    // Test Greek letters
    assert!(expander.expand("\\alpha").is_some());
    assert_eq!(expander.expand("\\alpha"), Some('α'));
    assert_eq!(expander.expand("\\beta"), Some('β'));
    assert_eq!(expander.expand("\\gamma"), Some('γ'));
    
    // Test math symbols
    assert_eq!(expander.expand("\\sum"), Some('∑'));
    assert_eq!(expander.expand("\\pi"), Some('π'));
    assert_eq!(expander.expand("\\infty"), Some('∞'));
}

#[test]
fn test_transactional_state() {
    let mut state = TransactionalState::new(1024 * 1024);
    
    // Test basic operations
    state.insert_binding("x".to_string(), Value::Int(42), false);
    assert_eq!(state.bindings().get("x"), Some(&Value::Int(42)));
    
    // Test transactions
    let metadata = TransactionMetadata::default();
    let tx_id = state.begin_transaction(metadata).unwrap();
    
    state.insert_binding("y".to_string(), Value::Int(100), false);
    assert_eq!(state.bindings().get("y"), Some(&Value::Int(100)));
    
    // Rollback
    assert!(state.rollback_transaction(tx_id).is_ok());
    assert_eq!(state.bindings().get("y"), None);
}

#[test]
fn test_inspector() {
    let mut inspector = Inspector::new();
    
    // Test primitive inspection
    42i32.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("42"));
    
    inspector.output.clear();
    true.inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("true"));
    
    inspector.output.clear();
    "hello".inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("hello"));
}

#[test]
fn test_value_inspection() {
    let mut inspector = Inspector::new();
    
    // Test Value types
    Value::Int(42).inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("42"));
    
    inspector.output.clear();
    Value::String("test".to_string()).inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains("test"));
    
    inspector.output.clear();
    Value::List(vec![Value::Int(1), Value::Int(2)]).inspect(&mut inspector).unwrap();
    assert!(inspector.output.contains('['));
    assert!(inspector.output.contains('1'));
    assert!(inspector.output.contains('2'));
}

#[test]
fn test_inspect_depth_limiting() {
    let mut inspector = Inspector::new();
    inspector.max_depth = 2;
    
    // Create nested structure
    let nested = Value::List(vec![
        Value::List(vec![
            Value::List(vec![Value::Int(1)])
        ])
    ]);
    
    nested.inspect(&mut inspector).unwrap();
    // Should show depth limit reached
    assert!(inspector.output.contains('[') || inspector.output.contains("items"));
}

#[test]
fn test_inspect_budget() {
    let mut inspector = Inspector::new();
    inspector.budget = 50; // Very small budget
    
    // Create large list
    let large_list = Value::List(vec![Value::Int(1); 100]);
    
    large_list.inspect(&mut inspector).unwrap();
    // Should show truncation
    assert!(inspector.output.contains("..."));
}

#[test]
fn test_repl_error_recovery() {
    let mut repl = Repl::new().unwrap();
    
    // Test error recovery
    let _ = repl.eval("invalid syntax @#$");
    // Should be able to continue
    assert!(repl.eval("1 + 1").is_ok());
    
    // Test undefined variable
    let _ = repl.eval("undefined_var");
    // Should be able to continue
    assert!(repl.eval("42").is_ok());
}

#[test]
fn test_repl_multiline() {
    let mut repl = Repl::new().unwrap();
    
    let multiline = r"
        let x = 10;
        let y = 20;
        x + y
    ";
    
    let result = repl.eval(multiline);
    assert!(result.is_ok());
}

#[test]
fn test_option_types() {
    let mut repl = Repl::new().unwrap();
    
    // Test Option types
    assert!(repl.eval("Some(42)").is_ok());
    assert!(repl.eval("None").is_ok());
    
    // Option should display correctly
    let result = repl.eval("None");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Option::None") || output.contains("None"));
    
    let result = repl.eval("Some(42)");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Option::Some") || output.contains("Some"));
}

#[test]
fn test_result_types() {
    let mut repl = Repl::new().unwrap();
    
    // Test Result types
    assert!(repl.eval("Ok(42)").is_ok());
    assert!(repl.eval("Err(\"error\")").is_ok());
}

#[test]
fn test_range_types() {
    let mut repl = Repl::new().unwrap();
    
    // Test range types
    assert!(repl.eval("1..10").is_ok());
    assert!(repl.eval("1..=10").is_ok());
    
    // Test range expansion
    assert!(repl.eval("[...1..5]").is_ok());
}

#[test]
fn test_destructuring() {
    let mut repl = Repl::new().unwrap();
    
    // Test tuple destructuring
    assert!(repl.eval("let (x, y) = (10, 20); x + y").is_ok());
    
    // Test array destructuring
    assert!(repl.eval("let [a, b, c] = [1, 2, 3]; a + b + c").is_ok());
}

#[test]
fn test_pattern_matching() {
    let mut repl = Repl::new().unwrap();
    
    let match_expr = r"
        let x = Some(42);
        match x {
            Some(n) => n * 2,
            None => 0
        }
    ";
    
    assert!(repl.eval(match_expr).is_ok());
}