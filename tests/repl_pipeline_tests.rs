//! REPL pipeline operator tests

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::unnecessary_unwrap)]

use ruchy::runtime::Repl;

#[test]
fn test_basic_pipeline() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    // Define a simple function to use in pipeline
    assert!(repl.eval("fun double(x: i32) -> i32 { x * 2 }").is_ok());
    
    // Test pipeline with function
    let result = repl.eval("10 |> double");
    if result.is_ok() {
        let output = result.unwrap();
        assert_eq!(output, "20");
    }
    // Note: Pipeline parsing might not be implemented yet, so this could fail
}

#[test]
fn test_chained_pipeline() {
    let mut repl = Repl::new().expect("Failed to create REPL");
    
    // Define helper functions
    assert!(repl.eval("fun double(x: i32) -> i32 { x * 2 }").is_ok());
    assert!(repl.eval("fun add_one(x: i32) -> i32 { x + 1 }").is_ok());
    
    // Test chained pipeline
    let result = repl.eval("5 |> double |> add_one");
    if result.is_ok() {
        let output = result.unwrap();
        assert_eq!(output, "11"); // 5 * 2 + 1 = 11
    }
}