//! Control flow statement transpilation (if, while, for, loop)
//! 
//! Each function maintains complexity ≤10 through focused responsibility

use super::super::Transpiler;
use crate::frontend::ast::{Expr, Pattern, PipelineStage};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{quote, format_ident};

impl Transpiler {
    /// Transpile if expression (complexity: 7)
    pub fn transpile_if(
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

    /// Transpile for loop (complexity: 8)
    pub fn transpile_for(
        &self,
        var: &str,
        pattern: Option<&Pattern>,
        iter: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let iter_tokens = self.transpile_expr(iter)?;
        let body_tokens = self.transpile_expr(body)?;

        if let Some(pat) = pattern {
            let pattern_tokens = self.transpile_pattern(pat)?;
            Ok(quote! {
                for #pattern_tokens in #iter_tokens {
                    #body_tokens
                }
            })
        } else {
            let var_ident = format_ident!("{}", var);
            Ok(quote! {
                for #var_ident in #iter_tokens {
                    #body_tokens
                }
            })
        }
    }

    /// Transpile while loop (complexity: 4)
    pub fn transpile_while(&self, condition: &Expr, body: &Expr) -> Result<TokenStream> {
        let cond_tokens = self.transpile_expr(condition)?;
        let body_tokens = self.transpile_expr(body)?;
        
        Ok(quote! {
            while #cond_tokens {
                #body_tokens
            }
        })
    }

    /// Transpile infinite loop (complexity: 3)
    pub fn transpile_loop(&self, body: &Expr) -> Result<TokenStream> {
        let body_tokens = self.transpile_expr(body)?;
        
        Ok(quote! {
            loop {
                #body_tokens
            }
        })
    }

    /// Transpile if-let expression (complexity: 8)
    pub fn transpile_if_let(
        &self,
        pattern: &Pattern,
        value: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let value_tokens = self.transpile_expr(value)?;
        let then_tokens = self.transpile_expr(then_branch)?;

        if let Some(else_expr) = else_branch {
            let else_tokens = self.transpile_expr(else_expr)?;
            Ok(quote! {
                if let #pattern_tokens = #value_tokens {
                    #then_tokens
                } else {
                    #else_tokens
                }
            })
        } else {
            Ok(quote! {
                if let #pattern_tokens = #value_tokens {
                    #then_tokens
                }
            })
        }
    }

    /// Transpile while-let loop (complexity: 5)
    pub fn transpile_while_let(
        &self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<TokenStream> {
        let pattern_tokens = self.transpile_pattern(pattern)?;
        let value_tokens = self.transpile_expr(value)?;
        let body_tokens = self.transpile_expr(body)?;

        Ok(quote! {
            while let #pattern_tokens = #value_tokens {
                #body_tokens
            }
        })
    }

    /// Transpile pipeline expression (complexity: 9)
    pub fn transpile_pipeline(
        &self,
        expr: &Expr,
        stages: &[PipelineStage],
    ) -> Result<TokenStream> {
        let mut result = self.transpile_expr(expr)?;
        
        for stage in stages {
            result = self.apply_pipeline_stage(result, stage)?;
        }
        
        Ok(result)
    }

    /// Apply a single pipeline stage (complexity: 7)
    fn apply_pipeline_stage(
        &self,
        input: TokenStream,
        stage: &PipelineStage,
    ) -> Result<TokenStream> {
        match stage {
            PipelineStage::FunctionCall(name, args) => {
                let name_ident = format_ident!("{}", name);
                let arg_tokens = self.transpile_expr_list(args)?;
                Ok(quote! { #name_ident(#input, #(#arg_tokens),*) })
            }
            PipelineStage::MethodCall(method, args) => {
                let method_ident = format_ident!("{}", method);
                let arg_tokens = self.transpile_expr_list(args)?;
                Ok(quote! { #input.#method_ident(#(#arg_tokens),*) })
            }
            PipelineStage::Lambda(lambda_expr) => {
                let lambda_tokens = self.transpile_expr(lambda_expr)?;
                Ok(quote! { (#lambda_tokens)(#input) })
            }
        }
    }

    /// Helper: Transpile list of expressions (complexity: 3)
    fn transpile_expr_list(&self, exprs: &[Expr]) -> Result<Vec<TokenStream>> {
        exprs.iter().map(|e| self.transpile_expr(e)).collect()
    }
}