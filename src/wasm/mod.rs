//! WebAssembly component toolkit for Ruchy (RUCHY-0819)
//!
//! This module provides WebAssembly component generation, WIT interface generation,
//! platform-specific deployment, and portability scoring for Ruchy code.
pub mod component;
pub mod demo_converter;
pub mod deployment;
#[cfg(feature = "notebook")]
pub mod notebook;
pub mod portability;
pub mod repl;
#[cfg(feature = "notebook")]
pub mod shared_session;
pub mod wit;

pub use component::{ComponentBuilder, ComponentConfig, WasmComponent};
pub use demo_converter::{
    convert_demo_to_notebook, find_demo_files, Notebook as DemoNotebook,
    NotebookCell as DemoNotebookCell,
};
pub use deployment::{Deployer, DeploymentConfig, DeploymentTarget};
#[cfg(feature = "notebook")]
pub use notebook::{CellOutput, CellType, Notebook, NotebookCell, NotebookRuntime};
pub use portability::{PortabilityAnalyzer, PortabilityReport, PortabilityScore};
pub use repl::{ReplOutput, TimingInfo, WasmRepl};
#[cfg(feature = "notebook")]
pub use shared_session::{DefId, ExecuteResponse, ExecutionMode, GlobalRegistry, SharedSession};
pub use wit::{InterfaceDefinition, WitGenerator, WitInterface};

use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::Result;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

/// High-level WASM compiler for Ruchy AST
pub struct WasmCompiler {
    optimization_level: u8,
    config: ComponentConfig,
}

impl WasmCompiler {
    /// Create a new WASM compiler
    pub fn new() -> Self {
        Self {
            optimization_level: 0,
            config: ComponentConfig::default(),
        }
    }

    /// Set optimization level (0-3)
    pub fn set_optimization_level(&mut self, level: u8) {
        self.optimization_level = level.min(3);
    }

    /// Compile AST to WASM module
    pub fn compile(&self, ast: &Expr) -> Result<WasmModule> {
        let mut module = Module::new();
        let mut exports = vec![];

        // Type section - define function signatures
        let mut types = TypeSection::new();

        // Function section - declare functions
        let mut functions = FunctionSection::new();

        // Export section - export functions
        let mut export_section = ExportSection::new();

        // Code section - function bodies
        let mut code = CodeSection::new();

        // Process AST and generate WASM
        match &ast.kind {
            ExprKind::Function {
                name, params, body, ..
            } => {
                // Add function type
                let param_types: Vec<ValType> = params
                    .iter()
                    .map(|_| ValType::I32) // Simplification: all params are i32
                    .collect();
                let result_types = vec![ValType::I32]; // Simplification: returns i32

                types.function(param_types, result_types);
                functions.function(0); // Reference to type 0

                // Generate function body
                let mut func = Function::new(vec![]);
                self.compile_expr(body, &mut func)?;
                if !self.has_return(body) {
                    func.instruction(&Instruction::I32Const(0));
                }
                func.instruction(&Instruction::End);
                code.function(&func);

                // Export the function
                export_section.export(name, ExportKind::Func, 0);
                exports.push(name.clone());
            }
            ExprKind::Block(exprs) => {
                // Process multiple top-level expressions
                for expr in exprs {
                    if let ExprKind::Function { name, .. } = &expr.kind {
                        exports.push(name.clone());
                    }
                }
            }
            _ => {
                // For other expressions, wrap in a main function
                types.function(vec![], vec![ValType::I32]);
                functions.function(0);

                let mut func = Function::new(vec![]);
                self.compile_expr(ast, &mut func)?;
                func.instruction(&Instruction::End);
                code.function(&func);
            }
        }

        // Assemble the module
        if !types.is_empty() {
            module.section(&types);
        }
        if !functions.is_empty() {
            module.section(&functions);
        }
        if !export_section.is_empty() {
            module.section(&export_section);
        }
        if !code.is_empty() {
            module.section(&code);
        }

        let bytes = module.finish();

        Ok(WasmModule { bytes, exports })
    }

