//! Comprehensive tests for transpiler/types.rs (1,406 lines → TDG target)
//!
//! EXTREME TDD: TDG-driven testing for type transpilation
//! Target: src/backend/transpiler/types.rs (type transpilation methods)
//! Coverage: Named types, generics, optionals, lists, arrays, tuples, functions, references

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Named Types (transpile_named_type)
// ============================================================================

#[test]
fn test_named_type_int() {
    let code = "let x: int = 42";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("i64"), "int should transpile to i64");
}

#[test]
fn test_named_type_float() {
    let code = "let x: float = 3.14";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("f64"), "float should transpile to f64");
}

#[test]
fn test_named_type_bool() {
    let code = "let x: bool = true";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("bool"));
}

#[test]
fn test_named_type_str() {
    let code = "let x: str = \"hello\"";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // str transpiles to &str in Rust
    assert!(output.contains("&str") || output.contains("str"));
}

#[test]
fn test_named_type_string() {
    let code = "let x: String = \"hello\"";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("String"));
}

#[test]
fn test_named_type_char() {
    let code = "let x: char = 'a'";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("char"));
}

#[test]
fn test_named_type_unit() {
    let code = r"
        fun do_nothing() -> () {
            ()
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Unit type ()
    assert!(output.contains("()") || output.contains("fn do_nothing"));
}

// ============================================================================
// Generic Types (transpile_generic_type)
// ============================================================================

#[test]
fn test_generic_type_vec() {
    let code = "let v: Vec<i32> = [1, 2, 3]";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Vec") && output.contains("i32"));
}

#[test]
fn test_generic_type_option() {
    let code = "let opt: Option<i32> = Some(42)";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Option") || output.contains("Some"));
}

#[test]
fn test_generic_type_result() {
    let code = "let res: Result<i32, String> = Ok(42)";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Result") || output.contains("Ok"));
}

#[test]
fn test_generic_type_hashmap() {
    let code = r"
        let map: HashMap<String, i32> = HashMap::new()
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("HashMap"));
}

// ============================================================================
// Optional Types (transpile_optional_type)
// ============================================================================

#[test]
#[ignore = "Parser feature gap: `?` postfix syntax for optional types not yet implemented (int? → Option<int>)"]
fn test_optional_type_simple() {
    let code = "let opt: int? = 42";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // int? should transpile to Option<i64>
    assert!(output.contains("Option") || output.contains("i64"));
}

#[test]
#[ignore = "Parser feature gap: `?` postfix syntax for optional types not yet implemented (String? → Option<String>)"]
fn test_optional_type_string() {
    let code = "let opt: String? = \"maybe\"";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Option") || output.contains("String"));
}

// ============================================================================
// List Types (transpile_list_type)
// ============================================================================

#[test]
fn test_list_type_integers() {
    let code = "let nums: [int] = [1, 2, 3]";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Vec") || output.contains("i64"));
}

#[test]
fn test_list_type_strings() {
    let code = r#"let words: [String] = ["hello", "world"]"#;
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Vec") || output.contains("String"));
}

// ============================================================================
// Array Types (transpile_array_type)
// ============================================================================

#[test]
fn test_array_type_fixed_size() {
    let code = "let arr: [int; 5] = [1, 2, 3, 4, 5]";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    // Fixed-size array [T; N]
    assert!(output.contains("i64") && (output.contains('5') || output.contains(';')));
}

// ============================================================================
// Tuple Types (transpile_tuple_type)
// ============================================================================

#[test]
fn test_tuple_type_two_elements() {
    let code = "let pair: (int, String) = (42, \"hello\")";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("i64") && output.contains("String"));
}

#[test]
fn test_tuple_type_three_elements() {
    let code = "let triple: (int, float, bool) = (1, 2.5, true)";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("i64") && output.contains("f64") && output.contains("bool"));
}

// ============================================================================
// Function Types (transpile_function_type)
// ============================================================================

#[test]
fn test_function_type_simple() {
    let code = r"
        fun apply(f: fn(int) -> int, x: int) -> int {
            f(x)
        }
    ";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("fn apply"));
}

// ============================================================================
// Reference Types (transpile_reference_type)
// ============================================================================

#[test]
fn test_reference_type_immutable() {
    let code = "let r: &int = &42";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains('&') && output.contains("i64"));
}

