//! Expression transpilation methods
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_pass_by_value)] // TokenStream by value is intentional for quote! macro
use super::Transpiler;
use crate::frontend::ast::{
    BinaryOp::{self},
    Expr, ExprKind, Literal, StringPart, UnaryOp,
};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[path = "expressions_helpers/mod.rs"]
mod expressions_helpers;

impl Transpiler {
    /// Transpiles literal values
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::Literal;
    ///
    /// let lit = Literal::Integer(42, None);
    /// let result = Transpiler::transpile_literal(&lit);
    /// // Returns TokenStream
    /// ```
    pub fn transpile_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::Integer(i, type_suffix) => Self::transpile_integer(*i, type_suffix.as_deref()),
            Literal::Float(f) => quote! { #f },
            Literal::Unit => quote! { () },
            Literal::Null => quote! { None },
            _ => Self::transpile_simple_literal(lit),
        }
    }
    fn transpile_simple_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Char(c) => quote! { #c },
            Literal::Unit => quote! { () },
            Literal::Null => quote! { None },
            _ => unreachable!(),
        }
    }
    fn transpile_integer(i: i64, type_suffix: Option<&str>) -> TokenStream {
        // DEFECT-002 FIX: Preserve type suffixes from source code
        if let Some(suffix) = type_suffix {
            // Emit integer with explicit type suffix (e.g., 5i32, 10u64)
            let tokens = format!("{i}{suffix}");
            tokens.parse().expect("Valid integer literal with suffix")
        } else if let Ok(i32_val) = i32::try_from(i) {
            // Use unsuffixed for cleaner output - Rust can infer the type
            let literal = proc_macro2::Literal::i32_unsuffixed(i32_val);
            quote! { #literal }
        } else {
            // For large integers, we need i64 suffix to avoid overflow
            let literal = proc_macro2::Literal::i64_suffixed(i);
            quote! { #literal }
        }
    }
    /// Transpiles string interpolation
    ///
    /// # Errors
    /// Returns an error if expression transpilation fails
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_string_interpolation;
    ///
    /// let result = transpile_string_interpolation(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_string_interpolation(&self, parts: &[StringPart]) -> Result<TokenStream> {
        if parts.is_empty() {
            return Ok(quote! { "" });
        }
        let mut format_string = String::new();
        let mut args = Vec::new();
        for part in parts {
            match part {
                StringPart::Text(s) => {
                    // Escape any format specifiers in literal parts
                    format_string.push_str(&s.replace('{', "{{").replace('}', "}}"));
                }
                StringPart::Expr(expr) => {
                    format_string.push_str("{}");
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
                StringPart::ExprWithFormat { expr, format_spec } => {
                    // Include the format specifier in the format string
                    format_string.push('{');
                    format_string.push_str(format_spec);
                    format_string.push('}');
                    let expr_tokens = self.transpile_expr(expr)?;
                    args.push(expr_tokens);
                }
            }
        }
        Ok(quote! {
            format!(#format_string #(, #args)*)
        })
    }
    /// Transpiles binary operations
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::Transpiler;
    /// use ruchy::frontend::ast::{Expr, BinaryOp};
    ///
    /// let transpiler = Transpiler::new();
    /// let left = Expr::literal(1.into());
    /// let right = Expr::literal(2.into());
    /// let result = transpiler.transpile_binary(&left, BinaryOp::Add, &right);
    /// assert!(result.is_ok());
    /// ```
    /// Transpiles assignment
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_assign;
    ///
    /// let result = transpile_assign(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_assign(&self, target: &Expr, value: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        Ok(quote! { #target_tokens = #value_tokens })
    }
    /// Transpiles compound assignment
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_compound_assign;
    ///
    /// let result = transpile_compound_assign(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_compound_assign(
        &self,
        target: &Expr,
        op: BinaryOp,
        value: &Expr,
    ) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        let op_tokens = Self::get_compound_op_token(op)?;
        Ok(quote! { #target_tokens #op_tokens #value_tokens })
    }
    fn get_compound_op_token(op: BinaryOp) -> Result<TokenStream> {
        use BinaryOp::{
            Add, BitwiseAnd, BitwiseOr, BitwiseXor, Divide, LeftShift, Modulo, Multiply,
            RightShift, Subtract,
        };
        match op {
            Add | Subtract | Multiply => Ok(Self::get_basic_compound_token(op)),
            Divide | Modulo => Ok(Self::get_division_compound_token(op)),
            BitwiseAnd | BitwiseOr | BitwiseXor | LeftShift | RightShift => {
                Ok(Self::get_bitwise_compound_token(op))
            }
            _ => {
                use anyhow::bail;
                bail!("Invalid operator for compound assignment: {:?}", op)
            }
        }
    }
    fn get_basic_compound_token(op: BinaryOp) -> TokenStream {
        match op {
            BinaryOp::Add => quote! { += },
            BinaryOp::Subtract => quote! { -= },
            BinaryOp::Multiply => quote! { *= },
            _ => unreachable!(),
        }
    }
    fn get_division_compound_token(op: BinaryOp) -> TokenStream {
        match op {
            BinaryOp::Divide => quote! { /= },
            BinaryOp::Modulo => quote! { %= },
            _ => unreachable!(),
        }
    }

    fn get_bitwise_compound_token(op: BinaryOp) -> TokenStream {
        match op {
            BinaryOp::BitwiseAnd => quote! { &= },
            BinaryOp::BitwiseOr => quote! { |= },
            BinaryOp::BitwiseXor => quote! { ^= },
            BinaryOp::LeftShift => quote! { <<= },
            BinaryOp::RightShift => quote! { >>= },
            _ => unreachable!(),
        }
    }
    /// Transpiles pre-increment
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_pre_increment;
    ///
    /// let result = transpile_pre_increment(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_pre_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! { { #target_tokens += 1; #target_tokens } })
    }
    /// Transpiles post-increment
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_post_increment;
    ///
    /// let result = transpile_post_increment(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_post_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            {
                let _tmp = #target_tokens;
                #target_tokens += 1;
                _tmp
            }
        })
    }
    /// Transpiles pre-decrement
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_pre_decrement;
    ///
    /// let result = transpile_pre_decrement(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_pre_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! { { #target_tokens -= 1; #target_tokens } })
    }
    /// Transpiles post-decrement
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_post_decrement;
    ///
    /// let result = transpile_post_decrement(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_post_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! {
            {
                let _tmp = #target_tokens;
                #target_tokens -= 1;
                _tmp
            }
        })
    }
    /// Transpiles array initialization syntax [value; size]
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_array_init;
    ///
    /// let result = transpile_array_init(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_array_init(&self, value: &Expr, size: &Expr) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        let size_tokens = self.transpile_expr(size)?;
        // Generate vec![value; size] for now
        // Future enhancement: generate actual arrays when size is known at compile time
        Ok(quote! { vec![#value_tokens; #size_tokens as usize] })
    }
    /// Transpiles list literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_list;
    ///
    /// let result = transpile_list(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
    /// Check if an expression is definitely a string (conservative detection)
    fn is_definitely_string(expr: &Expr) -> bool {
        match &expr.kind {
            // String literals are definitely strings
            ExprKind::Literal(Literal::String(_)) => true,
            // String interpolation is definitely strings
            ExprKind::StringInterpolation { .. } => true,
            // Binary expressions with + that involve strings are string concatenations
            ExprKind::Binary {
                op: BinaryOp::Add,
                left,
                right,
            } => Self::is_definitely_string(left) || Self::is_definitely_string(right),
            // Method calls on strings that return strings
            ExprKind::MethodCall {
                receiver, method, ..
            } => {
                matches!(
                    method.as_str(),
                    "to_string" | "trim" | "to_uppercase" | "to_lowercase"
                ) || Self::is_definitely_string(receiver)
            }
            // Variables could be strings, but we can't be sure without type info
            // For now, be conservative and don't assume variables are strings
            ExprKind::Identifier(_) => false,
            // Function calls are NOT definitely strings - they could return any type
            ExprKind::Call { .. } => false,
            // Other expressions are not strings
            _ => false,
        }
    }
    /// Transpile string concatenation using proper Rust string operations
    fn transpile_string_concatenation(&self, left: &Expr, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;
        // Use format! with proper string handling - convert both to strings to avoid type mismatches
        // This avoids the String + String issue in Rust by using format! exclusively
        Ok(quote! { format!("{}{}", #left_tokens, #right_tokens) })
    }

    /// Helper to detect if an expression looks like it belongs in a real set literal
    /// vs. a misparsed function body
    pub(super) fn looks_like_real_set(&self, expr: &Expr) -> bool {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            // These expressions are likely to be in real sets
            ExprKind::Literal(_) => true,    // {1, 2, 3}
            ExprKind::Identifier(_) => true, // {x, y, z}
            ExprKind::Call { .. } => true,   // {func(), other()}

            // These expressions are unlikely to be in real sets (more likely function bodies)
            ExprKind::Binary { .. } => false, // {a + b} is probably a function body
            ExprKind::Let { .. } => false,    // {let x = 1; x} is definitely a function body
            ExprKind::If { .. } => false, // {if cond { x } else { y }} is probably a function body
            ExprKind::Block { .. } => false, // {{...}} is probably a function body
            ExprKind::Return { .. } => false, // {return x} is definitely a function body

            // For other expressions, assume they might be real sets
            _ => true,
        }
    }
}
