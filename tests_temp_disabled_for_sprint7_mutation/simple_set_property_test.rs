// Simple property test for debugging
use ruchy::compile;

#[test]
fn test_single_element_42() {
    let code = "fun main() { let s = {42}; }";
    let result = compile(code);
    assert!(result.is_ok(), "Failed to compile: {result:?}");
    let output = result.unwrap();
    println!("Generated code:\n{output}");
    assert!(output.contains("HashSet"), "Output doesn't contain HashSet");
    assert!(
        output.contains("insert (42)") || output.contains("insert(42)"),
        "Output doesn't contain insert call"
    );
}

#[test]
fn test_two_elements() {
    let code = "fun main() { let s = {1, 2}; }";
    let result = compile(code);
    assert!(result.is_ok(), "Failed to compile: {result:?}");
    let output = result.unwrap();
    println!("Generated code:\n{output}");
    assert!(output.contains("HashSet"));
    assert!(output.contains("insert (1)") || output.contains("insert(1)"));
    assert!(output.contains("insert (2)") || output.contains("insert(2)"));
}
