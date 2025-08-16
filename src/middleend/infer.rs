//! Type inference engine using Algorithm W

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Param, Pattern, UnaryOp};
use crate::middleend::environment::TypeEnv;
use crate::middleend::types::{MonoType, TyVarGenerator, TypeScheme};
use crate::middleend::unify::Unifier;
use anyhow::{bail, Result};

/// Type inference context
pub struct InferenceContext {
    /// Type variable generator
    gen: TyVarGenerator,
    /// Unification engine
    unifier: Unifier,
    /// Type environment
    env: TypeEnv,
}

impl InferenceContext {
    #[must_use]
    pub fn new() -> Self {
        InferenceContext {
            gen: TyVarGenerator::new(),
            unifier: Unifier::new(),
            env: TypeEnv::standard(),
        }
    }

    #[must_use]
    pub fn with_env(env: TypeEnv) -> Self {
        InferenceContext {
            gen: TyVarGenerator::new(),
            unifier: Unifier::new(),
            env,
        }
    }

    /// Infer the type of an expression
    /// 
    /// # Errors
    /// 
    /// Returns an error if type inference fails (type error, undefined variable, etc.)
    pub fn infer(&mut self, expr: &Expr) -> Result<MonoType> {
        self.infer_expr(expr)
    }

    fn infer_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(self.infer_literal(lit)),
            ExprKind::Identifier(name) => self.infer_identifier(name),
            ExprKind::Binary { left, op, right } => self.infer_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.infer_unary(*op, operand),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.infer_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Let { name, value, body } => self.infer_let(name, value, body),
            ExprKind::Function {
                name,
                params,
                body,
                return_type,
            } => self.infer_function(name, params, body, return_type.as_ref()),
            ExprKind::Call { func, args } => self.infer_call(func, args),
            ExprKind::Block(exprs) => self.infer_block(exprs),
            ExprKind::List(elements) => self.infer_list(elements),
            ExprKind::Match { expr, arms } => self.infer_match(expr, arms),
            ExprKind::For { var, iter, body } => self.infer_for(var, iter, body),
            ExprKind::Range { start, end, .. } => self.infer_range(start, end),
            ExprKind::Pipeline { expr, stages } => self.infer_pipeline(expr, stages),
            ExprKind::Import { .. } => Ok(MonoType::Unit), // Imports don't have runtime values
        }
    }

    fn infer_literal(&self, lit: &Literal) -> MonoType {
        match lit {
            Literal::Integer(_) => MonoType::Int,
            Literal::Float(_) => MonoType::Float,
            Literal::String(_) => MonoType::String,
            Literal::Bool(_) => MonoType::Bool,
            Literal::Unit => MonoType::Unit,
        }
    }

    fn infer_identifier(&mut self, name: &str) -> Result<MonoType> {
        match self.env.lookup(name) {
            Some(scheme) => Ok(self.env.instantiate(scheme, &mut self.gen)),
            None => bail!("Undefined variable: {}", name),
        }
    }

    fn infer_binary(&mut self, left: &Expr, op: BinaryOp, right: &Expr) -> Result<MonoType> {
        let left_ty = self.infer_expr(left)?;
        let right_ty = self.infer_expr(right)?;

        match op {
            // Arithmetic operators
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                // Both operands must be numeric and same type
                self.unifier.unify(&left_ty, &right_ty)?;
                // For now, assume Int (could be Float too)
                self.unifier.unify(&left_ty, &MonoType::Int)?;
                Ok(MonoType::Int)
            }
            BinaryOp::Power => {
                self.unifier.unify(&left_ty, &MonoType::Int)?;
                self.unifier.unify(&right_ty, &MonoType::Int)?;
                Ok(MonoType::Int)
            }
            // Comparison operators
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => {
                // Operands must have same type
                self.unifier.unify(&left_ty, &right_ty)?;
                Ok(MonoType::Bool)
            }
            // Boolean operators
            BinaryOp::And | BinaryOp::Or => {
                self.unifier.unify(&left_ty, &MonoType::Bool)?;
                self.unifier.unify(&right_ty, &MonoType::Bool)?;
                Ok(MonoType::Bool)
            }
            // Bitwise operators
            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::LeftShift
            | BinaryOp::RightShift => {
                self.unifier.unify(&left_ty, &MonoType::Int)?;
                self.unifier.unify(&right_ty, &MonoType::Int)?;
                Ok(MonoType::Int)
            }
        }
    }

    fn infer_unary(&mut self, op: UnaryOp, operand: &Expr) -> Result<MonoType> {
        let operand_ty = self.infer_expr(operand)?;

        match op {
            UnaryOp::Not => {
                self.unifier.unify(&operand_ty, &MonoType::Bool)?;
                Ok(MonoType::Bool)
            }
            UnaryOp::Negate => {
                // Can negate Int or Float
                self.unifier.unify(&operand_ty, &MonoType::Int)?;
                Ok(MonoType::Int)
            }
            UnaryOp::BitwiseNot => {
                self.unifier.unify(&operand_ty, &MonoType::Int)?;
                Ok(MonoType::Int)
            }
        }
    }

    fn infer_if(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<MonoType> {
        // Condition must be Bool
        let cond_ty = self.infer_expr(condition)?;
        self.unifier.unify(&cond_ty, &MonoType::Bool)?;

        let then_ty = self.infer_expr(then_branch)?;

        if let Some(else_expr) = else_branch {
            let else_ty = self.infer_expr(else_expr)?;
            // Both branches must have same type
            self.unifier.unify(&then_ty, &else_ty)?;
            Ok(self.unifier.apply(&then_ty))
        } else {
            // No else branch means Unit type
            self.unifier.unify(&then_ty, &MonoType::Unit)?;
            Ok(MonoType::Unit)
        }
    }

    fn infer_let(&mut self, name: &str, value: &Expr, body: &Expr) -> Result<MonoType> {
        // Infer type of value
        let value_ty = self.infer_expr(value)?;

        // Generalize the value type
        let scheme = self.env.generalize(value_ty);

        // Extend environment and infer body
        let old_env = self.env.clone();
        self.env = self.env.extend(name, scheme);
        let body_ty = self.infer_expr(body)?;
        self.env = old_env;

        Ok(body_ty)
    }

    fn infer_function(
        &mut self,
        name: &str,
        params: &[Param],
        body: &Expr,
        _return_type: Option<&crate::frontend::ast::Type>,
    ) -> Result<MonoType> {
        // Create fresh type variables for parameters
        let mut param_types = Vec::new();
        let old_env = self.env.clone();

        for param in params {
            let param_ty = if param.ty.kind == crate::frontend::ast::TypeKind::Named("Any".to_string()) {
                // Untyped parameter - create fresh type variable
                MonoType::Var(self.gen.fresh())
            } else {
                // Convert AST type to MonoType
                self.ast_type_to_mono(&param.ty)?
            };
            param_types.push(param_ty.clone());
            self.env = self.env.extend(&param.name, TypeScheme::mono(param_ty));
        }

        // Add function itself to environment for recursion
        let result_var = MonoType::Var(self.gen.fresh());
        let func_type = param_types.iter().rev().fold(result_var.clone(), |acc, param_ty| {
            MonoType::Function(Box::new(param_ty.clone()), Box::new(acc))
        });
        self.env = self.env.extend(name, TypeScheme::mono(func_type.clone()));

        // Infer body type
        let body_ty = self.infer_expr(body)?;
        self.unifier.unify(&result_var, &body_ty)?;

        self.env = old_env;

        Ok(self.unifier.apply(&func_type))
    }

    fn infer_call(&mut self, func: &Expr, args: &[Expr]) -> Result<MonoType> {
        let func_ty = self.infer_expr(func)?;

        // Create type for the function we expect
        let result_ty = MonoType::Var(self.gen.fresh());
        let mut expected_func_ty = result_ty.clone();

        for arg in args.iter().rev() {
            let arg_ty = self.infer_expr(arg)?;
            expected_func_ty = MonoType::Function(Box::new(arg_ty), Box::new(expected_func_ty));
        }

        // Unify with actual function type
        self.unifier.unify(&func_ty, &expected_func_ty)?;

        Ok(self.unifier.apply(&result_ty))
    }

    fn infer_block(&mut self, exprs: &[Expr]) -> Result<MonoType> {
        if exprs.is_empty() {
            return Ok(MonoType::Unit);
        }

        let mut last_ty = MonoType::Unit;
        for expr in exprs {
            last_ty = self.infer_expr(expr)?;
        }

        Ok(last_ty)
    }

    fn infer_list(&mut self, elements: &[Expr]) -> Result<MonoType> {
        if elements.is_empty() {
            // Empty list with fresh type variable
            let elem_ty = MonoType::Var(self.gen.fresh());
            return Ok(MonoType::List(Box::new(elem_ty)));
        }

        // All elements must have same type
        let first_ty = self.infer_expr(&elements[0])?;
        for elem in &elements[1..] {
            let elem_ty = self.infer_expr(elem)?;
            self.unifier.unify(&first_ty, &elem_ty)?;
        }

        Ok(MonoType::List(Box::new(self.unifier.apply(&first_ty))))
    }

    fn infer_match(
        &mut self,
        expr: &Expr,
        arms: &[crate::frontend::ast::MatchArm],
    ) -> Result<MonoType> {
        let expr_ty = self.infer_expr(expr)?;

        if arms.is_empty() {
            bail!("Match expression must have at least one arm");
        }

        // All arms must return same type
        let result_ty = MonoType::Var(self.gen.fresh());

        for arm in arms {
            // Infer pattern and bind variables
            let old_env = self.env.clone();
            self.infer_pattern(&arm.pattern, &expr_ty)?;

            // Check guard if present
            if let Some(guard) = &arm.guard {
                let guard_ty = self.infer_expr(guard)?;
                self.unifier.unify(&guard_ty, &MonoType::Bool)?;
            }

            // Infer body type
            let body_ty = self.infer_expr(&arm.body)?;
            self.unifier.unify(&result_ty, &body_ty)?;

            self.env = old_env;
        }

        Ok(self.unifier.apply(&result_ty))
    }

    fn infer_pattern(&mut self, pattern: &Pattern, expected_ty: &MonoType) -> Result<()> {
        match pattern {
            Pattern::Wildcard => Ok(()),
            Pattern::Literal(lit) => {
                let lit_ty = self.infer_literal(lit);
                self.unifier.unify(expected_ty, &lit_ty)
            }
            Pattern::Identifier(name) => {
                // Bind the identifier to the expected type
                self.env = self.env.extend(name, TypeScheme::mono(expected_ty.clone()));
                Ok(())
            }
            Pattern::List(patterns) => {
                let elem_ty = MonoType::Var(self.gen.fresh());
                self.unifier.unify(expected_ty, &MonoType::List(Box::new(elem_ty.clone())))?;

                for pat in patterns {
                    self.infer_pattern(pat, &elem_ty)?;
                }
                Ok(())
            }
        }
    }

    fn infer_for(&mut self, var: &str, iter: &Expr, body: &Expr) -> Result<MonoType> {
        let iter_ty = self.infer_expr(iter)?;

        // Iterator should be a list
        let elem_ty = MonoType::Var(self.gen.fresh());
        self.unifier.unify(&iter_ty, &MonoType::List(Box::new(elem_ty.clone())))?;

        // Bind loop variable and infer body
        let old_env = self.env.clone();
        self.env = self.env.extend(var, TypeScheme::mono(elem_ty));
        let body_ty = self.infer_expr(body)?;
        self.env = old_env;

        // For loops return Unit
        self.unifier.unify(&body_ty, &MonoType::Unit)?;
        Ok(MonoType::Unit)
    }

    fn infer_range(&mut self, start: &Expr, end: &Expr) -> Result<MonoType> {
        let start_ty = self.infer_expr(start)?;
        let end_ty = self.infer_expr(end)?;

        // Both must be integers
        self.unifier.unify(&start_ty, &MonoType::Int)?;
        self.unifier.unify(&end_ty, &MonoType::Int)?;

        // Range produces a list of integers
        Ok(MonoType::List(Box::new(MonoType::Int)))
    }

    fn infer_pipeline(
        &mut self,
        expr: &Expr,
        stages: &[crate::frontend::ast::PipelineStage],
    ) -> Result<MonoType> {
        let mut current_ty = self.infer_expr(expr)?;

        for stage in stages {
            // Each stage is a function applied to current value
            let stage_ty = self.infer_expr(&stage.op)?;

            // Create expected function type
            let result_ty = MonoType::Var(self.gen.fresh());
            let expected_func = MonoType::Function(
                Box::new(current_ty.clone()),
                Box::new(result_ty.clone()),
            );

            self.unifier.unify(&stage_ty, &expected_func)?;
            current_ty = self.unifier.apply(&result_ty);
        }

        Ok(current_ty)
    }

    fn ast_type_to_mono(&self, ty: &crate::frontend::ast::Type) -> Result<MonoType> {
        Self::ast_type_to_mono_static(ty)
    }
    
    fn ast_type_to_mono_static(ty: &crate::frontend::ast::Type) -> Result<MonoType> {
        use crate::frontend::ast::TypeKind;
        
        Ok(match &ty.kind {
            TypeKind::Named(name) => match name.as_str() {
                "i32" | "i64" => MonoType::Int,
                "f32" | "f64" => MonoType::Float,
                "bool" => MonoType::Bool,
                "String" | "str" => MonoType::String,
                "Any" => MonoType::Var(TyVarGenerator::new().fresh()),
                _ => MonoType::Named(name.clone()),
            },
            TypeKind::Optional(inner) => {
                MonoType::Optional(Box::new(Self::ast_type_to_mono_static(inner)?))
            }
            TypeKind::List(inner) => {
                MonoType::List(Box::new(Self::ast_type_to_mono_static(inner)?))
            }
            TypeKind::Function { params, ret } => {
                let ret_ty = Self::ast_type_to_mono_static(ret)?;
                let result: Result<MonoType> = params.iter().rev().try_fold(ret_ty, |acc, param| {
                    Ok(MonoType::Function(
                        Box::new(Self::ast_type_to_mono_static(param)?),
                        Box::new(acc),
                    ))
                });
                result?
            }
        })
    }

    /// Get the final inferred type for a type variable
    #[must_use]
    pub fn solve(&self, var: &crate::middleend::types::TyVar) -> MonoType {
        self.unifier.solve(var)
    }

    /// Apply current substitution to a type
    #[must_use]
    pub fn apply(&self, ty: &MonoType) -> MonoType {
        self.unifier.apply(ty)
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    fn infer_str(input: &str) -> Result<MonoType> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        let mut ctx = InferenceContext::new();
        ctx.infer(&expr)
    }

    #[test]
    fn test_infer_literals() {
        assert_eq!(infer_str("42").unwrap(), MonoType::Int);
        assert_eq!(infer_str("3.14").unwrap(), MonoType::Float);
        assert_eq!(infer_str("true").unwrap(), MonoType::Bool);
        assert_eq!(infer_str("\"hello\"").unwrap(), MonoType::String);
    }

    #[test]
    fn test_infer_arithmetic() {
        assert_eq!(infer_str("1 + 2").unwrap(), MonoType::Int);
        assert_eq!(infer_str("3 * 4").unwrap(), MonoType::Int);
        assert_eq!(infer_str("5 - 2").unwrap(), MonoType::Int);
    }

    #[test]
    fn test_infer_comparison() {
        assert_eq!(infer_str("1 < 2").unwrap(), MonoType::Bool);
        assert_eq!(infer_str("3 == 3").unwrap(), MonoType::Bool);
        assert_eq!(infer_str("true != false").unwrap(), MonoType::Bool);
    }

    #[test]
    fn test_infer_if() {
        assert_eq!(infer_str("if true { 1 } else { 2 }").unwrap(), MonoType::Int);
        assert_eq!(
            infer_str("if false { \"yes\" } else { \"no\" }").unwrap(),
            MonoType::String
        );
    }

    #[test]
    fn test_infer_let() {
        assert_eq!(infer_str("let x = 42 in x + 1").unwrap(), MonoType::Int);
        assert_eq!(
            infer_str("let f = 3.14 in let g = 2.71 in f").unwrap(),
            MonoType::Float
        );
    }

    #[test]
    fn test_infer_list() {
        assert_eq!(
            infer_str("[1, 2, 3]").unwrap(),
            MonoType::List(Box::new(MonoType::Int))
        );
        assert_eq!(
            infer_str("[true, false]").unwrap(),
            MonoType::List(Box::new(MonoType::Bool))
        );
    }

    #[test]
    fn test_infer_function() {
        let result = infer_str("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        match result {
            MonoType::Function(first_arg, remaining) => {
                assert!(matches!(first_arg.as_ref(), MonoType::Int));
                match remaining.as_ref() {
                    MonoType::Function(second_arg, return_type) => {
                        assert!(matches!(second_arg.as_ref(), MonoType::Int));
                        assert!(matches!(return_type.as_ref(), MonoType::Int));
                    }
                    _ => panic!("Expected function type"),
                }
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_type_errors() {
        assert!(infer_str("1 + true").is_err());
        assert!(infer_str("if 42 { 1 } else { 2 }").is_err());
        assert!(infer_str("[1, true, 3]").is_err());
    }
}