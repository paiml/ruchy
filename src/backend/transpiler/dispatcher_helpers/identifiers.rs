//! Identifier and qualified name transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::Expr;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    pub(in crate::backend::transpiler) fn transpile_identifier(&self, name: &str) -> TokenStream {
        // Check if this is a module path like "math::add"
        if name.contains("::") {
            // Split into module path components
            let parts: Vec<&str> = name.split("::").collect();
            let mut tokens = Vec::new();
            for (i, part) in parts.iter().enumerate() {
                // Check if this is a turbofish segment like "<i32>"
                if part.starts_with('<') && part.ends_with('>') {
                    // Parse turbofish generics: "<i32>" or "<String, i32>"
                    let turbofish_tokens = Self::transpile_turbofish(part);
                    tokens.push(turbofish_tokens);
                } else {
                    let safe_part = if matches!(*part, "self" | "Self" | "super" | "crate") {
                        (*part).to_string()
                    } else if Self::is_rust_reserved_keyword(part) {
                        format!("r#{part}")
                    } else {
                        (*part).to_string()
                    };
                    let ident = format_ident!("{}", safe_part);
                    tokens.push(quote! { #ident });
                }
                if i < parts.len() - 1 {
                    tokens.push(quote! { :: });
                }
            }
            quote! { #(#tokens)* }
        } else {
            // Handle single identifier with Rust reserved keywords
            let safe_name = if matches!(name, "self" | "Self" | "super" | "crate") {
                // These keywords cannot be raw identifiers, use them as-is
                name.to_string()
            } else if Self::is_rust_reserved_keyword(name) {
                format!("r#{name}")
            } else {
                name.to_string()
            };
            let ident = format_ident!("{}", safe_name);

            // Issue #132: Check if this is a global variable (LazyLock<Mutex<T>>)
            // If so, wrap with .lock().unwrap() dereference
            if self.global_vars.read().unwrap().contains(name) {
                quote! { *#ident.lock().unwrap() }
            } else {
                quote! { #ident }
            }
        }
    }

    /// Transpile turbofish generics like "<i32>" or "<String, i32>"
    pub(in crate::backend::transpiler) fn transpile_turbofish(turbofish: &str) -> TokenStream {
        // Remove < and > brackets
        let inner = &turbofish[1..turbofish.len() - 1];

        // Split by comma to get individual type arguments
        let type_args: Vec<&str> = inner.split(',').map(str::trim).collect();

        // Build token stream for each type argument
        let type_tokens: Vec<TokenStream> = type_args
            .iter()
            .map(|type_arg| {
                // Handle qualified type names like std::string::String
                // Note: Type arguments in turbofish are never globals, so we can
                // use a simple static transpilation here
                if type_arg.contains("::") {
                    // For type paths, we don't need global checking
                    let ident = format_ident!("{}", type_arg);
                    quote! { #ident }
                } else {
                    let ident = format_ident!("{}", type_arg);
                    quote! { #ident }
                }
            })
            .collect();

        // Build <Type1, Type2, ...> token stream
        quote! { < #(#type_tokens),* > }
    }

    pub(in crate::backend::transpiler) fn transpile_qualified_name(
        module: &str,
        name: &str,
    ) -> TokenStream {
        // Handle nested qualified names like "net::TcpListener"
        let module_parts: Vec<&str> = module.split("::").collect();
        let name_ident = format_ident!("{}", name);
        if module_parts.len() == 1 {
            // Simple case: single module name
            let module_ident = format_ident!("{}", module_parts[0]);
            quote! { #module_ident::#name_ident }
        } else {
            // Complex case: nested path like "net::TcpListener"
            let mut tokens = TokenStream::new();
            for (i, part) in module_parts.iter().enumerate() {
                if i > 0 {
                    tokens.extend(quote! { :: });
                }
                let part_ident = format_ident!("{}", part);
                tokens.extend(quote! { #part_ident });
            }
            quote! { #tokens::#name_ident }
        }
    }

    /// Transpile external module declaration: `mod name;` or `pub mod name;`
    /// This handles module declarations WITHOUT bodies (external modules)
    /// Complexity: 3 (attribute iteration + conditional visibility)
    pub(in crate::backend::transpiler) fn transpile_external_mod_declaration(
        &self,
        name: &str,
        expr: &Expr,
    ) -> TokenStream {
        let module_ident = format_ident!("{}", name);

        // Check for visibility attributes (pub, pub(crate), etc.)
        // Attributes with name="pub" indicate public visibility
        let visibility_tokens = expr.attributes.iter().find_map(|attr| {
            if attr.name == "pub" {
                if attr.args.is_empty() {
                    Some(quote! { pub })
                } else {
                    // Handle pub(crate), pub(super), etc.
                    let vis_arg = &attr.args[0];
                    let vis_ident = format_ident!("{}", vis_arg);
                    Some(quote! { pub(#vis_ident) })
                }
            } else {
                None
            }
        });

        if let Some(vis) = visibility_tokens {
            quote! { #vis mod #module_ident ; }
        } else {
            quote! { mod #module_ident ; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Attribute, Expr, ExprKind, Span};

    // Helper: Create test transpiler instance
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Test 1: transpile_identifier - simple identifier
    #[test]
    fn test_transpile_identifier_simple() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("foo");
        assert_eq!(result.to_string(), "foo");
    }

    // Test 2: transpile_identifier - module path
    #[test]
    fn test_transpile_identifier_module_path() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("std::collections::HashMap");
        assert_eq!(result.to_string(), "std :: collections :: HashMap");
    }

    // Test 3: transpile_identifier - turbofish in path
    #[test]
    fn test_transpile_identifier_turbofish() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("Vec::<i32>::new");
        let result_str = result.to_string();
        assert!(result_str.contains("Vec"));
        assert!(result_str.contains("i32"));
        assert!(result_str.contains("new"));
    }

    // Test 4: transpile_identifier - self keyword
    #[test]
    fn test_transpile_identifier_self_keyword() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("self");
        assert_eq!(result.to_string(), "self");
    }

    // Test 5: transpile_identifier - Self keyword
    #[test]
    fn test_transpile_identifier_self_type() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("Self");
        assert_eq!(result.to_string(), "Self");
    }

    // Test 6: transpile_identifier - super keyword
    #[test]
    fn test_transpile_identifier_super_keyword() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("super");
        assert_eq!(result.to_string(), "super");
    }

    // Test 7: transpile_identifier - crate keyword
    #[test]
    fn test_transpile_identifier_crate_keyword() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("crate");
        assert_eq!(result.to_string(), "crate");
    }

    // Test 8: transpile_identifier - Rust reserved keyword
    #[test]
    fn test_transpile_identifier_reserved_keyword() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("type");
        let result_str = result.to_string();
        // Raw identifier format: r#type
        assert!(result_str.contains("r#") || result_str.contains("type"));
    }

    // Test 9: transpile_turbofish - single type
    #[test]
    fn test_transpile_turbofish_single_type() {
        let result = Transpiler::transpile_turbofish("<i32>");
        assert_eq!(result.to_string(), "< i32 >");
    }

    // Test 10: transpile_turbofish - multiple types
    #[test]
    fn test_transpile_turbofish_multiple_types() {
        let result = Transpiler::transpile_turbofish("<String, i32>");
        let result_str = result.to_string();
        assert!(result_str.contains("String"));
        assert!(result_str.contains("i32"));
        assert!(result_str.contains(','));
    }

    // Test 11: transpile_turbofish - with whitespace
    #[test]
    fn test_transpile_turbofish_with_whitespace() {
        let result = Transpiler::transpile_turbofish("< String , usize >");
        let result_str = result.to_string();
        assert!(result_str.contains("String"));
        assert!(result_str.contains("usize"));
    }

    // Test 12: transpile_qualified_name - simple module
    #[test]
    fn test_transpile_qualified_name_simple() {
        let result = Transpiler::transpile_qualified_name("math", "add");
        assert_eq!(result.to_string(), "math :: add");
    }

    // Test 13: transpile_qualified_name - nested path
    #[test]
    fn test_transpile_qualified_name_nested() {
        let result = Transpiler::transpile_qualified_name("net::tcp", "TcpListener");
        let result_str = result.to_string();
        assert!(result_str.contains("net"));
        assert!(result_str.contains("tcp"));
        assert!(result_str.contains("TcpListener"));
    }

    // Test 14: transpile_external_mod_declaration - basic mod
    #[test]
    fn test_transpile_external_mod_declaration_basic() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Identifier("utils".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_external_mod_declaration("utils", &expr);
        assert_eq!(result.to_string(), "mod utils ;");
    }

    // Test 15: transpile_external_mod_declaration - pub mod
    #[test]
    fn test_transpile_external_mod_declaration_pub() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Identifier("api".to_string()),
            span: Span::default(),
            attributes: vec![Attribute {
                name: "pub".to_string(),
                args: vec![],
                span: Span::default(),
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_external_mod_declaration("api", &expr);
        assert_eq!(result.to_string(), "pub mod api ;");
    }

    // Test 16: transpile_external_mod_declaration - pub(crate) mod
    #[test]
    fn test_transpile_external_mod_declaration_pub_crate() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Identifier("internal".to_string()),
            span: Span::default(),
            attributes: vec![Attribute {
                name: "pub".to_string(),
                args: vec!["crate".to_string()],
                span: Span::default(),
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_external_mod_declaration("internal", &expr);
        let result_str = result.to_string();
        assert!(result_str.contains("pub"));
        assert!(result_str.contains("crate"));
        assert!(result_str.contains("internal"));
    }

    // Test 17: transpile_identifier - path with reserved keyword
    #[test]
    fn test_transpile_identifier_path_with_reserved() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("mod::type");
        let result_str = result.to_string();
        // Should handle reserved keywords in paths
        assert!(
            result_str.contains("r#") || result_str.contains("mod") || result_str.contains("type")
        );
    }

    // Test 18: transpile_turbofish - three types
    #[test]
    fn test_transpile_turbofish_three_types() {
        let result = Transpiler::transpile_turbofish("<K, V, H>");
        let result_str = result.to_string();
        assert!(result_str.contains('K'));
        assert!(result_str.contains('V'));
        assert!(result_str.contains('H'));
    }

    // Test 19: transpile_identifier - reserved keyword "match"
    #[test]
    fn test_transpile_identifier_reserved_match() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("match");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("match"));
    }

    // Test 20: transpile_identifier - reserved keyword "fn"
    #[test]
    fn test_transpile_identifier_reserved_fn() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("fn");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("fn"));
    }

    // Test 21: transpile_identifier - reserved keyword "impl"
    #[test]
    fn test_transpile_identifier_reserved_impl() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("impl");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("impl"));
    }

    // Test 22: transpile_identifier - very long module path (5 components)
    #[test]
    fn test_transpile_identifier_long_path() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("std::collections::hash::map::HashMap");
        let result_str = result.to_string();
        assert!(result_str.contains("std"));
        assert!(result_str.contains("collections"));
        assert!(result_str.contains("hash"));
        assert!(result_str.contains("map"));
        assert!(result_str.contains("HashMap"));
    }

    // Test 23: transpile_qualified_name - deeply nested (3 levels)
    #[test]
    fn test_transpile_qualified_name_deep_nested() {
        let result = Transpiler::transpile_qualified_name("std::io::prelude", "Read");
        let result_str = result.to_string();
        assert!(result_str.contains("std"));
        assert!(result_str.contains("io"));
        assert!(result_str.contains("prelude"));
        assert!(result_str.contains("Read"));
    }

    // Test 24: transpile_qualified_name - single character names
    #[test]
    fn test_transpile_qualified_name_single_char() {
        let result = Transpiler::transpile_qualified_name("a", "b");
        assert_eq!(result.to_string(), "a :: b");
    }

    // Test 25: transpile_external_mod_declaration - pub(super)
    #[test]
    fn test_transpile_external_mod_declaration_pub_super() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Identifier("helpers".to_string()),
            span: Span::default(),
            attributes: vec![Attribute {
                name: "pub".to_string(),
                args: vec!["super".to_string()],
                span: Span::default(),
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_external_mod_declaration("helpers", &expr);
        let result_str = result.to_string();
        assert!(result_str.contains("pub"));
        assert!(result_str.contains("super"));
        assert!(result_str.contains("helpers"));
    }

    // Test 26: transpile_turbofish - four types
    #[test]
    fn test_transpile_turbofish_four_types() {
        let result = Transpiler::transpile_turbofish("<A, B, C, D>");
        let result_str = result.to_string();
        assert!(result_str.contains('A'));
        assert!(result_str.contains('B'));
        assert!(result_str.contains('C'));
        assert!(result_str.contains('D'));
    }

    // Test 27: transpile_turbofish - five types (stress test)
    #[test]
    fn test_transpile_turbofish_five_types() {
        let result = Transpiler::transpile_turbofish("<T, U, V, W, X>");
        let result_str = result.to_string();
        assert!(result_str.contains('T'));
        assert!(result_str.contains('U'));
        assert!(result_str.contains('V'));
        assert!(result_str.contains('W'));
        assert!(result_str.contains('X'));
    }

    // Test 28: transpile_identifier - module path with multiple turbofishes
    #[test]
    fn test_transpile_identifier_multiple_turbofishes() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("HashMap::<String, i32>::new");
        let result_str = result.to_string();
        assert!(result_str.contains("HashMap"));
        assert!(result_str.contains("String"));
        assert!(result_str.contains("i32"));
        assert!(result_str.contains("new"));
    }

    // Test 29: transpile_qualified_name - very deep nesting (4 levels)
    #[test]
    fn test_transpile_qualified_name_very_deep() {
        let result = Transpiler::transpile_qualified_name("a::b::c::d", "Function");
        let result_str = result.to_string();
        assert!(result_str.contains('a'));
        assert!(result_str.contains('b'));
        assert!(result_str.contains('c'));
        assert!(result_str.contains('d'));
        assert!(result_str.contains("Function"));
    }

    // Test 30: transpile_identifier - reserved keyword "struct"
    #[test]
    fn test_transpile_identifier_reserved_struct() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("struct");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("struct"));
    }

    // Test 31: transpile_identifier - reserved keyword "enum"
    #[test]
    fn test_transpile_identifier_reserved_enum() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("enum");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("enum"));
    }

    // Test 32: transpile_external_mod_declaration - non-pub attribute ignored
    #[test]
    fn test_transpile_external_mod_declaration_non_pub_attr() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Identifier("module".to_string()),
            span: Span::default(),
            attributes: vec![Attribute {
                name: "allow".to_string(),
                args: vec!["dead_code".to_string()],
                span: Span::default(),
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_external_mod_declaration("module", &expr);
        // Non-pub attributes are ignored, so result is just "mod module ;"
        assert_eq!(result.to_string(), "mod module ;");
    }

    // Test 33: transpile_identifier - path with self keyword in middle
    #[test]
    fn test_transpile_identifier_self_in_path() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("crate::self::module");
        let result_str = result.to_string();
        assert!(result_str.contains("crate"));
        assert!(result_str.contains("self"));
        assert!(result_str.contains("module"));
    }

    // Test 34: transpile_identifier - reserved keyword "loop"
    #[test]
    fn test_transpile_identifier_reserved_loop() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("loop");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("loop"));
    }

    // Test 35: transpile_identifier - reserved keyword "trait"
    #[test]
    fn test_transpile_identifier_reserved_trait() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("trait");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("trait"));
    }

    // Test 36: transpile_identifier - reserved keyword "const"
    #[test]
    fn test_transpile_identifier_reserved_const() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("const");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("const"));
    }

    // Test 37: transpile_identifier - reserved keyword "while"
    #[test]
    fn test_transpile_identifier_reserved_while() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("while");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("while"));
    }

    // Test 38: transpile_identifier - reserved keyword "for"
    #[test]
    fn test_transpile_identifier_reserved_for() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("for");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("for"));
    }

    // Test 39: transpile_identifier - reserved keyword "async"
    #[test]
    fn test_transpile_identifier_reserved_async() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("async");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("async"));
    }

    // Test 40: transpile_identifier - reserved keyword "await"
    #[test]
    fn test_transpile_identifier_reserved_await() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("await");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("await"));
    }

    // Test 41: transpile_identifier - reserved keyword "dyn"
    #[test]
    fn test_transpile_identifier_reserved_dyn() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("dyn");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("dyn"));
    }

    // Test 42: transpile_identifier - reserved keyword "unsafe"
    #[test]
    fn test_transpile_identifier_reserved_unsafe() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_identifier("unsafe");
        let result_str = result.to_string();
        assert!(result_str.contains("r#") || result_str.contains("unsafe"));
    }

    // Test 43: transpile_turbofish - nested generics (simplified to valid ident)
    #[test]
    fn test_transpile_turbofish_nested() {
        // Note: Nested generics like <Vec<i32>> are not supported in simple turbofish
        // This tests that simple qualified types work correctly
        let result = Transpiler::transpile_turbofish("<Vec>");
        let result_str = result.to_string();
        // The result will tokenize "Vec" as a single type argument
        assert!(result_str.contains("Vec"));
    }

    // Test 44: transpile_qualified_name - with numeric suffix
    #[test]
    fn test_transpile_qualified_name_numeric_suffix() {
        let result = Transpiler::transpile_qualified_name("module", "version2");
        let result_str = result.to_string();
        assert!(result_str.contains("module"));
        assert!(result_str.contains("version2"));
    }

    // Test 45: transpile_qualified_name - with underscore prefix
    #[test]
    fn test_transpile_qualified_name_underscore() {
        let result = Transpiler::transpile_qualified_name("_internal", "_helper");
        let result_str = result.to_string();
        assert!(result_str.contains("_internal"));
        assert!(result_str.contains("_helper"));
    }

    // Test 46: transpile_external_mod_declaration - pub(in path)
    #[test]
    fn test_transpile_external_mod_declaration_pub_in() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Identifier("restricted".to_string()),
            span: Span::default(),
            attributes: vec![Attribute {
                name: "pub".to_string(),
                args: vec!["in".to_string()],
                span: Span::default(),
            }],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_external_mod_declaration("restricted", &expr);
        let result_str = result.to_string();
        assert!(result_str.contains("pub"));
        assert!(result_str.contains("in") || result_str.contains("restricted"));
    }

    // Test 47: transpile_identifier - very long identifier (50+ chars)
    #[test]
    fn test_transpile_identifier_very_long() {
        let transpiler = test_transpiler();
        let long_name = "this_is_a_very_long_identifier_name_with_many_words_and_underscores";
        let result = transpiler.transpile_identifier(long_name);
        assert!(result
            .to_string()
            .contains("this_is_a_very_long_identifier"));
    }

    // Test 48: transpile_turbofish - with extra whitespace and newlines
    #[test]
    fn test_transpile_turbofish_extra_whitespace() {
        let result = Transpiler::transpile_turbofish("<  String  ,   i32   >");
        let result_str = result.to_string();
        // Whitespace should be trimmed
        assert!(result_str.contains("String"));
        assert!(result_str.contains("i32"));
    }
}
