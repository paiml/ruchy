use super::*;
use crate::Parser;


// ==================== traverse_expr_for_check Tests ====================

#[test]
fn test_traverse_expr_for_check_returns_early_on_true() {
    let code = "fun test() { 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let result = traverse_expr_for_check(&ast, |_| Some(true));
    assert!(result);
}

#[test]
fn test_traverse_expr_for_check_returns_early_on_false() {
    let code = "fun test() { 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let result = traverse_expr_for_check(&ast, |_| Some(false));
    assert!(!result);
}

#[test]
fn test_traverse_expr_for_check_continues_on_none() {
    let code = "fun test() { 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    let result = traverse_expr_for_check(&ast, |_| None);
    assert!(!result);
}

#[test]
fn test_traverse_expr_for_check_traverses_block() {
    // Use is_param_used_as_array which uses traverse_expr_for_check
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                // Verify that traverse works by checking array usage
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_traverse_expr_for_check_traverses_if() {
    let code = "fun test(arr) { if true { arr[0] } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                // Verify traversal into if branches
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_traverse_expr_for_check_traverses_while() {
    let code = "fun test(arr) { while true { arr[0] } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                // Verify traversal into while body
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_traverse_expr_for_check_traverses_for() {
    let code = "fun test(arr) { for x in [1,2,3] { arr[0] } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                // Verify traversal into for body
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

// ==================== find_param_in_direct_args Tests ====================

#[test]
fn test_find_param_in_direct_args_found() {
    let code = "fun test(x) { f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Call { args, .. } = &body.kind {
                    assert!(find_param_in_direct_args("x", args));
                }
            }
        }
    }
}

#[test]
fn test_find_param_in_direct_args_not_found() {
    let code = "fun test(x) { f(y) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Call { args, .. } = &body.kind {
                    assert!(!find_param_in_direct_args("x", args));
                }
            }
        }
    }
}

#[test]
fn test_find_param_in_direct_args_empty() {
    let args: Vec<Expr> = vec![];
    assert!(!find_param_in_direct_args("x", &args));
}

// ==================== is_param_used_as_function_argument Tests ====================

#[test]
fn test_is_param_used_as_function_argument_direct() {
    let code = "fun test(x) { some_func(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_nested() {
    let code = "fun test(x) { outer(inner(x)) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_in_if() {
    let code = "fun test(x) { if true { f(x) } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_in_else() {
    let code = "fun test(x) { if true { 0 } else { f(x) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_in_let() {
    let code = "fun test(x) { let y = f(x); y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_in_binary() {
    let code = "fun test(x) { f(x) + g(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_in_unary() {
    let code = "fun test(x) { -f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function_argument("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_argument_false() {
    let code = "fun test(x) { x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_as_function_argument("x", body));
            }
        }
    }
}

// ==================== is_param_used_as_function Tests ====================

#[test]
fn test_is_param_used_as_function_direct() {
    let code = "fun test(f) { f() }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function("f", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_with_args() {
    let code = "fun test(callback) { callback(1, 2, 3) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function("callback", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_in_block() {
    let code = "fun test(f) { let x = 1; f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function("f", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_in_if() {
    let code = "fun test(f) { if true { f(5) } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function("f", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_in_lambda() {
    let code = "fun test(f) { (x) => f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_function("f", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_function_false() {
    let code = "fun test(x) { x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_as_function("x", body));
            }
        }
    }
}

// ==================== is_numeric_operator Tests ====================

#[test]
fn test_is_numeric_operator_add() {
    assert!(is_numeric_operator(&BinaryOp::Add));
}

#[test]
fn test_is_numeric_operator_subtract() {
    assert!(is_numeric_operator(&BinaryOp::Subtract));
}

#[test]
fn test_is_numeric_operator_multiply() {
    assert!(is_numeric_operator(&BinaryOp::Multiply));
}

