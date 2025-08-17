//! Type inference engine using Algorithm W

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Param, Pattern, TypeKind, UnaryOp};
use crate::middleend::environment::TypeEnv;
use crate::middleend::types::{MonoType, TyVar, TyVarGenerator, TypeScheme};
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
    /// Deferred constraints for later resolution
    constraints: Vec<(TyVar, TyVar)>,
}

impl InferenceContext {
    #[must_use]
    pub fn new() -> Self {
        InferenceContext {
            gen: TyVarGenerator::new(),
            unifier: Unifier::new(),
            env: TypeEnv::standard(),
            constraints: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_env(env: TypeEnv) -> Self {
        InferenceContext {
            gen: TyVarGenerator::new(),
            unifier: Unifier::new(),
            env,
            constraints: Vec::new(),
        }
    }

    /// Infer the type of an expression
    ///
    /// # Errors
    ///
    /// Returns an error if type inference fails (type error, undefined variable, etc.)
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn infer(&mut self, expr: &Expr) -> Result<MonoType> {
        let result = self.infer_expr(expr)?;
        self.solve_constraints();
        Ok(result)
    }

    /// Solve deferred constraints
    fn solve_constraints(&mut self) {
        while let Some((a, b)) = self.constraints.pop() {
            // Convert TyVar to MonoType for unification
            let ty_a = MonoType::Var(a);
            let ty_b = MonoType::Var(b);
            // Ignore failures for now - this is a simplified implementation
            let _ = self.unifier.unify(&ty_a, &ty_b);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Literal(lit) => Ok(Self::infer_literal(lit)),
            ExprKind::Identifier(name) => self.infer_identifier(name),
            ExprKind::StringInterpolation { parts } => {
                // Check that all expression parts are well-typed
                for part in parts {
                    if let crate::frontend::ast::StringPart::Expr(expr) = part {
                        let _ = self.infer_expr(expr)?;
                    }
                }
                // String interpolation always produces a String
                Ok(MonoType::Named("String".to_string()))
            }
            ExprKind::Binary { left, op, right } => self.infer_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.infer_unary(*op, operand),
            ExprKind::Try { expr } => self.infer_try(expr),
            ExprKind::TryCatch {
                try_block,
                catch_var,
                catch_block,
            } => self.infer_try_catch(try_block, catch_var, catch_block),
            ExprKind::Await { expr } => self.infer_await(expr),
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
                is_async,
                ..
            } => self.infer_function(name, params, body, return_type.as_ref(), *is_async),
            ExprKind::Lambda { params, body } => self.infer_lambda(params, body),
            ExprKind::Call { func, args } => self.infer_call(func, args),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.infer_method_call(receiver, method, args),
            ExprKind::Block(exprs) => self.infer_block(exprs),
            ExprKind::List(elements) => self.infer_list(elements),
            ExprKind::ListComprehension {
                element,
                variable,
                iterable,
                condition,
            } => self.infer_list_comprehension(element, variable, iterable, condition.as_deref()),
            ExprKind::Match { expr, arms } => self.infer_match(expr, arms),
            ExprKind::For { var, iter, body } => self.infer_for(var, iter, body),
            ExprKind::While { condition, body } => self.infer_while(condition, body),
            ExprKind::Range { start, end, .. } => self.infer_range(start, end),
            ExprKind::Pipeline { expr, stages } => self.infer_pipeline(expr, stages),
            ExprKind::Import { .. } => Ok(MonoType::Unit), // Imports don't have runtime values
            ExprKind::DataFrame { .. } => Ok(MonoType::Named("DataFrame".to_string())), // DataFrame type
            ExprKind::Struct { .. } => {
                // Struct definitions return Unit, they just register the type
                Ok(MonoType::Unit)
            }
            ExprKind::StructLiteral { name, fields: _ } => {
                // For now, return a named type for the struct
                // In a full implementation, we'd validate fields against the struct definition
                Ok(MonoType::Named(name.clone()))
            }
            ExprKind::ObjectLiteral { fields } => {
                // Object literals are anonymous objects with dynamic fields
                // Type check each field value
                for field in fields {
                    match field {
                        crate::frontend::ast::ObjectField::KeyValue { value, .. } => {
                            let _ = self.infer_expr(value)?;
                        }
                        crate::frontend::ast::ObjectField::Spread { expr } => {
                            let _ = self.infer_expr(expr)?;
                        }
                    }
                }
                // Return a generic object type
                Ok(MonoType::Named("Object".to_string()))
            }
            ExprKind::FieldAccess { object, field: _ } => {
                // Infer the type of the object
                let _object_ty = self.infer_expr(object)?;

                // For now, return a fresh type variable
                // In a full implementation, we'd look up the field type in the struct definition
                Ok(MonoType::Var(self.gen.fresh()))
            }
            ExprKind::Trait { .. } => {
                // Trait definitions return Unit, they just register the trait
                Ok(MonoType::Unit)
            }
            ExprKind::Impl { .. } => {
                // Impl blocks return Unit, they just provide implementations
                Ok(MonoType::Unit)
            }
            ExprKind::Actor { name: _, .. } => {
                // Actor definitions return Unit, they register the actor type
                // In a full implementation, we'd register the actor in the type environment
                Ok(MonoType::Unit)
            }
            ExprKind::Send { actor, message } => {
                // Type check the actor and message
                let _actor_ty = self.infer_expr(actor)?;
                let _message_ty = self.infer_expr(message)?;
                // Send operations return Unit (fire-and-forget)
                Ok(MonoType::Unit)
            }
            ExprKind::Ask {
                actor,
                message,
                timeout,
            } => {
                // Type check the actor, message, and optional timeout
                let _actor_ty = self.infer_expr(actor)?;
                let _message_ty = self.infer_expr(message)?;
                if let Some(t) = timeout {
                    let timeout_ty = self.infer_expr(t)?;
                    // Timeout should be a duration/number type
                    self.unifier.unify(&timeout_ty, &MonoType::Int)?;
                }
                // Ask operations return the response type (for now, a type variable)
                Ok(MonoType::Var(self.gen.fresh()))
            }
            ExprKind::Break { .. } | ExprKind::Continue { .. } => {
                // Break and continue don't return a value (they diverge)
                // In Rust, they have type ! (never), but we'll use Unit for simplicity
                Ok(MonoType::Unit)
            }
        }
    }

    fn infer_literal(lit: &Literal) -> MonoType {
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

    fn infer_try(&mut self, expr: &Expr) -> Result<MonoType> {
        // The expression must be Result<T, E>
        let expr_ty = self.infer_expr(expr)?;

        // Create fresh type variables for ok and error types
        let ok_ty = MonoType::Var(self.gen.fresh());
        let err_ty = MonoType::Var(self.gen.fresh());

        // Unify with Result type
        let result_ty = MonoType::Result(Box::new(ok_ty.clone()), Box::new(err_ty));
        self.unifier.unify(&expr_ty, &result_ty)?;

        // The ? operator returns the Ok value or propagates the error
        Ok(self.unifier.apply(&ok_ty))
    }

    fn infer_try_catch(
        &mut self,
        try_block: &Expr,
        catch_var: &str,
        catch_block: &Expr,
    ) -> Result<MonoType> {
        // The try block can return any type T
        let try_ty = self.infer_expr(try_block)?;

        // The catch variable is bound to an error type
        // For now, we'll use a generic error type
        let error_ty = MonoType::Named("Error".to_string());

        // Extend environment with catch variable and infer catch block
        let old_env = self.env.clone();
        self.env = self.env.extend(catch_var, TypeScheme::mono(error_ty));
        let catch_ty = self.infer_expr(catch_block)?;
        self.env = old_env;

        // Both blocks must return the same type
        self.unifier.unify(&try_ty, &catch_ty)?;

        Ok(self.unifier.apply(&try_ty))
    }

    fn infer_await(&mut self, expr: &Expr) -> Result<MonoType> {
        // The expression must be a Future<Output = T>
        let expr_ty = self.infer_expr(expr)?;

        // For now, we'll just return the inner type
        // In a full implementation, we'd check for Future trait
        if let MonoType::Named(name) = &expr_ty {
            if name.starts_with("Future") {
                // Extract the output type
                return Ok(MonoType::Var(self.gen.fresh()));
            }
        }

        // For now, just return a fresh type variable
        Ok(MonoType::Var(self.gen.fresh()))
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
        is_async: bool,
    ) -> Result<MonoType> {
        // Create fresh type variables for parameters
        let mut param_types = Vec::new();
        let old_env = self.env.clone();

        for param in params {
            let param_ty =
                if param.ty.kind == crate::frontend::ast::TypeKind::Named("Any".to_string()) {
                    // Untyped parameter - create fresh type variable
                    MonoType::Var(self.gen.fresh())
                } else {
                    // Convert AST type to MonoType
                    Self::ast_type_to_mono_static(&param.ty)?
                };
            param_types.push(param_ty.clone());
            self.env = self.env.extend(&param.name, TypeScheme::mono(param_ty));
        }

        // Add function itself to environment for recursion
        let result_var = MonoType::Var(self.gen.fresh());
        let func_type = param_types
            .iter()
            .rev()
            .fold(result_var.clone(), |acc, param_ty| {
                MonoType::Function(Box::new(param_ty.clone()), Box::new(acc))
            });
        self.env = self.env.extend(name, TypeScheme::mono(func_type.clone()));

        // Infer body type
        let body_ty = self.infer_expr(body)?;
        self.unifier.unify(&result_var, &body_ty)?;

        self.env = old_env;

        let final_type = self.unifier.apply(&func_type);

        // If async, wrap the return type in a Future
        if is_async {
            // For simplicity, just mark it as a Named type
            // In a full implementation, we'd properly wrap in Future<Output = T>
            Ok(MonoType::Named(format!("Future<{final_type:?}>")))
        } else {
            Ok(final_type)
        }
    }

    fn infer_lambda(&mut self, params: &[Param], body: &Expr) -> Result<MonoType> {
        let old_env = self.env.clone();

        // Create type variables for parameters
        let mut param_types = Vec::new();
        for param in params {
            let param_ty = match &param.ty.kind {
                TypeKind::Named(name) if name == "Any" || name == "_" => {
                    // Untyped parameter - create fresh type variable
                    MonoType::Var(self.gen.fresh())
                }
                _ => {
                    // Convert AST type to MonoType
                    Self::ast_type_to_mono_static(&param.ty)?
                }
            };
            param_types.push(param_ty.clone());
            self.env = self.env.extend(&param.name, TypeScheme::mono(param_ty));
        }

        // Infer body type
        let body_ty = self.infer_expr(body)?;

        // Restore environment
        self.env = old_env;

        // Build function type from parameters and body
        let lambda_type = param_types.iter().rev().fold(body_ty, |acc, param_ty| {
            MonoType::Function(Box::new(param_ty.clone()), Box::new(acc))
        });

        Ok(self.unifier.apply(&lambda_type))
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

    fn infer_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
    ) -> Result<MonoType> {
        let receiver_ty = self.infer_expr(receiver)?;

        // For now, we'll handle some common methods
        // In a complete implementation, we'd have a method resolution system
        match (method, &receiver_ty) {
            // List and String length methods
            ("len" | "length", MonoType::List(_) | MonoType::String) => {
                if !args.is_empty() {
                    bail!("Method {} takes no arguments", method);
                }
                Ok(MonoType::Int)
            }
            ("push", MonoType::List(elem_ty)) => {
                if args.len() != 1 {
                    bail!("Method push takes exactly one argument");
                }
                let arg_ty = self.infer_expr(&args[0])?;
                self.unifier.unify(&arg_ty, elem_ty)?;
                Ok(MonoType::Unit)
            }
            ("pop", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method pop takes no arguments");
                }
                Ok(MonoType::Optional(elem_ty.clone()))
            }
            // Vec extension methods
            ("sorted", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method sorted takes no arguments");
                }
                Ok(MonoType::List(elem_ty.clone()))
            }
            ("sum", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method sum takes no arguments");
                }
                // Sum returns the element type (assuming numeric)
                Ok(*elem_ty.clone())
            }
            ("reversed", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method reversed takes no arguments");
                }
                Ok(MonoType::List(elem_ty.clone()))
            }
            ("unique", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method unique takes no arguments");
                }
                Ok(MonoType::List(elem_ty.clone()))
            }
            ("min", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method min takes no arguments");
                }
                Ok(MonoType::Optional(elem_ty.clone()))
            }
            ("max", MonoType::List(elem_ty)) => {
                if !args.is_empty() {
                    bail!("Method max takes no arguments");
                }
                Ok(MonoType::Optional(elem_ty.clone()))
            }
            ("chars", MonoType::String) => {
                if !args.is_empty() {
                    bail!("Method chars takes no arguments");
                }
                Ok(MonoType::List(Box::new(MonoType::String))) // List of chars (as strings for now)
            }
            // DataFrame methods
            ("filter" | "groupby" | "agg" | "select", MonoType::Named(name))
                if name == "DataFrame" =>
            {
                // These methods return a DataFrame
                Ok(MonoType::Named("DataFrame".to_string()))
            }
            ("mean" | "std" | "sum" | "count", MonoType::Named(name)) if name == "DataFrame" => {
                // These aggregation methods return numeric types
                Ok(MonoType::Float)
            }
            ("col", MonoType::Named(name)) if name == "DataFrame" => {
                // Column selection returns a Series
                Ok(MonoType::Named("Series".to_string()))
            }
            // Generic case - treat as a function call with receiver as first argument
            _ => {
                // Look up method in environment
                if let Some(scheme) = self.env.lookup(method) {
                    let method_ty = self.env.instantiate(scheme, &mut self.gen);

                    // Create type for the method call (receiver is first argument)
                    let result_ty = MonoType::Var(self.gen.fresh());
                    let mut expected_func_ty = result_ty.clone();

                    for arg in args.iter().rev() {
                        let arg_ty = self.infer_expr(arg)?;
                        expected_func_ty =
                            MonoType::Function(Box::new(arg_ty), Box::new(expected_func_ty));
                    }

                    // Add receiver as first argument
                    expected_func_ty =
                        MonoType::Function(Box::new(receiver_ty), Box::new(expected_func_ty));

                    self.unifier.unify(&method_ty, &expected_func_ty)?;
                    Ok(self.unifier.apply(&result_ty))
                } else {
                    // Unknown method - for now just return a type variable
                    Ok(MonoType::Var(self.gen.fresh()))
                }
            }
        }
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

    fn infer_list_comprehension(
        &mut self,
        element: &Expr,
        variable: &str,
        iterable: &Expr,
        condition: Option<&Expr>,
    ) -> Result<MonoType> {
        // Type check the iterable - must be a list
        let iterable_ty = self.infer_expr(iterable)?;
        let elem_ty = MonoType::Var(self.gen.fresh());
        self.unifier
            .unify(&iterable_ty, &MonoType::List(Box::new(elem_ty.clone())))?;

        // Save the old environment and add the loop variable
        let old_env = self.env.clone();
        self.env = self
            .env
            .extend(variable, TypeScheme::mono(self.unifier.apply(&elem_ty)));

        // Type check the optional condition (must be bool)
        if let Some(cond) = condition {
            let cond_ty = self.infer_expr(cond)?;
            self.unifier.unify(&cond_ty, &MonoType::Bool)?;
        }

        // Type check the element expression
        let result_elem_ty = self.infer_expr(element)?;

        // Restore the environment
        self.env = old_env;

        // Return List<T> where T is the type of the element expression
        Ok(MonoType::List(Box::new(
            self.unifier.apply(&result_elem_ty),
        )))
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
                let lit_ty = Self::infer_literal(lit);
                self.unifier.unify(expected_ty, &lit_ty)
            }
            Pattern::Identifier(name) => {
                // Bind the identifier to the expected type
                self.env = self.env.extend(name, TypeScheme::mono(expected_ty.clone()));
                Ok(())
            }
            Pattern::List(patterns) => {
                let elem_ty = MonoType::Var(self.gen.fresh());
                self.unifier
                    .unify(expected_ty, &MonoType::List(Box::new(elem_ty.clone())))?;

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
        self.unifier
            .unify(&iter_ty, &MonoType::List(Box::new(elem_ty.clone())))?;

        // Bind loop variable and infer body
        let old_env = self.env.clone();
        self.env = self.env.extend(var, TypeScheme::mono(elem_ty));
        let body_ty = self.infer_expr(body)?;
        self.env = old_env;

        // For loops return Unit
        self.unifier.unify(&body_ty, &MonoType::Unit)?;
        Ok(MonoType::Unit)
    }

    fn infer_while(&mut self, condition: &Expr, body: &Expr) -> Result<MonoType> {
        // Condition must be Bool
        let cond_ty = self.infer_expr(condition)?;
        self.unifier.unify(&cond_ty, &MonoType::Bool)?;

        // Type check body
        let body_ty = self.infer_expr(body)?;
        self.unifier.unify(&body_ty, &MonoType::Unit)?;

        // While loops return unit
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
            let expected_func =
                MonoType::Function(Box::new(current_ty.clone()), Box::new(result_ty.clone()));

            self.unifier.unify(&stage_ty, &expected_func)?;
            current_ty = self.unifier.apply(&result_ty);
        }

        Ok(current_ty)
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
            TypeKind::Generic { base, params } => {
                // For now, treat generic types as their base type
                // Full generic inference will be implemented later
                match base.as_str() {
                    "Vec" | "List" => {
                        if let Some(first_param) = params.first() {
                            MonoType::List(Box::new(Self::ast_type_to_mono_static(first_param)?))
                        } else {
                            MonoType::List(Box::new(MonoType::Var(TyVarGenerator::new().fresh())))
                        }
                    }
                    "Option" => {
                        if let Some(first_param) = params.first() {
                            MonoType::Optional(Box::new(Self::ast_type_to_mono_static(
                                first_param,
                            )?))
                        } else {
                            MonoType::Optional(Box::new(MonoType::Var(
                                TyVarGenerator::new().fresh(),
                            )))
                        }
                    }
                    _ => MonoType::Named(base.clone()),
                }
            }
            TypeKind::Optional(inner) => {
                MonoType::Optional(Box::new(Self::ast_type_to_mono_static(inner)?))
            }
            TypeKind::List(inner) => {
                MonoType::List(Box::new(Self::ast_type_to_mono_static(inner)?))
            }
            TypeKind::Function { params, ret } => {
                let ret_ty = Self::ast_type_to_mono_static(ret)?;
                let result: Result<MonoType> =
                    params.iter().rev().try_fold(ret_ty, |acc, param| {
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
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
        assert_eq!(
            infer_str("if true { 1 } else { 2 }").unwrap(),
            MonoType::Int
        );
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

    #[test]
    fn test_infer_lambda() {
        // Simple lambda: |x| x + 1
        let result = infer_str("|x| x + 1").unwrap();
        match result {
            MonoType::Function(arg, ret) => {
                assert!(matches!(arg.as_ref(), MonoType::Int));
                assert!(matches!(ret.as_ref(), MonoType::Int));
            }
            _ => panic!("Expected function type for lambda"),
        }

        // Lambda with multiple params: |x, y| x * y
        let result = infer_str("|x, y| x * y").unwrap();
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
            _ => panic!("Expected function type for lambda"),
        }

        // Lambda with no params: || 42
        let result = infer_str("|| 42").unwrap();
        assert_eq!(result, MonoType::Int);

        // Lambda used in let binding
        let result = infer_str("let f = |x| x + 1 in f(5)").unwrap();
        assert_eq!(result, MonoType::Int);
    }
}
