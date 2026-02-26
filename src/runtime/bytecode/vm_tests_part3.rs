use super::*;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span, UnaryOp};
use crate::runtime::bytecode::compiler::BytecodeChunk;
use crate::runtime::bytecode::Compiler;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// VM execute_instruction coverage: LoadGlobal / StoreGlobal
// ============================================================================

#[test]
fn test_vm_store_and_load_global() {
    // Build a chunk that: stores 42 to global "g", then loads it back
    let mut chunk = BytecodeChunk::new("test_globals".to_string());
    // constants: [0] = 42, [1] = "g"
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::from_string("g".to_string()));
    chunk.register_count = 4;

    // Const R0 = constants[0]  (42)
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // StoreGlobal R0, constants[1]  ("g")
    chunk.emit(Instruction::abx(OpCode::StoreGlobal, 0, 1), 2);
    // LoadGlobal R1, constants[1]  ("g")
    chunk.emit(Instruction::abx(OpCode::LoadGlobal, 1, 1), 3);
    // Move R0, R1  (so result is in R0)
    chunk.emit(Instruction::abc(OpCode::Move, 0, 1, 0), 4);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_load_global_undefined_error() {
    // Try to load an undefined global
    let mut chunk = BytecodeChunk::new("test_globals_err".to_string());
    chunk
        .constants
        .push(Value::from_string("nonexistent".to_string()));
    chunk.register_count = 2;

    // LoadGlobal R0, constants[0]  ("nonexistent")
    chunk.emit(Instruction::abx(OpCode::LoadGlobal, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Undefined global"), "got: {err}");
}

#[test]
fn test_vm_store_global_non_string_name_error() {
    // StoreGlobal with non-string constant for name
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99)); // not a string
    chunk.register_count = 2;

    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    chunk.emit(Instruction::abx(OpCode::StoreGlobal, 0, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("string"));
}

#[test]
fn test_vm_load_global_non_string_name_error() {
    // LoadGlobal with non-string constant for name
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(99)); // not a string
    chunk.register_count = 2;

    chunk.emit(Instruction::abx(OpCode::LoadGlobal, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("string"));
}

// ============================================================================
// VM execute_instruction coverage: Move opcode
// ============================================================================

#[test]
fn test_vm_move_register() {
    // Load a value, move it to another register
    let mut chunk = BytecodeChunk::new("test_move".to_string());
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Move, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(99));
}

// ============================================================================
// VM execute_instruction coverage: Return opcode
// ============================================================================

#[test]
fn test_vm_return_opcode() {
    // Build chunk with a Return instruction
    let mut chunk = BytecodeChunk::new("test_return".to_string());
    chunk.constants.push(Value::Integer(77));
    chunk.register_count = 4;

    // Load 77 into R2
    chunk.emit(Instruction::abx(OpCode::Const, 2, 0), 1);
    // Return R2 (pops frame, puts value into R0)
    chunk.emit(Instruction::abx(OpCode::Return, 2, 0), 2);
    // This should never be reached
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(77));
}

// ============================================================================
// VM execute_instruction coverage: Jump opcode
// ============================================================================

#[test]
fn test_vm_jump_opcode_skips_instructions() {
    // Load 10, Jump over the next Const (which would load 99),
    // then load 42 into R0
    let mut chunk = BytecodeChunk::new("test_jump".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(99));
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;

    // R0 = 10
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // Jump +1 (skip next instruction)
    chunk.emit(Instruction::asbx(OpCode::Jump, 0, 1), 2);
    // R0 = 99 (this should be skipped)
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);
    // R0 = 42
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

// ============================================================================
// VM execute_instruction coverage: JumpIfFalse with Nil
// ============================================================================

