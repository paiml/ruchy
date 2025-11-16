/// PARSER-147: Support pub fun in struct bodies
///
/// EXTREME TDD Test Suite for pub method parsing in struct definitions.
///
/// RED Phase: All tests MUST fail initially to prove they test the actual bug.
/// Coverage:
/// - pub fun with no params (constructor)
/// - pub fun with &self receiver
/// - pub fun with &mut self receiver
/// - pub fun with params
/// - Mixed pub/private methods
/// - pub fun with return types
/// - Multiple pub methods
/// - pub fun with body blocks

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_parser_147_01_pub_fun_constructor() {
    let code = r"
struct Calculator {
    value: i32,

    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }
}
";

    // RED: This MUST fail initially
    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Should parse pub fun constructor in struct body");
}

#[test]
fn test_parser_147_02_pub_fun_with_self() {
    let code = r"
struct Point {
    x: i32,

    pub fun get_x(&self) -> i32 {
        self.x
    }
}
";

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Should parse pub fun with &self");
}

#[test]
fn test_parser_147_03_pub_fun_with_mut_self() {
    let code = r"
struct Counter {
    count: i32,

    pub fun increment(&mut self) {
        self.count = self.count + 1
    }
}
";

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Should parse pub fun with &mut self");
}

#[test]
fn test_parser_147_04_mixed_pub_private_methods() {
    let code = r"
struct Widget {
    id: i32,

    fun internal_id(&self) -> i32 {
        self.id
    }

    pub fun get_id(&self) -> i32 {
        self.internal_id()
    }
}
";

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Should parse mixed pub/private methods");
}

#[test]
fn test_parser_147_05_multiple_pub_methods() {
    let code = r"
struct Calculator {
    value: i32,

    pub fun add(&mut self, n: i32) {
        self.value = self.value + n
    }

    pub fun subtract(&mut self, n: i32) {
        self.value = self.value - n
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}
";

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Should parse multiple pub methods");
}

#[test]
fn test_parser_147_06_pub_fun_with_params() {
    let code = r"
struct Math {
    pub fun add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}
";

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Should parse pub fun with multiple params");
}

#[test]
fn test_parser_147_07_transpile_pub_methods() {
    let code = r"
struct Calculator {
    value: i32,

    pub fun new() -> Calculator {
        Calculator { value: 0 }
    }

    pub fun get(&self) -> i32 {
        self.value
    }
}
";

    // Parse
    let ast = Parser::new(code).parse()
        .expect("Should parse pub methods in struct");

    // Transpile
    let tokens = Transpiler::new().transpile_to_program(&ast)
        .expect("Should transpile pub methods");

    // Format
    let syntax_tree = syn::parse2(tokens)
        .expect("Should parse as valid Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Validate: Should generate "pub fn" not "pub pub fn"
    assert!(rust_code.contains("pub fn new"), "Should generate pub fn new");
    assert!(rust_code.contains("pub fn get"), "Should generate pub fn get");
    assert!(!rust_code.contains("pub pub"), "Should NOT duplicate pub keyword");
}

#[test]
fn test_parser_147_08_compile_and_execute() {
    let code = r"
struct Point {
    x: i32,
    y: i32,

    pub fun new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fun get_x(&self) -> i32 {
        self.x
    }
}

let p = Point::new(3, 4)
let x = p.get_x()
";

    // Parse
    let ast = Parser::new(code).parse()
        .expect("Should parse");

    // Transpile
    let tokens = Transpiler::new().transpile_to_program(&ast)
        .expect("Should transpile");

    // Format
    let syntax_tree = syn::parse2(tokens)
        .expect("Should parse as Rust");
    let rust_code = prettyplease::unparse(&syntax_tree);

    // Compile with rustc
    std::fs::write("/tmp/test_parser_147_compile.rs", &rust_code)
        .expect("Failed to write temp file");

    let output = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/test_parser_147_compile.rs"])
        .args(["-o", "/tmp/test_parser_147_compile.rlib"])
        .output()
        .expect("Failed to run rustc");

    assert!(output.status.success(),
        "Rustc compilation should succeed:\n{}\n\nGenerated code:\n{}",
        String::from_utf8_lossy(&output.stderr),
        rust_code);
}
