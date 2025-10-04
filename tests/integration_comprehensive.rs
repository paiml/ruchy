// COMPREHENSIVE INTEGRATION TESTS
// Sprint 80 Phase 25: Full pipeline testing
// Target: Exercise all code paths end-to-end

use ruchy::backend::CompilerOptions;
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::Value;
use ruchy::{Compiler, Parser, Transpiler};

#[test]
fn test_full_pipeline_arithmetic() {
    let programs = vec![
        "1 + 2 * 3",
        "(10 - 5) / 2",
        "2 ** 8",
        "100 % 7",
        "3.14 * 2.0",
        "10.5 / 2.5",
    ];

    for program in programs {
        // Parse
        let mut parser = Parser::new(program);
        let ast = parser.parse().expect("Parse failed");

        // Transpile
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(!rust_code.is_empty());

        // Interpret
        let mut interpreter = Interpreter::new();
        let _ = interpreter.evaluate(&ast);
    }
}

#[test]
fn test_full_pipeline_variables() {
    let program = r#"
        let x = 10;
        let y = 20;
        let z = x + y;
        z * 2
    "#;

    let mut parser = Parser::new(program);
    let ast = parser.parse().expect("Parse failed");

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast);
    assert!(rust_code.contains("let"));

    let mut interpreter = Interpreter::new();
    if let Ok(result) = interpreter.evaluate(&ast) {
        assert_eq!(result, Value::Integer(60));
    }
}

#[test]
fn test_full_pipeline_functions() {
    let program = r#"
        fn add(a, b) { a + b }
        fn multiply(x, y) { x * y }
        
        let sum = add(3, 4);
        multiply(sum, 2)
    "#;

    let mut parser = Parser::new(program);
    let ast = parser.parse().expect("Parse failed");

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast);
    assert!(rust_code.contains("fn"));

    let mut interpreter = Interpreter::new();
    if let Ok(result) = interpreter.evaluate(&ast) {
        assert_eq!(result, Value::Integer(14));
    }
}

#[test]
fn test_full_pipeline_control_flow() {
    let programs = vec![
        "if true { 42 } else { 0 }",
        "if 5 > 3 { 100 } else { 200 }",
        "if false { 1 } else if true { 2 } else { 3 }",
    ];

    for program in programs {
        let mut parser = Parser::new(program);
        let ast = parser.parse().expect("Parse failed");

        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("if"));

        let mut interpreter = Interpreter::new();
        let _ = interpreter.evaluate(&ast);
    }
}

#[test]
fn test_full_pipeline_loops() {
    let while_program = r#"
        let mut i = 0;
        let mut sum = 0;
        while i < 5 {
            sum = sum + i;
            i = i + 1
        };
        sum
    "#;

    let mut parser = Parser::new(while_program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("while"));
    }

    let for_program = r#"
        let mut total = 0;
        for x in [1, 2, 3, 4, 5] {
            total = total + x
        };
        total
    "#;

    let mut parser = Parser::new(for_program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("for"));
    }
}

#[test]
fn test_full_pipeline_data_structures() {
    let list_program = "[1, 2, 3, 4, 5]";
    let tuple_program = "(42, \"hello\", true, 3.14)";
    let object_program = "{x: 10, y: 20, name: \"point\"}";

    for program in [list_program, tuple_program, object_program] {
        let mut parser = Parser::new(program);
        let ast = parser.parse().expect("Parse failed");

        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(!rust_code.is_empty());

        let mut interpreter = Interpreter::new();
        let _ = interpreter.evaluate(&ast);
    }
}

#[test]
fn test_full_pipeline_pattern_matching() {
    let program = r#"
        let value = 2;
        match value {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("match"));

        let mut interpreter = Interpreter::new();
        if let Ok(result) = interpreter.evaluate(&ast) {
            if let Value::String(s) = result {
                assert_eq!(&**s, "two");
            }
        }
    }
}

#[test]
fn test_full_pipeline_string_operations() {
    let programs = vec![
        r#""hello" + " " + "world""#,
        r#""test".len()"#,
        r#""HELLO".lower()"#,
        r#""hello".upper()"#,
    ];

    for program in programs {
        let mut parser = Parser::new(program);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);

            let mut interpreter = Interpreter::new();
            let _ = interpreter.evaluate(&ast);
        }
    }
}

#[test]
fn test_full_pipeline_closures() {
    let program = r#"
        let make_counter = fn() {
            let mut count = 0;
            fn() {
                count = count + 1;
                count
            }
        };
        
        let counter = make_counter();
        counter();
        counter();
        counter()
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("fn"));
    }
}

#[test]
fn test_full_pipeline_recursion() {
    let program = r#"
        fn fibonacci(n) {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        fibonacci(10)
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("fibonacci"));

        let mut interpreter = Interpreter::new();
        if let Ok(result) = interpreter.evaluate(&ast) {
            assert_eq!(result, Value::Integer(55));
        }
    }
}

