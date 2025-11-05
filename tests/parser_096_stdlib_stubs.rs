//! PARSER-096: Disable stdlib stub generation (Issue #137 - ruchy-lambda)
//!
//! BUG: Transpiler generates mock stubs for std:: types, shadowing real implementations.
//! ERROR: `mod net { pub struct TcpStream; impl TcpStream { ... } }` shadows `std::net::TcpStream`
//!
//! IMPACT: Requires manual post-processing to strip stubs. Non-functional mocks break production code.
//!
//! EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

/// Test 1: std::net::TcpStream should NOT generate stub
/// Example: `use std::net::TcpStream;`
/// Should: Use real TcpStream, not generate mock
#[test]
fn test_parser_096_01_no_stub_for_std_net_tcpstream() {
    let code = r#"
use std::net::TcpStream;

pub fn connect(addr: &str) -> bool {
    TcpStream::connect(addr).is_ok()
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile stdlib usage, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Should NOT contain mock TcpStream module
    assert!(
        !rust_code.contains("mod net {"),
        "Should NOT generate mod net stub, got: {}",
        rust_code
    );

    // Should NOT contain mock TcpStream struct
    assert!(
        !rust_code.contains("pub struct TcpStream;"),
        "Should NOT generate TcpStream stub, got: {}",
        rust_code
    );

    // Should contain the use statement
    assert!(
        rust_code.contains("use std :: net :: TcpStream"),
        "Should preserve use statement, got: {}",
        rust_code
    );
}

/// Test 2: std::io types should NOT generate stubs
#[test]
fn test_parser_096_02_no_stub_for_std_io() {
    let code = r#"
use std::io::{Read, Write};

pub fn process() -> bool {
    true
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("mod io {"),
        "Should NOT generate mod io stub, got: {}",
        rust_code
    );

    assert!(
        !rust_code.contains("pub struct Read") && !rust_code.contains("pub trait Read"),
        "Should NOT generate Read stub, got: {}",
        rust_code
    );
}

/// Test 3: Multiple stdlib imports should all skip stub generation
#[test]
fn test_parser_096_03_multiple_stdlib_imports() {
    let code = r#"
use std::net::TcpStream;
use std::io::Read;
use std::fs::File;

pub fn test() -> bool {
    true
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // None of these should have mocks
    assert!(!rust_code.contains("mod net {"), "Should NOT mock std::net");
    assert!(!rust_code.contains("mod io {"), "Should NOT mock std::io");
    assert!(!rust_code.contains("mod fs {"), "Should NOT mock std::fs");
}

/// Test 4: Non-stdlib modules SHOULD still work (user modules)
#[test]
fn test_parser_096_04_user_modules_still_work() {
    let code = r#"
// User's own http_client module (not stdlib)
use http_client::Client;

pub fn test() {
    let client = Client::new();
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile user modules, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // User modules should still work normally
    assert!(
        rust_code.contains("use http_client :: Client"),
        "Should preserve user module import, got: {}",
        rust_code
    );
}

/// Test 5: std::collections should NOT generate stubs
#[test]
fn test_parser_096_05_no_stub_for_std_collections() {
    let code = r#"
use std::collections::HashMap;

pub fn create_map() -> HashMap<String, i32> {
    HashMap::new()
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("mod collections {"),
        "Should NOT generate collections stub, got: {}",
        rust_code
    );

    assert!(
        !rust_code.contains("pub struct HashMap"),
        "Should NOT mock HashMap, got: {}",
        rust_code
    );
}

/// Test 6: Mixing stdlib and user modules
#[test]
fn test_parser_096_06_mixed_stdlib_and_user() {
    let code = r#"
use std::net::TcpStream;
use http_client::Client;

pub fn connect() -> bool {
    let stream = TcpStream::connect("127.0.0.1:8080");
    let client = Client::new();
    stream.is_ok()
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Stdlib should NOT have stubs
    assert!(
        !rust_code.contains("mod net {"),
        "Should NOT mock std::net, got: {}",
        rust_code
    );

    // User module should still work
    assert!(
        rust_code.contains("use http_client :: Client"),
        "Should preserve user module, got: {}",
        rust_code
    );
}

/// Test 7: std::sync types (Mutex, Arc) should NOT generate stubs
#[test]
fn test_parser_096_07_no_stub_for_std_sync() {
    let code = r#"
use std::sync::{Arc, Mutex};

pub fn create_shared() -> Arc<Mutex<i32>> {
    Arc::new(Mutex::new(42))
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("mod sync {"),
        "Should NOT mock std::sync, got: {}",
        rust_code
    );
}

/// Test 8: Issue #137 reproduction - ruchy-lambda TcpStream usage
#[test]
fn test_parser_096_08_issue_137_repro() {
    let code = r#"
use std::net::TcpStream;

pub struct HttpClient {
    endpoint: String,
}

impl HttpClient {
    pub fn new(endpoint: String) -> Self {
        HttpClient { endpoint }
    }

    pub fn get(&self, path: &str) -> bool {
        let addr = format!("{}{}", self.endpoint, path);
        TcpStream::connect(&addr).is_ok()
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

    // CRITICAL: TcpStream must NOT be mocked
    assert!(
        !rust_code.contains("pub struct TcpStream"),
        "BLOCKER: TcpStream stub shadows real implementation, got: {}",
        rust_code
    );

    // Should use real std::net::TcpStream
    assert!(
        rust_code.contains("use std :: net :: TcpStream"),
        "Should use real TcpStream, got: {}",
        rust_code
    );
}
