//! PARSER-093: Module declaration support (Issue #137 - ruchy-lambda)
//!
//! BUG: Parser creates `ModuleDeclaration` AST nodes, but transpiler doesn't support them.
//! ERROR: "Unsupported expression kind: `ModuleDeclaration` { name: \"`http_client`\" }"
//!
//! IMPACT: Cannot compose Ruchy code with external Rust modules, forces all-or-nothing approach.
//!
//! EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

/// Test 1: Simple external module declaration
/// Example: `mod http_client;`
/// Should transpile to: `mod http_client;`
#[test]
fn test_parser_093_01_simple_mod_declaration() {
    let code = "mod http_client;";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should successfully transpile mod declaration, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(
        rust_code.contains("mod http_client"),
        "Should contain mod declaration, got: {rust_code}"
    );
}

/// Test 2: Public module declaration
/// Example: `pub mod http_client;`
/// Should transpile to: `pub mod http_client;`
#[test]
fn test_parser_093_02_public_mod_declaration() {
    let code = "pub mod http_client;";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should successfully transpile pub mod, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(
        rust_code.contains("pub mod http_client"),
        "Should contain pub mod, got: {rust_code}"
    );
}

/// Test 3: Multiple module declarations
/// Should handle multiple mod statements
#[test]
fn test_parser_093_03_multiple_mod_declarations() {
    let code = r"
mod http_client;
mod websocket;
mod tls;
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile multiple mods, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(
        rust_code.contains("mod http_client"),
        "Should have http_client mod"
    );
    assert!(rust_code.contains("mod websocket"), "Should have websocket mod");
    assert!(rust_code.contains("mod tls"), "Should have tls mod");
}

/// Test 4: Module declaration with struct after
/// Verifies mod doesn't break subsequent code
#[test]
fn test_parser_093_04_mod_with_struct() {
    let code = r"
mod http_client;

pub struct Runtime {
    endpoint: String,
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile mod + struct, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("mod http_client"), "Should have mod");
    assert!(
        rust_code.contains("struct Runtime"),
        "Should have struct after mod"
    );
}

/// Test 5: Module with use statement after
/// Verifies mod works with import statements
#[test]
fn test_parser_093_05_mod_with_use() {
    let code = r"
mod http_client;
use std::io::Read;
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile mod + use, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("mod http_client"), "Should have mod");
    assert!(rust_code.contains("use std :: io :: Read"), "Should have use");
}

/// Test 6: pub(crate) module declaration
/// Restricted visibility modifier
#[test]
fn test_parser_093_06_pub_crate_mod() {
    let code = "pub(crate) mod internal;";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile pub(crate) mod, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(
        rust_code.contains("pub (crate) mod internal")
            || rust_code.contains("pub(crate) mod internal"),
        "Should have pub(crate) mod, got: {rust_code}"
    );
}

/// Test 7: Issue #137 reproduction - ruchy-lambda use case
/// Actual code pattern from ruchy-lambda project
#[test]
fn test_parser_093_07_issue_137_repro() {
    let code = r#"
mod http_client;

pub struct LambdaRuntime {
    endpoint: String,
}

impl LambdaRuntime {
    pub fn new(endpoint: String) -> Self {
        LambdaRuntime { endpoint }
    }

    pub fn invoke(&self) -> bool {
        let result = http_client::http_get(&self.endpoint, "/health");
        result
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Issue #137 pattern should work, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("mod http_client"), "Should have mod");
    assert!(
        rust_code.contains("struct LambdaRuntime"),
        "Should have struct"
    );
    assert!(
        rust_code.contains("http_client :: http_get"),
        "Should preserve :: in module call (PARSER-094)"
    );
}

/// Test 8: Inline module (mod foo { ... })
/// Tests inline module blocks (if supported)
#[test]
#[ignore = "Inline modules may not be supported yet"]
fn test_parser_093_08_inline_module() {
    let code = r"
mod utils {
    pub fn helper() -> i32 {
        42
    }
}

let x = utils::helper();
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile inline module, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("mod utils"), "Should have mod utils");
    assert!(rust_code.contains("fn helper"), "Should have function");
}
