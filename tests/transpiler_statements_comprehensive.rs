//! EXTREME TDD: Transpiler statements.rs Comprehensive Coverage
//!
//! Target: src/backend/transpiler/statements.rs (7,191 lines, 71.1/100 TDG score)
//! Strategy: Test all statement types via direct transpiler API
//! Quality: Increase TDG from 71.1 (B-) → 85+ (A-)
//!
//! Coverage approach:
//! - Let statements (immutable, mutable, type annotations)
//! - If/else statements (simple, elif chains, nested)
//! - Match statements (literals, patterns, guards)
//! - For loops (range, arrays, nested)
//! - While loops (simple, break, continue)
//! - Function definitions (no params, with params, return types)
//! - Return statements (early return, value return)
//! - Assignment statements (simple, compound)
//! - Expression statements
//!
//! Test Protocol: Parse → Transpile → Verify Code → Compile with rustc

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};
use std::fs;
use std::process::Command;

// ============================================================================
// Let Statements
// ============================================================================

#[test]
fn test_let_immutable_simple() {
    let code = r#"
pub fn test() {
    let x = 42;
    println!("{}", x);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify 'let' keyword present
    assert!(
        rust_code.contains("let x"),
        "Should contain 'let x', got:\n{rust_code}"
    );

    // Verify rustc compilation
    fs::write("/tmp/transpiler_statements_let_immutable.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/transpiler_statements_let_immutable.rs",
        ])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("Let statement compilation failed:\n{stderr}\n\nCode:\n{rust_code}");
    }
}

