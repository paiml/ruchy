//! WASM Emitter Implementation
//!
//! Core WASM code generation from Ruchy AST.

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, StringPart};
use wasm_encoder::{
    CodeSection, ConstExpr, ExportSection, Function, FunctionSection, GlobalSection, GlobalType,
    Instruction, MemorySection, MemoryType, Module, TypeSection, ValType,
};

use super::symbol_table::SymbolTable;
use super::types::WasmType;
use super::utils;

pub struct WasmEmitter {
    module: Module,
    symbols: std::cell::RefCell<SymbolTable>,
    /// Maps function name to `(index, is_void)`
    functions: std::cell::RefCell<std::collections::HashMap<String, (u32, bool)>>,
    /// Maps struct name to ordered field names (order determines memory offset)
    /// Complexity: Field at index N is at offset N * 4 bytes
    structs: std::cell::RefCell<std::collections::HashMap<String, Vec<String>>>,
    /// Maps variable name to tuple element types (for mixed-type tuple support)
    /// Example: "x" -> [I32, F32] for tuple (1, 3.0)
    tuple_types: std::cell::RefCell<std::collections::HashMap<String, Vec<WasmType>>>,
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
            functions: std::cell::RefCell::new(std::collections::HashMap::new()),
            structs: std::cell::RefCell::new(std::collections::HashMap::new()),
            tuple_types: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }

    /// Infer element WASM type from expression for tuple support
    /// Complexity: 5 (Toyota Way: <10 ✓)
    ///
    /// Returns `WasmType` based on expression kind
    /// Used for mixed-type tuple support
    pub(crate) fn infer_element_type(&self, expr: &Expr) -> WasmType {
        match &expr.kind {
            ExprKind::Literal(Literal::Float(_)) => WasmType::F32,
            ExprKind::Literal(Literal::Integer(_, _)) => WasmType::I32,
            ExprKind::Literal(Literal::Bool(_)) => WasmType::I32,
            ExprKind::Literal(Literal::String(_)) => WasmType::I32, // Address
            ExprKind::Binary { op, .. } => {
                // Float operations return float, others return int
                match op {
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo => WasmType::F32, // Could be either
                    _ => WasmType::I32, // Comparisons, logical ops
                }
            }
            _ => WasmType::I32, // Default to I32 for complex expressions
        }
    }

    /// Collect tuple element types from AST for mixed-type support
    /// Complexity: 7 (Toyota Way: <10 ✓)
    ///
    /// Traverses AST recursively to find `Let { value: Tuple(..) }` nodes
    /// Stores variable name → element types mapping for correct load/store
    fn collect_tuple_types(&self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Let {
                name, value, body, ..
            } => {
                // Check if value is a tuple and register element types
                if let ExprKind::Tuple(elements) = &value.kind {
                    let element_types: Vec<WasmType> = elements
                        .iter()
                        .map(|e| self.infer_element_type(e))
                        .collect();
                    self.tuple_types
                        .borrow_mut()
                        .insert(name.clone(), element_types);
                }
                // Recursively traverse
                self.collect_tuple_types(value);
                self.collect_tuple_types(body);
            }
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.collect_tuple_types(e);
                }
            }
            ExprKind::Function { body, .. } => {
                self.collect_tuple_types(body);
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_tuple_types(condition);
                self.collect_tuple_types(then_branch);
                if let Some(else_expr) = else_branch {
                    self.collect_tuple_types(else_expr);
                }
            }
            _ => {}
        }
    }

    /// Collect struct definitions from AST to build field layout map
    /// Complexity: 8 (Toyota Way: <10 ✓)
    ///
    /// Traverses AST recursively to find `ExprKind::Struct` nodes
    /// Stores struct name → field names mapping for offset calculation
    fn collect_struct_definitions(&self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Struct { name, fields, .. } => {
                // Extract field names in order (order determines memory offset)
                let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                self.structs.borrow_mut().insert(name.clone(), field_names);
            }
            // Recursively traverse block expressions
            ExprKind::Block(exprs) => {
                for e in exprs {
                    self.collect_struct_definitions(e);
                }
            }
            // Recursively traverse let bindings
            ExprKind::Let { value, body, .. } => {
                self.collect_struct_definitions(value);
                self.collect_struct_definitions(body);
            }
            // Recursively traverse function definitions
            ExprKind::Function { body, .. } => {
                self.collect_struct_definitions(body);
            }
            _ => {}
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
        // Collect tuple types BEFORE building symbol table (needed for type inference)
        self.collect_tuple_types(expr);

        // Build symbol table from entire expression tree
        self.build_symbol_table(expr);

        // Collect struct definitions for field layout mapping
        self.collect_struct_definitions(expr);

        let mut module = Module::new();
        let func_defs = self.collect_functions(expr);

        // Build function index map (must be done after collecting functions)
        self.build_function_index_map(expr, &func_defs);

        // Add sections to module (order matters in WASM)
        let types = self.emit_type_section(expr, &func_defs);
        module.section(&types);

        // Import section must come before function section
        if let Some(imports) = self.emit_import_section(expr) {
            module.section(&imports);
        }

        let functions = self.emit_function_section(&func_defs, expr);
        module.section(&functions);

        if let Some(memories) = self.emit_memory_section(expr) {
            module.section(&memories);
        }

        // Global section for heap pointer (if memory is needed)
        if let Some(globals) = self.emit_global_section(expr) {
            module.section(&globals);
        }

        if let Some(exports) = self.emit_export_section(expr) {
            module.section(&exports);
        }

        let codes = self.emit_code_section(expr, &func_defs)?;
        module.section(&codes);

        Ok(module.finish())
    }

    /// Emit type section with function signatures
    /// Complexity: 7 (Toyota Way: <10 ✓)
    fn emit_type_section(
        &self,
        expr: &Expr,
        func_defs: &[(String, Vec<crate::frontend::ast::Param>, Box<Expr>)],
    ) -> TypeSection {
        let mut types = TypeSection::new();
        let has_functions = !func_defs.is_empty();

        // Type index 0: Built-in functions println_i32 - (i32) -> ()
        // Type index 1: Built-in functions println_f32 - (f32) -> ()
        // These must be first because imports reference them
        if utils::uses_builtins(expr) {
            types.function(vec![wasm_encoder::ValType::I32], vec![]);
            types.function(vec![wasm_encoder::ValType::F32], vec![]);
        }

        if has_functions {
            // Add a type for each function
            for (_name, params, body) in func_defs {
                let param_types = vec![wasm_encoder::ValType::I32; params.len()];

                // Determine return type: check for explicit return OR implicit value
                let returns_value =
                    utils::has_return_with_value(body) || self.expression_produces_value(body);
                let return_types = if returns_value {
                    vec![wasm_encoder::ValType::I32]
                } else {
                    vec![] // Void function
                };

                types.function(param_types, return_types);
            }
            // Also add a type for the main function if there's non-function code
            if let Some(main_expr) = self.get_non_function_code(expr) {
                self.add_main_type(&mut types, &main_expr);
            }
        } else {
            // Single implicit main function
            self.add_main_type(&mut types, expr);
        }
        types
    }

    /// Add main function type based on expression
    /// Complexity: 3 (Toyota Way: <10 ✓)
    fn add_main_type(&self, types: &mut TypeSection, expr: &Expr) {
        if self.expression_produces_value(expr) {
            let wasm_ty = self.wasm_type_to_valtype(self.infer_type(expr));
            types.function(vec![], vec![wasm_ty]);
        } else {
            types.function(vec![], vec![]);
        }
    }

    /// Convert `WasmType` to `wasm_encoder::ValType`
    /// Complexity: 1 (Toyota Way: <10 ✓)
    pub(crate) fn wasm_type_to_valtype(&self, ty: WasmType) -> wasm_encoder::ValType {
        match ty {
            WasmType::I32 => wasm_encoder::ValType::I32,
            WasmType::F32 => wasm_encoder::ValType::F32,
            WasmType::I64 => wasm_encoder::ValType::I64,
            WasmType::F64 => wasm_encoder::ValType::F64,
        }
    }

    /// Emit import section for built-in functions
    /// Complexity: 3 (Toyota Way: <10 ✓)
    fn emit_import_section(&self, expr: &Expr) -> Option<wasm_encoder::ImportSection> {
        // Check if expression uses any built-in functions
        if !utils::uses_builtins(expr) {
            return None;
        }

        let mut imports = wasm_encoder::ImportSection::new();

        // Import println_i32 and println_f32 from host environment
        // Function index 0: println_i32 uses type index 0: (i32) -> ()
        // Function index 1: println_f32 uses type index 1: (f32) -> ()
        imports.import("env", "println_i32", wasm_encoder::EntityType::Function(0));
        imports.import("env", "println_f32", wasm_encoder::EntityType::Function(1));

        Some(imports)
    }

    /// Emit function section
    /// Complexity: 6 (Toyota Way: <10 ✓)
    fn emit_function_section(
        &self,
        func_defs: &[(String, Vec<crate::frontend::ast::Param>, Box<Expr>)],
        expr: &Expr,
    ) -> FunctionSection {
        let mut functions = FunctionSection::new();
        let has_functions = !func_defs.is_empty();

        // Type index offset: if we have built-ins, they occupy type indices 0 and 1
        let type_offset = if utils::uses_builtins(expr) { 2 } else { 0 };

        if has_functions {
            for i in 0..func_defs.len() {
                functions.function((i as u32) + type_offset);
            }
            if self.get_non_function_code(expr).is_some() {
                functions.function((func_defs.len() as u32) + type_offset);
            }
        } else {
            functions.function(type_offset);
        }
        functions
    }

    /// Emit memory section if needed
    /// Complexity: 2 (Toyota Way: <10 ✓)
    fn emit_memory_section(&self, expr: &Expr) -> Option<MemorySection> {
        if utils::needs_memory(expr) {
            let mut memories = MemorySection::new();
            memories.memory(MemoryType {
                minimum: 1,
                maximum: Some(1), // Fixed 64KB for MVP
                memory64: false,
                shared: false,
                page_size_log2: None,
            });
            Some(memories)
        } else {
            None
        }
    }

    /// Emit global section for heap pointer
    /// Complexity: 3 (Toyota Way: <10 ✓)
    ///
    /// Creates a mutable global `$heap_ptr` initialized to 0
    /// This is used by the bump allocator for memory allocation
    fn emit_global_section(&self, expr: &Expr) -> Option<GlobalSection> {
        if utils::needs_memory(expr) {
            let mut globals = GlobalSection::new();
            // Global 0: heap pointer (mutable i32, starts at 0)
            globals.global(
                GlobalType {
                    val_type: ValType::I32,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i32_const(0),
            );
            Some(globals)
        } else {
            None
        }
    }

    /// Emit export section if needed
    /// Complexity: 2 (Toyota Way: <10 ✓)
    fn emit_export_section(&self, expr: &Expr) -> Option<ExportSection> {
        if utils::has_main_function(expr) {
            let mut exports = ExportSection::new();
            exports.export("main", wasm_encoder::ExportKind::Func, 0);
            Some(exports)
        } else {
            None
        }
    }

    /// Emit code section with compiled functions
    /// Complexity: 8 (Toyota Way: <10 ✓)
    fn emit_code_section(
        &self,
        expr: &Expr,
        func_defs: &[(String, Vec<crate::frontend::ast::Param>, Box<Expr>)],
    ) -> Result<CodeSection, String> {
        let mut codes = CodeSection::new();
        let has_functions = !func_defs.is_empty();

        if has_functions {
            for (_name, _params, body) in func_defs {
                let func = self.compile_function(body.as_ref())?;
                codes.function(&func);
            }
            if let Some(main_expr) = self.get_non_function_code(expr) {
                let func = self.compile_function(&main_expr)?;
                codes.function(&func);
            }
        } else {
            let func = self.compile_function(expr)?;
            codes.function(&func);
        }
        Ok(codes)
    }

    /// Compile a single function body
    /// Complexity: 6 (Toyota Way: <10 ✓)
    fn compile_function(&self, body: &Expr) -> Result<Function, String> {
        let locals = self.collect_local_types(body);
        let mut func = Function::new(locals);
        let instructions = self.lower_expression(body)?;
        for instr in instructions {
            func.instruction(&instr);
        }

        // If expression produces value but function is void, drop it
        if self.expression_produces_value(body) {
            // Value producing expressions leave result on stack
            // If function should return void, we need to drop it
            // But we don't know here if this is a void function...
            // This is handled by type section already
        }

        func.instruction(&Instruction::End);
        Ok(func)
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
            ExprKind::LetPattern { pattern, body, .. } => {
                // Register all identifiers in the pattern
                self.register_pattern_symbols(pattern);
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
            ExprKind::While {
                condition, body, ..
            } => {
                self.build_symbol_table(condition);
                self.build_symbol_table(body);
            }
            ExprKind::Match { expr, arms } => {
                self.build_symbol_table(expr);
                // Note: Pattern variables in match arms are NOT registered as locals in MVP
                // This is because match arms have different scopes and WASM doesn't support
                // variable shadowing easily. For MVP, match patterns with bindings will fail
                // during lowering. Full implementation requires scoped locals.
                for arm in arms {
                    // Only recurse into body, don't register pattern variables
                    self.build_symbol_table(&arm.body);
                }
            }
            _ => {} // Other expression types don't introduce bindings
        }
    }

    /// Collect all local variable types from expression tree
    /// Returns vector of (count, type) for WASM function locals section
    /// Complexity: 9 (within <10 limit)
    fn collect_local_types(&self, expr: &Expr) -> Vec<(u32, wasm_encoder::ValType)> {
        let symbols = self.symbols.borrow();
        let local_count = symbols.local_count();
        let needs_temp = utils::needs_memory(expr); // Need temp local for tuple allocation

        if local_count == 0 && !needs_temp {
            return vec![];
        }

        // Collect all unique (type, index) pairs
        let mut locals: Vec<(WasmType, u32)> = symbols.all_locals();

        // Sort by index
        locals.sort_by_key(|(_, index)| *index);

        // Convert to (count, ValType) format
        // For now, just declare each local individually
        let mut result: Vec<(u32, wasm_encoder::ValType)> = locals
            .into_iter()
            .map(|(ty, _)| {
                let val_type = match ty {
                    WasmType::I32 => wasm_encoder::ValType::I32,
                    WasmType::F32 => wasm_encoder::ValType::F32,
                    WasmType::I64 => wasm_encoder::ValType::I64,
                    WasmType::F64 => wasm_encoder::ValType::F64,
                };
                (1, val_type)
            })
            .collect();

        // Add temporary local for tuple/struct allocation if needed
        if needs_temp {
            result.push((1, wasm_encoder::ValType::I32));
        }

        result
    }

    /// Register all identifiers in a pattern to the symbol table
    /// Complexity: 5 (Toyota Way: <10 ✓)
    fn register_pattern_symbols(&self, pattern: &Pattern) {
        match pattern {
            Pattern::Identifier(name) => {
                // Register as i32 type (default for MVP)
                self.symbols
                    .borrow_mut()
                    .insert(name.clone(), WasmType::I32);
            }
            Pattern::Tuple(patterns) => {
                // Recursively register all identifiers in tuple elements
                for p in patterns {
                    self.register_pattern_symbols(p);
                }
            }
            Pattern::Wildcard => {
                // Wildcard doesn't bind any variable
            }
            _ => {
                // Other patterns not yet supported in WASM
            }
        }
    }

    /// Infer the WASM type of an expression
    /// Complexity: 7 (Toyota Way: <10 ✓)
    fn infer_type(&self, expr: &Expr) -> WasmType {
        match &expr.kind {
            ExprKind::Literal(Literal::Integer(_, _)) => WasmType::I32,
            ExprKind::Literal(Literal::Float(_)) => WasmType::F32,
            ExprKind::Literal(Literal::Bool(_)) => WasmType::I32,
            ExprKind::Binary { op, left, right } => self.infer_binary_type(op, left, right),
            ExprKind::Let { body, .. } => self.infer_let_type(body),
            ExprKind::Identifier(name) => self.infer_identifier_type(name),
            ExprKind::Block(exprs) => exprs.last().map_or(WasmType::I32, |e| self.infer_type(e)),
            ExprKind::Call { .. } => WasmType::I32,
            ExprKind::Unary { operand, .. } => self.infer_type(operand),
            ExprKind::FieldAccess { object, field } => {
                // For tuple field access, look up the element type
                if let Ok(index) = field.parse::<usize>() {
                    if let ExprKind::Identifier(name) = &object.kind {
                        return self
                            .tuple_types
                            .borrow()
                            .get(name)
                            .and_then(|types| types.get(index))
                            .copied()
                            .unwrap_or(WasmType::I32);
                    }
                }
                WasmType::I32
            }
            _ => WasmType::I32,
        }
    }

    /// Convert `WasmType` to `wasm_encoder::ValType`
    /// Complexity: 1 (Toyota Way: <10 ✓)
    fn infer_wasm_type(&self, expr: &Expr) -> wasm_encoder::ValType {
        match self.infer_type(expr) {
            WasmType::I32 => wasm_encoder::ValType::I32,
            WasmType::I64 => wasm_encoder::ValType::I64,
            WasmType::F32 => wasm_encoder::ValType::F32,
            WasmType::F64 => wasm_encoder::ValType::F64,
        }
    }

    /// Infer type for binary expression
    /// Complexity: 3 (Toyota Way: <10 ✓)
    fn infer_binary_type(&self, op: &BinaryOp, left: &Expr, right: &Expr) -> WasmType {
        use crate::frontend::ast::BinaryOp;
        if matches!(
            op,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::Gt
        ) {
            return WasmType::I32;
        }
        // Arithmetic: Type promotion (f32 > i32)
        let left_ty = self.infer_type(left);
        let right_ty = self.infer_type(right);
        if left_ty == WasmType::F32 || right_ty == WasmType::F32 {
            WasmType::F32
        } else {
            WasmType::I32
        }
    }

    /// Infer type for let expression
    /// Complexity: 2 (Toyota Way: <10 ✓)
    fn infer_let_type(&self, body: &Expr) -> WasmType {
        match &body.kind {
            ExprKind::Literal(Literal::Unit) => WasmType::I32,
            _ => self.infer_type(body),
        }
    }

    /// Infer type for identifier
    /// Complexity: 1 (Toyota Way: <10 ✓)
    fn infer_identifier_type(&self, name: &str) -> WasmType {
        self.symbols
            .borrow()
            .lookup_type(name)
            .unwrap_or(WasmType::I32)
    }

    /// Lower a binary operation to WASM instructions
    /// Complexity: 8 (Toyota Way: <10 ✓)
    fn lower_binary(
        &self,
        op: &BinaryOp,
        left: &Expr,
        right: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Infer result type based on operands
        let result_type = self.infer_binary_result_type(left, right);

        // Emit left operand with type conversion
        instructions.extend(self.lower_expression(left)?);
        if self.infer_type(left) == WasmType::I32 && result_type == WasmType::F32 {
            instructions.push(Instruction::F32ConvertI32S);
        }

        // Emit right operand with type conversion
        instructions.extend(self.lower_expression(right)?);
        if self.infer_type(right) == WasmType::I32 && result_type == WasmType::F32 {
            instructions.push(Instruction::F32ConvertI32S);
        }

        // Emit operation instruction
        let op_instr = self.binary_op_to_instruction(op, result_type)?;
        instructions.push(op_instr);
        Ok(instructions)
    }

    /// Infer result type for binary operation
    /// Complexity: 2 (Toyota Way: <10 ✓)
    fn infer_binary_result_type(&self, left: &Expr, right: &Expr) -> WasmType {
        let left_ty = self.infer_type(left);
        let right_ty = self.infer_type(right);
        if left_ty == WasmType::F32 || right_ty == WasmType::F32 {
            WasmType::F32
        } else {
            WasmType::I32
        }
    }

    /// Map binary operation to WASM instruction
    /// Complexity: 1 (Toyota Way: <10 ✓)
    fn binary_op_to_instruction(
        &self,
        op: &BinaryOp,
        ty: WasmType,
    ) -> Result<Instruction<'static>, String> {
        match (op, ty) {
            (BinaryOp::Add, WasmType::I32) => Ok(Instruction::I32Add),
            (BinaryOp::Add, WasmType::F32) => Ok(Instruction::F32Add),
            (BinaryOp::Subtract, WasmType::I32) => Ok(Instruction::I32Sub),
            (BinaryOp::Subtract, WasmType::F32) => Ok(Instruction::F32Sub),
            (BinaryOp::Multiply, WasmType::I32) => Ok(Instruction::I32Mul),
            (BinaryOp::Multiply, WasmType::F32) => Ok(Instruction::F32Mul),
            (BinaryOp::Divide, WasmType::I32) => Ok(Instruction::I32DivS),
            (BinaryOp::Divide, WasmType::F32) => Ok(Instruction::F32Div),
            (BinaryOp::Modulo, WasmType::I32) => Ok(Instruction::I32RemS),
            (BinaryOp::Modulo, WasmType::F32) => Err("Modulo not supported for floats".to_string()),
            (BinaryOp::Equal, WasmType::I32) => Ok(Instruction::I32Eq),
            (BinaryOp::Equal, WasmType::F32) => Ok(Instruction::F32Eq),
            (BinaryOp::NotEqual, WasmType::I32) => Ok(Instruction::I32Ne),
            (BinaryOp::NotEqual, WasmType::F32) => Ok(Instruction::F32Ne),
            (BinaryOp::Less, WasmType::I32) => Ok(Instruction::I32LtS),
            (BinaryOp::Less, WasmType::F32) => Ok(Instruction::F32Lt),
            (BinaryOp::Greater, WasmType::I32) => Ok(Instruction::I32GtS),
            (BinaryOp::Greater, WasmType::F32) => Ok(Instruction::F32Gt),
            (BinaryOp::LessEqual, WasmType::I32) => Ok(Instruction::I32LeS),
            (BinaryOp::LessEqual, WasmType::F32) => Ok(Instruction::F32Le),
            (BinaryOp::GreaterEqual, WasmType::I32) => Ok(Instruction::I32GeS),
            (BinaryOp::GreaterEqual, WasmType::F32) => Ok(Instruction::F32Ge),
            (BinaryOp::And, _) => Ok(Instruction::I32And),
            (BinaryOp::Or, _) => Ok(Instruction::I32Or),
            _ => Err(format!("Unsupported binary operation: {op:?}")),
        }
    }

    /// Lower an if expression to WASM instructions
    /// Complexity: 4 (Toyota Way: <10 ✓)
    fn lower_if(
        &self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<Vec<Instruction<'static>>, String> {
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

    /// Lower a unary operation to WASM instructions
    /// Complexity: 6 (Toyota Way: <10 ✓)
    fn lower_unary(
        &self,
        op: &crate::frontend::ast::UnaryOp,
        operand: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Emit operand
        instructions.extend(self.lower_expression(operand)?);

        // Emit unary operation
        match op {
            crate::frontend::ast::UnaryOp::Negate => {
                self.emit_negate(&mut instructions, operand);
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
            crate::frontend::ast::UnaryOp::Reference
            | crate::frontend::ast::UnaryOp::MutableReference
            | crate::frontend::ast::UnaryOp::Deref => {
                // Reference/dereference operators not supported in WASM (needs memory)
                // Keep operand value on stack (PARSER-085: Issue #71)
            }
        }
        Ok(instructions)
    }

    /// Emit type-aware negation instruction
    /// Complexity: 1 (Toyota Way: <10 ✓)
    fn emit_negate(&self, instructions: &mut Vec<Instruction<'static>>, operand: &Expr) {
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

    /// Lower a block expression to WASM instructions
    /// Complexity: 3 (Toyota Way: <10 ✓)
    fn lower_block(&self, exprs: &[Expr]) -> Result<Vec<Instruction<'static>>, String> {
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

    /// Lower a Ruchy expression to WASM instructions
    /// Complexity: 4 (Toyota Way: <10 ✓)
    fn lower_expression(&self, expr: &Expr) -> Result<Vec<Instruction<'static>>, String> {
        match &expr.kind {
            ExprKind::Literal(literal) => self.lower_literal(literal),
            ExprKind::Binary { op, left, right } => self.lower_binary(op, left, right),
            ExprKind::Block(exprs) => self.lower_block(exprs),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.lower_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::While {
                condition, body, ..
            } => self.lower_while(condition, body),
            ExprKind::Function { .. } => Ok(vec![]),
            ExprKind::Lambda { .. } => Ok(vec![]), // Lambda definitions generate no instructions
            ExprKind::Call { func, args } => self.lower_call(func, args),
            ExprKind::Let {
                name, value, body, ..
            } => self.lower_let(name, value, body),
            ExprKind::LetPattern {
                pattern,
                value,
                body,
                ..
            } => self.lower_let_pattern(pattern, value, body),
            ExprKind::Identifier(name) => self.lower_identifier(name),
            ExprKind::Unary { op, operand } => self.lower_unary(op, operand),
            ExprKind::List(items) => self.lower_list(items),
            ExprKind::Return { value } => self.lower_return(value.as_deref()),
            ExprKind::StringInterpolation { parts } => self.lower_string_interpolation(parts),
            ExprKind::Match { expr, arms } => self.lower_match(expr, arms),
            ExprKind::Tuple(elements) => self.lower_tuple(elements),
            ExprKind::FieldAccess { object, field } => self.lower_field_access(object, field),
            ExprKind::StructLiteral { name, fields, .. } => self.lower_struct_literal(name, fields),
            ExprKind::IndexAccess { object, index } => self.lower_index_access(object, index),
            ExprKind::Assign { target, value } => self.lower_assign(target, value),
            _ => Ok(vec![]),
        }
    }

    /// Lower a while loop to WASM instructions
    /// Complexity: 4 (Toyota Way: <10 ✓)
    fn lower_while(
        &self,
        condition: &Expr,
        body: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];
        instructions.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
        instructions.extend(self.lower_expression(condition)?);
        instructions.push(Instruction::I32Eqz);
        instructions.push(Instruction::BrIf(1));
        instructions.extend(self.lower_expression(body)?);
        instructions.push(Instruction::Br(0));
        instructions.push(Instruction::End);
        Ok(instructions)
    }

    /// Lower a function call to WASM instructions
    /// Complexity: 7 (Toyota Way: <10 ✓)
    fn lower_call(&self, func: &Expr, args: &[Expr]) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Push arguments onto stack
        for arg in args {
            instructions.extend(self.lower_expression(arg)?);
        }

        // Determine function index
        let func_index = if let ExprKind::Identifier(name) = &func.kind {
            if matches!(name.as_str(), "println" | "print" | "eprintln" | "eprint") {
                // Built-in functions: choose based on argument type
                // Function index 0: println_i32 - (i32) -> ()
                // Function index 1: println_f32 - (f32) -> ()
                if let Some(first_arg) = args.first() {
                    let arg_type = self.infer_type(first_arg);
                    match arg_type {
                        WasmType::F32 => 1, // Use println_f32
                        _ => 0,             // Use println_i32 (default)
                    }
                } else {
                    0 // No args, use i32 version
                }
            } else {
                // Look up user-defined function index (extract index from tuple)
                self.functions
                    .borrow()
                    .get(name)
                    .map(|&(idx, _)| idx)
                    .ok_or_else(|| format!("Unknown function: {name}"))?
            }
        } else {
            return Err("Function calls must use identifiers".to_string());
        };

        instructions.push(Instruction::Call(func_index));
        Ok(instructions)
    }

    /// Lower a let binding to WASM instructions
    /// Complexity: 4 (Toyota Way: <10 ✓)
    fn lower_let(
        &self,
        name: &str,
        value: &Expr,
        body: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];
        instructions.extend(self.lower_expression(value)?);
        let local_index = self.symbols.borrow().lookup_index(name).unwrap_or(0);
        instructions.push(Instruction::LocalSet(local_index));

        // Track tuple element types for mixed-type support
        if let ExprKind::Tuple(elements) = &value.kind {
            let element_types: Vec<WasmType> = elements
                .iter()
                .map(|e| self.infer_element_type(e))
                .collect();
            self.tuple_types
                .borrow_mut()
                .insert(name.to_string(), element_types);
        }

        if !matches!(&body.kind, ExprKind::Literal(Literal::Unit)) {
            instructions.extend(self.lower_expression(body)?);
        }
        Ok(instructions)
    }

    /// Lower a let pattern binding to WASM instructions
    /// Complexity: 7 (Toyota Way: <10 ✓)
    ///
    /// For tuples: Loads each element from memory and stores to pattern variables
    /// Example: `let (x, y) = (3, 4)` loads values from tuple memory into x and y
    fn lower_let_pattern(
        &self,
        pattern: &Pattern,
        value: &Expr,
        body: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Evaluate the value expression (returns address for tuples)
        instructions.extend(self.lower_expression(value)?);

        // Store to all identifiers in the pattern
        // For tuples: loads from memory at address + offset
        self.store_pattern_values(pattern, &mut instructions)?;

        // Evaluate the body
        if !matches!(&body.kind, ExprKind::Literal(Literal::Unit)) {
            instructions.extend(self.lower_expression(body)?);
        }

        Ok(instructions)
    }

    /// Store value on stack to all identifiers in pattern
    /// Complexity: 9 (Toyota Way: <10 ✓)
    ///
    /// For tuple patterns, loads each element from memory and stores to locals
    fn store_pattern_values(
        &self,
        pattern: &Pattern,
        instructions: &mut Vec<Instruction<'static>>,
    ) -> Result<(), String> {
        match pattern {
            Pattern::Identifier(name) => {
                let local_index = self.symbols.borrow().lookup_index(name).unwrap_or(0);
                instructions.push(Instruction::LocalSet(local_index));
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                // Real implementation: Load each tuple element from memory
                // Value on stack is the tuple address

                // Use temp local to save the tuple address
                let temp_local = self.symbols.borrow().local_count();
                instructions.push(Instruction::LocalSet(temp_local));

                // Load each tuple element and store to pattern variable
                for (i, p) in patterns.iter().enumerate() {
                    // Get tuple address
                    instructions.push(Instruction::LocalGet(temp_local));

                    // Load element at offset (index * 4 bytes)
                    let offset = i as i32 * 4;
                    instructions.push(Instruction::I32Load(wasm_encoder::MemArg {
                        offset: offset as u64,
                        align: 2, // 4-byte alignment
                        memory_index: 0,
                    }));

                    // Store to pattern variable
                    self.store_pattern_values(p, instructions)?;
                }
                Ok(())
            }
            Pattern::Wildcard => {
                // Wildcard: pop value from stack without storing
                instructions.push(Instruction::Drop);
                Ok(())
            }
            _ => Err(format!("Pattern {pattern:?} not yet supported in WASM")),
        }
    }

    /// Lower an identifier to WASM instructions
    /// Complexity: 1 (Toyota Way: <10 ✓)
    fn lower_identifier(&self, name: &str) -> Result<Vec<Instruction<'static>>, String> {
        let local_index = self.symbols.borrow().lookup_index(name).unwrap_or(0);
        Ok(vec![Instruction::LocalGet(local_index)])
    }

    /// Lower a list/array literal to WASM instructions
    /// Complexity: 9 (Toyota Way: <10 ✓)
    ///
    /// Allocates memory for array elements and stores them sequentially
    /// Returns the address of the array in memory
    ///
    /// Memory layout: Each element is 4 bytes (i32), same as tuples
    /// Example: [1, 2, 3] -> [addr+0: 1, addr+4: 2, addr+8: 3]
    fn lower_list(&self, elements: &[Expr]) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Empty array: return placeholder 0 (no allocation needed)
        if elements.is_empty() {
            return Ok(vec![Instruction::I32Const(0)]);
        }

        // Calculate size needed: 4 bytes per element (all i32 for MVP)
        let size = elements.len() as i32 * 4;

        // Temporary local index: last local (reserved in collect_local_types)
        let temp_local = self.symbols.borrow().local_count();

        // Inline malloc: allocate memory using bump allocator
        // 1. Get current heap pointer (global 0) and save it
        instructions.push(Instruction::GlobalGet(0));
        instructions.push(Instruction::LocalSet(temp_local));

        // 2. Update heap pointer: old_ptr + size
        instructions.push(Instruction::GlobalGet(0));
        instructions.push(Instruction::I32Const(size));
        instructions.push(Instruction::I32Add);
        instructions.push(Instruction::GlobalSet(0));

        // 3. Store each array element in memory
        for (i, element) in elements.iter().enumerate() {
            let offset = i as i32 * 4;

            // Get the base address
            instructions.push(Instruction::LocalGet(temp_local));

            // Evaluate the element value
            instructions.extend(self.lower_expression(element)?);

            // Store at address + offset
            instructions.push(Instruction::I32Store(wasm_encoder::MemArg {
                offset: offset as u64,
                align: 2, // 4-byte alignment (2^2 = 4)
                memory_index: 0,
            }));
        }

        // 4. Return the base address
        instructions.push(Instruction::LocalGet(temp_local));

        Ok(instructions)
    }

    /// Lower a tuple literal to WASM instructions
    /// Complexity: 9 (Toyota Way: <10 ✓)
    ///
    /// Allocates memory for tuple elements and stores them sequentially
    /// Returns the address of the tuple in memory
    ///
    /// Memory layout: Each element is 4 bytes (i32)
    /// Example: (3, 4) -> [addr+0: 3, addr+4: 4]
    fn lower_tuple(&self, elements: &[Expr]) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Empty tuple: return placeholder 0 (no allocation needed)
        if elements.is_empty() {
            return Ok(vec![Instruction::I32Const(0)]);
        }

        // Calculate size needed: 4 bytes per element (all i32 for MVP)
        let size = elements.len() as i32 * 4;

        // Temporary local index: last local (reserved in collect_local_types)
        let temp_local = self.symbols.borrow().local_count();

        // Inline malloc: allocate memory using bump allocator
        // 1. Get current heap pointer (global 0) and save it
        instructions.push(Instruction::GlobalGet(0));
        instructions.push(Instruction::LocalSet(temp_local));

        // 2. Update heap pointer: old_ptr + size
        instructions.push(Instruction::GlobalGet(0));
        instructions.push(Instruction::I32Const(size));
        instructions.push(Instruction::I32Add);
        instructions.push(Instruction::GlobalSet(0));

        // 3. Store each tuple element in memory (with correct type)
        let element_types: Vec<WasmType> = elements
            .iter()
            .map(|e| self.infer_element_type(e))
            .collect();

        for (i, element) in elements.iter().enumerate() {
            let offset = i as i32 * 4;
            let elem_type = element_types[i];

            // Get the base address
            instructions.push(Instruction::LocalGet(temp_local));

            // Evaluate the element value
            instructions.extend(self.lower_expression(element)?);

            // Store at address + offset (use correct store instruction per type)
            match elem_type {
                WasmType::F32 => {
                    instructions.push(Instruction::F32Store(wasm_encoder::MemArg {
                        offset: offset as u64,
                        align: 2, // 4-byte alignment (2^2 = 4)
                        memory_index: 0,
                    }));
                }
                _ => {
                    instructions.push(Instruction::I32Store(wasm_encoder::MemArg {
                        offset: offset as u64,
                        align: 2, // 4-byte alignment (2^2 = 4)
                        memory_index: 0,
                    }));
                }
            }
        }

        // 4. Return the base address
        instructions.push(Instruction::LocalGet(temp_local));

        Ok(instructions)
    }

    /// Lower field access to WASM instructions
    /// Complexity: 9 (Toyota Way: <10 ✓)
    ///
    /// Loads field value from memory
    /// For tuples: field is numeric index ("0", "1", "2", ...)
    /// For structs: looks up field offset from struct registry
    fn lower_field_access(
        &self,
        object: &Expr,
        field: &str,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Evaluate object expression (should return address for tuples/structs)
        instructions.extend(self.lower_expression(object)?);

        // Determine field type and offset
        let (offset, field_type) = if let Ok(index) = field.parse::<i32>() {
            // Tuple field access: "0", "1", "2", etc.
            let offset = index * 4; // Each element is 4 bytes

            // Look up tuple element type if object is an identifier
            let elem_type = if let ExprKind::Identifier(name) = &object.kind {
                self.tuple_types
                    .borrow()
                    .get(name)
                    .and_then(|types| types.get(index as usize))
                    .copied()
                    .unwrap_or(WasmType::I32)
            } else {
                WasmType::I32 // Default for non-identifier tuples
            };

            (offset, elem_type)
        } else {
            // Struct field access: look up field in struct registry
            let structs = self.structs.borrow();
            let field_index = structs
                .values()
                .find_map(|fields| fields.iter().position(|f| f == field));

            match field_index {
                Some(index) => (index as i32 * 4, WasmType::I32),
                None => (0, WasmType::I32), // Field not found - use 0 as fallback
            }
        };

        // Load value from memory using correct instruction for field type
        match field_type {
            WasmType::F32 => {
                instructions.push(Instruction::F32Load(wasm_encoder::MemArg {
                    offset: offset as u64,
                    align: 2, // 4-byte alignment (2^2 = 4)
                    memory_index: 0,
                }));
            }
            _ => {
                instructions.push(Instruction::I32Load(wasm_encoder::MemArg {
                    offset: offset as u64,
                    align: 2, // 4-byte alignment (2^2 = 4)
                    memory_index: 0,
                }));
            }
        }

        Ok(instructions)
    }

    /// Lower struct literal to WASM instructions
    /// Complexity: 10 (Toyota Way: ≤10 ✓)
    ///
    /// Allocates memory for struct fields and stores them in field order
    /// Returns the address of the struct in memory
    ///
    /// Memory layout: Fields stored in definition order, 4 bytes per i32 field
    /// Example: Point { x: 3, y: 4 } with fields [x, y] → [addr+0: 3, addr+4: 4]
    fn lower_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // Look up struct definition to get field order
        let field_order = self.structs.borrow().get(name).cloned();
        let field_order = match field_order {
            Some(order) => order,
            None => {
                // Struct not defined - return placeholder for now
                // This can happen if struct is defined in external module
                return Ok(vec![Instruction::I32Const(0)]);
            }
        };

        // Calculate size needed: 4 bytes per field (all i32 for MVP)
        let size = field_order.len() as i32 * 4;

        // Temporary local index: last local (reserved in collect_local_types)
        let temp_local = self.symbols.borrow().local_count();

        // Inline malloc: allocate memory using bump allocator
        // 1. Get current heap pointer (global 0) and save it
        instructions.push(Instruction::GlobalGet(0));
        instructions.push(Instruction::LocalSet(temp_local));

        // 2. Update heap pointer: old_ptr + size
        instructions.push(Instruction::GlobalGet(0));
        instructions.push(Instruction::I32Const(size));
        instructions.push(Instruction::I32Add);
        instructions.push(Instruction::GlobalSet(0));

        // 3. Store each field at correct offset based on field order
        for (field_name, field_value) in fields {
            // Find field index in definition order
            let field_index = field_order
                .iter()
                .position(|f| f == field_name)
                .unwrap_or(0);
            let offset = field_index as i32 * 4;

            // Get the base address
            instructions.push(Instruction::LocalGet(temp_local));

            // Evaluate the field value
            instructions.extend(self.lower_expression(field_value)?);

            // Store at address + offset
            instructions.push(Instruction::I32Store(wasm_encoder::MemArg {
                offset: offset as u64,
                align: 2, // 4-byte alignment (2^2 = 4)
                memory_index: 0,
            }));
        }

        // 4. Return the base address
        instructions.push(Instruction::LocalGet(temp_local));

        Ok(instructions)
    }

    /// Lower index access to WASM instructions
    /// Complexity: 6 (Toyota Way: <10 ✓)
    ///
    /// Loads array/tuple element from memory using dynamic index
    /// Computes offset at runtime: `base_address` + (index * 4)
    fn lower_index_access(
        &self,
        object: &Expr,
        index: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        // 1. Evaluate object to get base address
        instructions.extend(self.lower_expression(object)?);

        // 2. Evaluate index (runtime value)
        instructions.extend(self.lower_expression(index)?);

        // 3. Compute offset: index * 4 (each element is 4 bytes)
        instructions.push(Instruction::I32Const(4));
        instructions.push(Instruction::I32Mul);

        // 4. Add base address + offset
        instructions.push(Instruction::I32Add);

        // 5. Load value from computed address
        instructions.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0, // Offset already computed, no additional offset needed
            align: 2,  // 4-byte alignment (2^2 = 4)
            memory_index: 0,
        }));

        Ok(instructions)
    }

    /// Lower assignment expression to WASM instructions
    /// Complexity: 10 (Toyota Way: ≤10 ✓)
    ///
    /// Supports identifiers, field access, and index access
    /// Field mutations now work with real memory stores
    fn lower_assign(
        &self,
        target: &Expr,
        value: &Expr,
    ) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];

        match &target.kind {
            ExprKind::Identifier(name) => {
                // Standard local variable assignment
                instructions.extend(self.lower_expression(value)?);
                let local_index = self.symbols.borrow().lookup_index(name).unwrap_or(0);
                instructions.push(Instruction::LocalSet(local_index));
            }
            ExprKind::FieldAccess { object, field } => {
                // Field mutation: store value to memory at address + offset
                // 1. Evaluate object to get base address
                instructions.extend(self.lower_expression(object)?);

                // 2. Evaluate value
                instructions.extend(self.lower_expression(value)?);

                // 3. Calculate field offset (same logic as lower_field_access)
                let offset = if let Ok(index) = field.parse::<i32>() {
                    index * 4 // Tuple field
                } else {
                    // Struct field: look up in registry
                    let structs = self.structs.borrow();
                    let field_index = structs
                        .values()
                        .find_map(|fields| fields.iter().position(|f| f == field));
                    match field_index {
                        Some(index) => index as i32 * 4,
                        None => 0,
                    }
                };

                // 4. Store value at address + offset
                instructions.push(Instruction::I32Store(wasm_encoder::MemArg {
                    offset: offset as u64,
                    align: 2, // 4-byte alignment
                    memory_index: 0,
                }));
            }
            ExprKind::IndexAccess { object, index } => {
                // Array element mutation: store value to memory at address + dynamic offset
                // 1. Evaluate object to get base address
                instructions.extend(self.lower_expression(object)?);

                // 2. Evaluate index (runtime value)
                instructions.extend(self.lower_expression(index)?);

                // 3. Compute offset: index * 4
                instructions.push(Instruction::I32Const(4));
                instructions.push(Instruction::I32Mul);

                // 4. Add base + offset to get final address
                instructions.push(Instruction::I32Add);

                // 5. Evaluate value
                instructions.extend(self.lower_expression(value)?);

                // 6. Store value at computed address
                instructions.push(Instruction::I32Store(wasm_encoder::MemArg {
                    offset: 0, // Offset already computed
                    align: 2,  // 4-byte alignment
                    memory_index: 0,
                }));
            }
            _ => {
                return Err(format!(
                    "Assignment target {:?} not supported in WASM",
                    target.kind
                ))
            }
        }

        Ok(instructions)
    }

    /// Lower a return statement to WASM instructions
    /// Complexity: 2 (Toyota Way: <10 ✓)
    fn lower_return(&self, value: Option<&Expr>) -> Result<Vec<Instruction<'static>>, String> {
        let mut instructions = vec![];
        if let Some(val) = value {
            instructions.extend(self.lower_expression(val)?);
        }
        instructions.push(Instruction::Return);
        Ok(instructions)
    }

    /// Lower string interpolation to WASM instructions
    /// Complexity: 8 (Toyota Way: <10 ✓)
    ///
    /// Current implementation: MVP string interpolation support
    /// - Text-only f-strings: concatenated into single string literal
    /// - F-strings with expressions: evaluated and represented as i32 (memory pointer)
    ///
    /// WASM strings are represented as i32 pointers to linear memory.
    /// Full string concatenation requires host function support (`string_concat`).
    /// This is implemented in stages per `docs/specifications/wasm-fstring-spec.md`
    fn lower_string_interpolation(
        &self,
        parts: &[StringPart],
    ) -> Result<Vec<Instruction<'static>>, String> {
        // Stage 1: Text-only f-strings
        let all_text = parts.iter().all(|p| matches!(p, StringPart::Text(_)));

        if all_text {
            // Concatenate all text parts into a single string literal
            let text: String = parts
                .iter()
                .filter_map(|p| {
                    if let StringPart::Text(s) = p {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            // Lower as a string literal (i32 memory pointer)
            self.lower_literal(&Literal::String(text))
        } else {
            // Stage 2: F-strings with expressions
            // Strategy: For single-expression f-strings, evaluate and return the expression value
            // Multi-part f-strings require host function support for concatenation

            // Check if this is a simple case: single expression, no text
            if parts.len() == 1 {
                if let StringPart::Expr(expr) = &parts[0] {
                    // Single expression: just evaluate it
                    return self.lower_expression(expr);
                }
            }

            // For multi-part f-strings: requires host function support
            // For now, evaluate first expression if present (partial fix)
            for part in parts {
                if let StringPart::Expr(expr) | StringPart::ExprWithFormat { expr, .. } = part {
                    return self.lower_expression(expr);
                }
            }

            // No expressions found: return placeholder
            Ok(vec![Instruction::I32Const(0)])
        }
    }

    /// Lower match expression to WASM instructions
    /// Complexity: 9 (Toyota Way: <10 ✓)
    ///
    /// Strategy: Desugar match into cascading if-else expressions
    /// Reference: `docs/specifications/wasm-match-spec.md`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match x { 1 => 10, 2 => 20, _ => 0 }
    /// // Becomes:
    /// if x == 1 { 10 } else if x == 2 { 20 } else { 0 }
    /// ```
    fn lower_match(
        &self,
        match_expr: &Expr,
        arms: &[crate::frontend::ast::MatchArm],
    ) -> Result<Vec<Instruction<'static>>, String> {
        if arms.is_empty() {
            return Ok(vec![Instruction::I32Const(0)]);
        }

        // Build cascading if-else from match arms
        // Start from the last arm (usually wildcard) and work backwards
        let mut result_instructions = vec![];

        // Process arms in reverse to build nested if-else
        for (i, arm) in arms.iter().enumerate().rev() {
            let is_last = i == arms.len() - 1;

            match &arm.pattern {
                Pattern::Wildcard => {
                    // Wildcard: just emit the body
                    result_instructions = self.lower_expression(&arm.body)?;
                }
                Pattern::Literal(lit) => {
                    if is_last {
                        // Last arm without wildcard: just emit body
                        result_instructions = self.lower_expression(&arm.body)?;
                    } else {
                        // Compare match_expr with pattern literal
                        let mut instr = vec![];
                        instr.extend(self.lower_expression(match_expr)?);
                        instr.extend(self.lower_literal(lit)?);
                        instr.push(Instruction::I32Eq);

                        // if (condition) { arm.body } else { rest }
                        let then_body = self.lower_expression(&arm.body)?;
                        let else_body = result_instructions;

                        // Determine result type from body
                        let result_type = self.infer_wasm_type(&arm.body);

                        instr.push(Instruction::If(wasm_encoder::BlockType::Result(
                            result_type,
                        )));
                        instr.extend(then_body);
                        instr.push(Instruction::Else);
                        instr.extend(else_body);
                        instr.push(Instruction::End);

                        result_instructions = instr;
                    }
                }
                Pattern::Or(patterns) => {
                    if is_last {
                        // Last arm: just emit body
                        result_instructions = self.lower_expression(&arm.body)?;
                    } else {
                        // OR pattern: match_expr == pat1 || match_expr == pat2 || ...
                        let mut instr = vec![];

                        // Build comparison for each pattern in OR
                        for (j, pattern) in patterns.iter().enumerate() {
                            // Compare match_expr with this pattern
                            instr.extend(self.lower_expression(match_expr)?);
                            if let Pattern::Literal(lit) = pattern {
                                instr.extend(self.lower_literal(lit)?);
                                instr.push(Instruction::I32Eq);
                            } else {
                                return Err(format!(
                                    "Non-literal patterns in OR not yet supported: {pattern:?}"
                                ));
                            }

                            // OR with previous comparison (except for first)
                            if j > 0 {
                                instr.push(Instruction::I32Or);
                            }
                        }

                        // if (any match) { arm.body } else { rest }
                        let then_body = self.lower_expression(&arm.body)?;
                        let else_body = result_instructions;
                        let result_type = self.infer_wasm_type(&arm.body);

                        instr.push(Instruction::If(wasm_encoder::BlockType::Result(
                            result_type,
                        )));
                        instr.extend(then_body);
                        instr.push(Instruction::Else);
                        instr.extend(else_body);
                        instr.push(Instruction::End);

                        result_instructions = instr;
                    }
                }
                Pattern::Tuple(_patterns) => {
                    // MVP: Tuple patterns in match always succeed (placeholder)
                    // Full implementation would destructure and compare tuple elements
                    if is_last {
                        result_instructions = self.lower_expression(&arm.body)?;
                    } else {
                        // For now, treat tuple patterns as "always match" (like wildcard)
                        // This allows code to compile but doesn't do proper pattern matching
                        result_instructions = self.lower_expression(&arm.body)?;
                    }
                }
                Pattern::Identifier(_name) => {
                    // Identifier pattern: binds the value to a variable (always matches)
                    result_instructions = self.lower_expression(&arm.body)?;
                }
                _ => {
                    // Other patterns not yet supported in MVP
                    return Err(format!(
                        "Pattern {:?} not yet supported in WASM",
                        arm.pattern
                    ));
                }
            }
        }

        Ok(result_instructions)
    }

    /// Build function index map for resolving function calls
    /// Complexity: 3 (Toyota Way: <10 ✓)
    ///
    /// Function indices in WASM:
    /// - Import functions come first (e.g., println at index 0 if imports exist)
    /// - User-defined functions follow
    fn build_function_index_map(
        &self,
        expr: &Expr,
        func_defs: &[(String, Vec<crate::frontend::ast::Param>, Box<Expr>)],
    ) {
        let mut index_map = std::collections::HashMap::new();

        // Calculate offset: imports come first (2 built-ins: println_i32 and println_f32)
        let import_offset = if utils::uses_builtins(expr) { 2 } else { 0 };

        // Map each user function to (index, is_void)
        for (i, (name, _, body)) in func_defs.iter().enumerate() {
            let index = (i as u32) + import_offset;
            let returns_value =
                utils::has_return_with_value(body) || self.expression_produces_value(body);
            let is_void = !returns_value;
            index_map.insert(name.clone(), (index, is_void));
        }

        *self.functions.borrow_mut() = index_map;
    }

    /// Collect all function definitions from the AST
    pub(crate) fn collect_functions(
        &self,
        expr: &Expr,
    ) -> Vec<(String, Vec<crate::frontend::ast::Param>, Box<Expr>)> {
        let mut functions = Vec::new();
        Self::collect_functions_rec(expr, &mut functions);
        functions
    }
    fn collect_functions_rec(
        expr: &Expr,
        functions: &mut Vec<(String, Vec<crate::frontend::ast::Param>, Box<Expr>)>,
    ) {
        match &expr.kind {
            ExprKind::Function {
                name, params, body, ..
            } => {
                functions.push((name.clone(), params.clone(), body.clone()));
            }
            ExprKind::Let {
                name, value, body, ..
            } => {
                // Check if the value is a lambda (closure)
                if let ExprKind::Lambda {
                    params,
                    body: lambda_body,
                } = &value.kind
                {
                    functions.push((name.clone(), params.clone(), lambda_body.clone()));
                }
                // Continue recursing into the let body
                Self::collect_functions_rec(body, functions);
            }
            ExprKind::Block(exprs) => {
                for e in exprs {
                    Self::collect_functions_rec(e, functions);
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
                    .filter(|e| {
                        // Filter out function definitions
                        if matches!(e.kind, ExprKind::Function { .. }) {
                            return false;
                        }
                        // Filter out let bindings that bind lambdas
                        if let ExprKind::Let { value, .. } = &e.kind {
                            if matches!(value.kind, ExprKind::Lambda { .. }) {
                                return false;
                            }
                        }
                        true
                    })
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

    // NOTE: uses_builtins, needs_memory, has_main_function, has_return_with_value,
    // needs_locals moved to utils.rs

    /// Check if an expression produces a value on the stack
    /// Complexity: 8 (Toyota Way: <10 ✓)
    fn expression_produces_value(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Literal(_) => true,
            ExprKind::Binary { .. } => true,
            ExprKind::Unary { .. } => true,
            ExprKind::Identifier(_) => true,
            ExprKind::Call { func, .. } => {
                // Check if this is a void function
                if let ExprKind::Identifier(name) = &func.kind {
                    // Built-in void functions
                    if matches!(name.as_str(), "println" | "print" | "eprintln" | "eprint") {
                        return false;
                    }
                    // User-defined functions - check registry
                    if let Some(&(_idx, is_void)) = self.functions.borrow().get(name) {
                        return !is_void;
                    }
                    // Unknown function - assume it produces a value
                    true
                } else {
                    true
                }
            }
            ExprKind::List(_) => true,
            ExprKind::Block(exprs) => {
                // Block produces value if last expression does
                exprs
                    .last()
                    .is_some_and(|e| self.expression_produces_value(e))
            }
            ExprKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                // If produces value only if both branches produce values
                let then_produces = self.expression_produces_value(then_branch);
                let else_produces = else_branch
                    .as_ref()
                    .is_some_and(|e| self.expression_produces_value(e));
                then_produces && else_produces
            }
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
            Literal::Integer(n, _) => Ok(vec![Instruction::I32Const(*n as i32)]),
            Literal::Float(f) => Ok(vec![Instruction::F32Const(*f as f32)]),
            Literal::Bool(b) => Ok(vec![Instruction::I32Const(i32::from(*b))]),
            Literal::String(_) => {
                // String literals would need memory allocation
                // For now, return a placeholder
                Ok(vec![Instruction::I32Const(0)])
            }
            Literal::Unit => {
                // Unit type () is represented as i32 const 0 in WASM
                Ok(vec![Instruction::I32Const(0)])
            }
            Literal::Char(c) => {
                // Character literals represented as i32 (UTF-32 code point)
                Ok(vec![Instruction::I32Const(*c as i32)])
            }
            _ => Ok(vec![Instruction::I32Const(0)]), // Other literals default to 0
        }
    }
}
impl Default for WasmEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_wasm_emitter_new() {
        let emitter = WasmEmitter::new();
        // Just verify creation works
        assert!(emitter.functions.borrow().is_empty());
    }

    #[test]
    fn test_wasm_emitter_default() {
        let emitter = WasmEmitter::default();
        assert!(emitter.functions.borrow().is_empty());
    }

    #[test]
    fn test_lower_literal_integer() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::Integer(42, None));
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_literal_float() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::Float(3.14));
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_literal_bool_true() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::Bool(true));
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_literal_bool_false() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::Bool(false));
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_literal_string() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::String("test".to_string()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_literal_unit() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::Unit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower_literal_char() {
        let emitter = WasmEmitter::new();
        let result = emitter.lower_literal(&Literal::Char('A'));
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_element_type_via_parser() {
        let emitter = WasmEmitter::new();
        let mut parser = Parser::new("42");
        if let Ok(ast) = parser.parse() {
            let ty = emitter.infer_element_type(&ast);
            // Integer literals infer to I32
            let _ = ty;
        }
    }

    #[test]
    fn test_infer_element_type_float_via_parser() {
        let emitter = WasmEmitter::new();
        let mut parser = Parser::new("3.14");
        if let Ok(ast) = parser.parse() {
            let ty = emitter.infer_element_type(&ast);
            let _ = ty;
        }
    }

    #[test]
    fn test_collect_tuple_types_via_parser() {
        let emitter = WasmEmitter::new();
        let mut parser = Parser::new("let x = (1, 2.0); x");
        if let Ok(ast) = parser.parse() {
            emitter.collect_tuple_types(&ast);
        }
    }

    #[test]
    fn test_structs_registration() {
        let emitter = WasmEmitter::new();
        // Register a struct manually
        emitter
            .structs
            .borrow_mut()
            .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
        assert!(emitter.structs.borrow().contains_key("Point"));
    }

    #[test]
    fn test_functions_registration() {
        let emitter = WasmEmitter::new();
        // Register a function manually
        emitter
            .functions
            .borrow_mut()
            .insert("main".to_string(), (0, false));
        let funcs = emitter.functions.borrow();
        assert!(funcs.contains_key("main"));
        assert_eq!(funcs["main"], (0, false));
    }

    #[test]
    fn test_tuple_types_registration() {
        let emitter = WasmEmitter::new();
        emitter
            .tuple_types
            .borrow_mut()
            .insert("pair".to_string(), vec![WasmType::I32, WasmType::F32]);
        let types = emitter.tuple_types.borrow();
        assert!(types.contains_key("pair"));
        assert_eq!(types["pair"].len(), 2);
    }
}
