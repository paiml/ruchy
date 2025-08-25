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