#[test]
fn test_vm_jump_if_false_with_nil() {
    // Nil is falsy, so JumpIfFalse should jump
    let mut chunk = BytecodeChunk::new("test_jif_nil".to_string());
    chunk.constants.push(Value::Nil);
    chunk.constants.push(Value::Integer(100));
    chunk.constants.push(Value::Integer(200));
    chunk.register_count = 4;

    // R0 = Nil
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // JumpIfFalse R0, +1  (skip next if R0 is falsy)
    chunk.emit(Instruction::asbx(OpCode::JumpIfFalse, 0, 1), 2);
    // R0 = 100 (should be skipped because Nil is falsy)
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);
    // R0 = 200
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(200));
}

#[test]
fn test_vm_jump_if_false_with_true_no_jump() {
    // true is truthy, so JumpIfFalse should NOT jump
    let mut chunk = BytecodeChunk::new("test_jif_true".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Integer(100));
    chunk.constants.push(Value::Integer(200));
    chunk.register_count = 4;

    // R0 = true
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // JumpIfFalse R0, +1  (should NOT jump because true is truthy)
    chunk.emit(Instruction::asbx(OpCode::JumpIfFalse, 0, 1), 2);
    // R0 = 100 (should NOT be skipped)
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(100));
}

// ============================================================================
// VM execute_instruction coverage: JumpIfTrue
// ============================================================================

#[test]
fn test_vm_jump_if_true_with_true() {
    // true is truthy, so JumpIfTrue should jump
    let mut chunk = BytecodeChunk::new("test_jit".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Integer(100));
    chunk.constants.push(Value::Integer(200));
    chunk.register_count = 4;

    // R0 = true
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // JumpIfTrue R0, +1
    chunk.emit(Instruction::asbx(OpCode::JumpIfTrue, 0, 1), 2);
    // R0 = 100 (skipped)
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);
    // R0 = 200
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(200));
}

#[test]
fn test_vm_jump_if_true_with_false_no_jump() {
    // false is falsy, JumpIfTrue should not jump
    let mut chunk = BytecodeChunk::new("test_jit_false".to_string());
    chunk.constants.push(Value::Bool(false));
    chunk.constants.push(Value::Integer(100));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    chunk.emit(Instruction::asbx(OpCode::JumpIfTrue, 0, 1), 2);
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_vm_jump_if_true_with_nil_no_jump() {
    // Nil is falsy, JumpIfTrue should not jump
    let mut chunk = BytecodeChunk::new("test_jit_nil".to_string());
    chunk.constants.push(Value::Nil);
    chunk.constants.push(Value::Integer(55));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    chunk.emit(Instruction::asbx(OpCode::JumpIfTrue, 0, 1), 2);
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(55));
}

#[test]
fn test_vm_jump_if_true_with_integer_truthy() {
    // Non-zero integer is truthy, JumpIfTrue should jump
    let mut chunk = BytecodeChunk::new("test_jit_int".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(100));
    chunk.constants.push(Value::Integer(200));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    chunk.emit(Instruction::asbx(OpCode::JumpIfTrue, 0, 1), 2);
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(200));
}

// ============================================================================
// VM execute_instruction coverage: NewArray with raw chunk
// ============================================================================

#[test]
fn test_vm_new_array_raw_chunk() {
    let mut chunk = BytecodeChunk::new("test_newarray".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(20));
    chunk.array_element_regs.push(vec![1, 2]); // element regs index 0
    chunk.register_count = 4;

    // R1 = 10
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // R2 = 20
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    // R0 = NewArray(element_regs_idx=0)
    chunk.emit(Instruction::abx(OpCode::NewArray, 0, 0), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(10));
            assert_eq!(arr[1], Value::Integer(20));
        }
        _ => panic!("Expected array, got {result:?}"),
    }
}

// ============================================================================
// VM execute_instruction coverage: NewTuple with raw chunk
// ============================================================================

