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

    /// Check if an identifier looks like a module name
    /// PARSER-094: Fix Issue #137 - distinguish module paths from instance fields
    /// Heuristic: Module names are typically all lowercase with underscores (e.g., http_client, std_env)
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
                // PARSER-094: Heuristic - nested paths are usually modules (nested::module::function)
                // not struct fields (obj.field.method), UNLESS we see clear field indicators
                if field.chars().all(|c| c.is_ascii_digit()) {
                    // Nested tuple access like (nested.0).1
                    let index: usize = field.parse().unwrap();
                    let index = syn::Index::from(index);
                    Ok(quote! { #obj_tokens.#index })
                } else {
                    // Check for known instance methods that definitely need .
                    let known_methods = ["success", "exists", "is_empty", "is_some", "is_none", "is_ok", "is_err"];
                    let field_ident = format_ident!("{}", field);

                    if known_methods.contains(&field) {
                        // Known method - use . and add ()
                        Ok(quote! { #obj_tokens.#field_ident() })
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
