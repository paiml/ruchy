// TDD Test Suite for Transpiler::try_transpile_type_conversion_old Complexity Reduction  
// Current: 62 cyclomatic complexity - NEW #1 HOTSPOT after Value::inspect fix
// Target: <20 for both metrics
// Strategy: Extract type-specific conversion handlers (str, int, float, bool)

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal};
use ruchy::frontend::span::Span;

#[cfg(test)]
mod transpiler_type_conversion_tdd {
    use super::*;

    fn create_test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    fn create_string_literal(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_integer_literal(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_float_literal(value: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(value)),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_bool_literal(value: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(value)),
            span: Span::default(),
            attributes: vec![],
        }
    }

    // Test str() conversion functionality
    #[test]
    fn test_str_conversion_basic() {
        let transpiler = create_test_transpiler();
        let args = vec![create_integer_literal(42)];
        
        let result = transpiler.try_transpile_type_conversion_old("str", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_str_conversion_wrong_args() {
        let transpiler = create_test_transpiler();
        let args = vec![create_integer_literal(42), create_integer_literal(24)];
        
        let result = transpiler.try_transpile_type_conversion_old("str", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("str() expects exactly 1 argument"));
    }

    // Test int() conversion functionality  
    #[test]
    fn test_int_conversion_from_string() {
        let transpiler = create_test_transpiler();
        let args = vec![create_string_literal("123")];
        
        let result = transpiler.try_transpile_type_conversion_old("int", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_int_conversion_from_float() {
        let transpiler = create_test_transpiler();
        let args = vec![create_float_literal(42.7)];
        
        let result = transpiler.try_transpile_type_conversion_old("int", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_int_conversion_from_bool() {
        let transpiler = create_test_transpiler();
        let args = vec![create_bool_literal(true)];
        
        let result = transpiler.try_transpile_type_conversion_old("int", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_int_conversion_wrong_args() {
        let transpiler = create_test_transpiler();
        let args = vec![];
        
        let result = transpiler.try_transpile_type_conversion_old("int", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("int() expects exactly 1 argument"));
    }

    // Test float() conversion functionality
    #[test]
    fn test_float_conversion_from_string() {
        let transpiler = create_test_transpiler();
        let args = vec![create_string_literal("3.14")];
        
        let result = transpiler.try_transpile_type_conversion_old("float", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_float_conversion_from_integer() {
        let transpiler = create_test_transpiler();
        let args = vec![create_integer_literal(42)];
        
        let result = transpiler.try_transpile_type_conversion_old("float", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_float_conversion_wrong_args() {
        let transpiler = create_test_transpiler();
        let args = vec![create_integer_literal(1), create_integer_literal(2)];
        
        let result = transpiler.try_transpile_type_conversion_old("float", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("float() expects exactly 1 argument"));
    }

    // Test bool() conversion functionality
    #[test] 
    fn test_bool_conversion_from_integer() {
        let transpiler = create_test_transpiler();
        let args = vec![create_integer_literal(1)];
        
        let result = transpiler.try_transpile_type_conversion_old("bool", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_bool_conversion_from_string() {
        let transpiler = create_test_transpiler();
        let args = vec![create_string_literal("hello")];
        
        let result = transpiler.try_transpile_type_conversion_old("bool", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_bool_conversion_from_bool() {
        let transpiler = create_test_transpiler();
        let args = vec![create_bool_literal(false)];
        
        let result = transpiler.try_transpile_type_conversion_old("bool", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_bool_conversion_wrong_args() {
        let transpiler = create_test_transpiler();
        let args = vec![create_bool_literal(true), create_bool_literal(false)];
        
        let result = transpiler.try_transpile_type_conversion_old("bool", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("bool() expects exactly 1 argument"));
    }

    // Test unsupported conversion types
    #[test]
    fn test_unsupported_conversion_type() {
        let transpiler = create_test_transpiler();
        let args = vec![create_integer_literal(42)];
        
        let result = transpiler.try_transpile_type_conversion_old("unknown_type", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Should return None for unsupported types
    }

    // Tests for refactored helper methods (to be implemented)
    mod refactored_helpers {
        use super::*;

        #[test]
        fn test_transpile_str_conversion() {
            // Test extracted str conversion handler
            let transpiler = create_test_transpiler();
            let args = vec![create_integer_literal(42)];
            
            // This would test the extracted transpile_str_conversion once implemented
            // let result = transpiler.transpile_str_conversion(&args);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_transpile_int_conversion() {
            // Test extracted int conversion handler
            let transpiler = create_test_transpiler();
            let args = vec![create_string_literal("123")];
            
            // This would test the extracted transpile_int_conversion once implemented  
            // let result = transpiler.transpile_int_conversion(&args);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_transpile_float_conversion() {
            // Test extracted float conversion handler
            let transpiler = create_test_transpiler();
            let args = vec![create_integer_literal(42)];
            
            // This would test the extracted transpile_float_conversion once implemented
            // let result = transpiler.transpile_float_conversion(&args);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_transpile_bool_conversion() {
            // Test extracted bool conversion handler
            let transpiler = create_test_transpiler();
            let args = vec![create_string_literal("hello")];
            
            // This would test the extracted transpile_bool_conversion once implemented
            // let result = transpiler.transpile_bool_conversion(&args);
            // assert!(result.is_ok());
        }
    }
}

// Demonstration of how the refactoring would work
// These would be the extracted helper methods to reduce complexity
/*
impl Transpiler {
    // Main method becomes a dispatcher (complexity ~5)
    fn try_transpile_type_conversion_old(&self, base_name: &str, args: &[Expr]) -> Result<Option<TokenStream>> {
        match base_name {
            "str" => self.transpile_str_conversion(args).map(Some),
            "int" => self.transpile_int_conversion(args).map(Some), 
            "float" => self.transpile_float_conversion(args).map(Some),
            "bool" => self.transpile_bool_conversion(args).map(Some),
            _ => Ok(None)
        }
    }

    // Extract str conversion logic (complexity ~3)
    fn transpile_str_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("str() expects exactly 1 argument");
        }
        let value = self.transpile_expr(&args[0])?;
        Ok(quote! { format!("{}", #value) })
    }

    // Extract int conversion logic (complexity ~8)  
    fn transpile_int_conversion(&self, args: &[Expr]) -> Result<TokenStream> {
        if args.len() != 1 {
            bail!("int() expects exactly 1 argument");
        }
        
        match &args[0].kind {
            ExprKind::Literal(Literal::String(_)) => self.transpile_int_from_string(&args[0]),
            ExprKind::Literal(Literal::Float(_)) => self.transpile_int_from_float(&args[0]),
            ExprKind::Literal(Literal::Bool(_)) => self.transpile_int_from_bool(&args[0]),
            ExprKind::StringInterpolation { parts } if parts.len() == 1 => {
                self.transpile_int_from_string_interpolation(&args[0], parts)
            }
            _ => self.transpile_int_generic(&args[0])
        }
    }

    // Similar extraction for float and bool conversions...
    // Each helper method focused on single conversion type with <10 complexity
}
*/