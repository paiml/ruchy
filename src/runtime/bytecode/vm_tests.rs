//! Comprehensive tests for the bytecode VM
//!
//! EXTREME TDD Round 87: Comprehensive tests for bytecode VM
//! Coverage target: 95% for vm.rs module
//!
//! Tests organized by opcode category with edge cases.

#[cfg(test)]
mod tests {
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, ObjectField, Span, UnaryOp};
    use crate::runtime::bytecode::{BytecodeChunk, Compiler, Instruction, OpCode, VM};
    use crate::runtime::Value;

    // ============== Helper Functions ==============

    fn make_int_expr(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Span::default(),
        )
    }

    fn make_float_expr(val: f64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Float(val)), Span::default())
    }

    fn make_bool_expr(val: bool) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Bool(val)), Span::default())
    }

    fn make_string_expr(val: &str) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::String(val.to_string())),
            Span::default(),
        )
    }

    fn make_nil_expr() -> Expr {
        Expr::new(ExprKind::Literal(Literal::Unit), Span::default())
    }

    fn make_binary_expr(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::new(
            ExprKind::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        )
    }

    fn make_unary_expr(op: UnaryOp, operand: Expr) -> Expr {
        Expr::new(
            ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            Span::default(),
        )
    }

    fn compile_and_execute(expr: &Expr) -> Result<Value, String> {
        let mut compiler = Compiler::new("test".to_string());
        compiler.compile_expr(expr)?;
        let chunk = compiler.finalize();
        let mut vm = VM::new();
        vm.execute(&chunk)
    }

    // ============== Literal Tests ==============

    #[test]
    fn test_vm_integer_literal_zero() {
        let expr = make_int_expr(0);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_vm_integer_literal_negative() {
        let expr = make_int_expr(-42);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_vm_integer_literal_large() {
        let expr = make_int_expr(i64::MAX);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(i64::MAX));
    }

    #[test]
    fn test_vm_float_literal() {
        let expr = make_float_expr(3.14159);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - 3.14159).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_float_literal_negative() {
        let expr = make_float_expr(-2.71828);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - (-2.71828)).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_bool_true() {
        let expr = make_bool_expr(true);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_bool_false() {
        let expr = make_bool_expr(false);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_string_literal() {
        let expr = make_string_expr("hello");
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_vm_string_literal_empty() {
        let expr = make_string_expr("");
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), ""),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_vm_nil_literal() {
        let expr = make_nil_expr();
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Nil);
    }

    // ============== Arithmetic Tests ==============

    #[test]
    fn test_vm_add_integers() {
        let expr = make_binary_expr(BinaryOp::Add, make_int_expr(10), make_int_expr(32));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_add_negative() {
        let expr = make_binary_expr(BinaryOp::Add, make_int_expr(-10), make_int_expr(52));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_add_floats() {
        let expr = make_binary_expr(BinaryOp::Add, make_float_expr(1.5), make_float_expr(2.5));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - 4.0).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_subtract_integers() {
        let expr = make_binary_expr(BinaryOp::Subtract, make_int_expr(50), make_int_expr(8));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_subtract_negative_result() {
        let expr = make_binary_expr(BinaryOp::Subtract, make_int_expr(10), make_int_expr(20));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(-10));
    }

    #[test]
    fn test_vm_multiply_integers() {
        let expr = make_binary_expr(BinaryOp::Multiply, make_int_expr(6), make_int_expr(7));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_multiply_by_zero() {
        let expr = make_binary_expr(BinaryOp::Multiply, make_int_expr(42), make_int_expr(0));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_vm_multiply_negative() {
        let expr = make_binary_expr(BinaryOp::Multiply, make_int_expr(-6), make_int_expr(7));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_vm_divide_integers() {
        let expr = make_binary_expr(BinaryOp::Divide, make_int_expr(84), make_int_expr(2));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_divide_floats() {
        let expr = make_binary_expr(BinaryOp::Divide, make_float_expr(10.0), make_float_expr(4.0));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - 2.5).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_modulo_integers() {
        let expr = make_binary_expr(BinaryOp::Modulo, make_int_expr(47), make_int_expr(5));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_vm_modulo_zero_result() {
        let expr = make_binary_expr(BinaryOp::Modulo, make_int_expr(42), make_int_expr(7));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    // ============== Unary Operation Tests ==============

    #[test]
    fn test_vm_negate_integer() {
        let expr = make_unary_expr(UnaryOp::Negate, make_int_expr(42));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_vm_negate_negative() {
        let expr = make_unary_expr(UnaryOp::Negate, make_int_expr(-42));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_negate_float() {
        let expr = make_unary_expr(UnaryOp::Negate, make_float_expr(3.14));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - (-3.14)).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_not_true() {
        let expr = make_unary_expr(UnaryOp::Not, make_bool_expr(true));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_not_false() {
        let expr = make_unary_expr(UnaryOp::Not, make_bool_expr(false));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_bitnot_integer() {
        let expr = make_unary_expr(UnaryOp::BitwiseNot, make_int_expr(0));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(-1));
    }

    // ============== Comparison Tests ==============

    #[test]
    fn test_vm_equal_integers_true() {
        let expr = make_binary_expr(BinaryOp::Equal, make_int_expr(42), make_int_expr(42));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_equal_integers_false() {
        let expr = make_binary_expr(BinaryOp::Equal, make_int_expr(42), make_int_expr(43));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_not_equal_true() {
        let expr = make_binary_expr(BinaryOp::NotEqual, make_int_expr(42), make_int_expr(43));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_not_equal_false() {
        let expr = make_binary_expr(BinaryOp::NotEqual, make_int_expr(42), make_int_expr(42));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_less_than_true() {
        let expr = make_binary_expr(BinaryOp::Less, make_int_expr(10), make_int_expr(20));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_less_than_false() {
        let expr = make_binary_expr(BinaryOp::Less, make_int_expr(20), make_int_expr(10));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_less_than_equal() {
        let expr = make_binary_expr(BinaryOp::Less, make_int_expr(10), make_int_expr(10));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_less_equal_true() {
        let expr = make_binary_expr(BinaryOp::LessEqual, make_int_expr(10), make_int_expr(10));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_less_equal_false() {
        let expr = make_binary_expr(BinaryOp::LessEqual, make_int_expr(20), make_int_expr(10));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_greater_than_true() {
        let expr = make_binary_expr(BinaryOp::Greater, make_int_expr(20), make_int_expr(10));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_greater_than_false() {
        let expr = make_binary_expr(BinaryOp::Greater, make_int_expr(10), make_int_expr(20));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_greater_equal_true() {
        let expr = make_binary_expr(BinaryOp::GreaterEqual, make_int_expr(10), make_int_expr(10));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_greater_equal_false() {
        let expr = make_binary_expr(BinaryOp::GreaterEqual, make_int_expr(10), make_int_expr(20));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    // ============== Logical Operation Tests ==============

    #[test]
    fn test_vm_and_true_true() {
        let expr = make_binary_expr(BinaryOp::And, make_bool_expr(true), make_bool_expr(true));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_and_true_false() {
        let expr = make_binary_expr(BinaryOp::And, make_bool_expr(true), make_bool_expr(false));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_and_false_true() {
        let expr = make_binary_expr(BinaryOp::And, make_bool_expr(false), make_bool_expr(true));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_and_false_false() {
        let expr = make_binary_expr(BinaryOp::And, make_bool_expr(false), make_bool_expr(false));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_or_true_true() {
        let expr = make_binary_expr(BinaryOp::Or, make_bool_expr(true), make_bool_expr(true));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_or_true_false() {
        let expr = make_binary_expr(BinaryOp::Or, make_bool_expr(true), make_bool_expr(false));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_or_false_true() {
        let expr = make_binary_expr(BinaryOp::Or, make_bool_expr(false), make_bool_expr(true));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_or_false_false() {
        let expr = make_binary_expr(BinaryOp::Or, make_bool_expr(false), make_bool_expr(false));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    // ============== Complex Expression Tests ==============

    #[test]
    fn test_vm_nested_arithmetic() {
        // (10 + 20) * 2 = 60
        let inner = make_binary_expr(BinaryOp::Add, make_int_expr(10), make_int_expr(20));
        let expr = make_binary_expr(BinaryOp::Multiply, inner, make_int_expr(2));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(60));
    }

    #[test]
    fn test_vm_deeply_nested() {
        // ((1 + 2) * (3 + 4)) - 5 = 16
        let left = make_binary_expr(BinaryOp::Add, make_int_expr(1), make_int_expr(2));
        let right = make_binary_expr(BinaryOp::Add, make_int_expr(3), make_int_expr(4));
        let mul = make_binary_expr(BinaryOp::Multiply, left, right);
        let expr = make_binary_expr(BinaryOp::Subtract, mul, make_int_expr(5));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(16));
    }

    #[test]
    fn test_vm_comparison_chain() {
        // (10 < 20) && (20 < 30) = true
        let left = make_binary_expr(BinaryOp::Less, make_int_expr(10), make_int_expr(20));
        let right = make_binary_expr(BinaryOp::Less, make_int_expr(20), make_int_expr(30));
        let expr = make_binary_expr(BinaryOp::And, left, right);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_mixed_arithmetic_comparison() {
        // (10 + 20) == 30
        let sum = make_binary_expr(BinaryOp::Add, make_int_expr(10), make_int_expr(20));
        let expr = make_binary_expr(BinaryOp::Equal, sum, make_int_expr(30));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    // ============== Array Tests ==============

    #[test]
    fn test_vm_array_literal() {
        let expr = Expr::new(
            ExprKind::List(vec![make_int_expr(1), make_int_expr(2), make_int_expr(3)]),
            Span::default(),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Integer(2));
                assert_eq!(arr[2], Value::Integer(3));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_vm_empty_array() {
        let expr = Expr::new(ExprKind::List(vec![]), Span::default());
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_vm_array_mixed_types() {
        let expr = Expr::new(
            ExprKind::List(vec![
                make_int_expr(42),
                make_string_expr("hello"),
                make_bool_expr(true),
            ]),
            Span::default(),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(42));
            }
            _ => panic!("Expected array"),
        }
    }

    // ============== Tuple Tests ==============

    #[test]
    fn test_vm_tuple_literal() {
        let expr = Expr::new(
            ExprKind::Tuple(vec![make_int_expr(1), make_int_expr(2)]),
            Span::default(),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 2);
                assert_eq!(t[0], Value::Integer(1));
                assert_eq!(t[1], Value::Integer(2));
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_vm_tuple_single_element() {
        let expr = Expr::new(ExprKind::Tuple(vec![make_int_expr(42)]), Span::default());
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 1);
                assert_eq!(t[0], Value::Integer(42));
            }
            _ => panic!("Expected tuple"),
        }
    }

    // ============== VM Struct Tests ==============

    #[test]
    fn test_vm_new() {
        let mut vm = VM::new();
        // VM should be created with default state
        assert!(vm.execute(&BytecodeChunk::new("test".to_string())).is_ok());
    }

    #[test]
    fn test_vm_default() {
        let mut vm = VM::default();
        assert!(vm.execute(&BytecodeChunk::new("test".to_string())).is_ok());
    }

    #[test]
    fn test_vm_empty_chunk() {
        let mut vm = VM::new();
        let chunk = BytecodeChunk::new("empty".to_string());
        let result = vm.execute(&chunk).expect("execution should succeed");
        assert_eq!(result, Value::Nil);
    }

    // ============== CallFrame Tests ==============

    #[test]
    fn test_bytecode_chunk_add_constant() {
        let mut chunk = BytecodeChunk::new("test".to_string());
        let idx1 = chunk.add_constant(Value::Integer(42));
        let idx2 = chunk.add_constant(Value::Integer(42)); // Same value
        let idx3 = chunk.add_constant(Value::Integer(100));

        // Same values should return same index
        assert_eq!(idx1, idx2);
        // Different values should have different indices
        assert_ne!(idx1, idx3);
    }

    #[test]
    fn test_bytecode_chunk_emit() {
        let mut chunk = BytecodeChunk::new("test".to_string());
        let instruction = Instruction::abc(OpCode::Add, 0, 1, 2);
        let idx = chunk.emit(instruction, 10);
        assert_eq!(idx, 0);
        assert_eq!(chunk.instructions.len(), 1);
        assert_eq!(chunk.line_numbers[0], 10);
    }

    // ============== String Operation Tests ==============

    #[test]
    fn test_vm_string_concatenation() {
        let expr = make_binary_expr(
            BinaryOp::Add,
            make_string_expr("hello"),
            make_string_expr(" world"),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "hello world"),
            _ => panic!("Expected string"),
        }
    }

    // ============== Edge Case Tests ==============

    #[test]
    fn test_vm_double_negation() {
        let inner = make_unary_expr(UnaryOp::Negate, make_int_expr(42));
        let expr = make_unary_expr(UnaryOp::Negate, inner);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_double_not() {
        let inner = make_unary_expr(UnaryOp::Not, make_bool_expr(true));
        let expr = make_unary_expr(UnaryOp::Not, inner);
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_add_zero() {
        let expr = make_binary_expr(BinaryOp::Add, make_int_expr(42), make_int_expr(0));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_subtract_self() {
        let expr = make_binary_expr(BinaryOp::Subtract, make_int_expr(42), make_int_expr(42));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_vm_multiply_one() {
        let expr = make_binary_expr(BinaryOp::Multiply, make_int_expr(42), make_int_expr(1));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_divide_self() {
        let expr = make_binary_expr(BinaryOp::Divide, make_int_expr(42), make_int_expr(42));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Integer(1));
    }

    // ============== Float Arithmetic Tests ==============

    #[test]
    fn test_vm_float_subtract() {
        let expr = make_binary_expr(
            BinaryOp::Subtract,
            make_float_expr(10.5),
            make_float_expr(3.2),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - 7.3).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_float_multiply() {
        let expr = make_binary_expr(
            BinaryOp::Multiply,
            make_float_expr(2.5),
            make_float_expr(4.0),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - 10.0).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_vm_mixed_int_float_add() {
        // Int + Float should promote to Float
        let expr = make_binary_expr(BinaryOp::Add, make_int_expr(10), make_float_expr(0.5));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Float(f) => assert!((f - 10.5).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    // ============== Boolean Comparison Tests ==============

    #[test]
    fn test_vm_equal_bools() {
        let expr = make_binary_expr(BinaryOp::Equal, make_bool_expr(true), make_bool_expr(true));
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_equal_strings() {
        let expr = make_binary_expr(
            BinaryOp::Equal,
            make_string_expr("hello"),
            make_string_expr("hello"),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_not_equal_strings() {
        let expr = make_binary_expr(
            BinaryOp::NotEqual,
            make_string_expr("hello"),
            make_string_expr("world"),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    // ============== Object Tests ==============

    #[test]
    fn test_vm_object_literal() {
        let expr = Expr::new(
            ExprKind::ObjectLiteral {
                fields: vec![
                    ObjectField::KeyValue {
                        key: "x".to_string(),
                        value: make_int_expr(10),
                    },
                    ObjectField::KeyValue {
                        key: "y".to_string(),
                        value: make_int_expr(20),
                    },
                ],
            },
            Span::default(),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Object(map) => {
                assert_eq!(map.get("x"), Some(&Value::Integer(10)));
                assert_eq!(map.get("y"), Some(&Value::Integer(20)));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_vm_empty_object() {
        let expr = Expr::new(
            ExprKind::ObjectLiteral { fields: vec![] },
            Span::default(),
        );
        let result = compile_and_execute(&expr).expect("execution should succeed");
        match result {
            Value::Object(map) => assert!(map.is_empty()),
            _ => panic!("Expected object"),
        }
    }

    // ============== Instruction Format Tests ==============

    #[test]
    fn test_instruction_abc_format() {
        let instr = Instruction::abc(OpCode::Add, 0, 1, 2);
        assert_eq!(instr.opcode(), OpCode::Add as u8);
        assert_eq!(instr.get_a(), 0);
        assert_eq!(instr.get_b(), 1);
        assert_eq!(instr.get_c(), 2);
    }

    #[test]
    fn test_instruction_abx_format() {
        let instr = Instruction::abx(OpCode::Const, 5, 100);
        assert_eq!(instr.opcode(), OpCode::Const as u8);
        assert_eq!(instr.get_a(), 5);
        assert_eq!(instr.get_bx(), 100);
    }

    #[test]
    fn test_instruction_asbx_format() {
        let instr = Instruction::asbx(OpCode::Jump, 0, 10);
        assert_eq!(instr.opcode(), OpCode::Jump as u8);
        assert_eq!(instr.get_a(), 0);
        assert_eq!(instr.get_sbx(), 10);
    }

    #[test]
    fn test_instruction_asbx_negative() {
        let instr = Instruction::asbx(OpCode::Jump, 0, -5);
        assert_eq!(instr.get_sbx(), -5);
    }
}
