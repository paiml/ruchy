//! QUALITY-001: Method receiver preservation (&self, &mut self, self)
//!
//! BUG: Transpiler transforms &self → self, causing move errors
//! ERROR: error[E0382]: use of moved value: `client`
//!
//! ROOT CAUSE: generate_param_tokens doesn't handle Rust's special receiver syntax
//!
//! IMPACT: Methods with &self cannot be called multiple times (ownership moved)
//!
//! EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

/// Test 1: Basic &self method - should preserve reference
#[test]
fn test_quality_001_01_immutable_self_reference() {
    let code = r#"
pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Should transpile &self methods, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // CRITICAL: Must preserve &self, not transform to self
    assert!(
        rust_code.contains("& self") || rust_code.contains("&self"),
        "Should preserve &self reference, got: {}",
        rust_code
    );

    // Should NOT have ownership-taking self
    assert!(
        !rust_code.contains("fn get_endpoint ( self )"),
        "Should NOT transform &self to self, got: {}",
        rust_code
    );
}

/// Test 2: Multiple &self method calls - must not move
#[test]
fn test_quality_001_02_multiple_self_calls_no_move() {
    let code = r#"
pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn new(endpoint: String) -> Self {
        Client { endpoint }
    }

    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }
}

pub fn test() -> String {
    let client = Client::new("http://localhost:8080".to_string());
    let ep1 = client.get_endpoint();
    let ep2 = client.get_endpoint();  // MUST work - &self doesn't move
    ep1
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify rustc compilation succeeds
    std::fs::write("/tmp/quality_001_02_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/quality_001_02_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Transpiled code fails rustc compilation:\n{}\n\nGenerated code:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 3: &mut self method - mutable reference
#[test]
fn test_quality_001_03_mutable_self_reference() {
    let code = r#"
pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn get(&self) -> i32 {
        self.count
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Must preserve &mut self
    assert!(
        rust_code.contains("& mut self") || rust_code.contains("&mut self"),
        "Should preserve &mut self, got: {}",
        rust_code
    );
}

/// Test 4: Owned self (consuming method) - should work as-is
#[test]
fn test_quality_001_04_owned_self_consuming() {
    let code = r#"
pub struct Builder {
    value: String,
}

impl Builder {
    pub fn build(self) -> String {
        self.value
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Owned self should remain as `self` (no &)
    assert!(
        rust_code.contains("fn build (self)") || rust_code.contains("fn build ( self )"),
        "Should preserve owned self, got: {}",
        rust_code
    );
}

/// Test 5: Mixed receiver types in same impl
#[test]
fn test_quality_001_05_mixed_receiver_types() {
    let code = r#"
pub struct State {
    value: i32,
}

impl State {
    pub fn new() -> Self {
        State { value: 0 }
    }

    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, value: i32) {
        self.value = value;
    }

    pub fn consume(self) -> i32 {
        self.value
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Should have all three receiver types
    assert!(
        rust_code.contains("& self") || rust_code.contains("&self"),
        "Should preserve &self in get(), got: {}",
        rust_code
    );

    assert!(
        rust_code.contains("& mut self") || rust_code.contains("&mut self"),
        "Should preserve &mut self in set(), got: {}",
        rust_code
    );

    assert!(
        rust_code.contains("fn consume (self)") || rust_code.contains("fn consume ( self )"),
        "Should preserve owned self in consume(), got: {}",
        rust_code
    );
}

/// Test 6: Issue #137 reproduction - ruchy-lambda pattern
#[test]
fn test_quality_001_06_issue_137_lambda_pattern() {
    let code = r#"
use std::net::TcpStream;

pub struct LambdaRuntime {
    endpoint: String,
}

impl LambdaRuntime {
    pub fn new(endpoint: String) -> Self {
        LambdaRuntime { endpoint }
    }

    pub fn invoke(&self) -> bool {
        let stream = TcpStream::connect(&self.endpoint);
        stream.is_ok()
    }

    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Issue #137 pattern should work, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Both invoke() and get_endpoint() must use &self
    assert!(
        rust_code.contains("& self") || rust_code.contains("&self"),
        "BLOCKER: &self must be preserved for ruchy-lambda, got: {}",
        rust_code
    );

    // Verify it compiles
    std::fs::write("/tmp/quality_001_06_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/quality_001_06_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: ruchy-lambda pattern fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}
