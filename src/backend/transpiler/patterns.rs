//! Pattern matching transpilation

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::only_used_in_recursion)]

use super::Transpiler;
use crate::frontend::ast::{Expr, MatchArm, Pattern};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles match expressions
    pub fn transpile_match(&self, expr: &Expr, arms: &[MatchArm]) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;

        let mut arm_tokens = Vec::new();
        for arm in arms {
            let pattern_tokens = self.transpile_pattern(&arm.pattern)?;
            let body_tokens = self.transpile_expr(&arm.body)?;

            // Handle pattern guards if present
            if let Some(guard_expr) = &arm.guard {
                let guard_tokens = self.transpile_expr(guard_expr)?;
                arm_tokens.push(quote! {
                    #pattern_tokens if #guard_tokens => #body_tokens
                });
            } else {
                arm_tokens.push(quote! {
                    #pattern_tokens => #body_tokens
                });
            }
        }

        Ok(quote! {
            match #expr_tokens {
                #(#arm_tokens,)*
            }
        })
    }

    /// Transpiles patterns
    pub fn transpile_pattern(&self, pattern: &Pattern) -> Result<TokenStream> {
        match pattern {
            Pattern::Wildcard => Ok(quote! { _ }),
            Pattern::Literal(lit) => Ok(Self::transpile_literal(lit)),
            Pattern::Identifier(name) => {
                let ident = format_ident!("{}", name);
                Ok(quote! { #ident })
            }
            Pattern::QualifiedName(path) => {
                // Generate qualified path like Ordering::Less
                let segments: Vec<_> = path.iter().map(|s| format_ident!("{}", s)).collect();
                Ok(quote! { #(#segments)::* })
            }
            Pattern::Tuple(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                Ok(quote! { (#(#pattern_tokens),*) })
            }
            Pattern::List(patterns) => {
                if patterns.is_empty() {
                    Ok(quote! { [] })
                } else {
                    // Check for rest pattern
                    let has_rest = patterns.iter().any(|p| matches!(p, Pattern::Rest));

                    if has_rest {
                        // Handle patterns with rest
                        let mut pattern_tokens = Vec::new();
                        for p in patterns {
                            if let Pattern::Rest = p {
                                pattern_tokens.push(quote! { .. });
                            } else {
                                pattern_tokens.push(self.transpile_pattern(p)?);
                            }
                        }
                        Ok(quote! { [#(#pattern_tokens),*] })
                    } else {
                        // Simple list pattern
                        let pattern_tokens: Result<Vec<_>> =
                            patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                        let pattern_tokens = pattern_tokens?;
                        Ok(quote! { [#(#pattern_tokens),*] })
                    }
                }
            }
            Pattern::Struct { name, fields } => {
                let struct_name = format_ident!("{}", name);

                if fields.is_empty() {
                    Ok(quote! { #struct_name {} })
                } else {
                    let field_patterns: Result<Vec<_>> = fields
                        .iter()
                        .map(|field| {
                            let field_ident = format_ident!("{}", field.name);
                            if let Some(ref pattern) = field.pattern {
                                let pattern_tokens = self.transpile_pattern(pattern)?;
                                Ok(quote! { #field_ident: #pattern_tokens })
                            } else {
                                // Shorthand field pattern
                                Ok(quote! { #field_ident })
                            }
                        })
                        .collect();
                    let field_patterns = field_patterns?;
                    Ok(quote! { #struct_name { #(#field_patterns),* } })
                }
            }
            Pattern::Or(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;

                // Join patterns with |
                let mut result = TokenStream::new();
                for (i, tokens) in pattern_tokens.iter().enumerate() {
                    if i > 0 {
                        result.extend(quote! { | });
                    }
                    result.extend(tokens.clone());
                }
                Ok(result)
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let start_tokens = self.transpile_pattern(start)?;
                let end_tokens = self.transpile_pattern(end)?;

                if *inclusive {
                    Ok(quote! { #start_tokens..=#end_tokens })
                } else {
                    Ok(quote! { #start_tokens..#end_tokens })
                }
            }
            Pattern::Rest => Ok(quote! { .. }),
            Pattern::Ok(pattern) => {
                let inner = self.transpile_pattern(pattern)?;
                Ok(quote! { Ok(#inner) })
            }
            Pattern::Err(pattern) => {
                let inner = self.transpile_pattern(pattern)?;
                Ok(quote! { Err(#inner) })
            }
            Pattern::Some(pattern) => {
                let inner = self.transpile_pattern(pattern)?;
                Ok(quote! { Some(#inner) })
            }
            Pattern::None => Ok(quote! { None }),
        }
    }
}
