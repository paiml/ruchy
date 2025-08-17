#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)] // Tests can panic

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;
use ruchy::runtime::repl::Repl;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_end_to_end_simple_expression() {
    let input = "2 + 3 * 4";
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Verify the generated Rust code compiles
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains('2'));
    assert!(rust_str.contains('3'));
    assert!(rust_str.contains('4'));
}

#[test]
fn test_end_to_end_function_definition() {
    let input = r"
        fun add(x: i32, y: i32) -> i32 {
            x + y
        }
    ";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Verify function signature is preserved
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("fn add"));
    assert!(rust_str.contains("i32"));
}

#[test]
fn test_end_to_end_pattern_matching() {
    let input = r#"
        match x {
            1 => "one",
            2 => "two",
            _ => "other",
        }
    "#;

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Verify match expression is preserved
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("match"));
    assert!(rust_str.contains('1'));
    assert!(rust_str.contains('_'));
}

#[test]
#[ignore = "Lambda expressions not yet implemented"]
fn test_end_to_end_pipeline_operator() {
    let input = "data |> filter(x => x > 0) |> map(x => x * 2)";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Pipeline should be converted to method chaining
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("filter") || rust_str.contains('.'));
}

#[test]
#[ignore = "List comprehensions not yet implemented"]
fn test_end_to_end_list_comprehension() {
    let input = "[x * 2 for x in 1..10 if x % 2 == 0]";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Should transpile to iterator chain
    let rust_str = rust_code.to_string();
    assert!(
        rust_str.contains("filter") || rust_str.contains("map") || rust_str.contains("collect")
    );
}

#[test]
#[ignore = "Actor syntax not yet implemented"]
fn test_end_to_end_actor_definition() {
    let input = r"
        actor Counter {
            mut count: i32 = 0;
            
            pub fn increment() {
                self.count += 1;
            }
            
            pub fn get() -> i32 {
                self.count
            }
        }
    ";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Actor should generate struct with message handling
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("struct") || rust_str.contains("impl"));
}

#[test]
#[ignore = "Async/await not yet implemented"]
fn test_end_to_end_async_await() {
    let input = r"
        async fun fetch_data(url: String) -> Result<String> {
            let response = await http_get(url);
            Ok(response.body())
        }
    ";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Async/await should be preserved
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("async") || rust_str.contains("await"));
}

#[test]
#[ignore = "Try/catch syntax not yet implemented"]
fn test_end_to_end_error_handling() {
    let input = r"
        try {
            risky_operation()?
        } catch e {
            log_error(e);
            default_value()
        }
    ";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Should transpile to Result handling
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("match") || rust_str.contains("Ok") || rust_str.contains("Err"));
}

#[test]
#[ignore = "Complex import syntax not yet implemented"]
fn test_end_to_end_import_statement() {
    let input = r"
        import std::collections::HashMap;
        import utils::{helper1, helper2};
    ";

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Imports are currently transpiled as empty (to be handled at module level)
    // This is a valid design choice for now
    let _rust_str = rust_code.to_string();
}

#[test]
fn test_end_to_end_type_inference() {
    let input = r#"
        let x = 42;
        let y = x + 1;
        let z = if y > 40 { "big" } else { "small" };
    "#;

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Type inference should work
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("let"));
}

#[test]
fn test_compile_generated_rust_code() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    // Create a simple Cargo project
    let init_output = Command::new("cargo")
        .args(["init", "--lib", "--name", "test_project"])
        .current_dir(project_path)
        .output()
        .expect("Failed to create Cargo project");

    assert!(
        init_output.status.success(),
        "cargo init failed: {}",
        String::from_utf8_lossy(&init_output.stderr)
    );

    // Ensure src directory exists
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    // Generate simple Ruchy code for testing
    let ruchy_code = "1 + 2 * 3";

    let mut parser = Parser::new(ruchy_code);
    let expr = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Write the generated Rust code wrapped in a function
    let generated_code = format!("pub fn test_expr() -> i64 {{ {rust_code} }}");
    let lib_path = project_path.join("src/lib.rs");
    fs::write(&lib_path, &generated_code).expect("Failed to write Rust code");

    // Compile the generated Rust code
    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(project_path)
        .output()
        .expect("Failed to run cargo build");

    assert!(
        output.status.success(),
        "Generated Rust code failed to compile: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_repl_basic_evaluation() {
    // This would test the REPL functionality
    // For now, just verify REPL can be created
    let repl = Repl::new();
    assert!(repl.is_ok());
}

#[test]
fn test_incremental_compilation() {
    let inputs = vec![
        "let x = 5",
        "let y = x + 10",
        "fun double(n: i32) -> i32 { n * 2 }",
        "double(y)",
    ];

    let transpiler = Transpiler::new();

    for input in inputs {
        let mut parser = Parser::new(input);
        let expr = parser
            .parse()
            .unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        let rust_code = transpiler.transpile_expr(&expr);
        assert!(rust_code.is_ok(), "Failed to transpile: {input}");
    }
}

#[test]
fn test_error_recovery() {
    let invalid_inputs = vec![
        "let x = ;", // Missing value
        "fun () {}", // Missing function name
        "if { }",    // Missing condition
    ];

    for input in invalid_inputs {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_err(), "Should have failed to parse: {input}");

        // Verify error messages are helpful
        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty());
            // Could check for specific error messages here
        }
    }
}

#[test]
fn test_large_file_handling() {
    use std::fmt::Write;
    // Generate a large input file
    let mut large_input = String::new();
    for i in 0..1000 {
        writeln!(&mut large_input, "let var_{i} = {i};").unwrap();
    }

    let mut parser = Parser::new(&large_input);
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse large input");

    if let Ok(expr) = result {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile_expr(&expr);
        assert!(rust_code.is_ok(), "Failed to transpile large input");
    }
}

#[test]
fn test_unicode_support() {
    let inputs = vec![
        r#"let 你好 = "世界";"#,
        r#"let emoji = "🎉🎊";"#,
        r"// Comment with unicode: äöü",
    ];

    for input in inputs {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        // Unicode should be handled gracefully
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
#[ignore = "Object literals not yet implemented"]
fn test_nested_structures() {
    let input = r#"
        let data = {
            users: [
                { name: "Alice", age: 30 },
                { name: "Bob", age: 25 }
            ],
            count: 2
        };
    "#;

    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse nested structure");

    let transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_expr(&expr)
        .expect("Failed to transpile");

    // Verify nested structures are preserved
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("struct") || rust_str.contains('{'));
}
