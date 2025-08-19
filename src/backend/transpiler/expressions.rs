//! Expression transpilation methods

#![allow(clippy::missing_errors_doc)]

use super::Transpiler;
use crate::frontend::ast::{BinaryOp, Expr, Literal, StringPart, UnaryOp};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    /// Transpiles literal values
    pub fn transpile_literal(lit: &Literal) -> TokenStream {
        match lit {
            Literal::Integer(i) => {
                if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                    quote! { #i i32 }
                } else {
                    quote! { #i i64 }
                }
            }
            Literal::Float(f) => quote! { #f },
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Char(c) => quote! { #c },
            Literal::Unit => quote! { () },
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
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;

        let result = match op {
            BinaryOp::Add => quote! { #left_tokens + #right_tokens },
            BinaryOp::Subtract => quote! { #left_tokens - #right_tokens },
            BinaryOp::Multiply => quote! { #left_tokens * #right_tokens },
            BinaryOp::Divide => quote! { #left_tokens / #right_tokens },
            BinaryOp::Modulo => quote! { #left_tokens % #right_tokens },
            BinaryOp::Power => quote! { #left_tokens.pow(#right_tokens) },
            BinaryOp::Equal => quote! { #left_tokens == #right_tokens },
            BinaryOp::NotEqual => quote! { #left_tokens != #right_tokens },
            BinaryOp::Less => quote! { #left_tokens < #right_tokens },
            BinaryOp::LessEqual => quote! { #left_tokens <= #right_tokens },
            BinaryOp::Greater => quote! { #left_tokens > #right_tokens },
            BinaryOp::GreaterEqual => quote! { #left_tokens >= #right_tokens },
            BinaryOp::And => quote! { #left_tokens && #right_tokens },
            BinaryOp::Or => quote! { #left_tokens || #right_tokens },
            BinaryOp::BitwiseAnd => quote! { #left_tokens & #right_tokens },
            BinaryOp::BitwiseOr => quote! { #left_tokens | #right_tokens },
            BinaryOp::BitwiseXor => quote! { #left_tokens ^ #right_tokens },
            BinaryOp::LeftShift => quote! { #left_tokens << #right_tokens },
            BinaryOp::RightShift => quote! { #left_tokens >> #right_tokens },
        };

        Ok(result)
    }

    /// Transpiles unary operations  
    pub fn transpile_unary(&self, op: UnaryOp, operand: &Expr) -> Result<TokenStream> {
        let operand_tokens = self.transpile_expr(operand)?;

        Ok(match op {
            UnaryOp::Not | UnaryOp::BitwiseNot => quote! { !#operand_tokens },
            UnaryOp::Negate => quote! { -#operand_tokens },
        })
    }

    /// Transpiles try operator (?)
    pub fn transpile_try(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { #expr_tokens? })
    }

    /// Transpiles await expressions
    pub fn transpile_await(&self, expr: &Expr) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! { #expr_tokens.await })
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

        let op_tokens = match op {
            BinaryOp::Add => quote! { += },
            BinaryOp::Subtract => quote! { -= },
            BinaryOp::Multiply => quote! { *= },
            BinaryOp::Divide => quote! { /= },
            BinaryOp::Modulo => quote! { %= },
            _ => {
                use anyhow::bail;
                bail!("Invalid operator for compound assignment: {:?}", op)
            }
        };

        Ok(quote! { #target_tokens #op_tokens #value_tokens })
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
        use crate::frontend::ast::ObjectField;

        let mut field_tokens = Vec::new();

        for field in fields {
            match field {
                ObjectField::KeyValue { key, value } => {
                    let key_ident = format_ident!("{}", key);
                    let value_tokens = self.transpile_expr(value)?;
                    field_tokens.push(quote! { #key_ident: #value_tokens });
                }
                ObjectField::Spread { expr } => {
                    let expr_tokens = self.transpile_expr(expr)?;
                    field_tokens.push(quote! { ..#expr_tokens });
                }
            }
        }

        // Generate a struct literal
        // For now, we'll use an anonymous struct pattern
        // In a real implementation, we might want to infer or specify the type
        Ok(quote! {
            {
                #(#field_tokens,)*
            }
        })
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
            let value_tokens = self.transpile_expr(value)?;
            field_tokens.push(quote! { #field_ident: #value_tokens });
        }

        Ok(quote! {
            #struct_name {
                #(#field_tokens,)*
            }
        })
    }
}