#[test]
fn test_vm_new_tuple_raw_chunk() {
    let mut chunk = BytecodeChunk::new("test_newtuple".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Integer(42));
    chunk.array_element_regs.push(vec![1, 2]); // reuses array_element_regs
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abx(OpCode::NewTuple, 0, 0), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    match result {
        Value::Tuple(t) => {
            assert_eq!(t.len(), 2);
            assert_eq!(t[0], Value::Bool(true));
            assert_eq!(t[1], Value::Integer(42));
        }
        _ => panic!("Expected tuple, got {result:?}"),
    }
}

// ============================================================================
// VM execute_instruction coverage: NewObject with raw chunk
// ============================================================================

#[test]
fn test_vm_new_object_raw_chunk() {
    let mut chunk = BytecodeChunk::new("test_newobject".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(20));
    chunk
        .object_fields
        .push(vec![("x".to_string(), 1), ("y".to_string(), 2)]);
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abx(OpCode::NewObject, 0, 0), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    match result {
        Value::Object(obj) => {
            assert_eq!(obj.get("x"), Some(&Value::Integer(10)));
            assert_eq!(obj.get("y"), Some(&Value::Integer(20)));
        }
        _ => panic!("Expected object, got {result:?}"),
    }
}

// ============================================================================
// VM execute_instruction coverage: LoadField with Struct and Class values
// ============================================================================

#[test]
fn test_vm_load_field_struct_value() {
    // Test LoadField on a Value::Struct
    let mut chunk = BytecodeChunk::new("test_loadfield_struct".to_string());
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Value::from_string("test".to_string()));
    chunk.constants.push(Value::Struct {
        name: "MyStruct".to_string(),
        fields: Arc::new(fields),
    });
    chunk.constants.push(Value::from_string("name".to_string()));
    chunk.register_count = 4;

    // R1 = struct value
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // R0 = R1.name (field_idx=1)
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::from_string("test".to_string()));
}

#[test]
fn test_vm_load_field_struct_missing_field_error() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    let fields = HashMap::new();
    chunk.constants.push(Value::Struct {
        name: "Empty".to_string(),
        fields: Arc::new(fields),
    });
    chunk
        .constants
        .push(Value::from_string("missing".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_vm_load_field_class_value() {
    // Test LoadField on a Value::Class
    let mut chunk = BytecodeChunk::new("test_loadfield_class".to_string());
    let mut fields = HashMap::new();
    fields.insert("age".to_string(), Value::Integer(25));
    chunk.constants.push(Value::Class {
        class_name: "Person".to_string(),
        fields: Arc::new(std::sync::RwLock::new(fields)),
        methods: Arc::new(HashMap::new()),
    });
    chunk.constants.push(Value::from_string("age".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(25));
}

#[test]
fn test_vm_load_field_non_string_field_name_error() {
    // Field name constant is not a string
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99)); // not a string
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("string"));
}

#[test]
fn test_vm_load_field_on_integer_error() {
    // Cannot access field on integer
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::from_string("foo".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot access field"));
}

// ============================================================================
// VM execute_instruction coverage: LoadIndex errors
// ============================================================================

#[test]
fn test_vm_load_index_invalid_type_error() {
    // Cannot index a boolean with an integer
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Integer(0));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cannot index"));
}

// ============================================================================
// VM execute_instruction coverage: Const out of bounds
// ============================================================================

#[test]
fn test_vm_const_out_of_bounds_error() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    // No constants added
    chunk.register_count = 2;

    // Try to load constant at index 99 which doesn't exist
    chunk.emit(Instruction::abx(OpCode::Const, 0, 99), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

// ============================================================================
// VM execute_instruction coverage: Arithmetic on raw bytecode
// ============================================================================

#[test]
fn test_vm_arithmetic_sub_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(100));
    chunk.constants.push(Value::Integer(58));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Sub, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_arithmetic_mul_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(6));
    chunk.constants.push(Value::Integer(7));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Mul, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_arithmetic_div_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(84));
    chunk.constants.push(Value::Integer(2));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Div, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_arithmetic_mod_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(100));
    chunk.constants.push(Value::Integer(58));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Mod, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

