use ruchy::backend::transpiler::Transpiler;
/// TRANSPILER-009: Standalone functions disappear
///
/// Root Cause: When transpiling programs with `fun main()` + standalone functions,
/// the standalone functions completely disappear from transpiled Rust output.
///
/// Example:
/// ```ruchy
/// fun square(n: i32) -> i32 { n * n }
/// fun main() {
///     let result = square(5);
///     println!("Result: {}", result);
/// }
/// ```
///
/// Current (BROKEN) output: Only `main()` appears, `square()` is missing
use ruchy::frontend::parser::Parser;

/// RED Test 1: Standalone function must appear in transpiled output
#[test]
fn test_transpiler_009_01_standalone_function_appears() {
    let code = r#"
fun square(n: i32) -> i32 {
    n * n
}

fun main() {
    let result = square(5);
    println!("5 squared = {}", result);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile(&ast)
        .expect("Transpilation failed")
        .to_string();

    println!("=== Transpiled Rust code ===");
    println!("{rust_code}");
    println!("============================");

    // BUG: This will FAIL because square() disappears
    assert!(
        rust_code.contains("fn square"),
        "TRANSPILER-009: Standalone function 'square' missing from output!\nGot:\n{rust_code}"
    );
}

/// RED Test 2: Main body must be complete
#[test]
fn test_transpiler_009_02_main_body_complete() {
    let code = r#"
fun square(n: i32) -> i32 {
    n * n
}

fun main() {
    let result = square(5);
    println!("5 squared = {}", result);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile(&ast)
        .expect("Transpilation failed")
        .to_string();

    println!("=== Main body ===");
    println!("{rust_code}");
    println!("=================");

    // BUG: This will FAIL because main body is truncated
    assert!(
        rust_code.contains("let result"),
        "TRANSPILER-009: Main body missing 'let result' statement!\nGot:\n{rust_code}"
    );
}

/// RED Test 3: Transpiled code must compile
#[test]
fn test_transpiler_009_03_output_compiles() {
    let code = r#"
fun square(n: i32) -> i32 {
    n * n
}

fun main() {
    let result = square(5);
    println!("5 squared = {}", result);
}
"#;

    let ast = Parser::new(code).parse().expect("Parse failed");
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile(&ast)
        .expect("Transpilation failed")
        .to_string();

    // Write to temp file
    std::fs::write("/tmp/test_standalone_output.rs", &rust_code)
        .expect("Failed to write temp file");

    // Compile with rustc
    let output = std::process::Command::new("rustc")
        .arg("/tmp/test_standalone_output.rs")
        .arg("-o")
        .arg("/tmp/test_standalone_binary")
        .output()
        .expect("Failed to execute rustc");

    // BUG: This will FAIL because square() is missing
    assert!(output.status.success(),
            "TRANSPILER-009: Code failed to compile (missing square function)!\n\nRust:\n{}\n\nErrors:\n{}",
            rust_code,
            String::from_utf8_lossy(&output.stderr)
        );
}
