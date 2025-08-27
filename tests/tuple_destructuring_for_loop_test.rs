// TDD: Test for tuple destructuring in for loops
// Bug: `for key, value in obj.items()` doesn't work

use ruchy::runtime::Repl;

#[test]
fn test_tuple_destructuring_in_for_loop() {
    let code = r#"
        let obj = {"a": 1, "b": 2}
        let result = []
        for key, value in obj.items() {
            result = result.push(key + "=" + value.to_string())
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    
    // Should produce something like ["a=1", "b=2"]
    assert!(result.to_string().contains("a=1") && result.to_string().contains("b=2"),
        "Expected key=value pairs, got: {:?}", result.to_string());
}

#[test]
fn test_tuple_destructuring_in_for_loop_simple() {
    // Simple test with list of tuples
    let code = r#"
        let pairs = [("x", 1), ("y", 2)]
        let result = []
        for key, value in pairs {
            result = result.push(key + "=" + value.to_string())
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    
    assert_eq!(result.to_string(), r#"["x=1", "y=2"]"#,
        "Simple tuple destructuring should work");
}

#[test]
fn test_for_loop_without_destructuring_works() {
    // This should work as baseline
    let code = r#"
        let pairs = [("x", 1), ("y", 2)]
        let result = []
        for pair in pairs {
            result = result.push("item")
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    
    assert_eq!(result.to_string(), r#"["item", "item"]"#,
        "Basic for loop should work");
}

#[test]
fn test_object_items_works() {
    // Verify obj.items() works independently
    let code = r#"
        let obj = {"a": 1, "b": 2}
        obj.items()
    "#;
    
    let mut repl = Repl::new().expect("REPL should work");
    let result = repl.eval(code).expect("Should eval");
    
    assert!(result.to_string().contains("(\"a\", 1)") && 
            result.to_string().contains("(\"b\", 2)"),
        "obj.items() should return tuples");
}