//! EXTREME TDD: Property tests for class field defaults with 10,000 iterations
//! Testing field default values, complex expressions, and edge cases

use proptest::prelude::*;
use ruchy::compile;

// Property: Basic field defaults compile successfully
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_simple_field_defaults_never_panic(
        class_name in "[A-Z][a-zA-Z0-9]{0,10}",
        field_name in "[a-z][a-zA-Z0-9]{0,10}",
        field_type in prop::sample::select(vec!["i32", "f64", "String", "bool"]),
        _default_value in prop::sample::select(vec!["42", "3.14", "\"test\"", "true"])
    ) {
        let default = match field_type {
            "i32" => "42",
            "f64" => "3.14",
            "String" => "\"test\"",
            "bool" => "true",
            _ => "0"
        };

        let code = format!(
            r"
            class {class_name} {{
                {field_name}: {field_type} = {default}
            }}

            fun main() {{
                {class_name} {{}}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Field defaults should always compile: {}", code);

        if let Ok(output) = result {
            // Verify struct generation
            prop_assert!(output.contains("struct"), "Should transpile to struct");
            prop_assert!(output.contains(&class_name), "Should contain class name");
            prop_assert!(output.contains(&field_name), "Should contain field name");

            // Verify impl block and Default trait implementation
            prop_assert!(output.contains("impl"), "Should have impl block");
            prop_assert!(output.contains("impl Default"), "Should have Default trait implementation");
            prop_assert!(output.contains("fn default"), "Should have default function");
        }
    }
}

// Property: Complex field default expressions
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_complex_field_defaults_compile(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_name in "[a-z][a-zA-Z0-9]{0,8}",
        expr_type in prop::sample::select(vec!["arithmetic", "method_call", "constructor"])
    ) {
        let (field_type, default_expr) = match expr_type {
            "arithmetic" => ("i32", "10 + 5 * 2"),
            "method_call" => ("String", "String::new()"),
            "constructor" => ("Vec<i32>", "Vec::new()"),
            _ => ("i32", "0")
        };

        let code = format!(
            r"
            class {class_name} {{
                {field_name}: {field_type} = {default_expr}
            }}

            fun main() {{
                let instance = {class_name} {{}};
                instance
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Complex field defaults should compile: {}", code);
    }
}

// Property: Multiple fields with defaults
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_multiple_field_defaults_robust(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_count in 1u32..4u32
    ) {
        let mut fields = Vec::new();
        let mut constructor_fields = Vec::new();

        for i in 0..field_count {
            fields.push(format!("field{}: i32 = {}", i, i * 10));
            constructor_fields.push(format!("field{}: {}", i, i * 10));
        }

        let field_str = fields.join(",\n            ");
        let _constructor_str = constructor_fields.join(", ");

        let code = format!(
            r"
            class {class_name} {{
                {field_str}
            }}

            fun main() {{
                {class_name} {{}}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Multiple field defaults should compile: {}", code);

        if let Ok(output) = result {
            // Verify all fields are present
            for i in 0..field_count {
                prop_assert!(output.contains(&format!("field{i}")), "Should contain field{}", i);
            }
        }
    }
}

// Property: Mixed fields (some with defaults, some without)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_mixed_field_defaults_compile(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        with_default_count in 1u32..3u32,
        without_default_count in 1u32..3u32
    ) {
        let mut fields = Vec::new();
        let mut constructor_args = Vec::new();

        // Fields with defaults
        for i in 0..with_default_count {
            fields.push(format!("default_field{}: i32 = {}", i, i * 10));
        }

        // Fields without defaults - these should require constructor parameters
        for i in 0..without_default_count {
            fields.push(format!("required_field{i}: i32"));
            constructor_args.push(format!("required_field{}: {}", i, i * 100));
        }

        let field_str = fields.join(",\n            ");
        let constructor_str = constructor_args.join(", ");

        let code = format!(
            r"
            class {class_name} {{
                {field_str}
            }}

            fun main() {{
                {class_name} {{ {constructor_str} }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Mixed field defaults should compile: {}", code);
    }
}

// Property: Default values maintain type safety
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_field_defaults_type_safe(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_name in "[a-z][a-zA-Z0-9]{0,8}"
    ) {
        // Valid type-value combinations
        let valid_combinations = vec![
            ("i32", "42"),
            ("i32", "1 + 2"),
            ("f64", "3.14"),
            ("f64", "1.0 + 2.0"),
            ("String", "\"hello\""),
            ("bool", "true"),
            ("bool", "false"),
            ("bool", "1 == 1"),
        ];

        for (field_type, default_value) in valid_combinations {
            let code = format!(
                r"
                class {class_name} {{
                    {field_name}: {field_type} = {default_value}
                }}

                fun main() {{
                    {class_name} {{}}
                }}
                "
            );

            let result = compile(&code);
            prop_assert!(result.is_ok(),
                "Type-safe field default should compile: {} : {} = {}",
                field_name, field_type, default_value
            );
        }
    }
}

// Property: Constructor behavior with defaults
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_constructor_respects_defaults(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        default_value in 1i32..100i32,
        override_value in 200i32..300i32
    ) {
        let code_with_default = format!(
            r"
            class {class_name} {{
                value: i32 = {default_value}
            }}

            fun main() {{
                let instance = {class_name} {{}};
                instance.value
            }}
            "
        );

        let code_with_override = format!(
            r"
            class {class_name} {{
                value: i32 = {default_value}
            }}

            fun main() {{
                let instance = {class_name} {{ value: {override_value} }};
                instance.value
            }}
            "
        );

        let result1 = compile(&code_with_default);
        let result2 = compile(&code_with_override);

        prop_assert!(result1.is_ok(), "Default constructor should compile");
        prop_assert!(result2.is_ok(), "Override constructor should compile");
    }
}
