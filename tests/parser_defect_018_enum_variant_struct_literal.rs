#![allow(missing_docs)]
// DEFECT-PARSER-018: Enum variant struct literals in array/collection contexts
//
// ROOT CAUSE: try_parse_struct_literal only handles ExprKind::Identifier,
// but enum variant paths like Shape::Circle are ExprKind::FieldAccess.
// When the parser sees `[Shape::Circle { radius: 5.0 }]`, it:
// 1. Parses Shape::Circle as FieldAccess
// 2. Sees `{` and calls try_parse_struct_literal
// 3. try_parse_struct_literal returns None (not an Identifier)
// 4. The `{` is left unconsumed and parser gets confused
//
// TEST STRATEGY:
// 1. RED tests for enum variant struct literals in various contexts
// 2. Regression tests for existing struct literal functionality
//
// MINIMAL REPRODUCTION:
// ```ruchy
// enum Shape { Circle { radius: float } }
// let shapes = [Shape::Circle { radius: 5.0 }]  // FAILS
// ```

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

fn test_code(code: &str) {
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let thread_id = thread::current().id();
    let temp_file = PathBuf::from(format!(
        "/tmp/test_enum_variant_struct_{timestamp}_{thread_id:?}.ruchy"
    ));

    fs::write(&temp_file, code).expect("Failed to write test file");

    let result = ruchy_cmd().arg("check").arg(&temp_file).assert();

    // Clean up
    let _ = fs::remove_file(&temp_file);

    result.success();
}

// ============================================================================
// RED TESTS: These should fail initially (the actual defect)
// ============================================================================

#[test]
fn test_enum_variant_in_array_literal() {
    // MINIMAL REPRODUCTION of the defect
    test_code(
        r"fn main() {
    enum Shape {
        Circle { radius: float }
    }

    let shapes = [Shape::Circle { radius: 5.0 }]
    println(shapes)
}",
    );
}

#[test]
fn test_multiple_enum_variants_in_array() {
    test_code(
        r"fn main() {
    enum Shape {
        Circle { radius: float },
        Rectangle { width: float, height: float }
    }

    let shapes = [
        Shape::Circle { radius: 5.0 },
        Shape::Rectangle { width: 10.0, height: 20.0 }
    ]
    println(shapes)
}",
    );
}

#[test]
fn test_enum_variant_in_function_call() {
    test_code(
        r"fn main() {
    enum Shape {
        Circle { radius: float }
    }

    fn process(shape: Shape) {
        println(shape)
    }

    process(Shape::Circle { radius: 5.0 })
}",
    );
}

#[test]
fn test_enum_variant_as_return_value() {
    test_code(
        r"fn main() {
    enum Shape {
        Circle { radius: float }
    }

    fn make_circle() -> Shape {
        Shape::Circle { radius: 3.14 }
    }

    let c = make_circle()
    println(c)
}",
    );
}

#[test]
fn test_enum_variant_in_let_binding() {
    test_code(
        r"fn main() {
    enum Shape {
        Circle { radius: float }
    }

    let c = Shape::Circle { radius: 5.0 }
    println(c)
}",
    );
}

#[test]
fn test_nested_enum_variant_struct() {
    test_code(
        r"fn main() {
    enum Outer {
        Inner { value: int }
    }

    enum Container {
        Item { nested: Outer }
    }

    let x = Container::Item { nested: Outer::Inner { value: 42 } }
    println(x)
}",
    );
}

// ============================================================================
// REGRESSION TESTS: These should already pass
// ============================================================================

#[test]
fn test_simple_struct_literal_still_works() {
    test_code(
        r"fn main() {
    struct Point {
        x: float,
        y: float
    }

    let p = Point { x: 1.0, y: 2.0 }
    println(p)
}",
    );
}

#[test]
fn test_struct_literal_in_array_still_works() {
    test_code(
        r"fn main() {
    struct Point {
        x: float,
        y: float
    }

    let points = [Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }]
    println(points)
}",
    );
}

#[test]
fn test_enum_without_struct_fields_still_works() {
    test_code(
        r"fn main() {
    enum Color {
        Red,
        Green,
        Blue
    }

    let colors = [Color::Red, Color::Green, Color::Blue]
    println(colors)
}",
    );
}

// ============================================================================
// PROPERTY TESTS: Randomized validation (REFACTOR phase)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn prop_enum_variant_with_single_field(
            field_name in "[a-z][a-z0-9_]{0,5}",
            value in any::<i32>()
        ) {
            let code = format!(
                r"fn main() {{
    enum Test {{
        Variant {{ {field_name}: int }}
    }}
    let x = Test::Variant {{ {field_name}: {value} }}
    println(x)
}}"
            );
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on enum variant struct literal");
        }

        #[test]
        fn prop_enum_variant_in_array(
            count in 1usize..4usize
        ) {
            let variants: String = (0..count)
                .map(|i| format!("Test::V {{ n: {i} }}"))
                .collect::<Vec<_>>()
                .join(", ");
            let code = format!(
                r"fn main() {{
    enum Test {{
        V {{ n: int }}
    }}
    let arr = [{variants}]
    println(arr)
}}"
            );
            let result = std::panic::catch_unwind(|| {
                test_code(&code);
            });
            prop_assert!(result.is_ok(), "Parser panicked on array of enum variant structs");
        }
    }
}
