
use super::*;
use crate::frontend::ast::{Expr, ExprKind, Literal, MatchArm, Pattern, Span};
use crate::runtime::bytecode::compiler::BytecodeChunk;
use std::sync::Arc;

fn make_span() -> Span {
    Span { start: 0, end: 0 }
}

fn make_expr(kind: ExprKind) -> Arc<Expr> {
    Arc::new(Expr {
        kind,
        span: make_span(),
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    })
}

fn make_int_expr(v: i64) -> Arc<Expr> {
    make_expr(ExprKind::Literal(Literal::Integer(v, None)))
}

// ============================================================================
// VM execute_instruction coverage: Call opcode
// ============================================================================

#[test]
fn test_vm_call_closure() {
    // Create a closure that returns its argument + 10
    // closure body: x + 10
    let body = make_expr(ExprKind::Binary {
        left: Box::new(Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
        op: crate::frontend::ast::BinaryOp::Add,
        right: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(10, None)),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
    });

    let closure = Value::Closure {
        params: vec![("x".to_string(), None)],
        body,
        env: std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
    };

    let mut chunk = BytecodeChunk::new("test_call".to_string());
    // R0 = closure, R1 = argument (5), constants[0] = closure (unused, stored in reg),
    // constants[1] = call_info array [0, 1] (func_reg=0, arg_reg=1)
    chunk.constants.push(Value::Integer(5)); // constant[0]
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(0), // func reg
        Value::Integer(1), // arg reg
    ])); // constant[1]
    chunk.register_count = 4;

    // Const R1 = 5
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // Call: result_reg=2, call_info_idx=1
    chunk.emit(Instruction::abx(OpCode::Call, 2, 1), 2);
    // Move R0 = R2 (return result)
    chunk.emit(Instruction::abc(OpCode::Move, 0, 2, 0), 3);

    let mut vm = VM::new();
    // Pre-load closure into R0
    vm.registers[0] = closure;
    // Push a call frame so execute_instruction has access to the chunk
    let result = vm.execute(&chunk);
    // The Call opcode should work - closure adds 5+10=15
    assert!(result.is_ok(), "Call should succeed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Integer(15));
}

#[test]
fn test_vm_call_non_function_error() {
    let mut chunk = BytecodeChunk::new("test_call_err".to_string());
    chunk.constants.push(Value::Integer(42)); // constant[0] - not a closure
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(0), // func_reg = R0
    ])); // constant[1] = call_info
    chunk.register_count = 4;

    // Const R0 = 42 (not a closure)
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // Call: result=1, call_info=1
    chunk.emit(Instruction::abx(OpCode::Call, 1, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("non-function"));
}

#[test]
fn test_vm_call_wrong_arg_count_error() {
    let body = make_int_expr(1);
    let closure = Value::Closure {
        params: vec![("a".to_string(), None), ("b".to_string(), None)],
        body,
        env: std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())),
    };

    let mut chunk = BytecodeChunk::new("test_call_args".to_string());
    // call_info has only 1 arg but closure expects 2
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(0), // func_reg
        Value::Integer(1), // only 1 arg
    ]));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Call, 2, 0), 1);

    let mut vm = VM::new();
    vm.registers[0] = closure;
    vm.registers[1] = Value::Integer(1);
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("expects 2 arguments"));
}

#[test]
fn test_vm_call_empty_call_info_error() {
    let mut chunk = BytecodeChunk::new("test_call_empty".to_string());
    chunk.constants.push(Value::from_array(vec![])); // empty call info
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Call, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[test]
fn test_vm_call_non_array_call_info_error() {
    let mut chunk = BytecodeChunk::new("test_call_nonarr".to_string());
    chunk.constants.push(Value::from_string("bad".to_string())); // not an array
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Call, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("array"));
}

// ============================================================================
// VM execute_instruction coverage: For opcode
// ============================================================================

