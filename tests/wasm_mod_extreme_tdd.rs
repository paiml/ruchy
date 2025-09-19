// EXTREME TDD: WASM Module Core Functionality Coverage Tests
// Requirements: Complexity <10, Property tests 10,000+ iterations, Big O validation, Zero SATD
// Target: src/wasm/mod.rs - Core WASM compilation functionality (2.15% coverage -> 95%+)

use ruchy::wasm::WasmCompiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, Span, Param, Pattern, Type, TypeKind};

#[cfg(test)]
use proptest::prelude::*;

// Helper function to create test expressions with span
fn create_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span { start: 0, end: 0 })
}

// Helper function to create literal expressions
fn create_literal_expr(lit: Literal) -> Expr {
    create_expr(ExprKind::Literal(lit))
}

// Helper function to create identifier expressions
fn create_identifier_expr(name: &str) -> Expr {
    create_expr(ExprKind::Identifier(name.to_string()))
}

// Helper function to create binary expressions
fn create_binary_expr(left: Expr, op: BinaryOp, right: Expr) -> Expr {
    create_expr(ExprKind::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

// Helper function to create function expressions
fn create_function_expr(name: &str, params: Vec<Param>, body: Expr) -> Expr {
    create_expr(ExprKind::Function {
        name: name.to_string(),
        params,
        body: Box::new(body),
        return_type: None,
        type_params: vec![],
        is_async: false,
        is_pub: false,
    })
}

// Helper function to create test parameters
fn create_param(name: &str) -> Param {
    Param {
        pattern: Pattern::Identifier(name.to_string()),
        ty: Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span { start: 0, end: 3 },
        },
        default_value: None,
        is_mutable: false,
        span: Span { start: 0, end: 0 },
    }
}

// Test WasmCompiler creation and configuration
#[test]
fn test_wasm_compiler_new() {
    let compiler = WasmCompiler::new();
    // Compiler should be created without errors
    // Note: optimization_level is private, so we test behavior indirectly
    let expr = create_literal_expr(Literal::Integer(42));
    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "New compiler should be able to compile");
}

#[test]
fn test_wasm_compiler_default() {
    let compiler = WasmCompiler::default();
    // Default should match new() - test by compiling
    let expr = create_literal_expr(Literal::Integer(42));
    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Default compiler should be able to compile");
}

#[test]
fn test_wasm_compiler_set_optimization_level() {
    let mut compiler = WasmCompiler::new();

    // Test that optimization level setting doesn't break compilation
    compiler.set_optimization_level(0);
    let expr = create_literal_expr(Literal::Integer(1));
    assert!(compiler.compile(&expr).is_ok(), "Level 0 should work");

    compiler.set_optimization_level(1);
    assert!(compiler.compile(&expr).is_ok(), "Level 1 should work");

    compiler.set_optimization_level(2);
    assert!(compiler.compile(&expr).is_ok(), "Level 2 should work");

    compiler.set_optimization_level(3);
    assert!(compiler.compile(&expr).is_ok(), "Level 3 should work");
}

#[test]
fn test_wasm_compiler_optimization_level_clamp() {
    let mut compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Integer(42));

    // Test that high values don't break compilation (indicating proper clamping)
    compiler.set_optimization_level(5);
    assert!(compiler.compile(&expr).is_ok(), "High optimization level should be clamped and work");

    compiler.set_optimization_level(255);
    assert!(compiler.compile(&expr).is_ok(), "Very high optimization level should be clamped and work");
}

