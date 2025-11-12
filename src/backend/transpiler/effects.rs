//! Effect system transpilation - SPEC-001-I, SPEC-001-J
use anyhow::bail;
use super::{Result, Transpiler};
use crate::frontend::ast::{EffectOperation, EffectHandler, Expr, Pattern};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// SPEC-001-I: Transpile effect declaration to Rust trait
    ///
    /// # Errors
    ///
    /// Returns an error if transpilation fails
    pub fn transpile_effect(&self, name: &str, operations: &[EffectOperation]) -> Result<TokenStream> {
        let effect_name = syn::parse_str::<syn::Ident>(name)?;
        let methods = transpile_effect_operations(self, operations)?;

        Ok(quote! {
            pub trait #effect_name {
                #(#methods)*
            }
        })
    }

    /// SPEC-001-J: Transpile effect handler expression
    ///
    /// # Errors
    ///
    /// Returns an error if transpilation fails
    pub fn transpile_handler(&self, expr: &Expr, _handlers: &[EffectHandler]) -> Result<TokenStream> {
        let expr_tokens = self.transpile_expr(expr)?;
        Ok(quote! {
            {
                let _ = #expr_tokens;
                ()
            }
        })
    }
}

fn transpile_effect_operations(
    transpiler: &Transpiler,
    operations: &[EffectOperation],
) -> Result<Vec<TokenStream>> {
    operations
        .iter()
        .map(|op| transpile_single_operation(transpiler, op))
        .collect()
}

fn transpile_single_operation(
    transpiler: &Transpiler,
    op: &EffectOperation,
) -> Result<TokenStream> {
    let op_name = syn::parse_str::<syn::Ident>(&op.name)?;
    let params = transpile_operation_params(transpiler, op)?;
    let return_type = transpile_operation_return_type(transpiler, op)?;
    
    Ok(quote! {
        fn #op_name(&self, #(#params),*) #return_type;
    })
}

fn transpile_operation_params(
    transpiler: &Transpiler,
    op: &EffectOperation,
) -> Result<Vec<TokenStream>> {
    op.params
        .iter()
        .map(|param| {
            let param_name = match &param.pattern {
                Pattern::Identifier(name) => syn::parse_str::<syn::Ident>(name)?,
                _ => bail!("Only identifier patterns supported in effect operation parameters"),
            };
            let param_type = transpiler.transpile_type(&param.ty)?;
            Ok(quote! { #param_name: #param_type })
        })
        .collect()
}

fn transpile_operation_return_type(
    transpiler: &Transpiler,
    op: &EffectOperation,
) -> Result<TokenStream> {
    if let Some(return_type) = &op.return_type {
        let return_tokens = transpiler.transpile_type(return_type)?;
        Ok(quote! { -> #return_tokens })
    } else {
        Ok(quote! {})
    }
}