#[test]
fn test_vm_for_loop() {
    // for x in [1,2,3] { x }
    // loop body just returns the identifier (last value)
    let body = make_expr(ExprKind::Identifier("x".to_string()));

    let mut chunk = BytecodeChunk::new("test_for".to_string());
    // R1 = array [1,2,3]
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ])); // constant[0]

    // loop_info: [iter_reg=1, var_name="x", body_idx=0]
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),                   // iter_reg
        Value::from_string("x".to_string()), // var_name
        Value::Integer(0),                   // body_idx
    ])); // constant[1]

    chunk.loop_bodies.push(body);
    chunk.register_count = 4;

    // Const R1 = [1,2,3]
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // For: result_reg=0, loop_info_idx=1
    chunk.emit(Instruction::abx(OpCode::For, 0, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(
        result.is_ok(),
        "For loop should succeed: {:?}",
        result.err()
    );
    // Last iteration returns 3
    assert_eq!(result.unwrap(), Value::Integer(3));
}

#[test]
fn test_vm_for_non_array_iterator_error() {
    let body = make_int_expr(0);
    let mut chunk = BytecodeChunk::new("test_for_err".to_string());
    chunk.constants.push(Value::Integer(42)); // constant[0] - not an array
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::from_string("x".to_string()),
        Value::Integer(0),
    ]));
    chunk.loop_bodies.push(body);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::For, 0, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("array"));
}

#[test]
fn test_vm_for_loop_info_not_array_error() {
    let mut chunk = BytecodeChunk::new("test_for_info".to_string());
    chunk.constants.push(Value::from_string("bad".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::For, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_for_loop_info_too_short_error() {
    let mut chunk = BytecodeChunk::new("test_for_short".to_string());
    chunk
        .constants
        .push(Value::from_array(vec![Value::Integer(0)])); // too short
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::For, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_for_loop_with_locals_map() {
    // Test locals_map synchronization during for-loop
    let body = make_expr(ExprKind::Binary {
        left: Box::new(Expr {
            kind: ExprKind::Identifier("sum".to_string()),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
        op: crate::frontend::ast::BinaryOp::Add,
        right: Box::new(Expr {
            kind: ExprKind::Identifier("x".to_string()),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
    });

    let mut chunk = BytecodeChunk::new("test_for_locals".to_string());
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::Integer(2),
    ]));
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::from_string("x".to_string()),
        Value::Integer(0),
    ]));
    chunk.loop_bodies.push(body);
    chunk.locals_map.insert("sum".to_string(), 2);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::For, 0, 1), 2);

    let mut vm = VM::new();
    vm.registers[2] = Value::Integer(0); // sum = 0
    let result = vm.execute(&chunk);
    // The loop should execute, synchronizing locals
    assert!(
        result.is_ok(),
        "For with locals should work: {:?}",
        result.err()
    );
}

// ============================================================================
// VM execute_instruction coverage: MethodCall opcode
// ============================================================================

