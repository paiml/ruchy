/// TDD: Minimal WASM emitter implementation
/// Following strict TDD - only implement what tests require

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use wasm_encoder::{
    CodeSection, ExportSection, Function, FunctionSection,
    Instruction, Module, TypeSection,
};

#[cfg(test)]
mod debug;

pub struct WasmEmitter {
    module: Module,
}

impl WasmEmitter {
    pub fn new() -> Self {
        Self {
            module: Module::new(),
        }
    }

    /// Emit a complete WASM module from a Ruchy AST expression
    pub fn emit(&self, expr: &Expr) -> Result<Vec<u8>, String> {
        let mut module = Module::new();

        // Add type section with main function type
        let mut types = TypeSection::new();
        
        // Check if we have return statements with values
        let has_return_value = self.has_return_with_value(expr);
        if has_return_value {
            types.function(vec![], vec![wasm_encoder::ValType::I32]); // () -> i32 type
        } else {
            types.function(vec![], vec![]); // () -> () type
        }
        module.section(&types);

        // Add function section pointing to type 0
        let mut functions = FunctionSection::new();
        functions.function(0); // Use type 0
        module.section(&functions);

        // Add code section with our expression
        let mut codes = CodeSection::new();
        
        // Create a minimal function body for the expression
        // Add one local for simple variable storage (if needed)
        let locals = if self.needs_locals(expr) {
            vec![(1, wasm_encoder::ValType::I32)]
        } else {
            vec![]
        };
        let mut func = Function::new(locals);
        
        // Lower the expression to WASM instructions
        let instructions = self.lower_expression(expr)?;
        let has_instructions = !instructions.is_empty();
        
        for instr in instructions {
            func.instruction(&instr);
        }
        
        // Drop any remaining stack values (void function)
        // Count how many values the expression produces
        if has_instructions && self.expression_produces_value(expr) && !has_return_value {
            func.instruction(&Instruction::Drop);
        }
        
        // End the function
        func.instruction(&Instruction::End);
        codes.function(&func);
        module.section(&codes);

        // Export section is optional - don't add if empty

        Ok(module.finish())
    }

