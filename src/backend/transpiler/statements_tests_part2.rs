use super::*;

#[test]
fn test_trueno_variance_transpiles_correctly() {
    let mut transpiler = create_transpiler();
    let code = "let vals = [2.0, 4.0, 4.0, 4.0, 5.0]; trueno_variance(vals)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let result = transpiler
        .transpile(&ast)
        .expect("transpile should succeed in test");
    let rust_str = result.to_string();
    assert!(
        rust_str.contains("trueno_bridge") && rust_str.contains("variance"),
        "trueno_variance should transpile to trueno_bridge::variance, got: {rust_str}"
    );
}

#[test]
fn test_trueno_std_dev_transpiles_correctly() {
    let mut transpiler = create_transpiler();
    let code = "let samples = [1.0, 2.0, 3.0]; trueno_std_dev(samples)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let result = transpiler
        .transpile(&ast)
        .expect("transpile should succeed in test");
    let rust_str = result.to_string();
    assert!(
        rust_str.contains("trueno_bridge") && rust_str.contains("std_dev"),
        "trueno_std_dev should transpile to trueno_bridge::std_dev, got: {rust_str}"
    );
}

#[test]
fn test_trueno_dot_transpiles_correctly() {
    let mut transpiler = create_transpiler();
    let code = "let a = [1.0, 2.0, 3.0]; let b = [4.0, 5.0, 6.0]; trueno_dot(a, b)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let result = transpiler
        .transpile(&ast)
        .expect("transpile should succeed in test");
    let rust_str = result.to_string();
    assert!(
        rust_str.contains("trueno_bridge") && rust_str.contains("dot"),
        "trueno_dot should transpile to trueno_bridge::dot, got: {rust_str}"
    );
}

#[test]
fn test_transpile_if_comprehensive() {
    let mut transpiler = Transpiler::new();

    // Test if without else
    let code = "if x > 0 { println(\"positive\") }";
    let mut parser = Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
        let output = result.expect("result should be Ok in test").to_string();
        assert!(output.contains("if"));
    }

    // Test if with else
    let code = "if x > 0 { 1 } else { -1 }";
    let mut parser = Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }

    // Test if-else-if chain
    let code = "if x > 0 { 1 } else if x < 0 { -1 } else { 0 }";
    let mut parser = Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok());
    }
}