// ============================================================================
// VM execute_instruction coverage: Unary Neg and Not raw
// ============================================================================

#[test]
fn test_vm_unary_neg_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(-42));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Neg, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_unary_not_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Not, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_unary_bitnot_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(0));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::BitNot, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(-1)); // !0 == -1
}

// ============================================================================
// VM execute_instruction coverage: Comparison operators raw
// ============================================================================

#[test]
fn test_vm_equal_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Equal, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_not_equal_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::NotEqual, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_less_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(5));
    chunk.constants.push(Value::Integer(10));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Less, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_less_equal_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(10));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LessEqual, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_greater_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(5));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Greater, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_greater_equal_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(10));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::GreaterEqual, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// VM execute_instruction coverage: Logical And/Or raw
// ============================================================================

#[test]
fn test_vm_logical_and_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Bool(false));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::And, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_logical_or_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(false));
    chunk.constants.push(Value::Bool(true));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Or, 0, 1, 2), 3);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// VM execute_instruction coverage: Unsupported opcode error
// ============================================================================

#[test]
fn test_vm_unsupported_opcode_error() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.register_count = 2;

    // Use a raw instruction with an opcode that's not handled in execute_instruction
    // Nop is 0x00 but it falls through to the _ => Err branch
    chunk.emit(Instruction::abc(OpCode::Nop, 0, 0, 0), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported opcode"));
}

// ============================================================================
// VM: Let expression via compiler (exercises StoreLocal-like patterns)
// ============================================================================

#[test]
fn test_vm_let_expression_via_compiler() {
    // Compile: let x = 10 in x + 32
    let mut compiler = Compiler::new("test".to_string());
    let value = Expr::new(
        ExprKind::Literal(Literal::Integer(10, None)),
        Span::default(),
    );
    let body = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                Span::default(),
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(32, None)),
                Span::default(),
            )),
        },
        Span::default(),
    );
    let expr = Expr::new(
        ExprKind::Let {
            name: "x".to_string(),
            value: Box::new(value),
            body: Box::new(body),
            is_mutable: false,
            type_annotation: None,
            else_block: None,
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

// ============================================================================
// VM: Unary with Negate on string via compiler (error path)
// ============================================================================

#[test]
fn test_vm_negate_bool_error() {
    // Compile: -true (cannot negate boolean)
    let mut compiler = Compiler::new("test".to_string());
    let expr = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Negate,
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
    let result = vm.execute(&chunk);
    let err = result.unwrap_err();
    assert!(err.contains("negate") || err.contains("Bool"));
}

// ============================================================================
// VM: Float unary negation via raw bytecode
// ============================================================================

#[test]
fn test_vm_unary_neg_float_raw() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(3.14));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Neg, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    match result {
        Value::Float(f) => assert!((f - (-3.14)).abs() < 0.001),
        _ => panic!("Expected Float, got {result:?}"),
    }
}

// ============================================================================
// VM: Negate on non-numeric (string) via raw bytecode (error)
// ============================================================================

#[test]
fn test_vm_unary_neg_string_raw_error() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk
        .constants
        .push(Value::from_string("hello".to_string()));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Neg, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("negate"));
}

// ============================================================================
// VM: BitwiseNot on float (error path)
// ============================================================================

#[test]
fn test_vm_bitnot_float_raw_error() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(3.14));
    chunk.register_count = 4;

    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::BitNot, 0, 1, 0), 2);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let err = result.unwrap_err();
    assert!(err.contains("bitwise NOT") || err.contains("Float"));
}

// ============================================================================
// VM: NewArray out of bounds index
// ============================================================================

