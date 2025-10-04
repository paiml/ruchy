#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Roundtrip tests ensure that parse -> transpile -> compile -> run produces expected results
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args
)]

use ruchy::{Parser, Transpiler};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

/// Helper to run the full pipeline and get output
fn run_pipeline(source: &str) -> Result<String, String> {
    // Parse
    let mut parser = Parser::new(source);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Transpile
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler
        .transpile_to_string(&ast)
        .map_err(|e| format!("Transpile error: {:?}", e))?;

    // Write to temp file
    let mut temp_file = NamedTempFile::new().map_err(|e| format!("Temp file error: {}", e))?;

    temp_file
        .write_all(rust_code.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;

    temp_file
        .flush()
        .map_err(|e| format!("Flush error: {}", e))?;

    // Compile
    let output_binary = temp_file.path().with_extension("exe");
    let compile_result = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-O")
        .arg("-o")
        .arg(&output_binary)
        .arg(temp_file.path())
        .output()
        .map_err(|e| format!("Compile command error: {}", e))?;

    if !compile_result.status.success() {
        return Err(format!(
            "Compile failed: {}",
            String::from_utf8_lossy(&compile_result.stderr)
        ));
    }

    // Run
    let run_result = Command::new(&output_binary)
        .output()
        .map_err(|e| format!("Run error: {}", e))?;

    // Clean up
    if output_binary.exists() {
        std::fs::remove_file(output_binary).ok();
    }

    Ok(String::from_utf8_lossy(&run_result.stdout).to_string())
}

#[test]
fn roundtrip_hello_world() {
    let source = r#"println("Hello, World!")"#;
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "Hello, World!");
}

#[test]
fn roundtrip_arithmetic() {
    let source = r"
        let x = 10
        let y = 20
        println(x + y)
    ";
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "30");
}

#[test]
fn roundtrip_function() {
    let source = r"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        
        println(add(5, 3))
    ";
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "8");
}

#[test]
fn roundtrip_if_else() {
    let source = r#"
        let x = 42
        if x > 40 {
            println("big")
        } else {
            println("small")
        }
    "#;
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "big");
}

#[test]
fn roundtrip_match() {
    let source = r#"
        let x = 2
        match x {
            1 => println("one"),
            2 => println("two"),
            _ => println("other")
        }
    "#;
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "two");
}

#[test]
fn roundtrip_array() {
    let source = r"
        let arr = [1, 2, 3, 4, 5]
        println(arr[2])
    ";
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "3");
}

#[test]
fn roundtrip_loop() {
    let source = r"
        let mut sum = 0
        for i in 1..=5 {
            sum = sum + i
        }
        println(sum)
    ";
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "15");
}

#[test]
fn roundtrip_string_interpolation() {
    let source = r#"
        let name = "Ruchy"
        let version = 4
        println(f"Welcome to {name} v{version}!")
    "#;
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "Welcome to Ruchy v4!");
}

#[test]
fn roundtrip_struct() {
    let source = r"
        struct Point {
            x: i32,
            y: i32
        }
        
        let p = Point { x: 10, y: 20 }
        println(p.x + p.y)
    ";
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "30");
}

#[test]
fn roundtrip_complex_expression() {
    let source = r"
        let result = (10 + 5) * 2 - (8 / 2)
        println(result)
    ";
    let output = run_pipeline(source).expect("Pipeline failed");
    assert_eq!(output.trim(), "26"); // (15 * 2) - 4 = 30 - 4 = 26
}