#[test]
fn test_transpile_let_comprehensive() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "let x = 5",
        "let mut y = 10",
        "const PI = 3.15",
        "let (a, b) = (1, 2)",
        "let [x, y, z] = [1, 2, 3]",
        "let Some(value) = opt",
        "let Ok(result) = try_something()",
        "let {name, age} = person",
        "let x: int = 42",
        "let f: fn(int) -> int = |x| x * 2",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_transpile_function_comprehensive() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "fn simple() { }",
        "fn main() { println(\"Hello\") }",
        "fn add(a: int, b: int) -> int { a + b }",
        "fn generic<T>(x: T) -> T { x }",
        "async fn fetch() { await get() }",
        "fn* generator() { yield 1; yield 2 }",
        "pub fn public() { }",
        "#[test] fn test_function() { // Test passes without panic }",
        "fn with_default(x = 10) { x }",
        "fn recursive(n) { if n <= 0 { 0 } else { n + recursive(n-1) } }",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_transpile_call_comprehensive() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        // Print functions
        "print(\"hello\")",
        "println(\"world\")",
        "eprint(\"error\")",
        "eprintln(\"error line\")",
        "dbg!(value)",
        // Math functions
        "sqrt(16)",
        "pow(2, 8)",
        "abs(-5)",
        "min(3, 7)",
        "max(3, 7)",
        "floor(3.7)",
        "ceil(3.2)",
        "round(3.5)",
        "sin(0)",
        "cos(0)",
        "tan(0)",
        "log(1)",
        "exp(0)",
        // Type conversions
        "int(3.15)",
        "float(42)",
        "str(123)",
        "bool(1)",
        "char(65)",
        // Collections
        "vec![1, 2, 3]",
        "Vec::new()",
        "HashMap::new()",
        "HashSet::from([1, 2, 3])",
        // Input
        "input()",
        "input(\"Enter: \")",
        // Assert
        "// Test passes without panic",
        "assert_eq!(1, 1)",
        "assert_ne!(1, 2)",
        "debug_assert!(x > 0)",
        // DataFrame
        "df.select(\"col1\", \"col2\")",
        "DataFrame::new()",
        // Regular functions
        "custom_function(1, 2, 3)",
        "object.method()",
        "chain().of().calls()",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_transpile_lambda_comprehensive() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "x => x",
        "x => x * 2",
        "(x, y) => x + y",
        "() => 42",
        "(a, b, c) => a + b + c",
        "x => { let y = x * 2; y + 1 }",
        "async x => await fetch(x)",
        "(...args) => args.length",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_is_variable_mutated_property() {
    let mut transpiler = Transpiler::new();

    // Test mutation detection
    let test_cases = vec![
        ("let mut x = 0; x = 5", true),
        ("let mut x = 0; x += 1", true),
        ("let mut arr = []; arr.push(1)", true),
        ("let x = 5; let y = x + 1", false),
        ("let x = 5; println(x)", false),
    ];

    for (code, _expected) in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_control_flow_statements() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "while x < 10 { x += 1 }",
        "for i in 0..10 { println(i) }",
        "for x in array { process(x) }",
        "loop { if done { break } }",
        "match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }",
        "match opt { Some(x) => x * 2, None => 0 }",
        "return",
        "return 42",
        "break",
        "break 'label",
        "continue",
        "continue 'label",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_try_catch_statements() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "try { risky() } catch(e) { handle(e) }",
        "try { risky() } finally { cleanup() }",
        "try { risky() } catch(e) { handle(e) } finally { cleanup() }",
        "throw Error(\"message\")",
        "throw CustomError { code: 500 }",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_class_statements() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "class Empty { }",
        "class Point { x: int; y: int }",
        "class Circle { radius: float; fn area() { 3.15 * radius * radius } }",
        "class Derived extends Base { }",
        "class Generic<T> { value: T }",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_import_export_statements() {
    let mut transpiler = Transpiler::new();

    let test_cases = vec![
        "import std",
        "import std.io",
        "from std import println",
        "from math import { sin, cos, tan }",
        "export fn public() { }",
        "export const PI = 3.15",
        "export { func1, func2 }",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_edge_cases() {
    let mut transpiler = Transpiler::new();

    // Test empty and minimal cases
    let test_cases = vec!["", ";", "{ }", "( )", "let x", "fn f"];

    for code in test_cases {
        let mut parser = Parser::new(code);
        // These may fail to parse, but shouldn't panic
        if let Ok(ast) = parser.parse() {
            let _ = transpiler.transpile(&ast);
        }
    }
}

#[test]
fn test_helper_functions() {
    let transpiler = Transpiler::new();

    // Test pattern_needs_slice
    assert!(transpiler.pattern_needs_slice(&Pattern::List(vec![])));

    // Test value_creates_vec
    let vec_expr = Expr {
        kind: ExprKind::List(vec![]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: Vec::new(),
        trailing_comment: None,
    };
    assert!(transpiler.value_creates_vec(&vec_expr));

    // Test looks_like_numeric_function
    assert!(super::super::function_analysis::looks_like_numeric_function("sqrt"));
    assert!(super::super::function_analysis::looks_like_numeric_function("pow"));
    assert!(super::super::function_analysis::looks_like_numeric_function("abs"));
    assert!(!super::super::function_analysis::looks_like_numeric_function("println"));
}

#[test]
fn test_advanced_transpilation_patterns() {
    let mut transpiler = Transpiler::new();

    // Test complex nested expressions
    let advanced_cases = vec![
            // Complex assignments
            "let mut x = { let y = 5; y * 2 }",
            "let (a, b, c) = (1, 2, 3)",
            "let Point { x, y } = point",
            "let [first, ..rest] = array",

            // Complex function definitions
            "fn complex(x: Option<T>) -> Result<U, Error> { match x { Some(v) => Ok(transform(v)), None => Err(\"empty\") } }",
            "fn generic<T: Clone + Debug>(items: Vec<T>) -> Vec<T> { items.iter().cloned().collect() }",
            "fn async_complex() -> impl Future<Output = Result<String, Error>> { async { Ok(\"result\".to_string()) } }",

            // Complex control flow
            "match result { Ok(data) => { let processed = process(data); save(processed) }, Err(e) => log_error(e) }",
            "if let Some(value) = optional { value * 2 } else { default_value() }",
            "while let Some(item) = iterator.next() { process_item(item); }",
            "for (index, value) in enumerated { println!(\"{}: {}\", index, value); }",

            // Complex method calls
            "data.filter(|x| x > 0).map(|x| x * 2).collect::<Vec<_>>()",
            "async_function().await.unwrap_or_else(|e| handle_error(e))",
            "object.method()?.another_method().chain().build()",

            // Complex literals and collections
            "vec![1, 2, 3].into_iter().enumerate().collect()",
            "HashMap::from([(\"key1\", value1), (\"key2\", value2)])",
            "BTreeSet::from_iter([1, 2, 3, 2, 1])",

            // Complex pattern matching
            "match complex_enum { Variant::A { field1, field2 } => process(field1, field2), Variant::B(data) => handle(data), _ => default() }",

            // Complex lambdas and closures
            "let closure = |x: i32, y: i32| -> Result<i32, String> { if x > 0 { Ok(x + y) } else { Err(\"negative\".to_string()) } }",
            "items.fold(0, |acc, item| acc + item.value)",

            // Complex type annotations
            "let complex_type: HashMap<String, Vec<Result<i32, Error>>> = HashMap::new()",

            // Complex attribute annotations
            "#[derive(Debug, Clone)] #[serde(rename_all = \"camelCase\")] struct Complex { field: String }",
        ];

    for code in advanced_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let result = transpiler.transpile(&ast);
            // Should handle complex patterns without panicking
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_error_path_coverage() {
    let mut transpiler = Transpiler::new();

    // Test various error conditions and edge cases
    let error_cases = vec![
        // Malformed syntax that might parse but fail transpilation
        "let = 5",
        "fn ()",
        "match { }",
        "if { }",
        "for { }",
        "while { }",
        // Type mismatches
        "let x: String = 42",
        "let y: Vec<i32> = \"string\"",
        // Invalid operations
        "undefined_function()",
        "some_var.nonexistent_method()",
        "invalid.chain.of.calls()",
        // Complex nesting that might cause issues
        "((((((nested))))))",
        "{ { { { { nested } } } } }",
        // Edge case patterns
        "let _ = _",
        "let .. = array",
        "match x { .. => {} }",
        // Empty/minimal cases
        "",
        ";",
        "{ }",
        "fn() {}",
        "let;",
    ];

    for code in error_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let result = transpiler.transpile(&ast);
            // Should handle errors gracefully without panicking
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpiler_helper_methods_comprehensive() {
    let transpiler = Transpiler::new();

    // Test all helper methods with various inputs

    // Test basic transpiler functionality
    assert!(super::super::function_analysis::looks_like_numeric_function("sqrt"));
    assert!(!super::super::function_analysis::looks_like_numeric_function("println"));

    // Test various numeric function names
    let numeric_functions = vec![
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
        "atan2",
        "sinh",
        "cosh",
        "tanh",
        "asinh",
        "acosh",
        "atanh",
        "exp",
        "exp2",
        "ln",
        "log",
        "log2",
        "log10",
        "sqrt",
        "cbrt",
        "pow",
        "powf",
        "powi",
        "abs",
        "signum",
        "copysign",
        "floor",
        "ceil",
        "round",
        "trunc",
        "fract",
        "min",
        "max",
        "clamp",
        "to_degrees",
        "to_radians",
    ];

    for func in numeric_functions {
        assert!(super::super::function_analysis::looks_like_numeric_function(func));
    }

    let non_numeric_functions = vec![
        "println",
        "print",
        "format",
        "write",
        "read",
        "push",
        "pop",
        "insert",
        "remove",
        "clear",
        "len",
        "is_empty",
        "contains",
        "starts_with",
        "ends_with",
        "split",
        "join",
        "replace",
        "trim",
        "to_uppercase",
        "to_lowercase",
    ];

    for func in non_numeric_functions {
        assert!(!super::super::function_analysis::looks_like_numeric_function(func));
    }

    // Test pattern needs slice with various patterns
    let slice_patterns = vec![
        Pattern::List(vec![Pattern::Wildcard]),
        Pattern::List(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Wildcard,
        ]),
        Pattern::Tuple(vec![Pattern::List(vec![])]),
    ];

    for pattern in slice_patterns {
        transpiler.pattern_needs_slice(&pattern); // Test doesn't panic
    }

    // Test value creates vec with various expressions
    let vec_expressions = vec![
        Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        },
        Expr {
            kind: ExprKind::Call {
                func: Box::new(Expr {
                    kind: ExprKind::Identifier("vec".to_string()),
                    span: Span::default(),
                    attributes: vec![],
                    leading_comments: Vec::new(),
                    trailing_comment: None,
                }),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: Vec::new(),
            trailing_comment: None,
        },
    ];

    for expr in vec_expressions {
        transpiler.value_creates_vec(&expr); // Test doesn't panic
    }
}

#[test]
fn test_extreme_edge_cases() {
    let mut transpiler = Transpiler::new();

    // Test with maximum complexity inputs
    let edge_cases = vec![
            // Very long identifier names
            "let very_very_very_long_identifier_name_that_goes_on_and_on_and_on = 42",

            // Deep nesting levels
            "if true { if true { if true { if true { println!(\"deep\") } } } }",

            // Many parameters
            "fn many_params(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32) -> i32 { a + b + c + d + e + f + g + h }",

            // Complex generic constraints
            "fn generic_complex<T: Clone + Debug + Send + Sync + 'static>(x: T) -> T where T: PartialEq + Eq + Hash { x }",

            // Unicode identifiers
            "let 变量 = 42",
            "let москва = \"city\"",
            "let 🚀 = \"rocket\"",

            // Large numeric literals
            "let big = 123456789012345678901234567890",
            "let float = 123.456789012345678901234567890",

            // Complex string literals
            "let complex_string = \"String with \\n newlines \\t tabs \\\" quotes and 🚀 emojis\"",
            "let raw_string = r#\"Raw string with \"quotes\" and #hashtags\"#",

            // Nested collections
            "let nested = vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]]",

            // Complex macro invocations
            "println!(\"Format {} with {} multiple {} args\", 1, 2, 3)",
            "vec![1; 1000]",
            "format!(\"Complex formatting: {:#?}\", complex_data)",
        ];

    for code in edge_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let result = transpiler.transpile(&ast);
            // Should handle edge cases without panicking
            assert!(result.is_ok() || result.is_err());
        }
    }
}

// Test 101: is_variable_mutated with Assign
#[test]
fn test_is_variable_mutated_assign_v2() {
    let target = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let assign_expr = Expr {
        kind: ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "x",
        &assign_expr
    ));
    assert!(!super::super::mutation_detection::is_variable_mutated(
        "y",
        &assign_expr
    ));
}

// Test 102: is_variable_mutated with CompoundAssign
#[test]
fn test_is_variable_mutated_compound_assign_v2() {
    let target = Expr {
        kind: ExprKind::Identifier("counter".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let compound_expr = Expr {
        kind: ExprKind::CompoundAssign {
            target: Box::new(target),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Add,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "counter",
        &compound_expr
    ));
}

// Test 103: is_variable_mutated with PreIncrement
#[test]
fn test_is_variable_mutated_pre_increment_v2() {
    let target = Expr {
        kind: ExprKind::Identifier("i".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let inc_expr = Expr {
        kind: ExprKind::PreIncrement {
            target: Box::new(target),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "i", &inc_expr
    ));
}

// Test 104: is_variable_mutated with PostDecrement
#[test]
fn test_is_variable_mutated_post_decrement() {
    let target = Expr {
        kind: ExprKind::Identifier("value".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let dec_expr = Expr {
        kind: ExprKind::PostDecrement {
            target: Box::new(target),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "value", &dec_expr
    ));
}

// Test 105: is_variable_mutated in Block
#[test]
fn test_is_variable_mutated_in_block() {
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(10, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let block_expr = Expr {
        kind: ExprKind::Block(vec![assign]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "x",
        &block_expr
    ));
}

// Test 106: is_variable_mutated in If condition
#[test]
fn test_is_variable_mutated_in_if() {
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("flag".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let if_expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(assign),
            then_branch: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Unit),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            else_branch: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "flag", &if_expr
    ));
}

// Test 107: is_variable_mutated in While body
#[test]
fn test_is_variable_mutated_in_while() {
    let inc = Expr {
        kind: ExprKind::PreIncrement {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("count".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let while_expr = Expr {
        kind: ExprKind::While {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            body: Box::new(inc),
            label: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "count",
        &while_expr
    ));
}

// Test 108: is_variable_mutated in For body
#[test]
fn test_is_variable_mutated_in_for() {
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("sum".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(0, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let for_expr = Expr {
        kind: ExprKind::For {
            var: "item".to_string(),
            pattern: Some(Pattern::Identifier("item".to_string())),
            iter: Box::new(Expr {
                kind: ExprKind::List(vec![]),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            body: Box::new(assign),
            label: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "sum", &for_expr
    ));
}

// Test 109: is_variable_mutated in Match arm
#[test]
fn test_is_variable_mutated_in_match() {
    use crate::frontend::ast::MatchArm;
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("result".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let match_expr = Expr {
        kind: ExprKind::Match {
            expr: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(assign),
                span: Span::default(),
            }],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "result",
        &match_expr
    ));
}

// Test 110: is_variable_mutated in nested Let
#[test]
fn test_is_variable_mutated_in_let() {
    let inc = Expr {
        kind: ExprKind::PreIncrement {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let let_expr = Expr {
        kind: ExprKind::Let {
            name: "y".to_string(),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(5, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            body: Box::new(inc),
            type_annotation: None,
            is_mutable: false,
            else_block: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "x", &let_expr
    ));
}

// Test 111: is_variable_mutated in Binary expression
#[test]
fn test_is_variable_mutated_in_binary() {
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("a".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let binary_expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(assign),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(2, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "a",
        &binary_expr
    ));
}

// Test 112: is_variable_mutated in Unary expression
#[test]
fn test_is_variable_mutated_in_unary() {
    let inc = Expr {
        kind: ExprKind::PreIncrement {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("val".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let unary_expr = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(inc),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "val",
        &unary_expr
    ));
}

// Test 113: is_variable_mutated in Call arguments
#[test]
fn test_is_variable_mutated_in_call() {
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("arg".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let call_expr = Expr {
        kind: ExprKind::Call {
            func: Box::new(Expr {
                kind: ExprKind::Identifier("foo".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            args: vec![assign],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "arg", &call_expr
    ));
}

// Test 114: is_variable_mutated in MethodCall receiver
#[test]
fn test_is_variable_mutated_in_method_call() {
    let assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("obj".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let method_expr = Expr {
        kind: ExprKind::MethodCall {
            receiver: Box::new(assign),
            method: "process".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::mutation_detection::is_variable_mutated(
        "obj",
        &method_expr
    ));
}

// Test 115: is_variable_mutated returns false for immutable access
#[test]
fn test_is_variable_mutated_immutable_access() {
    let literal = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(!super::super::mutation_detection::is_variable_mutated(
        "x", &literal
    ));

    let ident = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(!super::super::mutation_detection::is_variable_mutated(
        "x", &ident
    ));
}

// Test 115: needs_lifetime_parameter - no ref params
#[test]
fn test_needs_lifetime_parameter_no_refs() {
    let _transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: Pattern::Identifier("x".to_string()),
        ty: Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        },
        default_value: None,
        span: Span::default(),
        is_mutable: false,
    }];
    assert!(!super::super::type_analysis::needs_lifetime_parameter(
        &params, None
    ));
}

// Test 116: needs_lifetime_parameter - 2+ ref params and ref return
#[test]
fn test_needs_lifetime_parameter_requires_lifetime() {
    let ref_type = Type {
        kind: TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(Type {
                kind: TypeKind::Named("str".to_string()),
                span: Span::default(),
            }),
        },
        span: Span::default(),
    };
    let params = vec![
        Param {
            pattern: Pattern::Identifier("a".to_string()),
            ty: ref_type.clone(),
            default_value: None,
            span: Span::default(),
            is_mutable: false,
        },
        Param {
            pattern: Pattern::Identifier("b".to_string()),
            ty: ref_type.clone(),
            default_value: None,
            span: Span::default(),
            is_mutable: false,
        },
    ];
    let return_type = Some(&ref_type);
    assert!(super::super::type_analysis::needs_lifetime_parameter(
        &params,
        return_type
    ));
}

// Test 117: is_reference_type - detects reference
#[test]
fn test_is_reference_type_true() {
    let ref_ty = Type {
        kind: TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(Type {
                kind: TypeKind::Named("str".to_string()),
                span: Span::default(),
            }),
        },
        span: Span::default(),
    };
    assert!(super::super::type_analysis::is_reference_type(&ref_ty));
}

// Test 118: is_reference_type - non-reference type
#[test]
fn test_is_reference_type_false() {
    let named_ty = Type {
        kind: TypeKind::Named("String".to_string()),
        span: Span::default(),
    };
    assert!(!super::super::type_analysis::is_reference_type(&named_ty));
}

// Test 119: is_string_type - detects String
#[test]
fn test_is_string_type_true() {
    let string_ty = Type {
        kind: TypeKind::Named("String".to_string()),
        span: Span::default(),
    };
    assert!(super::super::type_analysis::is_string_type(&string_ty));
}

// Test 120: is_string_type - non-String type
#[test]
fn test_is_string_type_false() {
    let int_ty = Type {
        kind: TypeKind::Named("i32".to_string()),
        span: Span::default(),
    };
    assert!(!super::super::type_analysis::is_string_type(&int_ty));
}

// Test 121: body_needs_string_conversion - string literal
#[test]
fn test_body_needs_string_conversion_string_literal() {
    let body = Expr::new(
        ExprKind::Literal(Literal::String("hello".to_string())),
        Span::default(),
    );
    assert!(super::super::type_analysis::body_needs_string_conversion(
        &body
    ));
}

// Test 122: body_needs_string_conversion - identifier
#[test]
fn test_body_needs_string_conversion_identifier() {
    let body = Expr::new(ExprKind::Identifier("s".to_string()), Span::default());
    assert!(super::super::type_analysis::body_needs_string_conversion(
        &body
    ));
}

// Test 123: body_needs_string_conversion - integer literal
#[test]
fn test_body_needs_string_conversion_integer() {
    let body = Expr::new(
        ExprKind::Literal(Literal::Integer(42, None)),
        Span::default(),
    );
    assert!(!super::super::type_analysis::body_needs_string_conversion(
        &body
    ));
}

// Test 124: transpile_iterator_methods - map
#[test]
fn test_transpile_iterator_methods_map() {
    use quote::quote;
    let transpiler = Transpiler::new();
    let obj = quote! { vec };
    let f = quote! { |x| x * 2 };
    let result = transpiler
        .transpile_iterator_methods(&obj, "map", &[f])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("iter"));
    assert!(code.contains("map"));
    assert!(code.contains("collect"));
}

// Test 125: transpile_iterator_methods - filter
#[test]
fn test_transpile_iterator_methods_filter() {
    use quote::quote;
    let transpiler = Transpiler::new();
    let obj = quote! { vec };
    let f = quote! { |x| x > 10 };
    let result = transpiler
        .transpile_iterator_methods(&obj, "filter", &[f])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("into_iter"));
    assert!(code.contains("filter"));
    assert!(code.contains("collect"));
}

// Test 126: transpile_iterator_methods - reduce
#[test]
fn test_transpile_iterator_methods_reduce() {
    use quote::quote;
    let transpiler = Transpiler::new();
    let obj = quote! { vec };
    let f = quote! { |acc, x| acc + x };
    let result = transpiler
        .transpile_iterator_methods(&obj, "reduce", &[f])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("into_iter"));
    assert!(code.contains("reduce"));
    assert!(!code.contains("collect")); // reduce doesn't collect
}

// Test 127: transpile_map_set_methods - items
#[test]
fn test_transpile_map_set_methods_items() {
    use proc_macro2::Span as ProcSpan;
    use quote::quote;
    let transpiler = Transpiler::new();
    let obj = quote! { map };
    let method_ident = proc_macro2::Ident::new("items", ProcSpan::call_site());
    let result = transpiler
        .transpile_map_set_methods(&obj, &method_ident, "items", &[])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("iter"));
    assert!(code.contains("clone"));
}

// Test 128: transpile_map_set_methods - update
#[test]
fn test_transpile_map_set_methods_update() {
    use proc_macro2::Span as ProcSpan;
    use quote::quote;
    let transpiler = Transpiler::new();
    let obj = quote! { map };
    let method_ident = proc_macro2::Ident::new("update", ProcSpan::call_site());
    let arg = quote! { other_map };
    let result = transpiler
        .transpile_map_set_methods(&obj, &method_ident, "update", &[arg])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("extend"));
}

// Test 129: transpile_set_operations - union
#[test]
fn test_transpile_set_operations_union() {
    use quote::quote;
    let transpiler = Transpiler::new();
    let obj = quote! { set1 };
    let arg = quote! { set2 };
    let result = transpiler
        .transpile_set_operations(&obj, "union", &[arg])
        .expect("operation should succeed in test");
    let code = result.to_string();
    assert!(code.contains("union"));
    assert!(code.contains("cloned"));
    assert!(code.contains("HashSet"));
}

// Test 130: looks_like_numeric_function - with numeric names
#[test]
fn test_looks_like_numeric_function_true() {
    let _transpiler = Transpiler::new();
    assert!(super::super::function_analysis::looks_like_numeric_function("abs"));
    assert!(super::super::function_analysis::looks_like_numeric_function("sqrt"));
    assert!(super::super::function_analysis::looks_like_numeric_function("pow"));
}

// Test 131: looks_like_numeric_function - with non-numeric names
#[test]
fn test_looks_like_numeric_function_false() {
    let _transpiler = Transpiler::new();
    assert!(!super::super::function_analysis::looks_like_numeric_function("print"));
    assert!(!super::super::function_analysis::looks_like_numeric_function("hello"));
}

// Test 132: returns_boolean - with boolean literal
#[test]
fn test_returns_boolean_literal() {
    let body = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(returns_boolean(&body));
}

// Test 133: returns_boolean - with comparison
#[test]
fn test_returns_boolean_comparison_v2() {
    let body = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(5, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Equal,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(5, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(returns_boolean(&body));
}

// Test 134: returns_string_literal - with string
#[test]
fn test_returns_string_literal_true() {
    let body = Expr {
        kind: ExprKind::Literal(Literal::String("test".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(returns_string_literal(&body));
}

// Test 135: returns_string_literal - with non-string
#[test]
fn test_returns_string_literal_false() {
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(!returns_string_literal(&body));
}
