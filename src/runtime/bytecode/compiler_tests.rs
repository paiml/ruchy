    use super::*;

    #[test]
    fn test_constant_pool_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Integer(42));
        let idx2 = chunk.add_constant(Value::Integer(42));
        let idx3 = chunk.add_constant(Value::Integer(100));

        assert_eq!(idx1, idx2, "Duplicate constants should return same index");
        assert_ne!(
            idx1, idx3,
            "Different constants should have different indices"
        );
        assert_eq!(
            chunk.constants.len(),
            2,
            "Should only store 2 unique constants"
        );
    }

    #[test]
    fn test_register_allocator_basic() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let r1 = allocator.allocate();
        let r2 = allocator.allocate();

        assert_eq!(r0, 0);
        assert_eq!(r1, 1);
        assert_eq!(r2, 2);
        assert_eq!(allocator.max_count(), 3);
    }

    #[test]
    fn test_register_allocator_reuse() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let _r1 = allocator.allocate();

        allocator.free(r0);
        let r2 = allocator.allocate();

        assert_eq!(r2, r0, "Should reuse freed register");
        assert_eq!(allocator.max_count(), 2, "Max count shouldn't change");
    }

    #[test]
    fn test_compile_integer_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(result_reg, 0, "First expression should use register 0");
        assert_eq!(chunk.constants.len(), 1, "Should have 1 constant");
        assert_eq!(
            chunk.instructions.len(),
            2,
            "Should have CONST + RETURN instructions"
        );

        // Verify CONST instruction
        let const_instr = chunk.instructions[0];
        assert_eq!(const_instr.opcode(), OpCode::Const.to_u8());
        assert_eq!(const_instr.get_a(), 0, "Should load into register 0");
        assert_eq!(const_instr.get_bx(), 0, "Should load constant at index 0");
    }

    #[test]
    fn test_compile_binary_addition() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(32, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST (10), CONST (32), ADD, RETURN
        assert_eq!(chunk.instructions.len(), 4);
        assert_eq!(chunk.constants.len(), 2);

        // Verify ADD instruction
        let add_instr = chunk.instructions[2];
        assert_eq!(add_instr.opcode(), OpCode::Add.to_u8());
        assert_eq!(add_instr.get_a(), result_reg, "Result register");
    }

    #[test]
    fn test_compile_block() {
        let mut compiler = Compiler::new("test".to_string());

        // Block with 3 expressions: 1, 2, 3
        let exprs = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let block = Expr::new(
            ExprKind::Block(exprs),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&block).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST(1), CONST(2), CONST(3), RETURN
        assert_eq!(chunk.instructions.len(), 4);
        assert_eq!(chunk.constants.len(), 3, "Should have 3 constants");

        // Result should be the last expression (3)
        assert!(result_reg < 10, "Result register should be valid");
    }

    #[test]
    fn test_compile_if_with_else() {
        let mut compiler = Compiler::new("test".to_string());

        // if true { 42 } else { 0 }
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            crate::frontend::ast::Span::default(),
        );

        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&if_expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have:
        // CONST (true), JUMP_IF_FALSE, CONST (42), MOVE, JUMP, CONST (0), MOVE, RETURN
        assert!(
            chunk.instructions.len() >= 7,
            "Should have at least 7 instructions"
        );
        assert_eq!(chunk.constants.len(), 3, "Should have 3 constants");

        // Verify conditional jump exists
        let jump_if_false_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::JumpIfFalse.to_u8());
        assert!(jump_if_false_found, "Should have JumpIfFalse instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }

    #[test]
    fn test_compile_if_without_else() {
        let mut compiler = Compiler::new("test".to_string());

        // if true { 42 }
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );

        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&if_expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST (true), JUMP_IF_FALSE, CONST (42), MOVE, RETURN
        assert!(
            chunk.instructions.len() >= 4,
            "Should have at least 4 instructions"
        );

        // Verify conditional jump exists
        let jump_if_false_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::JumpIfFalse.to_u8());
        assert!(jump_if_false_found, "Should have JumpIfFalse instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }

    #[test]
    fn test_compile_function_call() {
        let mut compiler = Compiler::new("test".to_string());

        // foo(1, 2)
        let func = Expr::new(
            ExprKind::Identifier("foo".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let arg1 = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let arg2 = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            crate::frontend::ast::Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![arg1, arg2],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&call).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: LOAD_GLOBAL(foo), CONST(1), CONST(2), CALL, RETURN
        assert!(
            chunk.instructions.len() >= 5,
            "Should have at least 5 instructions"
        );

        // Verify CALL instruction exists
        let call_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Call.to_u8());
        assert!(call_found, "Should have Call instruction");

        assert!(result_reg < 10, "Result register should be valid");
    }

    // Test 10: BytecodeChunk creation and basic operations
    #[test]
    fn test_bytecode_chunk_new() {
        let chunk = BytecodeChunk::new("test_func".to_string());
        assert_eq!(chunk.name, "test_func");
        assert!(chunk.instructions.is_empty());
        assert!(chunk.constants.is_empty());
        assert_eq!(chunk.register_count, 0);
        assert_eq!(chunk.parameter_count, 0);
    }

    // Test 11: BytecodeChunk emit instruction
    #[test]
    fn test_bytecode_chunk_emit() {
        let mut chunk = BytecodeChunk::new("test".to_string());
        let instr = Instruction::abc(OpCode::Add, 0, 1, 2);
        let index = chunk.emit(instr, 10);

        assert_eq!(index, 0);
        assert_eq!(chunk.instructions.len(), 1);
        assert_eq!(chunk.line_numbers.len(), 1);
        assert_eq!(chunk.line_numbers[0], 10);
    }

    // Test 12: BytecodeChunk add_constant deduplication
    #[test]
    fn test_bytecode_chunk_constant_dedup() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Integer(42));
        let idx2 = chunk.add_constant(Value::Integer(42));
        let idx3 = chunk.add_constant(Value::Integer(100));

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 0, "Duplicate constant should return same index");
        assert_eq!(idx3, 1, "New constant should get new index");
        assert_eq!(chunk.constants.len(), 2);
    }

    // Test 13: RegisterAllocator multiple allocations
    #[test]
    fn test_register_allocator_multiple() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let r1 = allocator.allocate();
        let r2 = allocator.allocate();

        assert_eq!(r0, 0);
        assert_eq!(r1, 1);
        assert_eq!(r2, 2);
        assert_eq!(allocator.max_count(), 3);
    }

    // Test 14: RegisterAllocator free and reuse multiple
    #[test]
    fn test_register_allocator_free_multiple() {
        let mut allocator = RegisterAllocator::new();

        let r0 = allocator.allocate();
        let r1 = allocator.allocate();
        let _r2 = allocator.allocate();

        allocator.free(r1);
        allocator.free(r0);

        // Should reuse in LIFO order
        let r3 = allocator.allocate();
        let r4 = allocator.allocate();

        assert_eq!(r3, r0, "Should reuse r0 first (LIFO)");
        assert_eq!(r4, r1, "Should reuse r1 second");
        assert_eq!(allocator.max_count(), 3, "Max should remain 3");
    }

    // Test 15: Compile unary negation
    #[test]
    fn test_compile_unary_negate() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST(5), NEG, RETURN
        assert!(chunk.instructions.len() >= 2);

        // Verify NEG instruction exists
        let neg_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Neg.to_u8());
        assert!(neg_found, "Should have Neg instruction");

        assert!(result_reg < 10);
    }

    // Test 16: Compile unary not
    #[test]
    fn test_compile_unary_not() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have: CONST(true), NOT, RETURN
        assert!(chunk.instructions.len() >= 2);

        // Verify NOT instruction exists
        let not_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Not.to_u8());
        assert!(not_found, "Should have Not instruction");

        assert!(result_reg < 10);
    }

    // Test 17: Compile float literal
    #[test]
    fn test_compile_float_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(chunk.constants.len(), 1);
        match &chunk.constants[0] {
            Value::Float(f) => assert!((*f - 3.14).abs() < 0.001),
            _ => panic!("Expected float constant"),
        }
        assert!(result_reg < 10);
    }

    // Test 18: Compile string literal
    #[test]
    fn test_compile_string_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(chunk.constants.len(), 1);
        match &chunk.constants[0] {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string constant"),
        }
        assert!(result_reg < 10);
    }

    // Test 19: Compile boolean literal
    #[test]
    fn test_compile_bool_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert_eq!(chunk.constants.len(), 1);
        match &chunk.constants[0] {
            Value::Bool(b) => assert!(*b),
            _ => panic!("Expected bool constant"),
        }
        assert!(result_reg < 10);
    }

    // Test 20: Compile subtraction
    #[test]
    fn test_compile_binary_subtraction() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(50, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(8, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Verify SUB instruction exists
        let sub_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Sub.to_u8());
        assert!(sub_found, "Should have Sub instruction");
        assert!(result_reg < 10);
    }

    // Test 21: Compile multiplication
    #[test]
    fn test_compile_binary_multiplication() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(7, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(6, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Verify MUL instruction exists
        let mul_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Mul.to_u8());
        assert!(mul_found, "Should have Mul instruction");
        assert!(result_reg < 10);
    }

    // Test 22: Compile comparison equal
    #[test]
    fn test_compile_binary_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Verify EQUAL instruction exists
        let equal_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Equal.to_u8());
        assert!(equal_found, "Should have Equal instruction");
        assert!(result_reg < 10);
    }

    // Test 23: Compile list with literals (optimization path)
    #[test]
    fn test_compile_list_literals() {
        let mut compiler = Compiler::new("test".to_string());
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let expr = Expr::new(
            ExprKind::List(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Literal list optimization: array is created at compile-time in constant pool
        // Uses CONST instruction instead of NewArray
        assert!(chunk.constants.len() >= 1, "Should have array constant");
        assert!(result_reg < 10);
    }

    // Test 23b: Compile list with non-literals (NewArray path)
    #[test]
    fn test_compile_list_non_literals() {
        let mut compiler = Compiler::new("test".to_string());

        // Use identifier elements so it takes the NewArray path
        let elements = vec![
            Expr::new(
                ExprKind::Identifier("x".to_string()),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Identifier("y".to_string()),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let expr = Expr::new(
            ExprKind::List(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewArray instruction for non-literal elements
        let new_array_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewArray.to_u8());
        assert!(new_array_found, "Should have NewArray instruction");
        assert!(result_reg < 10);
    }

    // Test 24: Compile empty block
    #[test]
    fn test_compile_empty_block() {
        let mut compiler = Compiler::new("test".to_string());
        let block = Expr::new(
            ExprKind::Block(vec![]),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&block).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty block should produce nil
        assert_eq!(chunk.constants.len(), 1);
        assert!(result_reg < 10);
    }

    // Test 25: Compile while loop
    #[test]
    fn test_compile_while() {
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::While {
                label: None,
                condition: Box::new(condition),
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have JumpIfFalse and Jump instructions
        let jump_if_false_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::JumpIfFalse.to_u8());
        let jump_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Jump.to_u8());

        assert!(jump_if_false_found, "Should have JumpIfFalse instruction");
        assert!(jump_found, "Should have Jump instruction");
        assert!(result_reg < 10);
    }

    // ===== EXTREME TDD Round 116 - Additional Tests =====

    #[test]
    fn test_compile_boolean_true() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
    }

    #[test]
    fn test_compile_boolean_false() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
    }

    #[test]
    fn test_compile_float_literal_pi() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        // Verify constant is a float
        match &chunk.constants[0] {
            Value::Float(f) => assert!((*f - 3.14).abs() < 0.001),
            _ => panic!("Expected float constant"),
        }
    }

    #[test]
    fn test_compile_nil_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Null),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        // Should have nil constant
        assert!(chunk.constants.iter().any(|c| matches!(c, Value::Nil)));
    }

    #[test]
    fn test_register_allocator_many_allocations() {
        let mut allocator = RegisterAllocator::new();

        // Allocate many registers
        let registers: Vec<u8> = (0..100).map(|_| allocator.allocate()).collect();

        assert_eq!(allocator.max_count(), 100);
        assert_eq!(registers[0], 0);
        assert_eq!(registers[99], 99);
    }

    #[test]
    fn test_register_allocator_free_all_reuse() {
        let mut allocator = RegisterAllocator::new();

        // Allocate 10 registers
        let registers: Vec<u8> = (0..10).map(|_| allocator.allocate()).collect();

        // Free all in reverse order
        for r in registers.into_iter().rev() {
            allocator.free(r);
        }

        // Reallocate should reuse
        let r0 = allocator.allocate();
        assert_eq!(r0, 0);
        assert_eq!(allocator.max_count(), 10, "Max count should remain 10");
    }

    #[test]
    fn test_values_equal_integers() {
        assert!(values_equal(&Value::Integer(42), &Value::Integer(42)));
        assert!(!values_equal(&Value::Integer(42), &Value::Integer(43)));
    }

    #[test]
    fn test_values_equal_floats() {
        assert!(values_equal(&Value::Float(3.14), &Value::Float(3.14)));
        assert!(!values_equal(&Value::Float(3.14), &Value::Float(2.71)));
    }

    #[test]
    fn test_values_equal_bools() {
        assert!(values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(values_equal(&Value::Bool(false), &Value::Bool(false)));
        assert!(!values_equal(&Value::Bool(true), &Value::Bool(false)));
    }

    #[test]
    fn test_values_equal_nil() {
        assert!(values_equal(&Value::Nil, &Value::Nil));
    }

    #[test]
    fn test_values_equal_different_types() {
        assert!(!values_equal(&Value::Integer(42), &Value::Float(42.0)));
        assert!(!values_equal(&Value::Bool(true), &Value::Integer(1)));
        assert!(!values_equal(&Value::Nil, &Value::Integer(0)));
    }

    #[test]
    fn test_bytecode_chunk_name() {
        let chunk = BytecodeChunk::new("my_function".to_string());
        assert_eq!(chunk.name, "my_function");
        assert!(chunk.constants.is_empty());
        assert!(chunk.instructions.is_empty());
    }

    #[test]
    fn test_constant_pool_different_types() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let int_idx = chunk.add_constant(Value::Integer(42));
        let float_idx = chunk.add_constant(Value::Float(3.14));
        let bool_idx = chunk.add_constant(Value::Bool(true));
        let nil_idx = chunk.add_constant(Value::Nil);

        assert_ne!(int_idx, float_idx);
        assert_ne!(float_idx, bool_idx);
        assert_ne!(bool_idx, nil_idx);
        assert_eq!(chunk.constants.len(), 4);
    }

    // ===== EXTREME TDD Round 117 - Additional Coverage Tests =====

    // Helper to create a simple Param for tests
    fn make_test_param(name: &str) -> crate::frontend::ast::Param {
        crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty: crate::frontend::ast::Type {
                kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                span: crate::frontend::ast::Span::default(),
            },
            span: crate::frontend::ast::Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    // Helper to create a Param with default value
    fn make_test_param_with_default(name: &str, default: Expr) -> crate::frontend::ast::Param {
        crate::frontend::ast::Param {
            pattern: crate::frontend::ast::Pattern::Identifier(name.to_string()),
            ty: crate::frontend::ast::Type {
                kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                span: crate::frontend::ast::Span::default(),
            },
            span: crate::frontend::ast::Span::default(),
            is_mutable: false,
            default_value: Some(Box::new(default)),
        }
    }

    // Test literal types: Unit
    #[test]
    fn test_compile_unit_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Unit),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        assert!(chunk.constants.iter().any(|c| matches!(c, Value::Nil)));
    }

    // Test literal types: Char
    #[test]
    fn test_compile_char_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Char('x')),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        match &chunk.constants[0] {
            Value::String(s) => assert_eq!(s.as_ref(), "x"),
            _ => panic!("Expected string constant for char"),
        }
    }

    // Test literal types: Byte
    #[test]
    fn test_compile_byte_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Byte(255)),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        match &chunk.constants[0] {
            Value::Integer(i) => assert_eq!(*i, 255),
            _ => panic!("Expected integer constant for byte"),
        }
    }

    // Test literal types: Atom
    #[test]
    fn test_compile_atom_literal() {
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Atom("ok".to_string())),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(result_reg < 10);
        assert!(!chunk.constants.is_empty());
        match &chunk.constants[0] {
            Value::Atom(s) => assert_eq!(s.as_str(), "ok"),
            _ => panic!("Expected atom constant"),
        }
    }

    // Test binary operators: Division
    #[test]
    fn test_compile_binary_division() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Divide,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let div_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Div.to_u8());
        assert!(div_found, "Should have Div instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Modulo
    #[test]
    fn test_compile_binary_modulo() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(17, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Modulo,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let mod_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Mod.to_u8());
        assert!(mod_found, "Should have Mod instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: NotEqual
    #[test]
    fn test_compile_binary_not_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::NotEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let neq_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NotEqual.to_u8());
        assert!(neq_found, "Should have NotEqual instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Greater
    #[test]
    fn test_compile_binary_greater() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Greater,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let gt_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Greater.to_u8());
        assert!(gt_found, "Should have Greater instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: GreaterEqual
    #[test]
    fn test_compile_binary_greater_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::GreaterEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let ge_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::GreaterEqual.to_u8());
        assert!(ge_found, "Should have GreaterEqual instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Less
    #[test]
    fn test_compile_binary_less() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(3, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(7, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Less,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let lt_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Less.to_u8());
        assert!(lt_found, "Should have Less instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: LessEqual
    #[test]
    fn test_compile_binary_less_equal() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::LessEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let le_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LessEqual.to_u8());
        assert!(le_found, "Should have LessEqual instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: And
    #[test]
    fn test_compile_binary_and() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let and_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::And.to_u8());
        assert!(and_found, "Should have And instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: Or
    #[test]
    fn test_compile_binary_or() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let or_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Or.to_u8());
        assert!(or_found, "Should have Or instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: BitwiseAnd
    #[test]
    fn test_compile_binary_bitwise_and() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::BitwiseAnd,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let band_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitAnd.to_u8());
        assert!(band_found, "Should have BitAnd instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: BitwiseOr
    #[test]
    fn test_compile_binary_bitwise_or() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::BitwiseOr,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let bor_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitOr.to_u8());
        assert!(bor_found, "Should have BitOr instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: BitwiseXor
    #[test]
    fn test_compile_binary_bitwise_xor() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1100, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::BitwiseXor,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let bxor_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitXor.to_u8());
        assert!(bxor_found, "Should have BitXor instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: LeftShift
    #[test]
    fn test_compile_binary_left_shift() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(4, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::LeftShift,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let shl_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::ShiftLeft.to_u8());
        assert!(shl_found, "Should have ShiftLeft instruction");
        assert!(result_reg < 10);
    }

    // Test binary operators: RightShift
    #[test]
    fn test_compile_binary_right_shift() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(16, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::RightShift,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let shr_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::ShiftRight.to_u8());
        assert!(shr_found, "Should have ShiftRight instruction");
        assert!(result_reg < 10);
    }

    // Test unary operator: BitwiseNot
    #[test]
    fn test_compile_unary_bitwise_not() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Literal(Literal::Integer(0b1010, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::BitwiseNot,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let bitnot_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::BitNot.to_u8());
        assert!(bitnot_found, "Should have BitNot instruction");
        assert!(result_reg < 10);
    }

    // Test unsupported unary operator: Reference
    #[test]
    fn test_compile_unary_reference_error() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Reference,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "Reference operator should fail");
        assert!(result.unwrap_err().contains("Unsupported unary operator"));
    }

    // Test unsupported unary operator: MutableReference
    #[test]
    fn test_compile_unary_mutable_reference_error() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::MutableReference,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "MutableReference operator should fail");
        assert!(result.unwrap_err().contains("Unsupported unary operator"));
    }

    // Test unsupported unary operator: Deref
    #[test]
    fn test_compile_unary_deref_error() {
        let mut compiler = Compiler::new("test".to_string());
        let operand = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Deref,
                operand: Box::new(operand),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "Deref operator should fail");
        assert!(result.unwrap_err().contains("Unsupported unary operator"));
    }

    // Test let binding compilation
    #[test]
    fn test_compile_let_binding() {
        let mut compiler = Compiler::new("test".to_string());

        // let x = 42 in x
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        assert!(!chunk.local_names.is_empty(), "Should have local names");
        assert_eq!(chunk.local_names[0], "x");
        assert!(result_reg < 10);
    }

    // Test variable reference: local variable
    #[test]
    fn test_compile_variable_local() {
        let mut compiler = Compiler::new("test".to_string());

        // First, set up a local variable through let binding
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have MOVE instruction for copying local variable
        let move_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Move.to_u8());
        assert!(
            move_found,
            "Should have Move instruction for local variable access"
        );
        assert!(result_reg < 10);
    }

    // Test variable reference: global variable
    #[test]
    fn test_compile_variable_global() {
        let mut compiler = Compiler::new("test".to_string());

        // Reference a global variable (not defined locally)
        let expr = Expr::new(
            ExprKind::Identifier("global_var".to_string()),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have LOAD_GLOBAL instruction
        let load_global_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LoadGlobal.to_u8());
        assert!(load_global_found, "Should have LoadGlobal instruction");
        assert!(result_reg < 10);
    }

    // Test assignment to local variable
    #[test]
    fn test_compile_assignment() {
        let mut compiler = Compiler::new("test".to_string());

        // let mut x = 10 in { x = 20; x }
        let init_value = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let target = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let new_value = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            crate::frontend::ast::Span::default(),
        );
        let assign_expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(target.clone()),
                value: Box::new(new_value),
            },
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Block(vec![assign_expr, target]),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(init_value),
                body: Box::new(body),
                is_mutable: true,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have MOVE instructions for assignment
        let move_count = chunk
            .instructions
            .iter()
            .filter(|i| i.opcode() == OpCode::Move.to_u8())
            .count();
        assert!(
            move_count >= 1,
            "Should have Move instruction(s) for assignment"
        );
        assert!(result_reg < 10);
    }

    // Test assignment to undefined variable (error)
    #[test]
    fn test_compile_assignment_undefined_error() {
        let mut compiler = Compiler::new("test".to_string());

        let target = Expr::new(
            ExprKind::Identifier("undefined_var".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let assign_expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&assign_expr);
        assert!(
            result.is_err(),
            "Assignment to undefined variable should fail"
        );
        assert!(result.unwrap_err().contains("Undefined variable"));
    }

    // Test function definition
    #[test]
    fn test_compile_function_definition() {
        let mut compiler = Compiler::new("test".to_string());

        // fun add(a, b) { a + b }
        let param_a = make_test_param("a");
        let param_b = make_test_param("b");

        let a = Expr::new(
            ExprKind::Identifier("a".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let b = Expr::new(
            ExprKind::Identifier("b".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(a),
                right: Box::new(b),
            },
            crate::frontend::ast::Span::default(),
        );

        let func_expr = Expr::new(
            ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![param_a, param_b],
                return_type: None,
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&func_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Function should be stored in locals
        assert!(
            chunk.local_names.contains(&"add".to_string()),
            "Function should be in local names"
        );
        // Should have a Closure constant
        let has_closure = chunk
            .constants
            .iter()
            .any(|c| matches!(c, Value::Closure { .. }));
        assert!(has_closure, "Should have Closure constant");
        assert!(result_reg < 10);
    }

    // Test for loop compilation
    #[test]
    fn test_compile_for_loop() {
        let mut compiler = Compiler::new("test".to_string());

        // for i in [1, 2, 3] { i }
        let iter_elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let iter_expr = Expr::new(
            ExprKind::List(iter_elements),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("i".to_string()),
            crate::frontend::ast::Span::default(),
        );

        let for_expr = Expr::new(
            ExprKind::For {
                label: None,
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(iter_expr),
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&for_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have For opcode
        let for_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::For.to_u8());
        assert!(for_found, "Should have For instruction");
        // Should have stored loop body
        assert!(
            !chunk.loop_bodies.is_empty(),
            "Should have stored loop body"
        );
        assert!(result_reg < 10);
    }

    // Test method call compilation
    #[test]
    fn test_compile_method_call() {
        let mut compiler = Compiler::new("test".to_string());

        // "hello".len()
        let receiver = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::default(),
        );
        let method_call = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "len".to_string(),
                args: vec![],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&method_call)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have MethodCall opcode
        let method_call_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::MethodCall.to_u8());
        assert!(method_call_found, "Should have MethodCall instruction");
        // Should have stored method call info
        assert!(
            !chunk.method_calls.is_empty(),
            "Should have stored method call info"
        );
        assert!(result_reg < 10);
    }

    // Test field access compilation
    #[test]
    fn test_compile_field_access() {
        let mut compiler = Compiler::new("test".to_string());

        // obj.field
        let object = Expr::new(
            ExprKind::Identifier("obj".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let field_access = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(object),
                field: "field".to_string(),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&field_access)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have LoadField opcode
        let load_field_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LoadField.to_u8());
        assert!(load_field_found, "Should have LoadField instruction");
        assert!(result_reg < 10);
    }

    // Test index access compilation
    #[test]
    fn test_compile_index_access() {
        let mut compiler = Compiler::new("test".to_string());

        // arr[0]
        let object = Expr::new(
            ExprKind::Identifier("arr".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            crate::frontend::ast::Span::default(),
        );
        let index_access = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&index_access)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have LoadIndex opcode
        let load_index_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::LoadIndex.to_u8());
        assert!(load_index_found, "Should have LoadIndex instruction");
        assert!(result_reg < 10);
    }

    // Test tuple compilation with literals
    #[test]
    fn test_compile_tuple_literals() {
        let mut compiler = Compiler::new("test".to_string());

        // (1, 2, 3)
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let tuple_expr = Expr::new(
            ExprKind::Tuple(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&tuple_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Literal tuple should be in constant pool
        let has_tuple = chunk.constants.iter().any(|c| matches!(c, Value::Tuple(_)));
        assert!(has_tuple, "Should have Tuple constant");
        assert!(result_reg < 10);
    }

    // Test tuple compilation with non-literals
    #[test]
    fn test_compile_tuple_non_literals() {
        let mut compiler = Compiler::new("test".to_string());

        // (x, y)
        let elements = vec![
            Expr::new(
                ExprKind::Identifier("x".to_string()),
                crate::frontend::ast::Span::default(),
            ),
            Expr::new(
                ExprKind::Identifier("y".to_string()),
                crate::frontend::ast::Span::default(),
            ),
        ];
        let tuple_expr = Expr::new(
            ExprKind::Tuple(elements),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&tuple_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewTuple instruction for non-literal elements
        let new_tuple_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewTuple.to_u8());
        assert!(new_tuple_found, "Should have NewTuple instruction");
        assert!(result_reg < 10);
    }

    // Test object literal with literals
    #[test]
    fn test_compile_object_literal_literals() {
        use crate::frontend::ast::ObjectField;

        let mut compiler = Compiler::new("test".to_string());

        // { x: 1, y: 2 }
        let fields = vec![
            ObjectField::KeyValue {
                key: "x".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    crate::frontend::ast::Span::default(),
                ),
            },
            ObjectField::KeyValue {
                key: "y".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    crate::frontend::ast::Span::default(),
                ),
            },
        ];
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&obj_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Literal object should be in constant pool
        let has_object = chunk
            .constants
            .iter()
            .any(|c| matches!(c, Value::Object(_)));
        assert!(has_object, "Should have Object constant");
        assert!(result_reg < 10);
    }

    // Test object literal with non-literals
    #[test]
    fn test_compile_object_literal_non_literals() {
        use crate::frontend::ast::ObjectField;

        let mut compiler = Compiler::new("test".to_string());

        // { x: a, y: b }
        let fields = vec![
            ObjectField::KeyValue {
                key: "x".to_string(),
                value: Expr::new(
                    ExprKind::Identifier("a".to_string()),
                    crate::frontend::ast::Span::default(),
                ),
            },
            ObjectField::KeyValue {
                key: "y".to_string(),
                value: Expr::new(
                    ExprKind::Identifier("b".to_string()),
                    crate::frontend::ast::Span::default(),
                ),
            },
        ];
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&obj_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewObject instruction for non-literal elements
        let new_object_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewObject.to_u8());
        assert!(new_object_found, "Should have NewObject instruction");
        assert!(result_reg < 10);
    }

    // Test object literal with spread (error)
    #[test]
    fn test_compile_object_literal_spread_error() {
        use crate::frontend::ast::ObjectField;

        let mut compiler = Compiler::new("test".to_string());

        // { ...other }
        let fields = vec![ObjectField::Spread {
            expr: Expr::new(
                ExprKind::Identifier("other".to_string()),
                crate::frontend::ast::Span::default(),
            ),
        }];
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&obj_expr);
        assert!(result.is_err(), "Spread in object literal should fail");
        assert!(result.unwrap_err().contains("Spread operator"));
    }

    // Test match expression compilation
    #[test]
    fn test_compile_match_expression() {
        use crate::frontend::ast::{MatchArm, Pattern};

        let mut compiler = Compiler::new("test".to_string());

        // match x { 1 => "one", _ => "other" }
        let match_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1, None)),
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("one".to_string())),
                    crate::frontend::ast::Span::default(),
                )),
                span: crate::frontend::ast::Span::default(),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("other".to_string())),
                    crate::frontend::ast::Span::default(),
                )),
                span: crate::frontend::ast::Span::default(),
            },
        ];
        let match_full = Expr::new(
            ExprKind::Match {
                expr: Box::new(match_expr),
                arms,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&match_full)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have Match opcode
        let match_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Match.to_u8());
        assert!(match_found, "Should have Match instruction");
        // Should have stored match expression
        assert!(
            !chunk.match_exprs.is_empty(),
            "Should have stored match expression"
        );
        assert!(result_reg < 10);
    }

    // Test closure compilation
    #[test]
    fn test_compile_closure() {
        let mut compiler = Compiler::new("test".to_string());

        // |x| x + 1
        let param = make_test_param("x");
        let x = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let one = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(x),
                right: Box::new(one),
            },
            crate::frontend::ast::Span::default(),
        );

        let lambda = Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&lambda).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have NewClosure opcode
        let new_closure_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewClosure.to_u8());
        assert!(new_closure_found, "Should have NewClosure instruction");
        // Should have stored closure info
        assert!(
            !chunk.closures.is_empty(),
            "Should have stored closure info"
        );
        assert!(result_reg < 10);
    }

    // Test empty list compilation
    #[test]
    fn test_compile_empty_list() {
        let mut compiler = Compiler::new("test".to_string());

        // []
        let list_expr = Expr::new(
            ExprKind::List(vec![]),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&list_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty list uses NewArray path (not constant pool optimization)
        let new_array_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewArray.to_u8());
        assert!(
            new_array_found,
            "Should have NewArray instruction for empty list"
        );
        assert!(result_reg < 10);
    }

    // Test empty tuple compilation
    #[test]
    fn test_compile_empty_tuple() {
        let mut compiler = Compiler::new("test".to_string());

        // ()
        let tuple_expr = Expr::new(
            ExprKind::Tuple(vec![]),
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&tuple_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty tuple uses NewTuple path
        let new_tuple_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewTuple.to_u8());
        assert!(
            new_tuple_found,
            "Should have NewTuple instruction for empty tuple"
        );
        assert!(result_reg < 10);
    }

    // Test empty object compilation
    #[test]
    fn test_compile_empty_object() {
        let mut compiler = Compiler::new("test".to_string());

        // {}
        let obj_expr = Expr::new(
            ExprKind::ObjectLiteral { fields: vec![] },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&obj_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Empty object uses NewObject path
        let new_object_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::NewObject.to_u8());
        assert!(
            new_object_found,
            "Should have NewObject instruction for empty object"
        );
        assert!(result_reg < 10);
    }

    // Test patch_jump functionality
    #[test]
    fn test_patch_jump() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        // Emit a jump with placeholder offset
        let jump_idx = chunk.emit(Instruction::asbx(OpCode::Jump, 0, 0), 0);

        // Emit some instructions
        chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 0);
        chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 0);
        chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 0);

        // Patch the jump
        chunk.patch_jump(jump_idx);

        // Verify the jump offset was updated correctly
        let jump_instr = chunk.instructions[jump_idx];
        assert_eq!(jump_instr.opcode(), OpCode::Jump.to_u8());
        assert_eq!(
            jump_instr.get_sbx(),
            3,
            "Jump offset should be 3 (skip 3 instructions)"
        );
    }

    // Test finalize method
    #[test]
    fn test_finalize() {
        let mut compiler = Compiler::new("my_function".to_string());

        // Compile a simple expression
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let _ = compiler.compile_expr(&expr).expect("Compilation failed");

        let chunk = compiler.finalize();

        // Check finalize results
        assert_eq!(chunk.name, "my_function");
        assert!(chunk.register_count > 0, "Should have register count set");
        // Last instruction should be Return
        let last_instr = chunk.instructions.last().expect("Should have instructions");
        assert_eq!(last_instr.opcode(), OpCode::Return.to_u8());
    }

    // Test is_local_register helper
    #[test]
    fn test_is_local_register() {
        let mut compiler = Compiler::new("test".to_string());

        // Initially no locals
        assert!(!compiler.is_local_register(0));
        assert!(!compiler.is_local_register(1));

        // Add a local variable through let binding
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );
        let _ = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");

        // Register 0 should now be a local
        assert!(compiler.is_local_register(0));
        // Other registers should not be locals
        assert!(!compiler.is_local_register(100));
    }

    // Test unsupported expression kind
    #[test]
    fn test_compile_unsupported_expression() {
        let mut compiler = Compiler::new("test".to_string());

        // Use an unsupported expression kind (e.g., Import)
        let expr = Expr::new(
            ExprKind::Import {
                module: "std".to_string(),
                items: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result = compiler.compile_expr(&expr);
        assert!(result.is_err(), "Unsupported expression should fail");
        assert!(result.unwrap_err().contains("Unsupported expression kind"));
    }

    // Test function call with no arguments
    #[test]
    fn test_compile_function_call_no_args() {
        let mut compiler = Compiler::new("test".to_string());

        // foo()
        let func = Expr::new(
            ExprKind::Identifier("foo".to_string()),
            crate::frontend::ast::Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args: vec![],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&call).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have Call instruction
        let call_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Call.to_u8());
        assert!(call_found, "Should have Call instruction");
        assert!(result_reg < 10);
    }

    // Test method call with arguments
    #[test]
    fn test_compile_method_call_with_args() {
        let mut compiler = Compiler::new("test".to_string());

        // "hello".substring(0, 3)
        let receiver = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            crate::frontend::ast::Span::default(),
        );
        let method_call = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(receiver),
                method: "substring".to_string(),
                args: vec![
                    Expr::new(
                        ExprKind::Literal(Literal::Integer(0, None)),
                        crate::frontend::ast::Span::default(),
                    ),
                    Expr::new(
                        ExprKind::Literal(Literal::Integer(3, None)),
                        crate::frontend::ast::Span::default(),
                    ),
                ],
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&method_call)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have stored method call with args
        assert!(!chunk.method_calls.is_empty());
        let (_, method_name, args) = &chunk.method_calls[0];
        assert_eq!(method_name, "substring");
        assert_eq!(args.len(), 2);
        assert!(result_reg < 10);
    }

    // Test nested if expression
    #[test]
    fn test_compile_nested_if() {
        let mut compiler = Compiler::new("test".to_string());

        // if true { if false { 1 } else { 2 } } else { 3 }
        let inner_condition = Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            crate::frontend::ast::Span::default(),
        );
        let inner_then = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            crate::frontend::ast::Span::default(),
        );
        let inner_else = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            crate::frontend::ast::Span::default(),
        );
        let inner_if = Expr::new(
            ExprKind::If {
                condition: Box::new(inner_condition),
                then_branch: Box::new(inner_then),
                else_branch: Some(Box::new(inner_else)),
            },
            crate::frontend::ast::Span::default(),
        );

        let outer_condition = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            crate::frontend::ast::Span::default(),
        );
        let outer_else = Expr::new(
            ExprKind::Literal(Literal::Integer(3, None)),
            crate::frontend::ast::Span::default(),
        );
        let outer_if = Expr::new(
            ExprKind::If {
                condition: Box::new(outer_condition),
                then_branch: Box::new(inner_if),
                else_branch: Some(Box::new(outer_else)),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&outer_if)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have multiple JumpIfFalse instructions
        let jump_count = chunk
            .instructions
            .iter()
            .filter(|i| i.opcode() == OpCode::JumpIfFalse.to_u8())
            .count();
        assert!(
            jump_count >= 2,
            "Should have at least 2 JumpIfFalse instructions for nested if"
        );
        assert!(result_reg < 10);
    }

    // Test block with local variable not freed
    #[test]
    fn test_compile_block_preserves_local_registers() {
        let mut compiler = Compiler::new("test".to_string());

        // { let x = 1; let y = 2; x + y }
        let let_x = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    crate::frontend::ast::Span::default(),
                )),
                body: Box::new(Expr::new(
                    ExprKind::Let {
                        name: "y".to_string(),
                        type_annotation: None,
                        value: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Integer(2, None)),
                            crate::frontend::ast::Span::default(),
                        )),
                        body: Box::new(Expr::new(
                            ExprKind::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Expr::new(
                                    ExprKind::Identifier("x".to_string()),
                                    crate::frontend::ast::Span::default(),
                                )),
                                right: Box::new(Expr::new(
                                    ExprKind::Identifier("y".to_string()),
                                    crate::frontend::ast::Span::default(),
                                )),
                            },
                            crate::frontend::ast::Span::default(),
                        )),
                        is_mutable: false,
                        else_block: None,
                    },
                    crate::frontend::ast::Span::default(),
                )),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&let_x).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Should have both local names
        assert!(chunk.local_names.contains(&"x".to_string()));
        assert!(chunk.local_names.contains(&"y".to_string()));
        assert!(result_reg < 10);
    }

    // Test locals_map is populated in finalize
    #[test]
    fn test_finalize_populates_locals_map() {
        let mut compiler = Compiler::new("test".to_string());

        // let x = 42 in x
        let value = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            crate::frontend::ast::Span::default(),
        );

        let _ = compiler
            .compile_expr(&let_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // locals_map should contain x
        assert!(chunk.locals_map.contains_key("x"));
        assert_eq!(*chunk.locals_map.get("x").unwrap(), 0);
    }

    // Test Gt binary operator (alias for Greater)
    #[test]
    fn test_compile_binary_gt() {
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            crate::frontend::ast::Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Gt,
                left: Box::new(left),
                right: Box::new(right),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&expr).expect("Compilation failed");
        let chunk = compiler.finalize();

        let gt_found = chunk
            .instructions
            .iter()
            .any(|i| i.opcode() == OpCode::Greater.to_u8());
        assert!(gt_found, "Should have Greater instruction for Gt operator");
        assert!(result_reg < 10);
    }

    // Test float constant deduplication
    #[test]
    fn test_float_constant_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Float(3.14));
        let idx2 = chunk.add_constant(Value::Float(3.14));
        let idx3 = chunk.add_constant(Value::Float(2.71));

        assert_eq!(idx1, idx2, "Duplicate floats should return same index");
        assert_ne!(idx1, idx3, "Different floats should have different indices");
        assert_eq!(chunk.constants.len(), 2);
    }

    // Test bool constant deduplication
    #[test]
    fn test_bool_constant_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Bool(true));
        let idx2 = chunk.add_constant(Value::Bool(true));
        let idx3 = chunk.add_constant(Value::Bool(false));
        let idx4 = chunk.add_constant(Value::Bool(false));

        assert_eq!(idx1, idx2, "Duplicate true should return same index");
        assert_eq!(idx3, idx4, "Duplicate false should return same index");
        assert_ne!(idx1, idx3, "true and false should have different indices");
        assert_eq!(chunk.constants.len(), 2);
    }

    // Test nil constant deduplication
    #[test]
    fn test_nil_constant_deduplication() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::Nil);
        let idx2 = chunk.add_constant(Value::Nil);
        let idx3 = chunk.add_constant(Value::Nil);

        assert_eq!(idx1, idx2, "Duplicate Nil should return same index");
        assert_eq!(idx2, idx3, "All Nil should return same index");
        assert_eq!(chunk.constants.len(), 1);
    }

    // Test function with default parameters
    #[test]
    fn test_compile_function_with_defaults() {
        let mut compiler = Compiler::new("test".to_string());

        // fun greet(name, greeting = "Hello") { greeting + name }
        let param_name = make_test_param("name");
        let default_value = Expr::new(
            ExprKind::Literal(Literal::String("Hello".to_string())),
            crate::frontend::ast::Span::default(),
        );
        let param_greeting = make_test_param_with_default("greeting", default_value);

        let greeting_ident = Expr::new(
            ExprKind::Identifier("greeting".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let name_ident = Expr::new(
            ExprKind::Identifier("name".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(greeting_ident),
                right: Box::new(name_ident),
            },
            crate::frontend::ast::Span::default(),
        );

        let func_expr = Expr::new(
            ExprKind::Function {
                name: "greet".to_string(),
                type_params: vec![],
                params: vec![param_name, param_greeting],
                return_type: None,
                body: Box::new(body),
                is_async: false,
                is_pub: false,
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler
            .compile_expr(&func_expr)
            .expect("Compilation failed");
        let chunk = compiler.finalize();

        // Function should have closure with default parameters
        let has_closure = chunk.constants.iter().any(|c| {
            if let Value::Closure { params, .. } = c {
                // Second param should have a default value
                params.len() == 2 && params[1].1.is_some()
            } else {
                false
            }
        });
        assert!(has_closure, "Should have Closure with default parameter");
        assert!(result_reg < 10);
    }

    // Test closure with default parameters
    #[test]
    fn test_compile_closure_with_defaults() {
        let mut compiler = Compiler::new("test".to_string());

        // |x, y = 10| x + y
        let param_x = make_test_param("x");
        let default_y = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            crate::frontend::ast::Span::default(),
        );
        let param_y = make_test_param_with_default("y", default_y);

        let x = Expr::new(
            ExprKind::Identifier("x".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let y = Expr::new(
            ExprKind::Identifier("y".to_string()),
            crate::frontend::ast::Span::default(),
        );
        let body = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(x),
                right: Box::new(y),
            },
            crate::frontend::ast::Span::default(),
        );

        let lambda = Expr::new(
            ExprKind::Lambda {
                params: vec![param_x, param_y],
                body: Box::new(body),
            },
            crate::frontend::ast::Span::default(),
        );

        let result_reg = compiler.compile_expr(&lambda).expect("Compilation failed");
        let chunk = compiler.finalize();

        // Closure info should include default parameters
        assert!(!chunk.closures.is_empty());
        let (params, _) = &chunk.closures[0];
        assert_eq!(params.len(), 2);
        assert!(
            params[1].1.is_some(),
            "Second param should have default value"
        );
        assert!(result_reg < 10);
    }

    // Test string values not deduplicated (by reference)
    #[test]
    fn test_string_constant_not_deduplicated() {
        let mut chunk = BytecodeChunk::new("test".to_string());

        let idx1 = chunk.add_constant(Value::from_string("hello".to_string()));
        let idx2 = chunk.add_constant(Value::from_string("hello".to_string()));

        // String comparison returns false in values_equal (by reference, not value)
        // So strings are not deduplicated
        assert_ne!(
            idx1, idx2,
            "Strings should not be deduplicated (by reference)"
        );
        assert_eq!(chunk.constants.len(), 2);
    }
