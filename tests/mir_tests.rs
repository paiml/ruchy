//! Tests for MIR (Middle Intermediate Representation) module

// NOTE: Tests disabled due to API mismatch - the test expects types like MirModule
// which don't exist in the current MIR implementation
#[cfg(test)]
mod enabled_tests {

    use ruchy::middleend::mir::MirBuilder;
    use ruchy::middleend::mir::{
        BasicBlock, BlockId, Function, Local, Program, Statement, Terminator, Type,
    };

    #[test]
    fn test_mir_program_creation() {
        use std::collections::HashMap;
        let program = Program {
            functions: HashMap::new(),
            entry: "main".to_string(),
        };
        assert_eq!(program.entry, "main");
        assert!(program.functions.is_empty());
    }

    #[test]
    fn test_mir_function_creation() {
        let func = Function {
            name: "test_func".to_string(),
            params: vec![Local(0), Local(1)],
            return_ty: Type::I32,
            locals: vec![],
            blocks: vec![],
            entry_block: BlockId(0),
        };

        assert_eq!(func.name, "test_func");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0], Local(0));
        assert_eq!(func.params[1], Local(1));
        assert!(matches!(func.return_ty, Type::I32));
    }

    #[test]
    fn test_mir_builder_new() {
        let _builder = MirBuilder::new();
        // Builder structure is private, so we can only test it creates without panic
        // This is a minimal test to ensure construction works
        assert!(true); // If we get here, construction succeeded
    }

    #[test]
    fn test_mir_types() {
        let i32_type = Type::I32;
        let i64_type = Type::I64;
        let f32_type = Type::F32;
        let f64_type = Type::F64;
        let bool_type = Type::Bool;
        let unit_type = Type::Unit;

        assert_eq!(format!("{:?}", i32_type), "I32");
        assert_eq!(format!("{:?}", i64_type), "I64");
        assert_eq!(format!("{:?}", f32_type), "F32");
        assert_eq!(format!("{:?}", f64_type), "F64");
        assert_eq!(format!("{:?}", bool_type), "Bool");
        assert_eq!(format!("{:?}", unit_type), "Unit");
    }

    #[test]
    fn test_mir_ref_type() {
        use ruchy::middleend::mir::Mutability;
        let ref_type = Type::Ref(Box::new(Type::I32), Mutability::Immutable);

        match ref_type {
            Type::Ref(inner, mutability) => {
                assert!(matches!(*inner, Type::I32));
                assert_eq!(mutability, Mutability::Immutable);
            }
            _ => panic!("Expected reference type"),
        }
    }

    #[test]
    fn test_mir_array_type() {
        let array_type = Type::Array(Box::new(Type::I32), 10);

        match array_type {
            Type::Array(elem, size) => {
                assert!(matches!(*elem, Type::I32));
                assert_eq!(size, 10);
            }
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_mir_tuple_type() {
        let fields = vec![Type::I32, Type::Bool, Type::F64];
        let tuple_type = Type::Tuple(fields);

        match tuple_type {
            Type::Tuple(f) => {
                assert_eq!(f.len(), 3);
                assert!(matches!(f[0], Type::I32));
                assert!(matches!(f[1], Type::Bool));
                assert!(matches!(f[2], Type::F64));
            }
            _ => panic!("Expected tuple type"),
        }
    }

    #[test]
    fn test_mir_basic_block() {
        let mut block = BasicBlock {
            id: BlockId(0),
            statements: vec![],
            terminator: Terminator::Return(None),
        };

        assert_eq!(block.id, BlockId(0));
        assert!(block.statements.is_empty());
        assert!(matches!(block.terminator, Terminator::Return(None)));

        // Add a statement
        block.statements.push(Statement::Nop);
        assert_eq!(block.statements.len(), 1);
    }

    #[test]
    fn test_mir_statements() {
        use ruchy::middleend::mir::{Constant, Operand, Place, Rvalue};

        let nop = Statement::Nop;
        let storage_live = Statement::StorageLive(Local(0));
        let assign = Statement::Assign(
            Place::Local(Local(0)),
            Rvalue::Use(Operand::Constant(Constant::Int(42, Type::I32))),
        );

        assert!(matches!(nop, Statement::Nop));
        assert!(matches!(storage_live, Statement::StorageLive(_)));
        assert!(matches!(assign, Statement::Assign(_, _)));
    }

    #[test]
    fn test_mir_binary_operations() {
        use ruchy::middleend::mir::{BinOp, Constant, Operand, Rvalue};

        let add = Rvalue::BinaryOp(
            BinOp::Add,
            Operand::Constant(Constant::Int(1, Type::I32)),
            Operand::Constant(Constant::Int(2, Type::I32)),
        );
        let eq = Rvalue::BinaryOp(
            BinOp::Eq,
            Operand::Constant(Constant::Int(1, Type::I32)),
            Operand::Constant(Constant::Int(1, Type::I32)),
        );

        assert!(matches!(add, Rvalue::BinaryOp(BinOp::Add, _, _)));
        assert!(matches!(eq, Rvalue::BinaryOp(BinOp::Eq, _, _)));
    }

    #[test]
    fn test_mir_terminators() {
        use ruchy::middleend::mir::{Constant, Operand};

        let ret = Terminator::Return(Some(Operand::Constant(Constant::Int(42, Type::I32))));
        let ret_void = Terminator::Return(None);
        let goto = Terminator::Goto(BlockId(1));
        let if_term = Terminator::If {
            condition: Operand::Constant(Constant::Bool(true)),
            then_block: BlockId(2),
            else_block: BlockId(3),
        };

        assert!(matches!(ret, Terminator::Return(Some(_))));
        assert!(matches!(ret_void, Terminator::Return(None)));
        assert!(matches!(goto, Terminator::Goto(_)));
        assert!(matches!(if_term, Terminator::If { .. }));
    }

    #[test]
    fn test_mir_constants() {
        use ruchy::middleend::mir::Constant;

        let int_const = Constant::Int(42, Type::I64);
        let bool_const = Constant::Bool(true);
        let string_const = Constant::String("hello".to_string());
        let unit_const = Constant::Unit;

        assert!(matches!(int_const, Constant::Int(42, Type::I64)));
        assert!(matches!(bool_const, Constant::Bool(true)));
        assert!(matches!(string_const, Constant::String(_)));
        assert!(matches!(unit_const, Constant::Unit));
    }

    #[test]
    fn test_mir_places() {
        use ruchy::middleend::mir::{FieldIdx, Place};

        let local_place = Place::Local(Local(0));
        let field_place = Place::Field(Box::new(Place::Local(Local(0))), FieldIdx(1));
        let deref_place = Place::Deref(Box::new(Place::Local(Local(0))));

        assert!(matches!(local_place, Place::Local(_)));
        assert!(matches!(field_place, Place::Field(_, _)));
        assert!(matches!(deref_place, Place::Deref(_)));
    }

    // Property-based tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;
        use std::collections::HashMap;

        proptest! {
            #[test]
            fn prop_mir_program_entry(name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}") {
                let program = Program {
                    functions: HashMap::new(),
                    entry: name.clone(),
                };
                prop_assert_eq!(program.entry, name);
            }

            #[test]
            fn prop_mir_function_params(count in 0usize..10) {
                let params: Vec<Local> = (0..count).map(Local).collect();
                let func = Function {
                    name: "test".to_string(),
                    params: params.clone(),
                    return_ty: Type::Unit,
                    locals: vec![],
                    blocks: vec![],
                    entry_block: BlockId(0),
                };

                prop_assert_eq!(func.params.len(), count);
                for (i, param) in func.params.iter().enumerate() {
                    prop_assert_eq!(*param, Local(i));
                }
            }

            #[test]
            fn prop_mir_array_size(size in 1usize..1000) {
                let array_type = Type::Array(Box::new(Type::Bool), size);

                match array_type {
                    Type::Array(_, s) => prop_assert_eq!(s, size),
                    _ => prop_assert!(false, "Expected array type"),
                }
            }
        }
    }
} // End enabled_tests module
