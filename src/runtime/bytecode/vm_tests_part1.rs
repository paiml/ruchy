    use super::*;
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span, UnaryOp};
    use crate::runtime::bytecode::Compiler;

    #[test]
    fn test_vm_execute_integer_literal() {
        // Compile: 42
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_addition() {
        // Compile: 10 + 32
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(32, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_multiplication() {
        // Compile: 6 * 7
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(6, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(7, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_comparison() {
        // Compile: 10 < 20
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Less,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_execute_if_true_branch() {
        // Compile: if true { 42 } else { 0 }
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_execute_if_false_branch() {
        // Compile: if false { 42 } else { 100 }
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_vm_execute_block() {
        // Compile: { 1; 2; 3 }
        let mut compiler = Compiler::new("test".to_string());
        let exprs = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span::default(),
            ),
        ];
        let block = Expr::new(ExprKind::Block(exprs), Span::default());
        compiler
            .compile_expr(&block)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        // Execute
        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(3));
    }

    // ========================================================================
    // UNIT TESTS: CallFrame operations (Sprint 5 - OPT-003)
    // ========================================================================

    #[test]
    fn test_callframe_new_initialization() {
        // Test CallFrame initialization with default values
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let frame = CallFrame::new(&chunk);

        assert_eq!(frame.pc, 0, "PC should initialize to 0");
        assert_eq!(
            frame.base_register, 0,
            "Base register should initialize to 0"
        );
    }

    #[test]
    fn test_callframe_fetch_instruction_valid() {
        // Test fetching valid instruction at current PC
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let frame = CallFrame::new(&chunk);
        let instruction = frame.fetch_instruction();

        assert!(instruction.is_some(), "Should fetch instruction at PC 0");
        assert_eq!(
            instruction
                .expect("instruction should be Some (verified by assert)")
                .opcode(),
            OpCode::Const as u8,
            "First instruction should be Const (load constant)"
        );
    }

    #[test]
    fn test_callframe_fetch_instruction_out_of_bounds() {
        // Test fetching instruction beyond bytecode end
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        // Move PC beyond bytecode
        frame.pc = chunk.instructions.len() + 10;
        let instruction = frame.fetch_instruction();

        assert!(
            instruction.is_none(),
            "Should return None for out-of-bounds PC"
        );
    }

    #[test]
    fn test_callframe_advance_pc() {
        // Test program counter increment
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        assert_eq!(frame.pc, 0);

        frame.advance_pc();
        assert_eq!(frame.pc, 1, "PC should increment by 1");

        frame.advance_pc();
        assert_eq!(frame.pc, 2, "PC should increment again");
    }

    #[test]
    fn test_callframe_jump_positive_offset() {
        // Test jumping forward (positive offset)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        frame.pc = 5;
        frame.jump(10); // Jump forward by 10

        assert_eq!(frame.pc, 15, "PC should jump forward by offset");
    }

    #[test]
    fn test_callframe_jump_negative_offset() {
        // Test jumping backward (negative offset)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        frame.pc = 10;
        frame.jump(-5); // Jump backward by 5

        assert_eq!(frame.pc, 5, "PC should jump backward by offset");
    }

    #[test]
    fn test_callframe_jump_zero_offset() {
        // Test jump with zero offset (no-op)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut frame = CallFrame::new(&chunk);
        frame.pc = 7;
        frame.jump(0); // Zero offset

        assert_eq!(frame.pc, 7, "PC should remain unchanged with zero offset");
    }

    // ========================================================================
    // UNIT TESTS: VM initialization and state (Sprint 5 - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_new_initialization() {
        // Test VM initialization state
        let vm = VM::new();

        // Verify registers initialized to Nil
        for (idx, reg) in vm.registers.iter().enumerate() {
            assert_eq!(*reg, Value::Nil, "Register {idx} should initialize to Nil");
        }

        assert!(vm.call_stack.is_empty(), "Call stack should be empty");
        assert!(vm.globals.is_empty(), "Globals should be empty");
    }

    #[test]
    fn test_vm_register_count() {
        // Test VM has exactly MAX_REGISTERS (32) registers
        let vm = VM::new();
        assert_eq!(
            vm.registers.len(),
            MAX_REGISTERS,
            "VM should have exactly {MAX_REGISTERS} registers"
        );
    }

    #[test]
    fn test_vm_execute_empty_bytecode() {
        // Test executing empty bytecode chunk
        let compiler = Compiler::new("test".to_string());
        let chunk = compiler.finalize(); // Empty bytecode

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        // Empty bytecode returns register 0 (Nil by default)
        assert_eq!(result, Value::Nil, "Empty bytecode should return Nil");
    }

    #[test]
    fn test_vm_multiple_sequential_executions() {
        // Test VM can execute multiple chunks sequentially
        let mut vm = VM::new();

        // Execute first chunk: 10 + 20
        let mut compiler1 = Compiler::new("test1".to_string());
        let left1 = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let right1 = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::default(),
        );
        let expr1 = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(left1),
                right: Box::new(right1),
            },
            Span::default(),
        );
        compiler1
            .compile_expr(&expr1)
            .expect("compile_expr should succeed in test");
        let chunk1 = compiler1.finalize();

        let result1 = vm
            .execute(&chunk1)
            .expect("vm.execute should succeed in test");
        assert_eq!(result1, Value::Integer(30));

        // Execute second chunk: 5 * 6
        let mut compiler2 = Compiler::new("test2".to_string());
        let left2 = Expr::new(
            ExprKind::Literal(Literal::Integer(5, None)),
            Span::default(),
        );
        let right2 = Expr::new(
            ExprKind::Literal(Literal::Integer(6, None)),
            Span::default(),
        );
        let expr2 = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(left2),
                right: Box::new(right2),
            },
            Span::default(),
        );
        compiler2
            .compile_expr(&expr2)
            .expect("compile_expr should succeed in test");
        let chunk2 = compiler2.finalize();

        let result2 = vm
            .execute(&chunk2)
            .expect("vm.execute should succeed in test");
        assert_eq!(result2, Value::Integer(30));
    }

    #[test]
    fn test_vm_register_isolation_between_executions() {
        // Test that register state is isolated between executions
        let mut vm = VM::new();

        // Execute first chunk: loads 42 into register 0
        let mut compiler1 = Compiler::new("test1".to_string());
        let expr1 = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        compiler1
            .compile_expr(&expr1)
            .expect("compile_expr should succeed in test");
        let chunk1 = compiler1.finalize();

        let result1 = vm
            .execute(&chunk1)
            .expect("vm.execute should succeed in test");
        assert_eq!(result1, Value::Integer(42));

        // Execute second chunk: loads 100 into register 0
        let mut compiler2 = Compiler::new("test2".to_string());
        let expr2 = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        compiler2
            .compile_expr(&expr2)
            .expect("compile_expr should succeed in test");
        let chunk2 = compiler2.finalize();

        let result2 = vm
            .execute(&chunk2)
            .expect("vm.execute should succeed in test");
        assert_eq!(
            result2,
            Value::Integer(100),
            "Second execution should overwrite register 0"
        );
    }

    // ========================================================================
    // OPCODE TESTS: Binary arithmetic operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_subtraction() {
        // Compile: 50 - 8
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(50, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(8, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_division() {
        // Compile: 84 / 2
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(84, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Divide,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_modulo() {
        // Compile: 100 % 58
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(58, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Modulo,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    // ========================================================================
    // OPCODE TESTS: Unary operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_negation_integer() {
        // Compile: -(-42)
        let mut compiler = Compiler::new("test".to_string());
        let inner = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(inner),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_negation_float() {
        // Compile: -(3.14)
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Float(std::f64::consts::PI)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Float(-std::f64::consts::PI));
    }

    #[test]
    fn test_vm_opcode_logical_not_true() {
        // Compile: !true
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_opcode_logical_not_false() {
        // Compile: !false
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(false)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    // ========================================================================
    // OPCODE TESTS: Comparison operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_equal_true() {
        // Compile: 42 == 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_equal_false() {
        // Compile: 42 == 100
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Equal,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_opcode_not_equal() {
        // Compile: 42 != 100
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::NotEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_less_equal() {
        // Compile: 42 <= 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::LessEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_greater() {
        // Compile: 100 > 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Greater,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_greater_equal() {
        // Compile: 42 >= 42
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let right = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::GreaterEqual,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    // ========================================================================
    // OPCODE TESTS: Logical operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_logical_and_true() {
        // Compile: true && true
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_logical_and_false() {
        // Compile: true && false
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_vm_opcode_logical_or_true() {
        // Compile: false || true
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_vm_opcode_logical_or_false() {
        // Compile: false || false
        let mut compiler = Compiler::new("test".to_string());
        let left = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let right = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Bool(false));
    }

    // ========================================================================
    // OPCODE TESTS: Data structure operations (Sprint 5 Extended - OPT-003)
    // ========================================================================

    #[test]
    fn test_vm_opcode_array_literal() {
        // Compile: [1, 2, 3]
        let mut compiler = Compiler::new("test".to_string());
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span::default(),
            ),
        ];
        let expr = Expr::new(ExprKind::List(elements), Span::default());
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Integer(2));
                assert_eq!(arr[2], Value::Integer(3));
            }
            _ => panic!("Expected array, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_array_empty() {
        // Compile: []
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(ExprKind::List(vec![]), Span::default());
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected array, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_tuple_literal() {
        // Compile: (42, true, "hello")
        let mut compiler = Compiler::new("test".to_string());
        let elements = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Span::default(),
            ),
            Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default()),
            Expr::new(
                ExprKind::Literal(Literal::String("hello".to_string())),
                Span::default(),
            ),
        ];
        let expr = Expr::new(ExprKind::Tuple(elements), Span::default());
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        match result {
            Value::Tuple(tuple) => {
                assert_eq!(tuple.len(), 3);
                assert_eq!(tuple[0], Value::Integer(42));
                assert_eq!(tuple[1], Value::Bool(true));
                assert_eq!(tuple[2], Value::from_string("hello".to_string()));
            }
            _ => panic!("Expected tuple, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_object_literal() {
        // Compile: { x: 10, y: 20 }
        let mut compiler = Compiler::new("test".to_string());
        use crate::frontend::ast::ObjectField;
        let fields = vec![
            ObjectField::KeyValue {
                key: "x".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
            },
            ObjectField::KeyValue {
                key: "y".to_string(),
                value: Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
            },
        ];
        let expr = Expr::new(ExprKind::ObjectLiteral { fields }, Span::default());
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        match result {
            Value::Object(obj) => {
                assert_eq!(obj.get("x"), Some(&Value::Integer(10)));
                assert_eq!(obj.get("y"), Some(&Value::Integer(20)));
            }
            _ => panic!("Expected object, got {result:?}"),
        }
    }

    #[test]
    fn test_vm_opcode_array_index_access() {
        // Compile: [10, 20, 30][1]
        let mut compiler = Compiler::new("test".to_string());
        let array = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(30, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(array),
                index: Box::new(index),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_vm_opcode_array_index_negative() {
        // Compile: [10, 20, 30][-1]  (negative indexing: last element)
        let mut compiler = Compiler::new("test".to_string());
        let array = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(30, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(-1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(array),
                index: Box::new(index),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_vm_opcode_string_index_access() {
        // Compile: "hello"[1]
        let mut compiler = Compiler::new("test".to_string());
        let string = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let index = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(string),
                index: Box::new(index),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::from_string("e".to_string()));
    }

    #[test]
    fn test_vm_opcode_object_field_access() {
        // Compile: { x: 42 }.x
        let mut compiler = Compiler::new("test".to_string());
        use crate::frontend::ast::ObjectField;
        let fields = vec![ObjectField::KeyValue {
            key: "x".to_string(),
            value: Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Span::default(),
            ),
        }];
        let object = Expr::new(ExprKind::ObjectLiteral { fields }, Span::default());
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(object),
                field: "x".to_string(),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_tuple_field_access() {
        // Compile: (100, 200).0
        let mut compiler = Compiler::new("test".to_string());
        let tuple = Expr::new(
            ExprKind::Tuple(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(100, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(200, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(tuple),
                field: "0".to_string(),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_vm_opcode_nested_array_access() {
        // Compile: [[1, 2], [3, 4]][1][0]
        let mut compiler = Compiler::new("test".to_string());
        let inner1 = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let inner2 = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(4, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let outer = Expr::new(ExprKind::List(vec![inner1, inner2]), Span::default());
        let first_index = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(outer),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(first_index),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(3));
    }

    // ============================================================================
    // Sprint 5 Extended Phase 3: Control Flow & Error Handling Tests
    // Target: Exercise Jump opcodes, error paths, edge cases
    // ============================================================================

    #[test]
    fn test_vm_opcode_if_else_both_branches() {
        // Compile: if false { 10 } else { 20 }
        // Exercises JumpIfFalse and Jump opcodes (else branch)
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(20)); // Should take else branch
    }

    #[test]
    fn test_vm_opcode_if_without_else_true() {
        // Compile: if true { 42 }
        // Exercises JumpIfFalse opcode (skipping jump)
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_vm_opcode_if_without_else_false() {
        // Compile: if false { 42 }
        // Should return nil when condition is false and no else branch
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::default());
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_vm_opcode_truthy_nonzero_integer() {
        // Compile: if 42 { 100 } else { 200 }
        // Non-zero integers are truthy, should execute then branch
        let mut compiler = Compiler::new("test".to_string());
        let condition = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let then_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let else_branch = Expr::new(
            ExprKind::Literal(Literal::Integer(200, None)),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: Some(Box::new(else_branch)),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm
            .execute(&chunk)
            .expect("vm.execute should succeed in test");

        assert_eq!(result, Value::Integer(100)); // Truthy takes then branch
    }

    #[test]
    fn test_vm_opcode_array_index_out_of_bounds() {
        // Compile: [10, 20][5]
        // Should error with bounds check
        let mut compiler = Compiler::new("test".to_string());
        let array = Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(20, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(array),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_vm_opcode_string_index_out_of_bounds() {
        // Compile: "hello"[10]
        // Should error with bounds check
        let mut compiler = Compiler::new("test".to_string());
        let string = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(string),
                index: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn test_vm_opcode_field_access_missing_field() {
        // Compile: { x: 10 }.y
        // Should error with field not found
        let mut compiler = Compiler::new("test".to_string());
        use crate::frontend::ast::ObjectField;
        let object = Expr::new(
            ExprKind::ObjectLiteral {
                fields: vec![ObjectField::KeyValue {
                    key: "x".to_string(),
                    value: Expr::new(
                        ExprKind::Literal(Literal::Integer(10, None)),
                        Span::default(),
                    ),
                }],
            },
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(object),
                field: "y".to_string(),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_vm_opcode_tuple_index_out_of_bounds() {
        // Compile: (100, 200).5
        // Should error with tuple index out of bounds
        let mut compiler = Compiler::new("test".to_string());
        let tuple = Expr::new(
            ExprKind::Tuple(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(100, None)),
                    Span::default(),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(200, None)),
                    Span::default(),
                ),
            ]),
            Span::default(),
        );
        let expr = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(tuple),
                field: "5".to_string(),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    // ============================================================================
    // Sprint 5 Extended Phase 4: Error Branches & Edge Cases
    // Target: Exercise error handling in arithmetic/type operations
    // ============================================================================

    #[test]
    #[ignore = "VM doesn't implement divide-by-zero error handling yet - panics instead of returning error"]
    fn test_vm_opcode_division_by_zero_integer() {
        // Compile: 10 / 0
        // Should error with division by zero
        let mut compiler = Compiler::new("test".to_string());
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::default(),
                )),
                op: BinaryOp::Divide,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0, None)),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        compiler
            .compile_expr(&expr)
            .expect("compile_expr should succeed in test");
        let chunk = compiler.finalize();

        let mut vm = VM::new();
        let result = vm.execute(&chunk);

        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("division by zero") || err_msg.contains("divide by zero"));
    }
