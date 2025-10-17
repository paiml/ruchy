//! Collection transpilation helpers (list, set, tuple, range, objects, structs)
#![allow(clippy::missing_errors_doc)]

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    pub fn transpile_list(&self, elements: &[Expr]) -> Result<TokenStream> {
        // Check if any elements are spread expressions
        let has_spread = elements
            .iter()
            .any(|e| matches!(e.kind, crate::frontend::ast::ExprKind::Spread { .. }));
        if has_spread {
            // Handle spread expressions by building vector with extends
            let mut statements = Vec::new();
            statements.push(quote! { let mut __temp_vec = Vec::new(); });
            for element in elements {
                if let crate::frontend::ast::ExprKind::Spread { expr } = &element.kind {
                    let expr_tokens = self.transpile_expr(expr)?;
                    statements.push(quote! { __temp_vec.extend(#expr_tokens); });
                } else {
                    let expr_tokens = self.transpile_expr(element)?;
                    statements.push(quote! { __temp_vec.push(#expr_tokens); });
                }
            }
            statements.push(quote! { __temp_vec });
            Ok(quote! { { #(#statements)* } })
        } else if elements.is_empty() {
            // Empty arrays need vec![] to avoid type ambiguity ([] requires type annotation)
            Ok(quote! { vec![] })
        } else {
            // No spread expressions, use fixed-size array syntax [elem1, elem2, ...]
            // This preserves array type and matches Rust's array literal syntax
            // Rust will infer the type as [T; N] automatically
            let element_tokens: Result<Vec<_>> =
                elements.iter().map(|e| self.transpile_expr(e)).collect();
            let element_tokens = element_tokens?;
            Ok(quote! { [#(#element_tokens),*] })
        }
    }

    /// Transpiles set literals into `HashSet`
    pub fn transpile_set(&self, elements: &[Expr]) -> Result<TokenStream> {
        // Check if any elements are spread expressions
        let has_spread = elements
            .iter()
            .any(|e| matches!(e.kind, crate::frontend::ast::ExprKind::Spread { .. }));

        if has_spread {
            // Handle spread expressions by building hashset with extends
            let mut statements = Vec::new();
            statements.push(quote! { let mut __temp_set = std::collections::HashSet::new(); });

            for element in elements {
                if let crate::frontend::ast::ExprKind::Spread { expr } = &element.kind {
                    let expr_tokens = self.transpile_expr(expr)?;
                    statements.push(quote! { __temp_set.extend(#expr_tokens); });
                } else {
                    let expr_tokens = self.transpile_expr(element)?;
                    statements.push(quote! { __temp_set.insert(#expr_tokens); });
                }
            }

            statements.push(quote! { __temp_set });
            Ok(quote! { { #(#statements)* } })
        } else if elements.is_empty() {
            // Empty set literal
            Ok(quote! { std::collections::HashSet::new() })
        } else {
            // No spread expressions, build HashSet with inserts
            let mut statements = Vec::new();
            statements.push(quote! { let mut __temp_set = std::collections::HashSet::new(); });

            for element in elements {
                let expr_tokens = self.transpile_expr(element)?;
                statements.push(quote! { __temp_set.insert(#expr_tokens); });
            }

            statements.push(quote! { __temp_set });
            Ok(quote! { { #(#statements)* } })
        }
    }

    /// Transpiles tuple literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_tuple;
    ///
    /// let result = transpile_tuple(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_tuple(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;
        Ok(quote! { (#(#element_tokens),*) })
    }
    /// Transpiles range expressions
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_range;
    ///
    /// let result = transpile_range(true);
    /// assert_eq!(result, Ok(true));
    /// ```
    pub fn transpile_range(
        &self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<TokenStream> {
        let start_tokens = self.transpile_expr(start)?;
        let end_tokens = self.transpile_expr(end)?;
        if inclusive {
            Ok(quote! { #start_tokens..=#end_tokens })
        } else {
            Ok(quote! { #start_tokens..#end_tokens })
        }
    }
    /// Transpiles object literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_object_literal;
    ///
    /// let result = transpile_object_literal(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_object_literal(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<TokenStream> {
        let field_tokens = self.collect_hashmap_field_tokens(fields)?;
        // DEFECT-DICT-DETERMINISM FIX: Use BTreeMap for deterministic key ordering
        // BTreeMap maintains sorted order, HashMap has non-deterministic iteration order
        Ok(quote! {
            {
                let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
                #(#field_tokens)*
                map
            }
        })
    }
    fn collect_hashmap_field_tokens(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<Vec<TokenStream>> {
        use crate::frontend::ast::ObjectField;
        let mut field_tokens = Vec::new();
        for field in fields {
            let token = match field {
                ObjectField::KeyValue { key, value } => {
                    let value_tokens = self.transpile_expr(value)?;
                    quote! { map.insert(#key.to_string(), (#value_tokens).to_string()); }
                }
                ObjectField::Spread { expr } => {
                    let expr_tokens = self.transpile_expr(expr)?;
                    // For spread syntax, merge the other map into this one
                    quote! {
                        for (k, v) in #expr_tokens {
                            map.insert(k, v);
                        }
                    }
                }
            };
            field_tokens.push(token);
        }
        Ok(field_tokens)
    }
    /// Transpiles struct literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_struct_literal;
    ///
    /// let result = transpile_struct_literal("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
        base: Option<&Expr>,
    ) -> Result<TokenStream> {
        let struct_name = format_ident!("{}", name);
        let mut field_tokens = Vec::new();
        for (field_name, value) in fields {
            let field_ident = format_ident!("{}", field_name);
            let value_tokens = match &value.kind {
                // Convert string literals to String for struct fields
                ExprKind::Literal(Literal::String(s)) => {
                    quote! { #s.to_string() }
                }
                _ => self.transpile_expr(value)?,
            };
            field_tokens.push(quote! { #field_ident: #value_tokens });
        }

        // Handle struct update syntax
        if let Some(base_expr) = base {
            let base_tokens = self.transpile_expr(base_expr)?;
            Ok(quote! {
                #struct_name {
                    #(#field_tokens,)*
                    ..#base_tokens
                }
            })
        } else {
            Ok(quote! {
                #struct_name {
                    #(#field_tokens,)*
                }
            })
        }
    }
}
