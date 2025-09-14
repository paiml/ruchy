//! Tests for MIR (Middle Intermediate Representation) module

use ruchy::middleend::mir::types::{MirFunction, MirModule, MirType, MirInstruction, MirBasicBlock};
use ruchy::middleend::mir::builder::MirBuilder;

#[test]
fn test_mir_module_creation() {
    let module = MirModule::new("test_module");
    assert_eq!(module.name, "test_module");
    assert!(module.functions.is_empty());
    assert!(module.globals.is_empty());
}

#[test]
fn test_mir_function_creation() {
    let func = MirFunction::new(
        "test_func",
        vec![MirType::I32, MirType::I32],
        MirType::I32,
    );
    
    assert_eq!(func.name, "test_func");
    assert_eq!(func.params.len(), 2);
    assert!(matches!(func.params[0], MirType::I32));
    assert!(matches!(func.params[1], MirType::I32));
    assert!(matches!(func.return_type, MirType::I32));
}

#[test]
fn test_mir_builder_new() {
    let builder = MirBuilder::new();
    assert!(builder.current_function.is_none());
    assert!(builder.current_block.is_none());
}

#[test]
fn test_mir_types() {
    let i32_type = MirType::I32;
    let i64_type = MirType::I64;
    let f32_type = MirType::F32;
    let f64_type = MirType::F64;
    let bool_type = MirType::Bool;
    let void_type = MirType::Void;
    
    assert_eq!(format!("{:?}", i32_type), "I32");
    assert_eq!(format!("{:?}", i64_type), "I64");
    assert_eq!(format!("{:?}", f32_type), "F32");
    assert_eq!(format!("{:?}", f64_type), "F64");
    assert_eq!(format!("{:?}", bool_type), "Bool");
    assert_eq!(format!("{:?}", void_type), "Void");
}

#[test]
fn test_mir_pointer_type() {
    let ptr_type = MirType::Pointer(Box::new(MirType::I32));
    
    match ptr_type {
        MirType::Pointer(inner) => {
            assert!(matches!(*inner, MirType::I32));
        }
        _ => panic!("Expected pointer type"),
    }
}

#[test]
fn test_mir_array_type() {
    let array_type = MirType::Array(Box::new(MirType::I32), 10);
    
    match array_type {
        MirType::Array(elem, size) => {
            assert!(matches!(*elem, MirType::I32));
            assert_eq!(size, 10);
        }
        _ => panic!("Expected array type"),
    }
}

#[test]
fn test_mir_struct_type() {
    let fields = vec![MirType::I32, MirType::Bool, MirType::F64];
    let struct_type = MirType::Struct(fields);
    
    match struct_type {
        MirType::Struct(f) => {
            assert_eq!(f.len(), 3);
            assert!(matches!(f[0], MirType::I32));
            assert!(matches!(f[1], MirType::Bool));
            assert!(matches!(f[2], MirType::F64));
        }
        _ => panic!("Expected struct type"),
    }
}

#[test]
fn test_mir_basic_block() {
    let mut block = MirBasicBlock::new("entry");
    
    assert_eq!(block.label, "entry");
    assert!(block.instructions.is_empty());
    assert!(block.terminator.is_none());
    
    // Add an instruction
    block.instructions.push(MirInstruction::Nop);
    assert_eq!(block.instructions.len(), 1);
}

#[test]
fn test_mir_instructions() {
    let nop = MirInstruction::Nop;
    let const_i32 = MirInstruction::Const(42);
    let add = MirInstruction::Add(0, 1);
    let sub = MirInstruction::Sub(0, 1);
    let mul = MirInstruction::Mul(0, 1);
    let div = MirInstruction::Div(0, 1);
    
    assert!(matches!(nop, MirInstruction::Nop));
    assert!(matches!(const_i32, MirInstruction::Const(42)));
    assert!(matches!(add, MirInstruction::Add(0, 1)));
    assert!(matches!(sub, MirInstruction::Sub(0, 1)));
    assert!(matches!(mul, MirInstruction::Mul(0, 1)));
    assert!(matches!(div, MirInstruction::Div(0, 1)));
}

