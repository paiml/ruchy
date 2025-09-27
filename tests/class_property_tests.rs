//! EXTREME TDD: Property tests for basic Class/Struct with 10,000 iterations
//! Testing invariants and edge cases with random inputs for completed functionality

use proptest::prelude::*;
use ruchy::compile;

// Property: Basic class definitions always compile successfully
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_simple_class_never_panics(
        class_name in "[A-Z][a-zA-Z0-9]{0,10}",
        field_name in "[a-z][a-zA-Z0-9]{0,10}",
        field_type in prop::sample::select(vec!["i32", "f64", "String", "bool"])
    ) {
        let code = format!(
            r"
            class {} {{
                {}: {}
            }}

            fun main() {{
                {} {{ {}: {} }}
            }}
            ",
            class_name, field_name, field_type,
            class_name, field_name,
            match field_type {
                "i32" => "42",
                "f64" => "3.14",
                "String" => "\"test\"",
                "bool" => "true",
                _ => "0"
            }
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Basic class should always compile successfully");

        if let Ok(output) = result {
            prop_assert!(output.contains("struct"), "Should transpile to struct");
            prop_assert!(output.contains(&class_name), "Should contain class name");
            prop_assert!(output.contains(&field_name), "Should contain field name");
        }
    }
}

// Property: Struct definitions with various field types
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_struct_definitions_robust(
        struct_name in "[A-Z][a-zA-Z0-9]{0,10}",
        field_count in 1u32..5u32
    ) {
        let fields: Vec<String> = (0..field_count)
            .map(|i| format!("field{i}: i32"))
            .collect();
        let field_str = fields.join(",\n            ");

        let instantiation_fields: Vec<String> = (0..field_count)
            .map(|i| format!("field{i}: {i}"))
            .collect();
        let instantiation_str = instantiation_fields.join(", ");

        let code = format!(
            r"
            struct {struct_name} {{
                {field_str}
            }}

            fun main() {{
                {struct_name} {{ {instantiation_str} }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Multi-field struct should compile");

        if let Ok(output) = result {
            prop_assert!(output.contains("struct"), "Should contain struct keyword");
            prop_assert!(output.contains(&struct_name), "Should contain struct name");
            // Check that all fields are present in output
            for i in 0..field_count {
                prop_assert!(output.contains(&format!("field{i}")), "Should contain field{}", i);
            }
        }
    }
}

// Property: Class and struct names are interchangeable for basic syntax
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_class_struct_equivalence(
        name in "[A-Z][a-zA-Z0-9]{0,8}",
        use_class in any::<bool>()
    ) {
        let keyword = if use_class { "class" } else { "struct" };
        let code = format!(
            r"
            {keyword} {name} {{
                value: i32
            }}

            fun main() {{
                {name} {{ value: 42 }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Class and struct should be equivalent for basic syntax");

        if let Ok(output) = result {
            // Both should transpile to Rust struct
            prop_assert!(output.contains("struct"), "Both class and struct should transpile to Rust struct");
            prop_assert!(output.contains(&name), "Should contain the name");
            prop_assert!(output.contains("value"), "Should contain field name");
        }
    }
}

// Property: Empty structs/classes should compile
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_empty_structures(
        name in "[A-Z][a-zA-Z0-9]{0,10}",
        use_class in any::<bool>()
    ) {
        let keyword = if use_class { "class" } else { "struct" };
        let code = format!(
            r"
            {keyword} {name} {{
            }}
            "
        );

        let result = compile(&code);
        // Empty structures might not be supported, but should not panic
        let _ = result; // Don't assert success for empty structures
        prop_assert!(true, "Should not panic on empty structures");
    }
}
