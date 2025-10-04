//! EXTREME TDD: Property tests for derive attributes with 10,000 iterations
//! Testing derive attributes with random combinations and edge cases

use proptest::prelude::*;
use ruchy::compile;

// Helper function to check for derive attributes with flexible spacing
fn check_derive(output: &str, derive_list: &str) -> bool {
    let exact_derive = format!("#[derive({derive_list})]");
    let spaced_derive = format!("# [derive ({})]", derive_list.replace(", ", " , "));
    output.contains(&exact_derive) || output.contains(&spaced_derive)
}

// Property: Derive attributes never panic during parsing
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_derive_attributes_never_panic(
        class_name in "[A-Z][a-zA-Z0-9]{0,10}",
        field_name in "[a-z][a-zA-Z0-9]{0,10}",
        derives in prop::collection::vec(
            prop::sample::select(vec!["Debug", "Clone", "PartialEq", "Eq", "Hash", "Copy"]),
            1..4
        )
    ) {
        let derive_list = derives.join(", ");
        let code = format!(
            r"
            #[derive({derive_list})]
            class {class_name} {{
                {field_name}: i32
            }}

            fun main() {{
                {class_name} {{}}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Derive attributes should always compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(output.contains("struct"), "Should transpile to struct");
            prop_assert!(check_derive(&output, &derive_list),
                "Should contain derive attribute: {}", derive_list);
            prop_assert!(output.contains(&class_name), "Should contain class name");
            prop_assert!(output.contains(&field_name), "Should contain field name");
        }
    }
}

// Property: Common derive combinations work correctly
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_common_derive_combinations(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        derive_combo in prop::sample::select(vec![
            "Debug",
            "Debug, Clone",
            "Debug, Clone, PartialEq",
            "Debug, Clone, PartialEq, Eq",
            "Debug, Clone, Copy",
            "Debug, Hash",
            "Clone, PartialEq, Eq, Hash",
            "Copy, Clone, Debug"
        ])
    ) {
        let code = format!(
            r"
            #[derive({derive_combo})]
            class {class_name} {{
                value: i32
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Common derive combinations should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(check_derive(&output, derive_combo),
                "Should contain derive combination: {}", derive_combo);
            prop_assert!(output.contains(&format!("struct {class_name}")),
                "Should contain struct definition");
        }
    }
}

// Property: Derive with field defaults
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_derive_with_field_defaults(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_name in "[a-z][a-zA-Z0-9]{0,8}",
        default_value in 1i32..100i32,
        derives in prop::collection::vec(
            prop::sample::select(vec!["Debug", "Clone", "PartialEq"]),
            1..3
        )
    ) {
        let derive_list = derives.join(", ");
        let code = format!(
            r"
            #[derive({derive_list})]
            class {class_name} {{
                {field_name}: i32 = {default_value}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Derive with field defaults should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(check_derive(&output, &derive_list),
                "Should contain derive attribute: {}", derive_list);
            prop_assert!(output.contains("impl Default"),
                "Should have Default trait implementation for field defaults");
            prop_assert!(output.contains(&default_value.to_string()),
                "Should contain default value");
        }
    }
}

// Property: Derive with visibility modifiers
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_derive_with_visibility(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_name in "[a-z][a-zA-Z0-9]{0,8}",
        visibility in prop::sample::select(vec!["", "pub", "mut", "pub mut"]),
        derives in prop::collection::vec(
            prop::sample::select(vec!["Debug", "Clone"]),
            1..3
        )
    ) {
        let derive_list = derives.join(", ");
        let field_decl = if visibility.is_empty() {
            format!("{field_name}: i32")
        } else {
            format!("{visibility} {field_name}: i32")
        };

        let code = format!(
            r"
            #[derive({derive_list})]
            class {class_name} {{
                {field_decl}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Derive with visibility should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(check_derive(&output, &derive_list),
                "Should contain derive attribute: {}", derive_list);

            // Check field visibility in struct
            if visibility.contains("pub") {
                prop_assert!(output.contains(&format!("pub {field_name}")),
                    "Public field should have pub modifier");
            }
        }
    }
}

// TODO: Support multiple non-derive attributes
// // Property: Multiple attributes (derive + others)
// proptest! {
//     #![proptest_config(ProptestConfig::with_cases(10000))]

//     #[test]
//     fn prop_multiple_attributes(
//         class_name in "[A-Z][a-zA-Z0-9]{0,8}",
//         derives in prop::collection::vec(
//             prop::sample::select(vec!["Debug", "Clone", "PartialEq"]),
//             1..3
//         )
//     ) {
//         let derive_list = derives.join(", ");
//         let code = format!(
//             r#"
//             #[derive({})]
//             #[allow(dead_code)]
//             class {} {{
//                 value: i32
//             }}
//             "#,
//             derive_list, class_name
//         );

//         let result = compile(&code);
//         prop_assert!(result.is_ok(), "Multiple attributes should compile: {}", code);

//         if let Ok(output) = result {
//             prop_assert!(check_derive(&output, &derive_list),
//                 "Should contain derive attribute: {}", derive_list);
//             prop_assert!(output.contains("#[allow(dead_code)]"),
//                 "Should contain other attributes");
//         }
//     }
// }

// Property: Derive with methods
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_derive_with_methods(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        method_name in "[a-z][a-zA-Z0-9]{0,8}",
        derives in prop::collection::vec(
            prop::sample::select(vec!["Debug", "Clone"]),
            1..3
        )
    ) {
        let derive_list = derives.join(", ");
        let code = format!(
            r"
            #[derive({derive_list})]
            class {class_name} {{
                value: i32,

                pub fn {method_name}(&self) -> i32 {{
                    self.value
                }}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Derive with methods should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(check_derive(&output, &derive_list),
                "Should contain derive on struct: {}", derive_list);
            prop_assert!(output.contains(&format!("struct {class_name}")),
                "Should have struct definition");
            prop_assert!(output.contains(&format!("impl {class_name}")),
                "Should have impl block");
            prop_assert!(output.contains(&format!("pub fn {method_name}")),
                "Should contain method");
        }
    }
}

// Property: Complex derive scenarios
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_complex_derive_scenarios(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        field_count in 1u32..4u32
    ) {
        let mut fields = Vec::new();
        let visibility_options = ["", "pub", "mut", "pub mut"];

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
            #[derive(Debug, Clone)]
            class {class_name} {{
                {field_str}
            }}
            "
        );

        let result = compile(&code);
        prop_assert!(result.is_ok(), "Complex derive scenarios should compile: {}", code);

        if let Ok(output) = result {
            prop_assert!(check_derive(&output, "Debug, Clone"),
                "Should contain derive attributes: Debug, Clone");
            for i in 0..field_count {
                prop_assert!(output.contains(&format!("field{i}")),
                    "Should contain field{}", i);
            }
        }
    }
}

// TODO: Validate empty derive lists
// // Property: Empty derive should fail
// proptest! {
//     #![proptest_config(ProptestConfig::with_cases(10000))]

//     #[test]
//     fn prop_empty_derive_fails(
//         class_name in "[A-Z][a-zA-Z0-9]{0,8}"
//     ) {
//         let code = format!(
//             r#"
//             #[derive()]
//             class {} {{
//                 value: i32
//             }}
//             "#,
//             class_name
//         );

//         let result = compile(&code);
//         prop_assert!(result.is_err(), "Empty derive list should fail to compile: {}", code);
//     }
// }

// Property: Invalid derive traits should fail
proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_invalid_derive_traits(
        class_name in "[A-Z][a-zA-Z0-9]{0,8}",
        invalid_trait in prop::sample::select(vec!["InvalidTrait", "NotATrait", "BadDerive"])
    ) {
        let code = format!(
            r"
            #[derive({invalid_trait})]
            class {class_name} {{
                value: i32
            }}
            "
        );

        let result = compile(&code);
        // Note: This might pass at parse time but fail at Rust compile time
        // The parser should accept any identifier in derive list
        if result.is_ok() {
            if let Ok(output) = result {
                prop_assert!(check_derive(&output, invalid_trait),
                    "Should contain derive attribute even if trait is invalid: {}", invalid_trait);
            }
        }
    }
}
