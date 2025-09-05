//! Refactored type conversion with reduced complexity
//! Original complexity: 62, Target: <20 per function

use crate::backend::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal, StringPart};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;

impl Transpiler {
    /// Main dispatcher for type conversion (complexity: ~8)
    pub fn try_transpile_type_conversion_refactored(
        &self, 
        base_name: &str, 
        args: &[Expr]
    ) -> Result<Option<TokenStream>> {
        // Only check known type conversion functions
        match base_name {
            "str" | "int" | "float" | "bool" | "list" | "set" | "dict" => {
                // These functions require exactly 1 argument
                if args.len() != 1 {
                    bail!("{base_name}() expects exactly 1 argument");
                }
            }
            _ => return Ok(None), // Not a type conversion, don't validate
        }
        
        match base_name {
            "str" => self.convert_to_string(&args[0]),
            "int" => self.convert_to_int(&args[0]),
            "float" => self.convert_to_float(&args[0]),
            "bool" => self.convert_to_bool(&args[0]),
            "list" => self.convert_to_list(&args[0]),
            "set" => self.convert_to_set(&args[0]),
            "dict" => self.convert_to_dict(&args[0]),
            _ => Ok(None), // Not a type conversion
        }
    }
    
    /// Convert to string (complexity: 3)
    fn convert_to_string(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        let value = self.transpile_expr(arg)?;
        Ok(Some(quote! { format!("{}", #value) }))
    }
    
    /// Convert to integer (complexity: 12)
    fn convert_to_int(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        match &arg.kind {
            // String literal -> parse
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { #value.parse::<i64>().expect("Failed to parse integer") }))
            }
            // String interpolation with single text part -> parse
            ExprKind::StringInterpolation { parts } if is_single_text_part(parts) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { #value.parse::<i64>().expect("Failed to parse integer") }))
            }
            // Float literal -> cast
            ExprKind::Literal(Literal::Float(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { (#value as i64) }))
            }
            // Bool literal -> 0 or 1
            ExprKind::Literal(Literal::Bool(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { if #value { 1i64 } else { 0i64 } }))
            }
            // Default: runtime cast
            _ => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { (#value as i64) }))
            }
        }
    }
    
    /// Convert to float (complexity: 10)
    fn convert_to_float(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        match &arg.kind {
            // String literal -> parse
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { #value.parse::<f64>().expect("Failed to parse float") }))
            }
            // String interpolation with single text part -> parse
            ExprKind::StringInterpolation { parts } if is_single_text_part(parts) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { #value.parse::<f64>().expect("Failed to parse float") }))
            }
            // Integer literal -> cast
            ExprKind::Literal(Literal::Integer(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { (#value as f64) }))
            }
            // Default: runtime cast
            _ => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { (#value as f64) }))
            }
        }
    }
    
    /// Convert to bool (complexity: ~10)
    fn convert_to_bool(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        // Delegate to specific bool conversion based on literal type
        match &arg.kind {
            ExprKind::Literal(lit) => self.convert_literal_to_bool(lit, arg),
            ExprKind::StringInterpolation { .. } => self.convert_string_to_bool(arg),
            ExprKind::List(_) => self.convert_collection_to_bool(arg),
            ExprKind::None => Ok(Some(quote! { false })),
            _ => self.convert_generic_to_bool(arg),
        }
    }
    
    /// Convert literal to bool (complexity: ~8)
    fn convert_literal_to_bool(&self, lit: &Literal, arg: &Expr) -> Result<Option<TokenStream>> {
        match lit {
            Literal::Integer(_) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { (#value != 0) }))
            }
            Literal::String(_) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { !#value.is_empty() }))
            }
            Literal::Bool(_) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(value))
            }
            _ => self.convert_generic_to_bool(arg),
        }
    }
    
    /// Convert string to bool (complexity: 3)
    fn convert_string_to_bool(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        let value = self.transpile_expr(arg)?;
        Ok(Some(quote! { !#value.is_empty() }))
    }
    
    /// Convert collection to bool (complexity: 3)
    fn convert_collection_to_bool(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        let value = self.transpile_expr(arg)?;
        Ok(Some(quote! { !#value.is_empty() }))
    }
    
    /// Generic truthiness check (complexity: 3)
    fn convert_generic_to_bool(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        let value = self.transpile_expr(arg)?;
        Ok(Some(quote! { 
            {
                // Generic truthiness check
                match &#value {
                    0 => false,
                    _ => true,
                }
            }
        }))
    }
    
    /// Convert to list (complexity: 8)
    fn convert_to_list(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        match &arg.kind {
            // String -> chars as list
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { #value.chars().map(|c| c.to_string()).collect::<Vec<_>>() }))
            }
            // Range -> collect to vec
            ExprKind::Range { .. } => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { #value.collect::<Vec<_>>() }))
            }
            // Tuple -> convert to vec
            ExprKind::Tuple(_) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { vec![#value] }))
            }
            // Default: wrap in vec
            _ => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { vec![#value] }))
            }
        }
    }
    
    /// Convert to set (complexity: 8)
    fn convert_to_set(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        match &arg.kind {
            // List -> convert to HashSet
            ExprKind::List(_) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { 
                    #value.into_iter().collect::<std::collections::HashSet<_>>() 
                }))
            }
            // String -> chars as set
            ExprKind::Literal(Literal::String(_)) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { 
                    #value.chars().map(|c| c.to_string())
                        .collect::<std::collections::HashSet<_>>() 
                }))
            }
            // Default: single element set
            _ => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { 
                    {
                        let mut set = std::collections::HashSet::new();
                        set.insert(#value);
                        set
                    }
                }))
            }
        }
    }
    
    /// Convert to dict (complexity: 6)
    fn convert_to_dict(&self, arg: &Expr) -> Result<Option<TokenStream>> {
        match &arg.kind {
            // List of tuples -> dict
            ExprKind::List(items) if items.iter().all(is_tuple_expr) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { 
                    #value.into_iter()
                        .map(|(k, v)| (k, v))
                        .collect::<std::collections::HashMap<_, _>>() 
                }))
            }
            // ObjectLiteral -> convert to HashMap
            ExprKind::ObjectLiteral { .. } => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(quote! { 
                    #value.into_iter().collect::<std::collections::HashMap<_, _>>() 
                }))
            }
            // Default: empty dict
            _ => {
                Ok(Some(quote! { std::collections::HashMap::new() }))
            }
        }
    }
}

// Helper functions (complexity: 1-3 each)

fn is_single_text_part(parts: &[StringPart]) -> bool {
    parts.len() == 1 && matches!(&parts[0], StringPart::Text(_))
}

fn is_tuple_expr(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Tuple(items) if items.len() == 2)
}

#[cfg(test)]
#[path = "type_conversion_refactored_tests.rs"]
mod tests;