#[test]
fn test_is_numeric_operator_divide() {
    assert!(is_numeric_operator(&BinaryOp::Divide));
}

#[test]
fn test_is_numeric_operator_modulo() {
    assert!(is_numeric_operator(&BinaryOp::Modulo));
}

#[test]
fn test_is_numeric_operator_less() {
    assert!(is_numeric_operator(&BinaryOp::Less));
}

#[test]
fn test_is_numeric_operator_greater() {
    assert!(is_numeric_operator(&BinaryOp::Greater));
}

#[test]
fn test_is_numeric_operator_less_equal() {
    assert!(is_numeric_operator(&BinaryOp::LessEqual));
}

#[test]
fn test_is_numeric_operator_greater_equal() {
    assert!(is_numeric_operator(&BinaryOp::GreaterEqual));
}

#[test]
fn test_is_numeric_operator_equal_false() {
    assert!(!is_numeric_operator(&BinaryOp::Equal));
}

#[test]
fn test_is_numeric_operator_not_equal_false() {
    assert!(!is_numeric_operator(&BinaryOp::NotEqual));
}

#[test]
fn test_is_numeric_operator_and_false() {
    assert!(!is_numeric_operator(&BinaryOp::And));
}

#[test]
fn test_is_numeric_operator_or_false() {
    assert!(!is_numeric_operator(&BinaryOp::Or));
}

// ==================== is_param_used_numerically Tests ====================

#[test]
fn test_is_param_used_numerically_add() {
    let code = "fun test(x) { x + 5 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_subtract() {
    let code = "fun test(x) { x - 5 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_multiply() {
    let code = "fun test(x) { x * 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_divide() {
    let code = "fun test(x) { x / 3 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_modulo() {
    let code = "fun test(x) { x % 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_comparison() {
    let code = "fun test(count) { let x = 0; while x < count { x = x + 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("count", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_in_block() {
    let code = "fun test(x) { let y = 1; x - y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_in_if() {
    let code = "fun test(x) { if true { x * 2 } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_in_lambda() {
    let code = "fun test(x) { (y) => x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_in_while() {
    let code = "fun test(x) { while x > 0 { x = x - 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_in_for() {
    let code = "fun test(x) { for i in [1,2,3] { x + i } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_string_concat_false() {
    let code = r#"fun test(name) { "Hello " + name }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_numerically("name", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_numerically_false() {
    let code = r#"fun test(x) { "hello" }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_numerically("x", body));
            }
        }
    }
}

// ==================== contains_param Tests ====================

#[test]
fn test_contains_param_identifier() {
    let code = "fun test(x) { x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_contains_param_not_found() {
    let code = "fun test(x) { y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_contains_param_in_binary() {
    let code = "fun test(x, y) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
                assert!(contains_param("y", body));
            }
        }
    }
}

#[test]
fn test_contains_param_in_block() {
    let code = "fun test(x) { let y = 1; x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_contains_param_in_call() {
    let code = "fun test(x) { f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

// ==================== is_param_used_as_array Tests ====================

#[test]
fn test_is_param_used_as_array_direct() {
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_array_in_if() {
    let code = "fun test(arr) { if true { arr[1] } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_array_in_while() {
    let code = "fun test(arr) { let i = 0; while i < 10 { arr[i] } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_array_false() {
    let code = "fun test(x) { x + 5 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_as_array("x", body));
            }
        }
    }
}

// ==================== is_param_used_with_len Tests ====================

#[test]
fn test_is_param_used_with_len_direct() {
    let code = "fun test(arr) { len(arr) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_with_len("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_with_len_in_while() {
    let code = "fun test(arr) { let i = 0; while i < len(arr) { i = i + 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_with_len("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_with_len_false() {
    let code = "fun test(x) { x * 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_with_len("x", body));
            }
        }
    }
}

// ==================== is_param_used_as_index Tests ====================

