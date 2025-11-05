/// TRANSPILER-008 (actually PARSER-008): pub visibility lost (pub fun → fn)
///
/// GitHub Issue: #140 (ruchy-lambda blocker)
/// Blocks: ruchy-lambda v3.207.0 library visibility
///
/// BUG: `pub fun new()` → `fn new()` - pub keyword lost in impl methods
/// IMPACT: Library methods not accessible (private by default)
/// ROOT CAUSE: Parser checks for `pub` keyword (line 170) but doesn't store the flag
/// FIX: Pass is_pub from parse_impl_methods() to parse_impl_method()

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

/// Test 1: pub fun should become pub fn
#[test]
fn test_transpiler_008_01_pub_fun_preserved() {
    let code = r#"
pub struct Library {
    data: i32,
}

impl Library {
    pub fun new() -> Library {
        Library { data: 0 }
    }

    pub fun get(&self) -> i32 {
        self.data
    }

    fun internal(&self) -> i32 {
        self.data * 2
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // pub fun → pub fn
    assert!(
        rust_code.contains("pub fn new"),
        "BUG: pub fun new() should become pub fn new():\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("pub fn get"),
        "BUG: pub fun get() should become pub fn get():\n{}",
        rust_code
    );

    // fun (no pub) → fn (no pub)
    assert!(
        rust_code.contains("fn internal") && !rust_code.contains("pub fn internal"),
        "Private method should remain private:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_008_01_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_008_01_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: pub methods should compile as library:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 2: Mixed pub and private methods
#[test]
fn test_transpiler_008_02_mixed_visibility() {
    let code = r#"
pub struct API {
    token: String,
}

impl API {
    pub fun new(token: String) -> API {
        API { token }
    }

    pub fun authenticate(&self) -> bool {
        true
    }

    fun validate_token(&self) -> bool {
        self.token.len() > 0
    }

    pub fun get_token(&self) -> String {
        self.token.clone()
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Count pub fn occurrences
    let pub_fn_count = rust_code.matches("pub fn").count();
    assert!(
        pub_fn_count >= 3,
        "Should have at least 3 pub fn methods (new, authenticate, get_token):\n{}",
        rust_code
    );

    // Verify private method
    assert!(
        rust_code.contains("fn validate_token") && !rust_code.contains("pub fn validate_token"),
        "validate_token should be private:\n{}",
        rust_code
    );
}

/// Test 3: Default visibility (no pub) should be private
#[test]
fn test_transpiler_008_03_default_private() {
    let code = r#"
pub struct Internal {
    value: i32,
}

impl Internal {
    fun helper(&self) -> i32 {
        self.value
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Should NOT have pub fn (default is private)
    assert!(
        !rust_code.contains("pub fn helper"),
        "Default visibility should be private:\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("fn helper"),
        "Method should still be generated:\n{}",
        rust_code
    );
}

/// Test 4: ruchy-lambda Calculator example (from TEST-RESULTS-v3.207.0.md)
#[test]
fn test_transpiler_008_04_calculator_pub_methods() {
    let code = r#"
pub struct Calculator {
    value: i32,
}

impl Calculator {
    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount;
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // All methods should be public
    assert!(
        rust_code.contains("pub fn new"),
        "Calculator::new() should be public:\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("pub fn add"),
        "Calculator::add() should be public:\n{}",
        rust_code
    );

    assert!(
        rust_code.contains("pub fn get"),
        "Calculator::get() should be public:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_008_04_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_008_04_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "Calculator (ruchy-lambda example) should compile:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}
