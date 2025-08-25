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
    pub fn new() -> Self {
        Self {
            indent_width: 4,
            use_tabs: false,
            _line_width: 100,
        }
    }
    
    pub fn format(&self, ast: &Expr) -> Result<String> {
        // Simple formatter that converts AST back to source
        Ok(self.format_expr(ast, 0))
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
            _ => format!("{:?}", expr.kind), // Fallback for unimplemented cases
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}