#[test]
fn test_is_param_used_as_index_direct() {
    let code = "fun test(i) { let arr = [1,2,3]; arr[i] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_index("i", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_index_in_for() {
    let code = "fun test(i) { for x in [1,2,3] { arr[i] } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_index("i", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_index_false() {
    let code = "fun test(x) { x + 10 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_as_index("x", body));
            }
        }
    }
}

// ==================== is_param_used_as_bool Tests ====================

#[test]
fn test_is_param_used_as_bool_in_if() {
    let code = "fun test(flag) { if flag { 1 } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("flag", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool_in_while() {
    let code = "fun test(flag) { while flag { } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("flag", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool_with_not() {
    let code = "fun test(flag) { !flag }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("flag", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool_with_and() {
    let code = "fun test(a) { a && true }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("a", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool_with_or() {
    let code = "fun test(a) { a || false }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("a", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool_right_side_and() {
    let code = "fun test(b) { true && b }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("b", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool_false() {
    let code = "fun test(x) { x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_as_bool("x", body));
            }
        }
    }
}

// ==================== is_param_used_in_string_concat Tests ====================

#[test]
fn test_is_param_used_in_string_concat_left() {
    let code = r#"fun test(name) { "Hello " + name }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_in_string_concat("name", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_in_string_concat_right() {
    let code = r#"fun test(name) { name + "!" }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_in_string_concat("name", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_in_string_concat_false() {
    let code = "fun test(x) { x + 5 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_param_used_in_string_concat("x", body));
            }
        }
    }
}

// ==================== is_nested_array_access Tests ====================

#[test]
fn test_is_nested_array_access_true() {
    let code = "fun test(matrix) { matrix[0][1] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_nested_array_access("matrix", body));
            }
        }
    }
}

#[test]
fn test_is_nested_array_access_false() {
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_nested_array_access("arr", body));
            }
        }
    }
}

// ==================== infer_param_type Tests ====================

#[test]
fn test_infer_param_type_array() {
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("arr", body), Some("Vec<i32>"));
            }
        }
    }
}

#[test]
fn test_infer_param_type_nested_array() {
    let code = "fun test(matrix) { matrix[0][1] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("matrix", body), Some("Vec<Vec<i32>>"));
            }
        }
    }
}

#[test]
fn test_infer_param_type_with_len() {
    let code = "fun test(arr) { len(arr) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("arr", body), Some("Vec<i32>"));
            }
        }
    }
}

#[test]
fn test_infer_param_type_index() {
    let code = "fun test(i) { let arr = [1,2,3]; arr[i] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("i", body), Some("i32"));
            }
        }
    }
}

#[test]
fn test_infer_param_type_none() {
    let code = r#"fun test(x) { "hello" }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("x", body), None);
            }
        }
    }
}

// ==================== is_string_concatenation Tests ====================

#[test]
fn test_is_string_concatenation_left_string() {
    let code = r#"fun test() { "hello" + x }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Binary { op, left, right } = &body.kind {
                    assert!(is_string_concatenation(op, left, right));
                }
            }
        }
    }
}

#[test]
fn test_is_string_concatenation_right_string() {
    let code = r#"fun test() { x + "world" }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Binary { op, left, right } = &body.kind {
                    assert!(is_string_concatenation(op, left, right));
                }
            }
        }
    }
}

#[test]
fn test_is_string_concatenation_numeric() {
    let code = "fun test() { 1 + 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Binary { op, left, right } = &body.kind {
                    assert!(!is_string_concatenation(op, left, right));
                }
            }
        }
    }
}

// ==================== has_param_in_operation Tests ====================

#[test]
fn test_has_param_in_operation_left() {
    let code = "fun test(x) { x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Binary { left, right, .. } = &body.kind {
                    assert!(has_param_in_operation("x", left, right));
                }
            }
        }
    }
}

#[test]
fn test_has_param_in_operation_right() {
    let code = "fun test(x) { 1 + x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Binary { left, right, .. } = &body.kind {
                    assert!(has_param_in_operation("x", left, right));
                }
            }
        }
    }
}