    /// Lower a Ruchy expression to WASM instructions
    fn lower_expression(&self, expr: &Expr) -> Result<Vec<Instruction>, String> {
        match &expr.kind {
            ExprKind::Literal(literal) => self.lower_literal(literal),
            ExprKind::Binary { op, left, right } => {
                let mut instructions = vec![];
                
                // Emit left operand
                instructions.extend(self.lower_expression(left)?);
                
                // Emit right operand
                instructions.extend(self.lower_expression(right)?);
                
                // Emit operation
                let op_instr = match op {
                    BinaryOp::Add => Instruction::I32Add,
                    BinaryOp::Subtract => Instruction::I32Sub,
                    BinaryOp::Multiply => Instruction::I32Mul,
                    BinaryOp::Divide => Instruction::I32DivS,
                    BinaryOp::Modulo => Instruction::I32RemS,
                    BinaryOp::Equal => Instruction::I32Eq,
                    BinaryOp::NotEqual => Instruction::I32Ne,
                    BinaryOp::Less => Instruction::I32LtS,
                    BinaryOp::Greater => Instruction::I32GtS,
                    BinaryOp::LessEqual => Instruction::I32LeS,
                    BinaryOp::GreaterEqual => Instruction::I32GeS,
                    _ => return Ok(instructions), // Skip unsupported ops for now
                };
                instructions.push(op_instr);
                
                Ok(instructions)
            }
            ExprKind::Block(exprs) => {
                // Handle block expressions (e.g., from parse())
                let mut instructions = vec![];
                for (i, expr) in exprs.iter().enumerate() {
                    instructions.extend(self.lower_expression(expr)?);
                    
                    // Drop intermediate values (keep only last if block produces value)
                    let is_last = i == exprs.len() - 1;
                    if !is_last && self.expression_produces_value(expr) {
                        instructions.push(Instruction::Drop);
                    }
                }
                Ok(instructions)
            }
            ExprKind::If { condition, then_branch, else_branch } => {
                let mut instructions = vec![];
                
                // Emit condition
                instructions.extend(self.lower_expression(condition)?);
                
                // Determine block type based on whether branches produce values
                let block_type = if self.expression_produces_value(then_branch) {
                    wasm_encoder::BlockType::Result(wasm_encoder::ValType::I32)
                } else {
                    wasm_encoder::BlockType::Empty
                };
                
                // If instruction
                instructions.push(Instruction::If(block_type));
                
                // Then branch
                instructions.extend(self.lower_expression(then_branch)?);
                
                // Else branch (if present)
                if let Some(else_expr) = else_branch {
                    instructions.push(Instruction::Else);
                    instructions.extend(self.lower_expression(else_expr)?);
                } else if self.expression_produces_value(then_branch) {
                    // If no else branch but we expect a value, push a default
                    instructions.push(Instruction::Else);
                    instructions.push(Instruction::I32Const(0));
                }
                
                // End if
                instructions.push(Instruction::End);
                
                Ok(instructions)
            }
            ExprKind::While { condition, body } => {
                let mut instructions = vec![];
                
                // Loop instruction
                instructions.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
                
                // Check condition
                instructions.extend(self.lower_expression(condition)?);
                
                // Branch if false (exit loop)
                instructions.push(Instruction::I32Eqz);
                instructions.push(Instruction::BrIf(1)); // Break out of loop
                
                // Body
                instructions.extend(self.lower_expression(body)?);
                
                // Branch back to loop start
                instructions.push(Instruction::Br(0));
                
                // End loop
                instructions.push(Instruction::End);
                
                Ok(instructions)
            }
            ExprKind::Function { name: _, params: _, body: _, .. } => {
                // Function definitions don't produce instructions in the current function
                // They would be handled separately to create new WASM functions
                Ok(vec![])
            }
            ExprKind::Call { func, args } => {
                let mut instructions = vec![];
                
                // Push arguments onto stack
                for arg in args {
                    instructions.extend(self.lower_expression(arg)?);
                }
                
                // For now, we'll emit a placeholder call instruction
                // In a real implementation, this would resolve the function index
                instructions.push(Instruction::Call(0));
                
                Ok(instructions)
            }
            ExprKind::Let { name: _, value, body, .. } => {
                let mut instructions = vec![];
                
                // Compile the value
                instructions.extend(self.lower_expression(value)?);
                
                // Store in local (simplified - would need local index tracking)
                instructions.push(Instruction::LocalSet(0));
                
                // Compile the body if it's not Unit
                match &body.kind {
                    ExprKind::Literal(Literal::Unit) => {
                        // Statement-style let doesn't produce a value
                    }
                    _ => {
                        // Expression-style let produces the body value
                        instructions.extend(self.lower_expression(body)?);
                    }
                }
                
                Ok(instructions)
            }
            ExprKind::Identifier(_name) => {
                // Load from local (simplified - would need local index tracking)
                Ok(vec![Instruction::LocalGet(0)])
            }
            ExprKind::Unary { op, operand } => {
                let mut instructions = vec![];
                
                // Emit operand
                instructions.extend(self.lower_expression(operand)?);
                
                // Emit unary operation
                match op {
                    crate::frontend::ast::UnaryOp::Negate => {
                        // Negate by subtracting from 0
                        instructions.insert(0, Instruction::I32Const(0));
                        instructions.push(Instruction::I32Sub);
                    }
                    crate::frontend::ast::UnaryOp::Not => {
                        // Logical not: compare with 0
                        instructions.push(Instruction::I32Eqz);
                    }
                    crate::frontend::ast::UnaryOp::BitwiseNot => {
                        // Bitwise not: XOR with -1
                        instructions.push(Instruction::I32Const(-1));
                        instructions.push(Instruction::I32Xor);
                    }
                    crate::frontend::ast::UnaryOp::Reference => {
                        // Reference operator not supported in WASM (needs memory)
                        return Ok(instructions);
                    }
                }
                
                Ok(instructions)
            }
            ExprKind::Return { value } => {
                let mut instructions = vec![];
                
                // Compile the return value
                if let Some(val) = value {
                    instructions.extend(self.lower_expression(val)?);
                }
                
                // Return instruction
                instructions.push(Instruction::Return);
                
                Ok(instructions)
            }
            _ => Ok(vec![]), // Skip complex expressions for now
        }
    }

