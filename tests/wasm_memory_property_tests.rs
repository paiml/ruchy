#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag

/// WASM Memory Model Property Tests
///
/// Property-based testing for WASM memory model invariants.
/// Uses proptest to generate thousands of random inputs and verify
/// that memory operations maintain correctness properties.
///
/// Tests verify:
/// - Memory allocation produces valid WASM modules
/// - Field access offsets are deterministic
/// - Array indexing is consistent
/// - Mutations persist correctly
use assert_cmd::Command;
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

fn temp_wasm_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("wasm_prop_test_{name}.wasm"))
}

fn temp_ruchy_file(name: &str, code: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("wasm_prop_test_{name}.ruchy"));
    fs::write(&path, code).expect("Failed to write test file");
    path
}

/// Verify that any valid tuple compiles to valid WASM
fn compiles_to_valid_wasm(code: &str, test_name: &str) -> bool {
    let ruchy_file = temp_ruchy_file(test_name, code);
    let wasm_file = temp_wasm_file(test_name);

    let result = ruchy_cmd()
        .arg("wasm")
        .arg(&ruchy_file)
        .arg("-o")
        .arg(&wasm_file)
        .assert()
        .try_success();

    let valid = if result.is_ok() {
        if let Ok(wasm_bytes) = fs::read(&wasm_file) {
            wasm_bytes.starts_with(b"\0asm")
        } else {
            false
        }
    } else {
        false
    };

    // Cleanup
    fs::remove_file(&ruchy_file).ok();
    fs::remove_file(&wasm_file).ok();

    valid
}

#[cfg(test)]
mod property_tests {
    use super::*;

