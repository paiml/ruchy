//! TDD tests for refactored interpreter modules
//! Target: Improve coverage with complexity ≤10 per function

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    
    // Test 1: Value creation and type checking (complexity: 3)
    #[test]
    fn test_value_types() {
        use ruchy::runtime::interpreter::Value;
        
        let int_val = Value::Integer(42);
        assert_eq!(format!("{:?}", int_val), "Integer(42)");
        
        let str_val = Value::String("hello".to_string());
        assert_eq!(format!("{:?}", str_val), "String(\"hello\")");
        
        let bool_val = Value::Bool(true);
        assert_eq!(format!("{:?}", bool_val), "Boolean(true)");
    }
    
    // Test 2: List value operations (complexity: 4)
    #[test]
    fn test_list_operations() {
        use ruchy::runtime::interpreter::Value;
        
        let list = Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        
        if let Value::Array(items) = list {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(1));
            assert_eq!(items[2], Value::Integer(3));
        } else {
            panic!("Expected list");
        }
    }
    
    // Test 3: Tuple value operations (complexity: 4)
    #[test]
    fn test_tuple_operations() {
        use ruchy::runtime::interpreter::Value;
        use std::rc::Rc;
        
        let tuple = Value::Tuple(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        
        if let Value::Tuple(items) = tuple {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(1));
            assert_eq!(items[2], Value::Integer(3));
        } else {
            panic!("Expected tuple");
        }
    }
    
    // Test 4: Interpreter state management (complexity: 4)
    #[test]
    fn test_interpreter_state() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        
        let mut interpreter = Interpreter::new();
        
        // Set and get variables
        interpreter.set_variable("x".to_string(), Value::Integer(10));
        interpreter.set_variable("y".to_string(), Value::Integer(20));
        
        assert_eq!(interpreter.get_variable("x"), Some(&Value::Integer(10)));
        assert_eq!(interpreter.get_variable("y"), Some(&Value::Integer(20)));
        assert_eq!(interpreter.get_variable("z"), None);
    }
    
    // Test 5: Function value creation (complexity: 3)
    #[test]
    fn test_function_value() {
        use ruchy::runtime::interpreter::Value;
        use ruchy::frontend::ast::Expr;
        
        let params = vec!["x".to_string(), "y".to_string()];
        let body = Box::new(Expr::default());
        
        let func = Value::Function {
            params: params.clone(),
            body,
            closure: HashMap::new(),
        };
        
        if let Value::Function { params: p, .. } = func {
            assert_eq!(p.len(), 2);
            assert_eq!(p[0], "x");
            assert_eq!(p[1], "y");
        } else {
            panic!("Expected function");
        }
    }
    
    // Test 6: Closure capture (complexity: 5)
    #[test]
    fn test_closure_capture() {
        use ruchy::runtime::interpreter::Value;
        use ruchy::frontend::ast::Expr;
        
        let mut closure = HashMap::new();
        closure.insert("captured_var".to_string(), Value::Integer(42));
        
        let func = Value::Function {
            params: vec!["x".to_string()],
            body: Box::new(Expr::default()),
            closure,
        };
        
        if let Value::Function { closure: c, .. } = func {
            assert_eq!(c.len(), 1);
            assert_eq!(c.get("captured_var"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected function with closure");
        }
    }
    
    // Test 7: Range value creation (complexity: 3)
    #[test]
    fn test_range_value() {
        use ruchy::runtime::interpreter::Value;
        
        let range = Value::Range { start: 0, end: 10 };
        
        if let Value::Range { start, end } = range {
            assert_eq!(start, 0);
            assert_eq!(end, 10);
        } else {
            panic!("Expected range");
        }
    }
    
    // Test 8: Error value handling (complexity: 2)
    #[test]
    fn test_error_value() {
        use ruchy::runtime::interpreter::Value;
        
        let err = Value::Error("Division by zero".to_string());
        
        if let Value::Error(msg) = err {
            assert_eq!(msg, "Division by zero");
        } else {
            panic!("Expected error");
        }
    }
    
    // Test 9: DataFrame value structure (complexity: 5)
    #[test]
    fn test_dataframe_value() {
        use ruchy::runtime::interpreter::Value;
        
        let mut columns = HashMap::new();
        columns.insert("col1".to_string(), vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        columns.insert("col2".to_string(), vec![
            Value::String("a".to_string()),
            Value::String("b".to_string()),
            Value::String("c".to_string()),
        ]);
        
        let df = Value::DataFrame { columns };
        
        if let Value::DataFrame { columns: cols } = df {
            assert_eq!(cols.len(), 2);
            assert!(cols.contains_key("col1"));
            assert!(cols.contains_key("col2"));
        } else {
            panic!("Expected DataFrame");
        }
    }
    
    // Test 10: Interpreter arithmetic operations (complexity: 6)
    #[test]
    fn test_interpreter_arithmetic() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test addition: 10 + 20
        let mut parser = Parser::new("10 + 20");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(30));
        
        // Test multiplication: 5 * 6
        let mut parser = Parser::new("5 * 6");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(30));
    }
    
    // Test 11: Interpreter variable assignment (complexity: 5)
    #[test]
    fn test_interpreter_assignment() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Assign variable: x = 42
        let mut parser = Parser::new("x = 42");
        let expr = parser.parse_expr().unwrap();
        interpreter.eval_expr(&expr).unwrap();
        
        // Check variable value
        assert_eq!(interpreter.get_variable("x"), Some(&Value::Integer(42)));
    }
    
    // Test 12: Interpreter string concatenation (complexity: 5)
    #[test]
    fn test_interpreter_string_concat() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test string concatenation: "hello" + " world"
        let mut parser = Parser::new("\"hello\" + \" world\"");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }
    
    // Test 13: Interpreter boolean operations (complexity: 6)
    #[test]
    fn test_interpreter_boolean() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test AND: true && false
        let mut parser = Parser::new("true && false");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(false));
        
        // Test OR: true || false
        let mut parser = Parser::new("true || false");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
    
    // Test 14: Interpreter comparison operations (complexity: 6)
    #[test]
    fn test_interpreter_comparison() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test less than: 5 < 10
        let mut parser = Parser::new("5 < 10");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // Test equality: 42 == 42
        let mut parser = Parser::new("42 == 42");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
    
    // Test 15: Interpreter list creation (complexity: 5)
    #[test]
    fn test_interpreter_list() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Create list: [1, 2, 3]
        let mut parser = Parser::new("[1, 2, 3]");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        
        if let Value::Array(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Integer(1));
            assert_eq!(items[2], Value::Integer(3));
        } else {
            panic!("Expected list");
        }
    }
    
    // Test 16: Interpreter range creation (complexity: 4)
    #[test]
    fn test_interpreter_range() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Create range: 0..10
        let mut parser = Parser::new("0..10");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        
        if let Value::Range { start, end } = result {
            assert_eq!(start, 0);
            assert_eq!(end, 10);
        } else {
            panic!("Expected range");
        }
    }
    
    // Test 17: Interpreter if expression (complexity: 7)
    #[test]
    fn test_interpreter_if() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test if-then-else: if true { 1 } else { 2 }
        let mut parser = Parser::new("if true { 1 } else { 2 }");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(1));
        
        // Test with false condition
        let mut parser = Parser::new("if false { 1 } else { 2 }");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(2));
    }
    
    // Test 18: Interpreter block evaluation (complexity: 6)
    #[test]
    fn test_interpreter_block() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test block: { x = 10; y = 20; x + y }
        let code = "{ x = 10; y = 20; x + y }";
        let mut parser = Parser::new(code);
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(30));
    }
    
    // Test 19: Interpreter unary operations (complexity: 5)
    #[test]
    fn test_interpreter_unary() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test negation: -42
        let mut parser = Parser::new("-42");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(-42));
        
        // Test logical not: !true
        let mut parser = Parser::new("!true");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
    
    // Test 20: Interpreter nested expressions (complexity: 7)
    #[test]
    fn test_interpreter_nested() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test nested arithmetic: (10 + 20) * (5 - 3)
        let mut parser = Parser::new("(10 + 20) * (5 - 3)");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(60));
        
        // Test nested with variables
        let code = "{ x = 5; y = 10; (x + y) * 2 }";
        let mut parser = Parser::new(code);
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(30));
    }
    
    // Test 21: Module resolution (complexity: 4)
    #[test]
    fn test_module_resolution() {
        use ruchy::runtime::interpreter::Interpreter;
        
        let interpreter = Interpreter::new();
        
        // Test builtin modules
        assert!(interpreter.has_module("std"));
        assert!(interpreter.has_module("math"));
        assert!(!interpreter.has_module("nonexistent"));
    }
    
    // Test 22: Interpreter state reset (complexity: 4)
    #[test]
    fn test_interpreter_reset() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        
        let mut interpreter = Interpreter::new();
        
        // Add some state
        interpreter.set_variable("x".to_string(), Value::Integer(42));
        interpreter.set_variable("y".to_string(), Value::Integer(100));
        assert_eq!(interpreter.get_variable("x"), Some(&Value::Integer(42)));
        
        // Reset
        interpreter.reset();
        assert_eq!(interpreter.get_variable("x"), None);
        assert_eq!(interpreter.get_variable("y"), None);
    }
    
    // Test 23: Interpreter with custom context (complexity: 5)
    #[test]
    fn test_interpreter_context() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        
        let mut interpreter = Interpreter::new();
        
        // Create a new scope
        interpreter.push_scope();
        interpreter.set_variable("local".to_string(), Value::Integer(10));
        assert_eq!(interpreter.get_variable("local"), Some(&Value::Integer(10)));
        
        // Pop scope
        interpreter.pop_scope();
        assert_eq!(interpreter.get_variable("local"), None);
    }
    
    // Test 24: Interpreter error handling (complexity: 4)
    #[test]
    fn test_interpreter_error_handling() {
        use ruchy::runtime::interpreter::Interpreter;
        use ruchy::frontend::parser::Parser;
        
        let mut interpreter = Interpreter::new();
        
        // Test undefined variable
        let mut parser = Parser::new("undefined_var");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr);
        assert!(result.is_err());
        
        // Test division by zero
        let mut parser = Parser::new("10 / 0");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr);
        assert!(result.is_err());
    }
    
    // Test 25: Interpreter performance tracking (complexity: 6)
    #[test]
    fn test_interpreter_performance() {
        use ruchy::runtime::interpreter::{Interpreter, Value};
        use ruchy::frontend::parser::Parser;
        use std::time::Instant;
        
        let mut interpreter = Interpreter::new();
        
        // Measure evaluation time
        let start = Instant::now();
        let mut parser = Parser::new("1 + 2 + 3 + 4 + 5");
        let expr = parser.parse_expr().unwrap();
        let result = interpreter.eval_expr(&expr).unwrap();
        let duration = start.elapsed();
        
        assert_eq!(result, Value::Integer(15));
        assert!(duration.as_millis() < 100); // Should be fast
    }
}