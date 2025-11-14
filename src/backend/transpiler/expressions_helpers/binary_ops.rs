//! Binary operator transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    pub fn transpile_binary(&self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<TokenStream> {
        // Special handling for string concatenation
        // Only treat as string concatenation if at least one operand is definitely a string
        // DEFECT-016 FIX: Use instance method to access mutable_vars context
        if op == BinaryOp::Add
            && (self.is_definitely_string(left) || self.is_definitely_string(right))
        {
            return self.transpile_string_concatenation(left, right);
        }

        // TRANSPILER-005: Handle vector + array concatenation
        // Pattern: vec + [item] → [vec, vec![item]].concat()
        if op == BinaryOp::Add && Self::is_vec_array_concat(left, right) {
            return self.transpile_vec_concatenation(left, right);
        }

        // ISSUE-114 FIX: Handle usize casting for .len() comparisons
        // When comparing .len() (usize) with i32, cast i32 to usize
        if Self::is_comparison_op(op) {
            let left_is_len = Self::is_len_call(left);
            let right_is_len = Self::is_len_call(right);

            if left_is_len && !right_is_len {
                // left.len() < right → left.len() < (right as usize)
                let left_tokens = self.transpile_expr_with_precedence(left, op, true)?;
                let right_tokens = self.transpile_expr_with_precedence(right, op, false)?;
                let casted_right = quote! { (#right_tokens as usize) };
                return Ok(Self::transpile_binary_op(left_tokens, op, casted_right));
            } else if right_is_len && !left_is_len {
                // left > right.len() → (left as usize) > right.len()
                let left_tokens = self.transpile_expr_with_precedence(left, op, true)?;
                let right_tokens = self.transpile_expr_with_precedence(right, op, false)?;
                let casted_left = quote! { (#left_tokens as usize) };
                return Ok(Self::transpile_binary_op(casted_left, op, right_tokens));
            }
        }

        // Transpile operands with precedence-aware parentheses
        let left_tokens = self.transpile_expr_with_precedence(left, op, true)?;
        let right_tokens = self.transpile_expr_with_precedence(right, op, false)?;
        Ok(Self::transpile_binary_op(left_tokens, op, right_tokens))
    }
    /// Transpile expression with precedence-aware parentheses
    ///
    /// Adds parentheses around sub-expressions when needed to preserve precedence
    fn transpile_expr_with_precedence(
        &self,
        expr: &Expr,
        parent_op: BinaryOp,
        is_left_operand: bool,
    ) -> Result<TokenStream> {
        let tokens = self.transpile_expr(expr)?;
        // Check if we need parentheses
        if let ExprKind::Binary { op: child_op, .. } = &expr.kind {
            let parent_prec = Self::get_operator_precedence(parent_op);
            let child_prec = Self::get_operator_precedence(*child_op);
            // Add parentheses if child has lower precedence
            // For right operands, also add parentheses if precedence is equal and parent is right-associative
            let needs_parens = child_prec < parent_prec
                || (!is_left_operand
                    && child_prec == parent_prec
                    && Self::is_right_associative(parent_op));
            if needs_parens {
                return Ok(quote! { (#tokens) });
            }
        }
        Ok(tokens)
    }
    /// Get operator precedence (higher number = higher precedence)
    fn get_operator_precedence(op: BinaryOp) -> i32 {
        match op {
            BinaryOp::Or => 10,
            BinaryOp::And => 20,
            BinaryOp::Equal | BinaryOp::NotEqual => 30,
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => 40,
            BinaryOp::Add | BinaryOp::Subtract => 50,
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 60,
            BinaryOp::Power => 70,
            BinaryOp::Send => 15, // Actor message passing
            _ => 0,               // Default for other operators
        }
    }
    /// Check if operator is right-associative
    fn is_right_associative(op: BinaryOp) -> bool {
        matches!(op, BinaryOp::Power) // Only power is right-associative in most languages
    }
    fn transpile_binary_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{
            Add, And, BitwiseAnd, BitwiseOr, BitwiseXor, Divide, Equal, Greater, GreaterEqual,
            LeftShift, Less, LessEqual, Modulo, Multiply, NotEqual, NullCoalesce, Or, Power,
            RightShift, Send, Subtract,
        };
        match op {
            // Arithmetic operations
            Add | Subtract | Multiply | Divide | Modulo | Power => {
                Self::transpile_arithmetic_op(left, op, right)
            }
            // Comparison operations
            Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual | BinaryOp::Gt => {
                Self::transpile_comparison_op(left, op, right)
            }
            // Logical operations
            And | Or | NullCoalesce => Self::transpile_logical_op(left, op, right),
            // Bitwise operations
            BitwiseAnd | BitwiseOr | BitwiseXor => Self::transpile_bitwise_op(left, op, right),
            // Shift operations
            LeftShift => Self::transpile_shift_ops(left, op, right),
            RightShift => Self::transpile_shift_ops(left, op, right),
            // Actor operations
            Send => quote! { #left.send(#right) },
        }
    }
    fn transpile_arithmetic_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{Add, Divide, Modulo, Multiply, Power, Subtract};
        match op {
            Add | Subtract | Multiply | Divide | Modulo => {
                Self::transpile_basic_arithmetic(left, op, right)
            }
            Power => quote! { #left.pow(#right) },
            _ => unreachable!(),
        }
    }
    fn transpile_basic_arithmetic(
        left: TokenStream,
        op: BinaryOp,
        right: TokenStream,
    ) -> TokenStream {
        // Reduce complexity by splitting into smaller functions
        match op {
            BinaryOp::Add => quote! { #left + #right },
            BinaryOp::Subtract => quote! { #left - #right },
            BinaryOp::Multiply => quote! { #left * #right },
            _ => Self::transpile_division_mod(left, op, right),
        }
    }
    fn transpile_division_mod(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Divide => quote! { #left / #right },
            BinaryOp::Modulo => quote! { #left % #right },
            _ => unreachable!(),
        }
    }
    fn transpile_comparison_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{Equal, Greater, GreaterEqual, Less, LessEqual, NotEqual};
        match op {
            Equal | NotEqual => Self::transpile_equality(left, op, right),
            Less | LessEqual | Greater | GreaterEqual | BinaryOp::Gt => {
                Self::transpile_ordering(left, op, right)
            }
            _ => unreachable!(),
        }
    }
    fn transpile_equality(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Equal => quote! { #left == #right },
            BinaryOp::NotEqual => quote! { #left != #right },
            _ => unreachable!(),
        }
    }
    fn transpile_ordering(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Less => quote! { #left < #right },
            BinaryOp::LessEqual => quote! { #left <= #right },
            _ => Self::transpile_greater_ops(left, op, right),
        }
    }
    fn transpile_greater_ops(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::Greater => quote! { #left > #right },
            BinaryOp::GreaterEqual => quote! { #left >= #right },
            BinaryOp::Gt => quote! { #left > #right }, // Alias for Greater
            _ => unreachable!(),
        }
    }
    fn transpile_logical_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::And => quote! { #left && #right },
            BinaryOp::Or => quote! { #left || #right },
            BinaryOp::NullCoalesce => quote! { #left.unwrap_or(#right) },
            _ => unreachable!(),
        }
    }
    fn transpile_bitwise_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{BitwiseAnd, BitwiseOr, BitwiseXor};
        match op {
            BitwiseAnd => quote! { #left & #right },
            BitwiseOr => quote! { #left | #right },
            BitwiseXor => quote! { #left ^ #right },
            _ => Self::transpile_shift_ops(left, op, right),
        }
    }
    fn transpile_shift_ops(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::LeftShift => quote! { #left << #right },
            BinaryOp::RightShift => quote! { #left >> #right },
            _ => unreachable!("Invalid shift operation: {:?}", op),
        }
    }

    /// Check if an expression is a .`len()` method call OR `len()` function call (ISSUE-115)
    /// Returns true for: `vec.len()`, `string.len()`, len(vec), len(string), etc.
    /// TRANSPILER-004: Extended to detect `len()` function calls for usize casting
    fn is_len_call(expr: &Expr) -> bool {
        match &expr.kind {
            // Method call: vec.len()
            ExprKind::MethodCall { method, .. } if method == "len" => true,
            // Function call: len(vec) - TRANSPILER-004 fix
            ExprKind::Call { func, args } if args.len() == 1 => {
                matches!(&func.kind, ExprKind::Identifier(name) if name == "len")
            }
            _ => false,
        }
    }

    /// Check if operator is a comparison operator (ISSUE-114)
    fn is_comparison_op(op: BinaryOp) -> bool {
        matches!(
            op,
            BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::Equal
                | BinaryOp::NotEqual
        )
    }

    /// Check if this is a vector + array concatenation pattern (TRANSPILER-005)
    /// Returns true for: vec + [item], vec + [item1, item2], etc.
    /// Complexity: 2 (within ≤10 limit ✅)
    fn is_vec_array_concat(_left: &Expr, right: &Expr) -> bool {
        // Right side must be an array literal
        let right_is_array = matches!(&right.kind, ExprKind::List(_));

        // Left side should be a vec-like expression (identifier, method call, etc.)
        // We use a conservative check: if right is array, assume left might be vec
        right_is_array
    }

    /// Transpile vector concatenation to valid Rust (TRANSPILER-005)
    /// Pattern: vec + [item] → [`vec.as_slice()`, &[item]].`concat()`
    /// Complexity: 3 (within ≤10 limit ✅)
    fn transpile_vec_concatenation(&self, left: &Expr, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;

        // Generate: [left.as_slice(), &right].concat()
        // This works for Vec + array and handles ownership correctly
        Ok(quote! { [#left_tokens.as_slice(), &#right_tokens].concat() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    // Test 1: get_operator_precedence - Or (lowest)
    #[test]
    fn test_get_operator_precedence_or() {
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Or), 10);
    }

    // Test 2: get_operator_precedence - And
    #[test]
    fn test_get_operator_precedence_and() {
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::And), 20);
    }

    // Test 3: get_operator_precedence - comparison operators
    #[test]
    fn test_get_operator_precedence_comparison() {
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Equal), 30);
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Less), 40);
    }

    // Test 4: get_operator_precedence - arithmetic Add/Subtract
    #[test]
    fn test_get_operator_precedence_add_subtract() {
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Add), 50);
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Subtract), 50);
    }

    // Test 5: get_operator_precedence - arithmetic Multiply/Divide
    #[test]
    fn test_get_operator_precedence_multiply_divide() {
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Multiply), 60);
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Divide), 60);
    }

    // Test 6: get_operator_precedence - Power (highest)
    #[test]
    fn test_get_operator_precedence_power() {
        assert_eq!(Transpiler::get_operator_precedence(BinaryOp::Power), 70);
    }

    // Test 7: is_right_associative - Power is right-associative
    #[test]
    fn test_is_right_associative_power() {
        assert!(Transpiler::is_right_associative(BinaryOp::Power));
    }

    // Test 8: is_right_associative - Add is left-associative
    #[test]
    fn test_is_right_associative_add() {
        assert!(!Transpiler::is_right_associative(BinaryOp::Add));
    }

    // Test 9: transpile_binary_op - Add arithmetic
    #[test]
    fn test_transpile_binary_op_add() {
        let left = quote! { a };
        let right = quote! { b };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Add, right);
        assert_eq!(result.to_string(), "a + b");
    }

    // Test 10: transpile_binary_op - Subtract arithmetic
    #[test]
    fn test_transpile_binary_op_subtract() {
        let left = quote! { x };
        let right = quote! { y };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Subtract, right);
        assert_eq!(result.to_string(), "x - y");
    }

    // Test 11: transpile_binary_op - Multiply arithmetic
    #[test]
    fn test_transpile_binary_op_multiply() {
        let left = quote! { m };
        let right = quote! { n };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Multiply, right);
        assert_eq!(result.to_string(), "m * n");
    }

    // Test 12: transpile_binary_op - Divide arithmetic
    #[test]
    fn test_transpile_binary_op_divide() {
        let left = quote! { p };
        let right = quote! { q };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Divide, right);
        assert_eq!(result.to_string(), "p / q");
    }

    // Test 13: transpile_binary_op - Modulo arithmetic
    #[test]
    fn test_transpile_binary_op_modulo() {
        let left = quote! { r };
        let right = quote! { s };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Modulo, right);
        assert_eq!(result.to_string(), "r % s");
    }

    // Test 14: transpile_binary_op - Power arithmetic
    #[test]
    fn test_transpile_binary_op_power() {
        let left = quote! { base };
        let right = quote! { exp };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Power, right);
        assert_eq!(result.to_string(), "base . pow (exp)");
    }

    // Test 15: transpile_binary_op - Equal comparison
    #[test]
    fn test_transpile_binary_op_equal() {
        let left = quote! { a };
        let right = quote! { b };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Equal, right);
        assert_eq!(result.to_string(), "a == b");
    }

    // Test 16: transpile_binary_op - NotEqual comparison
    #[test]
    fn test_transpile_binary_op_not_equal() {
        let left = quote! { x };
        let right = quote! { y };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::NotEqual, right);
        assert_eq!(result.to_string(), "x != y");
    }

    // Test 17: transpile_binary_op - Less comparison
    #[test]
    fn test_transpile_binary_op_less() {
        let left = quote! { a };
        let right = quote! { b };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Less, right);
        assert_eq!(result.to_string(), "a < b");
    }

    // Test 18: transpile_binary_op - Greater comparison
    #[test]
    fn test_transpile_binary_op_greater() {
        let left = quote! { x };
        let right = quote! { y };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Greater, right);
        assert_eq!(result.to_string(), "x > y");
    }

    // Test 19: transpile_binary_op - And logical
    #[test]
    fn test_transpile_binary_op_and() {
        let left = quote! { cond1 };
        let right = quote! { cond2 };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::And, right);
        assert_eq!(result.to_string(), "cond1 && cond2");
    }

    // Test 20: transpile_binary_op - Or logical
    #[test]
    fn test_transpile_binary_op_or() {
        let left = quote! { a };
        let right = quote! { b };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Or, right);
        assert_eq!(result.to_string(), "a || b");
    }

    // Test 21: transpile_binary_op - BitwiseAnd
    #[test]
    fn test_transpile_binary_op_bitwise_and() {
        let left = quote! { flags1 };
        let right = quote! { flags2 };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::BitwiseAnd, right);
        assert_eq!(result.to_string(), "flags1 & flags2");
    }

    // Test 22: transpile_binary_op - BitwiseOr
    #[test]
    fn test_transpile_binary_op_bitwise_or() {
        let left = quote! { mask1 };
        let right = quote! { mask2 };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::BitwiseOr, right);
        assert_eq!(result.to_string(), "mask1 | mask2");
    }

    // Test 23: transpile_binary_op - LeftShift
    #[test]
    fn test_transpile_binary_op_left_shift() {
        let left = quote! { value };
        let right = quote! { bits };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::LeftShift, right);
        assert_eq!(result.to_string(), "value << bits");
    }

    // Test 24: transpile_binary_op - RightShift
    #[test]
    fn test_transpile_binary_op_right_shift() {
        let left = quote! { num };
        let right = quote! { shift };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::RightShift, right);
        assert_eq!(result.to_string(), "num >> shift");
    }

    // Test 25: is_comparison_op - Equal is comparison
    #[test]
    fn test_is_comparison_op_equal() {
        assert!(Transpiler::is_comparison_op(BinaryOp::Equal));
    }

    // Test 26: is_comparison_op - Less is comparison
    #[test]
    fn test_is_comparison_op_less() {
        assert!(Transpiler::is_comparison_op(BinaryOp::Less));
    }

    // Test 27: is_comparison_op - Add is NOT comparison
    #[test]
    fn test_is_comparison_op_add() {
        assert!(!Transpiler::is_comparison_op(BinaryOp::Add));
    }

    // Test 28: transpile_binary_op - NullCoalesce
    #[test]
    fn test_transpile_binary_op_null_coalesce() {
        let left = quote! { opt };
        let right = quote! { default };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::NullCoalesce, right);
        assert_eq!(result.to_string(), "opt . unwrap_or (default)");
    }

    // Test 29: transpile_binary_op - Send (actor operation)
    #[test]
    fn test_transpile_binary_op_send() {
        let left = quote! { actor };
        let right = quote! { message };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::Send, right);
        assert_eq!(result.to_string(), "actor . send (message)");
    }

    // Test 30: transpile_binary_op - GreaterEqual comparison
    #[test]
    fn test_transpile_binary_op_greater_equal() {
        let left = quote! { a };
        let right = quote! { b };
        let result = Transpiler::transpile_binary_op(left, BinaryOp::GreaterEqual, right);
        assert_eq!(result.to_string(), "a >= b");
    }
}