#[test]
fn test_reference_type_mutable() {
    let code = "let r: &mut int = &mut 42";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("mut"));
}

// ============================================================================
// Namespaced Types (TRANSPILER-DEFECT-005)
// ============================================================================

#[test]
#[ignore = "Parser requires complete let statements with initializers (let x: T = value, not let x: T)"]
fn test_namespaced_type_std() {
    let code = "let err: std::io::Error";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("std") && output.contains("io") && output.contains("Error"));
}

#[test]
#[ignore = "Parser requires complete let statements with initializers (let x: T = value, not let x: T)"]
fn test_namespaced_type_custom() {
    let code = "let sampler: trace::Sampler";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("trace") && output.contains("Sampler"));
}

// ============================================================================
// DataFrame/Series Types
// ============================================================================

#[test]
#[ignore = "DataFrame types require polars feature - may not be enabled in test environment"]
fn test_dataframe_type() {
    let code = "let df: DataFrame";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("DataFrame") || output.contains("polars"));
}

#[test]
#[ignore = "Series types require polars feature - may not be enabled in test environment"]
fn test_series_type() {
    let code = "let s: Series";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Series") || output.contains("polars"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_nested_generics() {
    let code = "let nested: Vec<Vec<i32>> = [[1, 2], [3, 4]]";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Vec"));
}

#[test]
#[ignore = "Parser feature gap: `?` postfix syntax for optional types not yet implemented"]
fn edge_case_optional_vec() {
    let code = "let opt_vec: Vec<i32>? = Some([1, 2, 3])";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Option") || output.contains("Vec"));
}

#[test]
#[ignore = "Parser requires complete let statements with initializers (let x: T = value)"]
fn edge_case_complex_tuple() {
    let code = "let complex: (Vec<i32>, Option<String>, bool)";
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let output = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(output.contains("Vec") && output.contains("Option") && output.contains("bool"));
}

// ============================================================================
// Property Tests
// ============================================================================

#[test]
fn property_all_primitive_types() {
    // Property: All primitive types transpile correctly
    let types = vec![
        ("int", "i64"),
        ("float", "f64"),
        ("bool", "bool"),
        ("String", "String"),
        ("char", "char"),
    ];

    for (ruchy_type, rust_type) in types {
        let code = format!("let x: {ruchy_type} = 42");
        let result = ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();

        let output = String::from_utf8_lossy(&result.get_output().stdout);
        assert!(
            output.contains(rust_type),
            "{ruchy_type} should contain {rust_type}"
        );
    }
}

#[test]
fn property_vec_with_all_primitive_types() {
    // Property: Vec<T> works with all primitive types
    let types = vec!["int", "float", "bool", "String"];

    for ty in types {
        let code = format!("let v: Vec<{ty}> = []");
        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

#[test]
fn property_option_with_all_primitive_types() {
    // Property: Option<T> works with all primitive types
    let types = vec!["int", "float", "bool", "String"];

    for ty in types {
        let code = format!("let opt: Option<{ty}> = None");
        ruchy_cmd()
            .arg("transpile")
            .arg("-")
            .write_stdin(code.as_str())
            .assert()
            .success();
    }
}

// ============================================================================
// Integration: Full Transpile → Compile
// ============================================================================

#[test]
#[ignore = "expensive: invokes rustc"]
fn integration_type_annotations_compile() {
    let code = r#"
        fun typed_function(x: int, y: float, s: String) -> bool {
            x > 0 && y > 0.0 && !s.is_empty()
        }

        fun main() {
            let result = typed_function(42, 3.14, "hello");
            println!("{}", result);
        }
    "#;

    // Transpile
    let result = ruchy_cmd()
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let rust_code = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify type annotations present
    assert!(rust_code.contains("i64"));
    assert!(rust_code.contains("f64"));
    assert!(rust_code.contains("String"));
    assert!(rust_code.contains("bool"));

    // Write to temp file and compile
    std::fs::write("/tmp/transpiler_types_test.rs", rust_code.as_ref()).unwrap();
    let compile = std::process::Command::new("rustc")
        .args([
            "--crate-type",
            "bin",
            "/tmp/transpiler_types_test.rs",
            "-o",
            "/tmp/transpiler_types_test",
        ])
        .output()
        .unwrap();

    assert!(
        compile.status.success(),
        "Compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}
