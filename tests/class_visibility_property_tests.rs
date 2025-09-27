//! EXTREME TDD: Property tests for class visibility modifiers with 10,000 iterations
//! Testing pub/mut modifiers for fields and methods

use proptest::prelude::*;
use ruchy::compile;

// Property: Basic visibility modifiers never panic
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_field_visibility_never_panics(
        class_name in "[A-Z][a-zA-Z0-9]{0,10}",
        field_name in "[a-z][a-zA-Z0-9]{0,10}",
        visibility in prop::sample::select(vec!["", "pub", "mut", "pub mut"])
    ) {
        let field_decl = if visibility.is_empty() {
            format!("{field_name}: i32")
        } else {
            format!("{visibility} {field_name}: i32")
        };

        let code = format!(
            r"
            class {class_name} {{
                {field_decl}
            }}

            fun main() {{
                {class_name} {{}}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Field visibility should always compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(output.contains("struct"), "Should transpile to struct");
            prop_assert!(output.contains(&class_name), "Should contain class name");
            prop_assert!(output.contains(&field_name), "Should contain field name");

            // Check visibility in struct definition
            if visibility.contains("pub") {
                prop_assert!(output.contains(&format!("pub {field_name}")),
                    "Public field should have pub modifier in struct");
            }
        }
    }
}

// Property: Method visibility modifiers
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_method_visibility_never_panics(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        method_name in "[a-z][a-zA-Z0-9]{0,8}",
        visibility in prop::sample::select(vec!["", "pub"]),
        self_param in prop::sample::select(vec!["&self", "&mut self", "self"])
    ) {
        let method_decl = if visibility.is_empty() {
            format!("fn {method_name}({self_param}) {{ 42 }}")
        } else {
            format!("{visibility} fn {method_name}({self_param}) {{ 42 }}")
        };

        let code = format!(
            r"
            class {class_name} {{
                {method_decl}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Method visibility should always compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(output.contains("impl"), "Should have impl block");
            prop_assert!(output.contains(&method_name), "Should contain method name");

            // Check method visibility in impl block
            if visibility.contains("pub") {
                prop_assert!(output.contains(&format!("pub fn {method_name}")),
                    "Public method should have pub modifier");
            } else {
                // Private methods should not have pub
                prop_assert!(!output.contains(&format!("pub fn {method_name}")),
                    "Private method should not have pub modifier");
            }
        }
    }
}

// Property: Complex visibility combinations
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_mixed_visibility_robust(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_count in 1u32..4u32
    ) {
        let visibility_options = ["", "pub", "mut", "pub mut"];
        let mut fields = Vec::new();

        for i in 0..field_count {
            let visibility = visibility_options[i as usize % visibility_options.len()];
            let field_decl = if visibility.is_empty() {
                format!("field{i}: i32")
            } else {
                format!("{visibility} field{i}: i32")
            };
            fields.push(field_decl);
        }

        let field_str = fields.join(",\n            ");

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
        prop_assert!(result.is_ok(), "Mixed visibility should compile: {}", code);

        if let Ok(output) = result {
            // Verify all fields are present
            for i in 0..field_count {
                prop_assert!(output.contains(&format!("field{i}")), "Should contain field{}", i);
            }
        }
    }
}

// Property: Mut fields enable modification
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_mut_fields_compile(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_name in "[a-z][a-zA-Z0-9]{0,8}"
    ) {
        let code = format!(
            r"
            class {class_name} {{
                mut {field_name}: i32 = 0
            }}

            fun main() {{
                let mut instance = {class_name} {{}};
                // In real usage, would modify field here
                instance
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Mut fields should compile: {}", code);

        if let Ok(output) = result {
            // Mut fields should not affect struct definition (mutability is usage-based in Rust)
            prop_assert!(output.contains("struct"), "Should have struct");
            prop_assert!(output.contains(&field_name), "Should contain field name");
        }
    }
}

// Property: Public fields are accessible
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_pub_fields_accessible(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_name in "[a-z][a-zA-Z0-9]{0,8}"
    ) {
        let code = format!(
            r"
            class {class_name} {{
                pub {field_name}: i32 = 42
            }}

            fun main() {{
                let instance = {class_name} {{}};
                instance.{field_name}  // Access public field
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Public field access should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(output.contains(&format!("pub {field_name}")),
                "Public field should have pub in struct definition");
        }
    }
}

// Property: Constructor visibility
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_constructor_visibility_compile(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        is_pub in any::<bool>()
    ) {
        let constructor_decl = if is_pub {
            "pub new() { self }"
        } else {
            "new() { self }"
        };

        let code = format!(
            r"
            class {class_name} {{
                {constructor_decl}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Constructor visibility should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(output.contains("impl"), "Should have impl block");
            if is_pub {
                prop_assert!(output.contains("pub fn new"), "Public constructor should have pub modifier");
            }
        }
    }
}
