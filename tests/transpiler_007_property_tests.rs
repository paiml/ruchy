// TRANSPILER-007: Property-Based Tests for Empty Vec Type Inference
//
// Property tests verify that the transpiler consistently generates correct type hints
// for empty vec initializations across hundreds of random inputs.

use proptest::prelude::*;
use ruchy::{compile, Parser, Transpiler};

/// Helper: Transpile Ruchy code to Rust
fn transpile_code(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse().map_err(|e| format!("Parse error: {e:?}"))?;

    let mut transpiler = Transpiler::new();
    let tokens = transpiler
        .transpile_to_program(&ast)
        .map_err(|e| format!("Transpile error: {e:?}"))?;

    Ok(tokens.to_string())
}

/// Property: Functions with explicit return type [T] generate Vec<T> for empty vecs
#[test]
fn property_explicit_return_type_generates_concrete_vec_type() {
    // Test with multiple primitive types
    let types = vec!["i32", "i64", "f64", "bool", "String", "u32", "usize"];

    for ty in types {
        let code = format!(
            r"
fun test_func() -> [{ty}] {{
    let items = []
    items
}}
"
        );

        let result = transpile_code(&code).expect("Should transpile");

        // Should contain Vec<T>, not Vec<_>
        let expected_vec = format!("Vec<{ty}>");
        let expected_spaced = format!("Vec < {ty} >"); // Handle spacing variations
        assert!(
            result.contains(&expected_vec) || result.contains(&expected_spaced),
            "Expected {expected_vec} for type {ty}, got:\n{result}"
        );
    }
}

/// Property: Functions without return type generate Vec<_> for empty vecs
#[test]
fn property_no_return_type_generates_generic_vec() {
    let code = r"
fun test_func() {
    let items = []
    items
}
";

    let result = transpile_code(code).expect("Should transpile");

    // Should contain Vec<_>
    assert!(
        result.contains("Vec<_>") || result.contains("Vec < _ >"),
        "Expected Vec<_> without return type, got:\n{result}"
    );
}

/// Property: Multiple empty vecs in same function all get same type
#[test]
fn property_multiple_empty_vecs_consistent_type() {
    let types = vec!["i32", "String", "f64", "bool"];

    for ty in types {
        let code = format!(
            r"
fun test_func() -> [{ty}] {{
    let a = []
    let b = []
    let c = []
    a
}}
"
        );

        let result = transpile_code(&code).expect("Should transpile");

        // Count occurrences of Vec<T> (with or without spaces)
        let expected_vec = format!("Vec<{ty}>");
        let expected_spaced = format!("Vec < {ty} >");
        let count =
            result.matches(&expected_vec).count() + result.matches(&expected_spaced).count();

        // Should have at least 3 occurrences (one for each empty vec)
        assert!(
            count >= 3,
            "Expected at least 3 occurrences of Vec<{ty}> or Vec < {ty} >, found {count} in:\n{result}"
        );
    }
}

/// Property: Nested vec types work correctly
#[test]
fn property_nested_vec_types() {
    let code = r"
fun test_func() -> [[i32]] {
    let matrix = []
    matrix
}
";

    let result = transpile_code(code).expect("Should transpile");

    // Should contain nested Vec types
    assert!(
        result.contains("Vec<Vec<i32>>")
            || result.contains("Vec < Vec < i32 > >")
            || result.contains("Vec<Vec < i32 >>"),
        "Expected nested Vec<Vec<i32>>, got:\n{result}"
    );
}

/// Property: Mutable and immutable vecs both get type hints
#[test]
fn property_mutability_doesnt_affect_type_hints() {
    for mutability in &["", "mut "] {
        let code = format!(
            r"
fun test_func() -> [i32] {{
    let {mutability}items = []
    items
}}
"
        );

        let result = transpile_code(&code).expect("Should transpile");

        assert!(
            result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
            "Expected Vec<i32> regardless of mutability ({}), got:\n{}",
            if mutability.is_empty() {
                "immutable"
            } else {
                "mutable"
            },
            result
        );
    }
}