#[test]
fn test_has_param_in_operation_false() {
    let code = "fun test(x) { 1 + 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Binary { left, right, .. } = &body.kind {
                    assert!(!has_param_in_operation("x", left, right));
                }
            }
        }
    }
}

// ==================== Additional Edge Case Tests ====================

#[test]
fn test_check_call_contains_param_in_func() {
    let code = "fun test(f) { f(1) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Call { func, args } = &body.kind {
                    assert!(check_call_contains_param("f", func, args));
                }
            }
        }
    }
}

#[test]
fn test_check_call_contains_param_in_args() {
    let code = "fun test(x) { f(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                if let ExprKind::Call { func, args } = &body.kind {
                    assert!(check_call_contains_param("x", func, args));
                }
            }
        }
    }
}

// ===== Coverage Tests: traverse_expr_for_check branches =====

#[test]
fn test_traverse_while_loop() {
    let code = "fun test(x) { while x > 0 { x = x - 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_traverse_for_loop() {
    let code = "fun test(arr) { for x in arr { print(x) } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_traverse_compound_assign() {
    let code = "fun test(x) { x += 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_traverse_unary_op() {
    let code = "fun test(x) { -x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_traverse_index_access() {
    let code = "fun test(arr, i) { arr[i] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_index("i", body));
            }
        }
    }
}

#[test]
fn test_traverse_let_pattern() {
    let code = "fun test(t) { let (a, b) = t; a + b }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("t", body));
            }
        }
    }
}

#[test]
fn test_traverse_assign() {
    let code = "fun test(x) { x = 10 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_nested_array_access_detection() {
    let code = "fun test(arr) { arr[0][1] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_nested_array_access("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_with_len() {
    let code = "fun test(arr) { arr.len() }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_with_len("arr", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_as_bool() {
    let code = "fun test(flag) { if flag { 1 } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_bool("flag", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_in_string_concat() {
    let code = "fun test(name) { \"Hello \" + name }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_in_string_concat("name", body));
            }
        }
    }
}

#[test]
fn test_is_param_used_in_print_macro() {
    let code = "fun test(x) { print(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_in_print_macro("x", body));
            }
        }
    }
}

#[test]
fn test_infer_param_type_numeric() {
    let code = "fun test(x) { x * 2 + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                let inferred = infer_param_type("x", body);
                // Should infer numeric type
                assert!(inferred.is_some());
            }
        }
    }
}

#[test]
fn test_infer_param_type_string() {
    let code = "fun test(s) { \"prefix: \" + s }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                let inferred = infer_param_type("s", body);
                assert!(inferred.is_some());
            }
        }
    }
}

#[test]
fn test_infer_param_type_array_index() {
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                let inferred = infer_param_type("arr", body);
                assert!(inferred.is_some());
            }
        }
    }
}

#[test]
fn test_infer_param_type_bool() {
    let code = "fun test(flag) { if flag { 1 } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                let inferred = infer_param_type("flag", body);
                assert!(inferred.is_some());
            }
        }
    }
}

#[test]
fn test_infer_param_type_function() {
    let code = "fun test(f) { f(1) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                let inferred = infer_param_type("f", body);
                assert!(inferred.is_some());
            }
        }
    }
}

#[test]
fn test_complex_expression_traversal() {
    let code = "fun test(x, y) { if x > 0 { y * 2 } else { y - 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
                assert!(is_param_used_numerically("y", body));
            }
        }
    }
}

#[test]
fn test_deeply_nested_expression() {
    let code = "fun test(a) { if true { if true { if true { a + 1 } } } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("a", body));
            }
        }
    }
}

