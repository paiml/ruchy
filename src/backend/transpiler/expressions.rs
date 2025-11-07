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
    /// let mut transpiler = Transpiler::new();
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
        // DEADLOCK FIX (Issue #132): Check if assigning to a global that's also in value
        // If so, use single-lock pattern to avoid deadlock
        if let ExprKind::Identifier(target_name) = &target.kind {
            if self.global_vars.read().unwrap().contains(target_name) {
                // Target is a global - check if value also references it
                if Self::expr_references_var(value, target_name) {
                    // DEADLOCK SCENARIO: counter = counter + 1
                    // Generate single-lock pattern:
                    //   { let mut guard = counter.lock().unwrap(); *guard = *guard + 1; }
                    return self.transpile_assign_global_self_ref(target_name, value);
                }
            }
        }

        // Standard assignment (no deadlock risk)
        let value_tokens = self.transpile_expr(value)?;

        // BUG-003: Handle IndexAccess specially for lvalue (no .clone())
        if let ExprKind::IndexAccess { .. } = &target.kind {
            let target_tokens = self.transpile_index_lvalue(target)?;
            Ok(quote! { #target_tokens = #value_tokens })
        } else {
            let target_tokens = self.transpile_expr(target)?;
            Ok(quote! { #target_tokens = #value_tokens })
        }
    }

    /// Check if an expression references a specific variable name
    fn expr_references_var(expr: &Expr, var_name: &str) -> bool {
        match &expr.kind {
            ExprKind::Identifier(name) => name == var_name,
            ExprKind::Binary { left, right, .. } => {
                Self::expr_references_var(left, var_name) || Self::expr_references_var(right, var_name)
            }
            ExprKind::Unary { operand, .. } => Self::expr_references_var(operand, var_name),
            ExprKind::Call { func, args } => {
                Self::expr_references_var(func, var_name)
                    || args.iter().any(|arg| Self::expr_references_var(arg, var_name))
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                Self::expr_references_var(receiver, var_name)
                    || args.iter().any(|arg| Self::expr_references_var(arg, var_name))
            }
            ExprKind::IndexAccess { object, index } => {
                Self::expr_references_var(object, var_name) || Self::expr_references_var(index, var_name)
            }
            _ => false,
        }
    }

    /// Transpile assignment to global that references itself (single-lock pattern)
    /// Prevents deadlock: counter = counter + 1
    fn transpile_assign_global_self_ref(&self, var_name: &str, value: &Expr) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var_name);

        // Transpile value, but temporarily disable global wrapping
        // We'll manually wrap with guard access
        let value_tokens = self.transpile_expr_for_guard(value, var_name)?;

        Ok(quote! {
            {
                let mut __guard = #var_ident.lock().unwrap();
                *__guard = #value_tokens;
            }
        })
    }

    /// Transpile expression replacing global var access with guard deref
    fn transpile_expr_for_guard(&self, expr: &Expr, var_name: &str) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Identifier(name) if name == var_name => {
                // Replace global access with guard deref
                Ok(quote! { *__guard })
            }
            ExprKind::Binary { left, op, right } => {
                // TRANSPILER-TYPE FIX: Handle vec + array concatenation in guard context
                // Pattern: *__guard + [item] → [(*__guard).as_slice(), &[item]].concat()
                if *op == BinaryOp::Add && matches!(right.kind, ExprKind::List(_)) {
                    let left_tokens = self.transpile_expr_for_guard(left, var_name)?;
                    let right_tokens = self.transpile_expr_for_guard(right, var_name)?;
                    // Wrap left in parens if needed for method call precedence
                    let left_wrapped = if matches!(left.kind, ExprKind::Binary { .. }) {
                        quote! { (#left_tokens) }
                    } else {
                        left_tokens
                    };
                    return Ok(quote! { [#left_wrapped.as_slice(), &#right_tokens].concat() });
                }

                let left_tokens = self.transpile_expr_for_guard(left, var_name)?;
                let right_tokens = self.transpile_expr_for_guard(right, var_name)?;

                // Wrap nested Binary expressions in parentheses to preserve precedence
                let left_wrapped = if matches!(left.kind, ExprKind::Binary { .. }) {
                    quote! { ( #left_tokens ) }
                } else {
                    left_tokens
                };
                let right_wrapped = if matches!(right.kind, ExprKind::Binary { .. }) {
                    quote! { ( #right_tokens ) }
                } else {
                    right_tokens
                };

                // Inline binary operator generation
                let op_token = match op {
                    BinaryOp::Add => quote! { + },
                    BinaryOp::Subtract => quote! { - },
                    BinaryOp::Multiply => quote! { * },
                    BinaryOp::Divide => quote! { / },
                    BinaryOp::Modulo => quote! { % },
                    BinaryOp::Power => quote! { .pow },
                    BinaryOp::Equal => quote! { == },
                    BinaryOp::NotEqual => quote! { != },
                    BinaryOp::Less => quote! { < },
                    BinaryOp::LessEqual => quote! { <= },
                    BinaryOp::Greater => quote! { > },
                    BinaryOp::GreaterEqual => quote! { >= },
                    BinaryOp::And => quote! { && },
                    BinaryOp::Or => quote! { || },
                    BinaryOp::BitwiseAnd => quote! { & },
                    BinaryOp::BitwiseOr => quote! { | },
                    BinaryOp::BitwiseXor => quote! { ^ },
                    BinaryOp::LeftShift => quote! { << },
                    BinaryOp::RightShift => quote! { >> },
                    _ => quote! { /* unsupported op */ },
                };
                Ok(quote! { #left_wrapped #op_token #right_wrapped })
            }
            ExprKind::Unary { op, operand } => {
                let operand_tokens = self.transpile_expr_for_guard(operand, var_name)?;
                let op_token = match op {
                    UnaryOp::Not | UnaryOp::BitwiseNot => quote! { ! },
                    UnaryOp::Negate => quote! { - },
                    UnaryOp::Reference => quote! { & },
                    UnaryOp::MutableReference => quote! { &mut },
                    UnaryOp::Deref => quote! { * },
                };
                Ok(quote! { #op_token #operand_tokens })
            }
            _ => {
                // For other expressions, use standard transpilation
                self.transpile_expr(expr)
            }
        }
    }

    /// Transpile `IndexAccess` as an lvalue (no .`clone()`)
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
        // DEADLOCK FIX: Compound assignments ALWAYS reference target on both sides
        // Example: total += x is really total = total + x
        // So if target is a global, we need single-lock pattern
        if let ExprKind::Identifier(target_name) = &target.kind {
            if self.global_vars.read().unwrap().contains(target_name) {
                let var_ident = format_ident!("{}", target_name);
                let value_tokens = self.transpile_expr(value)?;
                let op_tokens = Self::get_compound_op_token(op)?;

                return Ok(quote! {
                    {
                        let mut __guard = #var_ident.lock().unwrap();
                        *__guard #op_tokens #value_tokens
                    }
                });
            }
        }

        // Standard compound assignment (non-global)
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
            // TRANSPILER-001 FIX: Field access is NOT definitely a string
            // We cannot determine field types without full type inference
            // Conservative approach: assume numeric unless proven otherwise
            // This allows `self.value + amount` (i32 + i32) to use arithmetic operators
            // If both operands are actually Strings, Rust compiler will error on `+` operator
            // (which would require `.to_string()` or format!() explicitly in source code)
            ExprKind::FieldAccess { .. } => false,
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