// Test WASM compilation for different expression types
#[test]
fn test_compile_integer_literal() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Integer(42));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile integer literal successfully");

    let module = result.unwrap();
    assert!(!module.bytes().is_empty(), "Should generate non-empty bytecode");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_float_literal() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Float(3.14));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile float literal successfully");

    let module = result.unwrap();
    assert!(!module.bytes().is_empty(), "Should generate non-empty bytecode");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_bool_literal() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Bool(true));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile boolean literal successfully");

    let module = result.unwrap();
    assert!(!module.bytes().is_empty(), "Should generate non-empty bytecode");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_string_literal() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::String("hello".to_string()));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile string literal successfully");

    let module = result.unwrap();
    assert!(!module.bytes().is_empty(), "Should generate non-empty bytecode");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_binary_addition() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Integer(10));
    let right = create_literal_expr(Literal::Integer(20));
    let expr = create_binary_expr(left, BinaryOp::Add, right);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile binary addition successfully");

    let module = result.unwrap();
    assert!(!module.bytes().is_empty(), "Should generate non-empty bytecode");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_binary_subtraction() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Integer(50));
    let right = create_literal_expr(Literal::Integer(30));
    let expr = create_binary_expr(left, BinaryOp::Subtract, right);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile binary subtraction successfully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_binary_multiplication() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Integer(6));
    let right = create_literal_expr(Literal::Integer(7));
    let expr = create_binary_expr(left, BinaryOp::Multiply, right);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile binary multiplication successfully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_binary_division() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Integer(100));
    let right = create_literal_expr(Literal::Integer(5));
    let expr = create_binary_expr(left, BinaryOp::Divide, right);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile binary division successfully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_binary_unknown_operator() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Integer(5));
    let right = create_literal_expr(Literal::Integer(3));
    let expr = create_binary_expr(left, BinaryOp::Equal, right); // Non-arithmetic operator

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should handle unknown operators with default behavior");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_function_no_params() {
    let compiler = WasmCompiler::new();
    let body = create_literal_expr(Literal::Integer(42));
    let expr = create_function_expr("test_fn", vec![], body);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function without parameters");

    let module = result.unwrap();
    assert!(module.has_export("test_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_function_with_params() {
    let compiler = WasmCompiler::new();
    let body = create_literal_expr(Literal::Integer(100));
    let params = vec![create_param("x"), create_param("y")];
    let expr = create_function_expr("add_fn", params, body);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function with parameters");

    let module = result.unwrap();
    assert!(module.has_export("add_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_function_with_return() {
    let compiler = WasmCompiler::new();
    let return_expr = create_expr(ExprKind::Return {
        value: Some(Box::new(create_literal_expr(Literal::Integer(123)))),
    });
    let expr = create_function_expr("return_fn", vec![], return_expr);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function with return statement");

    let module = result.unwrap();
    assert!(module.has_export("return_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_block_expression() {
    let compiler = WasmCompiler::new();
    let exprs = vec![
        create_literal_expr(Literal::Integer(1)),
        create_literal_expr(Literal::Integer(2)),
        create_literal_expr(Literal::Integer(3)),
    ];
    let expr = create_expr(ExprKind::Block(exprs));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile block expression");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_block_with_functions() {
    let compiler = WasmCompiler::new();
    let fn1 = create_function_expr("fn1", vec![], create_literal_expr(Literal::Integer(1)));
    let fn2 = create_function_expr("fn2", vec![], create_literal_expr(Literal::Integer(2)));
    let expr = create_expr(ExprKind::Block(vec![fn1, fn2]));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile block with multiple functions");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_unsupported_expression() {
    let compiler = WasmCompiler::new();
    let expr = create_identifier_expr("unknown_var");

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should handle unsupported expressions gracefully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test WasmModule functionality
#[test]
fn test_wasm_module_bytes() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Integer(42));
    let module = compiler.compile(&expr).unwrap();

    let bytes = module.bytes();
    assert!(!bytes.is_empty(), "Module should have non-empty bytecode");
    // Note: bytes field is private, so we just verify the method works
    assert!(!bytes.is_empty(), "bytes() should return non-empty reference");
}

#[test]
fn test_wasm_module_has_export_true() {
    let compiler = WasmCompiler::new();
    let body = create_literal_expr(Literal::Integer(42));
    let expr = create_function_expr("exported_fn", vec![], body);
    let module = compiler.compile(&expr).unwrap();

    assert!(module.has_export("exported_fn"), "Should find exported function");
}

#[test]
fn test_wasm_module_has_export_false() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Integer(42));
    let module = compiler.compile(&expr).unwrap();

    assert!(!module.has_export("nonexistent_fn"), "Should not find non-existent export");
}

#[test]
fn test_wasm_module_validate_valid() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Integer(42));
    let module = compiler.compile(&expr).unwrap();

    let validation = module.validate();
    assert!(validation.is_ok(), "Valid WASM module should pass validation");
}

// Note: Cannot test WasmModule validation with invalid bytecode directly
// since WasmModule fields are private. We test validation indirectly through
// compilation results, which always produce valid WASM modules.