#[test]
fn test_vm_method_call() {
    // method_call on an array: [1,2].length()
    let receiver = make_expr(ExprKind::List(vec![
        Expr {
            kind: ExprKind::Literal(Literal::Integer(1, None)),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        },
        Expr {
            kind: ExprKind::Literal(Literal::Integer(2, None)),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        },
    ]));

    let mut chunk = BytecodeChunk::new("test_method".to_string());
    // constant[0] = method_call index (0)
    chunk.constants.push(Value::Integer(0));
    chunk
        .method_calls
        .push((receiver, "length".to_string(), vec![]));
    chunk.register_count = 4;

    // MethodCall: result_reg=0, method_call_idx_const=0
    chunk.emit(Instruction::abx(OpCode::MethodCall, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(
        result.is_ok(),
        "MethodCall should succeed: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), Value::Integer(2));
}

#[test]
fn test_vm_method_call_non_int_index_error() {
    let mut chunk = BytecodeChunk::new("test_method_err".to_string());
    chunk.constants.push(Value::from_string("bad".to_string())); // not an int
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::MethodCall, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("integer"));
}

#[test]
fn test_vm_method_call_out_of_bounds_error() {
    let mut chunk = BytecodeChunk::new("test_method_oob".to_string());
    chunk.constants.push(Value::Integer(99)); // out of bounds
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::MethodCall, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

// ============================================================================
// VM execute_instruction coverage: Match opcode
// ============================================================================

#[test]
fn test_vm_match_expr() {
    // match 42 { _ => 99 }
    let match_expr = make_int_expr(42);
    let arm = MatchArm {
        pattern: Pattern::Wildcard,
        guard: None,
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(99, None)),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
        span: make_span(),
    };

    let mut chunk = BytecodeChunk::new("test_match".to_string());
    chunk.constants.push(Value::Integer(0)); // match index
    chunk.match_exprs.push((match_expr, vec![arm]));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Match, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok(), "Match should succeed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::Integer(99));
}

#[test]
fn test_vm_match_non_int_index_error() {
    let mut chunk = BytecodeChunk::new("test_match_err".to_string());
    chunk.constants.push(Value::from_string("bad".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Match, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_match_out_of_bounds_error() {
    let mut chunk = BytecodeChunk::new("test_match_oob".to_string());
    chunk.constants.push(Value::Integer(5)); // out of bounds
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Match, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

// ============================================================================
// VM execute_instruction coverage: NewClosure opcode
// ============================================================================

#[test]
fn test_vm_new_closure() {
    let body = make_int_expr(42);
    let params = vec![("x".to_string(), None)];

    let mut chunk = BytecodeChunk::new("test_closure".to_string());
    chunk.constants.push(Value::Integer(0)); // closure index
    chunk.closures.push((params, body));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::NewClosure, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(
        result.is_ok(),
        "NewClosure should succeed: {:?}",
        result.err()
    );
    match result.unwrap() {
        Value::Closure { params, .. } => {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].0, "x");
        }
        other => panic!("Expected Closure, got {:?}", other),
    }
}

#[test]
fn test_vm_new_closure_non_int_index_error() {
    let mut chunk = BytecodeChunk::new("test_closure_err".to_string());
    chunk.constants.push(Value::from_string("bad".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::NewClosure, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_new_closure_out_of_bounds_error() {
    let mut chunk = BytecodeChunk::new("test_closure_oob".to_string());
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::NewClosure, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_new_closure_with_locals_map() {
    let body = make_expr(ExprKind::Identifier("y".to_string()));
    let params = vec![];

    let mut chunk = BytecodeChunk::new("test_closure_locals".to_string());
    chunk.constants.push(Value::Integer(0));
    chunk.closures.push((params, body));
    chunk.locals_map.insert("y".to_string(), 1);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::NewClosure, 0, 0), 1);

    let mut vm = VM::new();
    vm.registers[1] = Value::Integer(77);
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
}

// ============================================================================
// VM execute_instruction: additional error paths
// ============================================================================

#[test]
fn test_vm_for_loop_body_idx_not_int_error() {
    let mut chunk = BytecodeChunk::new("test_for_body_err".to_string());
    // loop_info: iter_reg is fine, var_name fine, but body_idx is a string
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::from_string("x".to_string()),
        Value::from_string("bad".to_string()), // body_idx not int
    ]));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::For, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_for_loop_var_name_not_string_error() {
    let body = make_int_expr(0);
    let mut chunk = BytecodeChunk::new("test_for_var_err".to_string());
    // loop_info: iter_reg ok, var_name is integer (wrong), body_idx ok
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::Integer(999), // var name not a string
        Value::Integer(0),
    ]));
    chunk.loop_bodies.push(body);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::For, 0, 0), 2);

    let mut vm = VM::new();
    vm.registers[1] = Value::from_array(vec![Value::Integer(1)]);
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_for_iter_reg_not_int_error() {
    let mut chunk = BytecodeChunk::new("test_for_iter_err".to_string());
    chunk.constants.push(Value::from_array(vec![
        Value::from_string("bad".to_string()), // iter_reg not int
        Value::from_string("x".to_string()),
        Value::Integer(0),
    ]));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::For, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_call_with_closure_env() {
    // Closure that captures 'y' from environment
    let body = make_expr(ExprKind::Identifier("y".to_string()));
    let mut env = std::collections::HashMap::new();
    env.insert("y".to_string(), Value::Integer(100));

    let closure = Value::Closure {
        params: vec![],
        body,
        env: std::rc::Rc::new(std::cell::RefCell::new(env)),
    };

    let mut chunk = BytecodeChunk::new("test_call_env".to_string());
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(0), // func_reg
    ])); // call_info
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Call, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Move, 0, 1, 0), 2);

    let mut vm = VM::new();
    vm.registers[0] = closure;
    let result = vm.execute(&chunk);
    assert!(
        result.is_ok(),
        "Call with env should work: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap(), Value::Integer(100));
}

