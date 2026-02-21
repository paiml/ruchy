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