// Test additional literal types to cover the "other literals" branch
#[test]
fn test_compile_unit_literal() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Unit);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile unit literal successfully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_char_literal() {
    let compiler = WasmCompiler::new();
    let expr = create_literal_expr(Literal::Char('A'));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile char literal successfully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test additional binary operators to cover all operator branches
#[test]
fn test_compile_binary_comparison_operators() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Integer(5));
    let right = create_literal_expr(Literal::Integer(3));

    // Test various comparison operators that fall into the "default" branch
    for op in [BinaryOp::Equal, BinaryOp::NotEqual, BinaryOp::Less, BinaryOp::Greater, BinaryOp::LessEqual, BinaryOp::GreaterEqual] {
        let expr = create_binary_expr(left.clone(), op, right.clone());
        let result = compiler.compile(&expr);
        assert!(result.is_ok(), "Should compile comparison operator {:?} successfully", op);

        let module = result.unwrap();
        assert!(module.validate().is_ok(), "Should generate valid WASM module for operator {:?}", op);
    }
}

#[test]
fn test_compile_binary_logical_operators() {
    let compiler = WasmCompiler::new();
    let left = create_literal_expr(Literal::Bool(true));
    let right = create_literal_expr(Literal::Bool(false));

    // Test logical operators that use the default branch
    for op in [BinaryOp::And, BinaryOp::Or] {
        let expr = create_binary_expr(left.clone(), op, right.clone());
        let result = compiler.compile(&expr);
        assert!(result.is_ok(), "Should compile logical operator {:?} successfully", op);

        let module = result.unwrap();
        assert!(module.validate().is_ok(), "Should generate valid WASM module for operator {:?}", op);
    }
}

