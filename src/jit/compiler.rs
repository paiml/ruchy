//! Cranelift JIT Compiler Implementation
//!
//! JIT-001: Minimal JIT compiler for arithmetic expressions
//!
//! # Performance Target
//! - fibonacci(20): <0.5ms (vs 19ms AST)
//! - Expected speedup: 50-100x
//!
//! # Implementation Status
//! - [x] Basic compiler setup
//! - [x] Integer arithmetic (+, -, *, /)
//! - [x] Control flow (if/else) - JIT-002
//! - [x] Variables and locals - JIT-002
//! - [x] Function calls and recursion - JIT-002
//! - [ ] Tiered optimization (JIT-003)

use anyhow::{anyhow, Result};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module, FuncId};
use cranelift_codegen::settings;
use std::collections::HashMap;

use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp};

/// JIT Compiler using Cranelift backend
///
/// # Example
/// ```ignore
/// let mut compiler = JitCompiler::new()?;
/// let result = compiler.compile_and_execute(&ast)?;
/// ```
pub struct JitCompiler {
    /// Cranelift JIT module for code generation
    module: JITModule,
    /// Function builder context (reusable)
    builder_context: FunctionBuilderContext,
    /// Code generation context
    ctx: codegen::Context,
    /// Variable mapping (name → Cranelift variable)
    variables: HashMap<String, Variable>,
    /// Next variable ID
    next_var: u32,
}

/// Compilation context passed through expression compilation
struct CompileContext {
    /// Variable mapping (name → Cranelift variable)
    variables: HashMap<String, Variable>,
    /// Next variable ID for creating fresh variables
    next_var: u32,
    /// Function table (name → FuncRef) for call resolution
    functions: HashMap<String, cranelift_codegen::ir::FuncRef>,
}

impl JitCompiler {
    /// Create a new JIT compiler instance
    ///
    /// # Errors
    /// Returns error if Cranelift initialization fails
    pub fn new() -> Result<Self> {
        // Get native ISA (Instruction Set Architecture)
        let mut flag_builder = settings::builder();
        // Enable optimization
        flag_builder.set("opt_level", "speed").map_err(|e| anyhow!("Failed to set opt_level: {}", e))?;
        let flags = settings::Flags::new(flag_builder);

        let isa = cranelift_native::builder().unwrap_or_else(|_| {
            panic!("Cranelift native ISA not available on this platform")
        }).finish(flags).map_err(|e| anyhow!("Failed to create ISA: {}", e))?;

        // Create JIT builder with native target
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);