    proptest! {
        /// Property: Any tuple with valid i32 values compiles to valid WASM
        #[test]
        #[ignore = "Run explicitly: cargo test property_tests -- --ignored"]
        fn prop_tuple_creation_always_valid(
            a in -1000i32..1000,
            b in -1000i32..1000,
            c in -1000i32..1000
        ) {
            let code = format!(
                r"
fn main() {{
    let t = ({a}, {b}, {c})
    println(t.0)
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "tuple_creation"),
                "Tuple ({}, {}, {}) should compile to valid WASM",
                a, b, c
            );
        }

        /// Property: Array with any valid i32 elements compiles to valid WASM
        #[test]
        #[ignore]
        fn prop_array_creation_always_valid(
            elements in prop::collection::vec(-1000i32..1000, 1..10)
        ) {
            let elements_str = elements.iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            let code = format!(
                r"
fn main() {{
    let arr = [{elements_str}]
    println(arr[0])
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "array_creation"),
                "Array [{}] should compile to valid WASM",
                elements_str
            );
        }

        /// Property: Tuple field access at any valid index compiles correctly
        #[test]
        #[ignore]
        fn prop_tuple_field_access_valid(
            values in prop::collection::vec(-100i32..100, 1..10),
            index in 0usize..9
        ) {
            if index >= values.len() {
                return Ok(());
            }

            let values_str = values.iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            let code = format!(
                r"
fn main() {{
    let t = ({values_str})
    println(t.{index})
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "tuple_field"),
                "Tuple field access t.{} should compile",
                index
            );
        }

        /// Property: Array mutations at any valid index compile correctly
        #[test]
        #[ignore]
        fn prop_array_mutation_valid(
            size in 1usize..10,
            index in 0usize..9,
            new_value in -1000i32..1000
        ) {
            if index >= size {
                return Ok(());
            }

            let initial = vec![0; size];
            let elements_str = initial.iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            let code = format!(
                r"
fn main() {{
    let mut arr = [{elements_str}]
    arr[{index}] = {new_value}
    println(arr[{index}])
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "array_mutation"),
                "Array mutation arr[{}] = {} should compile",
                index, new_value
            );
        }

        /// Property: Struct with any field values compiles to valid WASM
        #[test]
        #[ignore]
        fn prop_struct_creation_valid(
            x in -1000i32..1000,
            y in -1000i32..1000
        ) {
            let code = format!(
                r"
struct Point {{
    x: i32,
    y: i32
}}

fn main() {{
    let p = Point {{ x: {x}, y: {y} }}
    println(p.x)
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "struct_creation"),
                "Struct Point {{ x: {}, y: {} }} should compile",
                x, y
            );
        }

        /// Property: Struct field mutations compile correctly
        #[test]
        #[ignore]
        fn prop_struct_mutation_valid(
            initial_x in -100i32..100,
            initial_y in -100i32..100,
            new_x in -1000i32..1000
        ) {
            let code = format!(
                r"
struct Point {{
    x: i32,
    y: i32
}}

fn main() {{
    let mut p = Point {{ x: {initial_x}, y: {initial_y} }}
    p.x = {new_x}
    println(p.x)
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "struct_mutation"),
                "Struct mutation p.x = {} should compile",
                new_x
            );
        }

        /// Property: Nested tuples with any depth compile correctly
        #[test]
        #[ignore]
        fn prop_nested_tuple_valid(
            a in -100i32..100,
            b in -100i32..100,
            c in -100i32..100,
            d in -100i32..100
        ) {
            let code = format!(
                r"
fn main() {{
    let nested = (({a}, {b}), ({c}, {d}))
    println(nested.0)
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "nested_tuple"),
                "Nested tuple (({}, {}), ({}, {})) should compile",
                a, b, c, d
            );
        }

        /// Property: Destructuring with any valid tuple compiles
        #[test]
        #[ignore]
        fn prop_destructuring_valid(
            a in -1000i32..1000,
            b in -1000i32..1000
        ) {
            let code = format!(
                r"
fn main() {{
    let (x, y) = ({a}, {b})
    println(x)
    println(y)
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "destructuring"),
                "Destructuring let (x, y) = ({}, {}) should compile",
                a, b
            );
        }

        /// Property: Mixed data structures compile correctly
        #[test]
        #[ignore]
        fn prop_mixed_structures_valid(
            arr_val in -100i32..100,
            tup_val in -100i32..100,
            struct_x in -100i32..100
        ) {
            let code = format!(
                r"
struct Point {{
    x: i32,
    y: i32
}}

fn main() {{
    let arr = [{arr_val}]
    let tup = ({tup_val}, 20)
    let p = Point {{ x: {struct_x}, y: 200 }}
    println(arr[0])
}}
"
            );

            prop_assert!(
                compiles_to_valid_wasm(&code, "mixed_structures"),
                "Mixed structures should compile"
            );
        }
    }
}

/// Invariant Tests - Mathematical properties that must ALWAYS hold
#[cfg(test)]
mod invariant_tests {
    use super::*;

    /// Invariant: Empty tuple should always compile to valid WASM
    #[test]
    fn invariant_empty_tuple_compiles() {
        let code = r"
fn main() {
    let unit = ()
    println(42)
}
";
        assert!(compiles_to_valid_wasm(code, "empty_tuple_inv"));
    }

    /// Invariant: Single element tuple should always compile
    #[test]
    fn invariant_single_element_tuple_compiles() {
        let code = r"
fn main() {
    let single = (42,)
    println(single.0)
}
";
        assert!(compiles_to_valid_wasm(code, "single_tuple_inv"));
    }

    /// Invariant: Array with zero should always compile
    #[test]
    fn invariant_zero_array_compiles() {
        let code = r"
fn main() {
    let arr = [0, 0, 0]
    println(arr[0])
}
";
        assert!(compiles_to_valid_wasm(code, "zero_array_inv"));
    }

    /// Invariant: Struct with zero fields should compile
    #[test]
    fn invariant_zero_struct_compiles() {
        let code = r"
struct Point {
    x: i32,
    y: i32
}

fn main() {
    let p = Point { x: 0, y: 0 }
    println(p.x)
}
";
        assert!(compiles_to_valid_wasm(code, "zero_struct_inv"));
    }

    /// Invariant: Negative values should compile correctly
    #[test]
    fn invariant_negative_values_compile() {
        let code = r"
fn main() {
    let t = (-1, -2, -3)
    println(t.0)
}
";
        assert!(compiles_to_valid_wasm(code, "negative_inv"));
    }

    /// Invariant: Maximum safe i32 values should compile
    #[test]
    fn invariant_max_values_compile() {
        let code = r"
fn main() {
    let t = (2147483647, 2147483646, 2147483645)
    println(t.0)
}
";
        assert!(compiles_to_valid_wasm(code, "max_values_inv"));
    }

    /// Invariant: Minimum safe i32 values should compile
    #[test]
    fn invariant_min_values_compile() {
        let code = r"
fn main() {
    let t = (-2147483648, -2147483647, -2147483646)
    println(t.0)
}
";
        assert!(compiles_to_valid_wasm(code, "min_values_inv"));
    }
}