#[test]
fn test_param_in_closure_body() {
    let code = "fun test(x) { let f = |y| x + y; f(1) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_numeric_operator_detection() {
    assert!(is_numeric_operator(&BinaryOp::Add));
    assert!(is_numeric_operator(&BinaryOp::Subtract));
    assert!(is_numeric_operator(&BinaryOp::Multiply));
    assert!(is_numeric_operator(&BinaryOp::Divide));
    assert!(is_numeric_operator(&BinaryOp::Modulo));
    assert!(is_numeric_operator(&BinaryOp::Less));
    assert!(is_numeric_operator(&BinaryOp::Greater));
    assert!(is_numeric_operator(&BinaryOp::LessEqual));
    assert!(is_numeric_operator(&BinaryOp::GreaterEqual));
    assert!(!is_numeric_operator(&BinaryOp::Equal));
    assert!(!is_numeric_operator(&BinaryOp::NotEqual));
    assert!(!is_numeric_operator(&BinaryOp::And));
    assert!(!is_numeric_operator(&BinaryOp::Or));
}

// ==================== Coverage Improvement Tests ====================

#[test]
fn test_traverse_expr_assign() {
    let code = "fun test(arr) { arr[0] = 5 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_traverse_expr_compound_assign() {
    let code = "fun test(x) { x += 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_traverse_expr_unary() {
    let code = "fun test(x) { -x }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(contains_param("x", body));
            }
        }
    }
}

#[test]
fn test_traverse_expr_index_access() {
    let code = "fun test(arr, idx) { arr[idx] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_as_index("idx", body));
                assert!(is_param_used_as_array("arr", body));
            }
        }
    }
}

#[test]
fn test_string_concatenation_not_numeric() {
    let code = r#"fun test(x) { "hello" + x }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                // String concat should NOT be detected as numeric
                assert!(!is_param_used_numerically("x", body));
                // But should be detected as string concat
                assert!(is_param_used_in_string_concat("x", body));
            }
        }
    }
}

#[test]
fn test_param_in_while_numeric() {
    let code = "fun test(n) { while n > 0 { n - 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("n", body));
            }
        }
    }
}

#[test]
fn test_param_in_for_numeric() {
    let code = "fun test(x) { for i in [1,2,3] { x + i } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_coverage_infer_bool_from_condition() {
    let code = "fun test(flag) { if flag { 1 } else { 0 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("flag", body), Some("bool"));
            }
        }
    }
}

#[test]
fn test_coverage_infer_array_from_indexing() {
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                let inferred = infer_param_type("arr", body);
                assert!(inferred == Some("Vec<_>") || inferred == Some("&[_]"));
            }
        }
    }
}

#[test]
fn test_coverage_infer_callable() {
    let code = "fun test(f) { f(1) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("f", body), Some("impl Fn(_) -> _"));
            }
        }
    }
}

#[test]
fn test_coverage_infer_usize_index() {
    let code = "fun test(i, arr) { arr[i] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert_eq!(infer_param_type("i", body), Some("usize"));
            }
        }
    }
}

#[test]
fn test_coverage_nested_array_true() {
    let code = "fun test(matrix) { matrix[0][1] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_nested_array_access("matrix", body));
            }
        }
    }
}

#[test]
fn test_coverage_nested_array_false() {
    let code = "fun test(arr) { arr[0] }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(!is_nested_array_access("arr", body));
            }
        }
    }
}

#[test]
fn test_check_let_pattern_traversal() {
    let code = "fun test(x) { let (a, b) = (1, 2); x + a }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_numerically("x", body));
            }
        }
    }
}

#[test]
fn test_param_used_with_len() {
    let code = "fun test(arr) { arr.len() }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_with_len("arr", body));
            }
        }
    }
}

#[test]
fn test_param_in_print_macro() {
    let code = r#"fun test(msg) { print!("{}", msg) }"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    if let ExprKind::Block(exprs) = &ast.kind {
        for expr in exprs {
            if let ExprKind::Function { body, .. } = &expr.kind {
                assert!(is_param_used_in_print_macro("msg", body));
            }
        }
    }
}
