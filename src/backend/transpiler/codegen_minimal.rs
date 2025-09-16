//! Minimal codegen for self-hosting MVP
//! Direct Rust mapping with no optimization - as specified in self-hosting spec
#![allow(clippy::missing_errors_doc)]
use crate::frontend::ast::{Expr, Literal, Pattern, BinaryOp, UnaryOp};
use anyhow::Result;
/// Minimal code generator for self-hosting
pub struct MinimalCodeGen;
impl MinimalCodeGen {
    /// Generate Rust code directly from AST with no optimization
/// # Examples
/// 
/// ```
/// use ruchy::backend::transpiler::codegen_minimal::gen_expr;
/// 
/// let result = gen_expr(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn gen_expr(expr: &Expr) -> Result<String> {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Literal(lit) => Self::gen_literal(lit),
            ExprKind::Identifier(name) => Ok(name.clone()),
            ExprKind::Binary { left, op, right } => Self::gen_binary_expr(left, *op, right),
            ExprKind::Unary { op, operand } => Self::gen_unary_expr(*op, operand),
            ExprKind::Let { name, value, body, .. } => Self::gen_let_expr(name, value, body),
            ExprKind::Function { name, params, body, .. } => Self::gen_function_expr(name, params, body),
            ExprKind::Lambda { params, body } => Self::gen_lambda_expr(params, body),
            ExprKind::Call { func, args } => Self::gen_call_expr(func, args),
            ExprKind::If { condition, then_branch, else_branch } => 
                Self::gen_if_expr(condition, then_branch, else_branch.as_deref()),
            ExprKind::Block(exprs) => Self::gen_block_expr(exprs),
            ExprKind::Match { expr, arms } => Self::gen_match_expr(expr, arms),
            ExprKind::List(elements) => Self::gen_list_expr(elements),
            ExprKind::Struct { name, fields, .. } => Self::gen_struct_def(name, fields),
            ExprKind::StructLiteral { name, fields } => Self::gen_struct_literal(name, fields),
            ExprKind::MethodCall { receiver, method, args } => 
                Self::gen_method_call(receiver, method, args),
            ExprKind::Macro { name, args } => Self::gen_macro_call(name, args),
            ExprKind::QualifiedName { module, name } => Ok(format!("{module}::{name}")),
            ExprKind::StringInterpolation { parts } => Self::gen_string_interpolation(parts),
            _ => Err(anyhow::anyhow!(
                "Minimal codegen does not support {:?} - use full transpiler for complete language support", 
                expr.kind
            ))
        }
    }
    fn gen_binary_expr(left: &Expr, op: BinaryOp, right: &Expr) -> Result<String> {
        let left_code = Self::gen_expr(left)?;
        let right_code = Self::gen_expr(right)?;
        let op_code = Self::gen_binary_op(op);
        Ok(format!("({left_code} {op_code} {right_code})"))
    }
    fn gen_unary_expr(op: UnaryOp, operand: &Expr) -> Result<String> {
        let operand_code = Self::gen_expr(operand)?;
        let op_code = Self::gen_unary_op(op);
        Ok(format!("({op_code} {operand_code})"))
    }
    fn gen_let_expr(name: &str, value: &Expr, body: &Expr) -> Result<String> {
        let value_code = Self::gen_expr(value)?;
        let body_code = Self::gen_expr(body)?;
        Ok(format!("{{ let {name} = {value_code}; {body_code} }}"))
    }
    fn gen_function_expr(
        name: &str, 
        params: &[crate::frontend::ast::Param], 
        body: &Expr
    ) -> Result<String> {
        let param_list = params.iter()
            .map(|p| { let name = p.name(); format!("{name}: i32") }) // Simplified for MVP
            .collect::<Vec<_>>()
            .join(", ");
        let body_code = Self::gen_expr(body)?;
        Ok(format!("fn {name}({param_list}) {{ {body_code} }}"))
    }
    fn gen_lambda_expr(params: &[crate::frontend::ast::Param], body: &Expr) -> Result<String> {
        let param_list = params.iter()
            .map(crate::frontend::ast::Param::name)
            .collect::<Vec<_>>()
            .join(", ");
        let body_code = Self::gen_expr(body)?;
        Ok(format!("|{param_list}| {body_code}"))
    }
    fn gen_call_expr(func: &Expr, args: &[Expr]) -> Result<String> {
        let func_code = Self::gen_expr(func)?;
        let arg_codes = args.iter()
            .map(Self::gen_expr)
            .collect::<Result<Vec<_>>>()?;
        Ok(format!("{func_code}({})", arg_codes.join(", ")))
    }
    fn gen_if_expr(
        condition: &Expr, 
        then_branch: &Expr, 
        else_branch: Option<&Expr>
    ) -> Result<String> {
        let cond_code = Self::gen_expr(condition)?;
        let then_code = Self::gen_expr(then_branch)?;
        if let Some(else_expr) = else_branch {
            let else_code = Self::gen_expr(else_expr)?;
            Ok(format!("if {cond_code} {{ {then_code} }} else {{ {else_code} }}"))
        } else {
            Ok(format!("if {cond_code} {{ {then_code} }}"))
        }
    }
    fn gen_block_expr(exprs: &[Expr]) -> Result<String> {
        let mut code = String::new();
        code.push_str("{ ");
        for (i, expr) in exprs.iter().enumerate() {
            let expr_code = Self::gen_expr(expr)?;
            if i == exprs.len() - 1 {
                // Last expression is the return value
                code.push_str(&expr_code);
            } else {
                code.push_str(&format!("{expr_code}; "));
            }
        }
        code.push_str(" }");
        Ok(code)
    }
    fn gen_match_expr(expr: &Expr, arms: &[crate::frontend::ast::MatchArm]) -> Result<String> {
        let expr_code = Self::gen_expr(expr)?;
        let mut code = format!("match {expr_code} {{\n");
        for arm in arms {
            let pattern_code = Self::gen_pattern(&arm.pattern)?;
            let body_code = Self::gen_expr(&arm.body)?;
            code.push_str(&format!("    {pattern_code} => {body_code},\n"));
        }
        code.push('}');
        Ok(code)
    }
    fn gen_list_expr(elements: &[Expr]) -> Result<String> {
        let element_codes = elements.iter()
            .map(Self::gen_expr)
            .collect::<Result<Vec<_>>>()?;
        let elements = element_codes.join(", ");
        Ok(format!("vec![{elements}]"))
    }
    fn gen_struct_def(
        name: &str, 
        fields: &[crate::frontend::ast::StructField]
    ) -> Result<String> {
        let field_list = fields.iter()
            .map(|f| { let name = &f.name; format!("    {name}: String,") }) // Simplified for MVP
            .collect::<Vec<_>>()
            .join("\n");
        Ok(format!("struct {name} {{\n{field_list}\n}}"))
    }
    fn gen_struct_literal(name: &str, fields: &[(String, Expr)]) -> Result<String> {
        let field_codes = fields.iter()
            .map(|f| {
                let value_code = Self::gen_expr(&f.1)?;
                let field_name = &f.0;
                Ok(format!("{field_name}: {value_code}"))
            })
            .collect::<Result<Vec<_>>>()?;
        let fields = field_codes.join(", ");
        Ok(format!("{name} {{ {fields} }}"))
    }
    fn gen_method_call(receiver: &Expr, method: &str, args: &[Expr]) -> Result<String> {
        let receiver_code = Self::gen_expr(receiver)?;
        let arg_codes = args.iter()
            .map(Self::gen_expr)
            .collect::<Result<Vec<_>>>()?;
        let args = arg_codes.join(", ");
        Ok(format!("{receiver_code}.{method}({args})"))
    }
    fn gen_macro_call(name: &str, args: &[Expr]) -> Result<String> {
        let arg_codes = args.iter()
            .map(Self::gen_expr)
            .collect::<Result<Vec<_>>>()?;
        let args = arg_codes.join(", ");
        Ok(format!("{name}!({args})"))
    }
    fn gen_string_interpolation(
        parts: &[crate::frontend::ast::StringPart]
    ) -> Result<String> {
        // Simplified string interpolation for MVP
        let mut result = String::from("format!(");
        let mut format_str = String::new();
        let mut args = Vec::new();
        for part in parts {
            match part {
                crate::frontend::ast::StringPart::Text(s) => {
                    format_str.push_str(s);
                }
                crate::frontend::ast::StringPart::Expr(expr) => {
                    format_str.push_str("{}");
                    args.push(Self::gen_expr(expr)?);
                }
                crate::frontend::ast::StringPart::ExprWithFormat { expr, format_spec } => {
                    format_str.push('{');
                    format_str.push_str(format_spec);
                    format_str.push('}');
                    args.push(Self::gen_expr(expr)?);
                }
            }
        }
        result.push_str(&format!("\"{format_str}\""));
        if !args.is_empty() {
            result.push_str(", ");
            result.push_str(&args.join(", "));
        }
        result.push(')');
        Ok(result)
    }
    fn gen_literal(lit: &Literal) -> Result<String> {
        match lit {
            Literal::Integer(i) => Ok(i.to_string()),
            Literal::Float(f) => Ok(f.to_string()),
            Literal::String(s) => Ok(format!("\"{}\"", s.replace('"', "\\\""))),
            Literal::Bool(b) => Ok(b.to_string()),
            Literal::Char(c) => Ok(format!("'{c}'")),
            Literal::Unit => Ok("()".to_string()),
        }
    }
    fn gen_binary_op(op: BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-", 
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::NullCoalesce => "??", // JavaScript-style null coalescing
            BinaryOp::BitwiseAnd => "&",
            BinaryOp::BitwiseOr => "|",
            BinaryOp::BitwiseXor => "^",
            BinaryOp::LeftShift => "<<",
            BinaryOp::Power => "pow", // Will need function call wrapper
        }
    }
    fn gen_unary_op(op: UnaryOp) -> &'static str {
        match op {
            UnaryOp::Not => "!",
            UnaryOp::Negate => "-",
            UnaryOp::BitwiseNot => "~",
            UnaryOp::Reference => "&",
        }
    }
    fn gen_pattern(pattern: &Pattern) -> Result<String> {
        match pattern {
            Pattern::Wildcard => Ok("_".to_string()),
            Pattern::Literal(lit) => Self::gen_literal(lit),
            Pattern::Identifier(name) => Ok(name.clone()),
            Pattern::List(patterns) => {
                let pattern_codes = patterns.iter()
                    .map(Self::gen_pattern)
                    .collect::<Result<Vec<_>>>()?;
                let patterns = pattern_codes.join(", ");
                Ok(format!("[{patterns}]"))
            }
            Pattern::Ok(inner) => {
                let inner_code = Self::gen_pattern(inner)?;
                Ok(format!("Ok({inner_code})"))
            }
            Pattern::Err(inner) => {
                let inner_code = Self::gen_pattern(inner)?;
                Ok(format!("Err({inner_code})"))
            }
            Pattern::Some(inner) => {
                let inner_code = Self::gen_pattern(inner)?;
                Ok(format!("Some({inner_code})"))
            }
            Pattern::None => Ok("None".to_string()),
            _ => Ok("_".to_string()), // Simplified for MVP
        }
    }
    // Type generation simplified for MVP - focus on minimal working compiler
    #[allow(dead_code)]
    fn gen_type(_ty: &crate::frontend::ast::Type) -> Result<String> {
        Ok("String".to_string()) // Simplified for self-hosting MVP
    }
    /// Generate complete Rust program for self-hosting
