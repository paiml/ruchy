//! Type transpilation and struct/trait definitions
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::only_used_in_recursion)]
use super::*;
use crate::frontend::ast::{
    ClassMethod, Constructor, EnumVariant, ImplMethod, StructField, TraitMethod, Type, TypeKind,
};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Lifetime;
impl Transpiler {
    /// Transpiles type annotations
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// // Basic types
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("let x: i32 = 42");
    /// let ast = parser.parse().expect("Failed to parse");
    ///
    /// let result = transpiler.transpile(&ast).expect("operation should succeed in test");
    /// let code = result.to_string();
    /// assert!(code.contains("i32"));
    /// assert!(code.contains("42"));
    /// ```
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// // Generic types
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("let v = [1, 2, 3]");
    /// let ast = parser.parse().expect("operation should succeed in test");
    ///
    /// let result = transpiler.transpile(&ast).expect("operation should succeed in test");
    /// // Basic transpilation test - just check it compiles
    /// assert!(!result.to_string().is_empty());
    /// ```
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// // Optional types
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new("let opt = Some(42)");
    /// let ast = parser.parse().expect("operation should succeed in test");
    ///
    /// let result = transpiler.transpile(&ast).expect("operation should succeed in test");
    /// let code = result.to_string();
    /// assert!(code.contains("Some"));
    /// ```
    pub fn transpile_type(&self, ty: &Type) -> Result<TokenStream> {
        use crate::frontend::ast::TypeKind;
        match &ty.kind {
            TypeKind::Named(name) => self.transpile_named_type(name),
            TypeKind::Generic { base, params } => self.transpile_generic_type(base, params),
            TypeKind::Optional(inner) => self.transpile_optional_type(inner),
            TypeKind::List(elem_type) => self.transpile_list_type(elem_type),
            TypeKind::Array { elem_type, size } => self.transpile_array_type(elem_type, *size),
            TypeKind::Tuple(types) => self.transpile_tuple_type(types),
            TypeKind::Function { params, ret } => self.transpile_function_type(params, ret),
            TypeKind::DataFrame { .. } => Ok(quote! { polars::prelude::DataFrame }),
            TypeKind::Series { .. } => Ok(quote! { polars::prelude::Series }),
            TypeKind::Reference {
                is_mut,
                lifetime,
                inner,
            } => self.transpile_reference_type(*is_mut, lifetime.as_deref(), inner),
            // SPEC-001-H: Refined types - transpile base type only, ignore constraint
            // Rust's type system doesn't support runtime refinement checking
            // The constraint is a compile-time annotation in Ruchy
            TypeKind::Refined { base, .. } => self.transpile_type(base),
        }
    }
    /// Transpile named types with built-in type mapping
    pub(crate) fn transpile_named_type(&self, name: &str) -> Result<TokenStream> {
        let rust_type = match name {
            "int" => quote! { i64 },
            "float" => quote! { f64 },
            "bool" => quote! { bool },
            "str" => quote! { &str }, // String slice reference (sized type for function parameters)
            "string" | "String" => quote! { String },
            "char" => quote! { char },
            "()" => quote! { () },       // Unit type
            "_" | "Any" => quote! { _ }, // Use Rust type inference
            "Object" => quote! { std::collections::BTreeMap<String, String> }, // TRANSPILER-013 FIX: Use String for standalone rustc compatibility
            _ => {
                // TRANSPILER-DEFECT-005: Handle namespaced types (e.g., trace::Sampler, std::io::Error)
                if name.contains("::") {
                    // Split into path segments and build path token
                    let segments: Vec<_> = name
                        .split("::")
                        .map(|seg| format_ident!("{}", seg))
                        .collect();
                    quote! { #(#segments)::* }
                } else {
                    // Simple identifier
                    let type_ident = format_ident!("{}", name);
                    quote! { #type_ident }
                }
            }
        };
        Ok(rust_type)
    }
    /// Transpile generic types with type parameters
    pub(crate) fn transpile_generic_type(&self, base: &str, params: &[Type]) -> Result<TokenStream> {
        let base_ident = format_ident!("{}", base);
        let param_tokens: Result<Vec<_>> = params.iter().map(|p| self.transpile_type(p)).collect();
        let param_tokens = param_tokens?;
        Ok(quote! { #base_ident<#(#param_tokens),*> })
    }
    /// Transpile optional types to Option<T>
    fn transpile_optional_type(&self, inner: &Type) -> Result<TokenStream> {
        let inner_tokens = self.transpile_type(inner)?;
        Ok(quote! { Option<#inner_tokens> })
    }
    /// Transpile list types to Vec<T>
    fn transpile_list_type(&self, elem_type: &Type) -> Result<TokenStream> {
        let elem_tokens = self.transpile_type(elem_type)?;
        Ok(quote! { Vec<#elem_tokens> })
    }
    /// Transpile array types with fixed size
    fn transpile_array_type(&self, elem_type: &Type, size: usize) -> Result<TokenStream> {
        let elem_tokens = self.transpile_type(elem_type)?;
        let size_lit = proc_macro2::Literal::usize_unsuffixed(size);
        Ok(quote! { [#elem_tokens; #size_lit] })
    }
    /// Transpile tuple types
    fn transpile_tuple_type(&self, types: &[Type]) -> Result<TokenStream> {
        let type_tokens: Result<Vec<_>> = types.iter().map(|t| self.transpile_type(t)).collect();
        let type_tokens = type_tokens?;
        Ok(quote! { (#(#type_tokens),*) })
    }
    /// Transpile function types
    fn transpile_function_type(&self, params: &[Type], ret: &Type) -> Result<TokenStream> {
        let param_tokens: Result<Vec<_>> = params.iter().map(|p| self.transpile_type(p)).collect();
        let param_tokens = param_tokens?;
        let ret_tokens = self.transpile_type(ret)?;
        Ok(quote! { fn(#(#param_tokens),*) -> #ret_tokens })
    }
    /// Transpile reference types with special handling for &str and lifetimes
    pub(crate) fn transpile_reference_type(
        &self,
        is_mut: bool,
        lifetime: Option<&str>,
        inner: &Type,
    ) -> Result<TokenStream> {
        use crate::frontend::ast::TypeKind;

        // Create lifetime token if provided
        let lifetime_token = if let Some(lt) = lifetime {
            let lifetime = Lifetime::new(lt, proc_macro2::Span::call_site());
            quote! { #lifetime }
        } else {
            quote! {}
        };

        // Special case: &str should not become &&str
        if let TypeKind::Named(name) = &inner.kind {
            if name == "str" {
                return if is_mut {
                    Ok(quote! { &#lifetime_token mut str })
                } else {
                    Ok(quote! { &#lifetime_token str })
                };
            }
        }
        let inner_tokens = self.transpile_type(inner)?;
        if is_mut {
            Ok(quote! { &#lifetime_token mut #inner_tokens })
        } else {
            Ok(quote! { &#lifetime_token #inner_tokens })
        }
    }
    /// Transpiles tuple struct definitions
    pub fn transpile_tuple_struct(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[Type],
        derives: &[String],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let struct_name = format_ident!("{}", name);
        let type_param_tokens: Vec<TokenStream> = type_params
            .iter()
            .map(|p| Self::parse_type_param_to_tokens(p))
            .collect();

        // Convert field types to tokens
        let field_tokens: Vec<TokenStream> = fields
            .iter()
            .map(|ty| self.transpile_type(ty).unwrap_or_else(|_| quote! { _ }))
            .collect();

        let visibility = if is_pub {
            quote! { pub }
        } else {
            quote! {}
        };

        // DEFECT-014: Auto-add Clone to derives for Vec indexing support
        let mut extended_derives = derives.to_vec();
        if !extended_derives.contains(&"Clone".to_string()) {
            extended_derives.push("Clone".to_string());
        }

        // Generate derive attributes using helper (PARSER-077 fix)
        let derive_attrs = self.generate_derive_attributes(&extended_derives);

        // Generate tuple struct definition
        let struct_def = if type_params.is_empty() {
            quote! {
                #derive_attrs
                #visibility struct #struct_name(#(pub #field_tokens),*);
            }
        } else {
            quote! {
                #derive_attrs
                #visibility struct #struct_name<#(#type_param_tokens),*>(#(pub #field_tokens),*);
            }
        };

        Ok(struct_def)
    }

    /// Helper: Check if any field has a reference type (for lifetime detection)
    /// Complexity: 2 (simple iteration + match)
    pub(crate) fn has_reference_fields(&self, fields: &[StructField]) -> bool {
        use crate::frontend::ast::TypeKind;
        fields
            .iter()
            .any(|field| matches!(field.ty.kind, TypeKind::Reference { .. }))
    }

    /// Helper: Check if type params already contain a lifetime
    /// Complexity: 1 (simple predicate)
    pub(crate) fn has_lifetime_params(&self, type_params: &[String]) -> bool {
        type_params.iter().any(|p| p.starts_with('\''))
    }

    /// DEFECT-021 FIX: Parse type parameter string to `TokenStream`
    /// Handles both simple params ("T") and params with bounds ("T: Clone + Debug")
    fn parse_type_param_to_tokens(p: &str) -> TokenStream {
        if p.starts_with('\'') {
            // Lifetime parameter
            let lifetime = Lifetime::new(p, proc_macro2::Span::call_site());
            quote! { #lifetime }
        } else if p.contains(':') {
            // Type parameter with trait bounds (e.g., "T: Clone + Debug")
            syn::parse_str::<syn::TypeParam>(p).map_or_else(
                |_| {
                    // Fallback: just use the name part
                    let name = p.split(':').next().unwrap_or(p).trim();
                    let ident = format_ident!("{}", name);
                    quote! { #ident }
                },
                |tp| quote! { #tp },
            )
        } else {
            // Simple type parameter
            let ident = format_ident!("{}", p);
            quote! { #ident }
        }
    }

    /// Helper: Transpile type with explicit lifetime annotation for struct fields
    /// Complexity: 3 (type matching + recursive call)
    pub(crate) fn transpile_struct_field_type_with_lifetime(
        &self,
        ty: &Type,
        lifetime: &str,
    ) -> Result<TokenStream> {
        use crate::frontend::ast::TypeKind;
        match &ty.kind {
            TypeKind::Reference { is_mut, inner, .. } => {
                // Override lifetime for references
                self.transpile_reference_type(*is_mut, Some(lifetime), inner)
            }
            _ => {
                // For non-reference types, use regular transpilation
                self.transpile_type(ty)
            }
        }
    }

    /// Transpiles struct definitions
    pub fn transpile_struct(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[StructField],
        derives: &[String],
        is_pub: bool,
    ) -> Result<TokenStream> {
        self.transpile_struct_with_methods(name, type_params, fields, &[], derives, is_pub)
    }

    pub fn transpile_struct_with_methods(
        &self,
        name: &str,
        type_params: &[String],
        fields: &[StructField],
        methods: &[ClassMethod],
        derives: &[String],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let struct_name = format_ident!("{}", name);

        // BOOK-COMPAT-001: Auto-add lifetime parameter if struct has reference fields
        let needs_lifetime =
            self.has_reference_fields(fields) && !self.has_lifetime_params(type_params);
        let effective_type_params: Vec<String> = if needs_lifetime {
            let mut params = vec!["'a".to_string()];
            params.extend_from_slice(type_params);
            params
        } else {
            type_params.to_vec()
        };

        let type_param_tokens: Vec<TokenStream> = effective_type_params
            .iter()
            .map(|p| Self::parse_type_param_to_tokens(p))
            .collect();
        let field_tokens: Vec<TokenStream> = fields
            .iter()
            .map(|field| {
                let field_name = format_ident!("{}", field.name);

                // BOOK-COMPAT-001: Add lifetime to reference types if needed
                let field_type = if needs_lifetime {
                    self.transpile_struct_field_type_with_lifetime(&field.ty, "'a")
                        .unwrap_or_else(|_| quote! { _ })
                } else {
                    self.transpile_type(&field.ty)
                        .unwrap_or_else(|_| quote! { _ })
                };

                use crate::frontend::ast::Visibility;
                match &field.visibility {
                    Visibility::Public => quote! { pub #field_name: #field_type },
                    Visibility::PubCrate => quote! { pub(crate) #field_name: #field_type },
                    Visibility::PubSuper => quote! { pub(super) #field_name: #field_type },
                    Visibility::Private | Visibility::Protected => {
                        quote! { #field_name: #field_type }
                    }
                }
            })
            .collect();
        let visibility = if is_pub {
            quote! { pub }
        } else {
            quote! {}
        };

        // DEFECT-014: Auto-add Clone to derives for Vec indexing support
        let mut extended_derives = derives.to_vec();
        if !extended_derives.contains(&"Clone".to_string()) {
            extended_derives.push("Clone".to_string());
        }

        // Generate derive attributes using helper (PARSER-077 fix)
        let derive_attrs = self.generate_derive_attributes(&extended_derives);

        // Generate struct definition
        let struct_def = if effective_type_params.is_empty() {
            quote! {
                #derive_attrs
                #visibility struct #struct_name {
                    #(#field_tokens,)*
                }
            }
        } else {
            quote! {
                #derive_attrs
                #visibility struct #struct_name<#(#type_param_tokens),*> {
                    #(#field_tokens,)*
                }
            }
        };

        // Check if any fields have default values
        let has_defaults = fields.iter().any(|f| f.default_value.is_some());

        // Generate Default impl if there are default values
        if has_defaults {
            let default_field_tokens: Result<Vec<_>> = fields
                .iter()
                .map(|field| -> Result<TokenStream> {
                    let field_name = format_ident!("{}", field.name);
                    if let Some(ref default_expr) = field.default_value {
                        let default_value = self.transpile_expr(default_expr)?;
                        Ok(quote! { #field_name: #default_value })
                    } else {
                        Ok(quote! { #field_name: Default::default() })
                    }
                })
                .collect();
            let default_field_tokens = default_field_tokens?;

            let default_impl = if type_params.is_empty() {
                quote! {
                    impl Default for #struct_name {
                        fn default() -> Self {
                            Self {
                                #(#default_field_tokens,)*
                            }
                        }
                    }
                }
            } else {
                // For generic structs, we need to add Default bounds
                let where_clause_tokens: Vec<_> = type_params
                    .iter()
                    .map(|p| {
                        let param_ident = format_ident!("{}", p);
                        quote! { #param_ident: Default }
                    })
                    .collect();

                quote! {
                    impl<#(#type_param_tokens),*> Default for #struct_name<#(#type_param_tokens),*>
                    where
                        #(#where_clause_tokens),*
                    {
                        fn default() -> Self {
                            Self {
                                #(#default_field_tokens,)*
                            }
                        }
                    }
                }
            };

            if methods.is_empty() {
                Ok(quote! {
                    #struct_def

                    #default_impl
                })
            } else {
                let method_tokens = self.transpile_class_methods(methods)?;
                let type_param_tokens = self.generate_class_type_param_tokens(type_params);
                let impl_block = if type_param_tokens.is_empty() {
                    quote! {
                        impl #struct_name {
                            #(#method_tokens)*
                        }
                    }
                } else {
                    quote! {
                        impl<#(#type_param_tokens),*> #struct_name<#(#type_param_tokens),*> {
                            #(#method_tokens)*
                        }
                    }
                };
                Ok(quote! {
                    #struct_def

                    #default_impl

                    #impl_block
                })
            }
        } else {
            if methods.is_empty() {
                Ok(struct_def)
            } else {
                let method_tokens = self.transpile_class_methods(methods)?;
                let type_param_tokens = self.generate_class_type_param_tokens(type_params);
                let impl_block = if type_param_tokens.is_empty() {
                    quote! {
                        impl #struct_name {
                            #(#method_tokens)*
                        }
                    }
                } else {
                    quote! {
                        impl<#(#type_param_tokens),*> #struct_name<#(#type_param_tokens),*> {
                            #(#method_tokens)*
                        }
                    }
                };
                Ok(quote! {
                    #struct_def

                    #impl_block
                })
            }
        }
    }

    /// Transpiles class definitions to struct + impl blocks following Ruchy class sugar specification
    /// Transpile class to struct with impl blocks
    /// Complexity: 5 (within Toyota Way limits)
    pub fn transpile_class(
        &self,
        name: &str,
        type_params: &[String],
        _traits: &[String],
        fields: &[StructField],
        constructors: &[Constructor],
        methods: &[ClassMethod],
        constants: &[crate::frontend::ast::ClassConstant],
        derives: &[String],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let struct_tokens = self.transpile_struct(name, type_params, fields, derives, is_pub)?;
        let type_param_tokens = self.generate_class_type_param_tokens(type_params);
        let struct_name = format_ident!("{}", name);

        let constructor_tokens = self.transpile_constructors(constructors)?;
        let method_tokens = self.transpile_class_methods(methods)?;
        let constant_tokens = self.transpile_class_constants(constants)?;

        let impl_tokens = self.generate_impl_block(
            &struct_name,
            &type_param_tokens,
            &constant_tokens,
            &constructor_tokens,
            &method_tokens,
        );

        let default_impl = self.generate_default_impl(fields, &struct_name, &type_param_tokens)?;

        Ok(quote! {
            #struct_tokens
            #impl_tokens
            #default_impl
        })
    }

    /// Generate derive attributes
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn generate_derive_attributes(&self, derives: &[String]) -> TokenStream {
        if derives.is_empty() {
            quote! {}
        } else {
            let derive_idents: Vec<_> = derives.iter().map(|d| format_ident!("{}", d)).collect();
            quote! { #[derive(#(#derive_idents),*)] }
        }
    }

    /// Generate type parameter tokens for classes
    /// Complexity: 2 (within Toyota Way limits)
    pub(crate) fn generate_class_type_param_tokens(&self, type_params: &[String]) -> Vec<TokenStream> {
        type_params
            .iter()
            .map(|p| {
                if p.starts_with('\'') {
                    let lifetime = Lifetime::new(p, proc_macro2::Span::call_site());
                    quote! { #lifetime }
                } else {
                    let ident = format_ident!("{}", p);
                    quote! { #ident }
                }
            })
            .collect()
    }

    /// Transpile constructors to methods
    /// Complexity: 6 (within Toyota Way limits)
    pub(crate) fn transpile_constructors(&self, constructors: &[Constructor]) -> Result<Vec<TokenStream>> {
        constructors
            .iter()
            .map(|ctor| {
                let params = self.transpile_params(&ctor.params)?;
                let body = self.transpile_expr(&ctor.body)?;
                let visibility = if ctor.is_pub {
                    quote! { pub }
                } else {
                    quote! {}
                };
                let method_name = ctor
                    .name
                    .as_ref()
                    .map_or_else(|| format_ident!("new"), |n| format_ident!("{}", n));
                let return_type = if let Some(ref ret_ty) = ctor.return_type {
                    let ret_tokens = self.transpile_type(ret_ty)?;
                    quote! { -> #ret_tokens }
                } else {
                    quote! { -> Self }
                };

                Ok(quote! {
                    #visibility fn #method_name(#(#params),*) #return_type {
                        #body
                    }
                })
            })
            .collect()
    }

    /// Transpile class methods
    /// Complexity: 5 (within Toyota Way limits)
    pub(crate) fn transpile_class_methods(&self, methods: &[ClassMethod]) -> Result<Vec<TokenStream>> {
        methods
            .iter()
            .map(|method| {
                let method_name = format_ident!("{}", method.name);
                let params = self.transpile_params(&method.params)?;
                let return_type = if let Some(ref ret_ty) = method.return_type {
                    let ret_tokens = self.transpile_type(ret_ty)?;
                    quote! { -> #ret_tokens }
                } else {
                    quote! {}
                };
                let body = self.transpile_expr(&method.body)?;
                let visibility = if method.is_pub {
                    quote! { pub }
                } else {
                    quote! {}
                };

                Ok(quote! {
                    #visibility fn #method_name(#(#params),*) #return_type {
                        #body
                    }
                })
            })
            .collect()
    }

    /// Transpile class constants
    /// Complexity: 3 (within Toyota Way limits)
    pub(crate) fn transpile_class_constants(
        &self,
        constants: &[crate::frontend::ast::ClassConstant],
    ) -> Result<Vec<TokenStream>> {
        constants
            .iter()
            .map(|constant| {
                let const_name = format_ident!("{}", constant.name);
                let const_type = self.transpile_type(&constant.ty)?;
                let const_value = self.transpile_expr(&constant.value)?;
                let visibility = if constant.is_pub {
                    quote! { pub }
                } else {
                    quote! {}
                };

                Ok(quote! {
                    #visibility const #const_name: #const_type = #const_value;
                })
            })
            .collect()
    }

    /// Generate impl block
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn generate_impl_block(
        &self,
        struct_name: &proc_macro2::Ident,
        type_param_tokens: &[TokenStream],
        constant_tokens: &[TokenStream],
        constructor_tokens: &[TokenStream],
        method_tokens: &[TokenStream],
    ) -> TokenStream {
        if type_param_tokens.is_empty() {
            quote! {
                impl #struct_name {
                    #(#constant_tokens)*
                    #(#constructor_tokens)*
                    #(#method_tokens)*
                }
            }
        } else {
            quote! {
                impl<#(#type_param_tokens),*> #struct_name<#(#type_param_tokens),*> {
                    #(#constant_tokens)*
                    #(#constructor_tokens)*
                    #(#method_tokens)*
                }
            }
        }
    }

    /// Generate Default impl if needed
    /// Complexity: 4 (within Toyota Way limits)
    pub(crate) fn generate_default_impl(
        &self,
        fields: &[StructField],
        struct_name: &proc_macro2::Ident,
        type_param_tokens: &[TokenStream],
    ) -> Result<TokenStream> {
        let has_defaults = fields.iter().any(|f| f.default_value.is_some());
        if !has_defaults {
            return Ok(quote! {});
        }

        let default_field_tokens: Result<Vec<_>> = fields
            .iter()
            .map(|field| {
                let field_name = format_ident!("{}", field.name);
                if let Some(ref default_expr) = field.default_value {
                    let default_value = self.transpile_expr(default_expr)?;
                    Ok(quote! { #field_name: #default_value })
                } else {
                    Ok(quote! { #field_name: Default::default() })
                }
            })
            .collect();
        let default_field_tokens = default_field_tokens?;

        Ok(if type_param_tokens.is_empty() {
            quote! {
                impl Default for #struct_name {
                    fn default() -> Self {
                        Self {
                            #(#default_field_tokens,)*
                        }
                    }
                }
            }
        } else {
            quote! {
                impl<#(#type_param_tokens),*> Default for #struct_name<#(#type_param_tokens),*> {
                    fn default() -> Self {
                        Self {
                            #(#default_field_tokens,)*
                        }
                    }
                }
            }
        })
    }

    /// Simple parameter transpilation for class methods (no body analysis needed)
    pub(crate) fn transpile_params(&self, params: &[crate::frontend::ast::Param]) -> Result<Vec<TokenStream>> {
        params
            .iter()
            .map(|param| -> Result<TokenStream> {
                let param_name = param.name();

                // Handle self parameters specially
                if param_name == "self" {
                    use crate::frontend::ast::TypeKind;
                    match &param.ty.kind {
                        TypeKind::Reference { is_mut: true, .. } => Ok(quote! { &mut self }),
                        TypeKind::Reference { is_mut: false, .. } => Ok(quote! { &self }),
                        _ => {
                            // Check if it's a mutable move (mut self)
                            if param.is_mutable {
                                Ok(quote! { mut self })
                            } else {
                                Ok(quote! { self })
                            }
                        }
                    }
                } else {
                    // Regular parameter
                    // TRANSPILER-005 FIX: Preserve mut keyword for mutable parameters
                    let param_ident = format_ident!("{}", param_name);
                    let type_tokens = self.transpile_type(&param.ty)?;
                    if param.is_mutable {
                        Ok(quote! { mut #param_ident: #type_tokens })
                    } else {
                        Ok(quote! { #param_ident: #type_tokens })
                    }
                }
            })
            .collect()
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
        let type_param_tokens: Vec<_> = type_params
            .iter()
            .map(|p| Self::parse_type_param_to_tokens(p))
            .collect();
        // Check if any variant has discriminant values
        let has_discriminants = variants.iter().any(|v| v.discriminant.is_some());
        let variant_tokens: Vec<TokenStream> = variants
            .iter()
            .map(|variant| {
                use crate::frontend::ast::EnumVariantKind;
                let variant_name = format_ident!("{}", variant.name);

                match &variant.kind {
                    EnumVariantKind::Tuple(fields) => {
                        // Tuple variant: Write(String)
                        let field_types: Vec<TokenStream> = fields
                            .iter()
                            .map(|ty| self.transpile_type(ty).unwrap_or_else(|_| quote! { _ }))
                            .collect();
                        quote! { #variant_name(#(#field_types),*) }
                    }
                    EnumVariantKind::Struct(fields) => {
                        // Struct variant: Move { x: i32, y: i32 }
                        let field_defs: Vec<TokenStream> = fields
                            .iter()
                            .map(|field| {
                                let field_name = format_ident!("{}", field.name);
                                let field_type = self
                                    .transpile_type(&field.ty)
                                    .unwrap_or_else(|_| quote! { _ });
                                quote! { #field_name: #field_type }
                            })
                            .collect();
                        quote! { #variant_name { #(#field_defs),* } }
                    }
                    EnumVariantKind::Unit => {
                        // Unit variant with optional discriminant
                        if let Some(disc_value) = variant.discriminant {
                            let disc_literal =
                                proc_macro2::Literal::i32_unsuffixed(disc_value as i32);
                            quote! { #variant_name = #disc_literal }
                        } else {
                            quote! { #variant_name }
                        }
                    }
                }
            })
            .collect();
        let visibility = if is_pub {
            quote! { pub }
        } else {
            quote! {}
        };
        // Add #[derive(Debug, Clone, PartialEq)] for better usability
        let derive_attr = quote! { #[derive(Debug, Clone, PartialEq)] };

        // Add #[repr(i32)] attribute if enum has discriminant values
        let repr_attr = if has_discriminants {
            quote! { #[repr(i32)] }
        } else {
            quote! {}
        };
        if type_params.is_empty() {
            Ok(quote! {
                #derive_attr
                #repr_attr
                #visibility enum #enum_name {
                    #(#variant_tokens,)*
                }
            })
        } else {
            Ok(quote! {
                #derive_attr
                #repr_attr
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
        associated_types: &[String],
        methods: &[TraitMethod],
        is_pub: bool,
    ) -> Result<TokenStream> {
        let trait_name = format_ident!("{}", name);

        // Generate associated type declarations: type Item;
        let associated_type_tokens: Vec<TokenStream> = associated_types
            .iter()
            .map(|type_name| {
                let type_ident = format_ident!("{}", type_name);
                quote! { type #type_ident; }
            })
            .collect();

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
                let visibility = if method.is_pub {
                    quote! { pub }
                } else {
                    quote! {}
                };
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
        let type_param_tokens: Vec<_> = type_params
            .iter()
            .map(|p| Self::parse_type_param_to_tokens(p))
            .collect();
        let visibility = if is_pub {
            quote! { pub }
        } else {
            quote! {}
        };
        if type_params.is_empty() {
            Ok(quote! {
                #visibility trait #trait_name {
                    #(#associated_type_tokens)*
                    #(#method_tokens)*
                }
            })
        } else {
            Ok(quote! {
                #visibility trait #trait_name<#(#type_param_tokens),*> {
                    #(#associated_type_tokens)*
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
        // DEFECT-027 FIX: Strip generic parameters from for_type if present
        // e.g., "Container<T>" -> "Container"
        let base_type = for_type.split('<').next().unwrap_or(for_type).trim();
        let type_ident = format_ident!("{}", base_type);
        let method_tokens: Result<Vec<_>> = methods
            .iter()
            .map(|method| {
                let method_name = format_ident!("{}", method.name);
                // Process parameters
                let param_tokens: Vec<TokenStream> = method
                    .params
                    .iter()
                    .map(|param| {
                        let name = param.name();
                        // QUALITY-001: Handle special Rust receiver syntax (&self, &mut self, self)
                        // Method receivers in Rust have special syntax that differs from normal parameters
                        if name == "self" {
                            // Check if it's a reference type (in the TYPE, not the name)
                            if let TypeKind::Reference { is_mut, .. } = &param.ty.kind {
                                if *is_mut {
                                    quote! { &mut self }
                                } else {
                                    quote! { &self }
                                }
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
                let visibility = if method.is_pub {
                    quote! { pub }
                } else {
                    quote! {}
                };
                Ok(quote! {
                    #visibility fn #method_name(#(#param_tokens),*) #return_type_tokens {
                        #body_tokens
                    }
                })
            })
            .collect();
        let method_tokens = method_tokens?;
        let type_param_tokens: Vec<_> = type_params
            .iter()
            .map(|p| Self::parse_type_param_to_tokens(p))
            .collect();
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
    /// ```text
    /// Ruchy: extend String { fun is_palindrome(&self) -> bool { ... } }
    /// Rust:  trait StringExt { fn is_palindrome(&self) -> bool; }
    ///        impl StringExt for String { fn is_palindrome(&self) -> bool { ... } }
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
                    .map(|param| {
                        let name = param.name();
                        // QUALITY-001: Handle special Rust receiver syntax (&self, &mut self, self)
                        // Method receivers in Rust have special syntax that differs from normal parameters
                        if name == "self" {
                            // Check if it's a reference type (in the TYPE, not the name)
                            if let TypeKind::Reference { is_mut, .. } = &param.ty.kind {
                                if *is_mut {
                                    quote! { &mut self }
                                } else {
                                    quote! { &self }
                                }
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
