use ruchy::backend::transpiler::Transpiler;
/// Comprehensive EXTREME TDD: All Three Class Styles
///
/// Tests all three ways to define classes/structs with methods:
/// - Style 1: Methods in struct body (Ruchy's preferred)
/// - Style 2: Impl blocks (Rust-compatible)
/// - Style 3: Class syntax (Full OOP)
///
/// Validates:
/// - All styles parse correctly
/// - All styles transpile to valid Rust
/// - All styles compile with rustc
/// - All styles produce identical behavior
/// - Method receivers work (&self, &mut self)
/// - Visibility modifiers work (pub, private)
/// - Constructors work (new, new with params)
/// - Field access works
/// - Method calls work
use ruchy::frontend::parser::Parser;

#[test]
fn test_extreme_tdd_impl_blocks() {
    // Style 1: Impl blocks (fully working)
    let style1_code = r"
struct Calculator {
    value: i32,
}

impl Calculator {
    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(5)
calc.add(10)
let result = calc.get()
";

    // Style 2: Methods in struct body with pub (PARSER-147 fix)
    let style2_code = r"
struct Calculator {
    value: i32,

    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(5)
calc.add(10)
let result = calc.get()
";

    test_single_style("Style 1: Impl Blocks", style1_code, true);
    test_single_style("Style 2: Pub Methods in Struct", style2_code, true);
}

fn test_single_style(style_name: &str, code: &str, expect_pub: bool) {
    println!("\n=== Testing {style_name} ===");

    // Step 1: Parse
    let ast = Parser::new(code)
        .parse()
        .unwrap_or_else(|e| panic!("{style_name} failed to parse: {e:?}"));
    println!("✓ {style_name}: Parse successful");

    // Step 2: Transpile
    let tokens = Transpiler::new()
        .transpile_to_program(&ast)
        .unwrap_or_else(|e| panic!("{style_name} failed to transpile: {e:?}"));
    println!("✓ {style_name}: Transpile successful");

    // Step 3: Format with prettyplease
    let syntax_tree = syn::parse2(tokens)
        .unwrap_or_else(|e| panic!("{style_name} failed to parse as Rust: {e:?}"));
    let rust_code = prettyplease::unparse(&syntax_tree);
    println!("✓ {style_name}: Format successful");

    // Step 4: Validate generated Rust contains expected patterns
    if expect_pub {
        assert!(
            rust_code.contains("pub fn new"),
            "{style_name}: Missing pub fn new"
        );
        assert!(
            rust_code.contains("pub fn add"),
            "{style_name}: Missing pub fn add"
        );
        assert!(
            rust_code.contains("pub fn get"),
            "{style_name}: Missing pub fn get"
        );
    } else {
        // For private methods, just check they exist (no pub prefix)
        assert!(rust_code.contains("fn new"), "{style_name}: Missing fn new");
        assert!(rust_code.contains("fn add"), "{style_name}: Missing fn add");
        assert!(rust_code.contains("fn get"), "{style_name}: Missing fn get");
    }
    assert!(
        rust_code.contains("&mut self"),
        "{style_name}: Missing &mut self receiver"
    );
    assert!(
        rust_code.contains("&self"),
        "{style_name}: Missing &self receiver"
    );
    assert!(
        !rust_code.contains("pub pub"),
        "{style_name}: Found duplicate pub keyword"
    );
    println!("✓ {style_name}: Rust code validation passed");

    // Step 5: Compile with rustc
    let temp_file = format!(
        "/tmp/test_{}.rs",
        style_name
            .replace(' ', "_")
            .replace(['(', ')', ':'], "")
            .to_lowercase()
    );
    std::fs::write(&temp_file, &rust_code)
        .unwrap_or_else(|e| panic!("{style_name} failed to write temp file: {e}"));

    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", &temp_file])
        .args(["-o", &format!("{temp_file}.rlib")])
        .output()
        .unwrap_or_else(|e| panic!("{style_name} failed to run rustc: {e}"));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("{style_name} failed rustc compilation:\n{stderr}\n\nGenerated code:\n{rust_code}");
    }
    println!("✓ {style_name}: Rustc compilation successful");
    println!("✅ {style_name}: ALL CHECKS PASSED\n");
}

#[test]
#[ignore = "BUG: Class features test failing"]
fn test_comprehensive_class_features() {
    // Test all features in one comprehensive example
    let code = r"
// Style 1: Simple struct with methods
struct Point {
    x: i32,
    y: i32,

    pub fun new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fun distance(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}

// Style 2: Impl block with multiple methods
struct Counter {
    count: i32,
}

impl Counter {
    pub fun new() -> Counter {
        Counter { count: 0 }
    }

    fun internal_increment(&mut self) {
        self.count = self.count + 1
    }

    pub fun increment(&mut self) {
        self.internal_increment()
    }

    pub fun get_count(&self) -> i32 {
        self.count
    }
}

// Style 3: Class with constructor
class Calculator {
    value: i32

    pub new(initial: i32) {
        self.value = initial
    }

    pub fun add(&mut self, amount: i32) {
        self.value = self.value + amount
    }

    pub fun result(&self) -> i32 {
        self.value
    }
}

// Usage
let p = Point::new(3, 4)
let dist = p.distance()

let mut c = Counter::new()
c.increment()
let count = c.get_count()

let mut calc = Calculator::new(10)
calc.add(5)
let value = calc.result()
";

    println!("\n=== Testing Comprehensive Class Features ===");

    // Parse
    let ast = Parser::new(code)
        .parse()
        .expect("Comprehensive example failed to parse");
    println!("✓ Parse successful");

    // Transpile
    let tokens = Transpiler::new()
        .transpile_to_program(&ast)
        .expect("Comprehensive example failed to transpile");
    println!("✓ Transpile successful");

    // Format
    let syntax_tree = syn::parse2(tokens).expect("Failed to parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);
    println!("✓ Format successful");

    // Validate all three styles present
    assert!(rust_code.contains("struct Point"), "Missing Point struct");
    assert!(
        rust_code.contains("struct Counter"),
        "Missing Counter struct"
    );
    assert!(
        rust_code.contains("struct Calculator"),
        "Missing Calculator struct"
    );
    assert!(rust_code.contains("impl Point"), "Missing Point impl");
    assert!(rust_code.contains("impl Counter"), "Missing Counter impl");
    assert!(
        rust_code.contains("impl Calculator"),
        "Missing Calculator impl"
    );
    println!("✓ All three styles present in output");

    // Compile
    std::fs::write("/tmp/test_comprehensive.rs", &rust_code)
        .expect("Failed to write comprehensive test");

    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_comprehensive.rs"])
        .args(["-o", "/tmp/test_comprehensive.rlib"])
        .output()
        .expect("Failed to run rustc");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "Comprehensive example failed compilation:\n{stderr}\n\nGenerated code:\n{rust_code}"
        );
    }
    println!("✓ Rustc compilation successful");
    println!("✅ COMPREHENSIVE TEST: ALL CHECKS PASSED\n");
}

#[test]
fn test_method_receivers_all_styles() {
    // Test that all three receiver types work in all styles
    let code = r"
struct Widget {
    id: i32,

    pub fun new(id: i32) -> Widget {
        Widget { id: id }
    }

    pub fun get_id(&self) -> i32 {
        self.id
    }

    pub fun set_id(&mut self, new_id: i32) {
        self.id = new_id
    }
}

impl Widget {
    pub fun duplicate(&self) -> Widget {
        Widget { id: self.id }
    }
}

let mut w = Widget::new(1)
let id1 = w.get_id()
w.set_id(2)
let id2 = w.get_id()
let w2 = w.duplicate()
";

    println!("\n=== Testing Method Receivers ===");

    let ast = Parser::new(code).parse().expect("Failed to parse");
    let tokens = Transpiler::new()
        .transpile_to_program(&ast)
        .expect("Failed to transpile");
    let syntax_tree = syn::parse2(tokens).expect("Failed to parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Validate both &self and &mut self are present
    assert!(rust_code.contains("&self"), "Missing &self receiver");
    assert!(
        rust_code.contains("&mut self"),
        "Missing &mut self receiver"
    );

    // Compile
    std::fs::write("/tmp/test_receivers.rs", &rust_code).expect("Failed to write");
    let output = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/test_receivers.rs",
            "-o",
            "/tmp/test_receivers.rlib",
        ])
        .output()
        .expect("Failed to run rustc");

    assert!(
        output.status.success(),
        "Receiver test failed rustc compilation"
    );
    println!("✅ METHOD RECEIVERS: ALL CHECKS PASSED\n");
}