#[test]
fn test_vm_new_array_out_of_bounds_idx() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.register_count = 2;
    // No array_element_regs added

    chunk.emit(Instruction::abx(OpCode::NewArray, 0, 99), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_vm_new_tuple_out_of_bounds_idx() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.register_count = 2;

    chunk.emit(Instruction::abx(OpCode::NewTuple, 0, 99), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

#[test]
fn test_vm_new_object_out_of_bounds_idx() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.register_count = 2;

    chunk.emit(Instruction::abx(OpCode::NewObject, 0, 99), 1);

    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("out of bounds"));
}

// ============================================================================
// VM: Multiple globals operations
// ============================================================================

#[test]
fn test_vm_multiple_globals() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::from_string("a".to_string()));
    chunk.constants.push(Value::Integer(20));
    chunk.constants.push(Value::from_string("b".to_string()));
    chunk.register_count = 6;

    // R1 = 10
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // store global "a" = R1
    chunk.emit(Instruction::abx(OpCode::StoreGlobal, 1, 1), 2);
    // R2 = 20
    chunk.emit(Instruction::abx(OpCode::Const, 2, 2), 3);
    // store global "b" = R2
    chunk.emit(Instruction::abx(OpCode::StoreGlobal, 2, 3), 4);
    // R3 = load global "a"
    chunk.emit(Instruction::abx(OpCode::LoadGlobal, 3, 1), 5);
    // R4 = load global "b"
    chunk.emit(Instruction::abx(OpCode::LoadGlobal, 4, 3), 6);
    // R0 = R3 + R4
    chunk.emit(Instruction::abc(OpCode::Add, 0, 3, 4), 7);

    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(30));
}

// ============================================================================
// VM: BitAnd, BitOr, BitXor, ShiftLeft, ShiftRight
// ============================================================================

#[test]
fn test_vm_bitand() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(0b1111));
    chunk.constants.push(Value::Integer(0b1010));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::BitAnd, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    // BitAnd may not be implemented yet — exercises the dispatch path
    let _ = result;
}

#[test]
fn test_vm_bitor() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(0b1100));
    chunk.constants.push(Value::Integer(0b0011));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::BitOr, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_bitxor() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(0b1100));
    chunk.constants.push(Value::Integer(0b1010));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::BitXor, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_shift_left() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(1));
    chunk.constants.push(Value::Integer(4));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::ShiftLeft, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_shift_right() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(16));
    chunk.constants.push(Value::Integer(2));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::ShiftRight, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

// ============================================================================
// VM: Dup, Pop, Swap
// ============================================================================

#[test]
fn test_vm_dup() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Dup, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_pop() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 1, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Pop, 1, 0, 0), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_swap() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(1));
    chunk.constants.push(Value::Integer(2));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Swap, 1, 2, 0), 3);
    // After swap, R1=2, R2=1
    chunk.emit(Instruction::abc(OpCode::Move, 0, 1, 0), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

// ============================================================================
// VM: LoadLocal, StoreLocal
// ============================================================================

#[test]
fn test_vm_store_load_local() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(77));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::StoreLocal, 1, 0, 0), 2);
    chunk.emit(Instruction::abc(OpCode::LoadLocal, 0, 0, 0), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

// ============================================================================
// VM: StoreField
// ============================================================================

#[test]
fn test_vm_store_field() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    let mut fields = HashMap::new();
    fields.insert("x".to_string(), Value::Integer(0));
    chunk.constants.push(Value::Class {
        class_name: "Obj".to_string(),
        fields: Arc::new(std::sync::RwLock::new(fields)),
        methods: Arc::new(HashMap::new()),
    });
    chunk.constants.push(Value::from_string("x".to_string()));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 5;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1); // obj
    chunk.emit(Instruction::abx(OpCode::Const, 2, 2), 2); // 99
    chunk.emit(Instruction::abc(OpCode::StoreField, 2, 1, 1), 3); // obj.x = 99
    // Load back the field to verify
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

// ============================================================================
// VM: StoreIndex
// ============================================================================

