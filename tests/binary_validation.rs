#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args,
    clippy::print_stdout,
    clippy::expect_fun_call
)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

/// Test that all .ruchy example files can be compiled and executed
#[test]
fn test_binary_execution_all_examples() {
    let examples_dir = Path::new("examples");
    let ruchy_files: Vec<PathBuf> = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "ruchy" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    assert!(
        !ruchy_files.is_empty(),
        "No .ruchy files found in examples/"
    );

    for ruchy_file in ruchy_files {
        println!("Testing: {}", ruchy_file.display());
        validate_ruchy_file(&ruchy_file);
    }
}

/// Validate a single .ruchy file through the full compilation pipeline
fn validate_ruchy_file(path: &Path) {
    let content = fs::read_to_string(path).expect(&format!("Failed to read {}", path.display()));

    // Parse the file
    let mut parser = ruchy::Parser::new(&content);
    let ast = parser
        .parse()
        .expect(&format!("Failed to parse {}", path.display()));

    // Transpile to Rust
    let transpiler = ruchy::Transpiler::new();
    let rust_code = transpiler
        .transpile(&ast)
        .expect(&format!("Failed to transpile {}", path.display()));
    let rust_code_str = rust_code.to_string();

    // Write Rust code to temp file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(rust_code_str.as_bytes())
        .expect("Failed to write Rust code");
    temp_file.flush().expect("Failed to flush temp file");

    // Compile with rustc (which uses LLVM backend)
    let output_binary = temp_file.path().with_extension("exe");
    let compile_result = Command::new("rustc")
        .arg("--edition=2021")
        .arg("-O") // Enable optimizations (LLVM)
        .arg("-o")
        .arg(&output_binary)
        .arg(temp_file.path())
        .output()
        .expect("Failed to execute rustc");

    assert!(
        compile_result.status.success(),
        "Failed to compile {} to binary via LLVM:\nstderr: {}",
        path.display(),
        String::from_utf8_lossy(&compile_result.stderr)
    );

    // Check if expected output file exists
    let output_file = path.with_extension("output");
    if output_file.exists() {
        // Run the binary and compare output
        let run_result = Command::new(&output_binary)
            .output()
            .expect(&format!("Failed to run binary for {}", path.display()));

        let expected_output = fs::read_to_string(&output_file).expect(&format!(
            "Failed to read expected output {}",
            output_file.display()
        ));

        assert_eq!(
            String::from_utf8_lossy(&run_result.stdout).trim(),
            expected_output.trim(),
            "Output mismatch for {}",
            path.display()
        );
    }

    // Clean up
    if output_binary.exists() {
        fs::remove_file(output_binary).ok();
    }
}

/// Test specific example: fibonacci
#[test]
fn test_fibonacci_binary() {
    let fib_path = Path::new("examples/fibonacci.ruchy");
    if fib_path.exists() {
        validate_ruchy_file(fib_path);
    }
}

/// Test specific example: hello world
#[test]
fn test_hello_binary() {
    let hello_path = Path::new("examples/hello.ruchy");
    if hello_path.exists() {
        validate_ruchy_file(hello_path);
    }
}

/// Test compilation performance
#[test]
fn test_compilation_performance() {
    use std::time::Instant;

    let examples_dir = Path::new("examples");
    let ruchy_files: Vec<PathBuf> = fs::read_dir(examples_dir)
        .expect("Failed to read examples directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "ruchy" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for ruchy_file in ruchy_files {
        let content = fs::read_to_string(&ruchy_file)
            .expect(&format!("Failed to read {}", ruchy_file.display()));

        let start = Instant::now();

        // Parse
        let mut parser = ruchy::Parser::new(&content);
        let ast = parser
            .parse()
            .expect(&format!("Failed to parse {}", ruchy_file.display()));

        // Transpile
        let transpiler = ruchy::Transpiler::new();
        let _rust_code = transpiler
            .transpile(&ast)
            .expect(&format!("Failed to transpile {}", ruchy_file.display()));

        let elapsed = start.elapsed();

        // Assert compilation takes less than 5 seconds per example
        assert!(
            elapsed.as_secs() < 5,
            "Compilation of {} took too long: {:?}",
            ruchy_file.display(),
            elapsed
        );

        println!("Compiled {} in {:?}", ruchy_file.display(), elapsed);
    }
}
