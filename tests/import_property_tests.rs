#![allow(clippy::unwrap_used)] // Property tests need unwrap
#![allow(dead_code)] // Some test strategies may not be used

use proptest::prelude::*;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{ExprKind, ImportItem};
use ruchy::frontend::parser::Parser;

// Strategy for generating valid identifier names (excluding keywords)
fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_]{0,10}"
        .prop_map(|s| s)
        .prop_filter("must not be a keyword", |s| {
            !matches!(
                s.as_str(),
                "if" | "else"
                    | "match"
                    | "case"
                    | "fn"
                    | "let"
                    | "mut"
                    | "const"
                    | "for"
                    | "while"
                    | "break"
                    | "continue"
                    | "return"
                    | "import"
                    | "export"
                    | "module"
                    | "struct"
                    | "enum"
                    | "impl"
                    | "trait"
                    | "pub"
                    | "private"
                    | "as"
                    | "in"
                    | "where"
                    | "async"
                    | "await"
                    | "yield"
                    | "move"
                    | "ref"
                    | "static"
                    | "self"
                    | "Self"
                    | "super"
                    | "crate"
                    | "extern"
                    | "unsafe"
                    | "use"
                    | "mod"
                    | "type"
                    | "typeof"
                    | "sizeof"
                    | "alignof"
                    | "offsetof"
                    | "true"
                    | "false"
                    | "null"
                    | "nil"
                    | "undefined"
                    | "NaN"
                    | "Infinity"
                    | "and"
                    | "or"
                    | "not"
                    | "is"
                    | "from"
                    | "with"
                    | "try"
                    | "catch"
                    | "finally"
                    | "throw"
                    | "throws"
                    | "default"
                    | "do"
                    | "switch"
                    | "df"
            )
        })
}

// Strategy for generating module paths
fn module_path_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(identifier_strategy(), 1..5).prop_map(|parts| parts.join("::"))
}

// Strategy for generating import items
fn import_item_strategy() -> impl Strategy<Value = ImportItem> {
    prop_oneof![
        identifier_strategy().prop_map(ImportItem::Named),
        (identifier_strategy(), identifier_strategy())
            .prop_map(|(name, alias)| ImportItem::Aliased { name, alias }),
        Just(ImportItem::Wildcard),
    ]
}

