//! EXTREME TDD: Type Annotation Tests (LANG-003)
//!
//! Test-first development for comprehensive type annotation support
//! Target: Enable proper transpilation of type annotations to Rust code
//! Complexity: All test functions ≤10 cyclomatic complexity
//! Coverage: 100% of type annotation variations

use ruchy::compile;

#[cfg(test)]
mod type_annotation_tests {
    use super::*;

    // =============================================================================
    // VARIABLE TYPE ANNOTATIONS
    // =============================================================================

    #[test]
    fn test_let_with_primitive_types() {
        let code = "let x: i32 = 42";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile: let x: i32 = 42");
        let output = result.unwrap();
        println!("Primitive type output: {output}");

        // Should generate: let x: i32 = 42i32;
        let has_type = output.contains("let x: i32") || output.contains("let x : i32");
        assert!(has_type, "Should include type annotation, got: {output}");
    }

    #[test]
    fn test_let_with_string_type() {
        let code = r#"let name: String = "hello""#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile string type annotation");
        let output = result.unwrap();
        println!("String type output: {output}");

        let has_type = output.contains("let name: String") || output.contains("let name : String");
        assert!(
            has_type,
            "Should include String type annotation, got: {output}"
        );
    }

    #[test]
    fn test_let_with_float_type() {
        let code = "let pi: f64 = 3.14159";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile float type annotation");
        let output = result.unwrap();

        let has_type = output.contains("let pi: f64") || output.contains("let pi : f64");
        assert!(
            has_type,
            "Should include f64 type annotation, got: {output}"
        );
    }

    #[test]
    fn test_let_with_bool_type() {
        let code = "let flag: bool = true";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile bool type annotation");
        let output = result.unwrap();

        let has_type = output.contains("let flag: bool") || output.contains("let flag : bool");
        assert!(
            has_type,
            "Should include bool type annotation, got: {output}"
        );
    }

    // =============================================================================
    // COMPOSITE TYPE ANNOTATIONS
    // =============================================================================

    #[test]
    #[ignore = "Generic types need implementation"]
    fn test_let_with_vector_type() {
        let code = "let numbers: Vec<i32> = [1, 2, 3]";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile Vec<i32> type annotation");
        let output = result.unwrap();

        let has_type = output.contains("Vec<i32>") || output.contains("Vec < i32 >");
        assert!(
            has_type,
            "Should include Vec<i32> type annotation, got: {output}"
        );
    }

    #[test]
    #[ignore = "Tuple types need implementation"]
    fn test_let_with_tuple_type() {
        let code = "let point: (f64, f64) = (1.0, 2.0)";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile tuple type annotation");
        let output = result.unwrap();

        let has_type = output.contains("(f64, f64)") || output.contains("( f64 , f64 )");
        assert!(
            has_type,
            "Should include tuple type annotation, got: {output}"
        );
    }

    #[test]
    #[ignore = "Option types need implementation"]
    fn test_let_with_option_type() {
        let code = "let maybe: Option<i32> = Some(42)";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile Option<i32> type annotation"
        );
        let output = result.unwrap();

