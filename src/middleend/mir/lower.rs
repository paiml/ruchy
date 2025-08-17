//! AST to MIR lowering

use super::builder::MirBuilder;
use super::types::{BinOp, BlockId, Constant, Operand, Place, Program, Rvalue, Type, UnOp};
use crate::frontend::ast::{
    BinaryOp as AstBinOp, Expr, ExprKind, Literal, Param, Type as AstType, UnaryOp as AstUnOp,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

/// Context for lowering AST to MIR
pub struct LoweringContext {
    /// MIR builder
    builder: MirBuilder,
    /// Type environment for expressions
    #[allow(dead_code)]
    type_env: HashMap<String, Type>,
    /// Current block being built
    current_block: Option<BlockId>,
}

impl LoweringContext {
    /// Create a new lowering context
    #[must_use]
    pub fn new() -> Self {
        Self {
            builder: MirBuilder::new(),
            type_env: HashMap::new(),
            current_block: None,
        }
    }

    /// Lower an expression to MIR
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be lowered to MIR
    pub fn lower_expr(&mut self, expr: &Expr) -> Result<Program> {
        match &expr.kind {
            ExprKind::Function {
                name,
                params,
                return_type,
                body,
                ..
            } => self.lower_function(name, params, return_type.as_ref(), body),
            _ => {
                // For non-function expressions, create a main function
                self.lower_main_expr(expr)
            }
        }
    }

    /// Lower a function expression
    fn lower_function(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: Option<&AstType>,
        body: &Expr,
    ) -> Result<Program> {
        let func_name = name.to_string();
        let ret_ty = return_type
            .map_or(Type::Unit, |t| self.ast_to_mir_type(t));

        self.builder.start_function(func_name.clone(), ret_ty);

        // Add parameters
        for param in params {
            let ty = self.ast_to_mir_type(&param.ty);
            self.builder.add_param(param.name.clone(), ty);
        }

        // Create entry block
        let entry = self.builder.new_block();
        self.current_block = Some(entry);

        // Lower function body
        let result = self.lower_expr_to_operand(body)?;

        // Return the result
        self.builder.return_(entry, Some(result));

        let function = self
            .builder
            .finish_function()
            .ok_or_else(|| anyhow!("Failed to finish function"))?;

        let mut functions = HashMap::new();
        functions.insert(func_name.clone(), function);

        Ok(Program {
            functions,
            entry: func_name,
        })
    }

    /// Lower a main expression (wrap in main function)
    fn lower_main_expr(&mut self, expr: &Expr) -> Result<Program> {
        self.builder.start_function("main".to_string(), Type::Unit);

        let entry = self.builder.new_block();
        self.current_block = Some(entry);

        // Lower the expression
        let _result = self.lower_expr_to_operand(expr)?;

        // Main function returns unit
        self.builder
            .return_(entry, Some(Operand::Constant(Constant::Unit)));

        let function = self
            .builder
            .finish_function()
            .ok_or_else(|| anyhow!("Failed to finish main function"))?;

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        Ok(Program {
            functions,
            entry: "main".to_string(),
        })
    }

    /// Lower an expression to an operand
    fn lower_expr_to_operand(&mut self, expr: &Expr) -> Result<Operand> {
        let block = self
            .current_block
            .ok_or_else(|| anyhow!("No current block"))?;

        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Operand::Constant(Self::lower_literal(lit))),
            ExprKind::Identifier(name) => {
                if let Some(local) = self.builder.get_local(name) {
                    Ok(Operand::Copy(Place::Local(local)))
                } else {
                    Err(anyhow!("Unbound variable: {}", name))
                }
            }
            ExprKind::Binary { op, left, right } => {
                let left_op = self.lower_expr_to_operand(left)?;
                let right_op = self.lower_expr_to_operand(right)?;
                let mir_op = Self::lower_binary_op(*op);

                // Create a temporary for the result
                let result_ty = Self::infer_binary_result_type(*op);
                let temp = self.builder.alloc_local(result_ty, false, None);

                self.builder
                    .binary_op(block, temp, mir_op, left_op, right_op);
                Ok(Operand::Move(Place::Local(temp)))
            }
            ExprKind::Unary { op, operand } => {
                let operand_mir = self.lower_expr_to_operand(operand)?;
                let mir_op = Self::lower_unary_op(*op);

                let result_ty = Self::infer_unary_result_type(*op);
                let temp = self.builder.alloc_local(result_ty, false, None);

                self.builder.unary_op(block, temp, mir_op, operand_mir);
                Ok(Operand::Move(Place::Local(temp)))
            }
            ExprKind::Let { name, value, body } => {
                // Lower the value
                let value_op = self.lower_expr_to_operand(value)?;

                // Create a local for the binding
                let local_ty = Type::I32; // Default type inference
                let local = self
                    .builder
                    .alloc_local(local_ty, false, Some(name.clone()));

                // Assign the value
                self.builder
                    .assign(block, Place::Local(local), Rvalue::Use(value_op));

                // Lower the body with the binding in scope
                self.lower_expr_to_operand(body)
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_op = self.lower_expr_to_operand(condition)?;

                let then_block = self.builder.new_block();
                let else_block = self.builder.new_block();
                let merge_block = self.builder.new_block();

                // Branch based on condition
                self.builder.branch(block, cond_op, then_block, else_block);

                // Lower then branch
                self.current_block = Some(then_block);
                let then_result = self.lower_expr_to_operand(then_branch)?;
                self.builder.goto(then_block, merge_block);

                // Lower else branch
                self.current_block = Some(else_block);
                let _else_result = if let Some(else_expr) = else_branch {
                    self.lower_expr_to_operand(else_expr)?
                } else {
                    Operand::Constant(Constant::Unit)
                };
                self.builder.goto(else_block, merge_block);

                // Create a temporary for the result
                let result_ty = Type::I32; // Type inference would determine this
                let result_temp = self.builder.alloc_local(result_ty, false, None);

                // In merge block, we'd need phi nodes, but for simplicity we'll just use the then result
                self.current_block = Some(merge_block);
                self.builder.assign(
                    merge_block,
                    Place::Local(result_temp),
                    Rvalue::Use(then_result),
                );

                Ok(Operand::Move(Place::Local(result_temp)))
            }
            ExprKind::Call { func, args } => {
                let func_op = self.lower_expr_to_operand(func)?;
                let mut arg_ops = Vec::new();

                for arg in args {
                    arg_ops.push(self.lower_expr_to_operand(arg)?);
                }

                // Create a temporary for the result
                let result_ty = Type::I32; // Type inference would determine this
                let result_temp = self.builder.alloc_local(result_ty, false, None);

                // Create call terminator
                let next_block = self.builder.call(block, result_temp, func_op, arg_ops);
                self.current_block = Some(next_block);

                Ok(Operand::Move(Place::Local(result_temp)))
            }
            ExprKind::Block(exprs) => {
                if exprs.is_empty() {
                    Ok(Operand::Constant(Constant::Unit))
                } else {
                    // Lower all expressions, return the last one
                    let mut result = Operand::Constant(Constant::Unit);
                    for expr in exprs {
                        result = self.lower_expr_to_operand(expr)?;
                    }
                    Ok(result)
                }
            }
            _ => {
                // For unsupported expressions, return unit for now
                Ok(Operand::Constant(Constant::Unit))
            }
        }
    }

    /// Lower a literal to a constant
    fn lower_literal(lit: &Literal) -> Constant {
        match lit {
            Literal::Integer(i) => Constant::Int(i128::from(*i), Type::I32),
            Literal::Float(f) => Constant::Float(*f, Type::F64),
            Literal::String(s) => Constant::String(s.clone()),
            Literal::Bool(b) => Constant::Bool(*b),
            Literal::Unit => Constant::Unit,
        }
    }

    /// Lower binary operator
    fn lower_binary_op(op: AstBinOp) -> BinOp {
        match op {
            AstBinOp::Add => BinOp::Add,
            AstBinOp::Subtract => BinOp::Sub,
            AstBinOp::Multiply => BinOp::Mul,
            AstBinOp::Divide => BinOp::Div,
            AstBinOp::Modulo => BinOp::Rem,
            AstBinOp::Power => BinOp::Pow,
            AstBinOp::Equal => BinOp::Eq,
            AstBinOp::NotEqual => BinOp::Ne,
            AstBinOp::Less => BinOp::Lt,
            AstBinOp::LessEqual => BinOp::Le,
            AstBinOp::Greater => BinOp::Gt,
            AstBinOp::GreaterEqual => BinOp::Ge,
            AstBinOp::And => BinOp::And,
            AstBinOp::Or => BinOp::Or,
            AstBinOp::BitwiseAnd => BinOp::BitAnd,
            AstBinOp::BitwiseOr => BinOp::BitOr,
            AstBinOp::BitwiseXor => BinOp::BitXor,
            AstBinOp::LeftShift => BinOp::Shl,
            AstBinOp::RightShift => BinOp::Shr,
        }
    }

    /// Lower unary operator
    fn lower_unary_op(op: AstUnOp) -> UnOp {
        match op {
            AstUnOp::Negate => UnOp::Neg,
            AstUnOp::Not => UnOp::Not,
            AstUnOp::BitwiseNot => UnOp::BitNot,
        }
    }

    /// Convert AST Type to MIR Type
    fn ast_to_mir_type(&self, ast_ty: &AstType) -> Type {
        use crate::frontend::ast::TypeKind;
        match &ast_ty.kind {
            TypeKind::Named(name) => match name.as_str() {
                "bool" => Type::Bool,
                "i8" => Type::I8,
                "i16" => Type::I16,
                "i32" => Type::I32,
                "i64" => Type::I64,
                "i128" => Type::I128,
                "u8" => Type::U8,
                "u16" => Type::U16,
                "u32" => Type::U32,
                "u64" => Type::U64,
                "u128" => Type::U128,
                "f32" => Type::F32,
                "f64" => Type::F64,
                "String" => Type::String,
                "()" => Type::Unit,
                _ => Type::UserType(name.clone()),
            },
            TypeKind::Generic { base, params } => {
                match base.as_str() {
                    "Vec" if params.len() == 1 => {
                        Type::Vec(Box::new(self.ast_to_mir_type(&params[0])))
                    }
                    "Array" if params.len() == 1 => {
                        // For simplicity, treat arrays as vectors for now
                        Type::Vec(Box::new(self.ast_to_mir_type(&params[0])))
                    }
                    _ => Type::UserType(base.clone()),
                }
            }
            TypeKind::Optional(inner) => {
                // For simplicity, treat optionals as user types for now
                Type::UserType(format!("Option<{:?}>", inner.kind))
            }
            TypeKind::Function { params, ret } => {
                let param_types = params.iter().map(|p| self.ast_to_mir_type(p)).collect();
                Type::FnPtr(param_types, Box::new(self.ast_to_mir_type(ret)))
            }
            TypeKind::List(inner) => Type::Vec(Box::new(self.ast_to_mir_type(inner))),
        }
    }

    /// Infer result type for binary operations
    fn infer_binary_result_type(op: AstBinOp) -> Type {
        match op {
            AstBinOp::Add
            | AstBinOp::Subtract
            | AstBinOp::Multiply
            | AstBinOp::Divide
            | AstBinOp::Modulo
            | AstBinOp::Power => Type::I32,
            AstBinOp::Equal
            | AstBinOp::NotEqual
            | AstBinOp::Less
            | AstBinOp::LessEqual
            | AstBinOp::Greater
            | AstBinOp::GreaterEqual
            | AstBinOp::And
            | AstBinOp::Or => Type::Bool,
            AstBinOp::BitwiseAnd
            | AstBinOp::BitwiseOr
            | AstBinOp::BitwiseXor
            | AstBinOp::LeftShift
            | AstBinOp::RightShift => Type::I32,
        }
    }

    /// Infer result type for unary operations
    fn infer_unary_result_type(op: AstUnOp) -> Type {
        match op {
            AstUnOp::Negate | AstUnOp::BitwiseNot => Type::I32,
            AstUnOp::Not => Type::Bool,
        }
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::Parser;

    #[test]
    fn test_lower_literal() -> Result<()> {
        let mut parser = Parser::new("42");
        let ast = parser.parse()?;

        let mut ctx = LoweringContext::new();
        let program = ctx.lower_expr(&ast)?;

        assert_eq!(program.entry, "main");
        assert!(program.functions.contains_key("main"));

        Ok(())
    }

    #[test]
    fn test_lower_binary_expr() -> Result<()> {
        let mut parser = Parser::new("1 + 2");
        let ast = parser.parse()?;

        let mut ctx = LoweringContext::new();
        let program = ctx.lower_expr(&ast)?;

        let main_func = &program.functions["main"];
        assert!(!main_func.blocks.is_empty());

        Ok(())
    }

    #[test]
    fn test_lower_function() -> Result<()> {
        let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }");
        let ast = parser.parse()?;

        let mut ctx = LoweringContext::new();
        let program = ctx.lower_expr(&ast)?;

        assert!(program.functions.contains_key("add"));
        let func = &program.functions["add"];
        assert_eq!(func.params.len(), 2);

        Ok(())
    }

    #[test]
    fn test_lower_if_expr() -> Result<()> {
        let mut parser = Parser::new("if true { 1 } else { 2 }");
        let ast = parser.parse()?;

        let mut ctx = LoweringContext::new();
        let program = ctx.lower_expr(&ast)?;

        let main_func = &program.functions["main"];
        // Should have multiple blocks for if/else
        assert!(main_func.blocks.len() > 1);

        Ok(())
    }
}
