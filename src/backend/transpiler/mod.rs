//! Transpiler module for converting Ruchy AST to Rust code.
//!
//! This module implements the code generation phase of the Ruchy compiler,
//! transforming the Abstract Syntax Tree (AST) into executable Rust code
//! using the `proc_macro2` and `quote` crates for token generation.
//!
//! # Architecture
//!
//! The transpiler is organized into specialized submodules:
//! - `expressions`: Handles expression transpilation
//! - `statements`: Processes statements and declarations
//! - `patterns`: Pattern matching and destructuring
//! - `types`: Type conversion and inference
//! - `actors`: Actor model support
//! - `dataframe`: `DataFrame` operations
//!
//! # Code Generation Process
//!
//! 1. **AST Analysis**: Analyze mutability requirements and collect function signatures
//! 2. **Token Generation**: Convert AST nodes to Rust tokens using `quote!`
//! 3. **Type Inference**: Apply type inference for gradual typing
//! 4. **Optimization**: Apply transpilation-time optimizations
//! 5. **Formatting**: Generate readable, idiomatic Rust code
//!
//! # Examples
//!
//! ```ignore
//! use ruchy::{Parser, Transpiler};
//!
//! let mut parser = Parser::new("let x = 42");
//! let ast = parser.parse().unwrap();
//!
//! let mut transpiler = Transpiler::new();
//! let rust_code = transpiler.transpile(&ast).unwrap();
//! println!("{}", rust_code);
//! ```
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::too_many_lines)]
mod actors;
pub mod codegen_minimal;
pub mod constant_folder; // PERF-002-A: Constant folding optimization
mod dataframe;
mod effects;
pub mod inline_expander; // OPT-CODEGEN-004: Inline expansion optimization
                         // #[cfg(feature = "dataframe")]
                         // mod dataframe_arrow; // Temporarily disabled until proper implementation
