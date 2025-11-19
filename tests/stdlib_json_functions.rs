#![allow(missing_docs)]
//! STDLIB Phase 4: JSON Functions Tests
//!
//! **Task**: Implement 10 JSON functions
//! **Priority**: HIGH (Phase 4 of `STDLIB_ACCESS_PLAN`)
//! **Pattern**: Three-layer builtin function (proven from env/fs/path functions)
//!
//! Functions:
//! 1. `json_parse(str`: String) -> Value
//! 2. `json_stringify(value`: Value) -> String
//! 3. `json_pretty(value`: Value) -> String
//! 4. `json_read(path`: String) -> Value
//! 5. `json_write(path`: String, value: Value) -> Bool
//! 6. `json_validate(str`: String) -> Bool
//! 7. `json_type(str`: String) -> String
//! 8. `json_merge(obj1`: Value, obj2: Value) -> Value
//! 9. `json_get(obj`: Value, path: String) -> Value
//! 10. `json_set(obj`: Value, path: String, value: Value) -> Value
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== json_parse() Tests ====================

#[test]
fn test_json_parse_object() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json_str = "{\"name\": \"Alice\", \"age\": 30}";
    let obj = json_parse(json_str);
    println(obj);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_json_parse_array() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json_str = "[1, 2, 3, 4, 5]";
    let arr = json_parse(json_str);
    println(arr);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_stringify() Tests ====================

#[test]
fn test_json_stringify_object() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json_str = "{\"name\": \"Bob\", \"age\": 25}";
    let obj = json_parse(json_str);
    let json = json_stringify(obj);
    println(json);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_json_stringify_array() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r"
fun main() {
    let arr = [1, 2, 3, 4, 5];
    let json = json_stringify(arr);
    println(json);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_pretty() Tests ====================

#[test]
fn test_json_pretty() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json_str = "{\"name\": \"Charlie\", \"age\": 35}";
    let obj = json_parse(json_str);
    let pretty = json_pretty(obj);
    println(pretty);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_read() Tests ====================

#[test]
fn test_json_read() {
    let temp = temp_dir();
    let json_file = temp.path().join("data.json");
    fs::write(&json_file, r#"{"city": "Paris", "country": "France"}"#).unwrap();

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let data = json_read("{}");
    println(data);
}}
"#,
        json_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_write() Tests ====================

#[test]
fn test_json_write() {
    let temp = temp_dir();
    let json_file = temp.path().join("output.json");

    let source = temp.path().join("test.ruchy");
    let code = format!(
        r#"
fun main() {{
    let json_str = "{{\\\"status\\\": \\\"ok\\\", \\\"code\\\": 200}}";
    let obj = json_parse(json_str);
    let result = json_write("{}", obj);
    println(result);
}}
"#,
        json_file.display()
    );

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();

    // Verify file was created
    assert!(json_file.exists());
}

// ==================== json_validate() Tests ====================

#[test]
fn test_json_validate_valid() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let valid_json = "{\"key\": \"value\"}";
    let is_valid = json_validate(valid_json);
    println(is_valid);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_json_validate_invalid() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let invalid_json = "{key: value}";
    let is_valid = json_validate(invalid_json);
    println(is_valid);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_type() Tests ====================

#[test]
fn test_json_type() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let obj_str = "{\"name\": \"test\"}";
    let arr_str = "[1, 2, 3]";
    println(json_type(obj_str));
    println(json_type(arr_str));
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_merge() Tests ====================

#[test]
fn test_json_merge() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json1 = "{\"a\": 1, \"b\": 2}";
    let obj1 = json_parse(json1);

    let json2 = "{\"b\": 3, \"c\": 4}";
    let obj2 = json_parse(json2);

    let merged = json_merge(obj1, obj2);
    println(merged);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_get() Tests ====================

#[test]
fn test_json_get() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json_str = "{\"user\": {\"name\": \"David\", \"age\": 40}}";
    let obj = json_parse(json_str);
    let name = json_get(obj, "user.name");
    println(name);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== json_set() Tests ====================

#[test]
fn test_json_set() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");
    let code = r#"
fun main() {
    let json_str = "{\"user\": {\"name\": \"Eve\", \"age\": 28}}";
    let obj = json_parse(json_str);
    let updated = json_set(obj, "user.age", 29);
    println(updated);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== Summary Test ====================

#[test]
fn test_json_functions_summary() {
    println!("STDLIB Phase 4: JSON Functions");
    println!("1. json_parse(str) - Parse JSON string to value");
    println!("2. json_stringify(value) - Convert value to JSON string");
    println!("3. json_pretty(value) - Pretty-print JSON with indentation");
    println!("4. json_read(path) - Read and parse JSON file");
    println!("5. json_write(path, value) - Write value as JSON to file");
    println!("6. json_validate(str) - Check if string is valid JSON");
    println!("7. json_type(str) - Get JSON type without full parsing");
    println!("8. json_merge(obj1, obj2) - Deep merge two JSON objects");
    println!("9. json_get(obj, path) - Get nested value by path");
    println!("10. json_set(obj, path, value) - Set nested value by path");
    println!();
    println!("Three-Layer Implementation Required for each:");
    println!("1. Runtime: builtin_* in builtins.rs");
    println!("2. Transpiler: case in try_transpile_json_function()");
    println!("3. Environment: eval_* in eval_builtin.rs + builtin_init.rs");
}