    /// Check if an expression has return statements with values
    fn has_return_with_value(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Return { value } => value.is_some(),
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.has_return_with_value(e)),
            ExprKind::If { condition, then_branch, else_branch } => {
                self.has_return_with_value(condition) ||
                self.has_return_with_value(then_branch) ||
                else_branch.as_ref().map_or(false, |e| self.has_return_with_value(e))
            }
            ExprKind::While { condition, body } => {
                self.has_return_with_value(condition) || self.has_return_with_value(body)
            }
            ExprKind::Function { .. } => false, // Functions are compiled separately
            ExprKind::Let { value, body, .. } => {
                self.has_return_with_value(value) || self.has_return_with_value(body)
            }
            ExprKind::Binary { left, right, .. } => {
                self.has_return_with_value(left) || self.has_return_with_value(right)
            }
            _ => false,
        }
    }

    /// Check if an expression needs local variables
    fn needs_locals(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Let { .. } => true,
            ExprKind::Identifier(_) => true,
            ExprKind::Function { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.needs_locals(e)),
            ExprKind::If { condition, then_branch, else_branch } => {
                self.needs_locals(condition) ||
                self.needs_locals(then_branch) ||
                else_branch.as_ref().map_or(false, |e| self.needs_locals(e))
            }
            ExprKind::While { condition, body } => {
                self.needs_locals(condition) || self.needs_locals(body)
            }
            ExprKind::Binary { left, right, .. } => {
                self.needs_locals(left) || self.needs_locals(right)
            }
            _ => false,
        }
    }

    /// Check if an expression produces a value on the stack
    fn expression_produces_value(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(_) => true,
            ExprKind::Binary { .. } => true,
            ExprKind::Unary { .. } => true,
            ExprKind::Identifier(_) => true,
            ExprKind::Call { .. } => true,
            ExprKind::Block(exprs) => {
                // Block produces value if last expression does
                exprs.last().map_or(false, |e| self.expression_produces_value(e))
            }
            ExprKind::If { .. } => true,
            ExprKind::Let { body, .. } => {
                // Let produces value only if body is not Unit
                match &body.kind {
                    ExprKind::Literal(Literal::Unit) => false,
                    _ => self.expression_produces_value(body),
                }
            }
            ExprKind::Return { .. } => false, // Return doesn't leave value on stack
            ExprKind::While { .. } => false, // Loops are void
            ExprKind::Function { .. } => false, // Function definitions don't produce values
            _ => false,
        }
    }

    fn lower_literal(&self, literal: &Literal) -> Result<Vec<Instruction>, String> {
        match literal {
            Literal::Integer(n) => Ok(vec![Instruction::I32Const(*n as i32)]),
            Literal::Float(f) => Ok(vec![Instruction::F32Const(*f as f32)]),
            Literal::Bool(b) => Ok(vec![Instruction::I32Const(if *b { 1 } else { 0 })]),
            Literal::String(_) => {
                // String literals would need memory allocation
                // For now, return a placeholder
                Ok(vec![Instruction::I32Const(0)])
            }
            _ => Ok(vec![]), // Skip other literals for now
        }
    }
}

impl Default for WasmEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a compiled WASM module
pub struct WasmModule {
    bytes: Vec<u8>,
}

impl WasmModule {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_emitter_creates() {
        let _emitter = WasmEmitter::new();
    }

    #[test]
    fn test_empty_program_emits() {
        let mut parser = Parser::new("");
        let expr = parser.parse().unwrap_or_else(|_| {
            // Create an empty block expression for empty input
            Expr::new(
                ExprKind::Block(vec![]),
                Default::default(),
            )
        });
        
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        
        // Check WASM magic number
        assert_eq!(&bytes[0..4], b"\0asm");
        // Check version
        assert_eq!(&bytes[4..8], &[1, 0, 0, 0]);
    }

    #[test]
    fn test_integer_literal() {
        let mut parser = Parser::new("42");
        let expr = parser.parse().expect("Should parse integer");
        
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        
        assert!(result.is_ok());
        let bytes = result.unwrap();
        
        // Should contain i32.const instruction (0x41)
        assert!(bytes.iter().any(|&b| b == 0x41));
    }
}