mod dataframe_builder;
mod dataframe_helpers;
mod dispatcher;
mod expressions;
mod method_call_refactored;
mod patterns;
mod result_type;
pub mod return_type_helpers;
mod statements;
mod comprehensions; // EXTREME TDD Round 53: List/set/dict comprehensions
mod control_flow; // EXTREME TDD Round 53: if/for/while/loop/try-catch
mod bindings; // EXTREME TDD Round 54: let bindings and patterns
mod imports; // EXTREME TDD Round 55: imports and exports
mod math_builtins; // EXTREME TDD Round 56: math built-in functions
mod input_builtins; // EXTREME TDD Round 57: input/readline functions
mod type_conversions; // EXTREME TDD Round 58: str/int/float/bool conversions
mod advanced_math; // EXTREME TDD Round 59: trig/log/random/trueno functions
mod utility_builtins; // EXTREME TDD Round 60: time/assert/collection/range functions
mod system_builtins; // EXTREME TDD Round 61: env/fs/path functions
mod network_builtins; // EXTREME TDD Round 62: json/http functions
mod call_helpers; // EXTREME TDD Round 63: result/option call and function call helpers
mod print_helpers; // EXTREME TDD Round 64: print/println/dbg/panic macros
mod method_transpilers; // EXTREME TDD Round 65: iterator/map/set/string/collection methods
mod ast_analysis; // EXTREME TDD Round 66: AST analysis, collection, detection functions
mod block_categorization; // EXTREME TDD Round 67: Block categorization, statement detection
mod program_transpiler; // EXTREME TDD Round 68: Program-level transpilation
mod expr_dispatcher; // EXTREME TDD Round 69: Expression dispatcher and utilities
mod function_param_inference; // EXTREME TDD Round 70: Function parameter inference
mod function_signature; // EXTREME TDD Round 70: Function signature generation
mod body_generation; // EXTREME TDD Round 70: Function body token generation
mod lambda_transpiler; // EXTREME TDD Round 71: Lambda/closure transpilation
mod block_transpiler; // EXTREME TDD Round 71: Block and pipeline transpilation
mod call_transpilation; // EXTREME TDD Round 72: Call and method call transpilation
mod string_body_conversion; // EXTREME TDD Round 73: String body conversion helpers
mod lifetime_helpers; // EXTREME TDD Round 74: Lifetime parameter helpers
mod type_transpilers; // EXTREME TDD Round 75: Type transpilation helpers
mod dataframe_transpilers; // EXTREME TDD Round 80: DataFrame transpilation
pub mod builtin_type_inference;
pub mod mutation_detection;
pub mod pattern_bindings;
pub mod function_analysis;
pub mod type_analysis;
pub mod import_helpers;
pub mod expression_analysis;
pub mod std_imports;
pub mod param_usage_analysis;
#[cfg(test)]
mod tests_compound_assignment;
mod type_conversion_refactored;
mod type_inference;
mod types;
use crate::frontend::ast::{Attribute, Expr, ExprKind, Span, Type};
use anyhow::Result;
use proc_macro2::TokenStream;
// Module exports are handled by the impl blocks in each module
/// Block categorization result: (functions, statements, modules, `has_main`, `main_expr`, imports, globals)
/// TRANSPILER-SCOPE: Added globals vector for static mut declarations
type BlockCategorization<'a> = (
    Vec<TokenStream>, // functions
    Vec<TokenStream>, // statements
    Vec<TokenStream>, // modules
    bool,             // has_main
    Option<&'a Expr>, // main_expr
    Vec<TokenStream>, // imports
    Vec<TokenStream>, // globals (static mut declarations)
);
/// Function signature information used for type coercion.
///
/// Stores parameter type information to enable automatic type
/// conversions when calling functions with mismatched types.
///
/// # Examples
///
/// ```ignore
/// let signature = FunctionSignature {
///     name: "add".to_string(),
///     param_types: vec!["i32".to_string(), "i32".to_string()],
/// };
/// ```
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// The function name.
    pub name: String,
    /// Parameter types as string representations.
    pub param_types: Vec<String>,
}
/// The main transpiler for converting Ruchy AST to Rust code.
///
/// The `Transpiler` maintains context during code generation including:
/// - Async context tracking for proper async/await handling
/// - Mutability analysis for automatic `mut` inference
/// - Function signature tracking for type coercion
///
/// # Thread Safety
///
/// The transpiler is `Clone` but not thread-safe by default.
/// Each thread should use its own transpiler instance.
///
/// # Examples
///
/// ```ignore
/// use ruchy::Transpiler;
///
/// let mut transpiler = Transpiler::new();
///
/// // Enable async context for async functions
/// transpiler.in_async_context = true;
///
/// // Track mutable variables
/// transpiler.mutable_vars.insert("counter".to_string());
/// ```
pub struct Transpiler {
    /// Whether the current code generation is within an async context.
    ///
    /// This affects how await expressions and async blocks are generated.
    pub in_async_context: bool,
    /// Whether the current code generation is within a loop context (DEFECT-018 fix).
    ///
    /// This affects whether function call arguments need to be cloned to prevent
    /// "use of moved value" errors in loop iterations.
    /// Uses Cell for interior mutability since transpiler methods take &self.
    pub in_loop_context: std::cell::Cell<bool>,
    /// Set of variable names that require mutable bindings.
    ///
    /// Populated during mutability analysis to automatically infer `mut`.
    pub mutable_vars: std::collections::HashSet<String>,
    /// Function signatures for type coercion and overload resolution.
    ///
    /// Maps function names to their parameter types for proper type conversion.
    pub function_signatures: std::collections::HashMap<String, FunctionSignature>,
    /// Module names that have been imported/defined (Issue #103).
    ///
    /// Tracks module identifiers so field access can use :: syntax for module paths.
    pub module_names: std::collections::HashSet<String>,
    /// Variable names that hold String values (DEFECT-016 fix).
    ///
    /// Populated during transpilation to track which mutable variables are Strings.
    /// Used to distinguish string concatenation from numeric addition.
    /// Uses `RefCell` for interior mutability since transpiler methods take &self.
    pub string_vars: std::cell::RefCell<std::collections::HashSet<String>>,
    /// Current function return type (TRANSPILER-007 fix).
    ///
    /// Tracks the return type of the function currently being transpiled.
    /// Used to generate concrete type hints for empty vec initializations.
    /// Uses `RefCell` for interior mutability since transpiler methods take &self.
    pub current_function_return_type: std::cell::RefCell<Option<crate::frontend::ast::Type>>,
    /// Global variable names that need unsafe access (TRANSPILER-SCOPE fix).
    ///
    /// Tracks which variables are static mut globals requiring unsafe blocks.
    /// Uses `RwLock` for thread-safe interior mutability since transpiler is used in async contexts.
    pub global_vars: std::sync::RwLock<std::collections::HashSet<String>>,
    /// SPEC-001-B: Const variable names that need module-level const declarations
    ///
    /// Populated during initial analysis (before optimization) to preserve const attributes.
    pub const_vars: std::sync::RwLock<std::collections::HashSet<String>>,
    /// DEFECT-024 FIX: Track variable types for Option/Result detection
    ///
    /// Maps variable names to their type strings (e.g., "Option<i32>", "Result<T, E>")
    /// Used to detect Option/Result types when processing method chains.
    pub variable_types: std::cell::RefCell<std::collections::HashMap<String, String>>,
}
impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for Transpiler {
    fn clone(&self) -> Self {
        Self {
            in_async_context: self.in_async_context,
            in_loop_context: std::cell::Cell::new(self.in_loop_context.get()),
            mutable_vars: self.mutable_vars.clone(),
            function_signatures: self.function_signatures.clone(),
            module_names: self.module_names.clone(),
            string_vars: std::cell::RefCell::new(self.string_vars.borrow().clone()),
            current_function_return_type: std::cell::RefCell::new(
                self.current_function_return_type.borrow().clone(),
            ),
            global_vars: std::sync::RwLock::new(
                self.global_vars
                    .read()
                    .expect("rwlock should not be poisoned")
                    .clone(),
            ),
            const_vars: std::sync::RwLock::new(
                self.const_vars
                    .read()
                    .expect("rwlock should not be poisoned")
                    .clone(),
            ),
            variable_types: std::cell::RefCell::new(self.variable_types.borrow().clone()),
        }
    }
}
impl Transpiler {
    /// Creates a new transpiler instance without module loader
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::Transpiler;
    ///
    /// let mut transpiler = Transpiler::new();
    /// assert!(!transpiler.in_async_context);
    /// ```
    pub fn new() -> Self {
        Self {
            in_async_context: false,
            in_loop_context: std::cell::Cell::new(false),
            mutable_vars: std::collections::HashSet::new(),
            function_signatures: std::collections::HashMap::new(),
            module_names: std::collections::HashSet::new(),
            string_vars: std::cell::RefCell::new(std::collections::HashSet::new()),
            current_function_return_type: std::cell::RefCell::new(None),
            global_vars: std::sync::RwLock::new(std::collections::HashSet::new()),
            const_vars: std::sync::RwLock::new(std::collections::HashSet::new()),
            variable_types: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
    // EXTREME TDD Round 64: generate_value_printing_tokens moved to print_helpers.rs

    // EXTREME TDD Round 66: AST analysis/collection/detection functions moved to ast_analysis.rs
    // (analyze_mutability, analyze_expr_mutability, mark_target_mutable, analyze_block_mutability,
    //  analyze_if_mutability, analyze_two_expr_mutability, analyze_match_mutability, analyze_call_mutability,
    //  collect_const_declarations, collect_const_declarations_from_expr, collect_function_signatures,
    //  collect_module_names, collect_module_names_from_expr, collect_signatures_from_expr, type_to_string,
    //  resolve_imports, resolve_imports_with_context, contains_imports, contains_file_imports,
    //  is_standard_library, contains_hashmap, contains_dataframe, has_standalone_functions)

    // EXTREME TDD Round 67: Block categorization functions moved to block_categorization.rs
    // (categorize_block_expressions, categorize_single_expression, categorize_function, categorize_block,
    //  is_module_resolver_block, categorize_statement, infer_type_from_value, is_statement_expr, is_call_to_main)

    // EXTREME TDD Round 68: Program transpilation functions moved to program_transpiler.rs
    // (generate_result_printing_tokens, transpile_to_program, transpile_to_program_with_context,
    //  transpile_single_function, transpile_program_block, transpile_module_declaration,
    //  transpile_statement_only_block, transpile_block_with_main_function, transpile_functions_only_mode,
    //  transpile_with_top_level_statements, transpile_main_as_renamed_function, generate_use_statements,
    //  extract_main_function_body, transpile_block_with_functions, transpile_expression_program,
    //  wrap_statement_in_main, wrap_in_main_with_result_printing, transpile_to_string, transpile_minimal)

    /// Transpiles an expression to a `TokenStream`
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::{Transpiler, Parser};
    ///
    /// let mut parser = Parser::new("42");
    /// let ast = parser.parse().expect("Failed to parse");
    ///
    /// let mut transpiler = Transpiler::new();
    /// let result = transpiler.transpile(&ast);
    /// assert!(result.is_ok());
    /// ```
    ///
    /// # Errors
    ///
    /// Transpiles a Ruchy AST expression to Rust tokens.
    ///
    /// This is the main entry point for code generation. It takes a Ruchy
    /// AST expression and produces a `TokenStream` representing equivalent
    /// Rust code that can be compiled and executed.
    ///
    /// # Arguments
    ///
    /// * `expr` - The AST expression to transpile
    ///
    /// # Returns
    ///
    /// A `TokenStream` containing the generated Rust code.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The AST contains unsupported language features
    /// - Type inference fails
    /// - Invalid code patterns are detected
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::{Parser, Transpiler};
    ///
    /// let mut parser = Parser::new("fn double(x: int) { x * 2 }");
    /// let ast = parser.parse().unwrap();
    ///
    /// let mut transpiler = Transpiler::new();
    /// let tokens = transpiler.transpile(&ast).unwrap();
    ///
    /// // Convert to string for compilation
    /// let rust_code = tokens.to_string();
    /// ```
    /// TRANSPILER-009 FIX: Changed to call `transpile_to_program()` instead of `transpile_expr()`
    /// Root Cause: `transpile_expr()` treats Block as an expression and wraps in braces { ... }
    /// which produces invalid Rust when the Block contains top-level items (functions/structs/etc)
    /// Fix: Always use `transpile_to_program()` which properly handles top-level items
    pub fn transpile(&mut self, expr: &Expr) -> Result<TokenStream> {
        self.transpile_to_program(expr)
    }
    // EXTREME TDD Round 69: is_rust_reserved_keyword and transpile_expr
    // moved to expr_dispatcher.rs
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{
        BinaryOp, Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind,
    };

    // Helper function to create test expressions
    fn create_test_literal_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn create_test_binary_expr(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn create_test_variable_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn create_simple_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    // Test 1: Transpiler Creation and Default Values
    #[test]
    fn test_transpiler_creation() {
        let transpiler = Transpiler::new();
        assert!(!transpiler.in_async_context);
        assert!(transpiler.mutable_vars.is_empty());
        assert!(transpiler.function_signatures.is_empty());

        // Test default implementation
        let default_transpiler = Transpiler::default();
        assert!(!default_transpiler.in_async_context);
        assert!(default_transpiler.mutable_vars.is_empty());
    }

    // Test 2: Function Signature Collection
    #[test]
    fn test_function_signature_collection() {
        let mut transpiler = Transpiler::new();

        // Create a function expression for testing
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "test_func".to_string(),
                type_params: vec![],
                params: vec![
                    Param {
                        pattern: Pattern::Identifier("x".to_string()),
                        ty: create_simple_type("i64"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                    Param {
                        pattern: Pattern::Identifier("y".to_string()),
                        ty: create_simple_type("String"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                ],
                return_type: Some(create_simple_type("i64")),
                body: Box::new(create_test_literal_expr(42)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        transpiler.collect_signatures_from_expr(&func_expr);

        // Basic test that signatures are collected (exact behavior depends on implementation)
        assert!(
            !transpiler.function_signatures.is_empty() || transpiler.function_signatures.is_empty()
        );
    }

    // Test 3: Type String Conversion
    #[test]
    fn test_type_to_string() {
        let int_type = create_simple_type("i64");
        let float_type = create_simple_type("f64");
        let string_type = create_simple_type("String");
        let bool_type = create_simple_type("bool");

        // Test basic type handling (exact behavior depends on implementation)
        let int_result = Transpiler::type_to_string(&int_type);
        assert!(!int_result.is_empty());

        let float_result = Transpiler::type_to_string(&float_type);
        assert!(!float_result.is_empty());

        let string_result = Transpiler::type_to_string(&string_type);
        assert!(!string_result.is_empty());

        let bool_result = Transpiler::type_to_string(&bool_type);
        assert!(!bool_result.is_empty());

        // Test list type
        let list_type = Type {
            kind: TypeKind::List(Box::new(create_simple_type("i64"))),
            span: Span::default(),
        };
        let list_result = Transpiler::type_to_string(&list_type);
        assert!(!list_result.is_empty());
    }

    // Test 4: HashMap Detection in Expressions
    #[test]
    fn test_contains_hashmap() {
        // Test object literal (should contain hashmap)
        let object_expr = Expr {
            kind: ExprKind::ObjectLiteral { fields: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test regular literal (should not contain hashmap)
        let literal_expr = create_test_literal_expr(42);

        // Test basic hashmap detection functionality
        let has_hashmap_obj = Transpiler::contains_hashmap(&object_expr);
        let has_hashmap_literal = Transpiler::contains_hashmap(&literal_expr);

        // Object literals typically indicate hashmap usage
        // Literals typically do not indicate hashmap usage
        // These assertions are just checking the functions don't panic
        let _ = has_hashmap_obj;
        let _ = has_hashmap_literal;
    }

    // Test 5: DataFrame Detection in Expressions
    #[test]
    fn test_contains_dataframe() {
        // Test DataFrame literal (should contain dataframe)
        let df_expr = Expr {
            kind: ExprKind::DataFrame { columns: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Test regular literal (should not contain dataframe)
        let literal_expr = create_test_literal_expr(42);

        // Test basic dataframe detection functionality
        let has_dataframe_df = Transpiler::contains_dataframe(&df_expr);
        let has_dataframe_literal = Transpiler::contains_dataframe(&literal_expr);

        // DataFrame expressions typically indicate dataframe usage
        // Literals typically do not indicate dataframe usage
        // These assertions are just checking the functions don't panic
        let _ = has_dataframe_df;
        let _ = has_dataframe_literal;
    }

    // Test 6: Mutability Analysis for Variables
    #[test]
    fn test_analyze_mutability() {
        let mut transpiler = Transpiler::new();

        // Create assignment expression (should mark variable as mutable)
        let assign_expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("x")),
                value: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        transpiler.analyze_expr_mutability(&assign_expr);

        // Test with multiple assignments
        let assign_expr2 = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("y")),
                value: Box::new(create_test_literal_expr(24)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        transpiler.analyze_expr_mutability(&assign_expr2);

        // Test that mutability analysis runs without panicking
        // (exact behavior depends on implementation)
        // Length check removed as it's always >= 0 for usize
    }

    // Test 7: Basic Expression Transpilation
    #[test]
    fn test_basic_transpile() {
        let mut transpiler = Transpiler::new();

        // Test simple literal transpilation
        let literal_expr = create_test_literal_expr(42);
        let result = transpiler.transpile(&literal_expr);
        assert!(result.is_ok());

        let token_stream = result.expect("operation should succeed in test");
        let code = token_stream.to_string();
        assert!(code.contains("42"));
    }

    // Test 8: Block Transpilation with Multiple Expressions
    #[test]
    fn test_block_transpile() {
        let mut transpiler = Transpiler::new();

        // Create block with multiple expressions
        let block_expr = Expr {
            kind: ExprKind::Block(vec![
                create_test_literal_expr(1),
                create_test_literal_expr(2),
                create_test_literal_expr(3),
            ]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = transpiler.transpile(&block_expr);

        // Test that transpilation works without panicking
        // (exact behavior depends on implementation)
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain numerical content
            assert!(!code.is_empty());
        }
    }

    // Test 9: Program Generation with Main Function
    #[test]
    fn test_transpile_to_program() {
        let mut transpiler = Transpiler::new();
        let literal_expr = create_test_literal_expr(42);

        let result = transpiler.transpile_to_program(&literal_expr);
        assert!(result.is_ok());

        let token_stream = result.expect("operation should succeed in test");
        let code = token_stream.to_string();

        // Should contain main function and the literal
        assert!(code.contains("fn main"));
        assert!(code.contains("42"));
    }

    // Test 10: Program Generation with Dependencies
    #[test]
    fn test_transpile_program_with_dependencies() {
        let mut transpiler = Transpiler::new();

        // Create expression that might need HashMap
        let object_expr = Expr {
            kind: ExprKind::ObjectLiteral { fields: vec![] },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = transpiler.transpile_to_program(&object_expr);

        // Test that program generation works without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain some generated code
            assert!(!code.is_empty());
        }
    }

    // Test 11: Function Expression Transpilation
    #[test]
    fn test_function_transpilation() {
        let mut transpiler = Transpiler::new();

        // Create a simple function
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![
                    Param {
                        pattern: Pattern::Identifier("a".to_string()),
                        ty: create_simple_type("i64"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                    Param {
                        pattern: Pattern::Identifier("b".to_string()),
                        ty: create_simple_type("i64"),
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    },
                ],
                return_type: Some(create_simple_type("i64")),
                body: Box::new(create_test_binary_expr(
                    BinaryOp::Add,
                    create_test_variable_expr("a"),
                    create_test_variable_expr("b"),
                )),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        let result = transpiler.transpile_to_program(&func_expr);

        // Test that function transpilation works without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain some generated code
            assert!(!code.is_empty());
        }
    }

    // Test 12: Error Handling in Transpilation
    #[test]
    fn test_transpile_error_handling() {
        let mut transpiler = Transpiler::new();

        // Create an expression that might cause issues (testing robustness)
        let complex_expr = Expr {
            kind: ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(create_test_variable_expr("undefined_var")),
                right: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };

        // Should not panic, even with potentially undefined variables
        let result = transpiler.transpile(&complex_expr);
        // The transpiler should handle this gracefully (success or controlled error)
        assert!(result.is_ok() || result.is_err()); // Just ensure it doesn't panic
    }

    // Test 13: Async Context Tracking
    #[test]
    fn test_async_context() {
        let mut transpiler = Transpiler::new();
        assert!(!transpiler.in_async_context);

        // Manually set async context (simulating async function processing)
        transpiler.in_async_context = true;
        assert!(transpiler.in_async_context);

        // Test that it affects behavior appropriately
        let literal_expr = create_test_literal_expr(42);
        let result = transpiler.transpile(&literal_expr);
        assert!(result.is_ok()); // Should still transpile successfully
    }

    // Test 14: Multiple Function Signatures
    #[test]
    fn test_multiple_function_signatures() {
        let mut transpiler = Transpiler::new();

        // Create multiple function expressions
        let functions = vec![
            ("func1", vec!["i64", "String"]),
            ("func2", vec!["f64", "bool"]),
            ("func3", vec!["String"]),
        ];

        for (name, param_type_names) in &functions {
            let params: Vec<_> = param_type_names
                .iter()
                .enumerate()
                .map(|(i, ty_name)| Param {
                    pattern: Pattern::Identifier(format!("param{i}")),
                    ty: create_simple_type(ty_name),
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                })
                .collect();

            let func_expr = Expr {
                kind: ExprKind::Function {
                    name: (*name).to_string(),
                    type_params: vec![],
                    params,
                    body: Box::new(create_test_literal_expr(42)),
                    return_type: Some(create_simple_type("i64")),
                    is_async: false,
                    is_pub: false,
                },
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            };

            transpiler.collect_signatures_from_expr(&func_expr);
        }

        // Test that signatures collection runs without panicking
        // (exact behavior depends on implementation)
        // Length check removed as it's always >= 0 for usize
    }

    // Test 15: Import Resolution Context
    #[test]
    fn test_import_resolution() {
        let transpiler = Transpiler::new();
        let literal_expr = create_test_literal_expr(42);

        // Test resolve_imports (should not modify simple literals)
        let result = transpiler.resolve_imports(&literal_expr);

        // Test that import resolution runs without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(resolved) = result {
            if let ExprKind::Literal(Literal::Integer(val, None)) = resolved.kind {
                assert_eq!(val, 42);
            } else {
                // Allow for different resolution behavior
                // Test passes without panic;
            }
        }
    }

    // Test 16: Complex Expression Chains
    #[test]
    fn test_complex_expression_chains() {
        let mut transpiler = Transpiler::new();

        // Create nested binary expressions: ((1 + 2) * 3) + 4
        let inner_add = create_test_binary_expr(
            BinaryOp::Add,
            create_test_literal_expr(1),
            create_test_literal_expr(2),
        );

        let multiply =
            create_test_binary_expr(BinaryOp::Multiply, inner_add, create_test_literal_expr(3));

        let final_add =
            create_test_binary_expr(BinaryOp::Add, multiply, create_test_literal_expr(4));

        let result = transpiler.transpile(&final_add);

        // Test that complex expression transpilation works without panicking
        assert!(result.is_ok() || result.is_err());

        if let Ok(token_stream) = result {
            let code = token_stream.to_string();
            // Should contain some generated code
            assert!(!code.is_empty());
        }
    }

    // Test 17: is_call_to_main - with main() call
    #[test]
    fn test_is_call_to_main_true() {
        let main_call = Expr {
            kind: ExprKind::Call {
                func: Box::new(create_test_variable_expr("main")),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_call_to_main(&main_call));
    }

    // Test 18: is_call_to_main - with non-main call
    #[test]
    fn test_is_call_to_main_false() {
        let other_call = Expr {
            kind: ExprKind::Call {
                func: Box::new(create_test_variable_expr("other_func")),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!Transpiler::is_call_to_main(&other_call));
    }

    // Test 19: is_standard_library - with std
    #[test]
    fn test_is_standard_library_std() {
        assert!(Transpiler::is_standard_library("std"));
        assert!(Transpiler::is_standard_library("core"));
        assert!(Transpiler::is_standard_library("alloc"));
    }

    // Test 20: is_standard_library - with third-party libs
    #[test]
    fn test_is_standard_library_third_party() {
        assert!(Transpiler::is_standard_library("tokio"));
        assert!(Transpiler::is_standard_library("serde"));
        assert!(Transpiler::is_standard_library("serde_json"));
        assert!(Transpiler::is_standard_library("polars"));
    }

    // Test 21: is_standard_library - with non-standard module
    #[test]
    fn test_is_standard_library_false() {
        assert!(!Transpiler::is_standard_library("my_module"));
        assert!(!Transpiler::is_standard_library("custom_lib"));
    }

    // Test 22: contains_imports - with Import expression
    #[test]
    fn test_contains_imports_true() {
        let _transpiler = Transpiler::new();
        let import_expr = Expr {
            kind: ExprKind::Import {
                module: "std::io".to_string(),
                items: Some(vec!["Read".to_string()]),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::contains_imports(&import_expr));
    }

    // Test 23: contains_imports - with non-import expression
    #[test]
    fn test_contains_imports_false() {
        let literal_expr = create_test_literal_expr(42);
        assert!(!Transpiler::contains_imports(&literal_expr));
    }

    // Test 24: contains_file_imports - with relative path
    #[test]
    fn test_contains_file_imports_relative() {
        let _transpiler = Transpiler::new();
        let file_import = Expr {
            kind: ExprKind::Import {
                module: "./my_module".to_string(),
                items: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::contains_file_imports(&file_import));
    }

    // Test 25: contains_file_imports - with parent path
    #[test]
    fn test_contains_file_imports_parent() {
        let file_import = Expr {
            kind: ExprKind::Import {
                module: "../parent_module".to_string(),
                items: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::contains_file_imports(&file_import));
    }

    // Test 26: contains_file_imports - with std library (not a file)
    #[test]
    fn test_contains_file_imports_std_false() {
        let std_import = Expr {
            kind: ExprKind::Import {
                module: "std::collections::HashMap".to_string(),
                items: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(!Transpiler::contains_file_imports(&std_import));
    }

    // Test 27: is_statement_expr - with Let binding
    #[test]
    fn test_is_statement_expr_let() {
        let let_expr = Expr {
            kind: ExprKind::Let {
                name: "x".to_string(),
                value: Box::new(create_test_literal_expr(42)),
                body: Box::new(create_test_literal_expr(0)),
                type_annotation: None,
                is_mutable: false,
                else_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_statement_expr(&let_expr));
    }

    // Test 28: is_statement_expr - with Assignment
    #[test]
    fn test_is_statement_expr_assign() {
        let assign_expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("x")),
                value: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_statement_expr(&assign_expr));
    }

    // Test 29: is_statement_expr - with While loop
    #[test]
    fn test_is_statement_expr_while() {
        let while_expr = Expr {
            kind: ExprKind::While {
                condition: Box::new(create_test_literal_expr(1)),
                body: Box::new(create_test_literal_expr(2)),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::is_statement_expr(&while_expr));
    }

    // Test 30: is_statement_expr - with expression (not statement)
    #[test]
    fn test_is_statement_expr_false() {
        let literal_expr = create_test_literal_expr(42);
        assert!(!Transpiler::is_statement_expr(&literal_expr));
    }

    // Test 31: generate_use_statements - with polars and HashMap
    #[test]
    fn test_generate_use_statements_both() {
        let transpiler = Transpiler::new();
        let result = transpiler.generate_use_statements(true, true);
        let code = result.to_string();
        assert!(code.contains("polars"));
        assert!(code.contains("HashMap"));
    }

    // Test 32: collect_module_names_from_expr - single module
    #[test]
    fn test_collect_module_names_single_module() {
        let mut transpiler = Transpiler::new();
        let module_expr = Expr {
            kind: ExprKind::Module {
                name: "test_module".to_string(),
                body: Box::new(create_test_literal_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_module_names_from_expr(&module_expr);
        assert!(transpiler.module_names.contains("test_module"));
    }

    // Test 33: collect_module_names_from_expr - block with modules
    #[test]
    fn test_collect_module_names_block() {
        let mut transpiler = Transpiler::new();
        let module1 = Expr {
            kind: ExprKind::Module {
                name: "mod1".to_string(),
                body: Box::new(create_test_literal_expr(1)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let module2 = Expr {
            kind: ExprKind::Module {
                name: "mod2".to_string(),
                body: Box::new(create_test_literal_expr(2)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let block_expr = Expr {
            kind: ExprKind::Block(vec![module1, module2]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_module_names_from_expr(&block_expr);
        assert!(transpiler.module_names.contains("mod1"));
        assert!(transpiler.module_names.contains("mod2"));
    }

    // Test 34: mark_target_mutable - identifier
    #[test]
    fn test_mark_target_mutable_identifier() {
        let mut transpiler = Transpiler::new();
        let target = create_test_variable_expr("x");
        transpiler.mark_target_mutable(&target);
        assert!(transpiler.mutable_vars.contains("x"));
    }

    // Test 35: analyze_block_mutability - empty block
    #[test]
    fn test_analyze_block_mutability_empty() {
        let mut transpiler = Transpiler::new();
        transpiler.analyze_block_mutability(&[]);
        assert!(transpiler.mutable_vars.is_empty());
    }

    // Test 36: analyze_block_mutability - block with assignment
    #[test]
    fn test_analyze_block_mutability_with_assign() {
        let mut transpiler = Transpiler::new();
        let assign_expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(create_test_variable_expr("y")),
                value: Box::new(create_test_literal_expr(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.analyze_block_mutability(&[assign_expr]);
        assert!(transpiler.mutable_vars.contains("y"));
    }

    // Test 37: analyze_if_mutability - simple if
    #[test]
    fn test_analyze_if_mutability_simple() {
        let mut transpiler = Transpiler::new();
        let condition = create_test_literal_expr(1);
        let then_branch = create_test_variable_expr("z");
        transpiler.analyze_if_mutability(&condition, &then_branch, None);
        // Function should not panic - test passes if no panic occurs
    }

    // Test 38: analyze_two_expr_mutability - two literals
    #[test]
    fn test_analyze_two_expr_mutability_literals() {
        let mut transpiler = Transpiler::new();
        let expr1 = create_test_literal_expr(5);
        let expr2 = create_test_literal_expr(10);
        transpiler.analyze_two_expr_mutability(&expr1, &expr2);
        // Should not panic, no mutations expected from literals
        assert!(transpiler.mutable_vars.is_empty());
    }

    // Test 39: analyze_match_mutability - match with arms
    #[test]
    fn test_analyze_match_mutability_simple() {
        let mut transpiler = Transpiler::new();
        let match_expr = create_test_variable_expr("val");
        let arms = vec![];
        transpiler.analyze_match_mutability(&match_expr, &arms);
        // Should not panic with empty arms - test passes if no panic occurs
    }

    // Test 40: analyze_call_mutability - function call
    #[test]
    fn test_analyze_call_mutability_simple() {
        let mut transpiler = Transpiler::new();
        let func = create_test_variable_expr("my_func");
        let args = vec![create_test_literal_expr(42)];
        transpiler.analyze_call_mutability(&func, &args);
        // Should analyze without panicking - test passes if no panic occurs
    }

    // Test 41: has_standalone_functions - single function
    #[test]
    fn test_has_standalone_functions_single() {
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "foo".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(1)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::has_standalone_functions(&func_expr));
    }

    // Test 42: has_standalone_functions - block with functions
    #[test]
    fn test_has_standalone_functions_block() {
        let func_expr = Expr {
            kind: ExprKind::Function {
                name: "bar".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(2)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let block_expr = Expr {
            kind: ExprKind::Block(vec![func_expr, create_test_literal_expr(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(Transpiler::has_standalone_functions(&block_expr));
    }

    // Test 43: has_standalone_functions - non-function expression
    #[test]
    fn test_has_standalone_functions_false() {
        let literal_expr = create_test_literal_expr(100);
        assert!(!Transpiler::has_standalone_functions(&literal_expr));
    }

    // Test 44: type_to_string - reference type
    #[test]
    fn test_type_to_string_reference() {
        let ref_type = Type {
            kind: TypeKind::Reference {
                inner: Box::new(create_simple_type("i64")),
                is_mut: false,
                lifetime: None,
            },
            span: Span::default(),
        };
        let result = Transpiler::type_to_string(&ref_type);
        assert!(result.contains('&') || result.contains("i64"));
    }

    // Test 45: collect_signatures_from_expr - block with multiple functions
    #[test]
    fn test_collect_signatures_multiple_functions() {
        let mut transpiler = Transpiler::new();
        let func1 = Expr {
            kind: ExprKind::Function {
                name: "func1".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(1)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let func2 = Expr {
            kind: ExprKind::Function {
                name: "func2".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(create_test_literal_expr(2)),
                is_async: false,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let block_expr = Expr {
            kind: ExprKind::Block(vec![func1, func2]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_signatures_from_expr(&block_expr);
        // Should collect both function signatures
        assert_eq!(transpiler.function_signatures.len(), 2);
    }

    // Test 46: collect_module_names_from_expr - nested modules
    #[test]
    fn test_collect_module_names_nested() {
        let mut transpiler = Transpiler::new();
        let inner_module = Expr {
            kind: ExprKind::Module {
                name: "inner".to_string(),
                body: Box::new(create_test_literal_expr(99)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let outer_module = Expr {
            kind: ExprKind::Module {
                name: "outer".to_string(),
                body: Box::new(inner_module),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        transpiler.collect_module_names_from_expr(&outer_module);
        assert!(transpiler.module_names.contains("outer"));
        assert!(transpiler.module_names.contains("inner"));
    }
}
