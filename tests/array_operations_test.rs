use ruchy::{Parser, runtime::repl::ReplInterpreter};

#[test]
fn test_array_push() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let mut arr = [1, 2, 3];
        arr = arr.push(4);
        arr
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be [1, 2, 3, 4]
    assert_eq!(result.to_string(), "[1, 2, 3, 4]");
}

#[test]
fn test_array_pop() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let mut arr = [1, 2, 3];
        arr.pop()
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should return the popped element (3)
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_array_len() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let arr = [1, 2, 3, 4, 5];
        arr.len()
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    assert_eq!(result.to_string(), "5");
}

#[test]
fn test_array_insert() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let mut arr = [1, 2, 3];
        arr = arr.insert(1, 99);
        arr
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be [1, 99, 2, 3]
    assert_eq!(result.to_string(), "[1, 99, 2, 3]");
}

#[test]
fn test_array_remove() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let mut arr = [1, 2, 3, 4];
        arr.remove(1)
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should return the removed element (2)
    assert_eq!(result.to_string(), "2");
}

#[test]
fn test_array_append() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let mut arr1 = [1, 2, 3];
        let arr2 = [4, 5];
        arr1 = arr1.append(arr2);
        arr1
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be [1, 2, 3, 4, 5]
    assert_eq!(result.to_string(), "[1, 2, 3, 4, 5]");
}

#[test]
fn test_array_reverse() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let mut arr = [1, 2, 3];
        arr = arr.reverse();
        arr
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be [3, 2, 1]
    assert_eq!(result.to_string(), "[3, 2, 1]");
}

#[test]
fn test_array_first_last() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let arr = [10, 20, 30];
        let first = arr.first();
        let last = arr.last();
        [first, last]
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be [10, 30]
    assert_eq!(result.to_string(), "[10, 30]");
}

#[test]
fn test_array_sum() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let arr = [1, 2, 3, 4, 5];
        arr.sum()
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be 15
    assert_eq!(result.to_string(), "15");
}

#[test]
fn test_array_indexing() {
    let mut interpreter = ReplInterpreter::new();
    let code = r#"
        let arr = [10, 20, 30, 40];
        [arr[0], arr[2]]
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let result = interpreter.evaluate_expr(&ast, std::time::Instant::now() + std::time::Duration::from_secs(1), 0).unwrap();
    
    // Should be [10, 30]
    assert_eq!(result.to_string(), "[10, 30]");
}