//! PARSER-095: Grouped imports validation (Issue #137 - ruchy-lambda)
//!
//! DISCOVERY: Grouped imports ALREADY WORK in parser and transpiler!
//!
//! PURPOSE: Document + prevent regressions for this critical ruchy-lambda feature
//!
//! PATTERNS TESTED:
//! - Basic grouped imports: `use std::io::{Read, Write};`
//! - Multiple stdlib modules with groups
//! - User modules with grouped imports
//! - Mixed single and grouped imports
//!
//! EXTREME TDD: All tests GREEN (feature pre-existing, adding coverage)

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

/// Test 1: Basic grouped imports (`std::io`)
/// Pattern: `use std::io::{Read, Write};`
/// Expected: Transpiles to correct Rust grouped syntax
#[test]
fn test_parser_095_01_basic_grouped_imports() {
    let code = r"
use std::io::{Read, Write};

pub fn process() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile grouped imports, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Should preserve grouped import syntax
    assert!(
        rust_code.contains("use std :: io :: { Read , Write }"),
        "Should preserve grouped import, got: {rust_code}"
    );

    // Should NOT expand to separate imports
    assert!(
        !rust_code.contains("use std::io::Read;\nuse std::io::Write;"),
        "Should NOT expand grouped imports, got: {rust_code}"
    );
}

/// Test 2: Collections grouped imports
#[test]
fn test_parser_095_02_collections_grouped() {
    let code = r"
use std::collections::{HashMap, HashSet};

pub fn create() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("use std :: collections :: { HashMap , HashSet }"),
        "Should preserve collections grouped import, got: {rust_code}"
    );
}

/// Test 3: Multiple items in group (sync types)
#[test]
fn test_parser_095_03_multiple_sync_types() {
    let code = r"
use std::sync::{Arc, Mutex, RwLock};

pub fn shared() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("use std :: sync :: { Arc , Mutex , RwLock }"),
        "Should preserve all items in group, got: {rust_code}"
    );
}

/// Test 4: Multiple grouped imports in same file
#[test]
fn test_parser_095_04_multiple_grouped_imports() {
    let code = r"
use std::io::{Read, Write};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub fn combined() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // All three grouped imports should be preserved
    assert!(
        rust_code.contains("use std :: io :: { Read , Write }"),
        "Should preserve io imports, got: {rust_code}"
    );

    assert!(
        rust_code.contains("use std :: collections :: { HashMap , HashSet }"),
        "Should preserve collections imports, got: {rust_code}"
    );

    assert!(
        rust_code.contains("use std :: sync :: { Arc , Mutex }"),
        "Should preserve sync imports, got: {rust_code}"
    );
}

/// Test 5: User modules with grouped imports
#[test]
fn test_parser_095_05_user_module_grouped() {
    let code = r"
use http_client::{get, post};

pub fn api_call() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Should transpile user modules, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    assert!(
        rust_code.contains("use http_client :: { get , post }"),
        "Should preserve user module grouped import, got: {rust_code}"
    );
}

/// Test 6: Mixed single and grouped imports
#[test]
fn test_parser_095_06_mixed_imports() {
    let code = r"
use std::net::TcpStream;
use std::io::{Read, Write};
use std::collections::HashMap;

pub fn network() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Single imports preserved
    assert!(
        rust_code.contains("use std :: net :: TcpStream"),
        "Should preserve single import, got: {rust_code}"
    );

    // Grouped import preserved
    assert!(
        rust_code.contains("use std :: io :: { Read , Write }"),
        "Should preserve grouped import, got: {rust_code}"
    );

    // Collections type preserved
    assert!(
        rust_code.contains("use std :: collections :: HashMap"),
        "Should preserve collections type, got: {rust_code}"
    );
}

/// Test 7: Issue #137 reproduction (ruchy-lambda pattern)
#[test]
fn test_parser_095_07_issue_137_ruchy_lambda() {
    let code = r"
use std::net::TcpStream;
use std::io::{Read, Write};

pub struct LambdaRuntime {
    endpoint: String,
}

impl LambdaRuntime {
    pub fn new(endpoint: String) -> Self {
        LambdaRuntime { endpoint }
    }

    pub fn invoke(&self) -> bool {
        // Use TcpStream for HTTP requests
        let stream = TcpStream::connect(&self.endpoint);
        stream.is_ok()
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "ruchy-lambda pattern should work, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Critical: Both import styles work together
    assert!(
        rust_code.contains("use std :: net :: TcpStream"),
        "Should preserve single import, got: {rust_code}"
    );

    assert!(
        rust_code.contains("use std :: io :: { Read , Write }"),
        "Should preserve grouped import, got: {rust_code}"
    );

    // Should have struct and impl
    assert!(
        rust_code.contains("pub struct LambdaRuntime"),
        "Should preserve struct, got: {rust_code}"
    );

    assert!(
        rust_code.contains("impl LambdaRuntime"),
        "Should preserve impl, got: {rust_code}"
    );
}

/// Test 8: Single-item group optimization
/// Transpiler optimizes single-item groups to simple imports (smart behavior)
#[test]
fn test_parser_095_08_single_item_group_optimization() {
    let code = r"
use std::io::{Read};

pub fn read_only() -> bool {
    true
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile(&ast);

    assert!(
        result.is_ok(),
        "Single item group should work, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Transpiler optimizes single-item groups to simple imports
    assert!(
        rust_code.contains("use std :: io :: Read"),
        "Should optimize single-item group to simple import, got: {rust_code}"
    );

    // Should NOT keep braces for single item
    assert!(
        !rust_code.contains("{ Read }"),
        "Should NOT preserve braces for single item, got: {rust_code}"
    );
}
