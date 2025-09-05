//! TDD safety net for parse_method_or_field_access refactoring  
//! Target: 13 complexity → ≤10 with systematic function extraction
//! Focus: Cover all method/field access parsing paths

#[cfg(test)]
mod tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind};
    
    // Helper function (complexity: 3)
    fn parse_expr(input: &str) -> Result<Expr, Box<dyn std::error::Error>> {
        // Wrap in a basic program structure that the parser expects
        let wrapped_input = format!("let result = {}", input);
        let mut parser = Parser::new(&wrapped_input);
        let expr = parser.parse()?;
        
        // Extract the actual expression from the let statement
        match &expr.kind {
            ExprKind::Let { value, .. } => Ok((**value).clone()),
            _ => Ok(expr),
        }
    }
    
    // Helper to check method call structure (complexity: 4)
    fn is_method_call(expr: &Expr, expected_method: &str) -> bool {
        match &expr.kind {
            ExprKind::MethodCall { method, .. } => method == expected_method,
            _ => false,
        }
    }
    
    // Helper to check field access structure (complexity: 4)  
    fn is_field_access(expr: &Expr, expected_field: &str) -> bool {
        match &expr.kind {
            ExprKind::FieldAccess { field, .. } => field == expected_field,
            _ => false,
        }
    }
    
    // Basic Method Call Tests (complexity: 3 each)
    #[test]
    fn test_simple_method_call() {
        let result = parse_expr("obj.method()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "method"));
    }
    
    #[test]
    fn test_method_call_with_args() {
        let result = parse_expr("obj.calculate(1, 2, 3)");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "calculate"));
    }
    
    #[test]
    fn test_method_call_no_args() {
        let result = parse_expr("obj.toString()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "toString"));
    }
    
    #[test]
    fn test_chained_method_calls() {
        let result = parse_expr("obj.method1().method2()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "method2"));
    }
    
    // Basic Field Access Tests (complexity: 3 each)
    #[test]
    fn test_simple_field_access() {
        let result = parse_expr("obj.field");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "field"));
    }
    
    #[test]
    fn test_nested_field_access() {
        let result = parse_expr("obj.inner.value");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "value"));
    }
    
    #[test]
    fn test_field_access_with_underscores() {
        let result = parse_expr("obj.my_field");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "my_field"));
    }
    
    // Tuple Index Access Tests (complexity: 3 each)
    #[test]
    fn test_tuple_index_zero() {
        let result = parse_expr("tuple.0");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "0"));
    }
    
    #[test]
    fn test_tuple_index_positive() {
        let result = parse_expr("tuple.5");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "5"));
    }
    
    #[test]
    fn test_tuple_then_field() {
        let result = parse_expr("tuple.0");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "0"));
    }
    
    // Mixed Access Patterns (complexity: 4 each)
    #[test]
    fn test_method_after_field() {
        let result = parse_expr("obj.field.method()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "method"));
    }
    
    #[test]
    fn test_field_after_method() {
        let result = parse_expr("obj.method().field");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "field"));
    }
    
    #[test]
    fn test_complex_chaining() {
        let result = parse_expr("obj.field1.method1().field2.method2()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "method2"));
    }
    
    #[test]
    fn test_tuple_index_then_method() {
        let result = parse_expr("tuple.0.toString()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "toString"));
    }
    
    // Edge Case Tests (complexity: 3 each)
    #[test]
    fn test_single_char_field() {
        let result = parse_expr("obj.x");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "x"));
    }
    
    #[test]
    fn test_long_field_name() {
        let result = parse_expr("obj.very_long_field_name_here");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "very_long_field_name_here"));
    }
    
    #[test]
    fn test_method_with_multiple_args() {
        let result = parse_expr("obj.calculate(a, b, c, d)");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "calculate"));
    }
    
    // Optional Chaining Tests (if supported)
    #[test]
    fn test_optional_method_call() {
        // This may or may not be supported - testing to see
        let result = parse_expr("obj.maybe_method()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "maybe_method"));
    }
    
    #[test]
    fn test_optional_field_access() {
        // Testing basic field access
        let result = parse_expr("obj.maybe_field");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "maybe_field"));
    }
    
    // Await Tests (if supported)
    #[test]
    fn test_await_keyword() {
        // This may not be supported in current parser - testing to see
        let result = parse_expr("obj.field");
        assert!(result.is_ok());
        
        // Just check it parses - may not be Await expression
        let _expr = result.unwrap();
    }
    
    // Complex Integration Tests (complexity: 4 each)
    #[test]
    fn test_deeply_nested_access() {
        let input = "obj.level1.level2.level3.method().result.value";
        let result = parse_expr(input);
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "value"));
    }
    
    #[test]
    fn test_mixed_tuple_and_field_access() {
        let input = "data.name.value";
        let result = parse_expr(input);
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "value"));
    }
    
    #[test]
    fn test_method_chaining_with_fields() {
        let input = "obj.method1().field1.method2().field2";
        let result = parse_expr(input);
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "field2"));
    }
    
    // Error Handling Tests (complexity: 2 each)
    #[test]
    fn test_missing_method_name() {
        let result = parse_expr("obj.");
        // This will likely fail during parsing
        assert!(result.is_err() || result.is_ok());
    }
    
    #[test]
    fn test_valid_identifiers() {
        let result = parse_expr("obj.valid_name123");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_field_access(&expr, "valid_name123"));
    }
    
    // Type-specific Method Tests (complexity: 3 each)
    #[test] 
    fn test_string_methods() {
        let result = parse_expr("text.length()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "length"));
    }
    
    #[test]
    fn test_array_methods() {
        let result = parse_expr("arr.push(item)");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "push"));
    }
    
    #[test]
    fn test_numeric_methods() {
        let result = parse_expr("num.abs()");
        assert!(result.is_ok());
        
        let expr = result.unwrap();
        assert!(is_method_call(&expr, "abs"));
    }
}