#[test]
fn test_let_mutable() {
    let code = r#"
pub fn test() {
    let mut x = 10;
    x = 20;
    println!("{}", x);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify 'let mut' keyword present
    assert!(
        rust_code.contains("let mut x"),
        "Should contain 'let mut x', got:\n{rust_code}"
    );
}

#[test]
fn test_let_with_type_annotation() {
    let code = r#"
pub fn test() {
    let x: i32 = 42;
    println!("{}", x);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify type annotation preserved
    assert!(
        rust_code.contains("i32"),
        "Should preserve i32 type, got:\n{rust_code}"
    );
}

#[test]
fn test_let_array() {
    let code = r#"
pub fn test() {
    let arr = [1, 2, 3];
    println!("{:?}", arr);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify array literal present
    assert!(
        rust_code.contains("vec!") || rust_code.contains("[1"),
        "Should contain array/vec, got:\n{rust_code}"
    );
}

// ============================================================================
// If/Else Statements
// ============================================================================

#[test]
fn test_if_simple() {
    let code = r#"
pub fn test(x: i32) -> String {
    if x > 10 {
        return String::from("big");
    }
    String::from("small")
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("if"),
        "Should contain 'if', got:\n{rust_code}"
    );

    // Verify rustc compilation
    fs::write("/tmp/transpiler_statements_if_simple.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/transpiler_statements_if_simple.rs",
        ])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("If statement compilation failed:\n{stderr}\n\nCode:\n{rust_code}");
    }
}

#[test]
fn test_if_else() {
    let code = r#"
pub fn test(x: i32) -> String {
    if x > 10 {
        String::from("big")
    } else {
        String::from("small")
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("if") && rust_code.contains("else"),
        "Should contain 'if' and 'else', got:\n{rust_code}"
    );
}

#[test]
fn test_if_elif_else() {
    let code = r#"
pub fn test(x: i32) -> String {
    if x > 100 {
        String::from("huge")
    } else if x > 10 {
        String::from("big")
    } else {
        String::from("small")
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("else if") || rust_code.contains("} else {"),
        "Should contain elif chain, got:\n{rust_code}"
    );
}

#[test]
fn test_if_nested() {
    let code = r#"
pub fn test(x: i32, y: i32) -> String {
    if x > 10 {
        if y > 20 {
            String::from("both big")
        } else {
            String::from("x big, y small")
        }
    } else {
        String::from("x small")
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());
}

// ============================================================================
// Match Statements
// ============================================================================

#[test]
fn test_match_literal() {
    let code = r#"
pub fn test(x: i32) -> String {
    match x {
        1 => String::from("one"),
        2 => String::from("two"),
        _ => String::from("other")
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("match"),
        "Should contain 'match', got:\n{rust_code}"
    );

    // Verify rustc compilation
    fs::write("/tmp/transpiler_statements_match_literal.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/transpiler_statements_match_literal.rs",
        ])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("Match statement compilation failed:\n{stderr}\n\nCode:\n{rust_code}");
    }
}

#[test]
fn test_match_wildcard() {
    let code = r#"
pub fn test(x: i32) -> String {
    match x {
        _ => String::from("anything")
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains('_'),
        "Should contain wildcard, got:\n{rust_code}"
    );
}

// ============================================================================
// For Loop Statements
// ============================================================================

#[test]
fn test_for_range() {
    let code = r#"
pub fn test() {
    for i in 0..10 {
        println!("{}", i);
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("for"),
        "Should contain 'for', got:\n{rust_code}"
    );

    // Verify rustc compilation
    fs::write("/tmp/transpiler_statements_for_range.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/transpiler_statements_for_range.rs",
        ])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("For loop compilation failed:\n{stderr}\n\nCode:\n{rust_code}");
    }
}

#[test]
fn test_for_array() {
    let code = r#"
pub fn test() {
    let arr = vec![1, 2, 3];
    for item in arr {
        println!("{}", item);
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());
}

#[test]
fn test_for_nested() {
    let code = r#"
pub fn test() {
    for i in 0..5 {
        for j in 0..5 {
            println!("{} {}", i, j);
        }
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());
}

// ============================================================================
// While Loop Statements
// ============================================================================

#[test]
fn test_while_simple() {
    let code = r"
pub fn test() {
    let mut i = 0;
    while i < 10 {
        i = i + 1;
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("while"),
        "Should contain 'while', got:\n{rust_code}"
    );
}

#[test]
fn test_while_with_break() {
    let code = r"
pub fn test() {
    let mut i = 0;
    while true {
        i = i + 1;
        if i > 10 {
            break;
        }
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("break"),
        "Should contain 'break', got:\n{rust_code}"
    );
}

#[test]
fn test_while_with_continue() {
    let code = r#"
pub fn test() {
    let mut i = 0;
    while i < 10 {
        i = i + 1;
        if i % 2 == 0 {
            continue;
        }
        println!("{}", i);
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("continue"),
        "Should contain 'continue', got:\n{rust_code}"
    );
}

// ============================================================================
// Function Definition Statements
// ============================================================================

#[test]
fn test_fn_no_params() {
    let code = r#"
pub fn greet() {
    println!("hello");
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("fn greet"),
        "Should contain 'fn greet', got:\n{rust_code}"
    );
}

#[test]
fn test_fn_with_params() {
    let code = r"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("fn add") && rust_code.contains("i32"),
        "Should contain 'fn add' with i32, got:\n{rust_code}"
    );
}

#[test]
fn test_fn_with_return_type() {
    let code = r"
pub fn get_answer() -> i32 {
    42
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("-> i32"),
        "Should contain '-> i32', got:\n{rust_code}"
    );
}

// ============================================================================
// Return Statements
// ============================================================================

#[test]
fn test_return_early() {
    let code = r"
pub fn test(x: i32) -> i32 {
    if x < 0 {
        return 0;
    }
    x * 2
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("return"),
        "Should contain 'return', got:\n{rust_code}"
    );
}

#[test]
fn test_return_value() {
    let code = r"
pub fn test() -> i32 {
    return 42;
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());
}

// ============================================================================
// Assignment Statements
// ============================================================================

#[test]
fn test_assignment_simple() {
    let code = r"
pub fn test() {
    let mut x = 10;
    x = 20;
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("x = 20"),
        "Should contain assignment, got:\n{rust_code}"
    );
}

#[test]
fn test_assignment_with_expression() {
    let code = r"
pub fn test() {
    let mut x = 10;
    x = x + 5;
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());
}

// ============================================================================
// Expression Statements
// ============================================================================

#[test]
fn test_expression_statement() {
    let code = r#"
pub fn test() {
    println!("hello");
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());
}

// ============================================================================
// Complex Integration Tests
// ============================================================================

#[test]
fn test_integration_multiple_statements() {
    let code = r"
pub fn calculate(x: i32) -> i32 {
    let mut result = 0;

    if x > 100 {
        result = x * 2;
    } else {
        result = x + 10;
    }

    for i in 0..5 {
        result = result + i;
    }

    match result {
        r if r > 1000 => 1000,
        _ => result
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify all statement types present
    assert!(rust_code.contains("let"));
    assert!(rust_code.contains("if"));
    assert!(rust_code.contains("for"));
    assert!(rust_code.contains("match"));

    // Verify rustc compilation
    fs::write("/tmp/transpiler_statements_integration.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/transpiler_statements_integration.rs",
        ])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("Integration test compilation failed:\n{stderr}\n\nCode:\n{rust_code}");
    }
}
