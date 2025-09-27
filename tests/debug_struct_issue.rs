use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

#[test]
fn debug_single_expressions() {
    let mut interpreter = Interpreter::new();

    // First define the struct
    let struct_code = "struct Point { x: float, y: float }";
    let mut parser = Parser::new(struct_code);
    let expr = parser.parse().unwrap();
    let result = interpreter.eval_expr(&expr).unwrap();
    println!("Struct definition result: {:?}", result);
    assert!(matches!(result, Value::Object(_)));

    // Then instantiate it
    let instance_code = "Point { x: 3.0, y: 4.0 }";
    let mut parser = Parser::new(instance_code);
    let expr = parser.parse().unwrap();
    let result = interpreter.eval_expr(&expr).unwrap();
    println!("Instance result: {:?}", result);
    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn debug_function_execution() {
    let mut interpreter = Interpreter::new();

    // Define struct first
    let struct_code = "struct Point { x: float, y: float }";
    let mut parser = Parser::new(struct_code);
    let expr = parser.parse().unwrap();
    interpreter.eval_expr(&expr).unwrap();

    // Then define and call main function
    let main_code = r#"
        fn main() {
            let p = Point { x: 3.0, y: 4.0 }
            p
        }
        main()
    "#;
    let mut parser = Parser::new(main_code);
    let expr = parser.parse().unwrap();
    let result = interpreter.eval_expr(&expr).unwrap();
    println!("Main function result: {:?}", result);
    assert!(matches!(result, Value::Object(_)));
}