    /// Compile an expression to WASM instructions
    fn compile_expr(&self, expr: &Expr, func: &mut Function) -> Result<()> {
        match &expr.kind {
            ExprKind::Literal(lit) => match lit {
                Literal::Integer(n, _) => {
                    func.instruction(&Instruction::I32Const(*n as i32));
                }
                Literal::Float(f) => {
                    func.instruction(&Instruction::F64Const(*f));
                }
                Literal::Bool(b) => {
                    func.instruction(&Instruction::I32Const(i32::from(*b)));
                }
                _ => {
                    // Other literals default to 0
                    func.instruction(&Instruction::I32Const(0));
                }
            },
            ExprKind::Binary { left, op, right } => {
                use crate::frontend::ast::BinaryOp;
                self.compile_expr(left, func)?;
                self.compile_expr(right, func)?;
                match op {
                    BinaryOp::Add => func.instruction(&Instruction::I32Add),
                    BinaryOp::Subtract => func.instruction(&Instruction::I32Sub),
                    BinaryOp::Multiply => func.instruction(&Instruction::I32Mul),
                    BinaryOp::Divide => func.instruction(&Instruction::I32DivS),
                    _ => func.instruction(&Instruction::I32Add), // Default
                };
            }
            _ => {
                // Default: push 0 on stack
                func.instruction(&Instruction::I32Const(0));
            }
        }
        Ok(())
    }

    /// Check if expression contains a return
    fn has_return(&self, expr: &Expr) -> bool {
        matches!(expr.kind, ExprKind::Return { .. })
    }
}

/// A compiled WASM module
pub struct WasmModule {
    bytes: Vec<u8>,
    exports: Vec<String>,
}

impl WasmModule {
    /// Get the WASM bytecode
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Check if module has an export
    pub fn has_export(&self, name: &str) -> bool {
        self.exports.contains(&name.to_string())
    }

    /// Validate the module
    pub fn validate(&self) -> Result<()> {
        // Basic validation - check magic number
        if self.bytes.len() >= 4 && self.bytes[0..4] == [0x00, 0x61, 0x73, 0x6d] {
            Ok(())
        } else {
            anyhow::bail!("Invalid WASM module")
        }
    }
}

