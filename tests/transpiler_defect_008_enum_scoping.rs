//! TRANSPILER-DEFECT-008: Enum declarations must be at top-level, not inside `main()`
//!
//! Root Cause: `categorize_single_expression()` in mod.rs:938 was missing `ExprKind::Enum`,
//! causing enums to fall through to _ case and be categorized as statements (inside `main()`).
//!
//! Fix: Added `ExprKind::Enum` { .. } to top-level functions vector (line 941)
//!
//! Tests: 4 tests validating enum placement at file top-level

use assert_cmd::Command;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

#[test]
fn test_transpiler_defect_008_enum_at_top_level() {
    let code = r#"
        enum Status {
            Ok,
            Error(String),
        }

        fun main() {
            let s = Status::Ok;
            println!("Status: {:?}", s);
        }
    "#;

    let tempfile = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tempfile.path(), code).unwrap();

    // Transpile and check enum is at top-level
    let output = ruchy_cmd()
        .arg("transpile")
        .arg(tempfile.path())
        .output()
        .unwrap();

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Enum should appear BEFORE fn main()
    let enum_pos = transpiled
        .find("enum Status")
        .expect("enum Status not found");
    let main_pos = transpiled
        .find("fn __ruchy_main")
        .or_else(|| transpiled.find("fn main"))
        .expect("main function not found");

    assert!(
        enum_pos < main_pos,
        "Enum must be before main function. Enum at {enum_pos}, main at {main_pos}. Output:\n{transpiled}"
    );
}

#[test]
fn test_transpiler_defect_008_enum_used_in_function_signature() {
    let code = r#"
        enum Result {
            Ok(i32),
            Err(String),
        }

        fun test() -> Result {
            Result::Ok(42)
        }

        fun main() {
            match test() {
                Result::Ok(n) => println!("Success: {}", n),
                Result::Err(e) => println!("Error: {}", e),
            }
        }
    "#;

    let tempfile = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tempfile.path(), code).unwrap();

    // This should compile successfully (enum in scope for function signatures)
    ruchy_cmd()
        .arg("compile")
        .arg(tempfile.path())
        .assert()
        .success();
}

#[test]
fn test_transpiler_defect_008_multiple_enums_at_top_level() {
    let code = r#"
        enum Status {
            Active,
            Inactive,
        }

        enum Priority {
            High,
            Low,
        }

        fun main() {
            let s = Status::Active;
            let p = Priority::High;
            println!("Status: {:?}, Priority: {:?}", s, p);
        }
    "#;

    let tempfile = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tempfile.path(), code).unwrap();

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(tempfile.path())
        .output()
        .unwrap();

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Both enums should be at top-level
    let status_pos = transpiled
        .find("enum Status")
        .expect("enum Status not found");
    let priority_pos = transpiled
        .find("enum Priority")
        .expect("enum Priority not found");
    let main_pos = transpiled
        .find("fn __ruchy_main")
        .or_else(|| transpiled.find("fn main"))
        .expect("main function not found");

    assert!(
        status_pos < main_pos && priority_pos < main_pos,
        "Both enums must be before main. Status: {status_pos}, Priority: {priority_pos}, main: {main_pos}"
    );
}

#[test]
fn test_transpiler_defect_008_enum_with_struct_fields() {
    let code = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        enum Shape {
            Circle(i32),
            Rectangle(Point, Point),
        }

        fun main() {
            let c = Shape::Circle(10);
            println!("Shape: {:?}", c);
        }
    "#;

    let tempfile = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tempfile.path(), code).unwrap();

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(tempfile.path())
        .output()
        .unwrap();

    let transpiled = String::from_utf8_lossy(&output.stdout);

    // Struct and enum should both be at top-level, before main
    let struct_pos = transpiled
        .find("struct Point")
        .expect("struct Point not found");
    let enum_pos = transpiled.find("enum Shape").expect("enum Shape not found");
    let main_pos = transpiled
        .find("fn __ruchy_main")
        .or_else(|| transpiled.find("fn main"))
        .expect("main function not found");

    assert!(
        struct_pos < main_pos && enum_pos < main_pos,
        "Struct and enum must be before main. Struct: {struct_pos}, Enum: {enum_pos}, main: {main_pos}"
    );
}
