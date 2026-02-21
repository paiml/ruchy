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
            Literal::Atom(s) => {
                // Atoms transpile to &'static str
                quote! { #s }
            }
            _ => unreachable!("Unexpected literal in transpile_simple_literal"),
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
            if self
                .global_vars
                .read()
                .expect("rwlock should not be poisoned")
                .contains(target_name)
            {
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
                Self::expr_references_var(left, var_name)
                    || Self::expr_references_var(right, var_name)
            }
            ExprKind::Unary { operand, .. } => Self::expr_references_var(operand, var_name),
            ExprKind::Call { func, args } => {
                Self::expr_references_var(func, var_name)
                    || args
                        .iter()
                        .any(|arg| Self::expr_references_var(arg, var_name))
            }
            ExprKind::MethodCall { receiver, args, .. } => {
                Self::expr_references_var(receiver, var_name)
                    || args
                        .iter()
                        .any(|arg| Self::expr_references_var(arg, var_name))
            }
            ExprKind::IndexAccess { object, index } => {
                Self::expr_references_var(object, var_name)
                    || Self::expr_references_var(index, var_name)
            }
            _ => false,
        }
    }

    /// Transpile assignment to global that references itself (single-lock pattern)
    /// Prevents deadlock: counter = counter + 1
    fn transpile_assign_global_self_ref(
        &self,
        var_name: &str,
        value: &Expr,
    ) -> Result<TokenStream> {
        let var_ident = format_ident!("{}", var_name);

        // Transpile value, but temporarily disable global wrapping
        // We'll manually wrap with guard access
        let value_tokens = self.transpile_expr_for_guard(value, var_name)?;

        Ok(quote! {
            {
                let mut __guard = #var_ident.lock().expect("mutex should not be poisoned");
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
            if self
                .global_vars
                .read()
                .expect("rwlock should not be poisoned")
                .contains(target_name)
            {
                let var_ident = format_ident!("{}", target_name);
                let value_tokens = self.transpile_expr(value)?;
                let op_tokens = Self::get_compound_op_token(op)?;

                return Ok(quote! {
                    {
                        let mut __guard = #var_ident.lock().expect("mutex should not be poisoned");
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

    /// Helper: Generate increment/decrement operation
    /// Pre operations return new value, post operations return old value
    fn generate_inc_dec_op(
        target_tokens: TokenStream,
        is_increment: bool,
        is_pre: bool,
    ) -> TokenStream {
        let op = if is_increment {
            quote! { += 1 }
        } else {
            quote! { -= 1 }
        };

        if is_pre {
            quote! { { #target_tokens #op; #target_tokens } }
        } else {
            quote! {
                {
                    let _tmp = #target_tokens;
                    #target_tokens #op;
                    _tmp
                }
            }
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
        Ok(Self::generate_inc_dec_op(target_tokens, true, true))
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
        Ok(Self::generate_inc_dec_op(target_tokens, true, false))
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
        Ok(Self::generate_inc_dec_op(target_tokens, false, true))
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
        Ok(Self::generate_inc_dec_op(target_tokens, false, false))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    // Test 1: transpile_literal with Integer (no suffix)
    #[test]
    fn test_transpile_literal_integer_no_suffix() {
        let lit = Literal::Integer(42, None);
        let result = Transpiler::transpile_literal(&lit);
        assert!(!result.is_empty());
    }

    // Test 2: transpile_literal with Integer (i64 suffix)
    #[test]
    fn test_transpile_literal_integer_with_suffix() {
        let lit = Literal::Integer(100, Some("i64".to_string()));
        let result = Transpiler::transpile_literal(&lit);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("100i64"));
    }

    // Test 3: transpile_literal with Float
    #[test]
    fn test_transpile_literal_float() {
        let lit = Literal::Float(std::f64::consts::PI);
        let result = Transpiler::transpile_literal(&lit);
        assert!(!result.is_empty());
    }

    // Test 4: transpile_literal with Unit
    #[test]
    fn test_transpile_literal_unit() {
        let lit = Literal::Unit;
        let result = Transpiler::transpile_literal(&lit);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains('('));
        assert!(tokens_str.contains(')'));
    }

    // Test 5: transpile_literal with Null
    #[test]
    fn test_transpile_literal_null() {
        let lit = Literal::Null;
        let result = Transpiler::transpile_literal(&lit);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("None"));
    }

    // Test 6: transpile_simple_literal with String
    #[test]
    fn test_transpile_simple_literal_string() {
        let lit = Literal::String("hello".to_string());
        let result = Transpiler::transpile_simple_literal(&lit);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("hello"));
    }

    // Test 7: transpile_simple_literal with Bool
    #[test]
    fn test_transpile_simple_literal_bool() {
        let lit = Literal::Bool(true);
        let result = Transpiler::transpile_simple_literal(&lit);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("true"));
    }

    // Test 8: transpile_simple_literal with Char
    #[test]
    fn test_transpile_simple_literal_char() {
        let lit = Literal::Char('x');
        let result = Transpiler::transpile_simple_literal(&lit);
        assert!(!result.is_empty());
    }

    // Test 9: transpile_integer with type suffix
    #[test]
    fn test_transpile_integer_with_type_suffix() {
        let result = Transpiler::transpile_integer(42, Some("i32"));
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("42i32"));
    }

    // Test 10: transpile_integer i32 range (no suffix)
    #[test]
    fn test_transpile_integer_i32_range() {
        let result = Transpiler::transpile_integer(1000, None);
        assert!(!result.is_empty());
    }

    // Test 11: transpile_integer large i64 (requires suffix)
    #[test]
    fn test_transpile_integer_i64_large() {
        let large_val = i64::MAX;
        let result = Transpiler::transpile_integer(large_val, None);
        let tokens_str = result.to_string();
        // Should have i64 suffix for large values
        assert!(tokens_str.contains("i64"));
    }

    // Test 12: transpile_string_interpolation empty parts
    #[test]
    fn test_transpile_string_interpolation_empty() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_string_interpolation(&[]);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains("\"\""));
    }

    // Test 13: transpile_string_interpolation text only
    #[test]
    fn test_transpile_string_interpolation_text_only() {
        let transpiler = Transpiler::new();
        let parts = vec![StringPart::Text("hello".to_string())];
        let result = transpiler.transpile_string_interpolation(&parts);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        // Should contain either "hello" directly or in format! call
        assert!(tokens_str.contains("hello"));
    }

    // Test 14: expr_references_var with Identifier
    #[test]
    fn test_expr_references_var_identifier() {
        let expr = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 15: expr_references_var with different identifier
    #[test]
    fn test_expr_references_var_different_identifier() {
        let expr = Expr::new(ExprKind::Identifier("other".to_string()), Span::default());
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(!result);
    }

    // Test 16: expr_references_var with Binary expression
    #[test]
    fn test_expr_references_var_binary_expr() {
        let left = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 17: get_compound_op_token with Add
    #[test]
    fn test_get_compound_op_token_add() {
        let result = Transpiler::get_compound_op_token(BinaryOp::Add);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains("+="));
    }

    // Test 18: get_compound_op_token with invalid operator (ERROR PATH)
    #[test]
    fn test_get_compound_op_token_invalid_error_path() {
        let result = Transpiler::get_compound_op_token(BinaryOp::Equal);
        assert!(result.is_err());
    }

    // Test 19: get_basic_compound_token
    #[test]
    fn test_get_basic_compound_token() {
        let result = Transpiler::get_basic_compound_token(BinaryOp::Multiply);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("*="));
    }

    // Test 20: get_division_compound_token
    #[test]
    fn test_get_division_compound_token() {
        let result = Transpiler::get_division_compound_token(BinaryOp::Divide);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("/="));
    }

    // Test 21: get_bitwise_compound_token
    #[test]
    fn test_get_bitwise_compound_token() {
        let result = Transpiler::get_bitwise_compound_token(BinaryOp::BitwiseAnd);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("&="));
    }

    // Test 22: is_definitely_string with string literal
    #[test]
    fn test_is_definitely_string_literal() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("test".to_string())),
            Span::default(),
        );
        let result = transpiler.is_definitely_string(&expr);
        assert!(result);
    }

    // Test 23: is_definitely_string with integer literal
    #[test]
    fn test_is_definitely_string_integer_not_string() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.is_definitely_string(&expr);
        assert!(!result);
    }

    // Test 24: is_definitely_string with FieldAccess (not string)
    #[test]
    fn test_is_definitely_string_field_access_conservative() {
        let transpiler = Transpiler::new();
        let obj = Expr::new(ExprKind::Identifier("self".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(obj),
                field: "value".to_string(),
            },
            Span::default(),
        );
        let result = transpiler.is_definitely_string(&expr);
        assert!(!result); // Conservative: field access is not definitely string
    }

    // Test 25: looks_like_real_set with Literal (true)
    #[test]
    fn test_looks_like_real_set_literal() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let result = transpiler.looks_like_real_set(&expr);
        assert!(result);
    }

    // Test 26: looks_like_real_set with Binary (false - function body)
    #[test]
    fn test_looks_like_real_set_binary_false() {
        let transpiler = Transpiler::new();
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.looks_like_real_set(&expr);
        assert!(!result); // Binary expressions unlikely in real sets
    }

    // Test 27: looks_like_real_set with Let (false - definitely function body)
    #[test]
    fn test_looks_like_real_set_let_false() {
        let transpiler = Transpiler::new();
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let body = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            Span::default(),
        );
        let result = transpiler.looks_like_real_set(&expr);
        assert!(!result); // Let expressions are definitely function bodies
    }

    // Test 28: transpile_string_interpolation with expression
    #[test]
    fn test_transpile_string_interpolation_with_expr() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("name".to_string()), Span::default());
        let parts = vec![
            StringPart::Text("Hello ".to_string()),
            StringPart::Expr(Box::new(expr)),
        ];
        let result = transpiler.transpile_string_interpolation(&parts);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains("format"));
        assert!(tokens_str.contains("Hello"));
    }

    // Test 29: transpile_string_interpolation with format spec
    #[test]
    fn test_transpile_string_interpolation_with_format_spec() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("value".to_string()), Span::default());
        let parts = vec![StringPart::ExprWithFormat {
            expr: Box::new(expr),
            format_spec: ":>10".to_string(),
        }];
        let result = transpiler.transpile_string_interpolation(&parts);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains(":>10"));
    }

    // Test 30: expr_references_var with Unary expression
    #[test]
    fn test_expr_references_var_unary() {
        let operand = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 31: expr_references_var with Call expression
    #[test]
    fn test_expr_references_var_call() {
        let func = Expr::new(ExprKind::Identifier("compute".to_string()), Span::default());
        let arg = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![arg],
            },
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 32: expr_references_var with MethodCall
    #[test]
    fn test_expr_references_var_method_call() {
        let receiver = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "to_string".to_string(),
                args: vec![],
            },
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 33: expr_references_var with IndexAccess
    #[test]
    fn test_expr_references_var_index_access() {
        let object = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            },
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 34: get_compound_op_token with Subtract
    #[test]
    fn test_get_compound_op_token_subtract() {
        let result = Transpiler::get_compound_op_token(BinaryOp::Subtract);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains("-="));
    }

    // Test 35: get_compound_op_token with Modulo
    #[test]
    fn test_get_compound_op_token_modulo() {
        let result = Transpiler::get_compound_op_token(BinaryOp::Modulo);
        assert!(result.is_ok());
        let tokens = result.expect("operation should succeed in test");
        let tokens_str = tokens.to_string();
        assert!(tokens_str.contains("%="));
    }

    // Test 36: get_bitwise_compound_token with BitwiseOr
    #[test]
    fn test_get_bitwise_compound_token_or() {
        let result = Transpiler::get_bitwise_compound_token(BinaryOp::BitwiseOr);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("|="));
    }

    // Test 37: get_bitwise_compound_token with BitwiseXor
    #[test]
    fn test_get_bitwise_compound_token_xor() {
        let result = Transpiler::get_bitwise_compound_token(BinaryOp::BitwiseXor);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("^="));
    }

    // ===== EXTREME TDD Round 118 - Additional Tests =====

    // Test 38: expr_references_var negative case
    #[test]
    fn test_expr_references_var_not_found() {
        let expr = Expr::new(ExprKind::Identifier("other".to_string()), Span::default());
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(!result);
    }

    // Test 42: get_compound_op_token with Multiply
    #[test]
    fn test_get_compound_op_token_multiply() {
        let result = Transpiler::get_compound_op_token(BinaryOp::Multiply);
        assert!(result.is_ok());
        let tokens_str = result.expect("operation should succeed").to_string();
        assert!(tokens_str.contains("*="));
    }

    // Test 43: get_compound_op_token with Divide
    #[test]
    fn test_get_compound_op_token_divide() {
        let result = Transpiler::get_compound_op_token(BinaryOp::Divide);
        assert!(result.is_ok());
        let tokens_str = result.expect("operation should succeed").to_string();
        assert!(tokens_str.contains("/="));
    }

    // Test 44: get_bitwise_compound_token with BitwiseAnd
    #[test]
    fn test_get_bitwise_compound_token_and() {
        let result = Transpiler::get_bitwise_compound_token(BinaryOp::BitwiseAnd);
        let tokens_str = result.to_string();
        assert!(tokens_str.contains("&="));
    }

    // Test 45: expr_references_var with literal (negative)
    #[test]
    fn test_expr_references_var_literal() {
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(!result);
    }

    // Test 46: expr_references_var with Binary
    #[test]
    fn test_expr_references_var_binary() {
        let left = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = Transpiler::expr_references_var(&expr, "counter");
        assert!(result);
    }

    // Test 47: transpile_string_interpolation with Text part
    #[test]
    fn test_transpile_string_interpolation_text_part() {
        let transpiler = Transpiler::new();
        let parts = vec![StringPart::Text("Hello, World!".to_string())];
        let result = transpiler.transpile_string_interpolation(&parts);
        assert!(result.is_ok());
        let tokens_str = result.expect("operation should succeed").to_string();
        assert!(tokens_str.contains("Hello") || tokens_str.contains("format"));
    }

    // Test 48: transpile_string_interpolation with empty parts list
    #[test]
    fn test_transpile_string_interpolation_no_parts() {
        let transpiler = Transpiler::new();
        let parts: Vec<StringPart> = vec![];
        let result = transpiler.transpile_string_interpolation(&parts);
        assert!(result.is_ok());
    }

    // ===== EXTREME TDD Round 140 - Increment/Decrement Tests =====

    // Test 49: generate_inc_dec_op pre-increment
    #[test]
    fn test_generate_inc_dec_op_pre_increment() {
        let target = quote::quote! { x };
        let result = Transpiler::generate_inc_dec_op(target, true, true);
        let code = result.to_string();
        assert!(code.contains("+="));
        assert!(code.contains("1"));
    }

    // Test 50: generate_inc_dec_op post-increment
    #[test]
    fn test_generate_inc_dec_op_post_increment() {
        let target = quote::quote! { x };
        let result = Transpiler::generate_inc_dec_op(target, true, false);
        let code = result.to_string();
        assert!(code.contains("+="));
        assert!(code.contains("_tmp"));
    }

    // Test 51: generate_inc_dec_op pre-decrement
    #[test]
    fn test_generate_inc_dec_op_pre_decrement() {
        let target = quote::quote! { y };
        let result = Transpiler::generate_inc_dec_op(target, false, true);
        let code = result.to_string();
        assert!(code.contains("-="));
    }

    // Test 52: generate_inc_dec_op post-decrement
    #[test]
    fn test_generate_inc_dec_op_post_decrement() {
        let target = quote::quote! { y };
        let result = Transpiler::generate_inc_dec_op(target, false, false);
        let code = result.to_string();
        assert!(code.contains("-="));
        assert!(code.contains("_tmp"));
    }

    // Test 53: transpile_pre_increment
    #[test]
    fn test_transpile_pre_increment() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let result = transpiler.transpile_pre_increment(&expr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("+="));
    }

    // Test 54: transpile_post_increment
    #[test]
    fn test_transpile_post_increment() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("idx".to_string()), Span::default());
        let result = transpiler.transpile_post_increment(&expr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("_tmp"));
    }

    // Test 55: transpile_pre_decrement
    #[test]
    fn test_transpile_pre_decrement() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("count".to_string()), Span::default());
        let result = transpiler.transpile_pre_decrement(&expr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("-="));
    }

    // Test 56: transpile_post_decrement
    #[test]
    fn test_transpile_post_decrement() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("n".to_string()), Span::default());
        let result = transpiler.transpile_post_decrement(&expr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("_tmp"));
    }

    // Test 57: transpile_array_init
    #[test]
    fn test_transpile_array_init() {
        let transpiler = Transpiler::new();
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let size = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let result = transpiler.transpile_array_init(&value, &size);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        // Array init generates vec, array literal, or repeat syntax
        assert!(code.contains("vec") || code.contains('[') || code.contains("10"));
    }

    // Test 58: transpile_index_lvalue simple
    #[test]
    fn test_transpile_index_lvalue_simple() {
        let transpiler = Transpiler::new();
        let object = Expr::new(ExprKind::Identifier("arr".to_string()), Span::default());
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            },
            Span::default(),
        );
        let result = transpiler.transpile_index_lvalue(&expr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("arr"));
        assert!(code.contains("usize"));
    }

    // Test 59: transpile_index_lvalue nested
    #[test]
    fn test_transpile_index_lvalue_nested() {
        let transpiler = Transpiler::new();
        let matrix = Expr::new(ExprKind::Identifier("matrix".to_string()), Span::default());
        let i = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let j = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let inner = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(matrix),
                index: Box::new(i),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(inner),
                index: Box::new(j),
            },
            Span::default(),
        );
        let result = transpiler.transpile_index_lvalue(&expr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("matrix"));
    }

    // Test 60: transpile_index_lvalue non-index
    #[test]
    fn test_transpile_index_lvalue_non_index() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let result = transpiler.transpile_index_lvalue(&expr);
        assert!(result.is_ok());
    }

    // Test 61: is_definitely_string with StringInterpolation
    #[test]
    fn test_is_definitely_string_interpolation() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::StringInterpolation {
                parts: vec![StringPart::Text("hello".to_string())],
            },
            Span::default(),
        );
        assert!(transpiler.is_definitely_string(&expr));
    }

    // Test 62: is_definitely_string with Binary Add of strings
    #[test]
    fn test_is_definitely_string_binary_add() {
        let transpiler = Transpiler::new();
        let left = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::String(" world".to_string())),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Span::default(),
        );
        assert!(transpiler.is_definitely_string(&expr));
    }

    // Test 63: is_definitely_string with MethodCall to_string
    #[test]
    fn test_is_definitely_string_method_to_string() {
        let transpiler = Transpiler::new();
        let receiver = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "to_string".to_string(),
                args: vec![],
            },
            Span::default(),
        );
        assert!(transpiler.is_definitely_string(&expr));
    }

    // Test 64: is_definitely_string with Call (false)
    #[test]
    fn test_is_definitely_string_call_false() {
        let transpiler = Transpiler::new();
        let func = Expr::new(ExprKind::Identifier("compute".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![],
            },
            Span::default(),
        );
        assert!(!transpiler.is_definitely_string(&expr));
    }

    // Test 65: looks_like_real_set with Identifier
    #[test]
    fn test_looks_like_real_set_identifier() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        assert!(transpiler.looks_like_real_set(&expr));
    }

    // Test 66: looks_like_real_set with Call
    #[test]
    fn test_looks_like_real_set_call() {
        let transpiler = Transpiler::new();
        let func = Expr::new(ExprKind::Identifier("f".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![],
            },
            Span::default(),
        );
        assert!(transpiler.looks_like_real_set(&expr));
    }

    // Test 67: looks_like_real_set with If (false)
    #[test]
    fn test_looks_like_real_set_if_false() {
        let transpiler = Transpiler::new();
        let cond = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let then_b = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(cond),
                then_branch: Box::new(then_b),
                else_branch: None,
            },
            Span::default(),
        );
        assert!(!transpiler.looks_like_real_set(&expr));
    }

    // Test 68: looks_like_real_set with Block (false)
    #[test]
    fn test_looks_like_real_set_block_false() {
        let transpiler = Transpiler::new();
        let inner = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(ExprKind::Block(vec![inner]), Span::default());
        assert!(!transpiler.looks_like_real_set(&expr));
    }

    // Test 69: looks_like_real_set with Return (false)
    #[test]
    fn test_looks_like_real_set_return_false() {
        let transpiler = Transpiler::new();
        let val = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Return {
                value: Some(Box::new(val)),
            },
            Span::default(),
        );
        assert!(!transpiler.looks_like_real_set(&expr));
    }

    // Test 70: get_compound_op_token BitwiseAnd
    #[test]
    fn test_get_compound_op_token_bitwise_and() {
        let result = Transpiler::get_compound_op_token(BinaryOp::BitwiseAnd);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("&="));
    }

    // Test 71: get_compound_op_token BitwiseOr
    #[test]
    fn test_get_compound_op_token_bitwise_or() {
        let result = Transpiler::get_compound_op_token(BinaryOp::BitwiseOr);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("|="));
    }

    // Test 72: get_compound_op_token BitwiseXor
    #[test]
    fn test_get_compound_op_token_bitwise_xor() {
        let result = Transpiler::get_compound_op_token(BinaryOp::BitwiseXor);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("^="));
    }

    // Test 73: get_compound_op_token LeftShift
    #[test]
    fn test_get_compound_op_token_left_shift() {
        let result = Transpiler::get_compound_op_token(BinaryOp::LeftShift);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains("<<="));
    }

    // Test 74: get_compound_op_token RightShift
    #[test]
    fn test_get_compound_op_token_right_shift() {
        let result = Transpiler::get_compound_op_token(BinaryOp::RightShift);
        assert!(result.is_ok());
        let code = result.expect("should succeed").to_string();
        assert!(code.contains(">>="));
    }

    // Test 75: transpile_simple_literal with Unit
    #[test]
    fn test_transpile_simple_literal_unit() {
        let lit = Literal::Unit;
        let result = Transpiler::transpile_simple_literal(&lit);
        let code = result.to_string();
        assert!(code.contains('('));
    }

    // Test 76: transpile_simple_literal with Null
    #[test]
    fn test_transpile_simple_literal_null() {
        let lit = Literal::Null;
        let result = Transpiler::transpile_simple_literal(&lit);
        let code = result.to_string();
        assert!(code.contains("None"));
    }

    // Test 77: transpile_simple_literal with Atom
    #[test]
    fn test_transpile_simple_literal_atom() {
        let lit = Literal::Atom("hello".to_string());
        let result = Transpiler::transpile_simple_literal(&lit);
        let code = result.to_string();
        assert!(code.contains("hello"));
    }

    // Test 78: expr_references_var with MethodCall in args
    #[test]
    fn test_expr_references_var_method_call_args() {
        let receiver = Expr::new(ExprKind::Identifier("obj".to_string()), Span::default());
        let arg = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "process".to_string(),
                args: vec![arg],
            },
            Span::default(),
        );
        assert!(Transpiler::expr_references_var(&expr, "counter"));
    }

    // Test 79: expr_references_var with IndexAccess in index
    #[test]
    fn test_expr_references_var_index_access_index() {
        let object = Expr::new(ExprKind::Identifier("arr".to_string()), Span::default());
        let index = Expr::new(ExprKind::Identifier("counter".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            },
            Span::default(),
        );
        assert!(Transpiler::expr_references_var(&expr, "counter"));
    }

    // ========================================================================
    // transpile_expr_for_guard tests
    // ========================================================================

    #[test]
    fn test_guard_identifier_matching_var() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        assert!(
            result.to_string().contains("__guard"),
            "Should replace matching var with *__guard"
        );
    }

    #[test]
    fn test_guard_identifier_non_matching_var() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(ExprKind::Identifier("y".to_string()), Span::default());
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        // Non-matching identifier falls through to standard transpilation
        assert!(!result.to_string().contains("__guard"));
    }

    #[test]
    fn test_guard_binary_add() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"), "Left side should use guard deref");
        assert!(code.contains('+'), "Should contain add operator");
    }

    #[test]
    fn test_guard_binary_subtract() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Subtract,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"));
        assert!(code.contains('-'));
    }

    #[test]
    fn test_guard_binary_multiply() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(3, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Multiply,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        assert!(result.to_string().contains("__guard"));
    }

    #[test]
    fn test_guard_binary_divide() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(4, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Divide,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        assert!(result.to_string().contains("__guard"));
    }

    #[test]
    fn test_guard_binary_modulo() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Modulo,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"));
        assert!(code.contains('%'));
    }

    #[test]
    fn test_guard_binary_equal() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Equal,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"));
        assert!(code.contains("=="));
    }

    #[test]
    fn test_guard_binary_not_equal() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::NotEqual,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        assert!(result.to_string().contains("!="));
    }

    #[test]
    fn test_guard_binary_comparison_ops() {
        let transpiler = Transpiler::new();
        for (op, expected_str) in [
            (BinaryOp::Less, "<"),
            (BinaryOp::LessEqual, "<="),
            (BinaryOp::Greater, ">"),
            (BinaryOp::GreaterEqual, ">="),
        ] {
            let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
            let right = Expr::new(
                ExprKind::Literal(Literal::Integer(10, None)),
                Span::default(),
            );
            let expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                Span::default(),
            );
            let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
            let code = result.to_string();
            assert!(
                code.contains(expected_str),
                "Op {expected_str} not found in: {code}"
            );
        }
    }

    #[test]
    fn test_guard_binary_logical_ops() {
        let transpiler = Transpiler::new();
        for (op, expected_str) in [(BinaryOp::And, "&&"), (BinaryOp::Or, "||")] {
            let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
            let right = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
            let expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                Span::default(),
            );
            let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
            let code = result.to_string();
            assert!(
                code.contains(expected_str),
                "Op {expected_str} not found in: {code}"
            );
        }
    }

    #[test]
    fn test_guard_binary_bitwise_ops() {
        let transpiler = Transpiler::new();
        for (op, expected_str) in [
            (BinaryOp::BitwiseAnd, "&"),
            (BinaryOp::BitwiseOr, "|"),
            (BinaryOp::BitwiseXor, "^"),
            (BinaryOp::LeftShift, "<<"),
            (BinaryOp::RightShift, ">>"),
        ] {
            let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
            let right = Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::default(),
            );
            let expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                Span::default(),
            );
            let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
            let code = result.to_string();
            assert!(
                code.contains(expected_str),
                "Op {expected_str} not found in: {code}"
            );
        }
    }

    #[test]
    fn test_guard_binary_power() {
        let transpiler = Transpiler::new();
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Power,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("pow"), "Power op should use .pow");
    }

    #[test]
    fn test_guard_binary_nested_wraps_parens() {
        let transpiler = Transpiler::new();
        // (x + 1) * 2 - the left side is a Binary, so should get wrapped in parens
        let inner_left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let inner_right = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let left = Expr::new(
            ExprKind::Binary {
                left: Box::new(inner_left),
                op: BinaryOp::Add,
                right: Box::new(inner_right),
            },
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Multiply,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        // Left side is Binary, should be wrapped in parens
        assert!(code.contains("__guard"));
    }

    #[test]
    fn test_guard_vec_concat_pattern() {
        let transpiler = Transpiler::new();
        // x + [item] should use concat pattern
        let left = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let item = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(ExprKind::List(vec![item]), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("concat"), "Vec + List should use concat");
        assert!(code.contains("as_slice"), "Should call as_slice on left");
    }

    #[test]
    fn test_guard_unary_not() {
        let transpiler = Transpiler::new();
        let operand = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(
            code.contains("__guard"),
            "Should replace x with guard deref"
        );
        assert!(code.contains('!'), "Should contain NOT operator");
    }

    #[test]
    fn test_guard_unary_negate() {
        let transpiler = Transpiler::new();
        let operand = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"));
        assert!(code.contains('-'));
    }

    #[test]
    fn test_guard_unary_reference() {
        let transpiler = Transpiler::new();
        let operand = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Reference,
                operand: Box::new(operand),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"));
    }

    #[test]
    fn test_guard_unary_mut_reference() {
        let transpiler = Transpiler::new();
        let operand = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::MutableReference,
                operand: Box::new(operand),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("mut"));
    }

    #[test]
    fn test_guard_unary_deref() {
        let transpiler = Transpiler::new();
        let operand = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Deref,
                operand: Box::new(operand),
            },
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("__guard"));
    }

    #[test]
    fn test_guard_fallthrough_literal() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let result = transpiler.transpile_expr_for_guard(&expr, "x").unwrap();
        let code = result.to_string();
        assert!(code.contains("42"), "Literal should pass through unchanged");
    }
}
