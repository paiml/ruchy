//! EXTREME TDD: Property-based tests for class inheritance
//! Tests with 10,000+ random iterations for robustness

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn test_inheritance_with_random_names(
        base_name in "[A-Z][a-zA-Z0-9]{0,10}",
        derived_name in "[A-Z][a-zA-Z0-9]{0,10}",
        field_name in "[a-z][a-zA-Z0-9_]{0,10}",
        field_type in prop::sample::select(vec!["i32", "f64", "String", "bool"])
    ) {
        // Skip if names are the same or reserved
        if base_name == derived_name || is_reserved(&base_name) || is_reserved(&derived_name) {
            return Ok(());
        }

        let code = format!(r"
            class {base_name} {{
                {field_name}: {field_type},
            }}

            class {derived_name} : {base_name} {{
                extra: i32,
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        // Verify inheritance is parsed
        // The AST should have a Block with two classes
        if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
            assert_eq!(exprs.len(), 2, "Should have two classes");

            // Check the second class has inheritance
            if let ruchy::frontend::ast::ExprKind::Class { name, superclass, .. } = &exprs[1].kind {
                assert_eq!(name, &derived_name);
                assert!(superclass.is_some(), "Should have superclass");
                assert_eq!(superclass.as_ref().unwrap(), &base_name);
            }
        }
    }

    #[test]
    fn test_inheritance_chain_random(
        names in prop::collection::vec("[A-Z][a-zA-Z0-9]{0,5}", 2..5)
    ) {
        // Ensure unique names
        let mut unique_names = std::collections::HashSet::new();
        for name in &names {
            if is_reserved(name) {
                return Ok(());
            }
            unique_names.insert(name.clone());
        }
        if unique_names.len() != names.len() {
            return Ok(()); // Skip if names aren't unique
        }

        // Build inheritance chain: A -> B -> C -> ...
        let mut classes = Vec::new();
        for i in 0..names.len() {
            let inheritance = if i > 0 {
                format!(": {}", names[i - 1])
            } else {
                String::new()
            };
            classes.push(format!(
                "class {} {} {{ field{}: i32, }}",
                names[i], inheritance, i
            ));
        }
        let code = classes.join("\n");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        // Verify inheritance chain
        if let ruchy::frontend::ast::ExprKind::Block(exprs) = &ast.kind {
            assert_eq!(exprs.len(), names.len(), "Should have all classes");

            for i in 1..names.len() {
                if let ruchy::frontend::ast::ExprKind::Class { superclass, .. } = &exprs[i].kind {
                    assert_eq!(superclass.as_ref().unwrap(), &names[i - 1],
                               "Class {} should inherit from {}", names[i], names[i - 1]);
                }
            }
        }
    }

    #[test]
    fn test_trait_mixing_random(
        class_name in "[A-Z][a-zA-Z0-9]{0,10}",
        base_name in "[A-Z][a-zA-Z0-9]{0,10}",
        trait_names in prop::collection::vec("[A-Z][a-zA-Z0-9]{0,10}", 1..4)
    ) {
        // Skip if names conflict or are reserved
        if class_name == base_name || is_reserved(&class_name) || is_reserved(&base_name) {
            return Ok(());
        }
        for trait_name in &trait_names {
            if is_reserved(trait_name) || trait_name == &class_name || trait_name == &base_name {
                return Ok(());
            }
        }

        let traits_str = trait_names.join(" + ");
        let code = format!(r"
            class {class_name} : {base_name} + {traits_str} {{
                value: i32,
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        // Verify both superclass and traits are parsed
        if let ruchy::frontend::ast::ExprKind::Class { name, superclass, traits, .. } = &ast.kind {
            assert_eq!(name, &class_name);
            assert_eq!(superclass.as_ref().unwrap(), &base_name);
            assert_eq!(traits.len(), trait_names.len(), "Should have all traits");
            for (i, trait_name) in trait_names.iter().enumerate() {
                assert_eq!(&traits[i], trait_name, "Trait should match");
            }
        }
    }

    #[test]
    fn test_visibility_inheritance(
        is_pub_base in prop::bool::ANY,
        is_pub_derived in prop::bool::ANY
    ) {
        let base_vis = if is_pub_base { "pub " } else { "" };
        let derived_vis = if is_pub_derived { "pub " } else { "" };

        let code = format!(r"
            {base_vis}class Base {{
                value: i32,
            }}

            {derived_vis}class Derived : Base {{
                extra: String,
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast).expect("Should transpile");
        let result_str = result.to_string();

        if is_pub_base {
            assert!(result_str.contains("pub struct Base"),
                    "Base should be public");
        }
        if is_pub_derived {
            assert!(result_str.contains("pub struct Derived"),
                    "Derived should be public");
        }
    }
}

fn is_reserved(word: &str) -> bool {
    matches!(
        word,
        "new"
            | "self"
            | "super"
            | "type"
            | "match"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
            | "let"
            | "mut"
            | "pub"
            | "struct"
            | "enum"
            | "fn"
            | "fun"
            | "class"
            | "trait"
            | "impl"
            | "default"
            | "static"
            | "const"
            | "async"
            | "await"
            | "yield"
            | "move"
            | "ref"
            | "typeof"
            | "sizeof"
            | "as"
            | "in"
            | "where"
            | "true"
            | "false"
            | "null"
            | "Self"
            | "Result"
            | "Option"
            | "Vec"
            | "String"
            | "Box"
    )
}