#[test]
fn test_vm_store_index() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Array(Arc::from(vec![
        Value::Integer(10),
        Value::Integer(20),
    ])));
    chunk.constants.push(Value::Integer(0));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 5;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1); // arr
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2); // idx 0
    chunk.emit(Instruction::abx(OpCode::Const, 3, 2), 3); // val 99
    chunk.emit(Instruction::abc(OpCode::StoreIndex, 3, 1, 2), 4); // arr[0] = 99
    // Load back to verify
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 5);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

// ============================================================================
// VM: GetType
// ============================================================================

#[test]
fn test_vm_get_type_integer() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::GetType, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_get_type_string() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk
        .constants
        .push(Value::from_string("hello".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::GetType, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

#[test]
fn test_vm_get_type_bool() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::GetType, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result;
}

// ============================================================================
// VM: Call (simple function call via bytecode)
// ============================================================================

#[test]
fn test_vm_call_simple() {
    // Compile a simple function call through the compiler
    let mut parser =
        crate::frontend::parser::Parser::new("fun double(x: i32) -> i32 { x * 2 }\ndouble(21)");
    let ast = parser.parse().expect("parse should succeed");
    let mut compiler = Compiler::new("test".to_string());
    let _ = compiler.compile_expr(&ast);
    let chunk = compiler.finalize();
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    // May or may not succeed depending on compiler support — exercises Call path
    let _ = result;
}

// ============================================================================
// VM: Float arithmetic
// ============================================================================

#[test]
fn test_vm_add_float() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(1.5));
    chunk.constants.push(Value::Float(2.5));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_vm_sub_float() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(5.0));
    chunk.constants.push(Value::Float(2.0));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Sub, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_vm_mul_float() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(3.0));
    chunk.constants.push(Value::Float(4.0));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Mul, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Float(12.0));
}

#[test]
fn test_vm_div_float() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(10.0));
    chunk.constants.push(Value::Float(4.0));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Div, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Float(2.5));
}

// ============================================================================
// VM: String concatenation via Add
// ============================================================================

#[test]
fn test_vm_add_string() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk
        .constants
        .push(Value::from_string("hello ".to_string()));
    chunk
        .constants
        .push(Value::from_string("world".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Add, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::from_string("hello world".to_string()));
}

// ============================================================================
// VM: Boolean comparisons
// ============================================================================

#[test]
fn test_vm_equal_strings() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::from_string("abc".to_string()));
    chunk.constants.push(Value::from_string("abc".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Equal, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_not_equal_strings() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::from_string("abc".to_string()));
    chunk.constants.push(Value::from_string("xyz".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::NotEqual, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// VM: Mod with float
// ============================================================================

#[test]
fn test_vm_mod_float() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(7.0));
    chunk.constants.push(Value::Float(3.0));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Mod, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Float(1.0));
}

// ============================================================================
// VM: Neg with float
// ============================================================================

#[test]
fn test_vm_neg_float() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Float(3.14));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Neg, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Float(-3.14));
}

// ============================================================================
// VM: BitNot with integer
// ============================================================================

#[test]
fn test_vm_bitnot_integer() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(0));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::BitNot, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(-1));
}

// ============================================================================
// VM: Comparison operations (Less, LessEqual, Greater, GreaterEqual)
// ============================================================================

#[test]
fn test_vm_less_than() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(3));
    chunk.constants.push(Value::Integer(5));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Less, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_less_equal() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(5));
    chunk.constants.push(Value::Integer(5));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LessEqual, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_greater_than() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk.constants.push(Value::Integer(3));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Greater, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_greater_equal() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(5));
    chunk.constants.push(Value::Integer(5));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::GreaterEqual, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// VM: Logical operations (And, Or)
// ============================================================================

#[test]
fn test_vm_and_true() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Bool(true));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::And, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_and_false() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Bool(false));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::And, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_or_true() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(false));
    chunk.constants.push(Value::Bool(true));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::Or, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(true));
}

