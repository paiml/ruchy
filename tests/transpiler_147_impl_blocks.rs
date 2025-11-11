/// TRANSPILER-147: Impl block support with pub visibility
///
/// GitHub Issue: #147 - "pub pub fn" bug (actually: impl blocks not supported)
/// Impact: BLOCKER - all impl blocks fail to parse
/// Root Cause: Parser stub at expressions_helpers/impls.rs:42 just bails
/// Fix: Implement full impl block parsing

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

/// Test 1: Basic impl block with pub fun
#[test]
fn test_transpiler_147_01_basic_impl_pub_fun() {
    let code = r#"
pub struct Runtime {
    api_endpoint: String,
}

impl Runtime {
    pub fun new() -> Runtime {
        let endpoint = String::from("127.0.0.1:9001");
        Runtime { api_endpoint: endpoint }
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // CRITICAL: Should generate "pub fn" NOT "pub pub fn"
    assert!(
        !rust_code.contains("pub pub fn"),
        "BUG: Generated duplicate pub keyword:\n{}",
        rust_code
    );

    // Should have single pub fn
    assert!(
        rust_code.contains("pub fn new"),
        "Should have 'pub fn new':\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_147_01_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_147_01_output.rs"])
        .args(["-o", "/tmp/transpiler_147_01_output.rlib"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Impl block fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 2: Impl block with multiple pub methods
#[test]
fn test_transpiler_147_02_multiple_pub_methods() {
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

    // NO duplicate pub keywords
    assert!(
        !rust_code.contains("pub pub"),
        "BUG: Found duplicate pub:\n{}",
        rust_code
    );

    // All three methods should be pub fn
    assert!(rust_code.contains("pub fn new"), "Missing pub fn new");
    assert!(rust_code.contains("pub fn add"), "Missing pub fn add");
    assert!(rust_code.contains("pub fn get"), "Missing pub fn get");

    // Verify compilation
    std::fs::write("/tmp/transpiler_147_02_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_147_02_output.rs"])
        .args(["-o", "/tmp/transpiler_147_02_output.rlib"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Multiple methods fail compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 3: Impl block with mixed pub/private visibility
#[test]
fn test_transpiler_147_03_mixed_visibility() {
    let code = r#"
struct Counter {
    count: i32,
}

impl Counter {
    pub fun new() -> Counter {
        Counter { count: 0 }
    }

    fun internal_increment(&mut self) {
        self.count = self.count + 1;
    }

    pub fun increment(&mut self) {
        self.internal_increment();
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // NO duplicate pub
    assert!(!rust_code.contains("pub pub"), "Found duplicate pub");

    // Public methods
    assert!(rust_code.contains("pub fn new"), "Missing pub fn new");
    assert!(rust_code.contains("pub fn increment"), "Missing pub fn increment");

    // Private method (no pub prefix)
    assert!(rust_code.matches("fn internal_increment").count() == 1, "Should have private fn internal_increment");

    // Verify compilation
    std::fs::write("/tmp/transpiler_147_03_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_147_03_output.rs"])
        .args(["-o", "/tmp/transpiler_147_03_output.rlib"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Mixed visibility fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 4: Impl block with self receivers
#[test]
fn test_transpiler_147_04_self_receivers() {
    let code = r#"
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fun new(x: i32, y: i32) -> Point {
        Point { x, y }
    }

    pub fun distance(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }

    pub fun move_by(&mut self, dx: i32, dy: i32) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // NO duplicate pub
    assert!(!rust_code.contains("pub pub"), "Found duplicate pub");

    // All methods should be pub fn
    assert!(rust_code.contains("pub fn new"), "Missing pub fn new");
    assert!(rust_code.contains("pub fn distance"), "Missing pub fn distance");
    assert!(rust_code.contains("pub fn move_by"), "Missing pub fn move_by");

    // Self receivers
    assert!(rust_code.contains("&self"), "Missing &self receiver");
    assert!(rust_code.contains("&mut self"), "Missing &mut self receiver");

    // Verify compilation
    std::fs::write("/tmp/transpiler_147_04_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_147_04_output.rs"])
        .args(["-o", "/tmp/transpiler_147_04_output.rlib"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Self receivers fail compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}