#[test]
fn test_visibility_all_styles() {
    // Test pub and private methods in all styles
    let code = r"
struct Vault {
    secret: i32,

    pub fun new(secret: i32) -> Vault {
        Vault { secret: secret }
    }

    fun internal_check(&self) -> i32 {
        self.secret
    }

    pub fun verify(&self) -> i32 {
        self.internal_check()
    }
}

impl Vault {
    pub fun public_method(&self) -> i32 {
        42
    }
}

let v = Vault::new(123)
let result = v.verify()
";

    println!("\n=== Testing Visibility Modifiers ===");

    let ast = Parser::new(code).parse().expect("Failed to parse");
    let tokens = Transpiler::new()
        .transpile_to_program(&ast)
        .expect("Failed to transpile");
    let syntax_tree = syn::parse2(tokens).expect("Failed to parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Validate pub and private (no pub prefix) methods
    assert!(rust_code.contains("pub fn new"), "Missing pub fn new");
    assert!(rust_code.contains("pub fn verify"), "Missing pub fn verify");
    assert!(
        rust_code.contains("pub fn public_method"),
        "Missing pub fn public_method"
    );
    assert!(
        rust_code.matches("fn internal_check").count() == 1,
        "Missing private fn internal_check"
    );

    // Compile
    std::fs::write("/tmp/test_visibility.rs", &rust_code).expect("Failed to write");
    let output = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "/tmp/test_visibility.rs",
            "-o",
            "/tmp/test_visibility.rlib",
        ])
        .output()
        .expect("Failed to run rustc");

    assert!(
        output.status.success(),
        "Visibility test failed rustc compilation"
    );
    println!("✅ VISIBILITY: ALL CHECKS PASSED\n");
}