#[test]
fn test_vm_match_with_literal_pattern() {
    // match 1 { 1 => 10, _ => 20 }
    let match_expr = make_int_expr(1);
    let arm1 = MatchArm {
        pattern: Pattern::Literal(Literal::Integer(1, None)),
        guard: None,
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(10, None)),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
        span: make_span(),
    };
    let arm2 = MatchArm {
        pattern: Pattern::Wildcard,
        guard: None,
        body: Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(20, None)),
            span: make_span(),
            attributes: Vec::new(),
            leading_comments: vec![],
            trailing_comment: None,
        }),
        span: make_span(),
    };

    let mut chunk = BytecodeChunk::new("test_match_lit".to_string());
    chunk.constants.push(Value::Integer(0));
    chunk.match_exprs.push((match_expr, vec![arm1, arm2]));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Match, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(10));
}

#[test]
fn test_vm_method_call_with_locals_sync() {
    let receiver = make_expr(ExprKind::List(vec![Expr {
        kind: ExprKind::Literal(Literal::Integer(5, None)),
        span: make_span(),
        attributes: Vec::new(),
        leading_comments: vec![],
        trailing_comment: None,
    }]));

    let mut chunk = BytecodeChunk::new("test_method_locals".to_string());
    chunk.constants.push(Value::Integer(0));
    chunk
        .method_calls
        .push((receiver, "length".to_string(), vec![]));
    chunk.locals_map.insert("result".to_string(), 1);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::MethodCall, 0, 0), 1);

    let mut vm = VM::new();
    vm.registers[1] = Value::Integer(0);
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(1));
}

#[test]
fn test_vm_neg_float() {
    let mut chunk = BytecodeChunk::new("test_neg_f".to_string());
    chunk.constants.push(Value::Float(3.14));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Neg, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Float(-3.14));
}

#[test]
fn test_vm_bitnot_integer() {
    let mut chunk = BytecodeChunk::new("test_bitnot".to_string());
    chunk.constants.push(Value::Integer(0));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::BitNot, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(!0i64));
}

#[test]
fn test_vm_load_field_tuple_index() {
    let mut chunk = BytecodeChunk::new("test_tuple_idx".to_string());
    chunk.constants.push(Value::from_string("1".to_string())); // field name "1"
    chunk.register_count = 4;

    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 0), 1);

    let mut vm = VM::new();
    vm.registers[1] = Value::Tuple(Arc::from(
        vec![Value::Integer(10), Value::Integer(20)].as_slice(),
    ));
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(20));
}

#[test]
fn test_vm_load_field_tuple_out_of_bounds() {
    let mut chunk = BytecodeChunk::new("test_tuple_oob".to_string());
    chunk.constants.push(Value::from_string("5".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 0), 1);

    let mut vm = VM::new();
    vm.registers[1] = Value::Tuple(Arc::from(vec![Value::Integer(10)].as_slice()));
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_vm_for_empty_array() {
    let body = make_int_expr(99);
    let mut chunk = BytecodeChunk::new("test_for_empty".to_string());
    chunk.constants.push(Value::from_array(vec![])); // empty array
    chunk.constants.push(Value::from_array(vec![
        Value::Integer(1),
        Value::from_string("x".to_string()),
        Value::Integer(0),
    ]));
    chunk.loop_bodies.push(body);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::For, 0, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_ok());
    // Empty loop returns Nil
    assert_eq!(result.unwrap(), Value::Nil);
}
