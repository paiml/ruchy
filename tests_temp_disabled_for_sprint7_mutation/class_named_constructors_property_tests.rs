//! EXTREME TDD: Property-based tests for named constructors
//! Tests with 10,000+ random iterations for robustness

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn test_named_constructor_names_preserved(
        constructor_name in "[a-z][a-zA-Z0-9_]{0,20}",
        param_name in "[a-z][a-zA-Z0-9_]{0,10}",
        param_type in prop::sample::select(vec!["i32", "f64", "String", "bool"])
    ) {
        // Skip reserved keywords
        if constructor_name == "new" || constructor_name == "self" || constructor_name == "super"
           || constructor_name == "type" || constructor_name == "match" || constructor_name == "if"
           || constructor_name == "else" || constructor_name == "for" || constructor_name == "while"
           || constructor_name == "loop" || constructor_name == "break" || constructor_name == "continue"
           || constructor_name == "return" || constructor_name == "let" || constructor_name == "mut"
           || constructor_name == "pub" || constructor_name == "struct" || constructor_name == "enum"
           || constructor_name == "fn" || constructor_name == "fun" || constructor_name == "class"
           || constructor_name == "trait" || constructor_name == "impl" || constructor_name == "default"
           || constructor_name == "static" || constructor_name == "const" || constructor_name == "async"
           || constructor_name == "await" || constructor_name == "yield" || constructor_name == "move"
           || constructor_name == "ref" || constructor_name == "typeof" || constructor_name == "sizeof"
           || constructor_name == "as" || constructor_name == "in" || constructor_name == "where"
           || constructor_name == "true" || constructor_name == "false" || constructor_name == "null" {
            return Ok(());
        }

        let code = format!(r"
            class Test {{
                value: {param_type},

                new {constructor_name}({param_name}: {param_type}) {{
                    self.value = {param_name}
                }}
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        // Verify AST has the correct constructor name
        if let ruchy::frontend::ast::ExprKind::Class { constructors, .. } = &ast.kind {
            assert_eq!(constructors.len(), 1, "Should have 1 constructor");
            let ctor = &constructors[0];
            assert_eq!(ctor.name.as_ref().unwrap(), &constructor_name,
                       "Constructor name should be preserved");
        }

        // Verify transpilation includes the constructor name
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast).expect("Should transpile");
        let result_str = result.to_string();

        // Check that the method has the right name (might have spaces from quote!)
        assert!(result_str.contains(&format!("fn {constructor_name}")) ||
                result_str.contains(&format!("fn {constructor_name} ")),
                "Transpiled code should contain constructor method");
    }

    #[test]
    fn test_multiple_named_constructors_unique(
        names in prop::collection::vec("[a-z][a-zA-Z0-9_]{0,10}", 2..5)
    ) {
        // Ensure unique names and skip reserved keywords
        let mut unique_names = std::collections::HashSet::new();
        for name in &names {
            if name == "new" || name == "self" || name == "super" || name == "default"
               || name == "type" || name == "match" || name == "if" || name == "static" {
                return Ok(());
            }
            unique_names.insert(name.clone());
        }
        if unique_names.len() != names.len() {
            return Ok(()); // Skip if names aren't unique
        }

        let constructors = names.iter().map(|name| format!(
            "new {name}() {{ self.value = 0 }}"
        )).collect::<Vec<_>>().join("\n                ");

        let code = format!(r"
            class Test {{
                mut value: i32,
                {constructors}
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        // Verify all constructor names are preserved
        if let ruchy::frontend::ast::ExprKind::Class { constructors, .. } = &ast.kind {
            assert_eq!(constructors.len(), names.len(), "Should have all constructors");
            for (i, ctor) in constructors.iter().enumerate() {
                assert_eq!(ctor.name.as_ref().unwrap(), &names[i],
                           "Constructor name should match");
            }
        }
    }

    #[test]
    fn test_named_constructor_visibility(
        is_pub in prop::bool::ANY,
        constructor_name in "[a-z][a-zA-Z0-9_]{0,10}"
    ) {
        // Skip reserved keywords
        if constructor_name == "new" || constructor_name == "self" || constructor_name == "default"
           || constructor_name == "static" || constructor_name == "type" || constructor_name == "match" {
            return Ok(());
        }

        let visibility = if is_pub { "pub " } else { "" };
        let code = format!(r"
            class Test {{
                value: i32,

                {visibility}new {constructor_name}() {{
                    self.value = 42
                }}
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast).expect("Should transpile");
        let result_str = result.to_string();

        if is_pub {
            assert!(result_str.contains("pub fn") || result_str.contains("pub fn"),
                    "Public constructor should generate pub fn");
        }
    }

    #[test]
    fn test_named_constructor_with_params(
        num_params in 0usize..5,
        constructor_name in "[a-z][a-zA-Z0-9_]{0,10}"
    ) {
        // Skip reserved keywords
        if constructor_name == "new" || constructor_name == "self" || constructor_name == "default"
           || constructor_name == "static" || constructor_name == "super" {
            return Ok(());
        }

        let params = (0..num_params)
            .map(|i| format!("p{i}: i32"))
            .collect::<Vec<_>>()
            .join(", ");

        let assignments = (0..num_params)
            .map(|i| format!("self.field{i} = p{i}"))
            .collect::<Vec<_>>()
            .join("\n                    ");

        let fields = (0..num_params)
            .map(|i| format!("field{i}: i32,"))
            .collect::<Vec<_>>()
            .join("\n                ");

        let code = format!(r"
            class Test {{
                {fields}

                new {constructor_name}({params}) {{
                    {assignments}
                }}
            }}
        ");

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse successfully");

        // Verify parsing succeeded and has correct structure
        if let ruchy::frontend::ast::ExprKind::Class { constructors, fields, .. } = &ast.kind {
            assert_eq!(constructors.len(), 1, "Should have 1 constructor");
            assert_eq!(fields.len(), num_params, "Should have {num_params} fields");
            let ctor = &constructors[0];
            assert_eq!(ctor.params.len(), num_params, "Should have {num_params} params");
            assert_eq!(ctor.name.as_ref().unwrap(), &constructor_name);
        }
    }
}
