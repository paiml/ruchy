//! Assertion extraction from AST
//!
//! Extracts assert statements from Ruchy AST for proof verification

use crate::frontend::ast::{Expr, ExprKind, MatchArm};

/// Extract assert statements from AST
pub fn extract_assertions_from_ast(ast: &Expr) -> Vec<String> {
    let mut assertions = Vec::new();
    if let ExprKind::Block(exprs) = &ast.kind {
        extract_assert_sequence_from_block(exprs, &mut assertions);
    } else {
        extract_assertions_recursive(ast, &mut assertions);
    }
    assertions
}

/// Extract assert statements from a sequence of expressions
fn extract_assert_sequence_from_block(exprs: &[Expr], assertions: &mut Vec<String>) {
    let mut i = 0;
    while i < exprs.len() {
        i += process_expression_at_index(exprs, i, assertions);
    }
}

fn process_expression_at_index(exprs: &[Expr], i: usize, assertions: &mut Vec<String>) -> usize {
    if is_assert_statement(exprs, i) {
        extract_assert_at_index(exprs, i, assertions);
        2 // Skip assert + expression
    } else {
        extract_assertions_recursive(&exprs[i], assertions);
        1 // Move to next expression
    }
}

fn is_assert_statement(exprs: &[Expr], i: usize) -> bool {
    if let ExprKind::Identifier(name) = &exprs[i].kind {
        name == "assert" && i + 1 < exprs.len()
    } else {
        false
    }
}

fn extract_assert_at_index(exprs: &[Expr], i: usize, assertions: &mut Vec<String>) {
    let assertion_expr = &exprs[i + 1];
    let assertion_text = expr_to_assertion_string(assertion_expr);
    assertions.push(assertion_text);
}

fn extract_assertions_recursive(expr: &Expr, assertions: &mut Vec<String>) {
    match &expr.kind {
        ExprKind::Call { func, args } => extract_from_call(func, args, assertions),
        ExprKind::Block(exprs) => extract_from_block(exprs, assertions),
        ExprKind::Let { value, body, .. } => extract_from_let(value, body, assertions),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => extract_from_if(condition, then_branch, else_branch.as_deref(), assertions),
        ExprKind::Match { expr, arms } => extract_from_match(expr, arms, assertions),
        ExprKind::Lambda { body, .. } => extract_assertions_recursive(body, assertions),
        _ => {}
    }
}

fn extract_from_call(func: &Expr, args: &[Expr], assertions: &mut Vec<String>) {
    if let ExprKind::Identifier(name) = &func.kind {
        if name == "assert" && !args.is_empty() {
            let assertion_text = expr_to_assertion_string(&args[0]);
            assertions.push(assertion_text);
        }
    }
    extract_assertions_recursive(func, assertions);
    for arg in args {
        extract_assertions_recursive(arg, assertions);
    }
}

fn extract_from_block(exprs: &[Expr], assertions: &mut Vec<String>) {
    for expr in exprs {
        extract_assertions_recursive(expr, assertions);
    }
}

fn extract_from_let(value: &Expr, body: &Expr, assertions: &mut Vec<String>) {
    extract_assertions_recursive(value, assertions);
    extract_assertions_recursive(body, assertions);
}

fn extract_from_if(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
    assertions: &mut Vec<String>,
) {
    extract_assertions_recursive(condition, assertions);
    extract_assertions_recursive(then_branch, assertions);
    if let Some(else_br) = else_branch {
        extract_assertions_recursive(else_br, assertions);
    }
}

fn extract_from_match(expr: &Expr, arms: &[MatchArm], assertions: &mut Vec<String>) {
    extract_assertions_recursive(expr, assertions);
    for arm in arms {
        extract_assertions_recursive(&arm.body, assertions);
    }
}

pub fn expr_to_assertion_string(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Literal(lit) => format_literal(lit),
        ExprKind::Identifier(name) => name.clone(),
        ExprKind::Binary { op, left, right } => format_binary(op, left, right),
        ExprKind::Call { func, args } => format_call(func, args),
        ExprKind::MethodCall {
            receiver,
            method,
            args,
        } => format_method_call(receiver, method, args),
        _ => format!("UNKNOWN_EXPR({:?})", expr.kind),
    }
}

fn format_literal(lit: &crate::frontend::ast::Literal) -> String {
    match lit {
        crate::frontend::ast::Literal::Integer(n, _) => n.to_string(),
        crate::frontend::ast::Literal::Float(f) => f.to_string(),
        crate::frontend::ast::Literal::String(s) => format!("\"{s}\""),
        crate::frontend::ast::Literal::Bool(b) => b.to_string(),
        _ => format!("{lit:?}"),
    }
}

fn format_binary(op: &crate::frontend::ast::BinaryOp, left: &Expr, right: &Expr) -> String {
    let op_str = binary_op_to_string(op);
    format!(
        "{} {} {}",
        expr_to_assertion_string(left),
        op_str,
        expr_to_assertion_string(right)
    )
}

fn binary_op_to_string(op: &crate::frontend::ast::BinaryOp) -> &'static str {
    match op {
        crate::frontend::ast::BinaryOp::Add => "+",
        crate::frontend::ast::BinaryOp::Subtract => "-",
        crate::frontend::ast::BinaryOp::Multiply => "*",
        crate::frontend::ast::BinaryOp::Divide => "/",
        crate::frontend::ast::BinaryOp::Equal => "==",
        crate::frontend::ast::BinaryOp::NotEqual => "!=",
        crate::frontend::ast::BinaryOp::Greater => ">",
        crate::frontend::ast::BinaryOp::GreaterEqual => ">=",
        crate::frontend::ast::BinaryOp::Less => "<",
        crate::frontend::ast::BinaryOp::LessEqual => "<=",
        _ => "UNKNOWN_OP",
    }
}

fn format_call(func: &Expr, args: &[Expr]) -> String {
    let func_str = expr_to_assertion_string(func);
    let args_str = args
        .iter()
        .map(expr_to_assertion_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{func_str}({args_str})")
}

fn format_method_call(receiver: &Expr, method: &str, args: &[Expr]) -> String {
    let receiver_str = expr_to_assertion_string(receiver);
    let args_str = args
        .iter()
        .map(expr_to_assertion_string)
        .collect::<Vec<_>>()
        .join(", ");
    if args.is_empty() {
        format!("{receiver_str}.{method}()")
    } else {
        format!("{receiver_str}.{method}({args_str})")
    }
}
