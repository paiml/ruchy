//! TRANSPILER-011: Nested field access on untyped/typed parameters
//! Bug: event.requestContext.requestId transpiles to `event::requestContext::requestId` (invalid Rust)
//! Root Cause: Default heuristic assumes nested paths are modules, not struct fields
//! Fix: Check if root is a variable/parameter → use . syntax, not :: syntax

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

#[test]
#[ignore = "nested field access bug not fixed yet"]
fn test_transpiler_011_01_nested_field_access_on_parameter() {
    let code = r#"
fn main() {
    fn handler(event) {
        let request_id = event.requestContext.requestId;
        request_id
    }

    println!("Test");
}
"#;

    let ast = Parser::new(code).parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile(&ast)
        .expect("TRANSPILER-011: Transpilation should succeed!")
        .to_string();

    // Should use . (field access), not :: (module path)
    // Note: TokenStream formats with spaces, so check for "event . requestContext . requestId"
    assert!(
        rust_code.contains("event . requestContext . requestId")
            || rust_code.contains("event.requestContext.requestId"),
        "TRANSPILER-011: Should use . syntax for nested field access on variables!\nExpected: event . requestContext . requestId\nGot:\n{rust_code}"
    );

    // Should NOT use module path syntax
    assert!(
        !rust_code.contains("event::requestContext")
            && !rust_code.contains("event :: requestContext"),
        "TRANSPILER-011: Should NOT use :: syntax for variable field access!\nGot:\n{rust_code}"
    );
}

#[test]
#[ignore = "nested field access bug not fixed yet"]
fn test_transpiler_011_02_nested_field_access_typed_parameter() {
    let code = r"
fun handler(event: &str) -> &str {
    event.requestContext.requestId
}
";

    let ast = Parser::new(code).parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile(&ast)
        .expect("TRANSPILER-011: Transpilation should succeed!")
        .to_string();

    // Should use . (field access), not :: (module path)
    assert!(
        rust_code.contains("event.requestContext.requestId")
            || rust_code.contains("event . requestContext . requestId"),
        "TRANSPILER-011: Should use . syntax even with type annotation!\nGot:\n{rust_code}"
    );
}

#[test]
#[ignore = "nested field access bug not fixed yet"]
fn test_transpiler_011_03_hello_world_lambda_full_example() {
    // This is the actual failing example from ruchy-lambda
    let code = r#"
fn main() {
    fn handler(event) {
        let request_id = event.requestContext.requestId;
        let message = f"Hello from Ruchy Lambda! Request ID: {request_id}";

        {
            statusCode: 200,
            body: message
        }
    }

    println("Hello World Lambda Handler initialized");
}
"#;

    let ast = Parser::new(code).parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);

    assert!(
        result.is_ok(),
        "TRANSPILER-011: hello_world.ruchy should transpile successfully!\nError: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Should use . syntax for event fields
    assert!(
        rust_code.contains("event.requestContext") || rust_code.contains("event . requestContext"),
        "TRANSPILER-011: event.requestContext should use . syntax!\nGot:\n{rust_code}"
    );

    // Should NOT use :: for variable field access
    assert!(
        !rust_code.contains("event::requestContext"),
        "TRANSPILER-011: Should NOT use :: for variables!\nGot:\n{rust_code}"
    );
}
