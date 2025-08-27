// TDD test for while loop off-by-one error
// Issue: while i < 3 prints 0,1,2,3 instead of 0,1,2

use ruchy::runtime::Repl;

#[test]
fn test_while_loop_condition_boundary() {
    // Expected: prints 0, 1, 2 (stops when i=3)
    // Actual bug: prints 0, 1, 2, 3
    let code = r#"
        let i = 0
        let result = []
        while i < 3 {
            result.push(i)
            i = i + 1
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    let result = repl.eval(code).expect("Should eval");
    
    // The result should be [0, 1, 2], NOT [0, 1, 2, 3]
    assert_eq!(result.to_string(), "[0, 1, 2]", 
        "While loop should stop when condition becomes false");
}

#[test]
fn test_while_loop_zero_iterations() {
    let code = r#"
        let i = 5
        let result = []
        while i < 3 {
            result.push(i)
            i = i + 1
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    let result = repl.eval(code).expect("Should eval");
    
    assert_eq!(result.to_string(), "[]", 
        "While loop should not execute when condition is initially false");
}

#[test]
fn test_while_loop_exact_boundary() {
    let code = r#"
        let i = 0
        let result = []
        while i <= 2 {
            result.push(i)
            i = i + 1
        }
        result
    "#;
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    let result = repl.eval(code).expect("Should eval");
    
    assert_eq!(result.to_string(), "[0, 1, 2]", 
        "While loop with <= should include boundary value");
}

#[test]
fn test_object_items_method() {
    // TDD test for missing object.items() method
    let code = r#"
        let obj = { "a": 1, "b": 2 }
        obj.items()
    "#;
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    let result = repl.eval(code).expect("Should eval");
    
    // items() should return list of [key, value] tuples
    assert_eq!(result.to_string(), r#"[["a", 1], ["b", 2]]"#,
        "Object.items() should return key-value pairs as list of tuples");
}

#[test]
fn test_object_keys_method() {
    let code = r#"
        let obj = { "a": 1, "b": 2 }
        obj.keys()
    "#;
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    let result = repl.eval(code).expect("Should eval");
    
    assert_eq!(result.to_string(), r#"["a", "b"]"#,
        "Object.keys() should return list of keys");
}

#[test]
fn test_object_values_method() {
    let code = r#"
        let obj = { "a": 1, "b": 2 }
        obj.values()
    "#;
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    let result = repl.eval(code).expect("Should eval");
    
    assert_eq!(result.to_string(), "[1, 2]",
        "Object.values() should return list of values");
}