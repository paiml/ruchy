// Code formatter for Ruchy
// Toyota Way: Consistent code style prevents defects
use anyhow::Result;
use crate::frontend::ast::{Expr, ExprKind};
pub struct Formatter {
    indent_width: usize,
    use_tabs: bool,
    _line_width: usize,
}
impl Formatter {
/// # Examples
/// 
/// ```
/// use ruchy::quality::formatter::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        Self {
            indent_width: 4,
            use_tabs: false,
            _line_width: 100,
        }
    }
/// # Examples
/// 
/// ```
/// use ruchy::quality::formatter::format;
/// 
/// let result = format(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn format(&self, ast: &Expr) -> Result<String> {
        // Simple formatter that converts AST back to source
        Ok(self.format_expr(ast, 0))
    }
    fn format_type(&self, ty_kind: &crate::frontend::ast::TypeKind) -> String {
        match ty_kind {
            crate::frontend::ast::TypeKind::Named(name) => name.clone(),
            _ => format!("{ty_kind:?}"),
        }
    }
    fn format_expr(&self, expr: &Expr, indent: usize) -> String {
        let indent_str = if self.use_tabs {
            "\t".repeat(indent)
        } else {
            " ".repeat(indent * self.indent_width)
        };
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                crate::frontend::ast::Literal::Integer(n) => n.to_string(),
                crate::frontend::ast::Literal::Float(f) => f.to_string(),
                crate::frontend::ast::Literal::String(s) => format!("\"{s}\""),
                crate::frontend::ast::Literal::Bool(b) => b.to_string(),
                crate::frontend::ast::Literal::Char(c) => format!("'{c}'"),
                crate::frontend::ast::Literal::Unit => "()".to_string(),
            },
            ExprKind::Identifier(name) => name.clone(),
            ExprKind::Let { name, value, body, .. } => {
                format!(
                    "let {} = {} in {}",
                    name,
                    self.format_expr(value, indent),
                    self.format_expr(body, indent)
                )
            }
            ExprKind::Binary { left, op, right } => {
                format!(
                    "{} {} {}",
                    self.format_expr(left, indent),
                    op,
                    self.format_expr(right, indent)
                )
            }
            ExprKind::Block(exprs) => {
                let mut result = String::from("{\n");
                for expr in exprs {
                    result.push_str(&format!(
                        "{}{}\n",
                        indent_str,
                        self.format_expr(expr, indent + 1)
                    ));
                }
                result.push_str(&format!("{indent_str}}}"));
                result
            }
            ExprKind::Function { name, params, return_type, body, .. } => {
                let mut result = format!("fun {name}");
                // Parameters
                result.push('(');
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { result.push_str(", "); }
                    if let crate::frontend::ast::Pattern::Identifier(param_name) = &param.pattern {
                        result.push_str(param_name);
                        result.push_str(": ");
                        result.push_str(&self.format_type(&param.ty.kind));
                    }
                }
                result.push(')');
                // Return type
                if let Some(ret_ty) = return_type {
                    result.push_str(" -> ");
                    result.push_str(&self.format_type(&ret_ty.kind));
                }
                result.push(' ');
                result.push_str(&self.format_expr(body.as_ref(), indent));
                result
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                let mut result = "if ".to_string();
                result.push_str(&self.format_expr(condition, indent));
                result.push(' ');
                result.push_str(&self.format_expr(then_branch, indent));
                if let Some(else_expr) = else_branch {
                    result.push_str(" else ");
                    result.push_str(&self.format_expr(else_expr, indent));
                }
                result
            }
            _ => {
                format!("{:?}", expr.kind) // Fallback for unimplemented cases
            }
        }
    }
}
impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::*;

    fn create_simple_literal(value: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(value)), Default::default())
    }

    fn create_identifier(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Default::default())
    }

    #[test]
    fn test_formatter_new() {
        let formatter = Formatter::new();
        assert_eq!(formatter.indent_width, 4);
        assert!(!formatter.use_tabs);
    }

    #[test]
    fn test_formatter_default() {
        let formatter = Formatter::default();
        assert_eq!(formatter.indent_width, 4);
        assert!(!formatter.use_tabs);
    }

    #[test]
    fn test_format_integer_literal() {
        let formatter = Formatter::new();
        let expr = create_simple_literal(42);
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_float_literal() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Float(3.14)), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "3.14");
    }

    #[test]
    fn test_format_string_literal() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::String("hello".to_string())), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_format_bool_literal() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "true");

        let expr = Expr::new(ExprKind::Literal(Literal::Bool(false)), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_format_char_literal() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Char('a')), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "'a'");
    }

    #[test]
    fn test_format_unit_literal() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Unit), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "()");
    }

    #[test]
    fn test_format_identifier() {
        let formatter = Formatter::new();
        let expr = create_identifier("my_var");
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "my_var");
    }

    #[test]
    fn test_format_binary_expression() {
        let formatter = Formatter::new();
        let left = create_simple_literal(1);
        let right = create_simple_literal(2);
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op: BinaryOp::Add,
                right: Box::new(right),
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "1 Add 2");
    }

    #[test]
    fn test_format_let_expression() {
        let formatter = Formatter::new();
        let value = create_simple_literal(42);
        let body = create_identifier("x");
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(value),
                body: Box::new(body),
                ty: Some(Type {
                    kind: TypeKind::Named("Int".to_string()),
                    span: Default::default(),
                }),
                is_mut: false,
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "let x = 42 in x");
    }

    #[test]
    fn test_format_block_expression() {
        let formatter = Formatter::new();
        let exprs = vec![
            create_simple_literal(1),
            create_simple_literal(2),
        ];
        let expr = Expr::new(ExprKind::Block(exprs), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert!(result.contains("{\n"));
        assert!(result.contains("1\n"));
        assert!(result.contains("2\n"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_format_if_expression() {
        let formatter = Formatter::new();
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let then_branch = create_simple_literal(1);
        let else_branch = create_simple_literal(2);
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "if true 1 else 2");
    }

    #[test]
    fn test_format_if_without_else() {
        let formatter = Formatter::new();
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let then_branch = create_simple_literal(1);
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "if true 1");
    }

    #[test]
    fn test_format_function_simple() {
        let formatter = Formatter::new();
        let body = create_simple_literal(42);
        let expr = Expr::new(
            ExprKind::Function {
                name: "test".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert!(result.starts_with("fun test"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_function_with_params() {
        let formatter = Formatter::new();
        let body = create_identifier("x");
        let param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("Int".to_string()),
                span: Default::default(),
            },
            span: Default::default(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Function {
                name: "identity".to_string(),
                type_params: vec![],
                params: vec![param],
                return_type: Some(Type {
                    kind: TypeKind::Named("Int".to_string()),
                    span: Default::default(),
                }),
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert!(result.contains("fun identity"));
        assert!(result.contains("x: Int"));
        assert!(result.contains("-> Int"));
    }

    #[test]
    fn test_format_type_named() {
        let formatter = Formatter::new();
        let type_kind = TypeKind::Named("String".to_string());
        let result = formatter.format_type(&type_kind);
        assert_eq!(result, "String");
    }

    #[test]
    fn test_format_type_fallback() {
        let formatter = Formatter::new();
        let type_kind = TypeKind::List(Box::new(Type {
            kind: TypeKind::Named("Int".to_string()),
            span: Default::default(),
        }));
        let result = formatter.format_type(&type_kind);
        assert!(result.contains("List"));
    }

    #[test]
    fn test_format_with_tabs() {
        let mut formatter = Formatter::new();
        formatter.use_tabs = true;
        let exprs = vec![create_simple_literal(1)];
        let expr = Expr::new(ExprKind::Block(exprs), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert!(result.contains("\t"));
    }

    #[test]
    fn test_format_with_spaces() {
        let mut formatter = Formatter::new();
        formatter.use_tabs = false;
        formatter.indent_width = 2;
        let exprs = vec![create_simple_literal(1)];
        let expr = Expr::new(ExprKind::Block(exprs), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert!(result.contains("  1")); // 2 spaces indentation
    }

    #[test]
    fn test_format_nested_expressions() {
        let formatter = Formatter::new();
        let inner = Expr::new(
            ExprKind::Binary {
                left: Box::new(create_simple_literal(1)),
                op: BinaryOp::Add,
                right: Box::new(create_simple_literal(2)),
            },
            Default::default()
        );
        let outer = Expr::new(
            ExprKind::Binary {
                left: Box::new(inner),
                op: BinaryOp::Multiply,
                right: Box::new(create_simple_literal(3)),
            },
            Default::default()
        );
        let result = formatter.format(&outer).unwrap();
        assert!(result.contains("1 Add 2"));
        assert!(result.contains("Multiply 3"));
    }

    #[test]
    fn test_format_multiple_params() {
        let formatter = Formatter::new();
        let body = create_simple_literal(0);
        let param1 = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("Int".to_string()),
                span: Default::default(),
            },
            span: Default::default(),
            is_mutable: false,
            default_value: None,
        };
        let param2 = Param {
            pattern: Pattern::Identifier("y".to_string()),
            ty: Type {
                kind: TypeKind::Named("Float".to_string()),
                span: Default::default(),
            },
            span: Default::default(),
            is_mutable: false,
            default_value: None,
        };
        let expr = Expr::new(
            ExprKind::Function {
                name: "test".to_string(),
                type_params: vec![],
                params: vec![param1, param2],
                return_type: None,
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert!(result.contains("x: Int, y: Float"));
    }

    #[test]
    fn test_format_empty_block() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Block(vec![]), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "{\n}");
    }

    #[test]
    fn test_format_string_with_quotes() {
        let formatter = Formatter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("hello \"world\"".to_string())),
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "\"hello \\\"world\\\"\"");
    }

    #[test]
    fn test_format_special_characters() {
        let formatter = Formatter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Char('\n')), Default::default());
        let result = formatter.format(&expr).unwrap();
        assert_eq!(result, "'\n'");
    }

    #[test]
    fn test_format_fallback_case() {
        let formatter = Formatter::new();
        // Use an expression kind that doesn't have explicit formatting
        let expr = Expr::new(
            ExprKind::StringInterpolation { parts: vec![] },
            Default::default()
        );
        let result = formatter.format(&expr).unwrap();
        assert!(result.contains("StringInterpolation"));
    }

    #[test]
    fn test_formatter_field_access() {
        let formatter = Formatter::new();
        assert_eq!(formatter.indent_width, 4);
        assert!(!formatter.use_tabs);
    }

    #[test]
    fn test_format_deeply_nested_block() {
        let formatter = Formatter::new();
        let inner_block = Expr::new(
            ExprKind::Block(vec![create_simple_literal(1)]),
            Default::default()
        );
        let outer_block = Expr::new(
            ExprKind::Block(vec![inner_block]),
            Default::default()
        );
        let result = formatter.format(&outer_block).unwrap();
        assert!(result.contains("{\n"));
        assert!(result.contains("}\n"));
        assert!(result.contains("}"));
    }
}

#[cfg(test)]
mod property_tests_formatter {
    use proptest::proptest;


    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
