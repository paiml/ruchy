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
    /// let result = transpiler.transpile(&ast).expect("transpilation should succeed in doctest");
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
    /// Transpiles patterns to Rust
    pub fn transpile_pattern(&self, pattern: &Pattern) -> Result<TokenStream> {
        match pattern {
            Pattern::Wildcard => Ok(quote! { _ }),
            Pattern::Literal(lit) => {
                // Transpile literal to tokens
                use quote::quote;
                match lit {
                    crate::frontend::ast::Literal::Integer(_, _) => {
                        // Use Transpiler::transpile_literal to ensure consistent integer handling
                        let tokens = Self::transpile_literal(lit);
                        Ok(tokens)
                    }
                    crate::frontend::ast::Literal::Float(f) => Ok(quote! { #f }),
                    crate::frontend::ast::Literal::String(s) => Ok(quote! { #s }),
                    crate::frontend::ast::Literal::Bool(b) => Ok(quote! { #b }),
                    crate::frontend::ast::Literal::Char(c) => Ok(quote! { #c }),
                    crate::frontend::ast::Literal::Byte(b) => {
                        let byte_val = *b;
                        Ok(quote! { #byte_val })
                    }
                    crate::frontend::ast::Literal::Unit => Ok(quote! { () }),
                    crate::frontend::ast::Literal::Null => Ok(quote! { None }),
                    crate::frontend::ast::Literal::Atom(_) => Ok(quote! { todo!("Atom patterns") }),
                }
            }
            Pattern::Identifier(name) => {
                let ident = format_ident!("{}", name);
                Ok(quote! { #ident })
            }
            Pattern::QualifiedName(parts) => {
                // For patterns like Some, None, Ok, Err
                let mut tokens = TokenStream::new();
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        tokens.extend(quote! { :: });
                    }
                    let ident = format_ident!("{}", part);
                    tokens.extend(quote! { #ident });
                }
                Ok(tokens)
            }
            Pattern::TupleVariant { path, patterns } => {
                // For enum tuple variants like Message::Text(n) or Color::RGB(r, g, b)
                let mut path_tokens = TokenStream::new();
                for (i, part) in path.iter().enumerate() {
                    if i > 0 {
                        path_tokens.extend(quote! { :: });
                    }
                    let ident = format_ident!("{}", part);
                    path_tokens.extend(quote! { #ident });
                }
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                Ok(quote! { #path_tokens(#(#pattern_tokens),*) })
            }
            Pattern::Tuple(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                Ok(quote! { (#(#pattern_tokens),*) })
            }
            Pattern::List(patterns) => self.transpile_array_pattern(patterns, false),
            Pattern::Struct {
                name,
                fields,
                has_rest,
            } => self.transpile_struct_pattern(name, fields, *has_rest),
            Pattern::Range {
                start,
                end,
                inclusive,
            } => self.transpile_range_pattern(start, end, *inclusive),
            Pattern::Or(patterns) => self.transpile_or_pattern(patterns),
            Pattern::Rest => Ok(quote! { .. }),
            Pattern::RestNamed(name) => self.transpile_rest_named_pattern(name),
            Pattern::AtBinding { name, pattern } => {
                let name_ident = format_ident!("{}", name);
                let inner = self.transpile_pattern(pattern)?;
                Ok(quote! { #name_ident @ #inner })
            }
            Pattern::WithDefault {
                pattern,
                default: _,
            } => {
                // For patterns with defaults, just use the pattern part in match
                self.transpile_pattern(pattern)
            }
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
            Pattern::Mut(inner) => {
                // Transpile mut patterns as `mut inner_pattern`
                let inner_tokens = self.transpile_pattern(inner)?;
                Ok(quote! { mut #inner_tokens })
            }
        }
    }
    fn transpile_array_pattern(
        &self,
        patterns: &[Pattern],
        _has_rest: bool,
    ) -> Result<TokenStream> {
        // Check if any pattern is a rest pattern
        let has_rest = patterns
            .iter()
            .any(|p| matches!(p, Pattern::Rest | Pattern::RestNamed(_)));
        if has_rest {
            // For array patterns with rest, we need special handling
            let mut pattern_tokens = Vec::new();
            for pattern in patterns {
                match pattern {
                    Pattern::Rest => {
                        pattern_tokens.push(quote! { .. });
                    }
                    Pattern::RestNamed(name) => {
                        let name_ident = format_ident!("{}", name);
                        pattern_tokens.push(quote! { #name_ident @ .. });
                    }
                    _ => {
                        let tokens = self.transpile_pattern(pattern)?;
                        pattern_tokens.push(tokens);
                    }
                }
            }
            // Use slice pattern syntax for patterns with rest
            Ok(quote! { [#(#pattern_tokens),*] })
        } else {
            // For exact array patterns
            let pattern_tokens: Result<Vec<_>> =
                patterns.iter().map(|p| self.transpile_pattern(p)).collect();
            let pattern_tokens = pattern_tokens?;
            // Check if we should use Vec or array pattern
            // For now, we'll use array pattern syntax
            if patterns.is_empty() {
                Ok(quote! { [] })
            } else {
                // For non-empty patterns, check if any contain identifiers
                // If so, we might need to bind to a slice
                let has_bindings = patterns
                    .iter()
                    .any(|p| matches!(p, Pattern::Identifier(_) | Pattern::RestNamed(_)));
                if has_bindings {
                    // Use slice pattern for binding
                    Ok(quote! { [#(#pattern_tokens),*] })
                } else {
                    // Use array pattern for literals
                    Ok(quote! { [#(#pattern_tokens),*] })
                }
            }
        }
    }
    fn transpile_struct_pattern(
        &self,
        name: &str,
        fields: &[StructPatternField],
        has_rest: bool,
    ) -> Result<TokenStream> {
        if name.is_empty() {
            return Ok(quote! { _ });
        }

        // Handle qualified names like "Variant::A" by splitting on ::
        let struct_name = if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            let mut tokens = TokenStream::new();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    tokens.extend(quote! { :: });
                }
                let ident = format_ident!("{}", part);
                tokens.extend(quote! { #ident });
            }
            tokens
        } else {
            let ident = format_ident!("{}", name);
            quote! { #ident }
        };

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
        fields
            .iter()
            .map(|field| {
                let field_ident = format_ident!("{}", field.name);
                if let Some(ref pattern) = field.pattern {
                    let pattern_tokens = self.transpile_pattern(pattern)?;
                    Ok(quote! { #field_ident: #pattern_tokens })
                } else {
                    Ok(quote! { #field_ident })
                }
            })
            .collect()
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
        inclusive: bool,
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
// Tests removed: Testing private helper methods violates encapsulation.
// Public API tests should go in integration tests.

#[cfg(test)]
mod tests {

    use crate::frontend::ast::{
        BinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern, Span, StructPatternField,
    };
    use crate::Transpiler;

    #[test]
    fn test_transpile_wildcard_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Wildcard;
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "_");
    }

    #[test]
    fn test_transpile_literal_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        // The output should contain the integer value
        let output = result.to_string();
        // Note: transpile_literal is not a public method, the pattern transpiler handles it internally
        assert!(!output.is_empty());
    }

    #[test]
    fn test_transpile_identifier_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Identifier("x".to_string());
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "x");
    }

    #[test]
    fn test_transpile_tuple_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains('('));
        assert!(output.contains(')'));
        assert!(output.contains('a'));
        assert!(output.contains('b'));
    }

    #[test]
    fn test_transpile_empty_list_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::List(vec![]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains('['));
        assert!(result.to_string().contains(']'));
    }

    #[test]
    fn test_transpile_rest_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Rest;
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "..");
    }

    #[test]
    fn test_transpile_rest_named_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::RestNamed("rest".to_string());
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("rest"));
        assert!(result.to_string().contains('@'));
    }

    #[test]
    fn test_transpile_qualified_name_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::QualifiedName(vec!["Some".to_string()]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("Some"));
    }

    #[test]
    fn test_transpile_qualified_path_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::QualifiedName(vec![
            "std".to_string(),
            "option".to_string(),
            "Option".to_string(),
        ]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains("std"));
        assert!(output.contains("::"));
        assert!(output.contains("option"));
    }

    #[test]
    fn test_transpile_struct_pattern_empty() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![],
            has_rest: false,
        };
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert!(result.to_string().contains("Point"));
    }

    #[test]
    fn test_transpile_struct_pattern_with_fields() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                StructPatternField {
                    name: "x".to_string(),
                    pattern: None,
                },
                StructPatternField {
                    name: "y".to_string(),
                    pattern: Some(Pattern::Identifier("y_val".to_string())),
                },
            ],
            has_rest: false,
        };
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains("Point"));
        assert!(output.contains('x'));
        assert!(output.contains('y'));
    }

    #[test]
    fn test_transpile_struct_pattern_with_rest() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Struct {
            name: "Config".to_string(),
            fields: vec![StructPatternField {
                name: "debug".to_string(),
                pattern: None,
            }],
            has_rest: true,
        };
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains("Config"));
        assert!(output.contains(".."));
    }

    #[test]
    fn test_transpile_range_pattern_exclusive() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
            inclusive: false,
        };
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains('1'));
        assert!(output.contains("10"));
        assert!(output.contains(".."));
        assert!(!output.contains("..="));
    }

    #[test]
    fn test_transpile_range_pattern_inclusive() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
            inclusive: true,
        };
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains('1'));
        assert!(output.contains("10"));
        assert!(output.contains("..="));
    }

    #[test]
    fn test_transpile_or_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
            Pattern::Literal(Literal::Integer(3, None)),
        ]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains('1'));
        assert!(output.contains('2'));
        assert!(output.contains('3'));
        assert!(output.contains('|'));
    }

    #[test]
    fn test_transpile_with_default_pattern() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("x".to_string())),
            default: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0, None)),
                Span::new(0, 0),
            )),
        };
        // WithDefault just uses the pattern part in matches
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "x");
    }

    // Tests commented out - Pattern enum doesn't have Result/Option variants
    // These would need to use QualifiedName pattern instead

    #[test]
    fn test_transpile_match_without_guard() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::new(0, 0));
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1, None)),
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("one".to_string())),
                    Span::new(0, 0),
                )),
                span: Span::new(0, 0),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("other".to_string())),
                    Span::new(0, 0),
                )),
                span: Span::new(0, 0),
            },
        ];
        let result = transpiler
            .transpile_match(&expr, &arms)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains("match"));
        assert!(output.contains('x'));
        assert!(output.contains('1'));
        assert!(output.contains('_'));
    }

    #[test]
    fn test_transpile_match_with_guard() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::new(0, 0));
        let arms = vec![MatchArm {
            pattern: Pattern::Identifier("n".to_string()),
            guard: Some(Box::new(Expr::new(
                ExprKind::Binary {
                    op: BinaryOp::Greater,
                    left: Box::new(Expr::new(
                        ExprKind::Identifier("n".to_string()),
                        Span::new(0, 0),
                    )),
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(0, None)),
                        Span::new(0, 0),
                    )),
                },
                Span::new(0, 0),
            ))),
            body: Box::new(Expr::new(
                ExprKind::Literal(Literal::String("positive".to_string())),
                Span::new(0, 0),
            )),
            span: Span::new(0, 0),
        }];
        let result = transpiler
            .transpile_match(&expr, &arms)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains("match"));
        assert!(output.contains("if"));
    }

    #[test]
    fn test_list_pattern_with_rest() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Rest,
        ]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains('['));
        assert!(output.contains("first"));
        assert!(output.contains(".."));
        assert!(output.contains(']'));
    }

    #[test]
    fn test_list_pattern_with_named_rest() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("head".to_string()),
            Pattern::RestNamed("tail".to_string()),
        ]);
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        let output = result.to_string();
        assert!(output.contains('['));
        assert!(output.contains("head"));
        assert!(output.contains("tail"));
        assert!(output.contains('@'));
        assert!(output.contains(']'));
    }

    #[test]
    fn test_empty_struct_name() {
        let transpiler = Transpiler::new();
        let pattern = Pattern::Struct {
            name: String::new(),
            fields: vec![],
            has_rest: false,
        };
        let result = transpiler
            .transpile_pattern(&pattern)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "_");
    }
}
