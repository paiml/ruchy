//! Module system transpilation (import, export, module definitions)
//!
//! Handles module declarations and import/export statements

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ImportItem};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

impl Transpiler {
    /// Transpile module declaration (complexity: 5)
    pub fn transpile_module(&self, name: &str, body: &Expr) -> Result<TokenStream> {
        let mod_name = format_ident!("{}", name);
        let body_tokens = self.transpile_expr(body)?;
        
        Ok(quote! {
            pub mod #mod_name {
                #body_tokens
            }
        })
    }

    /// Transpile import statement (complexity: 8)
    pub fn transpile_import(path: &str, items: &[ImportItem]) -> TokenStream {
        let path_tokens = Self::generate_path_tokens(path);
        
        if items.is_empty() {
            // Import everything
            quote! { use #path_tokens::*; }
        } else if items.len() == 1 && items[0].alias.is_none() {
            // Single item without alias
            let item_ident = format_ident!("{}", items[0].name);
            quote! { use #path_tokens::#item_ident; }
        } else {
            // Multiple items or with aliases
            Self::generate_complex_import(&path_tokens, items)
        }
    }

    /// Helper: Generate complex import with multiple items/aliases (complexity: 7)
    fn generate_complex_import(path_tokens: &TokenStream, items: &[ImportItem]) -> TokenStream {
        let import_specs: Vec<TokenStream> = items.iter().map(|item| {
            let name_ident = format_ident!("{}", item.name);
            if let Some(alias) = &item.alias {
                let alias_ident = format_ident!("{}", alias);
                quote! { #name_ident as #alias_ident }
            } else {
                quote! { #name_ident }
            }
        }).collect();
        
        quote! { use #path_tokens::{#(#import_specs),*}; }
    }

    /// Transpile inline import for expressions (complexity: 9)
    pub fn transpile_import_inline(path: &str, items: &[ImportItem]) -> TokenStream {
        if path.starts_with("std") || path.starts_with("core") {
            Self::transpile_std_import(path, items)
        } else if items.is_empty() {
            Self::transpile_wildcard_import(path)
        } else {
            Self::transpile_specific_import(path, items)
        }
    }

    /// Helper: Transpile standard library import (complexity: 5)
    fn transpile_std_import(path: &str, items: &[ImportItem]) -> TokenStream {
        let path_tokens = Self::generate_path_tokens(path);
        
        if items.is_empty() {
            quote! { #path_tokens }
        } else {
            let item_ident = format_ident!("{}", items[0].name);
            quote! { #path_tokens::#item_ident }
        }
    }

    /// Helper: Transpile wildcard import (complexity: 3)
    fn transpile_wildcard_import(path: &str) -> TokenStream {
        let path_ident = format_ident!("{}", path);
        quote! { #path_ident }
    }

    /// Helper: Transpile specific item import (complexity: 6)
    fn transpile_specific_import(path: &str, items: &[ImportItem]) -> TokenStream {
        let path_ident = format_ident!("{}", path);
        let item_ident = format_ident!("{}", items[0].name);
        
        if let Some(alias) = &items[0].alias {
            let alias_ident = format_ident!("{}", alias);
            quote! { #path_ident::#item_ident as #alias_ident }
        } else {
            quote! { #path_ident::#item_ident }
        }
    }

    /// Transpile export statement (complexity: 4)
    pub fn transpile_export(items: &[String]) -> TokenStream {
        if items.is_empty() {
            quote! { pub use super::*; }
        } else {
            let item_idents: Vec<_> = items.iter()
                .map(|item| format_ident!("{}", item))
                .collect();
            quote! { pub use super::{#(#item_idents),*}; }
        }
    }

    /// Helper: Generate path tokens from string (complexity: 5)
    fn generate_path_tokens(path: &str) -> TokenStream {
        let segments: Vec<_> = path.split("::")
            .map(|seg| format_ident!("{}", seg))
            .collect();
        
        quote! { #(#segments)::* }
    }

    /// Generate from import (Python-style) (complexity: 7)
    pub fn transpile_from_import(
        &self,
        module: &str,
        items: Vec<String>,
        star: bool,
    ) -> Result<TokenStream> {
        let mod_path = Self::python_to_rust_path(module);
        let path_tokens = Self::generate_path_tokens(&mod_path);
        
        if star {
            Ok(quote! { use #path_tokens::*; })
        } else {
            let item_idents: Vec<_> = items.iter()
                .map(|item| format_ident!("{}", item))
                .collect();
            Ok(quote! { use #path_tokens::{#(#item_idents),*}; })
        }
    }

    /// Helper: Convert Python-style module path to Rust (complexity: 3)
    fn python_to_rust_path(module: &str) -> String {
        module.replace('.', "::")
    }

    /// Generate re-export statement (complexity: 4)
    pub fn transpile_reexport(&self, module: &str, items: &[String]) -> Result<TokenStream> {
        let mod_ident = format_ident!("{}", module);
        
        if items.is_empty() {
            Ok(quote! { pub use #mod_ident::*; })
        } else {
            let item_idents: Vec<_> = items.iter()
                .map(|item| format_ident!("{}", item))
                .collect();
            Ok(quote! { pub use #mod_ident::{#(#item_idents),*}; })
        }
    }
}