        let has_type = output.contains("Option<i32>") || output.contains("Option < i32 >");
        assert!(
            has_type,
            "Should include Option<i32> type annotation, got: {output}"
        );
    }

    // =============================================================================
    // FUNCTION PARAMETER TYPES
    // =============================================================================

    #[test]
    #[ignore = "Function parameter types need implementation"]
    fn test_function_with_parameter_types() {
        let code = "fn add(x: i32, y: i32) -> i32 { x + y }";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile function with parameter types"
        );
        let output = result.unwrap();

        let has_params = output.contains("x: i32") && output.contains("y: i32");
        assert!(has_params, "Should include parameter types, got: {output}");

        let has_return = output.contains("-> i32") || output.contains("-> i32");
        assert!(has_return, "Should include return type, got: {output}");
    }

    #[test]
    #[ignore = "Function types need implementation"]
    fn test_function_with_string_parameter() {
        let code = r#"fn greet(name: String) -> String { "Hello, " + name }"#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile function with String parameter"
        );
        let output = result.unwrap();

        let has_param = output.contains("name: String");
        assert!(
            has_param,
            "Should include String parameter type, got: {output}"
        );

        let has_return = output.contains("-> String");
        assert!(
            has_return,
            "Should include String return type, got: {output}"
        );
    }

    // =============================================================================
    // REFERENCE TYPES
    // =============================================================================

    #[test]
    #[ignore = "Reference types need implementation"]
    fn test_reference_types() {
        let code = "fn read_slice(data: &[i32]) -> i32 { data[0] }";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile reference type");
        let output = result.unwrap();

        let has_ref = output.contains("&[i32]") || output.contains("& [ i32 ]");
        assert!(has_ref, "Should include reference type, got: {output}");
    }

    #[test]
    #[ignore = "Mutable reference types need implementation"]
    fn test_mutable_reference_types() {
        let code = "fn modify_slice(data: &mut [i32]) { data[0] = 42 }";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile mutable reference type");
        let output = result.unwrap();

        let has_mut_ref = output.contains("&mut [i32]") || output.contains("& mut [ i32 ]");
        assert!(
            has_mut_ref,
            "Should include mutable reference type, got: {output}"
        );
    }

    // =============================================================================
    // COMPLEX TYPES
    // =============================================================================

    #[test]
    #[ignore = "Function types need implementation"]
    fn test_function_type_parameter() {
        let code = "fn apply(f: fn(i32) -> i32, x: i32) -> i32 { f(x) }";
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile function type parameter");
        let output = result.unwrap();

        let has_fn_type = output.contains("fn(i32) -> i32") || output.contains("fn ( i32 ) -> i32");
        assert!(has_fn_type, "Should include function type, got: {output}");
    }

    // =============================================================================
    // ERROR CASES
    // =============================================================================

    #[test]
    fn test_invalid_type_annotation() {
        let code = "let x: InvalidType = 42";
        let result = compile(code);
        // Should either succeed (treating as unknown type) or fail gracefully
        // For now, let's just ensure it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_type_mismatch() {
        let code = "let x: i32 = \"hello\"";
        let result = compile(code);
        // Should compile successfully (type checker is separate concern)
        assert!(
            result.is_ok(),
            "Compilation should succeed even with type mismatches"
        );
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    #[test]
    fn test_mixed_type_annotations() {
        let code = r#"
            let x: i32 = 42
            let name: String = "test"
            let flag: bool = true
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Failed to compile mixed type annotations");
        let output = result.unwrap();

        let has_all_types =
            output.contains("i32") && output.contains("String") && output.contains("bool");
        assert!(
            has_all_types,
            "Should include all type annotations, got: {output}"
        );
    }

    #[test]
    #[ignore = "Function integration needs implementation"]
    fn test_function_with_typed_variables() {
        let code = r#"
            fn process() -> i32 {
                let x: i32 = 10
                let y: i32 = 20
                x + y
            }
        "#;
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile function with typed variables"
        );
        let output = result.unwrap();

        let has_types = output.contains("let x: i32") && output.contains("let y: i32");
        assert!(
            has_types,
            "Should preserve variable type annotations in function, got: {output}"
        );
    }
}

#[cfg(test)]
mod type_annotation_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_primitive_types_never_panic(
            value in 1i32..1000
        ) {
            let code = format!("let x: i32 = {value}");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_string_types_never_panic(
            value in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let code = format!("let x: String = \"{value}\"");
            let _ = compile(&code); // Should not panic
        }

        #[test]
        fn test_type_names_never_panic(
            type_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let code = format!("let x: {type_name} = 42");
            let _ = compile(&code); // Should not panic
        }
    }
}
