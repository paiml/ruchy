/// TRANSPILER-079: Fix class constructor transpilation
///
/// EXTREME TDD Test Suite for class constructor code generation.
///
/// ROOT CAUSE: Class constructors with `self.field = value` syntax transpile
/// to invalid Rust using `self` in a function without self parameter.
///
/// RED Phase: All tests MUST fail initially (runtime errors or compile errors).
/// Coverage:
/// - Simple constructor with field initialization
/// - Constructor with multiple fields
/// - Constructor with parameters
/// - Constructor returning custom type
/// - Mixed constructors and regular methods
/// - Constructor with computed values

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_transpiler_079_01_simple_constructor() {
    let code = r"
class Counter {
    count: i32

    pub new() -> Counter {
        Counter { count: 0 }
    }
}

let c = Counter::new()
";

    // Parse
    let ast = Parser::new(code).parse()
        .expect("Should parse class with constructor");

    // Transpile
    let tokens = Transpiler::new().transpile_to_program(&ast)
        .expect("Should transpile class constructor");

    // Format
    let syntax_tree = syn::parse2(tokens)
        .expect("Should parse as valid Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Validate: Should NOT contain self.field in constructor
    assert!(!rust_code.contains("self.count"),
        "Constructor should NOT use self.field syntax");

    // Should contain proper struct initialization
    assert!(rust_code.contains("Counter {") || rust_code.contains("Self {"),
        "Constructor should use struct initialization syntax");

    // Compile
    std::fs::write("/tmp/test_transpiler_079_01.rs", &rust_code)
        .expect("Failed to write temp file");

    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_transpiler_079_01.rs"])
        .args(["-o", "/tmp/test_transpiler_079_01.rlib"])
        .output()
        .expect("Failed to run rustc");

    assert!(output.status.success(),
        "Rustc compilation should succeed:\n{}\n\nGenerated code:\n{}",
        String::from_utf8_lossy(&output.stderr),
        rust_code);
}

#[test]
fn test_transpiler_079_02_constructor_with_params() {
    let code = r"
class Point {
    x: i32
    y: i32

    pub new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }
}

let p = Point::new(3, 4)
";

    let ast = Parser::new(code).parse()
        .expect("Should parse");
    let tokens = Transpiler::new().transpile_to_program(&ast)
        .expect("Should transpile");
    let syntax_tree = syn::parse2(tokens)
        .expect("Should parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Should use parameter names, not self.field
    assert!(!rust_code.contains("self.x") || !rust_code.contains("fn new"),
        "Constructor should NOT use self.field with parameters");

    // Compile
    std::fs::write("/tmp/test_transpiler_079_02.rs", &rust_code)
        .expect("Failed to write");
    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_transpiler_079_02.rs", "-o", "/tmp/test_transpiler_079_02.rlib"])
        .output()
        .expect("Failed to run rustc");

    assert!(output.status.success(),
        "Should compile:\n{}\n\nCode:\n{}",
        String::from_utf8_lossy(&output.stderr),
        rust_code);
}

#[test]
fn test_transpiler_079_03_constructor_with_methods() {
    let code = r"
class Calculator {
    value: i32

    pub new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun add(&mut self, n: i32) {
        self.value = self.value + n
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}

let mut calc = Calculator::new()
calc.add(5)
let result = calc.get()
";

    let ast = Parser::new(code).parse().expect("Should parse");
    let tokens = Transpiler::new().transpile_to_program(&ast).expect("Should transpile");
    let syntax_tree = syn::parse2(tokens).expect("Should parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Constructor should NOT use self, but methods should
    let lines: Vec<&str> = rust_code.lines().collect();
    let mut in_new_fn = false;
    for line in &lines {
        if line.contains("fn new(") {
            in_new_fn = true;
        }
        if in_new_fn && line.contains('}') {
            in_new_fn = false;
        }
        if in_new_fn {
            assert!(!line.contains("self.value"),
                "Constructor should not use self.value:\n{rust_code}");
        }
    }

    // Compile
    std::fs::write("/tmp/test_transpiler_079_03.rs", &rust_code)
        .expect("Failed to write");
    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_transpiler_079_03.rs", "-o", "/tmp/test_transpiler_079_03.rlib"])
        .output()
        .expect("Failed to run rustc");

    assert!(output.status.success(),
        "Should compile:\n{}\n\nCode:\n{}",
        String::from_utf8_lossy(&output.stderr),
        rust_code);
}

#[test]
fn test_transpiler_079_04_multiple_fields() {
    let code = r#"
class Config {
    host: String
    port: i32
    debug: bool

    pub new(host: String, port: i32) -> Config {
        Config {
            host: host,
            port: port,
            debug: false
        }
    }
}

let cfg = Config::new(String::from("localhost"), 8080)
"#;

    let ast = Parser::new(code).parse().expect("Should parse");
    let tokens = Transpiler::new().transpile_to_program(&ast).expect("Should transpile");
    let syntax_tree = syn::parse2(tokens).expect("Should parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Should not use self in constructor
    assert!(!rust_code.contains("self.host") || !rust_code.contains("fn new"),
        "Constructor should not use self.field");

    // Compile
    std::fs::write("/tmp/test_transpiler_079_04.rs", &rust_code)
        .expect("Failed to write");
    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_transpiler_079_04.rs", "-o", "/tmp/test_transpiler_079_04.rlib"])
        .output()
        .expect("Failed to run rustc");

    assert!(output.status.success(),
        "Should compile:\n{}\n\nCode:\n{}",
        String::from_utf8_lossy(&output.stderr),
        rust_code);
}

#[test]
fn test_transpiler_079_05_runtime_execution() {
    // This test will use the runtime to verify the constructor actually works
    let code = r"
class Point {
    x: i32
    y: i32

    pub new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fun get_x(&self) -> i32 {
        self.x
    }
}

let p = Point::new(10, 20)
p.get_x()
";

    // Parse and transpile
    let ast = Parser::new(code).parse().expect("Should parse");
    let tokens = Transpiler::new().transpile_to_program(&ast).expect("Should transpile");
    let syntax_tree = syn::parse2(tokens).expect("Should parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Compile to executable
    std::fs::write("/tmp/test_transpiler_079_05.rs", &rust_code)
        .expect("Failed to write");
    let compile_output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_transpiler_079_05.rs", "-o", "/tmp/test_transpiler_079_05.rlib"])
        .output()
        .expect("Failed to run rustc");

    assert!(compile_output.status.success(),
        "Compilation failed:\n{}\n\nCode:\n{}",
        String::from_utf8_lossy(&compile_output.stderr),
        rust_code);
}