/// Property: Generated Rust is syntactically valid
#[test]
fn property_generated_rust_is_valid_syntax() {
    let test_cases = vec![
        r"fun f() -> [i32] { let x = []; x }",
        r"fun f() -> [String] { let mut y = []; y }",
        r"fun f() -> [f64] { let a = []; let b = []; a }",
        r"fun f() -> [[bool]] { let matrix = []; matrix }",
    ];

    for code in test_cases {
        let rust_code = compile(code).expect("Should compile");

        // Verify it's valid Rust syntax
        syn::parse_file(&rust_code)
            .unwrap_or_else(|e| panic!("Invalid Rust syntax for input '{code}': {e:?}"));
    }
}

/// Property: Empty vecs in loops get type hints
#[test]
fn property_empty_vecs_in_loops_get_type_hints() {
    let loop_constructs = vec!["while i < 10", "for i in 0..10"];

    for loop_construct in loop_constructs {
        let code = format!(
            r"
fun test_func() -> [i32] {{
    let mut result = []
    let mut i = 0
    {loop_construct} {{
        result = result + [i]
        i = i + 1
    }}
    result
}}
"
        );

        let result = transpile_code(&code).expect("Should transpile");

        assert!(
            result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
            "Expected Vec<i32> in {loop_construct} loop, got:\n{result}"
        );
    }
}

/// Property: Consistency across multiple invocations
#[test]
fn property_deterministic_transpilation() {
    let code = r"
fun test() -> [i32] {
    let items = []
    items
}
";

    let result1 = transpile_code(code).expect("First transpilation");
    let result2 = transpile_code(code).expect("Second transpilation");

    assert_eq!(result1, result2, "Transpilation should be deterministic");
}

// ============================================================================
// PROPTEST-BASED TESTS (100+ random cases)
// ============================================================================

/// Strategy: Generate valid Ruchy primitive type names
fn ruchy_primitive_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("i32".to_string()),
        Just("i64".to_string()),
        Just("f64".to_string()),
        Just("bool".to_string()),
        Just("String".to_string()),
        Just("u32".to_string()),
        Just("usize".to_string()),
    ]
}

/// Strategy: Generate valid function names
fn function_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_map(|s| s)
}

