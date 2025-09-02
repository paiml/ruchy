//! Type transpilation and struct/trait definitions

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::only_used_in_recursion)]

use super::*;
use crate::frontend::ast::{EnumVariant, ImplMethod, StructField, TraitMethod, Type};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles type annotations
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Basic types
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("let x: i32 = 42");
    /// let ast = parser.parse().unwrap();
    /// 
    /// let result = transpiler.transpile(&ast).unwrap();
    /// let code = result.to_string();
    /// assert!(code.contains("i32"));
    /// assert!(code.contains("42"));
    /// ```
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Generic types
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("let v: Vec<i32> = vec![1, 2, 3]");
    /// let ast = parser.parse().unwrap();
    /// 
    /// let result = transpiler.transpile(&ast).unwrap();
    /// let code = result.to_string();
    /// assert!(code.contains("Vec"));
    /// assert!(code.contains("i32"));
    /// ```
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Optional types
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("let opt: Option<i32> = Some(42)");
    /// let ast = parser.parse().unwrap();
    /// 
    /// let result = transpiler.transpile(&ast).unwrap();
    /// let code = result.to_string();
    /// assert!(code.contains("Option"));
    /// assert!(code.contains("Some"));
    /// ```
    pub fn transpile_type(&self, ty: &Type) -> Result<TokenStream> {
        use crate::frontend::ast::TypeKind;

        match &ty.kind {
            TypeKind::Named(name) => {
                // Map common Ruchy types to Rust types
                let rust_type = match name.as_str() {
                    "int" => quote! { i64 },
                    "float" => quote! { f64 },
                    "bool" => quote! { bool },
                    "str" => quote! { &str },  // Fix: Ruchy str -> Rust &str
                    "string" | "String" => quote! { String },
                    "char" => quote! { char },
                    // PERFORMANCE OPTIMIZATION: Use Rust type inference instead of Any
                    "_" | "Any" => quote! { _ },
                    _ => {
                        let type_ident = format_ident!("{}", name);
                        quote! { #type_ident }
                    }
                };
                Ok(rust_type)
            }
            TypeKind::Generic { base, params } => {
                let base_ident = format_ident!("{}", base);
                let param_tokens: Result<Vec<_>> =
                    params.iter().map(|p| self.transpile_type(p)).collect();
                let param_tokens = param_tokens?;
                Ok(quote! { #base_ident<#(#param_tokens),*> })
            }
            TypeKind::Optional(inner) => {
                let inner_tokens = self.transpile_type(inner)?;
                Ok(quote! { Option<#inner_tokens> })
            }
            TypeKind::List(elem_type) => {
                let elem_tokens = self.transpile_type(elem_type)?;
                Ok(quote! { Vec<#elem_tokens> })
            }
            TypeKind::Tuple(types) => {
                let type_tokens: Result<Vec<_>> =
                    types.iter().map(|t| self.transpile_type(t)).collect();
                let type_tokens = type_tokens?;
                Ok(quote! { (#(#type_tokens),*) })
            }
            TypeKind::Function { params, ret } => {
                let param_tokens: Result<Vec<_>> =
                    params.iter().map(|p| self.transpile_type(p)).collect();
                let param_tokens = param_tokens?;
                let ret_tokens = self.transpile_type(ret)?;

                // Rust function type syntax
                Ok(quote! { fn(#(#param_tokens),*) -> #ret_tokens })
            }
            TypeKind::DataFrame { .. } => {
                // DataFrames map to Polars DataFrame type
                Ok(quote! { polars::prelude::DataFrame })
            }
            TypeKind::Series { .. } => {
                // Series maps to Polars Series type
                Ok(quote! { polars::prelude::Series })
            }
        }
    }

    /// Transpiles struct definitions
    pub fn transpile_struct(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[StructField],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let struct_name = format_ident!("{}", name);

        let type_param_tokens: Vec<_> =
            type_params.iter().map(|p| format_ident!("{}", p)).collect();

        let field_tokens: Vec<TokenStream> = fields
            .iter()
            .map(|field| {
                let field_name = format_ident!("{}", field.name);
                let field_type = self
                    .transpile_type(&field.ty)
                    .unwrap_or_else(|_| quote! { _ });

                if field.is_pub {
                    quote! { pub #field_name: #field_type }
                } else {
                    quote! { #field_name: #field_type }
                }
            })
            .collect();

        let visibility = if is_pub { quote! { pub } } else { quote! {} };

        if type_params.is_empty() {
            Ok(quote! {
                #visibility struct #struct_name {
                    #(#field_tokens,)*
                }
            })
        } else {
            Ok(quote! {
                #visibility struct #struct_name<#(#type_param_tokens),*> {
                    #(#field_tokens,)*
                }
            })
        }
    }

    /// Transpiles trait definitions
    pub fn transpile_enum(
        &self,
        name: &str,
        type_params: &[String],
        variants: &[EnumVariant],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let enum_name = format_ident!("{}", name);

        let type_param_tokens: Vec<_> =
            type_params.iter().map(|p| format_ident!("{}", p)).collect();

        let variant_tokens: Vec<TokenStream> = variants
            .iter()
            .map(|variant| {
                let variant_name = format_ident!("{}", variant.name);

                if let Some(fields) = &variant.fields {
                    // Tuple variant
                    let field_types: Vec<TokenStream> = fields
                        .iter()
                        .map(|ty| self.transpile_type(ty).unwrap_or_else(|_| quote! { _ }))
                        .collect();
                    quote! { #variant_name(#(#field_types),*) }
                } else {
                    // Unit variant
                    quote! { #variant_name }
                }
            })
            .collect();

        let visibility = if is_pub { quote! { pub } } else { quote! {} };

        if type_params.is_empty() {
            Ok(quote! {
                #visibility enum #enum_name {
                    #(#variant_tokens,)*
                }
            })
        } else {
            Ok(quote! {
                #visibility enum #enum_name<#(#type_param_tokens),*> {
                    #(#variant_tokens,)*
                }
            })
        }
    }

    pub fn transpile_trait(
        &self,
        name: &str,
        type_params: &[String],
        methods: &[TraitMethod],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let trait_name = format_ident!("{}", name);

        let method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = format_ident!("{}", method.name);

                // Process parameters
                let param_tokens: Vec<TokenStream> = method
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, param)| {
                        if i == 0 && (param.name() == "self" || param.name() == "&self") {
                            // Handle self parameter - check if it's &self or self
                            if param.name().starts_with('&') {
                                quote! { &self }
                            } else {
                                quote! { self }
                            }
                        } else {
                            let param_name = format_ident!("{}", param.name());
                            let type_tokens = self
                                .transpile_type(&param.ty)
                                .unwrap_or_else(|_| quote! { _ });
                            quote! { #param_name: #type_tokens }
                        }
                    })
                    .collect();

                // Process return type
                let return_type_tokens = if let Some(ref ty) = method.return_type {
                    let ty_tokens = self.transpile_type(ty)?;
                    quote! { -> #ty_tokens }
                } else {
                    quote! {}
                };

                // Process method visibility
                let visibility = if method.is_pub { quote! { pub } } else { quote! {} };

                // Process method body (if default implementation)
                if let Some(ref body) = method.body {
                    let body_tokens = self.transpile_expr(body)?;
                    Ok(quote! {
                        #visibility fn #method_name(#(#param_tokens),*) #return_type_tokens {
                            #body_tokens
                        }
                    })
                } else {
                    Ok(quote! {
                        #visibility fn #method_name(#(#param_tokens),*) #return_type_tokens;
                    })
                }
            })
            .collect();

        let method_tokens = method_tokens?;

        let type_param_tokens: Vec<_> =
            type_params.iter().map(|p| format_ident!("{}", p)).collect();

        let visibility = if is_pub { quote! { pub } } else { quote! {} };

        if type_params.is_empty() {
            Ok(quote! {
                #visibility trait #trait_name {
                    #(#method_tokens)*
                }
            })
        } else {
            Ok(quote! {
                #visibility trait #trait_name<#(#type_param_tokens),*> {
                    #(#method_tokens)*
                }
            })
        }
    }

    /// Transpiles impl blocks
    pub fn transpile_impl(
        &self,
        for_type: &str,
        type_params: &[String],
        trait_name: Option<&str>,
        methods: &[ImplMethod],
        _is_pub: bool,
    ) -> Result<TokenStream> {
        let type_ident = format_ident!("{}", for_type);

        let method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = format_ident!("{}", method.name);

                // Process parameters
                let param_tokens: Vec<TokenStream> = method
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, param)| {
                        if i == 0 && (param.name() == "self" || param.name() == "&self") {
                            // Handle self parameter
                            if param.name().starts_with('&') {
                                quote! { &self }
                            } else {
                                quote! { self }
                            }
                        } else {
                            let param_name = format_ident!("{}", param.name());
                            let type_tokens = self
                                .transpile_type(&param.ty)
                                .unwrap_or_else(|_| quote! { _ });
                            quote! { #param_name: #type_tokens }
                        }
                    })
                    .collect();

                // Process return type
                let return_type_tokens = if let Some(ref ty) = method.return_type {
                    let ty_tokens = self.transpile_type(ty)?;
                    quote! { -> #ty_tokens }
                } else {
                    quote! {}
                };

                // Process method body (always present in ImplMethod)
                let body_tokens = self.transpile_expr(&method.body)?;

                // Process method visibility
                let visibility = if method.is_pub { quote! { pub } } else { quote! {} };

                Ok(quote! {
                    #visibility fn #method_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            })
            .collect();

        let method_tokens = method_tokens?;

        let type_param_tokens: Vec<_> =
            type_params.iter().map(|p| format_ident!("{}", p)).collect();

        if let Some(trait_name) = trait_name {
            let trait_ident = format_ident!("{}", trait_name);
            if type_params.is_empty() {
                Ok(quote! {
                    impl #trait_ident for #type_ident {
                        #(#method_tokens)*
                    }
                })
            } else {
                Ok(quote! {
                    impl<#(#type_param_tokens),*> #trait_ident for #type_ident<#(#type_param_tokens),*> {
                        #(#method_tokens)*
                    }
                })
            }
        } else {
            if type_params.is_empty() {
                Ok(quote! {
                    impl #type_ident {
                        #(#method_tokens)*
                    }
                })
            } else {
                Ok(quote! {
                    impl<#(#type_param_tokens),*> #type_ident<#(#type_param_tokens),*> {
                        #(#method_tokens)*
                    }
                })
            }
        }
    }

    /// Transpiles property test attributes
    pub fn transpile_property_test(&self, expr: &Expr, _attr: &Attribute) -> Result<TokenStream> {
        // Property tests in Rust typically use proptest or quickcheck
        // We'll generate proptest-compatible code

        if let ExprKind::Function {
            name, params, body, ..
        } = &expr.kind
        {
            let fn_name = format_ident!("{}", name);

            // Generate property test parameters
            let param_tokens: Vec<TokenStream> = params
                .iter()
                .map(|p| {
                    let param_name = format_ident!("{}", p.name());
                    let type_tokens = self
                        .transpile_type(&p.ty)
                        .unwrap_or_else(|_| quote! { i32 });
                    quote! { #param_name: #type_tokens }
                })
                .collect();

            let body_tokens = self.transpile_expr(body)?;

            // Generate the proptest macro invocation
            Ok(quote! {
                #[cfg(test)]
                mod #fn_name {
                    use super::*;
                    use proptest::prelude::*;

                    proptest! {
                        #[test]
                        fn #fn_name(#(#param_tokens),*) {
                            #body_tokens
                        }
                    }
                }
            })
        } else {
            bail!("Property test attribute can only be applied to functions");
        }
    }

    /// Transpiles extension methods into trait + impl
    ///
    /// Generates both a trait definition and an implementation according to the specification:
    /// ```rust
    /// // Ruchy: extend String { fun is_palindrome(&self) -> bool { ... } }
    /// // Rust:  trait StringExt { fn is_palindrome(&self) -> bool; }
    /// //        impl StringExt for String { fn is_palindrome(&self) -> bool { ... } }
    /// ```
    pub fn transpile_extend(
        &self,
        target_type: &str,
        methods: &[ImplMethod],
    ) -> Result<TokenStream> {
        let target_ident = format_ident!("{}", target_type);
        let trait_name = format_ident!("{}Ext", target_type); // e.g., StringExt

        // Generate trait definition
        let trait_method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = format_ident!("{}", method.name);

                // Process parameters
                let param_tokens: Vec<TokenStream> = method
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, param)| {
                        if i == 0 && (param.name() == "self" || param.name() == "&self") {
                            // Handle self parameter
                            if param.name().starts_with('&') {
                                quote! { &self }
                            } else {
                                quote! { self }
                            }
                        } else {
                            let param_name = format_ident!("{}", param.name());
                            let type_tokens = self
                                .transpile_type(&param.ty)
                                .unwrap_or_else(|_| quote! { _ });
                            quote! { #param_name: #type_tokens }
                        }
                    })
                    .collect();

                // Process return type
                let return_type_tokens = if let Some(ref ty) = method.return_type {
                    let ty_tokens = self.transpile_type(ty)?;
                    quote! { -> #ty_tokens }
                } else {
                    quote! {}
                };

                // Trait methods are just signatures (no body)
                Ok(quote! {
                    fn #method_name(#(#param_tokens),*) #return_type_tokens;
                })
            })
            .collect();

        let trait_method_tokens = trait_method_tokens?;

        // Generate impl definition
        let impl_method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = format_ident!("{}", method.name);

                // Process parameters (same as trait)
                let param_tokens: Vec<TokenStream> = method
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, param)| {
                        if i == 0 && (param.name() == "self" || param.name() == "&self") {
                            if param.name().starts_with('&') {
                                quote! { &self }
                            } else {
                                quote! { self }
                            }
                        } else {
                            let param_name = format_ident!("{}", param.name());
                            let type_tokens = self
                                .transpile_type(&param.ty)
                                .unwrap_or_else(|_| quote! { _ });
                            quote! { #param_name: #type_tokens }
                        }
                    })
                    .collect();

                // Process return type
                let return_type_tokens = if let Some(ref ty) = method.return_type {
                    let ty_tokens = self.transpile_type(ty)?;
                    quote! { -> #ty_tokens }
                } else {
                    quote! {}
                };

                // Impl methods have bodies
                let body_tokens = self.transpile_expr(&method.body)?;

                Ok(quote! {
                    fn #method_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            })
            .collect();

        let impl_method_tokens = impl_method_tokens?;

        // Generate both trait and impl
        Ok(quote! {
            trait #trait_name {
                #(#trait_method_tokens)*
            }

            impl #trait_name for #target_ident {
                #(#impl_method_tokens)*
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_transpile_result_helpers() {
        let helpers = Transpiler::generate_result_helpers();
        let code = helpers.to_string();
        
        // Check that the ResultExt trait is generated
        assert!(code.contains("trait ResultExt"));
        assert!(code.contains("map_err_with"));
        assert!(code.contains("unwrap_or_else_with"));
        assert!(code.contains("and_then_with"));
        assert!(code.contains("or_else_with"));
    }
}
