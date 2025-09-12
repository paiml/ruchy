//! Property-based tests for minimal codegen refactoring

use quickcheck::{quickcheck, TestResult};
use ruchy::backend::transpiler::codegen_minimal::MinimalCodeGen;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, Span};

// Property: Literals generate valid Rust code
fn prop_literal_generation(val: i64) -> TestResult {
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(val)),
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&expr) {
        Ok(code) => {
            // Should generate the exact number
            TestResult::from_bool(code == val.to_string())
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: Binary operations preserve operator precedence
fn prop_binary_op_generation(left: i32, right: i32, op_idx: u8) -> TestResult {
    let op = match op_idx % 5 {
        0 => BinaryOp::Add,
        1 => BinaryOp::Subtract,
        2 => BinaryOp::Multiply,
        3 => BinaryOp::Divide,
        _ => BinaryOp::Modulo,
    };
    
    let expr = Expr::new(
        ExprKind::Binary {
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(left as i64)),
                Span { start: 0, end: 0 },
            )),
            op,
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(right as i64)),
                Span { start: 0, end: 0 },
            )),
        },
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&expr) {
        Ok(code) => {
            // Should wrap in parentheses for precedence
            TestResult::from_bool(code.starts_with('(') && code.ends_with(')'))
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: Unary operations generate correct prefix
fn prop_unary_op_generation(val: i32, negate: bool) -> TestResult {
    let op = if negate { UnaryOp::Negate } else { UnaryOp::Not };
    let expected_op = if negate { "-" } else { "!" };
    
    let expr = Expr::new(
        ExprKind::Unary {
            op,
            operand: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(val as i64)),
                Span { start: 0, end: 0 },
            )),
        },
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&expr) {
        Ok(code) => {
            // Should have correct operator and parentheses
            TestResult::from_bool(
                code.starts_with('(') && 
                code.contains(expected_op) && 
                code.ends_with(')')
            )
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: Block expressions generate valid Rust blocks
fn prop_block_generation(count: u8) -> TestResult {
    let count = (count % 5) + 1; // 1-5 expressions
    let exprs: Vec<Expr> = (0..count)
        .map(|i| Expr::new(
            ExprKind::Literal(Literal::Integer(i as i64)),
            Span { start: 0, end: 0 },
        ))
        .collect();
    
    let block = Expr::new(
        ExprKind::Block(exprs.clone()),
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&block) {
        Ok(code) => {
            // Should be wrapped in braces
            let valid_block = code.starts_with("{ ") && code.ends_with(" }");
            // Should have semicolons for all but last expression
            let semicolon_count = code.matches(';').count();
            let expected_semicolons = if count > 1 { count - 1 } else { 0 };
            
            TestResult::from_bool(
                valid_block && semicolon_count == expected_semicolons as usize
            )
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: If expressions generate valid Rust if statements
fn prop_if_generation(has_else: bool) -> TestResult {
    let condition = Expr::new(
        ExprKind::Literal(Literal::Bool(true)),
        Span { start: 0, end: 0 },
    );
    let then_branch = Expr::new(
        ExprKind::Literal(Literal::Integer(1)),
        Span { start: 0, end: 0 },
    );
    let else_branch = if has_else {
        Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span { start: 0, end: 0 },
        )))
    } else {
        None
    };
    
    let if_expr = Expr::new(
        ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        },
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&if_expr) {
        Ok(code) => {
            let has_if = code.starts_with("if ");
            let has_else_clause = code.contains(" else ");
            
            TestResult::from_bool(
                has_if && (has_else_clause == has_else)
            )
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: List expressions generate vec! macros
fn prop_list_generation(count: u8) -> TestResult {
    let count = count % 10; // 0-9 elements
    let elements: Vec<Expr> = (0..count)
        .map(|i| Expr::new(
            ExprKind::Literal(Literal::Integer(i as i64)),
            Span { start: 0, end: 0 },
        ))
        .collect();
    
    let list = Expr::new(
        ExprKind::List(elements),
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&list) {
        Ok(code) => {
            // Should use vec! macro
            TestResult::from_bool(
                code.starts_with("vec![") && code.ends_with(']')
            )
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: Lambda expressions generate valid closures
fn prop_lambda_generation(param_count: u8) -> TestResult {
    use ruchy::frontend::ast::{Param, Type, TypeKind};
    
    let param_count = (param_count % 3) + 1; // 1-3 params
    let params: Vec<Param> = (0..param_count)
        .map(|i| Param::Simple {
            name: format!("p{}", i),
            ty: Some(Type::new(TypeKind::Named("i32".to_string()), Span::new(0, 0))),
        })
        .collect();
    
    let body = Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span { start: 0, end: 0 },
    );
    
    let lambda = Expr::new(
        ExprKind::Lambda {
            params,
            body: Box::new(body),
        },
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&lambda) {
        Ok(code) => {
            // Should generate closure syntax
            TestResult::from_bool(
                code.starts_with('|') && code.contains("| ")
            )
        }
        Err(_) => TestResult::failed(),
    }
}

// Property: Method calls generate correct Rust syntax
fn prop_method_call_generation(method_name: String, arg_count: u8) -> TestResult {
    if method_name.is_empty() {
        return TestResult::discard();
    }
    
    let receiver = Expr::new(
        ExprKind::Identifier("obj".to_string()),
        Span { start: 0, end: 0 },
    );
    
    let arg_count = arg_count % 3; // 0-2 args
    let args: Vec<Expr> = (0..arg_count)
        .map(|i| Expr::new(
            ExprKind::Literal(Literal::Integer(i as i64)),
            Span { start: 0, end: 0 },
        ))
        .collect();
    
    let method_call = Expr::new(
        ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method: method_name.clone(),
            args,
        },
        Span { start: 0, end: 0 },
    );
    
    match MinimalCodeGen::gen_expr(&method_call) {
        Ok(code) => {
            // Should have dot notation
            TestResult::from_bool(
                code.contains(&format!(".{}", method_name)) &&
                code.contains('(') && code.contains(')')
            )
        }
        Err(_) => TestResult::failed(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prop_literal_generation() {
        quickcheck(prop_literal_generation as fn(i64) -> TestResult);
    }

    #[test]
    fn test_prop_binary_op_generation() {
        quickcheck(prop_binary_op_generation as fn(i32, i32, u8) -> TestResult);
    }

    #[test]
    fn test_prop_unary_op_generation() {
        quickcheck(prop_unary_op_generation as fn(i32, bool) -> TestResult);
    }

    #[test]
    fn test_prop_block_generation() {
        quickcheck(prop_block_generation as fn(u8) -> TestResult);
    }

    #[test]
    fn test_prop_if_generation() {
        quickcheck(prop_if_generation as fn(bool) -> TestResult);
    }

    #[test]
    fn test_prop_list_generation() {
        quickcheck(prop_list_generation as fn(u8) -> TestResult);
    }

    #[test]
    fn test_prop_lambda_generation() {
        quickcheck(prop_lambda_generation as fn(u8) -> TestResult);
    }

    #[test]
    fn test_prop_method_call_generation() {
        quickcheck(prop_method_call_generation as fn(String, u8) -> TestResult);
    }
}