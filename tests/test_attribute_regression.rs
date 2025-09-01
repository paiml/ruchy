use ruchy::runtime::repl::Repl;
use anyhow::Result;

#[test]
fn test_test_attribute_compilation_regression() -> Result<()> {
    let mut repl = Repl::new()?;
    
    // This is the test case that was failing in P0-BOOK-001
    // The main thing is that this should NOT panic during compilation
    let test_code = r"
#[test]
fn test_simple_addition() {
    assert_eq!(2 + 2, 4);
}
";

    // This should not panic and should return success (empty result is fine)
    let _result = repl.eval(test_code)?;
    
    // The key success criterion is that we get here without panicking
    // The previous bug would cause: "generate_return_type_tokens called for function: 'test_simple_addition'"
    Ok(())
}

#[test] 
fn test_multiple_test_functions() -> Result<()> {
    let mut repl = Repl::new()?;
    
    let test_code = r"
#[test]
fn test_addition() {
    assert_eq!(1 + 1, 2);
}

#[test]
fn test_multiplication() {
    assert_eq!(3 * 4, 12);
}
";

    // Should not panic with multiple test functions
    let _result = repl.eval(test_code)?;
    
    Ok(())
}