#[test]
fn test_mir_comparison_instructions() {
    let eq = MirInstruction::Eq(0, 1);
    let ne = MirInstruction::Ne(0, 1);
    let lt = MirInstruction::Lt(0, 1);
    let le = MirInstruction::Le(0, 1);
    let gt = MirInstruction::Gt(0, 1);
    let ge = MirInstruction::Ge(0, 1);
    
    assert!(matches!(eq, MirInstruction::Eq(0, 1)));
    assert!(matches!(ne, MirInstruction::Ne(0, 1)));
    assert!(matches!(lt, MirInstruction::Lt(0, 1)));
    assert!(matches!(le, MirInstruction::Le(0, 1)));
    assert!(matches!(gt, MirInstruction::Gt(0, 1)));
    assert!(matches!(ge, MirInstruction::Ge(0, 1)));
}

#[test]
fn test_mir_control_flow() {
    let ret = MirInstruction::Return(Some(0));
    let ret_void = MirInstruction::Return(None);
    let br = MirInstruction::Branch("loop");
    let br_cond = MirInstruction::BranchCond(0, "then", "else");
    
    assert!(matches!(ret, MirInstruction::Return(Some(0))));
    assert!(matches!(ret_void, MirInstruction::Return(None)));
    
    match br {
        MirInstruction::Branch(label) => assert_eq!(label, "loop"),
        _ => panic!("Expected branch"),
    }
    
    match br_cond {
        MirInstruction::BranchCond(cond, then_label, else_label) => {
            assert_eq!(cond, 0);
            assert_eq!(then_label, "then");
            assert_eq!(else_label, "else");
        }
        _ => panic!("Expected conditional branch"),
    }
}

#[test]
fn test_mir_memory_instructions() {
    let alloca = MirInstruction::Alloca(MirType::I32);
    let load = MirInstruction::Load(0);
    let store = MirInstruction::Store(0, 1);
    
    match alloca {
        MirInstruction::Alloca(ty) => assert!(matches!(ty, MirType::I32)),
        _ => panic!("Expected alloca"),
    }
    
    assert!(matches!(load, MirInstruction::Load(0)));
    assert!(matches!(store, MirInstruction::Store(0, 1)));
}

#[test]
fn test_mir_call_instruction() {
    let call = MirInstruction::Call("printf", vec![0, 1, 2]);
    
    match call {
        MirInstruction::Call(name, args) => {
            assert_eq!(name, "printf");
            assert_eq!(args.len(), 3);
            assert_eq!(args[0], 0);
            assert_eq!(args[1], 1);
            assert_eq!(args[2], 2);
        }
        _ => panic!("Expected call instruction"),
    }
}

#[test]
fn test_mir_builder_operations() {
    let mut builder = MirBuilder::new();
    let mut module = MirModule::new("test");
    
    // Start a function
    builder.start_function("add", vec![MirType::I32, MirType::I32], MirType::I32);
    
    // Create entry block
    builder.start_block("entry");
    
    // Add instructions
    let a = builder.emit_load(0);
    let b = builder.emit_load(1);
    let sum = builder.emit_add(a, b);
    builder.emit_return(Some(sum));
    
    // Finish block and function
    builder.finish_block();
    let func = builder.finish_function();
    
    assert_eq!(func.name, "add");
    assert_eq!(func.params.len(), 2);
    assert_eq!(func.blocks.len(), 1);
    
    module.functions.push(func);
    assert_eq!(module.functions.len(), 1);
}

// Property-based tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_mir_module_name(name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
            let module = MirModule::new(&name);
            prop_assert_eq!(module.name, name);
        }
        
        #[test]
        fn prop_mir_function_params(count in 0usize..10) {
            let params = vec![MirType::I32; count];
            let func = MirFunction::new("test", params.clone(), MirType::Void);
            
            prop_assert_eq!(func.params.len(), count);
            for param in func.params {
                prop_assert!(matches!(param, MirType::I32));
            }
        }
        
        #[test]
        fn prop_mir_array_size(size in 1usize..1000) {
            let array_type = MirType::Array(Box::new(MirType::Bool), size);
            
            match array_type {
                MirType::Array(_, s) => prop_assert_eq!(s, size),
                _ => prop_assert!(false, "Expected array type"),
            }
        }
    }
}