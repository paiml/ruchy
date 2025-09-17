//! Refactored method call transpilation with reduced complexity
//! Original complexity: 58, Target: <20 per function
use crate::backend::Transpiler;
use crate::frontend::ast::Expr;
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{quote, format_ident};
impl Transpiler {
    /// Main dispatcher for method calls (complexity: ~15)
/// # Examples
/// 
/// ```ignore
/// use ruchy::backend::Transpiler;
/// let transpiler = Transpiler::new();
/// // Method call transpilation is handled internally
/// ```
pub fn transpile_method_call_refactored(
        &self,
        object: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<TokenStream> {
        let obj_tokens = self.transpile_expr(object)?;
        let arg_tokens: Result<Vec<_>> = args.iter().map(|a| self.transpile_expr(a)).collect();
        let arg_tokens = arg_tokens?;
        // Dispatch to specialized handlers based on method category
        match method {
            // Iterator methods
            "map" | "filter" | "reduce" | "fold" | "any" | "all" | "find" => 
                self.transpile_iterator_method(&obj_tokens, method, &arg_tokens),
            // HashMap/Dict methods
            "get" | "contains_key" | "keys" | "values" | "items" | "entry" =>
                self.transpile_hashmap_method(&obj_tokens, method, &arg_tokens),
            // HashSet methods
            "contains" | "union" | "intersection" | "difference" | "symmetric_difference" =>
                self.transpile_hashset_method(&obj_tokens, method, &arg_tokens),
            // Collection mutators
            "insert" | "remove" | "clear" | "push" | "pop" | "append" | "extend" =>
                self.transpile_collection_mutator(&obj_tokens, method, &arg_tokens),
            // Collection accessors
            "len" | "is_empty" | "iter" | "slice" | "first" | "last" =>
                self.transpile_collection_accessor(&obj_tokens, method, &arg_tokens),
            // String methods
            "to_s" | "to_string" | "to_upper" | "to_lower" | "length" | 
            "trim" | "split" | "replace" | "starts_with" | "ends_with" =>
                self.transpile_string_method(&obj_tokens, method, &arg_tokens),
            // DataFrame methods
            "select" | "groupby" | "agg" | "sort" | "mean" | "std" | "min" | "max" |
            "sum" | "count" | "drop_nulls" | "fill_null" | "pivot" | "melt" | 
            "head" | "tail" | "sample" | "describe" =>
                self.transpile_dataframe_method_refactored(&obj_tokens, method, &arg_tokens),
            // Default: pass through
            _ => {
                let method_ident = format_ident!("{}", method);
                Ok(quote! { #obj_tokens.#method_ident(#(#arg_tokens),*) })
            }
        }
    }
    /// Handle iterator methods (complexity: ~10)
    fn transpile_iterator_method(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "map" => Ok(quote! { #obj.iter().map(#(#args),*).collect::<Vec<_>>() }),
            "filter" => Ok(quote! { #obj.into_iter().filter(#(#args),*).collect::<Vec<_>>() }),
            "reduce" => Ok(quote! { #obj.into_iter().reduce(#(#args),*) }),
            "fold" => {
                if args.len() != 2 {
                    bail!("fold requires exactly 2 arguments");
                }
                let init = &args[0];
                let func = &args[1];
                Ok(quote! { #obj.into_iter().fold(#init, #func) })
            }
            "any" => Ok(quote! { #obj.iter().any(#(#args),*) }),
            "all" => Ok(quote! { #obj.iter().all(#(#args),*) }),
            "find" => Ok(quote! { #obj.iter().find(#(#args),*).cloned() }),
            _ => unreachable!("Unknown iterator method: {}", method),
        }
    }
    /// Handle HashMap/Dict methods (complexity: ~10)
    fn transpile_hashmap_method(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        match method {
            "get" => Ok(quote! { #obj.#method_ident(#(#args),*).cloned() }),
            "items" => Ok(quote! { #obj.iter().map(|(k, v)| (k.clone(), v.clone())) }),
            "contains_key" | "keys" | "values" | "entry" => 
                Ok(quote! { #obj.#method_ident(#(#args),*) }),
            _ => unreachable!("Unknown HashMap method: {}", method),
        }
    }
    /// Handle `HashSet` methods (complexity: ~12)
    fn transpile_hashset_method(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "contains" => {
                let method_ident = format_ident!("{}", method);
                Ok(quote! { #obj.#method_ident(#(#args),*) })
            }
            "union" | "intersection" | "difference" | "symmetric_difference" => {
                if args.len() != 1 {
                    bail!("{} requires exactly 1 argument", method);
                }
                let other = &args[0];
                let method_ident = format_ident!("{}", method);
                Ok(quote! { 
                    {
                        use std::collections::HashSet;
#[cfg(test)]
                        #obj.#method_ident(&#other).cloned().collect::<HashSet<_>>()
                    }
                })
            }
            _ => unreachable!("Unknown HashSet method: {}", method),
        }
    }
    /// Handle collection mutator methods (complexity: ~5)
    fn transpile_collection_mutator(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        Ok(quote! { #obj.#method_ident(#(#args),*) })
    }
    /// Handle collection accessor methods (complexity: ~10)
    fn transpile_collection_accessor(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "slice" => {
                if args.len() != 2 {
                    bail!("slice requires exactly 2 arguments");
                }
                let start = &args[0];
                let end = &args[1];
                Ok(quote! { #obj[#start..#end].to_vec() })
            }
            "first" => Ok(quote! { #obj.first().cloned() }),
            "last" => Ok(quote! { #obj.last().cloned() }),
            _ => {
                let method_ident = format_ident!("{}", method);
                Ok(quote! { #obj.#method_ident(#(#args),*) })
            }
        }
    }
    /// Handle string methods (complexity: ~12)
    fn transpile_string_method(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        match method {
            "to_s" | "to_string" => Ok(quote! { #obj }),
            "to_upper" => Ok(quote! { #obj.to_uppercase(#(#args),*) }),
            "to_lower" => Ok(quote! { #obj.to_lowercase(#(#args),*) }),
            "length" => Ok(quote! { #obj.len(#(#args),*) }),
            "trim" => Ok(quote! { #obj.trim(#(#args),*).to_string() }),
            "split" => {
                if args.is_empty() {
                    Ok(quote! { #obj.split_whitespace().map(String::from).collect::<Vec<_>>() })
                } else {
                    Ok(quote! { #obj.split(#(#args),*).map(String::from).collect::<Vec<_>>() })
                }
            }
            "replace" => {
                if args.len() != 2 {
                    bail!("replace requires exactly 2 arguments");
                }
                Ok(quote! { #obj.replace(#(#args),*) })
            }
            "starts_with" | "ends_with" => {
                let method_ident = format_ident!("{}", method);
                Ok(quote! { #obj.#method_ident(#(#args),*) })
            }
            _ => unreachable!("Unknown string method: {}", method),
        }
    }
    /// Handle `DataFrame` methods (complexity: ~5)
    fn transpile_dataframe_method_refactored(
        &self,
        obj: &TokenStream,
        method: &str,
        args: &[TokenStream],
    ) -> Result<TokenStream> {
        let method_ident = format_ident!("{}", method);
        Ok(quote! { #obj.#method_ident(#(#args),*) })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Transpiler;
    use crate::frontend::ast::{Expr, ExprKind, Literal};
    use proc_macro2::TokenStream;

    fn setup_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn make_ident_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Default::default(),
            attributes: Vec::new(),
        }
    }

    fn make_string_expr(s: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(s.to_string())),
            span: Default::default(),
            attributes: Vec::new(),
        }
    }

    #[test]
    fn test_iterator_methods() {
        let t = setup_transpiler();
        let obj = make_ident_expr("vec");
        let arg = make_ident_expr("x");

        // Test map method
        let result = t.transpile_method_call_refactored(&obj, "map", &[arg.clone()]);
        assert!(result.is_ok());

        // Test filter method
        let result = t.transpile_method_call_refactored(&obj, "filter", &[arg.clone()]);
        assert!(result.is_ok());

        // Test reduce method
        let result = t.transpile_method_call_refactored(&obj, "reduce", &[arg.clone()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hashmap_methods() {
        let t = setup_transpiler();
        let obj = make_ident_expr("map");
        let key = make_string_expr("key");

        // Test get method
        let result = t.transpile_method_call_refactored(&obj, "get", &[key.clone()]);
        assert!(result.is_ok());

        // Test contains_key method
        let result = t.transpile_method_call_refactored(&obj, "contains_key", &[key.clone()]);
        assert!(result.is_ok());

        // Test keys method
        let result = t.transpile_method_call_refactored(&obj, "keys", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hashset_methods() {
        let t = setup_transpiler();
        let obj = make_ident_expr("set");
        let val = make_string_expr("value");

        // Test contains method
        let result = t.transpile_method_call_refactored(&obj, "contains", &[val.clone()]);
        assert!(result.is_ok());

        // Test union method
        let other = make_ident_expr("other_set");
        let result = t.transpile_method_call_refactored(&obj, "union", &[other]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collection_mutators() {
        let t = setup_transpiler();
        let obj = make_ident_expr("vec");
        let val = make_string_expr("item");

        // Test push method
        let result = t.transpile_method_call_refactored(&obj, "push", &[val.clone()]);
        assert!(result.is_ok());

        // Test pop method
        let result = t.transpile_method_call_refactored(&obj, "pop", &[]);
        assert!(result.is_ok());

        // Test clear method
        let result = t.transpile_method_call_refactored(&obj, "clear", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_collection_accessors() {
        let t = setup_transpiler();
        let obj = make_ident_expr("vec");

        // Test len method
        let result = t.transpile_method_call_refactored(&obj, "len", &[]);
        assert!(result.is_ok());

        // Test is_empty method
        let result = t.transpile_method_call_refactored(&obj, "is_empty", &[]);
        assert!(result.is_ok());

        // Test first method
        let result = t.transpile_method_call_refactored(&obj, "first", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_methods() {
        let t = setup_transpiler();
        let obj = make_string_expr("hello");

        // Test to_upper method
        let result = t.transpile_method_call_refactored(&obj, "to_upper", &[]);
        assert!(result.is_ok());

        // Test to_lower method
        let result = t.transpile_method_call_refactored(&obj, "to_lower", &[]);
        assert!(result.is_ok());

        // Test length method
        let result = t.transpile_method_call_refactored(&obj, "length", &[]);
        assert!(result.is_ok());

        // Test trim method
        let result = t.transpile_method_call_refactored(&obj, "trim", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dataframe_methods() {
        let t = setup_transpiler();
        let obj = make_ident_expr("df");
        let col = make_string_expr("column");

        // Test select method
        let result = t.transpile_method_call_refactored(&obj, "select", &[col.clone()]);
        assert!(result.is_ok());

        // Test mean method
        let result = t.transpile_method_call_refactored(&obj, "mean", &[]);
        assert!(result.is_ok());

        // Test sum method
        let result = t.transpile_method_call_refactored(&obj, "sum", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_method() {
        let t = setup_transpiler();
        let obj = make_ident_expr("obj");
        let arg = make_string_expr("arg");

        // Test unknown method falls through to default
        let result = t.transpile_method_call_refactored(&obj, "unknown_method", &[arg]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_iterator_method_implementations() {
        let t = setup_transpiler();
        let tokens: TokenStream = quote! { vec };
        let args = vec![quote! { |x| x * 2 }];

        // Test transpile_iterator_method directly
        let result = t.transpile_iterator_method(&tokens, "map", &args);
        assert!(result.is_ok());

        // Test with filter
        let result = t.transpile_iterator_method(&tokens, "filter", &[quote! { |x| x > 0 }]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hashmap_method_implementations() {
        let t = setup_transpiler();
        let tokens: TokenStream = quote! { hashmap };
        let key_arg = vec![quote! { "key" }];

        // Test transpile_hashmap_method directly
        let result = t.transpile_hashmap_method(&tokens, "get", &key_arg);
        assert!(result.is_ok());

        // Test keys method with no args
        let result = t.transpile_hashmap_method(&tokens, "keys", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_method_implementations() {
        let t = setup_transpiler();
        let tokens: TokenStream = quote! { "hello" };

        // Test various string methods
        let result = t.transpile_string_method(&tokens, "to_upper", &[]);
        assert!(result.is_ok());

        let split_arg = vec![quote! { " " }];
        let result = t.transpile_string_method(&tokens, "split", &split_arg);
        assert!(result.is_ok());
    }
}
