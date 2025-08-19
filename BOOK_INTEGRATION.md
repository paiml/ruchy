# Ruchy Book Integration Guide

## Quick Start for ruchy-book Repository

### 1. Add Ruchy as a Dependency

In your `ruchy-book/Cargo.toml`:

```toml
[dev-dependencies]
ruchy = "0.4.14"
```

### 2. Create Test File

Create `ruchy-book/tests/validate_examples.rs`:

```rust
use ruchy::testing::{RuchyTestHarness, OptLevel};
use std::path::Path;

#[test]
fn test_all_book_examples() {
    let harness = RuchyTestHarness::new();
    
    // Test all examples in listings directory
    let results = harness.validate_directory(Path::new("listings/"))
        .expect("Failed to validate examples");
    
    for result in results {
        assert!(result.compile_success, 
                "Example {} failed to compile", result.name);
        
        // If there's an expected output file, check it
        let output_file = Path::new(&result.name).with_extension("output");
        if output_file.exists() {
            let expected = std::fs::read_to_string(output_file).unwrap();
            assert_eq!(
                result.execution_output.unwrap().trim(), 
                expected.trim(),
                "Output mismatch for {}", result.name
            );
        }
    }
}

#[test]
fn test_specific_example() {
    let harness = RuchyTestHarness {
        keep_intermediates: false,
        optimization_level: OptLevel::Basic,
        timeout_secs: 30,
    };
    
    // Test a specific example with expected output
    harness.assert_output(
        r#"println("Hello, World!")"#,
        "Hello, World!",
        "hello_world_test"
    ).expect("Hello world test failed");
}
```

### 3. Directory Structure

Organize your book examples like this:

```
ruchy-book/
├── Cargo.toml
├── book.toml
├── src/
│   ├── SUMMARY.md
│   └── chapters/
├── listings/
│   ├── ch01-hello-world/
│   │   ├── hello.ruchy
│   │   └── hello.output      # Expected output
│   ├── ch02-variables/
│   │   ├── variables.ruchy
│   │   └── variables.output
│   └── ...
└── tests/
    └── validate_examples.rs
```

### 4. CI/CD Configuration

Add to `.github/workflows/test.yml`:

```yaml
name: Test Book Examples

on: [push, pull_request]

jobs:
  test-examples:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: 1.75.0
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
      
      - name: Test all examples
        run: cargo test --test validate_examples
      
      - name: Check compilation performance
        run: |
          # Ensure examples compile in <5s
          timeout 5m cargo test test_compilation_performance
```

## API Reference

### RuchyTestHarness

Main testing interface with these methods:

```rust
// Validate a single file
let result = harness.validate_file(Path::new("example.ruchy"))?;

// Validate source code directly
let result = harness.validate_source("let x = 42", "test_name")?;

// Assert specific output
harness.assert_output(source, expected_output, name)?;

// Validate entire directory
let results = harness.validate_directory(Path::new("examples/"))?;
```

### Configuration Options

```rust
let harness = RuchyTestHarness {
    // Keep intermediate Rust files for debugging
    keep_intermediates: false,
    
    // LLVM optimization level
    optimization_level: OptLevel::Basic, // None, Basic, or Full
    
    // Execution timeout in seconds
    timeout_secs: 30,
};
```

### ValidationResult

Each validation returns:

```rust
pub struct ValidationResult {
    pub name: String,
    pub parse_success: bool,
    pub transpile_success: bool,
    pub compile_success: bool,
    pub execution_output: Option<String>,
    pub rust_code: Option<String>, // If keep_intermediates = true
}
```

## Testing Philosophy

1. **Every example must compile** - No broken code in the book
2. **Expected outputs** - Use `.output` files for validation
3. **Performance targets** - Examples should compile in <5 seconds
4. **Progressive disclosure** - Simple examples in early chapters

## Compilation Pipeline

Your examples go through this pipeline:

```
.ruchy file → Parser → AST → Transpiler → Rust code → LLVM → Native Binary → Execution
```

## Performance Benchmarks

The infrastructure includes benchmarks to ensure:
- Parsing throughput >50MB/s
- Compilation <5s per example
- Binary execution <100ms

## Troubleshooting

### Common Issues

1. **Compilation fails**: Check that the Ruchy syntax is valid
2. **Output mismatch**: Ensure `.output` files have correct line endings
3. **Timeout**: Complex examples may need longer timeout
4. **Missing rustc**: The test harness requires Rust toolchain installed

### Debug Mode

To see intermediate Rust code:

```rust
let harness = RuchyTestHarness {
    keep_intermediates: true, // Keeps .rs files
    ..Default::default()
};
```

## Future Enhancements

When Ruchy becomes self-hosting (v1.0), you'll be able to use:

```bash
ruchy test examples/
ruchy check examples/hello.ruchy
ruchy fmt examples/
ruchy lint --strict examples/
```

For now, use the Rust testing harness API.

## Support

- GitHub Issues: https://github.com/paiml/ruchy/issues
- Documentation: https://docs.rs/ruchy
- Examples: See `ruchy/tests/` directory

## Version Compatibility

- Minimum Ruchy version: 0.4.14
- Rust toolchain: 1.75.0+
- LLVM: Provided by rustc