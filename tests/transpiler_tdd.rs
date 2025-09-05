//! TDD tests for transpiler to boost coverage
//! Target: Test transpilation of Ruchy code to Rust

#[cfg(test)]
mod tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;
    
    // Test basic transpilation (complexity: 4 each)
    #[test]
    fn test_transpile_integer() {
        let mut parser = Parser::new("42");
        let ast = parser.parse().unwrap();
        let transpiler = Transpiler::default();
        let result = transpiler.transpile_to_string(&ast);
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("42"));
    }
    
    #[test]
    fn test_transpile_float() {
        let result = transpile("3.14");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("3.14"));
    }
    
    #[test]
    fn test_transpile_string() {
        let result = transpile(r#""hello""#);
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains(r#""hello""#));
    }
    
    #[test]
    fn test_transpile_bool_true() {
        let result = transpile("true");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("true"));
    }
    
    #[test]
    fn test_transpile_bool_false() {
        let result = transpile("false");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("false"));
    }
    
    // Test arithmetic operations (complexity: 4 each)
    #[test]
    fn test_transpile_addition() {
        let result = transpile("1 + 2");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("+"));
    }
    
    #[test]
    fn test_transpile_subtraction() {
        let result = transpile("5 - 3");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("-"));
    }
    
    #[test]
    fn test_transpile_multiplication() {
        let result = transpile("2 * 3");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("*"));
    }
    
    #[test]
    fn test_transpile_division() {
        let result = transpile("10 / 2");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("/"));
    }
    
    #[test]
    fn test_transpile_modulo() {
        let result = transpile("7 % 3");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("%"));
    }
    
    // Test comparison operations (complexity: 4 each)
    #[test]
    fn test_transpile_equal() {
        let result = transpile("1 == 1");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("=="));
    }
    
    #[test]
    fn test_transpile_not_equal() {
        let result = transpile("1 != 2");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("!="));
    }
    
    #[test]
    fn test_transpile_less_than() {
        let result = transpile("1 < 2");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("<"));
    }
    
    #[test]
    fn test_transpile_greater_than() {
        let result = transpile("2 > 1");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains(">"));
    }
    
    #[test]
    fn test_transpile_less_equal() {
        let result = transpile("1 <= 2");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("<="));
    }
    
    #[test]
    fn test_transpile_greater_equal() {
        let result = transpile("2 >= 1");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains(">="));
    }
    
    // Test logical operations (complexity: 4 each)
    #[test]
    fn test_transpile_and() {
        let result = transpile("true && false");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("&&"));
    }
    
    #[test]
    fn test_transpile_or() {
        let result = transpile("true || false");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("||"));
    }
    
    #[test]
    fn test_transpile_not() {
        let result = transpile("!true");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("!"));
    }
    
    // Test variable operations (complexity: 5 each)
    #[test]
    fn test_transpile_let_binding() {
        let result = transpile("let x = 42");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("let"));
        assert!(rust_code.contains("x"));
        assert!(rust_code.contains("42"));
    }
    
    #[test]
    fn test_transpile_let_mutable() {
        let result = transpile("let mut x = 42");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("mut"));
    }
    
    // Test collections (complexity: 4 each)
    #[test]
    fn test_transpile_list() {
        let result = transpile("[1, 2, 3]");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("vec!"));
    }
    
    #[test]
    fn test_transpile_empty_list() {
        let result = transpile("[]");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("vec!"));
    }
    
    #[test]
    fn test_transpile_object() {
        let result = transpile("{x: 1, y: 2}");
        assert!(result.is_ok());
        // Objects might transpile to structs or hashmaps
    }
    
    #[test]
    fn test_transpile_empty_object() {
        let result = transpile("{}");
        assert!(result.is_ok());
    }
    
    // Test control flow (complexity: 5 each)
    #[test]
    fn test_transpile_if() {
        let result = transpile("if true { 1 }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("if"));
    }
    
    #[test]
    fn test_transpile_if_else() {
        let result = transpile("if true { 1 } else { 2 }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("if"));
        assert!(rust_code.contains("else"));
    }
    
    #[test]
    fn test_transpile_while() {
        let result = transpile("while true { 1 }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("while"));
    }
    
    #[test]
    fn test_transpile_for() {
        let result = transpile("for x in [1, 2, 3] { x }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("for"));
        assert!(rust_code.contains("in"));
    }
    
    // Test functions (complexity: 5 each)
    #[test]
    fn test_transpile_function() {
        let result = transpile("fun add(a, b) { a + b }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn"));
        assert!(rust_code.contains("add"));
    }
    
    #[test]
    fn test_transpile_function_no_params() {
        let result = transpile("fun hello() { 42 }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("fn"));
        assert!(rust_code.contains("hello"));
    }
    
    #[test]
    fn test_transpile_lambda() {
        let result = transpile("|x| x + 1");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("|"));
    }
    
    // Test match expressions (complexity: 5 each)
    #[test]
    fn test_transpile_match() {
        let result = transpile("match x { 1 => true, _ => false }");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("match"));
    }
    
    // Test complex expressions (complexity: 5 each)
    #[test]
    fn test_transpile_nested_expr() {
        let result = transpile("(1 + 2) * 3");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("("));
        assert!(rust_code.contains(")"));
        assert!(rust_code.contains("*"));
    }
    
    #[test]
    fn test_transpile_function_call() {
        let result = transpile("print(42)");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("print"));
        assert!(rust_code.contains("("));
        assert!(rust_code.contains(")"));
    }
    
    #[test]
    fn test_transpile_member_access() {
        let result = transpile("obj.field");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("."));
    }
    
    #[test]
    fn test_transpile_index_access() {
        let result = transpile("arr[0]");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains("["));
        assert!(rust_code.contains("]"));
    }
    
    // Test string operations (complexity: 4 each)
    #[test]
    fn test_transpile_string_concat() {
        let result = transpile(r#""hello" + " world""#);
        assert!(result.is_ok());
        // String concatenation might use format! or +
    }
    
    // Test ranges (complexity: 4 each)
    #[test]
    fn test_transpile_range() {
        let result = transpile("0..10");
        assert!(result.is_ok());
        let rust_code = result.unwrap();
        assert!(rust_code.contains(".."));
    }
}