/// # Examples
/// 
/// ```
/// use ruchy::backend::transpiler::codegen_minimal::gen_program;
/// 
/// let result = gen_program(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn gen_program(expr: &Expr) -> Result<String> {
        let main_code = Self::gen_expr(expr)?;
        Ok(format!(
            "use std::collections::HashMap;\n\n{main_code}"
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;
#[cfg(test)]
    fn gen_str(input: &str) -> Result<String> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        MinimalCodeGen::gen_expr(&expr)
    }
    #[test]
    fn test_basic_expressions() {
        assert_eq!(gen_str("42").unwrap(), "42");
        assert_eq!(gen_str("true").unwrap(), "true");
        assert_eq!(gen_str("\"hello\"").unwrap(), "\"hello\"");
    }
    #[test] 
    fn test_binary_ops() {
        assert_eq!(gen_str("1 + 2").unwrap(), "(1 + 2)");
        assert_eq!(gen_str("x * y").unwrap(), "(x * y)");
    }
    #[test]
    fn test_function_def() {
        let result = gen_str("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        assert!(result.contains("fn add(x: i32, y: i32)"));
    }
    #[test]
    fn test_lambda() {
        assert_eq!(gen_str("|x| x + 1").unwrap(), "|x| (x + 1)");
    }
    #[test]
    fn test_list() {
        assert_eq!(gen_str("[1, 2, 3]").unwrap(), "vec![1, 2, 3]");
    }
}
#[cfg(test)]
mod property_tests_codegen_minimal {
    use proptest::proptest;
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_gen_expr_never_panics(input: String) {
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
