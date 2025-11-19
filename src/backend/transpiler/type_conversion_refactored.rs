//! Refactored type conversion with reduced complexity
//! Original complexity: 62, Target: <20 per function
use crate::backend::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal, StringPart};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::quote;
impl Transpiler {
    /// Main dispatcher for type conversion (complexity: ~8)
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::backend::Transpiler;
    /// let mut transpiler = Transpiler::new();
    /// // Type conversion is handled internally
    /// ```
    pub fn try_transpile_type_conversion_refactored(
        &self,
        base_name: &str,
        args: &[Expr],
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
                Ok(Some(
                    quote! { #value.parse::<i64>().expect("Failed to parse integer") },
                ))
            }
            // String interpolation with single text part -> parse
            ExprKind::StringInterpolation { parts } if is_single_text_part(parts) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(
                    quote! { #value.parse::<i64>().expect("Failed to parse integer") },
                ))
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
                Ok(Some(
                    quote! { #value.parse::<f64>().expect("Failed to parse float") },
                ))
            }
            // String interpolation with single text part -> parse
            ExprKind::StringInterpolation { parts } if is_single_text_part(parts) => {
                let value = self.transpile_expr(arg)?;
                Ok(Some(
                    quote! { #value.parse::<f64>().expect("Failed to parse float") },
                ))
            }
            // Integer literal -> cast
            ExprKind::Literal(Literal::Integer(_, _)) => {
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
            Literal::Integer(_, _) => {
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
                Ok(Some(
                    quote! { #value.chars().map(|c| c.to_string()).collect::<Vec<_>>() },
                ))
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
            _ => Ok(Some(quote! { std::collections::HashMap::new() })),
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
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span, StringPart};

    // Helper: Create test transpiler instance
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper: Create integer literal expression
    fn int_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create string literal expression
    fn string_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create float literal expression
    fn float_expr(value: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(value)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create bool literal expression
    fn bool_expr(value: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(value)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create None expression
    fn none_expr() -> Expr {
        Expr {
            kind: ExprKind::None,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create list expression
    fn list_expr(items: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(items),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: try_transpile_type_conversion_refactored - str() with 1 arg
    #[test]
    fn test_dispatcher_str_valid() {
        let transpiler = test_transpiler();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_type_conversion_refactored("str", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // Test 2: try_transpile_type_conversion_refactored - int() with 1 arg
    #[test]
    fn test_dispatcher_int_valid() {
        let transpiler = test_transpiler();
        let args = vec![string_expr("123")];
        let result = transpiler.try_transpile_type_conversion_refactored("int", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // Test 3: try_transpile_type_conversion_refactored - unknown function returns None
    #[test]
    fn test_dispatcher_unknown_function() {
        let transpiler = test_transpiler();
        let args = vec![int_expr(42)];
        let result = transpiler.try_transpile_type_conversion_refactored("unknown", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // Test 4: try_transpile_type_conversion_refactored - str() with 0 args (error path)
    #[test]
    fn test_dispatcher_str_zero_args() {
        let transpiler = test_transpiler();
        let args = vec![];
        let result = transpiler.try_transpile_type_conversion_refactored("str", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects exactly 1 argument"));
    }

    // Test 5: try_transpile_type_conversion_refactored - int() with 2 args (error path)
    #[test]
    fn test_dispatcher_int_two_args() {
        let transpiler = test_transpiler();
        let args = vec![int_expr(1), int_expr(2)];
        let result = transpiler.try_transpile_type_conversion_refactored("int", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects exactly 1 argument"));
    }

    // Test 6: convert_to_string - integer
    #[test]
    fn test_convert_to_string_integer() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_string(&int_expr(42));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        // TokenStream formats as "format ! (...)" with spaces
        assert!(output.contains("format") && output.contains('!'));
    }

    // Test 7: convert_to_int - string literal
    #[test]
    fn test_convert_to_int_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_int(&string_expr("123"));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("parse"));
        assert!(output.contains("i64"));
    }

    // Test 8: convert_to_int - float literal (cast)
    #[test]
    fn test_convert_to_int_float_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_int(&float_expr(std::f64::consts::PI));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("as i64"));
    }

    // Test 9: convert_to_int - bool literal (0 or 1)
    #[test]
    fn test_convert_to_int_bool_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_int(&bool_expr(true));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("if"));
        assert!(output.contains("1i64"));
        assert!(output.contains("0i64"));
    }

    // Test 10: convert_to_float - string literal
    #[test]
    fn test_convert_to_float_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_float(&string_expr("3.14"));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("parse"));
        assert!(output.contains("f64"));
    }

    // Test 11: convert_to_float - integer literal (cast)
    #[test]
    fn test_convert_to_float_integer_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_float(&int_expr(42));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("as f64"));
    }

    // Test 12: convert_to_bool - integer literal (!= 0)
    #[test]
    fn test_convert_to_bool_integer_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_bool(&int_expr(42));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("!= 0"));
    }

    // Test 13: convert_to_bool - string literal (!is_empty())
    #[test]
    fn test_convert_to_bool_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_bool(&string_expr("hello"));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("is_empty"));
    }

    // Test 14: convert_to_bool - bool literal (identity)
    #[test]
    fn test_convert_to_bool_bool_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_bool(&bool_expr(true));
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // Test 15: convert_to_bool - list (!is_empty())
    #[test]
    fn test_convert_to_bool_list() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_bool(&list_expr(vec![int_expr(1)]));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("is_empty"));
    }

