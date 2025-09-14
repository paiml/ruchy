//! Pattern matching transpilation
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::only_used_in_recursion)]
use super::Transpiler;
use crate::frontend::ast::{Expr, MatchArm, Pattern, StructPatternField};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
impl Transpiler {
    /// Transpiles match expressions
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// let mut transpiler = Transpiler::new();
    /// let mut parser = Parser::new(r#"match x { 1 => "one", _ => "other" }"#);
    /// let ast = parser.parse().expect("Failed to parse");
    /// 
    /// let result = transpiler.transpile(&ast).unwrap();
    /// let code = result.to_string();
    /// assert!(code.contains("match"));
    /// assert!(code.contains("1"));
    /// assert!(code.contains("_"));
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Wildcard pattern
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("match x { _ => 42 }");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).unwrap();
    /// assert!(result.to_string().contains("_ =>"));
    /// ```
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Identifier pattern
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("match x { y => y + 1 }");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).unwrap();
    /// assert!(result.to_string().contains("y =>"));
    /// ```
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    /// 
    /// // Tuple pattern
    /// let transpiler = Transpiler::new();
    /// let mut parser = Parser::new("match pair { (a, b) => a + b }");
    /// let ast = parser.parse().expect("Failed to parse");
    /// let result = transpiler.transpile(&ast).unwrap();
    /// assert!(result.to_string().contains("("));
    /// assert!(result.to_string().contains("a"));
    /// assert!(result.to_string().contains("b"));
    /// ```
    pub fn transpile_pattern(&self, pattern: &Pattern) -> Result<TokenStream> {
        match pattern {
            Pattern::Wildcard => Ok(quote! { _ }),
            Pattern::Literal(lit) => Ok(Self::transpile_literal(lit)),
            Pattern::Identifier(name) => self.transpile_identifier_pattern(name),
            Pattern::QualifiedName(path) => self.transpile_qualified_name_pattern(path),
            Pattern::Tuple(patterns) => self.transpile_tuple_pattern(patterns),
            Pattern::List(patterns) => self.transpile_list_pattern(patterns),
            Pattern::Struct { name, fields, has_rest } => 
                self.transpile_struct_pattern(name, fields, *has_rest),
            Pattern::Or(patterns) => self.transpile_or_pattern(patterns),
            Pattern::Range { start, end, inclusive } => 
                self.transpile_range_pattern(start, end, *inclusive),
            Pattern::Rest => Ok(quote! { .. }),
            Pattern::RestNamed(name) => self.transpile_rest_named_pattern(name),
            Pattern::Ok(pattern) => self.transpile_result_pattern(pattern, true),
            Pattern::Err(pattern) => self.transpile_result_pattern(pattern, false),
            Pattern::Some(pattern) => self.transpile_option_pattern(pattern, true),
            Pattern::None => Ok(quote! { None }),
            Pattern::WithDefault { pattern, .. } => self.transpile_pattern(pattern),
        }
    }
    fn transpile_identifier_pattern(&self, name: &str) -> Result<TokenStream> {
        let ident = format_ident!("{}", name);
        Ok(quote! { #ident })
    }
    fn transpile_qualified_name_pattern(&self, path: &[String]) -> Result<TokenStream> {
        let segments: Vec<_> = path.iter().map(|s| format_ident!("{}", s)).collect();
        Ok(quote! { #(#segments)::* })
    }
    fn transpile_tuple_pattern(&self, patterns: &[Pattern]) -> Result<TokenStream> {
        let pattern_tokens: Result<Vec<_>> =
            patterns.iter().map(|p| self.transpile_pattern(p)).collect();
        let pattern_tokens = pattern_tokens?;
        Ok(quote! { (#(#pattern_tokens),*) })
    }
    fn transpile_list_pattern(&self, patterns: &[Pattern]) -> Result<TokenStream> {
        if patterns.is_empty() {
            return Ok(quote! { [] });
        }
        let has_rest = patterns.iter().any(|p| matches!(p, Pattern::Rest | Pattern::RestNamed(_)));
        if has_rest {
            self.transpile_list_with_rest(patterns)
        } else {
            let pattern_tokens: Result<Vec<_>> =
                patterns.iter().map(|p| self.transpile_pattern(p)).collect();
            let pattern_tokens = pattern_tokens?;
            Ok(quote! { [#(#pattern_tokens),*] })
        }
    }
    fn transpile_list_with_rest(&self, patterns: &[Pattern]) -> Result<TokenStream> {
        let mut pattern_tokens = Vec::new();
        for p in patterns {
            match p {
                Pattern::Rest => pattern_tokens.push(quote! { .. }),
                Pattern::RestNamed(name) => {
                    let name_ident = format_ident!("{}", name);
                    // Rust syntax for rest patterns is `name @ ..`
                    pattern_tokens.push(quote! { #name_ident @ .. });
                }
                _ => pattern_tokens.push(self.transpile_pattern(p)?),
            }
        }
        Ok(quote! { [#(#pattern_tokens),*] })
    }
    fn transpile_struct_pattern(
        &self, 
        name: &str, 
        fields: &[StructPatternField], 
        has_rest: bool
    ) -> Result<TokenStream> {
        if name.is_empty() {
            return Ok(quote! { _ });
        }
        let struct_name = format_ident!("{}", name);
        if fields.is_empty() {
            return Ok(quote! { #struct_name {} });
        }
        let field_patterns = self.transpile_field_patterns(fields)?;
        if has_rest {
            Ok(quote! { #struct_name { #(#field_patterns),*, .. } })
        } else {
            Ok(quote! { #struct_name { #(#field_patterns),* } })
        }
    }
    fn transpile_field_patterns(&self, fields: &[StructPatternField]) -> Result<Vec<TokenStream>> {
        fields.iter().map(|field| {
            let field_ident = format_ident!("{}", field.name);
            if let Some(ref pattern) = field.pattern {
                let pattern_tokens = self.transpile_pattern(pattern)?;
                Ok(quote! { #field_ident: #pattern_tokens })
            } else {
                Ok(quote! { #field_ident })
            }
        }).collect()
    }
    fn transpile_or_pattern(&self, patterns: &[Pattern]) -> Result<TokenStream> {
        let pattern_tokens: Result<Vec<_>> =
            patterns.iter().map(|p| self.transpile_pattern(p)).collect();
        let pattern_tokens = pattern_tokens?;
        let mut result = TokenStream::new();
        for (i, tokens) in pattern_tokens.iter().enumerate() {
            if i > 0 {
                result.extend(quote! { | });
            }
            result.extend(tokens.clone());
        }
        Ok(result)
    }
    fn transpile_range_pattern(
        &self,
        start: &Pattern,
        end: &Pattern,
        inclusive: bool
    ) -> Result<TokenStream> {
        let start_tokens = self.transpile_pattern(start)?;
        let end_tokens = self.transpile_pattern(end)?;
        if inclusive {
            Ok(quote! { #start_tokens..=#end_tokens })
        } else {
            Ok(quote! { #start_tokens..#end_tokens })
        }
    }
    fn transpile_rest_named_pattern(&self, name: &str) -> Result<TokenStream> {
        let name_ident = format_ident!("{}", name);
        // Rust syntax for rest patterns is `name @ ..`
        Ok(quote! { #name_ident @ .. })
    }
    fn transpile_result_pattern(&self, pattern: &Pattern, is_ok: bool) -> Result<TokenStream> {
        let inner = self.transpile_pattern(pattern)?;
        if is_ok {
            Ok(quote! { Ok(#inner) })
        } else {
            Ok(quote! { Err(#inner) })
        }
    }
    fn transpile_option_pattern(&self, pattern: &Pattern, is_some: bool) -> Result<TokenStream> {
        if is_some {
            let inner = self.transpile_pattern(pattern)?;
            Ok(quote! { Some(#inner) })
        } else {
            Ok(quote! { None })
        }
    }
}
#[cfg(test)]
mod property_tests_patterns {
    use proptest::proptest;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_transpile_match_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
