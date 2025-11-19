//! AST to Cranelift IR Lowering
//!
//! JIT-001: Basic lowering for arithmetic expressions
//! JIT-002: Full lowering for control flow, functions, data structures
//!
//! # Lowering Strategy
//! ```
//! Ruchy AST → Typed AST → Cranelift IR
//! ```
//!
//! # Status
//! - [x] Integer literals
//! - [x] Binary arithmetic (+, -, *, /)
//! - [ ] Variables and locals
//! - [ ] Function calls
//! - [ ] Control flow (if/else, loops)
//! - [ ] Data structures (arrays, objects)

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use anyhow::{anyhow, Result};
use cranelift::prelude::*;

/// Lower a Ruchy expression to Cranelift IR value
///
/// This is a helper for the main JIT compiler
pub fn lower_expr_to_value(builder: &mut FunctionBuilder, expr: &Expr) -> Result<Value> {
    match &expr.kind {
        ExprKind::Literal(Literal::Integer(n, _)) => Ok(builder.ins().iconst(types::I64, *n)),

        ExprKind::Binary { left, op, right } => {
            let lhs = lower_expr_to_value(builder, left)?;
            let rhs = lower_expr_to_value(builder, right)?;

            lower_binary_op(builder, *op, lhs, rhs)
        }

        _ => Err(anyhow!(
            "Unsupported expression in lowering: {:?}",
            expr.kind
        )),
    }
}

/// Lower a binary operation to Cranelift instruction
fn lower_binary_op(
    builder: &mut FunctionBuilder,
    op: BinaryOp,
    lhs: Value,
    rhs: Value,
) -> Result<Value> {
    let result = match op {
        BinaryOp::Add => builder.ins().iadd(lhs, rhs),
        BinaryOp::Subtract => builder.ins().isub(lhs, rhs),
        BinaryOp::Multiply => builder.ins().imul(lhs, rhs),
        BinaryOp::Divide => builder.ins().sdiv(lhs, rhs),
        BinaryOp::Modulo => builder.ins().srem(lhs, rhs),

        // Comparison operations
        BinaryOp::Equal => {
            let cmp = builder.ins().icmp(IntCC::Equal, lhs, rhs);
            builder.ins().uextend(types::I64, cmp)
        }
        BinaryOp::NotEqual => {
            let cmp = builder.ins().icmp(IntCC::NotEqual, lhs, rhs);
            builder.ins().uextend(types::I64, cmp)
        }
        BinaryOp::Less => {
            let cmp = builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs);
            builder.ins().uextend(types::I64, cmp)
        }
        BinaryOp::LessEqual => {
            let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs);
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

        // Logical operations
        BinaryOp::And => builder.ins().band(lhs, rhs),
        BinaryOp::Or => builder.ins().bor(lhs, rhs),

        _ => return Err(anyhow!("Unsupported binary operation: {op:?}")),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;
    use cranelift_jit::{JITBuilder, JITModule};
    use cranelift_module::Module;

    #[test]
    fn test_lower_literal() {
        let code = "42";
        let ast = Parser::new(code).parse().unwrap();

        // Create minimal Cranelift context for testing
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        let module = JITModule::new(builder);
        let mut ctx = codegen::Context::new();
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));
        ctx.func.signature = sig;

        let mut func_builder_ctx = FunctionBuilderContext::new();
        let mut func_builder = FunctionBuilder::new(&mut ctx.func, &mut func_builder_ctx);

        let entry_block = func_builder.create_block();
        func_builder.switch_to_block(entry_block);

        let result = lower_expr_to_value(&mut func_builder, &ast);
        assert!(result.is_ok(), "Should lower literal successfully");
    }
}
