//! Expression transpilation methods
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_pass_by_value)] // TokenStream by value is intentional for quote! macro
use super::Transpiler;
use crate::frontend::ast::{
    BinaryOp::{self},
    Expr, ExprKind, Literal, StringPart,
};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

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
        let value_tokens = self.transpile_expr(value)?;

        // BUG-003: Handle IndexAccess specially for lvalue (no .clone())
        match &target.kind {
            ExprKind::IndexAccess { .. } => {
                let target_tokens = self.transpile_index_lvalue(target)?;
                Ok(quote! { #target_tokens = #value_tokens })
            }
            _ => {
                let target_tokens = self.transpile_expr(target)?;
                Ok(quote! { #target_tokens = #value_tokens })
            }
        }
    }

    /// Transpile IndexAccess as an lvalue (no .clone())
    /// Handles nested cases like matrix[i][j]
    fn transpile_index_lvalue(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::IndexAccess { object, index } => {
                // Recursively handle nested IndexAccess
                let obj_tokens = if matches!(object.kind, ExprKind::IndexAccess { .. }) {
                    self.transpile_index_lvalue(object)?
                } else {
                    self.transpile_expr(object)?
                };
                let idx_tokens = self.transpile_expr(index)?;
                Ok(quote! { #obj_tokens[#idx_tokens as usize] })
            }
            _ => self.transpile_expr(expr),
        }
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
                bail!("Invalid operator for compound assignment: {op:?}")
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

    /// Check if an expression is definitely a string (less conservative detection for DEFECT-016)
    fn is_definitely_string(&self, expr: &Expr) -> bool {
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
            } => self.is_definitely_string(left) || self.is_definitely_string(right),
            // Method calls on strings that return strings
            ExprKind::MethodCall {
                receiver, method, ..
            } => {
                matches!(
                    method.as_str(),
                    "to_string" | "trim" | "to_uppercase" | "to_lowercase"
                ) || self.is_definitely_string(receiver)
            }
            // DEFECT-016 FIX: Identifiers that are tracked string variables ARE strings
            ExprKind::Identifier(name) => {
                // Check if this identifier is a known string variable (not just any mutable var)
                self.string_vars.borrow().contains(name.as_str())
            }
            // DEFECT-016 FIX: Field access MIGHT be a String field - be less conservative
            // Assume field access could be String (let Rust type system catch errors if not)
            ExprKind::FieldAccess { .. } => true,
            // Function calls are NOT definitely strings - they could return any type
            ExprKind::Call { .. } => false,
            // Other expressions are not strings
            _ => false,
        }
    }
    /// Transpile string concatenation using proper Rust string operations
    fn transpile_string_concatenation(&self, left: &Expr, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;

        // DEFECT-016 FIX: Auto-borrow String types on right side
        // When right operand is a String type (FieldAccess, Identifier, Call), wrap with & to borrow
        let right_tokens = self.transpile_expr(right)?;
        let right_final = match &right.kind {
            // String fields need borrowing: rule.name → &rule.name
            ExprKind::FieldAccess { .. } => quote! { &#right_tokens },
            // String identifiers need borrowing: name → &name
            ExprKind::Identifier(_) => quote! { &#right_tokens },
            // Function calls returning String need borrowing: to_string() → &to_string()
            ExprKind::Call { .. } | ExprKind::MethodCall { .. } => quote! { &#right_tokens },
            // String literals and other expressions don't need borrowing
            _ => right_tokens,
        };

        // Use format! with proper string handling - convert both to strings to avoid type mismatches
        // This avoids the String + String issue in Rust by using format! exclusively
        Ok(quote! { format!("{}{}", #left_tokens, #right_final) })
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