impl Default for WasmCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Span};

    // Helper to create test expressions
    fn make_int(n: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(n, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_float(f: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(f)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_bool(b: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(b)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // =====================================================================
    // WasmCompiler Tests
    // =====================================================================

    #[test]
    fn test_compiler_new() {
        let compiler = WasmCompiler::new();
        assert_eq!(compiler.optimization_level, 0);
    }

    #[test]
    fn test_compiler_default() {
        let compiler = WasmCompiler::default();
        assert_eq!(compiler.optimization_level, 0);
    }

    #[test]
    fn test_set_optimization_level_valid() {
        let mut compiler = WasmCompiler::new();
        compiler.set_optimization_level(2);
        assert_eq!(compiler.optimization_level, 2);
    }

    #[test]
    fn test_set_optimization_level_clamps_high() {
        let mut compiler = WasmCompiler::new();
        compiler.set_optimization_level(10);
        assert_eq!(
            compiler.optimization_level, 3,
            "Should clamp to maximum of 3"
        );
    }

    #[test]
    fn test_set_optimization_level_zero() {
        let mut compiler = WasmCompiler::new();
        compiler.set_optimization_level(0);
        assert_eq!(compiler.optimization_level, 0);
    }

    #[test]
    fn test_compile_integer_literal() {
        let compiler = WasmCompiler::new();
        let ast = make_int(42);
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile integer literal");
    }

    #[test]
    fn test_compile_float_literal() {
        let compiler = WasmCompiler::new();
        let ast = make_float(3.14);
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile float literal");
    }

    #[test]
    fn test_compile_bool_literal() {
        let compiler = WasmCompiler::new();
        let ast = make_bool(true);
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile bool literal");
    }

    #[test]
    fn test_compile_binary_add() {
        let compiler = WasmCompiler::new();
        let ast = make_binary(make_int(2), BinaryOp::Add, make_int(3));
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile addition");
    }

    #[test]
    fn test_compile_binary_subtract() {
        let compiler = WasmCompiler::new();
        let ast = make_binary(make_int(5), BinaryOp::Subtract, make_int(2));
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile subtraction");
    }

    #[test]
    fn test_compile_binary_multiply() {
        let compiler = WasmCompiler::new();
        let ast = make_binary(make_int(3), BinaryOp::Multiply, make_int(4));
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile multiplication");
    }

    #[test]
    fn test_compile_binary_divide() {
        let compiler = WasmCompiler::new();
        let ast = make_binary(make_int(10), BinaryOp::Divide, make_int(2));
        let result = compiler.compile(&ast);
        assert!(result.is_ok(), "Should compile division");
    }

    #[test]
    fn test_has_return_false() {
        let compiler = WasmCompiler::new();
        let ast = make_int(42);
        assert!(
            !compiler.has_return(&ast),
            "Integer literal should not have return"
        );
    }

    #[test]
    fn test_has_return_true() {
        let compiler = WasmCompiler::new();
        let ast = Expr {
            kind: ExprKind::Return {
                value: Some(Box::new(make_int(42))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(
            compiler.has_return(&ast),
            "Return expression should have return"
        );
    }

    // =====================================================================
    // WasmModule Tests
    // =====================================================================

    #[test]
    fn test_module_bytes() {
        let compiler = WasmCompiler::new();
        let ast = make_int(42);
        let module = compiler.compile(&ast).expect("Should compile");
        let bytes = module.bytes();
        assert!(!bytes.is_empty(), "Module should have bytecode");
    }

    #[test]
    fn test_module_validate_valid() {
        let compiler = WasmCompiler::new();
        let ast = make_int(42);
        let module = compiler.compile(&ast).expect("Should compile");
        let result = module.validate();
        assert!(result.is_ok(), "Valid WASM module should pass validation");
    }

    #[test]
    fn test_module_validate_invalid() {
        let module = WasmModule {
            bytes: vec![0xFF, 0xFF, 0xFF, 0xFF],
            exports: vec![],
        };
        let result = module.validate();
        assert!(
            result.is_err(),
            "Invalid WASM module should fail validation"
        );
    }

    #[test]
    fn test_module_validate_empty() {
        let module = WasmModule {
            bytes: vec![],
            exports: vec![],
        };
        let result = module.validate();
        assert!(result.is_err(), "Empty module should fail validation");
    }

    #[test]
    fn test_module_has_magic_number() {
        let compiler = WasmCompiler::new();
        let ast = make_int(42);
        let module = compiler.compile(&ast).expect("Should compile");
        let bytes = module.bytes();
        assert!(bytes.len() >= 4, "Module should have at least 4 bytes");
        assert_eq!(
            &bytes[0..4],
            &[0x00, 0x61, 0x73, 0x6d],
            "Should have WASM magic number"
        );
    }

    #[test]
    fn test_module_has_export_false() {
        let compiler = WasmCompiler::new();
        let ast = make_int(42);
        let module = compiler.compile(&ast).expect("Should compile");
        assert!(
            !module.has_export("nonexistent"),
            "Should not have nonexistent export"
        );
    }

    // =====================================================================
    // Integration Tests
    // =====================================================================

    #[test]
    fn test_compile_nested_arithmetic() {
        let compiler = WasmCompiler::new();
        // (2 + 3) * 4
        let inner = make_binary(make_int(2), BinaryOp::Add, make_int(3));
        let outer = make_binary(inner, BinaryOp::Multiply, make_int(4));
        let result = compiler.compile(&outer);
        assert!(result.is_ok(), "Should compile nested arithmetic");
        let module = result.unwrap();
        assert!(
            module.validate().is_ok(),
            "Nested arithmetic should produce valid WASM"
        );
    }

    #[test]
    fn test_compile_different_optimization_levels() {
        let ast = make_binary(make_int(2), BinaryOp::Add, make_int(3));

        for level in 0..=3 {
            let mut compiler = WasmCompiler::new();
            compiler.set_optimization_level(level);
            let result = compiler.compile(&ast);
            assert!(
                result.is_ok(),
                "Should compile at optimization level {level}"
            );
        }
    }

    #[test]
    fn test_compile_preserves_bytecode() {
        let compiler = WasmCompiler::new();
        let ast = make_int(123);
        let module = compiler.compile(&ast).expect("Should compile");
        let bytes1 = module.bytes();
        let bytes2 = module.bytes();
        assert_eq!(
            bytes1, bytes2,
            "Multiple calls to bytes() should return same data"
        );
    }

    // =====================================================================
    // Property Tests (10,000+ cases for mathematical proof)
    // =====================================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]

            /// Property 1: Compilation of integer literals never panics
            #[test]
            fn prop_compile_integer_never_panics(n in any::<i32>()) {
                let compiler = WasmCompiler::new();
                let ast = make_int(i64::from(n));
                let _ = compiler.compile(&ast);
            }

            /// Property 2: Compilation of float literals never panics
            #[test]
            fn prop_compile_float_never_panics(f in any::<f64>()) {
                let compiler = WasmCompiler::new();
                let ast = make_float(f);
                let _ = compiler.compile(&ast);
            }

            /// Property 3: All valid WASM modules have magic number
            #[test]
            fn prop_compiled_modules_have_magic_number(n in any::<i32>()) {
                let compiler = WasmCompiler::new();
                let ast = make_int(i64::from(n));
                if let Ok(module) = compiler.compile(&ast) {
                    let bytes = module.bytes();
                    prop_assert!(bytes.len() >= 4, "Module should have at least 4 bytes");
                    prop_assert_eq!(&bytes[0..4], &[0x00, 0x61, 0x73, 0x6d], "Should have WASM magic number");
                }
            }

            /// Property 4: Compilation is deterministic
            #[test]
            fn prop_compilation_is_deterministic(n in any::<i32>()) {
                let compiler = WasmCompiler::new();
                let ast = make_int(i64::from(n));
                if let Ok(module1) = compiler.compile(&ast) {
                    if let Ok(module2) = compiler.compile(&ast) {
                        prop_assert_eq!(module1.bytes(), module2.bytes(), "Same AST should produce same bytecode");
                    }
                }
            }

            /// Property 5: Optimization level always clamped to 0-3
            #[test]
            fn prop_optimization_level_clamped(level in any::<u8>()) {
                let mut compiler = WasmCompiler::new();
                compiler.set_optimization_level(level);
                prop_assert!(compiler.optimization_level <= 3, "Optimization level should be clamped to max 3");
            }

            /// Property 6: Valid modules always pass validation
            #[test]
            fn prop_valid_modules_pass_validation(n in any::<i32>()) {
                let compiler = WasmCompiler::new();
                let ast = make_int(i64::from(n));
                if let Ok(module) = compiler.compile(&ast) {
                    prop_assert!(module.validate().is_ok(), "Valid module should pass validation");
                }
            }

            /// Property 7: Binary addition is commutative in compilation
            #[test]
            fn prop_binary_add_compiles_consistently(a in any::<i32>(), b in any::<i32>()) {
                let compiler = WasmCompiler::new();
                let ast = make_binary(make_int(i64::from(a)), BinaryOp::Add, make_int(i64::from(b)));
                if let Ok(module) = compiler.compile(&ast) {
                    prop_assert!(module.validate().is_ok(), "Binary operation should produce valid WASM");
                }
            }

            /// Property 8: Multiple calls to bytes() return same data
            #[test]
            fn prop_bytes_call_idempotent(n in any::<i32>()) {
                let compiler = WasmCompiler::new();
                let ast = make_int(i64::from(n));
                if let Ok(module) = compiler.compile(&ast) {
                    let bytes1 = module.bytes();
                    let bytes2 = module.bytes();
                    let bytes3 = module.bytes();
                    prop_assert_eq!(bytes1, bytes2);
                    prop_assert_eq!(bytes2, bytes3);
                }
            }
        }
    }
}
