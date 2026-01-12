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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span};

    fn create_int_literal(n: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(n, None)),
            Span::default(),
        )
    }

    fn create_identifier(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Span::default())
    }

    fn create_binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::new(
            ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        )
    }

    #[test]
    fn test_extract_assertions_empty() {
        let block = Expr::new(ExprKind::Block(vec![]), Span::default());
        let assertions = extract_assertions_from_ast(&block);
        assert!(assertions.is_empty());
    }

    #[test]
    fn test_extract_assertions_from_call() {
        let assert_call = Expr::new(
            ExprKind::Call {
                func: Box::new(create_identifier("assert")),
                args: vec![create_binary(
                    BinaryOp::Equal,
                    create_int_literal(1),
                    create_int_literal(1),
                )],
            },
            Span::default(),
        );
        let block = Expr::new(ExprKind::Block(vec![assert_call]), Span::default());
        let assertions = extract_assertions_from_ast(&block);
        assert_eq!(assertions.len(), 1);
        assert!(assertions[0].contains("=="));
    }

    #[test]
    fn test_expr_to_assertion_string_integer() {
        let expr = create_int_literal(42);
        assert_eq!(expr_to_assertion_string(&expr), "42");
    }

    #[test]
    fn test_expr_to_assertion_string_identifier() {
        let expr = create_identifier("x");
        assert_eq!(expr_to_assertion_string(&expr), "x");
    }

    #[test]
    fn test_expr_to_assertion_string_binary() {
        let expr = create_binary(BinaryOp::Add, create_int_literal(1), create_int_literal(2));
        assert_eq!(expr_to_assertion_string(&expr), "1 + 2");
    }

    #[test]
    fn test_format_literal_float() {
        let result = format_literal(&Literal::Float(3.14));
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_format_literal_string() {
        let result = format_literal(&Literal::String("hello".to_string()));
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_format_literal_bool() {
        assert_eq!(format_literal(&Literal::Bool(true)), "true");
        assert_eq!(format_literal(&Literal::Bool(false)), "false");
    }

    #[test]
    fn test_binary_op_to_string() {
        assert_eq!(binary_op_to_string(&BinaryOp::Add), "+");
        assert_eq!(binary_op_to_string(&BinaryOp::Subtract), "-");
        assert_eq!(binary_op_to_string(&BinaryOp::Multiply), "*");
        assert_eq!(binary_op_to_string(&BinaryOp::Divide), "/");
        assert_eq!(binary_op_to_string(&BinaryOp::Equal), "==");
        assert_eq!(binary_op_to_string(&BinaryOp::NotEqual), "!=");
        assert_eq!(binary_op_to_string(&BinaryOp::Greater), ">");
        assert_eq!(binary_op_to_string(&BinaryOp::GreaterEqual), ">=");
        assert_eq!(binary_op_to_string(&BinaryOp::Less), "<");
        assert_eq!(binary_op_to_string(&BinaryOp::LessEqual), "<=");
    }

    #[test]
    fn test_format_call() {
        let func = create_identifier("foo");
        let args = vec![create_int_literal(1), create_int_literal(2)];
        let result = format_call(&func, &args);
        assert_eq!(result, "foo(1, 2)");
    }

    #[test]
    fn test_format_method_call_no_args() {
        let receiver = create_identifier("obj");
        let result = format_method_call(&receiver, "method", &[]);
        assert_eq!(result, "obj.method()");
    }

    #[test]
    fn test_format_method_call_with_args() {
        let receiver = create_identifier("obj");
        let args = vec![create_int_literal(1)];
        let result = format_method_call(&receiver, "method", &args);
        assert_eq!(result, "obj.method(1)");
    }

    #[test]
    fn test_extract_from_let() {
        let value = create_int_literal(42);
        let body = create_int_literal(1);
        let mut assertions = vec![];
        extract_from_let(&value, &body, &mut assertions);
        // No assertions in simple literals
        assert!(assertions.is_empty());
    }

    #[test]
    fn test_extract_from_if() {
        let condition = create_identifier("true");
        let then_branch = create_int_literal(1);
        let else_branch = create_int_literal(2);
        let mut assertions = vec![];
        extract_from_if(
            &condition,
            &then_branch,
            Some(&else_branch),
            &mut assertions,
        );
        assert!(assertions.is_empty());
    }

    #[test]
    fn test_extract_from_block() {
        let exprs = vec![create_int_literal(1), create_int_literal(2)];
        let mut assertions = vec![];
        extract_from_block(&exprs, &mut assertions);
        assert!(assertions.is_empty());
    }

    #[test]
    fn test_is_assert_statement_true() {
        let exprs = vec![create_identifier("assert"), create_int_literal(1)];
        assert!(is_assert_statement(&exprs, 0));
    }

    #[test]
    fn test_is_assert_statement_false() {
        let exprs = vec![create_int_literal(1)];
        assert!(!is_assert_statement(&exprs, 0));
    }

    #[test]
    fn test_process_expression_at_index_non_assert() {
        let exprs = vec![create_int_literal(1)];
        let mut assertions = vec![];
        let skip = process_expression_at_index(&exprs, 0, &mut assertions);
        assert_eq!(skip, 1);
    }

    #[test]
    fn test_expr_to_assertion_string_unknown() {
        let expr = Expr::new(ExprKind::Literal(Literal::Unit), Span::default());
        let result = expr_to_assertion_string(&expr);
        // Unit literal formats as "Unit" via Debug
        assert!(!result.is_empty());
    }
}
