use crate::frontend::ast::*;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn;

/// Transpiler from Ruchy AST to Rust code
pub struct Transpiler {
    /// Whether to include type annotations
    pub include_types: bool,
}

impl Default for Transpiler {
    fn default() -> Self {
        Self {
            include_types: true,
        }
    }
}

impl Transpiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Transpile a Ruchy expression to Rust TokenStream
    pub fn transpile(&self, expr: &Expr) -> Result<TokenStream> {
        self.transpile_expr(expr)
    }

    /// Transpile to a formatted Rust string
    pub fn transpile_to_string(&self, expr: &Expr) -> Result<String> {
        let tokens = self.transpile(expr)?;
        let file = syn::parse2::<syn::File>(quote! {
            fn main() {
                #tokens
            }
        })?;
        Ok(prettyplease::unparse(&file))
    }

    fn transpile_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Literal(lit) => self.transpile_literal(lit),
            ExprKind::Identifier(name) => {
                let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                Ok(quote! { #ident })
            }
            ExprKind::Binary { left, op, right } => self.transpile_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.transpile_unary(*op, operand),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.transpile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Let { name, value, body } => self.transpile_let(name, value, body),
            ExprKind::Function {
                name,
                params,
                return_type,
                body,
            } => self.transpile_function(name, params, return_type.as_ref(), body),
            ExprKind::Call { func, args } => self.transpile_call(func, args),
            ExprKind::Block(exprs) => self.transpile_block(exprs),
            ExprKind::Pipeline { expr, stages } => self.transpile_pipeline(expr, stages),
            ExprKind::Match { expr, arms } => self.transpile_match(expr, arms),
            ExprKind::List(elements) => self.transpile_list(elements),
            ExprKind::For { var, iter, body } => self.transpile_for(var, iter, body),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.transpile_range(start, end, *inclusive),
            ExprKind::Import { path, items } => self.transpile_import(path, items),
        }
    }

    fn transpile_literal(&self, lit: &Literal) -> Result<TokenStream> {
        Ok(match lit {
            Literal::Integer(n) => quote! { #n },
            Literal::Float(f) => quote! { #f },
            Literal::String(s) => quote! { #s },
            Literal::Bool(b) => quote! { #b },
            Literal::Unit => quote! { () },
        })
    }

    fn transpile_binary(&self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<TokenStream> {
        let left_tokens = self.transpile_expr(left)?;
        let right_tokens = self.transpile_expr(right)?;

        let op_tokens = match op {
            BinaryOp::Add => quote! { + },
            BinaryOp::Subtract => quote! { - },
            BinaryOp::Multiply => quote! { * },
            BinaryOp::Divide => quote! { / },
            BinaryOp::Modulo => quote! { % },
            BinaryOp::Power => {
                // Rust doesn't have a power operator, use method
                return Ok(quote! { (#left_tokens).pow(#right_tokens as u32) });
            }
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
        };

        Ok(quote! { (#left_tokens #op_tokens #right_tokens) })
    }

    fn transpile_unary(&self, op: UnaryOp, operand: &Expr) -> Result<TokenStream> {
        let operand_tokens = self.transpile_expr(operand)?;

        Ok(match op {
            UnaryOp::Not => quote! { !(#operand_tokens) },
            UnaryOp::Negate => quote! { -(#operand_tokens) },
            UnaryOp::BitwiseNot => quote! { !(#operand_tokens) },
        })
    }

    fn transpile_if(
        &self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;
        let then_tokens = self.transpile_expr(then_branch)?;

        if let Some(else_expr) = else_branch {
            let else_tokens = self.transpile_expr(else_expr)?;
            Ok(quote! {
                if #cond_tokens {
                    #then_tokens
                } else {
                    #else_tokens
                }
            })
        } else {
            Ok(quote! {
                if #cond_tokens {
                    #then_tokens
                }
            })
        }
    }

    fn transpile_let(&self, name: &str, value: &Expr, body: &Expr) -> Result<TokenStream> {
        let value_tokens = self.transpile_expr(value)?;
        let body_tokens = self.transpile_expr(body)?;
        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());

        Ok(quote! {
            {
                let #name_ident = #value_tokens;
                #body_tokens
            }
        })
    }

    fn transpile_function(
        &self,
        name: &str,
        params: &[Param],
        return_type: Option<&Type>,
        body: &Expr,
    ) -> Result<TokenStream> {
        let name_ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let body_tokens = self.transpile_expr(body)?;

        let param_tokens: Vec<TokenStream> = params
            .iter()
            .map(|p| {
                let param_name = syn::Ident::new(&p.name, proc_macro2::Span::call_site());
                let param_type = self.transpile_type(&p.ty).unwrap_or_else(|_| quote! { _ });
                quote! { #param_name: #param_type }
            })
            .collect();

        let return_type_tokens = if let Some(ret_ty) = return_type {
            let ty = self.transpile_type(ret_ty)?;
            quote! { -> #ty }
        } else {
            quote! {}
        };

        Ok(quote! {
            fn #name_ident(#(#param_tokens),*) #return_type_tokens {
                #body_tokens
            }
        })
    }

    fn transpile_call(&self, func: &Expr, args: &[Expr]) -> Result<TokenStream> {
        let func_tokens = self.transpile_expr(func)?;
        let arg_tokens: Result<Vec<_>> = args.iter().map(|arg| self.transpile_expr(arg)).collect();
        let arg_tokens = arg_tokens?;

        Ok(quote! {
            #func_tokens(#(#arg_tokens),*)
        })
    }

    fn transpile_block(&self, exprs: &[Expr]) -> Result<TokenStream> {
        if exprs.is_empty() {
            return Ok(quote! { {} });
        }

        let mut tokens = Vec::new();
        for (i, expr) in exprs.iter().enumerate() {
            let expr_tokens = self.transpile_expr(expr)?;
            if i < exprs.len() - 1 {
                // Not the last expression, add semicolon
                tokens.push(quote! { #expr_tokens; });
            } else {
                // Last expression, no semicolon (it's the return value)
                tokens.push(expr_tokens);
            }
        }

        Ok(quote! {
            {
                #(#tokens)*
            }
        })
    }

    fn transpile_pipeline(&self, expr: &Expr, stages: &[PipelineStage]) -> Result<TokenStream> {
        // Desugar pipeline: expr |> f |> g becomes g(f(expr))
        let mut result = self.transpile_expr(expr)?;

        for stage in stages {
            // Each stage is a function call with the previous result as first argument
            match &stage.op.kind {
                ExprKind::Identifier(func_name) => {
                    let func = syn::Ident::new(func_name, proc_macro2::Span::call_site());
                    result = quote! { #func(#result) };
                }
                ExprKind::Call { func, args } => {
                    // If the stage is already a call, insert the result as first argument
                    let func_tokens = self.transpile_expr(func)?;
                    let arg_tokens: Result<Vec<_>> =
                        args.iter().map(|arg| self.transpile_expr(arg)).collect();
                    let arg_tokens = arg_tokens?;
                    result = quote! { #func_tokens(#result, #(#arg_tokens),*) };
                }
                _ => bail!("Invalid pipeline stage"),
            }
        }

        Ok(result)
    }

    fn transpile_match(&self, expr: &Expr, arms: &[MatchArm]) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;

        let arm_tokens: Result<Vec<_>> = arms
            .iter()
            .map(|arm| {
                let pattern_tokens = self.transpile_pattern(&arm.pattern)?;
                let body_tokens = self.transpile_expr(&arm.body)?;

                if let Some(guard) = &arm.guard {
                    let guard_tokens = self.transpile_expr(guard)?;
                    Ok(quote! {
                        #pattern_tokens if #guard_tokens => #body_tokens
                    })
                } else {
                    Ok(quote! {
                        #pattern_tokens => #body_tokens
                    })
                }
            })
            .collect();
        let arm_tokens = arm_tokens?;

        Ok(quote! {
            match #expr_tokens {
                #(#arm_tokens),*
            }
        })
    }

    fn transpile_pattern(&self, pattern: &Pattern) -> Result<TokenStream> {
        Ok(match pattern {
            Pattern::Wildcard => quote! { _ },
            Pattern::Literal(lit) => self.transpile_literal(lit)?,
            Pattern::Identifier(name) => {
                let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                quote! { #ident }
            }
            Pattern::List(patterns) => {
                let pattern_tokens: Result<Vec<_>> =
                    patterns.iter().map(|p| self.transpile_pattern(p)).collect();
                let pattern_tokens = pattern_tokens?;
                quote! { [#(#pattern_tokens),*] }
            }
        })
    }

    fn transpile_list(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;

        Ok(quote! {
            vec![#(#element_tokens),*]
        })
    }

    fn transpile_for(&self, var: &str, iter: &Expr, body: &Expr) -> Result<TokenStream> {
        let var_ident = syn::Ident::new(var, proc_macro2::Span::call_site());
        let iter_tokens = self.transpile_expr(iter)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            for #var_ident in #iter_tokens {
                #body_tokens
            }
        })
    }

    fn transpile_range(&self, start: &Expr, end: &Expr, inclusive: bool) -> Result<TokenStream> {
        let start_tokens = self.transpile_expr(start)?;
        let end_tokens = self.transpile_expr(end)?;

        if inclusive {
            Ok(quote! { (#start_tokens..=#end_tokens) })
        } else {
            Ok(quote! { (#start_tokens..#end_tokens) })
        }
    }

    fn transpile_import(&self, _path: &str, _items: &[String]) -> Result<TokenStream> {
        // For now, just skip imports - they would be handled at module level
        // In a full implementation, we'd collect these and emit them at the top
        Ok(quote! {})
    }

    fn transpile_type(&self, ty: &Type) -> Result<TokenStream> {
        let _ = self; // Suppress unused self warning
        Ok(match &ty.kind {
            TypeKind::Named(name) => {
                // Map common Ruchy types to Rust types
                let rust_type = match name.as_str() {
                    "i32" => quote! { i32 },
                    "i64" => quote! { i64 },
                    "f32" => quote! { f32 },
                    "f64" => quote! { f64 },
                    "bool" => quote! { bool },
                    "String" => quote! { String },
                    _ => {
                        let ident = syn::Ident::new(name, proc_macro2::Span::call_site());
                        quote! { #ident }
                    }
                };
                rust_type
            }
            TypeKind::Optional(inner) => {
                let inner_tokens = self.transpile_type(inner)?;
                quote! { Option<#inner_tokens> }
            }
            TypeKind::List(inner) => {
                let inner_tokens = self.transpile_type(inner)?;
                quote! { Vec<#inner_tokens> }
            }
            TypeKind::Function { params, ret } => {
                let param_tokens: Result<Vec<_>> =
                    params.iter().map(|p| self.transpile_type(p)).collect();
                let param_tokens = param_tokens?;
                let ret_tokens = self.transpile_type(ret)?;
                quote! { fn(#(#param_tokens),*) -> #ret_tokens }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::Parser;

    fn transpile_str(input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        transpiler.transpile_to_string(&ast)
    }

    #[test]
    fn test_transpile_literals() {
        let result = transpile_str("42").unwrap();
        assert!(result.contains("42"));

        let result = transpile_str("3.14").unwrap();
        assert!(result.contains("3.14"));

        let result = transpile_str("\"hello\"").unwrap();
        assert!(result.contains("\"hello\""));

        let result = transpile_str("true").unwrap();
        assert!(result.contains("true"));
    }

    #[test]
    fn test_transpile_binary_ops() {
        let result = transpile_str("1 + 2 * 3").unwrap();
        // Check that the operations are present (formatting may vary)
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
        assert!(result.contains("+"));
        assert!(result.contains("*"));
    }

    #[test]
    fn test_transpile_if() {
        let result = transpile_str("if x > 0 { positive } else { negative }").unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_transpile_function() {
        let result = transpile_str("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        assert!(result.contains("fn add"));
        assert!(result.contains("x: i32"));
        assert!(result.contains("y: i32"));
        assert!(result.contains("-> i32"));
    }

    #[test]
    fn test_transpile_list() {
        let result = transpile_str("[1, 2, 3]").unwrap();
        assert!(result.contains("vec!"));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_transpile_match() {
        let result = transpile_str(r#"match x { 1 => "one", _ => "other" }"#).unwrap();
        assert!(result.contains("match"));
        assert!(result.contains("1"));
        assert!(result.contains("\"one\""));
        assert!(result.contains("\"other\""));
    }

    #[test]
    fn test_transpile_let() {
        let result = transpile_str("let x = 42 in x + 1").unwrap();
        assert!(result.contains("let x"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_transpile_for() {
        let result = transpile_str("for i in 1..10 { print(i) }").unwrap();
        assert!(result.contains("for i"));
        assert!(result.contains("in"));
    }

    #[test]
    fn test_transpile_range() {
        let result = transpile_str("1..10").unwrap();
        assert!(result.contains(".."));
        assert!(!result.contains("..="));

        let result = transpile_str("1..=10").unwrap();
        assert!(result.contains("..="));
    }

    #[test]
    fn test_transpile_pipeline() {
        let result = transpile_str("x |> f |> g").unwrap();
        // Pipeline becomes nested function calls: g(f(x))
        assert!(result.contains("g"));
        assert!(result.contains("f"));
    }

    #[test]
    fn test_transpile_unary() {
        let result = transpile_str("!true").unwrap();
        assert!(result.contains("!"));
        assert!(result.contains("true"));

        let result = transpile_str("-42").unwrap();
        assert!(result.contains("-"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_transpile_block() {
        // Blocks are part of function bodies or if expressions
        let result = transpile_str("if true { let x = 1; x + 1 } else { 0 }").unwrap();
        // Block should have braces
        assert!(result.contains("{"));
        assert!(result.contains("}"));
        assert!(result.contains("let x"));
    }
}
