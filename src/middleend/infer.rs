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
            ExprKind::QualifiedName { module: _, name } => {
                // For now, just treat qualified names like regular identifiers
                // In a full implementation, we'd resolve the module and check its exports
                self.infer_identifier(name)
            }
            ExprKind::StringInterpolation { parts } => self.infer_string_interpolation(parts),
            ExprKind::Binary { left, op, right } => self.infer_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.infer_unary(*op, operand),
            ExprKind::Try { expr } => self.infer_try(expr),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => self.infer_try_catch(try_block, catch_clauses, finally_block.as_deref()),
            ExprKind::Throw { expr } => self.infer_throw(expr),
            ExprKind::Ok { value } => self.infer_result_ok(value),
            ExprKind::Err { error } => self.infer_result_err(error),
            ExprKind::Await { expr } => self.infer_await(expr),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.infer_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Let {
                name,
                type_annotation: _,
                value,
                body,
                is_mutable,
            } => self.infer_let(name, value, body, *is_mutable),
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
            ExprKind::Tuple(elements) => {
                // Infer tuple type
                let element_types: Result<Vec<_>> =
                    elements.iter().map(|e| self.infer_expr(e)).collect();
                Ok(MonoType::Tuple(element_types?))
            }
            ExprKind::ListComprehension {
                element,
                variable,
                iterable,
                condition,
            } => self.infer_list_comprehension(element, variable, iterable, condition.as_deref()),
            ExprKind::Match { expr, arms } => self.infer_match(expr, arms),
            ExprKind::For { var, iter, body } => self.infer_for(var, iter, body),
            ExprKind::While { condition, body } => self.infer_while(condition, body),
            ExprKind::Loop { body } => self.infer_loop(body),
            ExprKind::Range { start, end, .. } => self.infer_range(start, end),
            ExprKind::Pipeline { expr, stages } => self.infer_pipeline(expr, stages),
            ExprKind::Import { .. } | ExprKind::Export { .. } => Ok(MonoType::Unit), // Imports/exports don't have runtime values
            ExprKind::Module { body, .. } => {
                // Modules evaluate to their body type
                self.infer_expr(body)
            }
            ExprKind::DataFrame { columns } => self.infer_dataframe(columns),
            ExprKind::Struct { .. } => {
                // Struct definitions return Unit, they just register the type
                Ok(MonoType::Unit)
            }
            ExprKind::Enum { .. } => {
                // Enum definitions return Unit, they just register the type
                Ok(MonoType::Unit)
            }
            ExprKind::StructLiteral { name, fields: _ } => {
                // For now, return a named type for the struct
                // In a full implementation, we'd validate fields against the struct definition
                Ok(MonoType::Named(name.clone()))
            }
            ExprKind::ObjectLiteral { fields } => self.infer_object_literal(fields),
            ExprKind::FieldAccess { object, field: _ } => self.infer_field_access(object),
            ExprKind::IndexAccess { object, index: _ } => self.infer_index_access(object),
            ExprKind::Trait { .. } => {
                // Trait definitions return Unit, they just register the trait
                Ok(MonoType::Unit)
            }
            ExprKind::Impl { .. } => {
                // Impl blocks return Unit, they just provide implementations
                Ok(MonoType::Unit)
            }
            ExprKind::Extension { .. } => {
                // Extension blocks return Unit, they just provide implementations
                Ok(MonoType::Unit)
            }
            ExprKind::Actor { name: _, .. } => {
                // Actor definitions return Unit, they register the actor type
                // In a full implementation, we'd register the actor in the type environment
                Ok(MonoType::Unit)
            }
            ExprKind::Send { actor, message } => self.infer_send(actor, message),
            ExprKind::Ask {
                actor,
                message,
                timeout,
            } => self.infer_ask(actor, message, timeout.as_deref()),
            ExprKind::Break { .. } | ExprKind::Continue { .. } | ExprKind::Return { .. } => {
                // Break, continue, and return don't return a value (they diverge)
                // In Rust, they have type ! (never), but we'll use Unit for simplicity
                Ok(MonoType::Unit)
            }
            ExprKind::Assign { target, value } => self.infer_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.infer_compound_assign(target, *op, value)
            }
            ExprKind::PreIncrement { target }
            | ExprKind::PostIncrement { target }
            | ExprKind::PreDecrement { target }
            | ExprKind::PostDecrement { target } => self.infer_increment_decrement(target),
            ExprKind::DataFrameOperation { source, operation } => {
                self.infer_dataframe_operation(source, operation)
            }
            ExprKind::Some { value } => {
                let inner_type = self.infer_expr(value)?;
                Ok(MonoType::Optional(Box::new(inner_type)))
            }
            ExprKind::None => {
                // None is polymorphic - its type is Option<T> where T is inferred from context
                let type_var = MonoType::Var(self.gen.fresh());
                Ok(MonoType::Optional(Box::new(type_var)))
            }
            ExprKind::IfLet {
                pattern: _,
                expr,
                then_branch,
                else_branch,
            } => {
                // Type check the expression being matched
                let _expr_ty = self.infer_expr(expr)?;
                
                // Type check the branches
                let then_ty = self.infer_expr(then_branch)?;
                let else_ty = if let Some(else_expr) = else_branch {
                    self.infer_expr(else_expr)?
                } else {
                    MonoType::Unit
                };
                
                // Both branches should have the same type
                self.unifier.unify(&then_ty, &else_ty)?;
                Ok(then_ty)
            }
            ExprKind::WhileLet {
                pattern: _,
                expr,
                body,
            } => {
                // Type check the expression being matched
                let _expr_ty = self.infer_expr(expr)?;
                
                // Type check the body
                let _body_ty = self.infer_expr(body)?;
                
                // While-let expressions return Unit
                Ok(MonoType::Unit)
            }
            ExprKind::AsyncBlock { body } => self.infer_async_block(body),
        }
    }

    fn infer_literal(lit: &Literal) -> MonoType {
        match lit {
            Literal::Integer(_) => MonoType::Int,
            Literal::Float(_) => MonoType::Float,
            Literal::String(_) => MonoType::String,
            Literal::Bool(_) => MonoType::Bool,
            Literal::Char(_) => MonoType::Char,
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
            UnaryOp::Reference => {
                // Reference operator &x: T -> &T
                Ok(MonoType::Reference(Box::new(operand_ty)))
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
        catch_clauses: &[crate::frontend::ast::CatchClause],
        finally_block: Option<&Expr>,
    ) -> Result<MonoType> {
        // The try block can return any type T
        let try_ty = self.infer_expr(try_block)?;

        // Infer types for all catch clauses
        let mut catch_types = Vec::new();
        for clause in catch_clauses {
            let old_env = self.env.clone();

            // Bind catch variable with appropriate error type
            let error_ty = if let Some(ref exc_type) = clause.exception_type {
                MonoType::Named(exc_type.clone())
            } else {
                MonoType::Named("Error".to_string()) // Generic error
            };

            self.env = self
                .env
                .extend(&clause.variable, TypeScheme::mono(error_ty));

            // Check guard condition if present
            if let Some(ref condition) = clause.condition {
                let cond_ty = self.infer_expr(condition)?;
                self.unifier.unify(&cond_ty, &MonoType::Bool)?;
            }

            // Infer catch body type
            let catch_ty = self.infer_expr(&clause.body)?;
            catch_types.push(catch_ty);

            self.env = old_env;
        }

        // All catch clauses and try block must return the same type
        for catch_ty in &catch_types {
            self.unifier.unify(&try_ty, catch_ty)?;
        }

        // Infer finally block type (should be Unit)
        if let Some(finally) = finally_block {
            let finally_ty = self.infer_expr(finally)?;
            // Finally block's type is ignored, but we still check it
            // In a full implementation, we might want to ensure it's Unit
            drop(finally_ty);
        }

        Ok(self.unifier.apply(&try_ty))
    }

    fn infer_throw(&mut self, expr: &Expr) -> Result<MonoType> {
        // Infer the type of the expression being thrown
        let _expr_ty = self.infer_expr(expr)?;

        // The expression must implement Error trait
        // For now, we'll just ensure it's a valid type
        // In a more complete implementation, we'd check Error trait bounds

        // The throw expression itself has the Never type (!)
        // But we'll represent it as a generic type for now
        Ok(MonoType::Var(self.gen.fresh()))
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

    fn infer_let(
        &mut self,
        name: &str,
        value: &Expr,
        body: &Expr,
        _is_mutable: bool,
    ) -> Result<MonoType> {
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
        _is_async: bool,
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
            self.env = self.env.extend(param.name(), TypeScheme::mono(param_ty));
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

        // Always return the function type for type inference
        // The distinction between statements and expressions should be handled at a higher level
        Ok(final_type)
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
            self.env = self.env.extend(param.name(), TypeScheme::mono(param_ty));
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
            ("filter" | "groupby" | "agg" | "select", MonoType::DataFrame(columns)) => {
                // These methods return a DataFrame (preserve structure for now)
                Ok(MonoType::DataFrame(columns.clone()))
            }
            ("filter" | "groupby" | "agg" | "select", MonoType::Named(name))
                if name == "DataFrame" =>
            {
                // Fallback for untyped DataFrames
                Ok(MonoType::Named("DataFrame".to_string()))
            }
            ("mean" | "std" | "sum" | "count", MonoType::DataFrame(_) | MonoType::Series(_)) => {
                // These aggregation methods return numeric types
                Ok(MonoType::Float)
            }
            ("mean" | "std" | "sum" | "count", MonoType::Named(name))
                if name == "DataFrame" || name == "Series" =>
            {
                // Fallback for untyped DataFrames/Series
                Ok(MonoType::Float)
            }
            ("col", MonoType::DataFrame(columns)) => {
                // Column selection returns a Series with the column's type
                if let Some(arg) = args.first() {
                    if let ExprKind::Literal(Literal::String(col_name)) = &arg.kind {
                        if let Some((_, col_type)) =
                            columns.iter().find(|(name, _)| name == col_name)
                        {
                            return Ok(MonoType::Series(Box::new(col_type.clone())));
                        }
                    }
                }
                // Default to generic Series if column not found or not literal
                Ok(MonoType::Series(Box::new(MonoType::Var(self.gen.fresh()))))
            }
            ("col", MonoType::Named(name)) if name == "DataFrame" => {
                // Fallback for untyped DataFrames
                Ok(MonoType::Series(Box::new(MonoType::Var(self.gen.fresh()))))
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
            Pattern::Ok(inner) => {
                // Expected type should be Result<T, E>, extract T for inner pattern
                if let MonoType::Result(ok_ty, _) = expected_ty {
                    self.infer_pattern(inner, ok_ty)
                } else {
                    // Create a fresh Result type
                    let error_ty = MonoType::Var(self.gen.fresh());
                    let inner_ty = MonoType::Var(self.gen.fresh());
                    let result_ty =
                        MonoType::Result(Box::new(inner_ty.clone()), Box::new(error_ty));
                    self.unifier.unify(expected_ty, &result_ty)?;
                    self.infer_pattern(inner, &inner_ty)
                }
            }
            Pattern::Err(inner) => {
                // Expected type should be Result<T, E>, extract E for inner pattern
                if let MonoType::Result(_, err_ty) = expected_ty {
                    self.infer_pattern(inner, err_ty)
                } else {
                    // Create a fresh Result type
                    let ok_ty = MonoType::Var(self.gen.fresh());
                    let inner_ty = MonoType::Var(self.gen.fresh());
                    let result_ty = MonoType::Result(Box::new(ok_ty), Box::new(inner_ty.clone()));
                    self.unifier.unify(expected_ty, &result_ty)?;
                    self.infer_pattern(inner, &inner_ty)
                }
            }
            Pattern::Some(inner) => {
                // Expected type should be Option<T>, extract T for inner pattern
                if let MonoType::Optional(inner_ty) = expected_ty {
                    self.infer_pattern(inner, inner_ty)
                } else {
                    // Create a fresh Option type
                    let inner_ty = MonoType::Var(self.gen.fresh());
                    let option_ty = MonoType::Optional(Box::new(inner_ty.clone()));
                    self.unifier.unify(expected_ty, &option_ty)?;
                    self.infer_pattern(inner, &inner_ty)
                }
            }
            Pattern::None => {
                // None pattern matches Option<T> where T can be any type
                let type_var = MonoType::Var(self.gen.fresh());
                let option_ty = MonoType::Optional(Box::new(type_var));
                self.unifier.unify(expected_ty, &option_ty)
            }
            Pattern::Tuple(patterns) => {
                // Create tuple type with each pattern's inferred type
                let mut elem_types = Vec::new();
                for pat in patterns {
                    let elem_ty = MonoType::Var(self.gen.fresh());
                    self.infer_pattern(pat, &elem_ty)?;
                    elem_types.push(elem_ty);
                }
                let tuple_ty = MonoType::Tuple(elem_types);
                self.unifier.unify(expected_ty, &tuple_ty)
            }
            Pattern::Struct { name, fields } => {
                // For now, treat struct patterns as a named type
                // In a more complete implementation, we'd look up the struct definition
                let struct_ty = MonoType::Named(name.clone());
                self.unifier.unify(expected_ty, &struct_ty)?;

                // Infer field patterns (simplified approach)
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        let field_ty = MonoType::Var(self.gen.fresh());
                        self.infer_pattern(pattern, &field_ty)?;
                    }
                }
                Ok(())
            }
            Pattern::Range { start, end, .. } => {
                // Range patterns should match numeric types
                let start_ty = MonoType::Var(self.gen.fresh());
                let end_ty = MonoType::Var(self.gen.fresh());
                self.infer_pattern(start, &start_ty)?;
                self.infer_pattern(end, &end_ty)?;

                // Unify start and end types, and with expected type
                self.unifier.unify(&start_ty, &end_ty)?;
                self.unifier.unify(expected_ty, &start_ty)
            }
            Pattern::Or(patterns) => {
                // All patterns in an OR must have the same type
                for pat in patterns {
                    self.infer_pattern(pat, expected_ty)?;
                }
                Ok(())
            }
            Pattern::Rest => {
                // Rest patterns don't bind to specific types
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
        let _body_ty = self.infer_expr(body)?;
        self.env = old_env;

        // For loops always return Unit regardless of body type
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

    fn infer_loop(&mut self, body: &Expr) -> Result<MonoType> {
        // Type check body
        let body_ty = self.infer_expr(body)?;
        self.unifier.unify(&body_ty, &MonoType::Unit)?;

        // Loop expressions return unit
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

    fn infer_assign(&mut self, target: &Expr, value: &Expr) -> Result<MonoType> {
        // Infer the type of the value being assigned
        let value_ty = self.infer_expr(value)?;

        // Infer the type of the target (lvalue)
        let target_ty = self.infer_expr(target)?;

        // Target and value must have compatible types
        self.unifier.unify(&target_ty, &value_ty)?;

        // Assignment expressions return Unit
        Ok(MonoType::Unit)
    }

    fn infer_compound_assign(
        &mut self,
        target: &Expr,
        op: BinaryOp,
        value: &Expr,
    ) -> Result<MonoType> {
        // Infer the types of target and value
        let target_ty = self.infer_expr(target)?;
        let value_ty = self.infer_expr(value)?;

        // For compound assignment, we need to ensure the operation is valid
        // This is equivalent to: target = target op value
        let result_ty = self.infer_binary_op_type(op, &target_ty, &value_ty)?;

        // The result type must be compatible with the target type
        self.unifier.unify(&target_ty, &result_ty)?;

        // Compound assignment expressions return Unit
        Ok(MonoType::Unit)
    }

    fn infer_binary_op_type(
        &mut self,
        op: BinaryOp,
        left_ty: &MonoType,
        right_ty: &MonoType,
    ) -> Result<MonoType> {
        match op {
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo => {
                // Arithmetic operations: both operands should be numbers, result is same type
                // Try Int first, then Float
                if let Ok(()) = self.unifier.unify(left_ty, &MonoType::Int) {
                    if let Ok(()) = self.unifier.unify(right_ty, &MonoType::Int) {
                        return Ok(MonoType::Int);
                    }
                }
                // Fall back to Float
                self.unifier.unify(left_ty, &MonoType::Float)?;
                self.unifier.unify(right_ty, &MonoType::Float)?;
                Ok(MonoType::Float)
            }
            BinaryOp::Power => {
                // Power operation: base and exponent are numbers, result is same as base
                self.unifier.unify(left_ty, right_ty)?;
                if let Ok(()) = self.unifier.unify(left_ty, &MonoType::Int) {
                    Ok(MonoType::Int)
                } else {
                    self.unifier.unify(left_ty, &MonoType::Float)?;
                    Ok(MonoType::Float)
                }
            }
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual => {
                // Comparison operations: operands must be same type, result is Bool
                self.unifier.unify(left_ty, right_ty)?;
                Ok(MonoType::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                // Logical operations: both operands must be Bool, result is Bool
                self.unifier.unify(left_ty, &MonoType::Bool)?;
                self.unifier.unify(right_ty, &MonoType::Bool)?;
                Ok(MonoType::Bool)
            }
            BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::BitwiseXor
            | BinaryOp::LeftShift
            | BinaryOp::RightShift => {
                // Bitwise operations: both operands must be Int, result is Int
                self.unifier.unify(left_ty, &MonoType::Int)?;
                self.unifier.unify(right_ty, &MonoType::Int)?;
                Ok(MonoType::Int)
            }
        }
    }

    fn infer_increment_decrement(&mut self, target: &Expr) -> Result<MonoType> {
        // Infer the type of the target
        let target_ty = self.infer_expr(target)?;

        // Target must be a numeric type (Int or Float)
        // Try Int first, then Float
        if let Ok(()) = self.unifier.unify(&target_ty, &MonoType::Int) {
            Ok(MonoType::Int)
        } else {
            self.unifier.unify(&target_ty, &MonoType::Float)?;
            Ok(MonoType::Float)
        }
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
            TypeKind::DataFrame { columns } => {
                let mut col_types = Vec::new();
                for (name, ty) in columns {
                    col_types.push((name.clone(), Self::ast_type_to_mono_static(ty)?));
                }
                MonoType::DataFrame(col_types)
            }
            TypeKind::Series { dtype } => {
                MonoType::Series(Box::new(Self::ast_type_to_mono_static(dtype)?))
            }
            TypeKind::Tuple(types) => {
                let mono_types: Result<Vec<_>> = types
                    .iter()
                    .map(Self::ast_type_to_mono_static)
                    .collect();
                MonoType::Tuple(mono_types?)
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

    /// Helper methods for complex expression groups
    fn infer_string_interpolation(
        &mut self,
        parts: &[crate::frontend::ast::StringPart],
    ) -> Result<MonoType> {
        for part in parts {
            if let crate::frontend::ast::StringPart::Expr(expr) = part {
                let _ = self.infer_expr(expr)?;
            }
        }
        Ok(MonoType::Named("String".to_string()))
    }

    fn infer_result_ok(&mut self, value: &Expr) -> Result<MonoType> {
        let value_type = self.infer_expr(value)?;
        let error_type = MonoType::Var(self.gen.fresh());
        Ok(MonoType::Result(Box::new(value_type), Box::new(error_type)))
    }

    fn infer_result_err(&mut self, error: &Expr) -> Result<MonoType> {
        let error_type = self.infer_expr(error)?;
        let value_type = MonoType::Var(self.gen.fresh());
        Ok(MonoType::Result(Box::new(value_type), Box::new(error_type)))
    }

    fn infer_object_literal(
        &mut self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<MonoType> {
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
        Ok(MonoType::Named("Object".to_string()))
    }

    fn infer_field_access(&mut self, object: &Expr) -> Result<MonoType> {
        let _object_ty = self.infer_expr(object)?;
        Ok(MonoType::Var(self.gen.fresh()))
    }

    fn infer_index_access(&mut self, object: &Expr) -> Result<MonoType> {
        let object_ty = self.infer_expr(object)?;
        // For arrays/lists, return the element type
        // For now, we'll use a fresh type variable until we have proper collection typing
        match object_ty {
            MonoType::List(element_ty) => Ok(*element_ty),
            _ => Ok(MonoType::Var(self.gen.fresh())),
        }
    }

    fn infer_send(&mut self, actor: &Expr, message: &Expr) -> Result<MonoType> {
        let _actor_ty = self.infer_expr(actor)?;
        let _message_ty = self.infer_expr(message)?;
        Ok(MonoType::Unit)
    }

    fn infer_ask(
        &mut self,
        actor: &Expr,
        message: &Expr,
        timeout: Option<&Expr>,
    ) -> Result<MonoType> {
        let _actor_ty = self.infer_expr(actor)?;
        let _message_ty = self.infer_expr(message)?;
        if let Some(t) = timeout {
            let timeout_ty = self.infer_expr(t)?;
            self.unifier.unify(&timeout_ty, &MonoType::Int)?;
        }
        Ok(MonoType::Var(self.gen.fresh()))
    }

    fn infer_dataframe(
        &mut self,
        columns: &[crate::frontend::ast::DataFrameColumn],
    ) -> Result<MonoType> {
        let mut column_types = Vec::new();

        for col in columns {
            // Infer the type of the first value to determine column type
            let col_type = if col.values.is_empty() {
                MonoType::Var(self.gen.fresh())
            } else {
                let first_ty = self.infer_expr(&col.values[0])?;
                // Verify all values in the column have the same type
                for value in &col.values[1..] {
                    let value_ty = self.infer_expr(value)?;
                    self.unifier.unify(&first_ty, &value_ty)?;
                }
                first_ty
            };
            column_types.push((col.name.clone(), col_type));
        }

        Ok(MonoType::DataFrame(column_types))
    }

    fn infer_dataframe_operation(
        &mut self,
        source: &Expr,
        operation: &crate::frontend::ast::DataFrameOp,
    ) -> Result<MonoType> {
        use crate::frontend::ast::DataFrameOp;

        let source_ty = self.infer_expr(source)?;

        // Ensure source is a DataFrame
        match &source_ty {
            MonoType::DataFrame(columns) => {
                match operation {
                    DataFrameOp::Filter(_) => {
                        // Filter preserves the DataFrame structure
                        Ok(source_ty.clone())
                    }
                    DataFrameOp::Select(selected_cols) => {
                        // Select creates a new DataFrame with only the selected columns
                        let mut new_columns = Vec::new();
                        for col_name in selected_cols {
                            if let Some((_, ty)) = columns.iter().find(|(name, _)| name == col_name)
                            {
                                new_columns.push((col_name.clone(), ty.clone()));
                            }
                        }
                        Ok(MonoType::DataFrame(new_columns))
                    }
                    DataFrameOp::GroupBy(_) => {
                        // GroupBy returns a grouped DataFrame (for now, same type)
                        Ok(source_ty.clone())
                    }
                    DataFrameOp::Aggregate(_) => {
                        // Aggregation returns a DataFrame with aggregated values
                        Ok(source_ty.clone())
                    }
                    DataFrameOp::Join { .. } => {
                        // Join returns a DataFrame (simplified for now)
                        Ok(source_ty.clone())
                    }
                    DataFrameOp::Sort { .. } => {
                        // Sort preserves the DataFrame structure
                        Ok(source_ty.clone())
                    }
                    DataFrameOp::Limit(_) | DataFrameOp::Head(_) | DataFrameOp::Tail(_) => {
                        // These operations preserve the DataFrame structure
                        Ok(source_ty.clone())
                    }
                }
            }
            MonoType::Named(name) if name == "DataFrame" => {
                // Fallback for untyped DataFrames
                Ok(MonoType::Named("DataFrame".to_string()))
            }
            _ => bail!("DataFrame operation on non-DataFrame type: {}", source_ty),
        }
    }

    fn infer_async_block(&mut self, body: &Expr) -> Result<MonoType> {
        // Infer the body type
        let body_ty = self.infer_expr(body)?;

        // Async blocks return Future<Output = body_type>
        Ok(MonoType::Named(format!("Future<{body_ty}>")))
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
    fn test_infer_dataframe() {
        let df_str = r#"df![
            age => [25, 30, 35],
            name => ["Alice", "Bob", "Charlie"]
        ]"#;

        let result = infer_str(df_str).unwrap();
        match result {
            MonoType::DataFrame(columns) => {
                assert_eq!(columns.len(), 2);
                assert_eq!(columns[0].0, "age");
                assert!(matches!(columns[0].1, MonoType::Int));
                assert_eq!(columns[1].0, "name");
                assert!(matches!(columns[1].1, MonoType::String));
            }
            _ => panic!("Expected DataFrame type, got {result:?}"),
        }
    }

    #[test]
    fn test_infer_dataframe_operations() {
        // Test filter operation
        let filter_str = r"df![age => [25, 30]].filter(age > 25)";
        let result = infer_str(filter_str).unwrap();
        assert!(matches!(result, MonoType::DataFrame(_)));

        // Test select operation
        let select_str = r#"df![age => [25], name => ["Alice"]].select(["age"])"#;
        let result = infer_str(select_str).unwrap();
        match result {
            MonoType::DataFrame(columns) => {
                assert_eq!(columns.len(), 1);
                assert_eq!(columns[0].0, "age");
            }
            _ => panic!("Expected DataFrame type, got {result:?}"),
        }
    }

    #[test]
    fn test_infer_series() {
        // Test column selection returns Series
        let col_str = r#"df![age => [25, 30]].col("age")"#;
        let result = infer_str(col_str).unwrap();
        assert!(matches!(result, MonoType::Series(_)));

        // Test aggregation on Series
        let mean_str = r#"df![age => [25, 30]].col("age").mean()"#;
        let result = infer_str(mean_str).unwrap();
        assert_eq!(result, MonoType::Float);
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