/// Strategy: Generate variable names
fn variable_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_map(|s| s)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Any function with [T] return type generates Vec<T>
    #[test]
    fn prop_explicit_return_type_always_generates_concrete_type(
        func_name in function_name(),
        var_name in variable_name(),
        ty in ruchy_primitive_type()
    ) {
        let code = format!(
            r"
fun {func_name}() -> [{ty}] {{
    let {var_name} = []
    {var_name}
}}
"
        );

        if let Ok(result) = transpile_code(&code) {
            let expected = format!("Vec<{ty}>");
            let expected_spaced = format!("Vec < {ty} >");
            prop_assert!(
                result.contains(&expected) || result.contains(&expected_spaced),
                "Expected Vec<{}> for function {}, got:\n{}",
                ty,
                func_name,
                result
            );
        }
    }

    /// Property: Functions without return type use Vec<_>
    #[test]
    fn prop_no_return_type_always_uses_generic_vec(
        func_name in function_name(),
        var_name in variable_name()
    ) {
        let code = format!(
            r"
fun {func_name}() {{
    let {var_name} = []
    {var_name}
}}
"
        );

        if let Ok(result) = transpile_code(&code) {
            prop_assert!(
                result.contains("Vec<_>") || result.contains("Vec < _ >"),
                "Expected Vec<_> without return type, got:\n{}",
                result
            );
        }
    }

    /// Property: Multiple empty vecs get consistent type
    #[test]
    fn prop_multiple_empty_vecs_consistent(
        func_name in function_name(),
        var1 in variable_name(),
        var2 in variable_name(),
        ty in ruchy_primitive_type()
    ) {
        // Ensure var names are different
        prop_assume!(var1 != var2);

        let code = format!(
            r"
fun {func_name}() -> [{ty}] {{
    let {var1} = []
    let {var2} = []
    {var1}
}}
"
        );

        if let Ok(result) = transpile_code(&code) {
            let expected = format!("Vec<{ty}>");
            let expected_spaced = format!("Vec < {ty} >");
            let count = result.matches(&expected).count() + result.matches(&expected_spaced).count();
            prop_assert!(
                count >= 2,
                "Expected at least 2 Vec<{}> or Vec < {} > for variables {} and {}, found {} in:\n{}",
                ty,
                ty,
                var1,
                var2,
                count,
                result
            );
        }
    }

    /// Property: Mutable vs immutable doesn't affect type inference
    #[test]
    fn prop_mutability_independent(
        func_name in function_name(),
        var_name in variable_name(),
        ty in ruchy_primitive_type(),
        is_mut in prop::bool::ANY
    ) {
        let mutability = if is_mut { "mut " } else { "" };
        let code = format!(
            r"
fun {func_name}() -> [{ty}] {{
    let {mutability}{var_name} = []
    {var_name}
}}
"
        );

        if let Ok(result) = transpile_code(&code) {
            let expected = format!("Vec<{ty}>");
            let expected_spaced = format!("Vec < {ty} >");
            prop_assert!(
                result.contains(&expected) || result.contains(&expected_spaced),
                "Expected Vec<{}> regardless of mutability, got:\n{}",
                ty,
                result
            );
        }
    }

    /// Property: Transpilation is deterministic
    #[test]
    fn prop_deterministic(
        func_name in function_name(),
        var_name in variable_name(),
        ty in ruchy_primitive_type()
    ) {
        let code = format!(
            r"
fun {func_name}() -> [{ty}] {{
    let {var_name} = []
    {var_name}
}}
"
        );

        if let Ok(result1) = transpile_code(&code) {
            if let Ok(result2) = transpile_code(&code) {
                prop_assert_eq!(result1, result2, "Transpilation must be deterministic");
            }
        }
    }
}

// ============================================================================
// EDGE CASE PROPERTY TESTS
// ============================================================================

/// Property: Deep nesting works correctly
#[test]
fn property_deep_nesting() {
    // Test up to 3 levels of nesting
    let test_cases = vec![
        (
            "[[i32]]",
            vec![
                "Vec<Vec<i32>>",
                "Vec < Vec < i32 > >",
                "Vec<Vec < i32 >>",
                "Vec < Vec<i32> >",
            ],
        ),
        (
            "[[[i32]]]",
            vec![
                "Vec<Vec<Vec<i32>>>",
                "Vec < Vec < Vec < i32 > > >",
                "Vec<Vec<Vec < i32 >>>",
                "Vec<Vec < Vec<i32> >>",
            ],
        ),
    ];

    for (ruchy_type, expected_variants) in test_cases {
        let code = format!(
            r"
fun test() -> {ruchy_type} {{
    let data = []
    data
}}
"
        );

        let result = transpile_code(&code).expect("Should transpile");

        // Check for any expected type variant
        let has_type = expected_variants
            .iter()
            .any(|variant| result.contains(variant));

        assert!(
            has_type,
            "Expected one of {expected_variants:?} for type {ruchy_type}, got:\n{result}"
        );
    }
}

/// Property: Type inference works with shadowing
#[test]
fn property_shadowing_preserves_type_hints() {
    let code = r"
fun test() -> [i32] {
    let x = []
    let x = x + [1]
    let x = x + [2]
    x
}
";

    let result = transpile_code(code).expect("Should transpile");

    // First occurrence should have Vec<i32>
    assert!(
        result.contains("Vec<i32>") || result.contains("Vec < i32 >"),
        "Expected Vec<i32> with shadowing, got:\n{result}"
    );
}
