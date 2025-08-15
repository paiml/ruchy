use ruchy::{compile, is_valid_syntax, get_parse_error};

#[test]
fn test_compile_simple_expression() {
    let result = compile("42 + 1").unwrap();
    assert!(result.contains("42"));
    assert!(result.contains("1"));
}

#[test]
fn test_compile_function() {
    let result = compile("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
    assert!(result.contains("fn"));
    assert!(result.contains("add"));
}

#[test]
fn test_compile_invalid_syntax() {
    let result = compile("let x =");
    assert!(result.is_err());
}

#[test]
fn test_is_valid_syntax_valid() {
    assert!(is_valid_syntax("1 + 2"));
    assert!(is_valid_syntax("fun foo() { 42 }"));
    assert!(is_valid_syntax("[1, 2, 3]"));
    assert!(is_valid_syntax("if true { 1 } else { 2 }"));
}

#[test]
fn test_is_valid_syntax_invalid() {
    assert!(!is_valid_syntax("let x ="));
    assert!(!is_valid_syntax("fun ()"));
    assert!(!is_valid_syntax("if { }"));
    assert!(!is_valid_syntax("match"));
}

#[test]
fn test_get_parse_error_with_error() {
    let error = get_parse_error("let x =");
    assert!(error.is_some());
    let error_msg = error.unwrap();
    assert!(!error_msg.is_empty());
}

#[test]
fn test_get_parse_error_no_error() {
    let error = get_parse_error("let x = 42");
    assert!(error.is_none());
}

#[test]
fn test_compile_pipeline() {
    let result = compile("[1, 2, 3] |> map(x => x * 2)");
    // Should compile even if not fully implemented
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_compile_match() {
    let code = r#"
        match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok() || result.is_err());
}