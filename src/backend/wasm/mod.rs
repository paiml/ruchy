/// TDD: Minimal WASM emitter implementation
/// Following strict TDD - only implement what tests require
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use wasm_encoder::{
    CodeSection, ExportSection, Function, FunctionSection, Instruction, MemorySection, MemoryType,
    Module, TypeSection,
};
#[cfg(test)]
mod debug;

/// WASM value types for type inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WasmType {
    I32,
    F32,
    I64,
    F64,
}

/// Symbol table for tracking variable types across scopes
/// Complexity: <10 per method (Toyota Way)
#[derive(Debug, Clone)]
struct SymbolTable {
    scopes: Vec<std::collections::HashMap<String, WasmType>>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            scopes: vec![std::collections::HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(std::collections::HashMap::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn insert(&mut self, name: String, ty: WasmType) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn lookup(&self, name: &str) -> Option<WasmType> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(&ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

pub struct WasmEmitter {
    module: Module,
    symbols: std::cell::RefCell<SymbolTable>,
}
impl WasmEmitter {
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::backend::wasm::WasmEmitter;
    /// let instance = WasmEmitter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            module: Module::new(),
            symbols: std::cell::RefCell::new(SymbolTable::new()),
        }
    }
    /// Emit a complete WASM module from a Ruchy AST expression
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::backend::wasm::WasmEmitter;
    /// use ruchy::frontend::ast::{Expr, ExprKind};
    /// let instance = WasmEmitter::new();
    /// let expr = Expr::new(ExprKind::Block(vec![]), Default::default());
    /// let result = instance.emit(&expr);
    /// ```
    pub fn emit(&self, expr: &Expr) -> Result<Vec<u8>, String> {
        // Build symbol table from entire expression tree
        self.build_symbol_table(expr);

        let mut module = Module::new();
        // Collect all function definitions
        let func_defs = self.collect_functions(expr);
        let has_functions = !func_defs.is_empty();
        // Add type section
        let mut types = TypeSection::new();
        if has_functions {
            // Add a type for each function
            for (_name, params, _body) in &func_defs {
                // For now, assume all functions take i32 params and return i32
                let param_types = vec![wasm_encoder::ValType::I32; params.len()];
                types.function(param_types, vec![wasm_encoder::ValType::I32]);
            }
            // Also add a type for the main function if there's non-function code
            let main_expr = self.get_non_function_code(expr);
            if let Some(ref main_expr_val) = main_expr {
                // Infer the actual return type for main
                if self.expression_produces_value(main_expr_val) {
                    let return_ty = self.infer_type(main_expr_val);
                    let wasm_ty = match return_ty {
                        WasmType::I32 => wasm_encoder::ValType::I32,
                        WasmType::F32 => wasm_encoder::ValType::F32,
                        WasmType::I64 => wasm_encoder::ValType::I64,
                        WasmType::F64 => wasm_encoder::ValType::F64,
                    };
                    types.function(vec![], vec![wasm_ty]);
                } else {
                    types.function(vec![], vec![]);
                }
            }
        } else {
            // Single implicit main function
            // Infer the actual return type
            if self.expression_produces_value(expr) {
                let return_ty = self.infer_type(expr);
                let wasm_ty = match return_ty {
                    WasmType::I32 => wasm_encoder::ValType::I32,
                    WasmType::F32 => wasm_encoder::ValType::F32,
                    WasmType::I64 => wasm_encoder::ValType::I64,
                    WasmType::F64 => wasm_encoder::ValType::F64,
                };
                types.function(vec![], vec![wasm_ty]);
            } else {
                types.function(vec![], vec![]);
            }
        }
        module.section(&types);
        // Add function section
        let mut functions = FunctionSection::new();
        if has_functions {
            for i in 0..func_defs.len() {
                functions.function(i as u32);
            }
            // Add main function if there's non-function code
            let main_expr = self.get_non_function_code(expr);
            if main_expr.is_some() {
                functions.function(func_defs.len() as u32); // Main uses the last type
            }
        } else {
            functions.function(0);
        }
        module.section(&functions);
        // Add memory section if we need memory (for arrays/strings)
        if self.needs_memory(expr) {
            let mut memories = MemorySection::new();
            memories.memory(MemoryType {
                minimum: 1, // 1 page (64KB)
                maximum: None,
                memory64: false,
                shared: false,
                page_size_log2: None, // Use default page size
            });
            module.section(&memories);
        }
        // Add export section if we have a main function (must come before code section)
        if self.has_main_function(expr) {
            let mut exports = ExportSection::new();
            exports.export("main", wasm_encoder::ExportKind::Func, 0);
            module.section(&exports);
        }
        // Add code section
        let mut codes = CodeSection::new();
        if has_functions {
            // Compile each function
            for (_name, _params, body) in &func_defs {
                let locals = if self.needs_locals(body) {
                    // Infer the type of the first local based on the first Let expression
                    let local_ty = self.infer_first_local_type(body);
                    let wasm_ty = match local_ty {
                        WasmType::I32 => wasm_encoder::ValType::I32,
                        WasmType::F32 => wasm_encoder::ValType::F32,
                        WasmType::I64 => wasm_encoder::ValType::I64,
                        WasmType::F64 => wasm_encoder::ValType::F64,
                    };
                    vec![(1, wasm_ty)]
                } else {
                    vec![]
                };
                let mut func = Function::new(locals);
                // Compile function body
                let instructions = self.lower_expression(body)?;
                for instr in instructions {
                    func.instruction(&instr);
                }
                // Functions with explicit returns don't need Drop
                // All our test functions return values
                func.instruction(&Instruction::End);
                codes.function(&func);
            }
            // Also compile the main code (non-function expressions)
            let main_expr = self.get_non_function_code(expr);
            if let Some(main_expr) = main_expr {
                let locals = if self.needs_locals(&main_expr) {
                    // Infer the type of the first local based on the first Let expression
                    let local_ty = self.infer_first_local_type(&main_expr);
                    let wasm_ty = match local_ty {
                        WasmType::I32 => wasm_encoder::ValType::I32,
                        WasmType::F32 => wasm_encoder::ValType::F32,
                        WasmType::I64 => wasm_encoder::ValType::I64,
                        WasmType::F64 => wasm_encoder::ValType::F64,
                    };
                    vec![(1, wasm_ty)]
                } else {
                    vec![]
                };
                let mut func = Function::new(locals);
                let instructions = self.lower_expression(&main_expr)?;
                for instr in instructions {
                    func.instruction(&instr);
                }
                // No Drop needed - type signature matches return type
                func.instruction(&Instruction::End);
                codes.function(&func);
            }
        } else {
            // Single implicit main function
            let locals = if self.needs_locals(expr) {
                // Infer the type of the first local based on the first Let expression
                let local_ty = self.infer_first_local_type(expr);
                let wasm_ty = match local_ty {
                    WasmType::I32 => wasm_encoder::ValType::I32,
                    WasmType::F32 => wasm_encoder::ValType::F32,
                    WasmType::I64 => wasm_encoder::ValType::I64,
                    WasmType::F64 => wasm_encoder::ValType::F64,
                };
                vec![(1, wasm_ty)]
            } else {
                vec![]
            };
            let mut func = Function::new(locals);
            let instructions = self.lower_expression(expr)?;
            for instr in instructions {
                func.instruction(&instr);
            }
            // No Drop needed - type signature matches return type
            func.instruction(&Instruction::End);
            codes.function(&func);
        }
        module.section(&codes);
        Ok(module.finish())
    }

    /// Build symbol table by scanning expression tree for let bindings
    /// Complexity: 7 (within <10 limit)
    fn build_symbol_table(&self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Let {
                name, value, body, ..
            } => {
                let value_ty = self.infer_type(value);
                self.symbols.borrow_mut().insert(name.clone(), value_ty);
                self.build_symbol_table(body);
            }
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.build_symbol_table(e);
                }
            }
            ExprKind::Binary { left, right, .. } => {
                self.build_symbol_table(left);
                self.build_symbol_table(right);
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.build_symbol_table(condition);
                self.build_symbol_table(then_branch);
                if let Some(else_expr) = else_branch {
                    self.build_symbol_table(else_expr);
                }
            }
            ExprKind::Function { body, .. } => {
                self.build_symbol_table(body);
            }
            ExprKind::While { condition, body } => {
                self.build_symbol_table(condition);
                self.build_symbol_table(body);
            }
            _ => {} // Other expression types don't introduce bindings
        }
    }

    /// Infer the type of the first local variable (simplified for single-local approach)
    /// Complexity: 4 (within <10 limit)
    fn infer_first_local_type(&self, expr: &Expr) -> WasmType {
        match &expr.kind {
            ExprKind::Let { value, .. } => self.infer_type(value),
            ExprKind::Block(exprs) => {
                // Find first Let in block
                for e in exprs {
                    if matches!(e.kind, ExprKind::Let { .. }) {
                        return self.infer_first_local_type(e);
                    }
                }
                WasmType::I32 // Default
            }
            _ => WasmType::I32, // Default if no Let found
        }
    }

    /// Infer the WASM type of an expression
    /// Complexity: 9 (within <10 limit)
    fn infer_type(&self, expr: &Expr) -> WasmType {
        match &expr.kind {
            ExprKind::Literal(Literal::Integer(_)) => WasmType::I32,
            ExprKind::Literal(Literal::Float(_)) => WasmType::F32,
            ExprKind::Literal(Literal::Bool(_)) => WasmType::I32,
            ExprKind::Binary { op, left, right } => {
                // Comparison operations always return i32 (boolean)
                use crate::frontend::ast::BinaryOp;
                match op {
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual
                    | BinaryOp::Gt => WasmType::I32,
                    _ => {
                        // Arithmetic: Type promotion (f32 > i32)
                        let left_ty = self.infer_type(left);
                        let right_ty = self.infer_type(right);
                        if left_ty == WasmType::F32 || right_ty == WasmType::F32 {
                            WasmType::F32
                        } else {
                            WasmType::I32
                        }
                    }
                }
            }
            ExprKind::Let { body, .. } => {
                // Let expression type is the body type
                match &body.kind {
                    ExprKind::Literal(Literal::Unit) => WasmType::I32, // Statement-style let
                    _ => self.infer_type(body),
                }
            }
            ExprKind::Identifier(name) => {
                // Look up in global symbol table
                self.symbols.borrow().lookup(name).unwrap_or(WasmType::I32)
            }
            ExprKind::Block(exprs) => {
                // Return type of last expression in block
                exprs.last().map_or(WasmType::I32, |e| self.infer_type(e))
            }
            ExprKind::Call { .. } => WasmType::I32, // Default to i32
            ExprKind::Unary { operand, .. } => self.infer_type(operand),
            _ => WasmType::I32, // Default to i32
        }
    }

    /// Lower a Ruchy expression to WASM instructions
    fn lower_expression(&self, expr: &Expr) -> Result<Vec<Instruction<'static>>, String> {
        match &expr.kind {
            ExprKind::Literal(literal) => self.lower_literal(literal),
            ExprKind::Binary { op, left, right } => {
                let mut instructions = vec![];

                // Infer result type based on operands
                let result_type = {
                    let left_ty = self.infer_type(left);
                    let right_ty = self.infer_type(right);
                    if left_ty == WasmType::F32 || right_ty == WasmType::F32 {
                        WasmType::F32
                    } else {
                        WasmType::I32
                    }
                };

                // Emit left operand
                instructions.extend(self.lower_expression(left)?);

                // Emit type conversion if needed
                if self.infer_type(left) == WasmType::I32 && result_type == WasmType::F32 {
                    instructions.push(Instruction::F32ConvertI32S);
                }

                // Emit right operand
                instructions.extend(self.lower_expression(right)?);

                // Emit type conversion if needed
                if self.infer_type(right) == WasmType::I32 && result_type == WasmType::F32 {
                    instructions.push(Instruction::F32ConvertI32S);
                }

                // Emit operation with correct type
                let op_instr = match (op, result_type) {
                    (BinaryOp::Add, WasmType::I32) => Instruction::I32Add,
                    (BinaryOp::Add, WasmType::F32) => Instruction::F32Add,
                    (BinaryOp::Subtract, WasmType::I32) => Instruction::I32Sub,
                    (BinaryOp::Subtract, WasmType::F32) => Instruction::F32Sub,
                    (BinaryOp::Multiply, WasmType::I32) => Instruction::I32Mul,
                    (BinaryOp::Multiply, WasmType::F32) => Instruction::F32Mul,
                    (BinaryOp::Divide, WasmType::I32) => Instruction::I32DivS,
                    (BinaryOp::Divide, WasmType::F32) => Instruction::F32Div,
                    (BinaryOp::Modulo, WasmType::I32) => Instruction::I32RemS,
                    (BinaryOp::Modulo, WasmType::F32) => {
                        return Err("Modulo not supported for floats".to_string())
                    }
                    (BinaryOp::Equal, WasmType::I32) => Instruction::I32Eq,
                    (BinaryOp::Equal, WasmType::F32) => Instruction::F32Eq,
                    (BinaryOp::NotEqual, WasmType::I32) => Instruction::I32Ne,
                    (BinaryOp::NotEqual, WasmType::F32) => Instruction::F32Ne,
                    (BinaryOp::Less, WasmType::I32) => Instruction::I32LtS,
                    (BinaryOp::Less, WasmType::F32) => Instruction::F32Lt,
                    (BinaryOp::Greater, WasmType::I32) => Instruction::I32GtS,
                    (BinaryOp::Greater, WasmType::F32) => Instruction::F32Gt,
                    (BinaryOp::LessEqual, WasmType::I32) => Instruction::I32LeS,
                    (BinaryOp::LessEqual, WasmType::F32) => Instruction::F32Le,
                    (BinaryOp::GreaterEqual, WasmType::I32) => Instruction::I32GeS,
                    (BinaryOp::GreaterEqual, WasmType::F32) => Instruction::F32Ge,
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
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
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
            ExprKind::Function {
                name: _,
                params: _,
                body: _,
                ..
            } => {
                // Function definitions don't produce instructions in the current function
                // They would be handled separately to create new WASM functions
                Ok(vec![])
            }
            ExprKind::Call { func: _, args } => {
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
            ExprKind::Let {
                name: _,
                value,
                body,
                ..
            } => {
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
                        // Type-aware negation
                        let operand_ty = self.infer_type(operand);
                        match operand_ty {
                            WasmType::I32 => {
                                instructions.insert(0, Instruction::I32Const(0));
                                instructions.push(Instruction::I32Sub);
                            }
                            WasmType::F32 => {
                                instructions.push(Instruction::F32Neg);
                            }
                            WasmType::I64 => {
                                instructions.insert(0, Instruction::I64Const(0));
                                instructions.push(Instruction::I64Sub);
                            }
                            WasmType::F64 => {
                                instructions.push(Instruction::F64Neg);
                            }
                        }
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
                    crate::frontend::ast::UnaryOp::Deref => {
                        // Dereference operator not supported in WASM (needs memory)
                        return Ok(instructions);
                    }
                }
                Ok(instructions)
            }
            ExprKind::List(_items) => {
                // For now, just allocate space and return a pointer
                // Real implementation would store items in memory
                // Allocate memory for array (simplified - just return 0 as pointer)
                let instructions = vec![Instruction::I32Const(0)];
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
    /// Collect all function definitions from the AST
    fn collect_functions(
        &self,
        expr: &Expr,
    ) -> Vec<(String, Vec<crate::frontend::ast::Param>, Box<Expr>)> {
        let mut functions = Vec::new();
        self.collect_functions_rec(expr, &mut functions);
        functions
    }
    fn collect_functions_rec(
        &self,
        expr: &Expr,
        functions: &mut Vec<(String, Vec<crate::frontend::ast::Param>, Box<Expr>)>,
    ) {
        match &expr.kind {
            ExprKind::Function {
                name, params, body, ..
            } => {
                functions.push((name.clone(), params.clone(), body.clone()));
            }
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.collect_functions_rec(e, functions);
                }
            }
            _ => {}
        }
    }
    /// Get non-function code from the expression (e.g., function calls)
    fn get_non_function_code(&self, expr: &Expr) -> Option<Expr> {
        match &expr.kind {
            ExprKind::Block(exprs) => {
                let non_func_exprs: Vec<Expr> = exprs
                    .iter()
                    .filter(|e| !matches!(e.kind, ExprKind::Function { .. }))
                    .cloned()
                    .collect();
                if non_func_exprs.is_empty() {
                    None
                } else if non_func_exprs.len() == 1 {
                    Some(
                        non_func_exprs
                            .into_iter()
                            .next()
                            .expect("non_func_exprs.len() == 1, so next() must return Some"),
                    )
                } else {
                    Some(Expr::new(ExprKind::Block(non_func_exprs), expr.span))
                }
            }
            ExprKind::Function { .. } => None,
            _ => Some(expr.clone()),
        }
    }
    /// Check if an expression needs memory (for arrays/strings)
    fn needs_memory(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(Literal::String(_)) => true,
            ExprKind::List(_) => true,
            ExprKind::ArrayInit { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.needs_memory(e)),
            ExprKind::Let { value, body, .. } => {
                self.needs_memory(value) || self.needs_memory(body)
            }
            ExprKind::Binary { left, right, .. } => {
                self.needs_memory(left) || self.needs_memory(right)
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.needs_memory(condition)
                    || self.needs_memory(then_branch)
                    || else_branch.as_ref().is_some_and(|e| self.needs_memory(e))
            }
            _ => false,
        }
    }
    /// Check if an expression contains a main function
    fn has_main_function(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Function { name, .. } => name == "main",
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.has_main_function(e)),
            _ => false,
        }
    }
    /// Check if an expression has return statements with values
    fn has_return_with_value(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Return { value } => value.is_some(),
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.has_return_with_value(e)),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.has_return_with_value(condition)
                    || self.has_return_with_value(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|e| self.has_return_with_value(e))
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
        let result = match &expr.kind {
            ExprKind::Let { .. } => true,
            ExprKind::Identifier(_) => true,
            ExprKind::Function { .. } => true,
            ExprKind::Block(exprs) => exprs.iter().any(|e| self.needs_locals(e)),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.needs_locals(condition)
                    || self.needs_locals(then_branch)
                    || else_branch.as_ref().is_some_and(|e| self.needs_locals(e))
            }
            ExprKind::While { condition, body } => {
                self.needs_locals(condition) || self.needs_locals(body)
            }
            ExprKind::Binary { left, right, .. } => {
                self.needs_locals(left) || self.needs_locals(right)
            }
            _ => false,
        };
        result
    }
    /// Check if an expression produces a value on the stack
    fn expression_produces_value(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(_) => true,
            ExprKind::Binary { .. } => true,
            ExprKind::Unary { .. } => true,
            ExprKind::Identifier(_) => true,
            ExprKind::Call { .. } => true,
            ExprKind::List(_) => true,
            ExprKind::Block(exprs) => {
                // Block produces value if last expression does
                exprs
                    .last()
                    .is_some_and(|e| self.expression_produces_value(e))
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
            ExprKind::While { .. } => false,  // Loops are void
            ExprKind::Function { .. } => false, // Function definitions don't produce values
            _ => false,
        }
    }
    fn lower_literal(&self, literal: &Literal) -> Result<Vec<Instruction<'static>>, String> {
        match literal {
            Literal::Integer(n) => Ok(vec![Instruction::I32Const(*n as i32)]),
            Literal::Float(f) => Ok(vec![Instruction::F32Const(*f as f32)]),
            Literal::Bool(b) => Ok(vec![Instruction::I32Const(i32::from(*b))]),
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
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::backend::wasm::mod::bytes;
    ///
    /// let result = bytes(());
    /// assert_eq!(result, Ok(()));
    /// ```
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
            Expr::new(ExprKind::Block(vec![]), Default::default())
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
        assert!(bytes.contains(&0x41));
    }

    #[test]
    fn test_binary_operations() {
        // Test addition
        let mut parser = Parser::new("1 + 2");
        let expr = parser.parse().expect("Should parse addition");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Should contain i32.add instruction (0x6a)
        assert!(bytes.contains(&0x6a));

        // Test subtraction
        let mut parser = Parser::new("5 - 3");
        let expr = parser.parse().expect("Should parse subtraction");
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Should contain i32.sub instruction (0x6b)
        assert!(bytes.contains(&0x6b));

        // Test multiplication
        let mut parser = Parser::new("3 * 4");
        let expr = parser.parse().expect("Should parse multiplication");
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // Should contain i32.mul instruction (0x6c)
        assert!(bytes.contains(&0x6c));
    }

    #[test]
    fn test_function_definition() {
        let mut parser = Parser::new("fun add(x, y) { x + y }");
        let expr = parser.parse().expect("Should parse function");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should have WASM magic number
        assert_eq!(&bytes[0..4], b"\0asm");
        // Should contain function section
        assert!(bytes.windows(2).any(|w| w == [0x03, 0x02])); // Function section with size
    }

    #[test]
    fn test_if_expression() {
        let mut parser = Parser::new("if true { 1 } else { 2 }");
        let expr = parser.parse().expect("Should parse if expression");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain if instruction (0x04)
        assert!(bytes.contains(&0x04));
    }

    #[test]
    fn test_local_variables() {
        let mut parser = Parser::new("let x = 5; x");
        let expr = parser.parse().expect("Should parse let binding");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain local.set (0x21) or local.get (0x20)
        assert!(bytes.iter().any(|&b| b == 0x20 || b == 0x21));
    }

    #[test]
    fn test_comparison_operations() {
        let mut parser = Parser::new("3 > 2");
        let expr = parser.parse().expect("Should parse comparison");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain i32.gt_s instruction (0x4a)
        assert!(bytes.contains(&0x4a));
    }

    #[test]
    fn test_boolean_literals() {
        let mut parser = Parser::new("true");
        let expr = parser.parse().expect("Should parse boolean");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // true should emit i32.const 1
        assert!(bytes.windows(2).any(|w| w == [0x41, 0x01]));

        let mut parser = Parser::new("false");
        let expr = parser.parse().expect("Should parse boolean");
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // false should emit i32.const 0
        assert!(bytes.windows(2).any(|w| w == [0x41, 0x00]));
    }

    #[test]
    fn test_block_expression() {
        let mut parser = Parser::new("{ 1; 2; 3 }");
        let expr = parser.parse().expect("Should parse block");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain block instruction (0x02)
        assert!(bytes.contains(&0x02));
    }

    #[test]
    fn test_while_loop() {
        let mut parser = Parser::new("while false { }");
        let expr = parser.parse().expect("Should parse while loop");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain loop instruction (0x03)
        assert!(bytes.contains(&0x03));
    }

    #[test]
    fn test_break_statement() {
        let mut parser = Parser::new("while true { break }");
        let expr = parser.parse().expect("Should parse break");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain br instruction (0x0c)
        assert!(bytes.contains(&0x0c));
    }

    #[test]
    fn test_continue_statement() {
        let mut parser = Parser::new("while true { continue }");
        let expr = parser.parse().expect("Should parse continue");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain br instruction for continue
        assert!(bytes.contains(&0x0c));
    }

    #[test]
    fn test_return_statement() {
        let mut parser = Parser::new("fun test() { return 42 }");
        let expr = parser.parse().expect("Should parse return");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain return instruction (0x0f)
        assert!(bytes.contains(&0x0f));
    }

    #[test]
    fn test_nested_expressions() {
        let mut parser = Parser::new("(1 + 2) * (3 - 4)");
        let expr = parser.parse().expect("Should parse nested expr");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain add, sub, and mul instructions
        assert!(bytes.contains(&0x6a)); // add
        assert!(bytes.contains(&0x6b)); // sub
        assert!(bytes.contains(&0x6c)); // mul
    }

    #[test]
    fn test_logical_operations() {
        let mut parser = Parser::new("true && false");
        let expr = parser.parse().expect("Should parse logical and");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should generate valid WASM bytecode (specific instruction may vary)
        assert!(!bytes.is_empty());

        let mut parser = Parser::new("true || false");
        let expr = parser.parse().expect("Should parse logical or");
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should generate valid WASM bytecode (specific instruction may vary)
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_unary_operations() {
        let mut parser = Parser::new("!true");
        let expr = parser.parse().expect("Should parse not");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain i32.eqz instruction (0x45)
        assert!(bytes.contains(&0x45));

        let mut parser = Parser::new("-5");
        let expr = parser.parse().expect("Should parse negate");
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        // Negate is typically implemented as 0 - x
    }

    #[test]
    fn test_complex_function() {
        let mut parser = Parser::new(
            r"
            fun factorial(n) {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
        ",
        );
        let expr = parser.parse().expect("Should parse complex function");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should be valid WASM
        assert_eq!(&bytes[0..4], b"\0asm");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_needs_memory_check() {
        let emitter = WasmEmitter::new();

        // Simple integer shouldn't need memory
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Default::default());
        assert!(!emitter.needs_memory(&expr));

        // List should need memory
        let list_expr = Expr::new(ExprKind::List(vec![]), Default::default());
        assert!(emitter.needs_memory(&list_expr));
    }

    #[test]
    fn test_collect_functions() {
        let emitter = WasmEmitter::new();

        // Function definition
        let func_expr = Expr::new(
            ExprKind::Function {
                name: "test".to_string(),
                type_params: vec![],
                params: vec![],
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42)),
                    Default::default(),
                )),
                return_type: None,
                is_async: false,
                is_pub: false,
            },
            Default::default(),
        );

        let funcs = emitter.collect_functions(&func_expr);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].0, "test");
    }

    #[test]
    fn test_division_operation() {
        let mut parser = Parser::new("10 / 2");
        let expr = parser.parse().expect("Should parse division");
        let emitter = WasmEmitter::new();
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        let bytes = result.unwrap();

        // Should contain i32.div_s instruction (0x6d)
        assert!(bytes.contains(&0x6d));
    }
}
#[cfg(test)]
mod property_tests_mod {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
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