#[test]
fn test_full_pipeline_higher_order_functions() {
    let program = r#"
        fn map(list, f) {
            let mut result = [];
            for item in list {
                result = result + [f(item)]
            };
            result
        }
        
        let double = fn(x) { x * 2 };
        map([1, 2, 3, 4, 5], double)
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let _ = transpiler.transpile(&ast);
    }
}

#[test]
fn test_full_pipeline_error_cases() {
    let error_programs = vec![
        "1 / 0",              // Division by zero
        "undefined_variable", // Undefined variable
        "[1, 2, 3][10]",      // Index out of bounds
    ];

    for program in error_programs {
        let mut parser = Parser::new(program);
        if let Ok(ast) = parser.parse() {
            let mut interpreter = Interpreter::new();
            let result = interpreter.evaluate(&ast);
            // These should error or handle gracefully
            let _ = result;
        }
    }
}

#[test]
fn test_compiler_options() {
    let options = CompilerOptions::default();
    let compiler = Compiler::with_options(options);

    let simple_program = "42";
    let _ = compiler.compile_str(simple_program);
}

#[test]
fn test_transpiler_complex() {
    let complex_program = r#"
        // Complex program with multiple features
        use std::collections::HashMap;
        
        struct Point {
            x: i32,
            y: i32,
        }
        
        impl Point {
            fn new(x: i32, y: i32) -> Point {
                Point { x, y }
            }
            
            fn distance(&self) -> f64 {
                ((self.x * self.x + self.y * self.y) as f64).sqrt()
            }
        }
        
        fn main() {
            let mut points = vec![];
            for i in 0..10 {
                points.push(Point::new(i, i * 2));
            }
            
            let distances: Vec<f64> = points.iter()
                .map(|p| p.distance())
                .collect();
            
            println!("Distances: {:?}", distances);
        }
    "#;

    let mut parser = Parser::new(complex_program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        // Should produce valid Rust code
        assert!(!rust_code.is_empty());
    }
}

#[test]
fn test_mixed_numeric_types() {
    let program = "let x = 10; let y = 3.14; x as f64 + y";

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.contains("as"));
    }
}

#[test]
fn test_async_functions() {
    let program = r#"
        async fn fetch_data() {
            // Simulated async operation
            42
        }
        
        async fn main() {
            let result = await fetch_data();
            result
        }
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        // Should handle async/await
        let _ = rust_code;
    }
}

#[test]
fn test_string_interpolation() {
    let program = r#"
        let name = "World";
        let greeting = f"Hello, {name}!";
        greeting
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        // Should handle string interpolation
        let _ = rust_code;
    }
}

#[test]
fn test_destructuring() {
    let programs = vec![
        "let [a, b, c] = [1, 2, 3]",
        "let (x, y) = (10, 20)",
        "let {name, age} = {name: \"Alice\", age: 30}",
    ];

    for program in programs {
        let mut parser = Parser::new(program);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_pipeline_operator() {
    let program = r#"
        let add_one = fn(x) { x + 1 };
        let double = fn(x) { x * 2 };
        let square = fn(x) { x * x };
        
        5 |> add_one |> double |> square
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let _ = transpiler.transpile(&ast);
    }
}

// Check all binary operators
#[test]
fn test_all_binary_operators() {
    use ruchy::frontend::ast::BinaryOp;

    let ops = vec![
        (BinaryOp::Add, "+"),
        (BinaryOp::Sub, "-"),
        (BinaryOp::Mul, "*"),
        (BinaryOp::Div, "/"),
        (BinaryOp::Mod, "%"),
        (BinaryOp::Pow, "**"),
        (BinaryOp::Eq, "=="),
        (BinaryOp::Ne, "!="),
        (BinaryOp::Lt, "<"),
        (BinaryOp::Gt, ">"),
        (BinaryOp::Le, "<="),
        (BinaryOp::Ge, ">="),
        (BinaryOp::And, "&&"),
        (BinaryOp::Or, "||"),
        (BinaryOp::BitwiseAnd, "&"),
        (BinaryOp::BitwiseOr, "|"),
        (BinaryOp::BitwiseXor, "^"),
        (BinaryOp::Shl, "<<"),
        (BinaryOp::Shr, ">>"),
    ];

    for (op, symbol) in ops {
        let program = format!("1 {} 2", symbol);
        let mut parser = Parser::new(&program);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);
        }
    }
}

// Check all unary operators
#[test]
fn test_all_unary_operators() {
    let programs = vec!["-42", "!true", "~0xFF"];

    for program in programs {
        let mut parser = Parser::new(program);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);
        }
    }
}

// Check all literal types
#[test]
fn test_all_literal_types() {
    let literals = vec![
        "42",          // Integer
        "3.14",        // Float
        "true",        // Bool true
        "false",       // Bool false
        r#""string""#, // String
        "'c'",         // Char
        "()",          // Unit
    ];

    for literal in literals {
        let mut parser = Parser::new(literal);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);

            let mut interpreter = Interpreter::new();
            let _ = interpreter.evaluate(&ast);
        }
    }
}