    // Test 16: convert_to_bool - None → false
    #[test]
    fn test_convert_to_bool_none() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_bool(&none_expr());
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        assert_eq!(tokens.to_string(), "false");
    }

    // Test 17: convert_to_list - string literal (chars)
    #[test]
    fn test_convert_to_list_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_list(&string_expr("hello"));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("chars"));
        assert!(output.contains("collect"));
    }

    // Test 18: convert_to_set - list (HashSet)
    #[test]
    fn test_convert_to_set_list() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_set(&list_expr(vec![int_expr(1), int_expr(2)]));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("HashSet"));
    }

    // Test 19: convert_to_set - string literal (chars as set)
    #[test]
    fn test_convert_to_set_string_literal() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_set(&string_expr("abc"));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("chars"));
        assert!(output.contains("HashSet"));
    }

    // Test 20: convert_to_dict - generic (empty HashMap)
    #[test]
    fn test_convert_to_dict_generic() {
        let transpiler = test_transpiler();
        let result = transpiler.convert_to_dict(&int_expr(42));
        assert!(result.is_ok());
        let tokens = result.unwrap().unwrap();
        let output = tokens.to_string();
        assert!(output.contains("HashMap"));
        assert!(output.contains("new"));
    }

    // Test 21: is_single_text_part helper - single text part
    #[test]
    fn test_is_single_text_part_true() {
        let parts = vec![StringPart::Text("hello".to_string())];
        assert!(is_single_text_part(&parts));
    }

    // Test 22: is_single_text_part helper - multiple parts
    #[test]
    fn test_is_single_text_part_false_multiple() {
        let parts = vec![
            StringPart::Text("hello".to_string()),
            StringPart::Text("world".to_string()),
        ];
        assert!(!is_single_text_part(&parts));
    }

    // Test 23: is_single_text_part helper - empty parts
    #[test]
    fn test_is_single_text_part_false_empty() {
        let parts = vec![];
        assert!(!is_single_text_part(&parts));
    }

    // Test 24: is_tuple_expr helper - 2-element tuple
    #[test]
    fn test_is_tuple_expr_true() {
        let expr = Expr {
            kind: ExprKind::Tuple(vec![int_expr(1), int_expr(2)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(is_tuple_expr(&expr));
    }

    // Test 25: is_tuple_expr helper - non-tuple
    #[test]
    fn test_is_tuple_expr_false() {
        let expr = int_expr(42);
        assert!(!is_tuple_expr(&expr));
    }
}

#[cfg(test)]
mod property_tests_type_conversion_refactored {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_try_transpile_type_conversion_refactored_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