        Ok(Self {
            module,
            builder_context: FunctionBuilderContext::new(),
            ctx: codegen::Context::new(),
            variables: HashMap::new(),
            next_var: 0,
        })
    }

    /// Compile and execute a Ruchy expression
    ///
    /// # Example
    /// ```ignore
    /// let ast = parse("2 + 2");
    /// let result = compiler.compile_and_execute(&ast)?;
    /// assert_eq!(result, 4);
    /// ```
    pub fn compile_and_execute(&mut self, ast: &Expr) -> Result<i64> {
        // JIT-001: Compile simple arithmetic to start
        let func_id = self.compile_expr_as_function(ast)?;

        // Finalize the function
        self.module.finalize_definitions()?;

        // Get function pointer
        let code = self.module.get_finalized_function(func_id);

        // SAFETY: We control the function signature and know it's correct
        // JIT-compiled function has signature: fn() -> i64
        let func: fn() -> i64 = unsafe { std::mem::transmute(code) };

        // Execute!
        Ok(func())
    }

    /// Compile an expression as a standalone function
    ///
    /// Creates a function with signature: `fn() -> i64`
    fn compile_expr_as_function(&mut self, ast: &Expr) -> Result<FuncId> {
        // Pre-scan for function definitions and declare them all
        let mut function_table = HashMap::new();
        self.collect_and_declare_functions(ast, &mut function_table)?;

        // Create main function signature: fn() -> i64
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));

        // Declare main function
        let main_func_id = self.module.declare_function(
            "main",
            Linkage::Export,
            &sig,
        )?;

        // Compile all nested functions first (so they're available for calls)
        self.compile_declared_functions(ast, &function_table)?;

        // Now compile main function body
        self.ctx.func.signature = sig;

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

            // Create entry block
            let entry_block = builder.create_block();
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // Import all functions as FuncRefs for this function
            let mut func_refs = HashMap::new();
            for (name, func_id) in &function_table {
                let func_ref = self.module.declare_func_in_func(*func_id, &mut builder.func);
                func_refs.insert(name.clone(), func_ref);
            }

            // Create compilation context with function refs
            let mut ctx = CompileContext {
                variables: HashMap::new(),
                next_var: 0,
                functions: func_refs,
            };

            // Compile expression
            let result = Self::compile_expr(&mut builder, &mut ctx, ast)?;

            // Return result
            builder.ins().return_(&[result]);

            builder.finalize();
        }

        // JIT compile main
        self.module.define_function(main_func_id, &mut self.ctx)?;
        self.module.clear_context(&mut self.ctx);

        Ok(main_func_id)
    }

    /// Collect all function definitions and declare them (get FuncIds)
    fn collect_and_declare_functions(&mut self, expr: &Expr, table: &mut HashMap<String, FuncId>) -> Result<()> {
        match &expr.kind {
            ExprKind::Block(exprs) => {
                for e in exprs {
                    if let ExprKind::Function { name, params, .. } = &e.kind {
                        // Create signature: (i64, i64, ...) -> i64
                        let mut sig = self.module.make_signature();
                        for _ in params {
                            sig.params.push(AbiParam::new(types::I64));
                        }
                        sig.returns.push(AbiParam::new(types::I64));

                        // Declare function
                        let func_id = self.module.declare_function(
                            name,
                            Linkage::Local,
                            &sig,
                        )?;

                        table.insert(name.clone(), func_id);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Compile all declared functions
    fn compile_declared_functions(&mut self, expr: &Expr, table: &HashMap<String, FuncId>) -> Result<()> {
        if let ExprKind::Block(exprs) = &expr.kind {
            for e in exprs {
                if let ExprKind::Function { name, params, body, .. } = &e.kind {
                    let func_id = *table.get(name).unwrap();
                    self.compile_function_body(func_id, params, body, table)?;
                }
            }
        }
        Ok(())
    }

    /// Compile a single function body
    fn compile_function_body(
        &mut self,
        func_id: FuncId,
        params: &[crate::frontend::ast::Param],
        body: &Expr,
        function_table: &HashMap<String, FuncId>,
    ) -> Result<()> {
        // Set up function context
        self.ctx.func.signature = self.module.declarations().get_function_decl(func_id).signature.clone();

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

            // Create entry block
            let entry_block = builder.create_block();
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            builder.seal_block(entry_block);

            // Import all functions as FuncRefs (for recursion)
            let mut func_refs = HashMap::new();
            for (name, func_id) in function_table {
                let func_ref = self.module.declare_func_in_func(*func_id, &mut builder.func);
                func_refs.insert(name.clone(), func_ref);
            }

            // Create context with parameters as variables
            let mut ctx = CompileContext {
                variables: HashMap::new(),
                next_var: 0,
                functions: func_refs,
            };

            // Map parameters to Cranelift variables
            let block_params = builder.block_params(entry_block).to_vec();
            for (i, param) in params.iter().enumerate() {
                if let crate::frontend::ast::Pattern::Identifier(param_name) = &param.pattern {
                    let var = builder.declare_var(types::I64);
                    builder.def_var(var, block_params[i]);
                    ctx.variables.insert(param_name.clone(), var);
                }
            }

            // Compile function body
            let result = Self::compile_expr(&mut builder, &mut ctx, body)?;

            // Return result
            builder.ins().return_(&[result]);

            builder.finalize();
        }

        // Define the function
        self.module.define_function(func_id, &mut self.ctx)?;
        self.module.clear_context(&mut self.ctx);

        Ok(())
    }

    /// Compile a Ruchy expression to Cranelift IR with variable context
    ///
    /// Returns the Cranelift value representing the expression result
    fn compile_expr(builder: &mut FunctionBuilder, ctx: &mut CompileContext, expr: &Expr) -> Result<Value> {
        match &expr.kind {
            // JIT-001: Integer literals
            ExprKind::Literal(Literal::Integer(n, _)) => {
                Ok(builder.ins().iconst(types::I64, *n))
            }

            // JIT-002: Boolean literals
            ExprKind::Literal(Literal::Bool(b)) => {
                Ok(builder.ins().iconst(types::I64, if *b { 1 } else { 0 }))
            }

            // JIT-002: Unit literal ()
            ExprKind::Literal(Literal::Unit) => {
                Ok(builder.ins().iconst(types::I64, 0))
            }

            // JIT-001: Binary operations (+, -, *, /)
            // JIT-002: Comparisons (<=, ==, >, etc)
            ExprKind::Binary { left, op, right } => {
                Self::compile_binary_op(builder, ctx, left, op, right)
            }

            // JIT-002: Block expressions (sequence of statements, return last value)
            ExprKind::Block(exprs) => {
                Self::compile_block(builder, ctx, exprs)
            }

            // JIT-002: If/else control flow
            ExprKind::If { condition, then_branch, else_branch } => {
                Self::compile_if(builder, ctx, condition, then_branch, else_branch.as_deref())
            }

            // JIT-002: Let bindings (variable declaration)
            ExprKind::Let { name, value, body, .. } => {
                Self::compile_let(builder, ctx, name, value, body)
            }

            // JIT-002: Identifier (variable lookup)
            ExprKind::Identifier(name) => {
                Self::compile_identifier(builder, ctx, name)
            }

            // JIT-002: Function definition (skip in expression context - handled by Block)
            ExprKind::Function { .. } => {
                // Functions are declarations, not values - return unit
                Ok(builder.ins().iconst(types::I64, 0))
            }

            // JIT-002: Call expression (function invocation)
            ExprKind::Call { func, args } => {
                Self::compile_call(builder, ctx, func, args)
            }

            // JIT-003: Unary operations (negation, boolean NOT)
            ExprKind::Unary { op, operand } => {
                Self::compile_unary_op(builder, ctx, op, operand)
            }

            // JIT-002: Not yet implemented - fall back to error
            _ => Err(anyhow!("JIT-002: Expression kind not yet supported: {:?}", expr.kind)),
        }
    }

    /// Compile binary operations (complexity ≤10)
    fn compile_binary_op(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        left: &Expr,
        op: &BinaryOp,
        right: &Expr,
    ) -> Result<Value> {
        // JIT-004: Logical operators require short-circuit evaluation
        match op {
            BinaryOp::And => return Self::compile_logical_and(builder, ctx, left, right),
            BinaryOp::Or => return Self::compile_logical_or(builder, ctx, left, right),
            _ => {} // Fall through to standard evaluation
        }

        // Standard evaluation: both operands always evaluated
        let lhs = Self::compile_expr(builder, ctx, left)?;
        let rhs = Self::compile_expr(builder, ctx, right)?;

        let result = match op {
            // Arithmetic
            BinaryOp::Add => builder.ins().iadd(lhs, rhs),
            BinaryOp::Subtract => builder.ins().isub(lhs, rhs),
            BinaryOp::Multiply => builder.ins().imul(lhs, rhs),
            BinaryOp::Divide => builder.ins().sdiv(lhs, rhs),
            BinaryOp::Modulo => builder.ins().srem(lhs, rhs),

            // Comparisons - return 0 (false) or 1 (true) as i64
            BinaryOp::Equal => {
                let cmp = builder.ins().icmp(IntCC::Equal, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }
            BinaryOp::NotEqual => {
                let cmp = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }
            BinaryOp::LessEqual => {
                let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }
            BinaryOp::Less => {
                let cmp = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }
            BinaryOp::Greater => {
                let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }
            BinaryOp::GreaterEqual => {
                let cmp = builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }

            _ => return Err(anyhow!("Unsupported binary operation in JIT: {:?}", op)),
        };

        Ok(result)
    }

    /// Compile unary operations (negation, boolean NOT) - complexity ≤5
    fn compile_unary_op(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        op: &crate::frontend::ast::UnaryOp,
        operand: &Expr,
    ) -> Result<Value> {
        let operand_value = Self::compile_expr(builder, ctx, operand)?;

        let result = match op {
            crate::frontend::ast::UnaryOp::Negate => {
                // -x: Negate the value (0 - x)
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().isub(zero, operand_value)
            }
            crate::frontend::ast::UnaryOp::Not => {
                // !bool: Boolean NOT
                // In our representation: true=1, false=0
                // !x = 1 - x (flips 0→1, 1→0)
                let one = builder.ins().iconst(types::I64, 1);
                builder.ins().isub(one, operand_value)
            }
            _ => return Err(anyhow!("JIT-003: Unsupported unary operation: {:?}", op)),
        };

        Ok(result)
    }

    /// Compile logical AND with short-circuit evaluation - complexity ≤10
    ///
    /// Semantics: left && right
    /// - If left is false (0), return false without evaluating right
    /// - If left is true (1), evaluate and return right
    fn compile_logical_and(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        left: &Expr,
        right: &Expr,
    ) -> Result<Value> {
        // Evaluate left operand
        let left_value = Self::compile_expr(builder, ctx, left)?;

        // Create blocks for short-circuit logic
        let eval_right_block = builder.create_block();
        let short_circuit_block = builder.create_block();
        let merge_block = builder.create_block();

        // Create variable to hold result
        let result_var = builder.declare_var(types::I64);

        // If left is true (non-zero), evaluate right; else short-circuit to false
        builder.ins().brif(left_value, eval_right_block, &[], short_circuit_block, &[]);

        // Left is true - evaluate right operand
        builder.switch_to_block(eval_right_block);
        builder.seal_block(eval_right_block);
        let right_value = Self::compile_expr(builder, ctx, right)?;
        builder.def_var(result_var, right_value);
        builder.ins().jump(merge_block, &[]);

        // Left is false - short-circuit to false (0)
        builder.switch_to_block(short_circuit_block);
        builder.seal_block(short_circuit_block);
        let false_val = builder.ins().iconst(types::I64, 0);
        builder.def_var(result_var, false_val);
        builder.ins().jump(merge_block, &[]);

        // Merge both paths
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        // Return result
        Ok(builder.use_var(result_var))
    }

    /// Compile logical OR with short-circuit evaluation - complexity ≤10
    ///
    /// Semantics: left || right
    /// - If left is true (1), return true without evaluating right
    /// - If left is false (0), evaluate and return right
    fn compile_logical_or(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        left: &Expr,
        right: &Expr,
    ) -> Result<Value> {
        // Evaluate left operand
        let left_value = Self::compile_expr(builder, ctx, left)?;

        // Create blocks for short-circuit logic
        let short_circuit_block = builder.create_block();
        let eval_right_block = builder.create_block();
        let merge_block = builder.create_block();

        // Create variable to hold result
        let result_var = builder.declare_var(types::I64);

        // If left is true (non-zero), short-circuit to true; else evaluate right
        builder.ins().brif(left_value, short_circuit_block, &[], eval_right_block, &[]);

        // Left is true - short-circuit to true (1)
        builder.switch_to_block(short_circuit_block);
        builder.seal_block(short_circuit_block);
        let true_val = builder.ins().iconst(types::I64, 1);
        builder.def_var(result_var, true_val);
        builder.ins().jump(merge_block, &[]);

        // Left is false - evaluate right operand
        builder.switch_to_block(eval_right_block);
        builder.seal_block(eval_right_block);
        let right_value = Self::compile_expr(builder, ctx, right)?;
        builder.def_var(result_var, right_value);
        builder.ins().jump(merge_block, &[]);

        // Merge both paths
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        // Return result
        Ok(builder.use_var(result_var))
    }

    /// Compile block expression (sequence of statements, return last value)
    fn compile_block(builder: &mut FunctionBuilder, ctx: &mut CompileContext, exprs: &[Expr]) -> Result<Value> {
        if exprs.is_empty() {
            // Empty block returns unit ()
            return Ok(builder.ins().iconst(types::I64, 0));
        }

        let mut last_value = None;
        for expr in exprs {
            last_value = Some(Self::compile_expr(builder, ctx, expr)?);
        }

        // Return the value of the last expression
        Ok(last_value.expect("Non-empty block should have at least one value"))
    }

    /// Compile if/else control flow using Cranelift variables (complexity ≤10)
    fn compile_if(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<Value> {
        // Evaluate condition
        let cond_value = Self::compile_expr(builder, ctx, condition)?;

        // Create blocks for then, else, and merge
        let then_block = builder.create_block();
        let else_block = builder.create_block();
        let merge_block = builder.create_block();

        // Create a variable to hold the result (enables SSA phi nodes)
        let result_var = builder.declare_var(types::I64);

        // Branch based on condition
        builder.ins().brif(cond_value, then_block, &[], else_block, &[]);

        // Then branch
        builder.switch_to_block(then_block);
        builder.seal_block(then_block);
        let then_value = Self::compile_expr(builder, ctx, then_branch)?;
        builder.def_var(result_var, then_value);
        builder.ins().jump(merge_block, &[]);

        // Else branch
        builder.switch_to_block(else_block);
        builder.seal_block(else_block);
        let else_value = if let Some(else_expr) = else_branch {
            Self::compile_expr(builder, ctx, else_expr)?
        } else {
            // No else branch, return unit ()
            builder.ins().iconst(types::I64, 0)
        };
        builder.def_var(result_var, else_value);
        builder.ins().jump(merge_block, &[]);

        // Merge block
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        // Read the result variable (automatically creates phi node)
        let result = builder.use_var(result_var);

        Ok(result)
    }

    /// Compile let binding (variable declaration and initialization) - complexity ≤10
    fn compile_let(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        name: &str,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value> {
        // Compile the value expression
        let init_value = Self::compile_expr(builder, ctx, value)?;

        // Create a Cranelift variable
        let var = builder.declare_var(types::I64);

        // Define the variable with initial value
        builder.def_var(var, init_value);

        // Store variable in context for future lookups (persists for rest of block)
        ctx.variables.insert(name.to_string(), var);

        // Compile the body expression (usually Unit in block-level let bindings)
        let result = Self::compile_expr(builder, ctx, body)?;

        // NOTE: Variable stays in scope for rest of block (block-scoped binding)
        // It will be cleaned up when CompileContext is dropped at function end

        Ok(result)
    }

    /// Compile identifier (variable lookup) - complexity ≤5
    fn compile_identifier(
        builder: &mut FunctionBuilder,
        ctx: &CompileContext,
        name: &str,
    ) -> Result<Value> {
        // Lookup variable in context
        let var = ctx.variables.get(name)
            .ok_or_else(|| anyhow!("Undefined variable: {}", name))?;

        // Read variable value (Cranelift handles SSA)
        Ok(builder.use_var(*var))
    }

    /// Compile call expression (function invocation) - complexity ≤10
    fn compile_call(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        func: &Expr,
        args: &[Expr],
    ) -> Result<Value> {
        // Extract function name from identifier
        let func_name = match &func.kind {
            ExprKind::Identifier(name) => name,
            _ => return Err(anyhow!("JIT-002: Only direct function calls supported")),
        };

        // Lookup function reference in table (copy it to avoid borrow issues)
        let func_ref = *ctx.functions.get(func_name)
            .ok_or_else(|| anyhow!("Undefined function: {}", func_name))?;

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(Self::compile_expr(builder, ctx, arg)?);
        }

        // Emit call instruction
        let call_inst = builder.ins().call(func_ref, &arg_values);

        // Get return value (first result of call)
        let results = builder.inst_results(call_inst);
        Ok(results[0])
    }

    /// Get a fresh Cranelift variable ID
    fn next_variable(&mut self) -> Variable {
        let var = Variable::new(self.next_var as usize);
        self.next_var += 1;
        var
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_jit_compiler_creation() {
        let compiler = JitCompiler::new();
        assert!(compiler.is_ok(), "JIT compiler should initialize successfully");
    }

    #[test]
    fn test_jit_simple_literal() {
        let code = "42";
        let ast = Parser::new(code).parse().unwrap();
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile literal: {:?}", result);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_jit_simple_addition() {
        let code = "2 + 3";
        let ast = Parser::new(code).parse().unwrap();
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile addition: {:?}", result);
        assert_eq!(result.unwrap(), 5);
    }

    #[test]
    fn test_jit_complex_arithmetic() {
        let code = "(10 + 5) * 2 - 8 / 4";
        let ast = Parser::new(code).parse().unwrap();
        let mut compiler = JitCompiler::new().unwrap();
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile complex arithmetic: {:?}", result);
        assert_eq!(result.unwrap(), 28); // (15) * 2 - 2 = 30 - 2 = 28
    }
}