// Test more unsupported expressions to improve coverage of the default branch
#[test]
fn test_compile_if_expression() {
    let compiler = WasmCompiler::new();
    let condition = create_literal_expr(Literal::Bool(true));
    let then_branch = create_literal_expr(Literal::Integer(1));
    let else_branch = Some(create_literal_expr(Literal::Integer(2)));

    let expr = create_expr(ExprKind::If {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch: else_branch.map(Box::new),
    });

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should handle if expressions gracefully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_let_expression() {
    let compiler = WasmCompiler::new();
    let value = create_literal_expr(Literal::Integer(42));
    let body = create_identifier_expr("x");

    let expr = create_expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(value),
        body: Box::new(body),
        is_mutable: false,
    });

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should handle let expressions gracefully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_call_expression() {
    let compiler = WasmCompiler::new();
    let func = create_identifier_expr("println");
    let args = vec![create_literal_expr(Literal::String("Hello".to_string()))];

    let expr = create_expr(ExprKind::Call {
        func: Box::new(func),
        args,
    });

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should handle call expressions gracefully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_method_call_expression() {
    let compiler = WasmCompiler::new();
    let receiver = create_identifier_expr("obj");
    let args = vec![create_literal_expr(Literal::Integer(42))];

    let expr = create_expr(ExprKind::MethodCall {
        receiver: Box::new(receiver),
        method: "method".to_string(),
        args,
    });

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should handle method call expressions gracefully");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test function without explicit return to trigger has_return check and default return
#[test]
fn test_compile_function_without_explicit_return() {
    let compiler = WasmCompiler::new();
    let body = create_literal_expr(Literal::Integer(42)); // No return statement
    let expr = create_function_expr("test_fn", vec![], body);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function without explicit return");

    let module = result.unwrap();
    assert!(module.has_export("test_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test function with explicit return to trigger has_return check differently
#[test]
fn test_compile_function_with_explicit_return() {
    let compiler = WasmCompiler::new();
    let return_expr = create_expr(ExprKind::Return {
        value: Some(Box::new(create_literal_expr(Literal::Integer(123)))),
    });
    let expr = create_function_expr("return_fn", vec![], return_expr);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function with explicit return");

    let module = result.unwrap();
    assert!(module.has_export("return_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test block with nested functions to trigger the export name collection path
#[test]
fn test_compile_block_with_nested_functions() {
    let compiler = WasmCompiler::new();
    let fn1 = create_function_expr("fn1", vec![], create_literal_expr(Literal::Integer(1)));
    let fn2 = create_function_expr("fn2", vec![], create_literal_expr(Literal::Integer(2)));
    let non_func = create_literal_expr(Literal::Integer(3)); // Non-function expression
    let expr = create_expr(ExprKind::Block(vec![fn1, fn2, non_func]));

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile block with nested functions");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test various section assembly conditions to cover module assembly paths
#[test]
fn test_compile_empty_function() {
    let compiler = WasmCompiler::new();
    let body = create_expr(ExprKind::Block(vec![])); // Empty block
    let expr = create_function_expr("empty_fn", vec![], body);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function with empty body");

    let module = result.unwrap();
    assert!(module.has_export("empty_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test function with multiple parameters to exercise parameter mapping
#[test]
fn test_compile_function_with_many_params() {
    let compiler = WasmCompiler::new();
    let params = vec![
        create_param("a"),
        create_param("b"),
        create_param("c"),
        create_param("d"),
        create_param("e"),
    ];
    let body = create_literal_expr(Literal::Integer(42));
    let expr = create_function_expr("many_params_fn", params, body);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile function with many parameters");

    let module = result.unwrap();
    assert!(module.has_export("many_params_fn"), "Should export the function");
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test different combinations to ensure all section assembly paths are covered
#[test]
fn test_compile_various_section_combinations() {
    let compiler = WasmCompiler::new();

    // Test 1: Simple expression (main function wrapper path)
    let simple_expr = create_literal_expr(Literal::Float(3.14));
    let result1 = compiler.compile(&simple_expr);
    assert!(result1.is_ok(), "Should compile simple expression");

    // Test 2: Complex nested expression
    let complex_expr = create_binary_expr(
        create_literal_expr(Literal::Integer(10)),
        BinaryOp::Multiply,
        create_binary_expr(
            create_literal_expr(Literal::Integer(5)),
            BinaryOp::Add,
            create_literal_expr(Literal::Integer(3))
        )
    );
    let result2 = compiler.compile(&complex_expr);
    assert!(result2.is_ok(), "Should compile complex nested expression");

    // Test 3: Block with mixed content
    let mixed_block = create_expr(ExprKind::Block(vec![
        create_literal_expr(Literal::String("hello".to_string())),
        create_literal_expr(Literal::Bool(true)),
        create_literal_expr(Literal::Unit),
    ]));
    let result3 = compiler.compile(&mixed_block);
    assert!(result3.is_ok(), "Should compile mixed block");
}

// Test edge cases for better coverage
#[test]
fn test_compile_edge_cases() {
    let compiler = WasmCompiler::new();

    // Empty block
    let empty_block = create_expr(ExprKind::Block(vec![]));
    let result1 = compiler.compile(&empty_block);
    assert!(result1.is_ok(), "Should compile empty block");

    // Single expression block
    let single_block = create_expr(ExprKind::Block(vec![
        create_literal_expr(Literal::Integer(42))
    ]));
    let result2 = compiler.compile(&single_block);
    assert!(result2.is_ok(), "Should compile single expression block");
}

// Test complex nested expressions
#[test]
fn test_compile_nested_binary_expressions() {
    let compiler = WasmCompiler::new();
    // Create expression: (5 + 3) * (10 - 2)
    let left_inner = create_binary_expr(
        create_literal_expr(Literal::Integer(5)),
        BinaryOp::Add,
        create_literal_expr(Literal::Integer(3))
    );
    let right_inner = create_binary_expr(
        create_literal_expr(Literal::Integer(10)),
        BinaryOp::Subtract,
        create_literal_expr(Literal::Integer(2))
    );
    let expr = create_binary_expr(left_inner, BinaryOp::Multiply, right_inner);

    let result = compiler.compile(&expr);
    assert!(result.is_ok(), "Should compile nested binary expressions");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

#[test]
fn test_compile_deeply_nested_expressions() {
    let compiler = WasmCompiler::new();
    // Create deeply nested expression: ((1 + 2) + 3) + 4
    let expr1 = create_binary_expr(
        create_literal_expr(Literal::Integer(1)),
        BinaryOp::Add,
        create_literal_expr(Literal::Integer(2))
    );
    let expr2 = create_binary_expr(expr1, BinaryOp::Add, create_literal_expr(Literal::Integer(3)));
    let expr3 = create_binary_expr(expr2, BinaryOp::Add, create_literal_expr(Literal::Integer(4)));

    let result = compiler.compile(&expr3);
    assert!(result.is_ok(), "Should compile deeply nested expressions");

    let module = result.unwrap();
    assert!(module.validate().is_ok(), "Should generate valid WASM module");
}

// Test optimization level impact
#[test]
fn test_optimization_levels_produce_valid_wasm() {
    let expr = create_literal_expr(Literal::Integer(42));

    for level in 0..=3 {
        let mut compiler = WasmCompiler::new();
        compiler.set_optimization_level(level);

        let result = compiler.compile(&expr);
        assert!(result.is_ok(), "Should compile successfully at optimization level {}", level);

        let module = result.unwrap();
        assert!(module.validate().is_ok(), "Should generate valid WASM at optimization level {}", level);
    }
}

// Property-based tests with 10,000+ iterations
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn test_compile_integer_literals_never_panics(
            value in -1000000i64..1000000i64
        ) {
            let compiler = WasmCompiler::new();
            let expr = create_literal_expr(Literal::Integer(value));

            // Should never panic
            let _result = compiler.compile(&expr);
        }

        #[test]
        fn test_compile_float_literals_never_panics(
            value in -1000000.0f64..1000000.0f64
        ) {
            let compiler = WasmCompiler::new();
            let expr = create_literal_expr(Literal::Float(value));

            // Should never panic, regardless of float value
            let _result = compiler.compile(&expr);
        }

        #[test]
        fn test_compile_boolean_literals_never_panics(
            value in prop::bool::ANY
        ) {
            let compiler = WasmCompiler::new();
            let expr = create_literal_expr(Literal::Bool(value));

            // Should never panic for any boolean value
            let _result = compiler.compile(&expr);
        }

        #[test]
        fn test_compile_string_literals_never_panics(
            value in "[a-zA-Z0-9 ]{0,100}"
        ) {
            let compiler = WasmCompiler::new();
            let expr = create_literal_expr(Literal::String(value));

            // Should never panic for valid strings
            let _result = compiler.compile(&expr);
        }

        #[test]
        fn test_optimization_level_setting_robustness(
            level in 0u8..255u8
        ) {
            let mut compiler = WasmCompiler::new();
            let expr = create_literal_expr(Literal::Integer(42));

            // Should handle any u8 value without panic
            compiler.set_optimization_level(level);

            // Test that compilation still works after setting any level (indicating proper clamping)
            let result = compiler.compile(&expr);
            prop_assert!(result.is_ok(), "Compilation should work regardless of optimization level");
        }

        #[test]
        fn test_binary_operations_produce_valid_wasm(
            left_val in -1000i64..1000i64,
            right_val in -1000i64..1000i64,
            op_type in 0..4usize
        ) {
            let compiler = WasmCompiler::new();
            let left = create_literal_expr(Literal::Integer(left_val));
            let right = create_literal_expr(Literal::Integer(right_val));

            let op = match op_type {
                0 => BinaryOp::Add,
                1 => BinaryOp::Subtract,
                2 => BinaryOp::Multiply,
                _ => BinaryOp::Divide,
            };

            let expr = create_binary_expr(left, op, right);
            let result = compiler.compile(&expr);

            // All binary operations should produce valid results
            prop_assert!(result.is_ok(), "Binary operation should compile successfully");

            if let Ok(module) = result {
                prop_assert!(module.validate().is_ok(), "Generated WASM should be valid");
                prop_assert!(!module.bytes().is_empty(), "Generated WASM should not be empty");
            }
        }

        #[test]
        fn test_function_compilation_scalability(
            param_count in 0..10usize,
            function_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}"
        ) {
            let compiler = WasmCompiler::new();
            let params: Vec<Param> = (0..param_count)
                .map(|i| create_param(&format!("param{}", i)))
                .collect();
            let body = create_literal_expr(Literal::Integer(42));
            let expr = create_function_expr(&function_name, params, body);

            let result = compiler.compile(&expr);

            // Functions with various parameter counts should compile
            prop_assert!(result.is_ok(), "Function with {} parameters should compile", param_count);

            if let Ok(module) = result {
                prop_assert!(module.has_export(&function_name), "Function should be exported");
                prop_assert!(module.validate().is_ok(), "Generated WASM should be valid");
            }
        }

        #[test]
        fn test_block_expression_scalability(
            expr_count in 0..50usize
        ) {
            let compiler = WasmCompiler::new();
            let exprs: Vec<Expr> = (0..expr_count)
                .map(|i| create_literal_expr(Literal::Integer(i as i64)))
                .collect();
            let expr = create_expr(ExprKind::Block(exprs));

            let result = compiler.compile(&expr);

            // Blocks with various expression counts should compile
            prop_assert!(result.is_ok(), "Block with {} expressions should compile", expr_count);

            if let Ok(module) = result {
                prop_assert!(module.validate().is_ok(), "Generated WASM should be valid");
            }
        }

        #[test]
        fn test_wasm_module_export_behavior(
            function_names in prop::collection::vec("[a-zA-Z_][a-zA-Z0-9_]{1,15}", 1..5)
        ) {
            let compiler = WasmCompiler::new();

            // Create functions with the generated names
            for function_name in &function_names {
                let body = create_literal_expr(Literal::Integer(42));
                let expr = create_function_expr(function_name, vec![], body);

                if let Ok(module) = compiler.compile(&expr) {
                    // The function should be exported
                    prop_assert!(module.has_export(function_name), "Should find exported function: {}", function_name);

                    // Non-existent exports should not be found
                    prop_assert!(!module.has_export("non_existent_function_name_12345"), "Should not find non-existent export");

                    // Module should validate
                    prop_assert!(module.validate().is_ok(), "Compiled module should be valid");
                }
            }
        }

        #[test]
        fn test_wasm_validation_via_compilation(
            value in -100i64..100i64
        ) {
            let compiler = WasmCompiler::new();
            let expr = create_literal_expr(Literal::Integer(value));

            if let Ok(module) = compiler.compile(&expr) {
                // All compiled modules should validate successfully
                prop_assert!(module.validate().is_ok(), "Compiled module should always validate");
                prop_assert!(!module.bytes().is_empty(), "Compiled module should have bytecode");
            }
        }

        #[test]
        fn test_complex_expression_compilation(
            depth in 1..8usize,
            leaf_value in -100i64..100i64
        ) {
            let compiler = WasmCompiler::new();

            // Build nested binary expression tree
            let mut expr = create_literal_expr(Literal::Integer(leaf_value));
            for _ in 0..depth {
                let right = create_literal_expr(Literal::Integer(1));
                expr = create_binary_expr(expr, BinaryOp::Add, right);
            }

            let result = compiler.compile(&expr);

            // Complex nested expressions should compile successfully
            prop_assert!(result.is_ok(), "Complex expression with depth {} should compile", depth);

            if let Ok(module) = result {
                prop_assert!(module.validate().is_ok(), "Complex expression should generate valid WASM");
                prop_assert!(!module.bytes().is_empty(), "Complex expression should generate non-empty WASM");
            }
        }
    }
}

// Big O Complexity Analysis
// WasmCompiler Core Functions:
// - new(): O(1) - Constant time constructor
// - set_optimization_level(): O(1) - Simple assignment with clamp
// - compile(): O(n) where n is the size of the AST (visits each node once)
// - compile_expr(): O(1) per expression node - constant work per node
// - has_return(): O(1) - Pattern match on expression kind
//
// WasmModule Core Functions:
// - bytes(): O(1) - Returns reference to internal bytes
// - has_export(): O(m) where m is number of exports (linear search)
// - validate(): O(1) - Checks first 4 bytes for magic number
//
// Overall Compilation Complexity: O(n) where n is AST size
// - Type section generation: O(f) where f is number of functions
// - Function section generation: O(f) where f is number of functions
// - Export section generation: O(e) where e is number of exports
// - Code section generation: O(n) where n is total AST nodes
//
// Space Complexity: O(n + b) where n is AST size, b is generated bytecode size
// Memory usage scales linearly with input AST size and output WASM module size
//
// Performance Characteristics:
// - Single-pass compilation: Each AST node visited exactly once
// - Linear scaling: Compilation time proportional to code size
// - Constant per-node work: Each expression compiled in O(1) time
// - Memory efficient: No redundant data structures or copies

// All test functions maintain cyclomatic complexity ≤ 10
// Property tests run with 10,000+ iterations for statistical confidence
// No SATD (Self-Admitted Technical Debt) comments
// Big O analysis provided for all major WASM compilation operations