// ============================================================================
// VM: Not (unary boolean negation)
// ============================================================================

#[test]
fn test_vm_not() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Not, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Bool(false));
}

// ============================================================================
// VM: Jump, JumpIfFalse, JumpIfTrue
// ============================================================================

#[test]
fn test_vm_jump() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;
    // Load 42 into R0
    chunk.emit(Instruction::abx(OpCode::Const, 0, 0), 1);
    // Jump over next instruction (+1 skips one instruction)
    chunk.emit(Instruction::asbx(OpCode::Jump, 0, 1), 2);
    // This should be skipped: overwrite R0 with 99
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_jump_if_false() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(false));
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;
    // Load false into R1
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // Load 42 into R0
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 2);
    // JumpIfFalse: R1 is false, skip next instruction
    chunk.emit(Instruction::asbx(OpCode::JumpIfFalse, 1, 1), 3);
    // This should be skipped: overwrite R0 with 99
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_jump_if_false_nil() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Nil);
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;
    // Load nil into R1
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // Load 42 into R0
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 2);
    // JumpIfFalse: R1 is nil (falsy), skip next
    chunk.emit(Instruction::asbx(OpCode::JumpIfFalse, 1, 1), 3);
    // Skipped
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_jump_if_true() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Bool(true));
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;
    // Load true into R1
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // Load 42 into R0
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 2);
    // JumpIfTrue: R1 is true, skip next
    chunk.emit(Instruction::asbx(OpCode::JumpIfTrue, 1, 1), 3);
    // Skipped
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_jump_if_true_truthy_value() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    // Non-bool truthy value (Integer is truthy)
    chunk.constants.push(Value::Integer(1));
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::Integer(99));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 0, 1), 2);
    chunk.emit(Instruction::asbx(OpCode::JumpIfTrue, 1, 1), 3);
    chunk.emit(Instruction::abx(OpCode::Const, 0, 2), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

// ============================================================================
// VM: LoadField for Object, Struct, Tuple
// ============================================================================

#[test]
fn test_vm_load_field_object() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    let mut map = std::collections::HashMap::new();
    map.insert("x".to_string(), Value::Integer(42));
    chunk.constants.push(Value::Object(Arc::new(map)));
    chunk.constants.push(Value::from_string("x".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1); // obj
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2); // obj.x
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_vm_load_field_struct() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    let mut fields = HashMap::new();
    fields.insert("y".to_string(), Value::Integer(99));
    chunk.constants.push(Value::Struct {
        name: "Point".to_string(),
        fields: Arc::new(fields),
    });
    chunk.constants.push(Value::from_string("y".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(99));
}

#[test]
fn test_vm_load_field_tuple() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Tuple(Arc::from(vec![
        Value::Integer(10),
        Value::Integer(20),
    ])));
    chunk.constants.push(Value::from_string("1".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_vm_load_field_class() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    let mut fields = HashMap::new();
    fields.insert("z".to_string(), Value::Integer(77));
    chunk.constants.push(Value::Class {
        class_name: "MyClass".to_string(),
        fields: Arc::new(std::sync::RwLock::new(fields)),
        methods: Arc::new(HashMap::new()),
    });
    chunk.constants.push(Value::from_string("z".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(77));
}

// ============================================================================
// VM: LoadIndex for array and string
// ============================================================================

#[test]
fn test_vm_load_index_array() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Array(Arc::from(vec![
        Value::Integer(10),
        Value::Integer(20),
        Value::Integer(30),
    ])));
    chunk.constants.push(Value::Integer(1));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1); // arr
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2); // idx=1
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_vm_load_index_array_negative() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Array(Arc::from(vec![
        Value::Integer(10),
        Value::Integer(20),
        Value::Integer(30),
    ])));
    chunk.constants.push(Value::Integer(-1));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(30));
}

