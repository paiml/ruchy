//! Expression transpilation methods

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::needless_pass_by_value)] // TokenStream by value is intentional for quote! macro

use super::Transpiler;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, StringPart, UnaryOp};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles literal values
    pub fn transpile_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::Integer(i) => Self::transpile_integer(*i),
            Literal::Float(f) => quote! { #f },
            Literal::Unit => quote! { () },
            _ => Self::transpile_simple_literal(lit),
        }
    }

    fn transpile_simple_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Char(c) => quote! { #c },
            _ => unreachable!(),
        }
    }

    fn transpile_integer(i: i64) -> TokenStream {
        // Integer literals in Rust need proper type handling
        // Use i32 for values that fit, i64 otherwise
        if let Ok(i32_val) = i32::try_from(i) {
            // Use i32 suffix for clarity and to match struct field types
            let literal = proc_macro2::Literal::i32_suffixed(i32_val);
            quote! { #literal }
        } else {
            // For large integers, we need i64 suffix
            let literal = proc_macro2::Literal::i64_suffixed(i);
            quote! { #literal }
        }
    }

    /// Transpiles string interpolation
    ///
    /// # Errors
    /// Returns an error if expression transpilation fails
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
            }
        }

        Ok(quote! {
            format!(#format_string #(, #args)*)
        })
    }

    /// Transpiles binary operations
    pub fn transpile_binary(&self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<TokenStream> {
        // Special handling for string concatenation
        // If at least one operand is definitely a string, treat as string concatenation
        if op == BinaryOp::Add && (Self::is_definitely_string(left) || Self::is_definitely_string(right)) {
            return self.transpile_string_concatenation(left, right);
        }

        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;

        Ok(Self::transpile_binary_op(left_tokens, op, right_tokens))
    }

    fn transpile_binary_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        use BinaryOp::{
            Add, And, BitwiseAnd, BitwiseOr, BitwiseXor, Divide, Equal, Greater, GreaterEqual,
            LeftShift, Less, LessEqual, Modulo, Multiply, NotEqual, Or, Power,
            Subtract,
        };
        match op {
            // Arithmetic operations
            Add | Subtract | Multiply | Divide | Modulo | Power => {
                Self::transpile_arithmetic_op(left, op, right)
            }
            // Comparison operations
            Equal | NotEqual | Less | LessEqual | Greater | GreaterEqual => {
                Self::transpile_comparison_op(left, op, right)
            }
            // Logical operations
            And | Or => Self::transpile_logical_op(left, op, right),
            // Bitwise operations
            BitwiseAnd | BitwiseOr | BitwiseXor | LeftShift => {
                Self::transpile_bitwise_op(left, op, right)
            }
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
            Less | LessEqual | Greater | GreaterEqual => Self::transpile_ordering(left, op, right),
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
            _ => unreachable!(),
        }
    }

    fn transpile_logical_op(left: TokenStream, op: BinaryOp, right: TokenStream) -> TokenStream {
        match op {
            BinaryOp::And => quote! { #left && #right },
            BinaryOp::Or => quote! { #left || #right },
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
            _ => unreachable!(),
        }
    }

    /// Transpiles unary operations  
    pub fn transpile_unary(&self, op: UnaryOp, operand: &Expr) -> Result<TokenStream> {
        let operand_tokens = self.transpile_expr(operand)?;

        Ok(match op {
            UnaryOp::Not | UnaryOp::BitwiseNot => quote! { !#operand_tokens },
            UnaryOp::Negate => quote! { -#operand_tokens },
            UnaryOp::Reference => quote! { &#operand_tokens },
        })
    }


    /// Transpiles await expressions
    pub fn transpile_await(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { #expr_tokens.await })
    }

    /// Transpiles async blocks
    pub fn transpile_async_block(&self, body: &Expr) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;
        Ok(quote! { async { #body_tokens } })
    }

    /// Transpiles throw expressions (panic in Rust)
    pub fn transpile_throw(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! {
            panic!(#expr_tokens)
        })
    }

    /// Transpiles field access
    pub fn transpile_field_access(&self, object: &Expr, field: &str) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        let field_ident = format_ident!("{}", field);
        Ok(quote! { #obj_tokens.#field_ident })
    }

    /// Transpiles index access (array[index])
    pub fn transpile_index_access(&self, object: &Expr, index: &Expr) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        let index_tokens = self.transpile_expr(index)?;
        Ok(quote! { #obj_tokens[#index_tokens] })
    }

    /// Transpiles slice access (array[start:end])
    pub fn transpile_slice(&self, object: &Expr, start: Option<&Expr>, end: Option<&Expr>) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        
        match (start, end) {
            (None, None) => {
                // Full slice [..]
                Ok(quote! { &#obj_tokens[..] })
            }
            (None, Some(end)) => {
                // Slice from beginning [..end]
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[..#end_tokens] })
            }
            (Some(start), None) => {
                // Slice to end [start..]
                let start_tokens = self.transpile_expr(start)?;
                Ok(quote! { &#obj_tokens[#start_tokens..] })
            }
            (Some(start), Some(end)) => {
                // Full range slice [start..end]
                let start_tokens = self.transpile_expr(start)?;
                let end_tokens = self.transpile_expr(end)?;
                Ok(quote! { &#obj_tokens[#start_tokens..#end_tokens] })
            }
        }
    }

    /// Transpiles assignment
    pub fn transpile_assign(&self, target: &Expr, value: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        let value_tokens = self.transpile_expr(value)?;
        Ok(quote! { #target_tokens = #value_tokens })
    }

    /// Transpiles compound assignment
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
        use BinaryOp::{Add, Divide, Modulo, Multiply, Subtract};
        match op {
            Add | Subtract | Multiply => Ok(Self::get_basic_compound_token(op)),
            Divide | Modulo => Ok(Self::get_division_compound_token(op)),
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

    /// Transpiles pre-increment
    pub fn transpile_pre_increment(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! { { #target_tokens += 1; #target_tokens } })
    }

    /// Transpiles post-increment
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
    pub fn transpile_pre_decrement(&self, target: &Expr) -> Result<TokenStream> {
        let target_tokens = self.transpile_expr(target)?;
        Ok(quote! { { #target_tokens -= 1; #target_tokens } })
    }

    /// Transpiles post-decrement
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

    /// Transpiles list literals
    pub fn transpile_list(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;
        Ok(quote! { vec![#(#element_tokens),*] })
    }

    /// Transpiles tuple literals
    pub fn transpile_tuple(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;
        Ok(quote! { (#(#element_tokens),*) })
    }

    /// Transpiles range expressions
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
    pub fn transpile_object_literal(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<TokenStream> {
        let field_tokens = self.collect_object_field_tokens(fields)?;
        Ok(quote! {
            {
                #(#field_tokens,)*
            }
        })
    }

    fn collect_object_field_tokens(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<Vec<TokenStream>> {
        use crate::frontend::ast::ObjectField;
        let mut field_tokens = Vec::new();

        for field in fields {
            let token = match field {
                ObjectField::KeyValue { key, value } => {
                    let key_ident = format_ident!("{}", key);
                    let value_tokens = self.transpile_expr(value)?;
                    quote! { #key_ident: #value_tokens }
                }
                ObjectField::Spread { expr } => {
                    let expr_tokens = self.transpile_expr(expr)?;
                    quote! { ..#expr_tokens }
                }
            };
            field_tokens.push(token);
        }
        Ok(field_tokens)
    }

    /// Transpiles struct literals
    pub fn transpile_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
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

        Ok(quote! {
            #struct_name {
                #(#field_tokens,)*
            }
        })
    }

    /// Check if an expression is definitely a string (no ambiguous identifiers)
    fn is_definitely_string(expr: &Expr) -> bool {
        matches!(&expr.kind, ExprKind::Literal(Literal::String(_)) | ExprKind::StringInterpolation { .. })
    }

    /// Transpile string concatenation using format! macro
    fn transpile_string_concatenation(&self, left: &Expr, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;
        
        // Use format! for string concatenation to avoid ownership issues
        Ok(quote! { format!("{}{}", #left_tokens, #right_tokens) })
    }
}
