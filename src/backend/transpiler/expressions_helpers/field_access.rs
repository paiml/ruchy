//! Field and index access transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Check if an expression represents a module path (like `std::time`)
    /// STDLIB-003: Helper for distinguishing module paths from struct field access
    /// PARSER-094: Enhanced to recognize module-like identifiers and common module names
    fn is_module_path(&self, expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Identifier(name) => {
                // Check if this is a known module
                name == "std"  // stdlib module
                || self.module_names.contains(name)  // known user module
                || Self::is_module_like_identifier(name)  // lowercase_underscore pattern
                || name.chars().next().is_some_and(char::is_uppercase)  // Type names (associated functions)
            }
            ExprKind::FieldAccess { object, .. } => self.is_module_path(object),
            _ => false,
        }
    }

    /// TRANSPILER-011: Get the root identifier from a field access chain
    /// Used to determine if a chain like event.requestContext.requestId is:
    /// - Variable field access: event.field.subfield (use . syntax)
    /// - Module path: `std::time::Duration` (use :: syntax)
    /// - Type associated: `String::from` (use :: syntax)
    fn get_root_identifier(expr: &Expr) -> Option<&str> {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Identifier(name) => Some(name.as_str()),
            ExprKind::FieldAccess { object, .. } => Self::get_root_identifier(object),
            _ => None,
        }
    }

    /// TRANSPILER-011: Check if the root of a field access chain is a variable/parameter
    /// Variables are lowercase identifiers that are NOT modules or types
    /// Returns true for: event, obj, data, request (simple variables)
    /// Returns false for: std, String, `http_client`, `MyType` (modules/types)
    fn is_variable_chain(&self, expr: &Expr) -> bool {
        if let Some(root) = Self::get_root_identifier(expr) {
            // Variables are simple lowercase identifiers (no underscores, not modules, not types)
            let is_simple_lowercase = root.chars().all(char::is_lowercase) && !root.contains('_');
            let is_not_module = !self.module_names.contains(root) && root != "std";
            let is_not_type = root.chars().next().is_some_and(char::is_lowercase);

            is_simple_lowercase && is_not_module && is_not_type
        } else {
            false
        }
    }

    /// Check if an identifier looks like a module name
    /// PARSER-094: Fix Issue #137 - distinguish module paths from instance fields
    /// Heuristic: Module names are typically all lowercase with underscores (e.g., `http_client`, `std_env`)
    fn is_module_like_identifier(name: &str) -> bool {
        // Module names are all lowercase with underscores
        // Examples: http_client, my_module (with underscore)
        // NOT module-like: myVar (camelCase), self, this, obj (no underscore)
        if name.is_empty() || name == "self" || name == "this" {
            return false;
        }
        // Must be all lowercase/digits/underscores AND contain at least one underscore
        // This distinguishes modules (http_client) from variables (obj, x)
        let has_underscore = name.contains('_');
        let is_lowercase = name.chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '_');
        has_underscore && is_lowercase
    }

    pub fn transpile_field_access(&self, object: &Expr, field: &str) -> Result<TokenStream> {
        use crate::frontend::ast::ExprKind;
        let obj_tokens = self.transpile_expr(object)?;

        // DEFECT-PROPERTY-001: Check for numeric field access FIRST (tuple fields)
        // Prevents panic in format_ident! when field is pure number like "0"
        if field.chars().all(|c| c.is_ascii_digit()) {
            // Tuple field access - use numeric index (works for any object type)
            let index: usize = field.parse().unwrap();
            let index = syn::Index::from(index);
            return Ok(quote! { #obj_tokens.#index });
        }

        // Check if the object is an ObjectLiteral (HashMap) or module path
        match &object.kind {
            ExprKind::ObjectLiteral { .. } => {
                // Direct object literal access - use get()
                Ok(quote! {
                    #obj_tokens.get(#field)
                        .cloned()
                        .unwrap_or_else(|| panic!("Field '{}' not found", #field))
                })
            }
            ExprKind::FieldAccess { .. } => {
                // Nested field access - check if module path, numeric (tuple), or struct field
                // TRANSPILER-011: Check if this is a variable chain FIRST (event.field.subfield)
                // before defaulting to module path syntax (std::time::Duration)
                if field.chars().all(|c| c.is_ascii_digit()) {
                    // Nested tuple access like (nested.0).1
                    let index: usize = field.parse().unwrap();
                    let index = syn::Index::from(index);
                    Ok(quote! { #obj_tokens.#index })
                } else if field.is_empty() || field.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    // DEFECT: Empty field or starts with digit - invalid identifier
                    // Return error instead of panicking in format_ident!
                    anyhow::bail!("Invalid field name '{field}': field names cannot be empty or start with a digit")
                } else {
                    // Check for known instance methods that definitely need .
                    let known_methods = ["success", "exists", "is_empty", "is_some", "is_none", "is_ok", "is_err"];
                    let field_ident = format_ident!("{}", field);

                    if known_methods.contains(&field) {
                        // Known method - use . and add ()
                        Ok(quote! { #obj_tokens.#field_ident() })
                    } else if self.is_variable_chain(object) {
                        // TRANSPILER-011: Variable chain (event.requestContext.requestId) - use . syntax
                        Ok(quote! { #obj_tokens.#field_ident })
                    } else if self.is_module_path(object) {
                        // Confirmed module path - use ::
                        Ok(quote! { #obj_tokens::#field_ident })
                    } else {
                        // PARSER-094: Default to :: for nested paths (conservative heuristic)
                        // Rationale: nested::module::function more common than obj.field1.field2
                        Ok(quote! { #obj_tokens::#field_ident })
                    }
                }
            }
            ExprKind::Identifier(name) if name.contains("::") => {
                // Module path identifier - use :: syntax
                let field_ident = format_ident!("{}", field);
                Ok(quote! { #obj_tokens::#field_ident })
            }
            ExprKind::Identifier(name) if name == "std" => {
                // STDLIB-003: std module - use :: syntax for std::time, std::fs, etc.
                let field_ident = format_ident!("{}", field);
                Ok(quote! { #obj_tokens::#field_ident })
            }
            ExprKind::Identifier(name) if self.module_names.contains(name) => {
                // ISSUE-103: Module name - use :: syntax for module::function()
                // Examples: helper::get_message(), logger::log_info()
                let field_ident = format_ident!("{}", field);
                Ok(quote! { #obj_tokens::#field_ident })
            }
            ExprKind::Identifier(name) if name.chars().next().is_some_and(char::is_uppercase) => {
                // TRANSPILER-065: Type name (PascalCase) - use :: for associated functions/constructors
                // Examples: String::from(), Result::Ok(), Vec::new()
                // Heuristic: Rust types start with uppercase, instances with lowercase
                let field_ident = format_ident!("{}", field);
                Ok(quote! { #obj_tokens::#field_ident })
            }
            ExprKind::Identifier(name) if Self::is_module_like_identifier(name) => {
                // PARSER-094: Module-like identifier (lowercase_underscore pattern)
                // Examples: http_client::http_get(), my_module::function()
                // Issue #137: Fixes ruchy-lambda AWS Lambda runtime module calls
                let field_ident = format_ident!("{}", field);
                Ok(quote! { #obj_tokens::#field_ident })
            }
            _ => {
                // Check if field is numeric (tuple field access)
                if field.chars().all(|c| c.is_ascii_digit()) {
                    // Tuple field access - use numeric index
                    let index: usize = field.parse().unwrap();
                    let index = syn::Index::from(index);
                    Ok(quote! { #obj_tokens.#index })
                } else {
                    // TYPE-INFERENCE-001: Known stdlib methods need () for method calls
                    // ExitStatus::success, Path::exists, String::is_empty, etc.
                    let known_methods = ["success", "exists", "is_empty", "is_some", "is_none", "is_ok", "is_err"];
                    let field_ident = format_ident!("{}", field);

                    if known_methods.contains(&field) {
                        // Known method - add () for method call
                        Ok(quote! { #obj_tokens.#field_ident() })
                    } else {
                        // Regular struct field access
                        Ok(quote! { #obj_tokens.#field_ident })
                    }
                }
            }
        }
    }
    /// Transpiles index access `(array[index])`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_index_access;
    ///
    /// let result = transpile_index_access(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_index_access(&self, object: &Expr, index: &Expr) -> Result<TokenStream> {
        use crate::frontend::ast::{ExprKind, Literal};
        let obj_tokens = self.transpile_expr(object)?;
        let index_tokens = self.transpile_expr(index)?;
        // Smart index access: HashMap.get() for string keys, array indexing for numeric
        match &index.kind {
            // String literal keys use HashMap.get()
            ExprKind::Literal(Literal::String(_)) => Ok(quote! {
                #obj_tokens.get(#index_tokens)
                    .cloned()
                    .unwrap_or_else(|| panic!("Key not found"))
            }),
            // Numeric and other keys use array indexing with clone for non-Copy types
            // DEFECT-014: Auto-clone to prevent E0507 (cannot move out of index)
            _ => Ok(quote! { #obj_tokens[#index_tokens as usize].clone() }),
        }
    }
    /// Transpiles slice access `(array[start:end])`
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_slice;
    ///
    /// let result = transpile_slice(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_slice(
        &self,
        object: &Expr,
        start: Option<&Expr>,
        end: Option<&Expr>,
    ) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        match (start, end) {
            (None, None) => {
                // Full slice [..]
                Ok(quote! { &#obj_tokens[..] })
            }
            (None, Some(end)) => {
                // Slice from beginning [..end]
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[..#end_tokens as usize] })
            }
            (Some(start), None) => {
                // Slice to end [start..]
                let start_tokens = self.transpile_expr(start)?;
                Ok(quote! { &#obj_tokens[#start_tokens as usize..] })
            }
            (Some(start), Some(end)) => {
                // Full range slice [start..end]
                let start_tokens = self.transpile_expr(start)?;
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[#start_tokens as usize..#end_tokens as usize] })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    // Helper function to create test transpiler
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper function to create test transpiler with module names
    fn test_transpiler_with_modules(modules: Vec<&str>) -> Transpiler {
        let mut transpiler = Transpiler::new();
        transpiler.module_names = modules.iter().map(std::string::ToString::to_string).collect();
        transpiler
    }

    // Helper to create identifier expression
    fn ident_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper to create field access expression
    fn field_access_expr(object: Expr, field: &str) -> Expr {
        Expr {
            kind: ExprKind::FieldAccess {
                object: Box::new(object),
                field: field.to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper to create integer literal expression
    fn int_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper to create string literal expression
    fn string_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: is_module_like_identifier - valid module names
    #[test]
    fn test_is_module_like_identifier_valid() {
        assert!(Transpiler::is_module_like_identifier("http_client"));
        assert!(Transpiler::is_module_like_identifier("my_module"));
        assert!(Transpiler::is_module_like_identifier("std_env"));
    }

    // Test 2: is_module_like_identifier - invalid (no underscore)
    #[test]
    fn test_is_module_like_identifier_no_underscore() {
        assert!(!Transpiler::is_module_like_identifier("obj"));
        assert!(!Transpiler::is_module_like_identifier("myvar"));
        assert!(!Transpiler::is_module_like_identifier("x"));
    }

    // Test 3: is_module_like_identifier - special cases
    #[test]
    fn test_is_module_like_identifier_special() {
        assert!(!Transpiler::is_module_like_identifier("self"));
        assert!(!Transpiler::is_module_like_identifier("this"));
        assert!(!Transpiler::is_module_like_identifier(""));
    }

    // Test 4: is_module_path - stdlib module
    #[test]
    fn test_is_module_path_std() {
        let transpiler = test_transpiler();
        let expr = ident_expr("std");
        assert!(transpiler.is_module_path(&expr));
    }

    // Test 5: is_module_path - known user module
    #[test]
    fn test_is_module_path_user_module() {
        let transpiler = test_transpiler_with_modules(vec!["helper"]);
        let expr = ident_expr("helper");
        assert!(transpiler.is_module_path(&expr));
    }

    // Test 6: is_module_path - module-like identifier
    #[test]
    fn test_is_module_path_module_like() {
        let transpiler = test_transpiler();
        let expr = ident_expr("http_client");
        assert!(transpiler.is_module_path(&expr));
    }

    // Test 7: is_module_path - type name (uppercase)
    #[test]
    fn test_is_module_path_type_name() {
        let transpiler = test_transpiler();
        let expr = ident_expr("String");
        assert!(transpiler.is_module_path(&expr));
    }

    // Test 8: is_module_path - nested field access
    #[test]
    fn test_is_module_path_nested() {
        let transpiler = test_transpiler();
        let std_expr = ident_expr("std");
        let nested = field_access_expr(std_expr, "time");
        assert!(transpiler.is_module_path(&nested));
    }

    // Test 9: get_root_identifier - identifier
    #[test]
    fn test_get_root_identifier_simple() {
        let expr = ident_expr("obj");
        assert_eq!(Transpiler::get_root_identifier(&expr), Some("obj"));
    }

    // Test 10: get_root_identifier - nested field access
    #[test]
    fn test_get_root_identifier_nested() {
        let obj = ident_expr("event");
        let access = field_access_expr(obj, "requestContext");
        assert_eq!(Transpiler::get_root_identifier(&access), Some("event"));
    }

    // Test 11: is_variable_chain - simple variable
    #[test]
    fn test_is_variable_chain_simple() {
        let transpiler = test_transpiler();
        let expr = ident_expr("event");
        assert!(transpiler.is_variable_chain(&expr));
    }

    // Test 12: is_variable_chain - not a module
    #[test]
    fn test_is_variable_chain_not_module() {
        let transpiler = test_transpiler();
        let expr = ident_expr("http_client");
        assert!(!transpiler.is_variable_chain(&expr)); // Has underscore
    }

    // Test 13: transpile_field_access - tuple field (numeric)
    #[test]
    fn test_transpile_field_access_tuple() {
        let transpiler = test_transpiler();
        let obj = ident_expr("tuple");
        let result = transpiler.transpile_field_access(&obj, "0").unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("tuple") && result_str.contains(". 0"));
    }

    // Test 14: transpile_field_access - std module
    #[test]
    fn test_transpile_field_access_std_module() {
        let transpiler = test_transpiler();
        let std = ident_expr("std");
        let result = transpiler.transpile_field_access(&std, "time").unwrap();
        assert_eq!(result.to_string(), "std :: time");
    }

    // Test 15: transpile_field_access - type associated function
    #[test]
    fn test_transpile_field_access_type_associated() {
        let transpiler = test_transpiler();
        let string_type = ident_expr("String");
        let result = transpiler.transpile_field_access(&string_type, "from").unwrap();
        assert_eq!(result.to_string(), "String :: from");
    }

    // Test 16: transpile_field_access - known method
    #[test]
    fn test_transpile_field_access_known_method() {
        let transpiler = test_transpiler();
        let obj = ident_expr("result");
        let result = transpiler.transpile_field_access(&obj, "is_ok").unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("result") && result_str.contains("is_ok") && result_str.contains("()"));
    }

    // Test 17: transpile_field_access - variable chain
    #[test]
    fn test_transpile_field_access_variable_chain() {
        let transpiler = test_transpiler();
        let event = ident_expr("event");
        let result = transpiler.transpile_field_access(&event, "field").unwrap();
        assert_eq!(result.to_string(), "event . field");
    }

    // Test 18: transpile_field_access - module-like identifier
    #[test]
    fn test_transpile_field_access_module_like_identifier() {
        let transpiler = test_transpiler();
        let http_client = ident_expr("http_client");
        let result = transpiler.transpile_field_access(&http_client, "get").unwrap();
        assert_eq!(result.to_string(), "http_client :: get");
    }

    // Test 19: transpile_field_access - invalid field (starts with digit)
    #[test]
    fn test_transpile_field_access_invalid_field_starts_digit() {
        let transpiler = test_transpiler();
        let nested = field_access_expr(ident_expr("obj"), "field");
        let result = transpiler.transpile_field_access(&nested, "9field");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid field name"));
    }

    // Test 20: transpile_index_access - string key (HashMap)
    #[test]
    fn test_transpile_index_access_string_key() {
        let transpiler = test_transpiler();
        let map = ident_expr("map");
        let key = string_expr("key");
        let result = transpiler.transpile_index_access(&map, &key).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("map . get") && result_str.contains("cloned"));
    }

    // Test 21: transpile_index_access - numeric key (array)
    #[test]
    fn test_transpile_index_access_numeric() {
        let transpiler = test_transpiler();
        let array = ident_expr("arr");
        let index = int_expr(0);
        let result = transpiler.transpile_index_access(&array, &index).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("arr") && result_str.contains('[') && result_str.contains("clone"));
    }

    // Test 22: transpile_slice - full slice [..]
    #[test]
    fn test_transpile_slice_full() {
        let transpiler = test_transpiler();
        let array = ident_expr("arr");
        let result = transpiler.transpile_slice(&array, None, None).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("& arr") && result_str.contains('[') && result_str.contains(".."));
    }

    // Test 23: transpile_slice - from beginning [..end]
    #[test]
    fn test_transpile_slice_to_end() {
        let transpiler = test_transpiler();
        let array = ident_expr("arr");
        let end = int_expr(5);
        let result = transpiler.transpile_slice(&array, None, Some(&end)).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("arr") && result_str.contains("..") && result_str.contains('5'));
    }

    // Test 24: transpile_slice - from start [start..]
    #[test]
    fn test_transpile_slice_from_start() {
        let transpiler = test_transpiler();
        let array = ident_expr("arr");
        let start = int_expr(2);
        let result = transpiler.transpile_slice(&array, Some(&start), None).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("arr") && result_str.contains('2') && result_str.contains(".."));
    }

    // Test 25: transpile_slice - range [start..end]
    #[test]
    fn test_transpile_slice_range() {
        let transpiler = test_transpiler();
        let array = ident_expr("arr");
        let start = int_expr(1);
        let end = int_expr(4);
        let result = transpiler.transpile_slice(&array, Some(&start), Some(&end)).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("arr") && result_str.contains('1') && result_str.contains('4') && result_str.contains(".."));
    }
}
