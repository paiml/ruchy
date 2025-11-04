//! Identifier and qualified name transpilation helpers

use super::super::Transpiler;
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

    pub(in crate::backend::transpiler) fn transpile_qualified_name(module: &str, name: &str) -> TokenStream {
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
}
