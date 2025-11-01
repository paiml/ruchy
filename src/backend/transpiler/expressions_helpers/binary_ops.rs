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
}