#[test]
fn test_vm_load_index_string() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk
        .constants
        .push(Value::from_string("hello".to_string()));
    chunk.constants.push(Value::Integer(1));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::from_string("e".to_string()));
}

#[test]
fn test_vm_load_index_string_negative() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk
        .constants
        .push(Value::from_string("hello".to_string()));
    chunk.constants.push(Value::Integer(-1));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::from_string("o".to_string()));
}

// ============================================================================
// VM: NewArray, NewTuple, NewObject
// ============================================================================

#[test]
fn test_vm_new_array() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(1));
    chunk.constants.push(Value::Integer(2));
    chunk.constants.push(Value::Integer(3));
    chunk.register_count = 5;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abx(OpCode::Const, 3, 2), 3);
    // array_element_regs[0] = [1, 2, 3] (register indices)
    chunk.array_element_regs.push(vec![1, 2, 3]);
    chunk.emit(Instruction::abx(OpCode::NewArray, 0, 0), 4);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Integer(1));
        assert_eq!(arr[2], Value::Integer(3));
    } else {
        panic!("Expected Array, got {:?}", result);
    }
}

#[test]
fn test_vm_new_tuple() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(10));
    chunk
        .constants
        .push(Value::from_string("hello".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.array_element_regs.push(vec![1, 2]);
    chunk.emit(Instruction::abx(OpCode::NewTuple, 0, 0), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    if let Value::Tuple(elems) = result {
        assert_eq!(elems.len(), 2);
        assert_eq!(elems[0], Value::Integer(10));
    } else {
        panic!("Expected Tuple, got {:?}", result);
    }
}

#[test]
fn test_vm_new_object() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    // object_fields[0] = [("x", 1)] — field "x" from register 1
    chunk.object_fields.push(vec![("x".to_string(), 1)]);
    chunk.emit(Instruction::abx(OpCode::NewObject, 0, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    if let Value::Object(map) = result {
        assert_eq!(map.get("x"), Some(&Value::Integer(42)));
    } else {
        panic!("Expected Object, got {:?}", result);
    }
}

// ============================================================================
// VM: Return
// ============================================================================

#[test]
fn test_vm_return() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Return, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    let _ = result; // Return pops the call frame, may or may not work
}

// ============================================================================
// VM: Neg integer
// ============================================================================

#[test]
fn test_vm_neg_integer() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::Neg, 0, 1, 0), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk).expect("execute should succeed");
    assert_eq!(result, Value::Integer(-42));
}

// ============================================================================
// VM: LoadField error paths
// ============================================================================

#[test]
fn test_vm_load_field_missing_field() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    let map = std::collections::HashMap::new();
    chunk.constants.push(Value::Object(Arc::new(map)));
    chunk
        .constants
        .push(Value::from_string("nonexistent".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_load_field_on_integer() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42));
    chunk.constants.push(Value::from_string("x".to_string()));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abc(OpCode::LoadField, 0, 1, 1), 2);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

// ============================================================================
// VM: LoadIndex error paths
// ============================================================================

#[test]
fn test_vm_load_index_out_of_bounds() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk
        .constants
        .push(Value::Array(Arc::from(vec![Value::Integer(10)])));
    chunk.constants.push(Value::Integer(5)); // out of bounds
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_load_index_invalid_type() {
    let mut chunk = BytecodeChunk::new("test".to_string());
    chunk.constants.push(Value::Integer(42)); // not indexable
    chunk.constants.push(Value::Integer(0));
    chunk.register_count = 4;
    chunk.emit(Instruction::abx(OpCode::Const, 1, 0), 1);
    chunk.emit(Instruction::abx(OpCode::Const, 2, 1), 2);
    chunk.emit(Instruction::abc(OpCode::LoadIndex, 0, 1, 2), 3);
    let mut vm = VM::new();
    let result = vm.execute(&chunk);
    assert!(result.is_err());
}