proptest! {
    #[test]
    fn test_simple_import_parsing(path in module_path_strategy()) {
        let input = format!("import {path}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Import { path: parsed_path, items } = &expr.kind {
            prop_assert_eq!(parsed_path, &path);
            // Simple import should have one item with the last part of the path
            prop_assert_eq!(items.len(), 1);
            let expected_name = path.split("::").last().unwrap();
            prop_assert!(matches!(&items[0], ImportItem::Named(name) if name == expected_name));
        } else {
            prop_assert!(false, "Expected Import expression");
        }
    }

    #[test]
    fn test_wildcard_import_parsing(path in module_path_strategy()) {
        let input = format!("import {path}::*");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Import { path: parsed_path, items } = &expr.kind {
            prop_assert_eq!(parsed_path, &path);
            prop_assert_eq!(items.len(), 1);
            prop_assert!(matches!(&items[0], ImportItem::Wildcard));
        } else {
            prop_assert!(false, "Expected Import expression");
        }
    }

    #[test]
    fn test_aliased_import_parsing(path in module_path_strategy(), alias in identifier_strategy()) {
        let input = format!("import {path} as {alias}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Import { path: parsed_path, items } = &expr.kind {
            prop_assert_eq!(parsed_path, &path);
            prop_assert_eq!(items.len(), 1);
            let expected_name = path.split("::").last().unwrap();
            match &items[0] {
                ImportItem::Aliased { name, alias: a } => {
                    prop_assert_eq!(name, expected_name);
                    prop_assert_eq!(a, &alias);
                }
                _ => prop_assert!(false, "Expected Aliased import item"),
            }
        } else {
            prop_assert!(false, "Expected Import expression");
        }
    }

    #[test]
    fn test_multiple_imports_parsing(
        path in module_path_strategy(),
        items in prop::collection::vec(identifier_strategy(), 1..5)
    ) {
        let items_str = items.join(", ");
        let input = format!("import {path}::{{{items_str}}}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Import { path: parsed_path, items: parsed_items } = &expr.kind {
            prop_assert_eq!(parsed_path, &path);
            prop_assert_eq!(parsed_items.len(), items.len());

            for (i, expected_name) in items.iter().enumerate() {
                prop_assert!(matches!(&parsed_items[i], ImportItem::Named(name) if name == expected_name));
            }
        } else {
            prop_assert!(false, "Expected Import expression");
        }
    }

    #[test]
    fn test_import_transpilation_roundtrip(path in module_path_strategy()) {
        let input = format!("import {path}");
        let ast = Parser::new(&input).parse();

        prop_assert!(ast.is_ok());
        let ast = ast.unwrap();

        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);

        prop_assert!(result.is_ok());
        let output = result.unwrap().to_string();

        // The transpiled output should be a valid Rust use statement
        let expected = format!("use {path}");
        let normalize = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        prop_assert!(normalize(&output).contains(&normalize(&expected)));
    }

    #[test]
    fn test_import_with_mixed_items(
        path in module_path_strategy(),
        names in prop::collection::vec(identifier_strategy(), 1..3),
        aliases in prop::collection::vec((identifier_strategy(), identifier_strategy()), 0..2)
    ) {
        let mut items_parts = Vec::new();
        for name in &names {
            items_parts.push(name.clone());
        }
        for (name, alias) in &aliases {
            items_parts.push(format!("{name} as {alias}"));
        }

        if items_parts.is_empty() {
            return Ok(()); // Skip empty case
        }

        let items_str = items_parts.join(", ");
        let input = format!("import {path}::{{{items_str}}}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Import { path: parsed_path, items: parsed_items } = &expr.kind {
            prop_assert_eq!(parsed_path, &path);
            prop_assert_eq!(parsed_items.len(), names.len() + aliases.len());
        } else {
            prop_assert!(false, "Expected Import expression");
        }
    }
}

// Property tests for module parsing
proptest! {
    #[test]
    fn test_empty_module_parsing(name in identifier_strategy()) {
        let input = format!("module {name} {{}}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Module { name: module_name, .. } = &expr.kind {
            prop_assert_eq!(module_name, &name);
        } else {
            prop_assert!(false, "Expected Module expression");
        }
    }

    #[test]
    fn test_module_with_content_parsing(name in identifier_strategy(), value in 0i64..10000) {
        let input = format!("module {name} {{ {value} }}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Module { name: module_name, body } = &expr.kind {
            prop_assert_eq!(module_name, &name);
            // Verify the body contains the literal value
            prop_assert!(matches!(&body.kind, ExprKind::Literal(_)));
        } else {
            prop_assert!(false, "Expected Module expression");
        }
    }
}

// Property tests for export parsing
proptest! {
    #[test]
    fn test_single_export_parsing(name in identifier_strategy()) {
        let input = format!("export {name}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Export { items } = &expr.kind {
            prop_assert_eq!(items.len(), 1);
            prop_assert_eq!(&items[0], &name);
        } else {
            prop_assert!(false, "Expected Export expression");
        }
    }

    #[test]
    fn test_multiple_exports_parsing(names in prop::collection::vec(identifier_strategy(), 1..5)) {
        let items_str = names.join(", ");
        let input = format!("export {{ {items_str} }}");
        let result = Parser::new(&input).parse();

        prop_assert!(result.is_ok());
        let expr = result.unwrap();

        if let ExprKind::Export { items } = &expr.kind {
            prop_assert_eq!(items.len(), names.len());
            for name in &names {
                prop_assert!(items.contains(name));
            }
        } else {
            prop_assert!(false, "Expected Export expression");
        }
    }
}
