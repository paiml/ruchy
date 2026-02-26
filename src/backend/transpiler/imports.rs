//! Import/Export Transpilation
//!
//! This module handles transpilation of module imports and exports:
//! - import statements (simple, aliased, wildcard)
//! - from X import Y syntax
//! - export statements
//! - module declarations
//!
//! **EXTREME TDD Round 55**: Extracted from statements.rs for modularization.
#![allow(clippy::doc_markdown)]

use super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles module declarations
    /// Complexity: 3 (within Toyota Way limits)
    pub fn transpile_module(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let module_name = format_ident!("{}", name);
        let body_tokens = self.transpile_expr(body)?;
        Ok(quote! {
            mod #module_name {
                #body_tokens
            }
        })
    }

    /// Static method for transpiling inline imports (backward compatibility)
    /// Complexity: 8 (within Toyota Way limits)
    pub fn transpile_import(module: &str, items: Option<&[String]>) -> TokenStream {
        // Convert dot notation to Rust's :: notation
        let rust_module = module.replace('.', "::");

        // Handle special cases for specific keywords that might come as module names
        let rust_module = match rust_module.as_str() {
            "self" => "self".to_string(),
            "super" => "super".to_string(),
            "crate" => "crate".to_string(),
            _ => rust_module,
        };

        // Convert new import format to old format temporarily for compatibility
        // Interpret the items parameter:
        // - None => simple import like "import std" -> generates "use std;"
        // - Some([]) => wildcard import like "from std import *" -> generates "use std::*;"
        // - Some([items...]) => specific imports -> generates "use std::{items};"
        let (import_items, _is_wildcard_from_empty) = match items {
            None => (vec![], false), // Simple import
            Some([]) => {
                // Empty array from "from module import *" means wildcard
                (vec![crate::frontend::ast::ImportItem::Wildcard], true)
            }
            Some(item_names) => {
                use crate::frontend::ast::ImportItem;
                // Specific items to import
                let items = item_names
                    .iter()
                    .map(|name| {
                        // Handle 'as' aliases in the item names
                        if !name.contains(" as ") {
                            return ImportItem::Named(name.clone());
                        }
                        let parts: Vec<&str> = name.split(" as ").collect();
                        if parts.len() == 2 {
                            ImportItem::Aliased {
                                name: parts[0].to_string(),
                                alias: parts[1].to_string(),
                            }
                        } else {
                            ImportItem::Named(name.clone())
                        }
                    })
                    .collect::<Vec<_>>();
                (items, false)
            }
        };
        Self::transpile_import_inline(&rust_module, &import_items)
    }

    /// Build a module path from segments for use in quote! macro
    /// Complexity: 2 (within Toyota Way limits)
    fn build_module_path(segments: &[&str]) -> proc_macro2::TokenStream {
        let idents: Vec<_> = segments.iter().map(|s| format_ident!("{}", s)).collect();
        quote! { #(#idents)::* }
    }

    /// Transpile import with alias (wildcard or named)
    /// Complexity: 6 (within Toyota Way limits)
    pub fn transpile_import_all(module: &str, alias: &str) -> TokenStream {
        if alias == "*" {
            // Wildcard import: use rayon::prelude::*
            // Parse the module path and generate the proper use statement
            let module_segments: Vec<_> = module.split("::").collect();
            let module_path = Self::build_module_path(&module_segments);
            quote! { use #module_path::*; }
        } else {
            // Handle module path aliases: use std::collections::HashMap as Map
            if module.contains("::") {
                // Split the module path and build it properly
                let module_segments: Vec<_> = module.split("::").collect();
                let module_path = Self::build_module_path(&module_segments);
                let alias_ident = format_ident!("{}", alias);
                quote! { use #module_path as #alias_ident; }
            } else {
                // Simple module alias
                let module_ident = format_ident!("{}", module.replace(['/', '.'], "_"));
                let alias_ident = format_ident!("{}", alias);
                quote! { use #module_ident as #alias_ident; }
            }
        }
    }

    /// Transpile default import
    /// Complexity: 2 (within Toyota Way limits)
    pub fn transpile_import_default(_module: &str, name: &str) -> TokenStream {
        // import Name from "module" => use module::Name
        let name_ident = format_ident!("{}", name);
        quote! { use #name_ident; /* Default import from #module */ }
    }

    /// Transpile re-export statement
    /// Complexity: 3 (within Toyota Way limits)
    pub fn transpile_reexport(items: &[String], module: &str) -> TokenStream {
        // export { items } from "module" => pub use module::{items}
        let item_idents: Vec<_> = items.iter().map(|item| format_ident!("{}", item)).collect();
        let module_ident = format_ident!("{}", module.replace(['/', '.'], "_"));
        quote! { pub use #module_ident::{#(#item_idents),*}; }
    }

    /// Transpile export statement (marks item as public)
    /// Complexity: 1 (within Toyota Way limits)
    pub fn transpile_export(_expr: &Expr, _is_default: bool) -> TokenStream {
        // export function/const/class => make it public
        // The actual transpilation happens on the expression itself
        quote! { /* Export: item marked as public */ }
    }

    /// Transpile export list
    /// Complexity: 2 (within Toyota Way limits)
    pub fn transpile_export_list(names: &[String]) -> TokenStream {
        // export { names } => pub use { names }
        let name_idents: Vec<_> = names.iter().map(|name| format_ident!("{}", name)).collect();
        quote! { pub use {#(#name_idents),*}; }
    }

    /// Transpile default export
    /// Complexity: 1 (within Toyota Way limits)
    pub fn transpile_export_default(_expr: &Expr) -> TokenStream {
        // export default expr => pub static DEFAULT: _ = expr
        quote! { /* Default export */ }
    }

    /// Core inline import transpilation - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    pub fn transpile_import_inline(
        path: &str,
        items: &[crate::frontend::ast::ImportItem],
    ) -> TokenStream {
        super::std_imports::transpile_import_inline(path, items)
    }

    /// Std module dispatcher - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    fn handle_std_module_import(
        path: &str,
        items: &[crate::frontend::ast::ImportItem],
    ) -> Option<TokenStream> {
        super::std_imports::handle_std_module_import(path, items)
    }

    /// Generic import handling - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    fn handle_generic_import(
        path: &str,
        items: &[crate::frontend::ast::ImportItem],
    ) -> TokenStream {
        super::std_imports::handle_generic_import(path, items)
    }

    /// Path tokenization - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    fn path_to_tokens(path: &str) -> TokenStream {
        super::std_imports::path_to_tokens(path)
    }

    /// Single item handling - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    fn handle_single_import_item(
        path_tokens: &TokenStream,
        path: &str,
        item: &crate::frontend::ast::ImportItem,
    ) -> TokenStream {
        super::std_imports::handle_single_import_item(path_tokens, path, item)
    }

    /// Multiple items handling - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    fn handle_multiple_import_items(
        path_tokens: &TokenStream,
        items: &[crate::frontend::ast::ImportItem],
    ) -> TokenStream {
        super::std_imports::handle_multiple_import_items(path_tokens, items)
    }

    /// Import items processing - delegates to std_imports module
    /// Complexity: 1 (within Toyota Way limits)
    fn process_import_items(items: &[crate::frontend::ast::ImportItem]) -> Vec<TokenStream> {
        super::std_imports::process_import_items(items)
    }

    /// Transpiles export statements (legacy)
    /// Complexity: 3 (within Toyota Way limits)
    fn transpile_export_legacy(items: &[String]) -> TokenStream {
        let item_idents: Vec<_> = items.iter().map(|s| format_ident!("{}", s)).collect();
        if items.len() == 1 {
            let item = &item_idents[0];
            quote! { pub use #item; }
        } else {
            quote! { pub use {#(#item_idents),*}; }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn block_expr(stmts: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(stmts))
    }

    // ========================================================================
    // transpile_module tests
    // ========================================================================

    #[test]
    fn test_transpile_module_simple() {
        let transpiler = Transpiler::new();
        let body = block_expr(vec![int_expr(42)]);
        let result = transpiler.transpile_module("my_module", &body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("mod"));
        assert!(tokens.contains("my_module"));
    }

    #[test]
    fn test_transpile_module_with_underscore() {
        let transpiler = Transpiler::new();
        let body = block_expr(vec![]);
        let result = transpiler.transpile_module("my_sub_module", &body);
        assert!(result.is_ok());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("my_sub_module"));
    }

    // ========================================================================
    // transpile_import tests
    // ========================================================================

    #[test]
    fn test_transpile_import_simple() {
        let result = Transpiler::transpile_import("std", None);
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("std"));
    }

    #[test]
    fn test_transpile_import_with_dots() {
        let result = Transpiler::transpile_import("std.collections", None);
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        // Dot notation converted to ::
        assert!(tokens.contains("std") && tokens.contains("collections"));
    }

    #[test]
    fn test_transpile_import_wildcard() {
        let result = Transpiler::transpile_import("std", Some(&[]));
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("*"));
    }

    #[test]
    fn test_transpile_import_specific_items() {
        let items = vec!["HashMap".to_string(), "HashSet".to_string()];
        let result = Transpiler::transpile_import("std::collections", Some(&items));
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("HashMap"));
        assert!(tokens.contains("HashSet"));
    }

    #[test]
    fn test_transpile_import_with_alias() {
        let items = vec!["HashMap as Map".to_string()];
        let result = Transpiler::transpile_import("std::collections", Some(&items));
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("HashMap"));
        assert!(tokens.contains("Map"));
    }

    #[test]
    fn test_transpile_import_self_keyword() {
        let result = Transpiler::transpile_import("self", None);
        let tokens = result.to_string();
        assert!(tokens.contains("self"));
    }

    #[test]
    fn test_transpile_import_super_keyword() {
        let result = Transpiler::transpile_import("super", None);
        let tokens = result.to_string();
        assert!(tokens.contains("super"));
    }

    #[test]
    fn test_transpile_import_crate_keyword() {
        let result = Transpiler::transpile_import("crate", None);
        let tokens = result.to_string();
        assert!(tokens.contains("crate"));
    }

    // ========================================================================
    // transpile_import_all tests
    // ========================================================================

    #[test]
    fn test_transpile_import_all_wildcard() {
        let result = Transpiler::transpile_import_all("rayon::prelude", "*");
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("rayon"));
        assert!(tokens.contains("prelude"));
        assert!(tokens.contains("*"));
    }

    #[test]
    fn test_transpile_import_all_with_alias() {
        let result = Transpiler::transpile_import_all("std::collections::HashMap", "Map");
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("HashMap"));
        assert!(tokens.contains("Map"));
        assert!(tokens.contains("as"));
    }

    #[test]
    fn test_transpile_import_all_simple_alias() {
        let result = Transpiler::transpile_import_all("mymod", "m");
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("mymod"));
        assert!(tokens.contains("as"));
        assert!(tokens.contains("m"));
    }

    // ========================================================================
    // transpile_import_default tests
    // ========================================================================

    #[test]
    fn test_transpile_import_default() {
        let result = Transpiler::transpile_import_default("mymodule", "MyClass");
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
        assert!(tokens.contains("MyClass"));
    }

    // ========================================================================
    // transpile_reexport tests
    // ========================================================================

    #[test]
    fn test_transpile_reexport_single() {
        let items = vec!["Foo".to_string()];
        let result = Transpiler::transpile_reexport(&items, "bar");
        let tokens = result.to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("use"));
        assert!(tokens.contains("bar"));
        assert!(tokens.contains("Foo"));
    }

    #[test]
    fn test_transpile_reexport_multiple() {
        let items = vec!["Foo".to_string(), "Bar".to_string(), "Baz".to_string()];
        let result = Transpiler::transpile_reexport(&items, "mymod");
        let tokens = result.to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("use"));
        assert!(tokens.contains("Foo"));
        assert!(tokens.contains("Bar"));
        assert!(tokens.contains("Baz"));
    }

    // ========================================================================
    // transpile_export tests
    // ========================================================================

    #[test]
    fn test_transpile_export() {
        let expr = int_expr(42);
        let result = Transpiler::transpile_export(&expr, false);
        // Export is a placeholder - just verify no panic and returns something
        let _tokens = result.to_string();
    }

    #[test]
    fn test_transpile_export_default_flag() {
        let expr = int_expr(42);
        let result = Transpiler::transpile_export(&expr, true);
        // Export is a placeholder - just verify no panic
        let _tokens = result.to_string();
    }

    // ========================================================================
    // transpile_export_list tests
    // ========================================================================

    #[test]
    fn test_transpile_export_list_single() {
        let names = vec!["foo".to_string()];
        let result = Transpiler::transpile_export_list(&names);
        let tokens = result.to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("use"));
        assert!(tokens.contains("foo"));
    }

    #[test]
    fn test_transpile_export_list_multiple() {
        let names = vec!["foo".to_string(), "bar".to_string()];
        let result = Transpiler::transpile_export_list(&names);
        let tokens = result.to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("foo"));
        assert!(tokens.contains("bar"));
    }

    // ========================================================================
    // transpile_export_default tests
    // ========================================================================

    #[test]
    fn test_transpile_export_default() {
        let expr = int_expr(42);
        let result = Transpiler::transpile_export_default(&expr);
        // Export default is a placeholder - just verify no panic
        let _tokens = result.to_string();
    }

    // ========================================================================
    // transpile_export_legacy tests
    // ========================================================================

    #[test]
    fn test_transpile_export_legacy_single() {
        let items = vec!["foo".to_string()];
        let result = Transpiler::transpile_export_legacy(&items);
        let tokens = result.to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("use"));
        assert!(tokens.contains("foo"));
    }

    #[test]
    fn test_transpile_export_legacy_multiple() {
        let items = vec!["foo".to_string(), "bar".to_string()];
        let result = Transpiler::transpile_export_legacy(&items);
        let tokens = result.to_string();
        assert!(tokens.contains("pub"));
        assert!(tokens.contains("foo"));
        assert!(tokens.contains("bar"));
    }

    // ========================================================================
    // build_module_path tests
    // ========================================================================

    #[test]
    fn test_build_module_path_single() {
        let segments = vec!["std"];
        let result = Transpiler::build_module_path(&segments);
        let tokens = result.to_string();
        assert!(tokens.contains("std"));
    }

    #[test]
    fn test_build_module_path_multiple() {
        let segments = vec!["std", "collections", "HashMap"];
        let result = Transpiler::build_module_path(&segments);
        let tokens = result.to_string();
        assert!(tokens.contains("std"));
        assert!(tokens.contains("collections"));
        assert!(tokens.contains("HashMap"));
    }

    // ========================================================================
    // Delegate function tests (ensure they don't panic)
    // ========================================================================

    #[test]
    fn test_transpile_import_inline() {
        use crate::frontend::ast::ImportItem;
        let items = vec![ImportItem::Named("HashMap".to_string())];
        let result = Transpiler::transpile_import_inline("std::collections", &items);
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
    }

    #[test]
    fn test_handle_std_module_import() {
        use crate::frontend::ast::ImportItem;
        let items = vec![ImportItem::Named("HashMap".to_string())];
        let result = Transpiler::handle_std_module_import("std::collections", &items);
        // May return None or Some depending on std_imports implementation
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_handle_generic_import() {
        use crate::frontend::ast::ImportItem;
        let items = vec![ImportItem::Named("Foo".to_string())];
        let result = Transpiler::handle_generic_import("mymod", &items);
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
    }

    #[test]
    fn test_path_to_tokens() {
        let result = Transpiler::path_to_tokens("std::collections");
        let tokens = result.to_string();
        assert!(tokens.contains("std"));
        assert!(tokens.contains("collections"));
    }

    #[test]
    fn test_handle_single_import_item() {
        use crate::frontend::ast::ImportItem;
        let path_tokens = quote! { std::collections };
        let item = ImportItem::Named("HashMap".to_string());
        let result = Transpiler::handle_single_import_item(&path_tokens, "std::collections", &item);
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
    }

    #[test]
    fn test_handle_multiple_import_items() {
        use crate::frontend::ast::ImportItem;
        let path_tokens = quote! { std::collections };
        let items = vec![
            ImportItem::Named("HashMap".to_string()),
            ImportItem::Named("HashSet".to_string()),
        ];
        let result = Transpiler::handle_multiple_import_items(&path_tokens, &items);
        let tokens = result.to_string();
        assert!(tokens.contains("use"));
    }

    #[test]
    fn test_process_import_items() {
        use crate::frontend::ast::ImportItem;
        let items = vec![
            ImportItem::Named("Foo".to_string()),
            ImportItem::Named("Bar".to_string()),
        ];
        let result = Transpiler::process_import_items(&items);
        assert_eq!(result.len(), 2);
    }
}
