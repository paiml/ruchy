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

// Allow unsafe code in JIT module - required for function pointer transmutation
#![allow(unsafe_code)]

use anyhow::{anyhow, Result};
use cranelift::prelude::*;
use cranelift_codegen::settings;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};
use std::collections::HashMap;

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};

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
    /// Function table (name → `FuncRef`) for call resolution
    functions: HashMap<String, cranelift_codegen::ir::FuncRef>,
    /// JIT-005: Current loop's merge block (for break statements)
    loop_merge_block: Option<cranelift_codegen::ir::Block>,
    /// JIT-005B: Current loop's continue block (for continue statements)
    loop_continue_block: Option<cranelift_codegen::ir::Block>,
    /// JIT-005: Track if current block is terminated (by break/continue)
    block_terminated: bool,
    /// JIT-007: Track tuple sizes (varname → `element_count`)
    tuple_sizes: HashMap<String, usize>,
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
        flag_builder
            .set("opt_level", "speed")
            .map_err(|e| anyhow!("Failed to set opt_level: {e}"))?;
        let flags = settings::Flags::new(flag_builder);

        let isa = cranelift_native::builder()
            .unwrap_or_else(|_| panic!("Cranelift native ISA not available on this platform"))
            .finish(flags)
            .map_err(|e| anyhow!("Failed to create ISA: {e}"))?;

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
        let main_func_id = self
            .module
            .declare_function("main", Linkage::Export, &sig)?;

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
                let func_ref = self.module.declare_func_in_func(*func_id, builder.func);
                func_refs.insert(name.clone(), func_ref);
            }

            // Create compilation context with function refs
            let mut ctx = CompileContext {
                variables: HashMap::new(),
                next_var: 0,
                functions: func_refs,
                loop_merge_block: None,
                loop_continue_block: None,
                block_terminated: false,
                tuple_sizes: HashMap::new(),
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

    /// Collect all function definitions and declare them (get `FuncIds`)
    fn collect_and_declare_functions(
        &mut self,
        expr: &Expr,
        table: &mut HashMap<String, FuncId>,
    ) -> Result<()> {
        if let ExprKind::Block(exprs) = &expr.kind {
            for e in exprs {
                if let ExprKind::Function { name, params, .. } = &e.kind {
                    // Create signature: (i64, i64, ...) -> i64
                    let mut sig = self.module.make_signature();
                    for _ in params {
                        sig.params.push(AbiParam::new(types::I64));
                    }
                    sig.returns.push(AbiParam::new(types::I64));

                    // Declare function
                    let func_id = self.module.declare_function(name, Linkage::Local, &sig)?;

                    table.insert(name.clone(), func_id);
                }
            }
        }
        Ok(())
    }

    /// Compile all declared functions
    fn compile_declared_functions(
        &mut self,
        expr: &Expr,
        table: &HashMap<String, FuncId>,
    ) -> Result<()> {
        if let ExprKind::Block(exprs) = &expr.kind {
            for e in exprs {
                if let ExprKind::Function {
                    name, params, body, ..
                } = &e.kind
                {
                    let func_id = *table
                        .get(name)
                        .expect("function name should exist in symbol table");
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
        self.ctx.func.signature = self
            .module
            .declarations()
            .get_function_decl(func_id)
            .signature
            .clone();

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
                let func_ref = self.module.declare_func_in_func(*func_id, builder.func);
                func_refs.insert(name.clone(), func_ref);
            }

            // Create context with parameters as variables
            let mut ctx = CompileContext {
                variables: HashMap::new(),
                next_var: 0,
                functions: func_refs,
                loop_merge_block: None,
                loop_continue_block: None,
                block_terminated: false,
                tuple_sizes: HashMap::new(),
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

            // Return result (only if block not already terminated by explicit return)
            if !ctx.block_terminated {
                builder.ins().return_(&[result]);
            }

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
    fn compile_expr(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        expr: &Expr,
    ) -> Result<Value> {
        match &expr.kind {
            // JIT-001: Integer literals
            ExprKind::Literal(Literal::Integer(n, _)) => Ok(builder.ins().iconst(types::I64, *n)),

            // JIT-002: Boolean literals
            ExprKind::Literal(Literal::Bool(b)) => {
                Ok(builder.ins().iconst(types::I64, i64::from(*b)))
            }

            // JIT-002: Unit literal ()
            ExprKind::Literal(Literal::Unit) => Ok(builder.ins().iconst(types::I64, 0)),

            // JIT-001: Binary operations (+, -, *, /)
            // JIT-002: Comparisons (<=, ==, >, etc)
            ExprKind::Binary { left, op, right } => {
                Self::compile_binary_op(builder, ctx, left, op, right)
            }

            // JIT-002: Block expressions (sequence of statements, return last value)
            ExprKind::Block(exprs) => Self::compile_block(builder, ctx, exprs),

            // JIT-002: If/else control flow
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => Self::compile_if(builder, ctx, condition, then_branch, else_branch.as_deref()),

            // JIT-002: Let bindings (variable declaration)
            ExprKind::Let {
                name, value, body, ..
            } => Self::compile_let(builder, ctx, name, value, body),

            // JIT-002: Identifier (variable lookup)
            ExprKind::Identifier(name) => Self::compile_identifier(builder, ctx, name),

            // JIT-002: Function definition (skip in expression context - handled by Block)
            ExprKind::Function { .. } => {
                // Functions are declarations, not values - return unit
                Ok(builder.ins().iconst(types::I64, 0))
            }

            // JIT-002: Call expression (function invocation)
            ExprKind::Call { func, args } => Self::compile_call(builder, ctx, func, args),

            // JIT-003: Unary operations (negation, boolean NOT)
            ExprKind::Unary { op, operand } => Self::compile_unary_op(builder, ctx, op, operand),

            // JIT-005: While loops
            ExprKind::While {
                condition, body, ..
            } => Self::compile_while(builder, ctx, condition, body),

            // JIT-005: For loops (desugar to while)
            ExprKind::For {
                var, iter, body, ..
            } => Self::compile_for(builder, ctx, var, iter, body),

            // JIT-005: Break statement
            ExprKind::Break { .. } => Self::compile_break(builder, ctx),

            // JIT-005B: Continue statement
            ExprKind::Continue { .. } => Self::compile_continue(builder, ctx),

            // JIT-008: Return statement
            ExprKind::Return { value } => Self::compile_return(builder, ctx, value.as_deref()),

            // JIT-005: Assignment (for loop variables)
            ExprKind::Assign { target, value } => Self::compile_assign(builder, ctx, target, value),

            // JIT-007: Tuple literals
            ExprKind::Tuple(elements) => Self::compile_tuple(builder, ctx, elements),

            // JIT-007: Field access (tuple.0, tuple.1, etc.)
            ExprKind::FieldAccess { object, field } => {
                Self::compile_field_access(builder, ctx, object, field)
            }

            // JIT-002: Not yet implemented - fall back to error
            _ => Err(anyhow!(
                "JIT-002: Expression kind not yet supported: {:?}",
                expr.kind
            )),
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
                let cmp = builder
                    .ins()
                    .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs);
                builder.ins().uextend(types::I64, cmp)
            }

            _ => return Err(anyhow!("Unsupported binary operation in JIT: {op:?}")),
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
            _ => return Err(anyhow!("JIT-003: Unsupported unary operation: {op:?}")),
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
        builder
            .ins()
            .brif(left_value, eval_right_block, &[], short_circuit_block, &[]);

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
        builder
            .ins()
            .brif(left_value, short_circuit_block, &[], eval_right_block, &[]);

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

    /// Compile while loop - complexity ≤10
    ///
    /// Structure: `loop_block` → check condition → `body_block` or `merge_block`
    fn compile_while(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        condition: &Expr,
        body: &Expr,
    ) -> Result<Value> {
        // Create blocks for loop control flow
        let loop_block = builder.create_block();
        let body_block = builder.create_block();
        let merge_block = builder.create_block();

        // Save previous loop context (for nested loops)
        let prev_loop = ctx.loop_merge_block;
        let prev_continue = ctx.loop_continue_block;
        ctx.loop_merge_block = Some(merge_block);
        ctx.loop_continue_block = Some(loop_block);

        // Jump to loop block
        builder.ins().jump(loop_block, &[]);

        // Loop block: Evaluate condition
        builder.switch_to_block(loop_block);
        let cond_value = Self::compile_expr(builder, ctx, condition)?;
        builder
            .ins()
            .brif(cond_value, body_block, &[], merge_block, &[]);

        // Body block: Execute loop body
        builder.switch_to_block(body_block);
        builder.seal_block(body_block);
        Self::compile_expr(builder, ctx, body)?;
        // Only jump back if block isn't already terminated (e.g., by break)
        if !ctx.block_terminated {
            builder.ins().jump(loop_block, &[]);
        }
        ctx.block_terminated = false; // Reset for next block

        // Now seal loop_block (all predecessors known)
        builder.seal_block(loop_block);

        // Merge block: After loop
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        // Restore previous loop context
        ctx.loop_merge_block = prev_loop;
        ctx.loop_continue_block = prev_continue;

        // Loops return unit ()
        Ok(builder.ins().iconst(types::I64, 0))
    }

    /// Compile for loop - complexity ≤10
    ///
    /// Structure: `loop_block` → `body_block` → `incr_block` → `loop_block`
    /// Continue jumps to `incr_block` (not `loop_block`) to ensure increment happens
    fn compile_for(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        var_name: &str,
        iter: &Expr,
        body: &Expr,
    ) -> Result<Value> {
        // Extract range start/end from iterator
        let (start, end, inclusive) = match &iter.kind {
            crate::frontend::ast::ExprKind::Range {
                start,
                end,
                inclusive,
            } => (start, end, *inclusive),
            _ => return Err(anyhow!("JIT-005: For loop requires range iterator")),
        };

        // Evaluate range bounds
        let start_val = Self::compile_expr(builder, ctx, start)?;
        let end_val = Self::compile_expr(builder, ctx, end)?;

        // Create loop variable
        let loop_var = builder.declare_var(types::I64);
        builder.def_var(loop_var, start_val);
        ctx.variables.insert(var_name.to_string(), loop_var);

        // Create blocks for loop control flow
        let loop_block = builder.create_block();
        let body_block = builder.create_block();
        let incr_block = builder.create_block(); // JIT-005B: Separate increment block for continue
        let merge_block = builder.create_block();

        // Save previous loop context (for nested loops)
        let prev_loop = ctx.loop_merge_block;
        let prev_continue = ctx.loop_continue_block;
        ctx.loop_merge_block = Some(merge_block);
        ctx.loop_continue_block = Some(incr_block); // JIT-005B: Continue jumps to increment

        // Jump to loop block
        builder.ins().jump(loop_block, &[]);

        // Loop block: Check i < end (or i <= end for inclusive)
        builder.switch_to_block(loop_block);
        let current_val = builder.use_var(loop_var);
        let cond = if inclusive {
            builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                current_val,
                end_val,
            )
        } else {
            builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
                current_val,
                end_val,
            )
        };
        builder.ins().brif(cond, body_block, &[], merge_block, &[]);

        // Body block: Execute body
        builder.switch_to_block(body_block);
        builder.seal_block(body_block);
        Self::compile_expr(builder, ctx, body)?;
        // Jump to increment block if not already terminated
        if !ctx.block_terminated {
            builder.ins().jump(incr_block, &[]);
        }
        ctx.block_terminated = false; // Reset for next block

        // Increment block: i = i + 1, then loop back
        builder.switch_to_block(incr_block);
        builder.seal_block(incr_block);
        let current_val = builder.use_var(loop_var);
        let one = builder.ins().iconst(types::I64, 1);
        let next_val = builder.ins().iadd(current_val, one);
        builder.def_var(loop_var, next_val);
        builder.ins().jump(loop_block, &[]);

        // Now seal loop_block (all predecessors known)
        builder.seal_block(loop_block);

        // Merge block: After loop
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        // Restore previous loop context
        ctx.loop_merge_block = prev_loop;
        ctx.loop_continue_block = prev_continue;

        // Loops return unit ()
        Ok(builder.ins().iconst(types::I64, 0))
    }

    /// Compile break statement - complexity ≤5
    ///
    /// Jump to current loop's merge block
    fn compile_break(builder: &mut FunctionBuilder, ctx: &mut CompileContext) -> Result<Value> {
        match ctx.loop_merge_block {
            Some(merge_block) => {
                // Create dummy value BEFORE terminating block
                let dummy = builder.ins().iconst(types::I64, 0);
                // Jump to merge block (terminates current block)
                builder.ins().jump(merge_block, &[]);
                // Mark block as terminated so caller doesn't try to add more instructions
                ctx.block_terminated = true;
                Ok(dummy)
            }
            None => Err(anyhow!("JIT-005: Break outside of loop")),
        }
    }

    /// Compile continue statement - complexity ≤5
    ///
    /// Jump to current loop's continue target (loop header or increment)
    fn compile_continue(builder: &mut FunctionBuilder, ctx: &mut CompileContext) -> Result<Value> {
        match ctx.loop_continue_block {
            Some(continue_block) => {
                // Create dummy value BEFORE terminating block
                let dummy = builder.ins().iconst(types::I64, 0);
                // Jump to continue block (terminates current block)
                builder.ins().jump(continue_block, &[]);
                // Mark block as terminated so caller doesn't try to add more instructions
                ctx.block_terminated = true;
                Ok(dummy)
            }
            None => Err(anyhow!("JIT-005B: Continue outside of loop")),
        }
    }

    /// Compile return statement - complexity ≤5
    ///
    /// Return value from function and exit immediately
    /// # Errors
    /// Returns error if return value compilation fails
    fn compile_return(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        value: Option<&Expr>,
    ) -> Result<Value> {
        // Evaluate return value (or use 0 as default unit value)
        let return_value = match value {
            Some(expr) => Self::compile_expr(builder, ctx, expr)?,
            None => builder.ins().iconst(types::I64, 0),
        };

        // Return from function (terminates current block)
        builder.ins().return_(&[return_value]);

        // Mark block as terminated so caller doesn't try to add more instructions
        ctx.block_terminated = true;

        // Return the value we're returning (for expression result, though it won't be used)
        Ok(return_value)
    }

    /// Compile assignment - complexity ≤5
    ///
    /// Update variable value: x = value
    fn compile_assign(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        target: &Expr,
        value: &Expr,
    ) -> Result<Value> {
        // Get target variable name
        let var_name = match &target.kind {
            crate::frontend::ast::ExprKind::Identifier(name) => name,
            _ => return Err(anyhow!("JIT-005: Assignment target must be identifier")),
        };

        // Evaluate new value
        let new_val = Self::compile_expr(builder, ctx, value)?;

        // Look up variable
        let var = ctx
            .variables
            .get(var_name)
            .ok_or_else(|| anyhow!("JIT-005: Undefined variable: {var_name}"))?;

        // Update variable
        builder.def_var(*var, new_val);

        // Assignment returns unit ()
        Ok(builder.ins().iconst(types::I64, 0))
    }

    /// Compile block expression (sequence of statements, return last value)
    fn compile_block(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        exprs: &[Expr],
    ) -> Result<Value> {
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
        builder
            .ins()
            .brif(cond_value, then_block, &[], else_block, &[]);

        // Then branch
        builder.switch_to_block(then_block);
        builder.seal_block(then_block);
        let then_value = Self::compile_expr(builder, ctx, then_branch)?;
        // Only add instructions if block wasn't terminated (e.g., by break)
        if !ctx.block_terminated {
            builder.def_var(result_var, then_value);
            builder.ins().jump(merge_block, &[]);
        }
        let then_terminated = ctx.block_terminated;
        ctx.block_terminated = false; // Reset for else branch

        // Else branch
        builder.switch_to_block(else_block);
        builder.seal_block(else_block);
        let else_value = if let Some(else_expr) = else_branch {
            Self::compile_expr(builder, ctx, else_expr)?
        } else {
            // No else branch, return unit ()
            builder.ins().iconst(types::I64, 0)
        };
        // Only add instructions if block wasn't terminated (e.g., by break)
        if !ctx.block_terminated {
            builder.def_var(result_var, else_value);
            builder.ins().jump(merge_block, &[]);
        }
        let else_terminated = ctx.block_terminated;
        ctx.block_terminated = false; // Reset for merge block

        // Merge block
        builder.switch_to_block(merge_block);
        builder.seal_block(merge_block);

        // If both branches terminated, mark context as terminated
        if then_terminated && else_terminated {
            ctx.block_terminated = true;
            // Merge block is unreachable, but Cranelift requires it be filled with terminator
            // Return dummy value (this block will never execute, but Cranelift needs it)
            let dummy = builder.ins().iconst(types::I64, 0);
            builder.ins().return_(&[dummy]);
            Ok(dummy)
        } else {
            // Read the result variable (automatically creates phi node)
            let result = builder.use_var(result_var);
            Ok(result)
        }
    }

    /// Compile let binding (variable declaration and initialization) - complexity ≤10
    fn compile_let(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        name: &str,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value> {
        // JIT-007: Check if value is a tuple literal
        if let ExprKind::Tuple(elements) = &value.kind {
            // Compile tuple elements directly into named variables
            for (i, elem) in elements.iter().enumerate() {
                let elem_value = Self::compile_expr(builder, ctx, elem)?;
                let elem_var = builder.declare_var(types::I64);
                builder.def_var(elem_var, elem_value);
                let elem_name = format!("{name}${i}");
                ctx.variables.insert(elem_name, elem_var);
            }
            // Track tuple size
            ctx.tuple_sizes.insert(name.to_string(), elements.len());

            // Still create a dummy variable for the tuple itself
            let var = builder.declare_var(types::I64);
            let dummy = builder.ins().iconst(types::I64, 0);
            builder.def_var(var, dummy);
            ctx.variables.insert(name.to_string(), var);
        } else {
            // Standard let binding
            let init_value = Self::compile_expr(builder, ctx, value)?;
            let var = builder.declare_var(types::I64);
            builder.def_var(var, init_value);
            ctx.variables.insert(name.to_string(), var);
        }

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
        let var = ctx
            .variables
            .get(name)
            .ok_or_else(|| anyhow!("Undefined variable: {name}"))?;

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
        let func_ref = *ctx
            .functions
            .get(func_name)
            .ok_or_else(|| anyhow!("Undefined function: {func_name}"))?;

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

    /// Compile tuple literal - complexity ≤5
    ///
    /// Strategy: Store tuple elements as separate variables with naming scheme
    /// Returns a "tuple handle" (index into a conceptual tuple table)
    fn compile_tuple(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        elements: &[Expr],
    ) -> Result<Value> {
        if elements.is_empty() {
            return Err(anyhow!("JIT-007: Empty tuples not supported"));
        }

        // Generate unique tuple ID based on next_var
        let tuple_id = ctx.next_var;
        ctx.next_var += 1;
        let tuple_name = format!("$tuple{tuple_id}");

        // Compile and store each element
        for (i, elem) in elements.iter().enumerate() {
            let elem_value = Self::compile_expr(builder, ctx, elem)?;
            let elem_var = builder.declare_var(types::I64);
            builder.def_var(elem_var, elem_value);
            let elem_name = format!("{tuple_name}${i}");
            ctx.variables.insert(elem_name, elem_var);
        }

        // Track tuple size for field access
        ctx.tuple_sizes.insert(tuple_name.clone(), elements.len());
        ctx.variables
            .insert(tuple_name, builder.declare_var(types::I64));

        // Return tuple ID as handle
        Ok(builder.ins().iconst(types::I64, i64::from(tuple_id)))
    }

    /// Compile field access (tuple.0, tuple.1, etc.) - complexity ≤5
    fn compile_field_access(
        builder: &mut FunctionBuilder,
        ctx: &mut CompileContext,
        object: &Expr,
        field: &str,
    ) -> Result<Value> {
        // Parse field index
        let field_idx: usize = field
            .parse()
            .map_err(|_| anyhow!("JIT-007: Field must be numeric index: {field}"))?;

        // For tuple access, object must be an identifier
        if let ExprKind::Identifier(var_name) = &object.kind {
            // Check if this is a tuple variable
            if let Some(&tuple_size) = ctx.tuple_sizes.get(var_name) {
                if field_idx >= tuple_size {
                    return Err(anyhow!(
                        "JIT-007: Tuple index {field_idx} out of bounds (size {tuple_size})"
                    ));
                }
                let elem_name = format!("{var_name}${field_idx}");
                if let Some(&var) = ctx.variables.get(&elem_name) {
                    return Ok(builder.use_var(var));
                }
            }

            // Try looking up as tuple handle (for inline tuples)
            let tuple_handle = Self::compile_expr(builder, ctx, object)?;
            // Extract tuple ID from handle
            // For now, we'll use a simple approach: inline tuples store directly
            let tuple_id = tuple_handle; // This will be the iconst value
            let tuple_name = format!("$tuple{tuple_id}");
            let elem_name = format!("{tuple_name}${field_idx}");
            if let Some(&var) = ctx.variables.get(&elem_name) {
                return Ok(builder.use_var(var));
            }

            Err(anyhow!("JIT-007: Variable '{var_name}' is not a tuple"))
        } else {
            Err(anyhow!(
                "JIT-007: Field access only supported on identifiers"
            ))
        }
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
        assert!(
            compiler.is_ok(),
            "JIT compiler should initialize successfully"
        );
    }

    #[test]
    fn test_jit_simple_literal() {
        let code = "42";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile literal: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 42);
    }

    #[test]
    fn test_jit_simple_addition() {
        let code = "2 + 3";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile addition: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 5);
    }

    #[test]
    fn test_jit_complex_arithmetic() {
        let code = "(10 + 5) * 2 - 8 / 4";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(
            result.is_ok(),
            "JIT should compile complex arithmetic: {result:?}"
        );
        assert_eq!(result.expect("operation should succeed in test"), 28); // (15) * 2 - 2 = 30 - 2 = 28
    }

    // Test 5: Subtraction
    #[test]
    fn test_jit_subtraction() {
        let code = "10 - 3";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile subtraction");
        assert_eq!(result.expect("operation should succeed in test"), 7);
    }

    // Test 6: Multiplication
    #[test]
    fn test_jit_multiplication() {
        let code = "6 * 7";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile multiplication");
        assert_eq!(result.expect("operation should succeed in test"), 42);
    }

    // Test 7: Division
    #[test]
    fn test_jit_division() {
        let code = "100 / 4";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile division");
        assert_eq!(result.expect("operation should succeed in test"), 25);
    }

    // Test 8: Modulo
    #[test]
    fn test_jit_modulo() {
        let code = "17 % 5";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile modulo: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 2);
    }

    // Test 9: Negative literal
    #[test]
    fn test_jit_negative_literal() {
        let code = "-42";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile negative: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), -42);
    }

    // Test 10: Comparison equal (true = 1)
    #[test]
    fn test_jit_comparison_equal_true() {
        let code = "5 == 5";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile equal: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 1); // true = 1
    }

    // Test 11: Comparison equal (false = 0)
    #[test]
    fn test_jit_comparison_equal_false() {
        let code = "5 == 6";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile equal: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 0); // false = 0
    }

    // Test 12: Comparison less than
    #[test]
    fn test_jit_comparison_less_than() {
        let code = "3 < 5";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile less than: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 1); // true = 1
    }

    // Test 13: Comparison greater than
    #[test]
    fn test_jit_comparison_greater_than() {
        let code = "10 > 3";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile greater: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 1); // true = 1
    }

    // Test 14: Boolean literal true
    #[test]
    fn test_jit_bool_true() {
        let code = "true";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile true: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 1);
    }

    // Test 15: Boolean literal false
    #[test]
    fn test_jit_bool_false() {
        let code = "false";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile false: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 0);
    }

    // Test 16: Simple if-else (true branch)
    #[test]
    fn test_jit_if_else_true() {
        let code = "if true { 42 } else { 0 }";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile if-else: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 42);
    }

    // Test 17: Simple if-else (false branch)
    #[test]
    fn test_jit_if_else_false() {
        let code = "if false { 42 } else { 99 }";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile if-else: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 99);
    }

    // Test 18: Let binding
    #[test]
    fn test_jit_let_binding() {
        let code = "let x = 10 in x * 2";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile let: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 20);
    }

    // Test 19: Nested let bindings
    #[test]
    fn test_jit_nested_let() {
        let code = "let x = 5 in let y = 3 in x + y";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile nested let: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 8);
    }

    // Test 20: Block expression
    #[test]
    fn test_jit_block() {
        let code = "{ 1; 2; 3 }";
        let ast = Parser::new(code)
            .parse()
            .expect("operation should succeed in test");
        let mut compiler = JitCompiler::new().expect("operation should succeed in test");
        let result = compiler.compile_and_execute(&ast);
        assert!(result.is_ok(), "JIT should compile block: {result:?}");
        assert_eq!(result.expect("operation should succeed in test"), 3);
    }
}
