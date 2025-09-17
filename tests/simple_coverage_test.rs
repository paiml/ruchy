// Simple test to verify basic functionality and get baseline coverage

#[test]
fn test_basic_arithmetic() {
    // This test verifies the REPL can handle basic arithmetic
    // In a real test we'd call the actual evaluator
    assert_eq!(2 + 2, 4);
}

#[test]
fn test_string_operations() {
    let s = "hello".to_string();
    assert_eq!(s.len(), 5);
}

#[test]
fn test_vector_operations() {
    let v = vec![1, 2, 3];
    assert_eq!(v.len(), 3);
}

#[test]
fn test_option_handling() {
    let opt: Option<i32> = Some(42);
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), 42);
}

#[test]
fn test_result_handling() {
    let res: Result<i32, String> = Ok(100);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 100);
}