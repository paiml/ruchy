// DEFECT-WASM-TUPLE-TYPES: WASM tuple compilation should handle mixed types
//
// BUG: WASM compilation fails for tuples with mixed types (int + float)
// ROOT CAUSE: lower_tuple() uses I32Store for all elements regardless of type
// FIX STRATEGY: Detect element type and use appropriate store/load instructions

use ruchy::frontend::parser::Parser;
use ruchy::WasmEmitter;

/// Helper to compile Ruchy code to WASM and validate it
fn compile_to_wasm(code: &str) -> anyhow::Result<Vec<u8>> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter
        .emit(&ast)
        .map_err(|e| anyhow::anyhow!("Failed to generate WASM: {e}"))?;

    // CRITICAL: Validate the WASM bytecode
    // This will fail if there are type mismatches
    eprintln!("Compiled {} bytes", wasm_bytes.len());
    eprintln!(
        "First 20 bytes: {:?}",
        &wasm_bytes[..wasm_bytes.len().min(20)]
    );

    // Write to temp file for inspection
    std::fs::write("/tmp/test_module.wasm", &wasm_bytes).ok();

    match wasmparser::validate(&wasm_bytes) {
        Ok(_) => eprintln!("Validation: PASSED"),
        Err(e) => {
            eprintln!("Validation: FAILED - {e}");
            return Err(e.into());
        }
    }

    Ok(wasm_bytes)
}

// ============================================================================
// Unit Tests (RED → GREEN → REFACTOR)
// ============================================================================

#[test]
fn test_wasm_tuple_int_float() {
    let code = r"
let x = (1, 3.0)
let a = x.0
let b = x.1
";

    let result = compile_to_wasm(code);
    assert!(
        result.is_ok(),
        "Should compile tuple with (int, float), but got: {:?}",
        result.err()
    );
}

#[test]
fn test_wasm_tuple_float_int() {
    let code = r"
let x = (3.0, 1)
println(x.0)
println(x.1)
";

    let result = compile_to_wasm(code);
    assert!(
        result.is_ok(),
        "Should compile tuple with (float, int), but got: {:?}",
        result.err()
    );
}

#[test]
fn test_wasm_tuple_all_floats() {
    let code = r"
let x = (1.0, 2.0, 3.0)
println(x.0)
println(x.1)
println(x.2)
";

    let result = compile_to_wasm(code);
    assert!(
        result.is_ok(),
        "Should compile tuple with all floats, but got: {:?}",
        result.err()
    );
}

#[test]
fn test_wasm_tuple_string_int() {
    let code = r#"
let x = ("hello", 42)
println(x.0)
println(x.1)
"#;

    let result = compile_to_wasm(code);
    assert!(
        result.is_ok(),
        "Should compile tuple with (string, int), but got: {:?}",
        result.err()
    );
}

#[test]
fn test_wasm_tuple_mixed_triple() {
    let code = r#"
let person = ("Alice", 30, 5.5)
println(person.0)
println(person.1)
println(person.2)
"#;

    let result = compile_to_wasm(code);
    assert!(
        result.is_ok(),
        "Should compile tuple with (string, int, float), but got: {:?}",
        result.err()
    );
}

// ============================================================================
// Integration Test - Full Example File
// ============================================================================

#[test]
fn test_example_file_03_tuples() {
    let code = std::fs::read_to_string("examples/lang_comp/06-data-structures/03_tuples.ruchy")
        .expect("Failed to read example file");

    let result = compile_to_wasm(&code);
    assert!(
        result.is_ok(),
        "Should compile 03_tuples.ruchy, but got: {:?}",
        result.err()
    );
}
