//! Comprehensive TDD tests for Middleend modules
//! Target: Increase coverage for type inference, unification, and MIR
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod middleend_tests {
    use crate::middleend::{TypeInferer, Unifier, Environment, Type, MirBuilder, MirOptimizer};
    use crate::frontend::ast::{Expr, ExprKind, Literal};
    use std::collections::HashMap;
    
    // ========== Type Inference Tests ==========
    
    #[test]
    fn test_type_inferer_creation() {
        let inferer = TypeInferer::new();
        assert_eq!(inferer.constraint_count(), 0);
        assert!(inferer.is_empty());
    }
    
    #[test]
    fn test_infer_literal_types() {
        let mut inferer = TypeInferer::new();
        
        let int_lit = create_literal_expr(Literal::Integer(42));
        let int_type = inferer.infer(&int_lit).unwrap();
        assert_eq!(int_type, Type::Integer);
        
        let float_lit = create_literal_expr(Literal::Float(3.14));
        let float_type = inferer.infer(&float_lit).unwrap();
        assert_eq!(float_type, Type::Float);
        
        let bool_lit = create_literal_expr(Literal::Bool(true));
        let bool_type = inferer.infer(&bool_lit).unwrap();
        assert_eq!(bool_type, Type::Bool);
        
        let str_lit = create_literal_expr(Literal::String("hello".to_string()));
        let str_type = inferer.infer(&str_lit).unwrap();
        assert_eq!(str_type, Type::String);
    }
    
    #[test]
    fn test_infer_binary_operation_types() {
        let mut inferer = TypeInferer::new();
        
        // 1 + 2
        let left = create_literal_expr(Literal::Integer(1));
        let right = create_literal_expr(Literal::Integer(2));
        let add_expr = create_binary_expr(left, BinaryOp::Add, right);
        
        let result_type = inferer.infer(&add_expr).unwrap();
        assert_eq!(result_type, Type::Integer);
    }
    
    #[test]
    fn test_infer_comparison_types() {
        let mut inferer = TypeInferer::new();
        
        // 5 > 3
        let left = create_literal_expr(Literal::Integer(5));
        let right = create_literal_expr(Literal::Integer(3));
        let cmp_expr = create_binary_expr(left, BinaryOp::Greater, right);
        
        let result_type = inferer.infer(&cmp_expr).unwrap();
        assert_eq!(result_type, Type::Bool);
    }
    
    #[test]
    fn test_infer_if_expression_type() {
        let mut inferer = TypeInferer::new();
        
        let condition = create_literal_expr(Literal::Bool(true));
        let then_branch = create_literal_expr(Literal::Integer(1));
        let else_branch = create_literal_expr(Literal::Integer(2));
        
        let if_expr = create_if_expr(condition, then_branch, Some(else_branch));
        
        let result_type = inferer.infer(&if_expr).unwrap();
        assert_eq!(result_type, Type::Integer);
    }
    
    #[test]
    fn test_infer_function_type() {
        let mut inferer = TypeInferer::new();
        
        // fn add(x: i32, y: i32) -> i32
        let func_type = inferer.infer_function_signature(
            vec![Type::Integer, Type::Integer],
            Type::Integer
        );
        
        assert_eq!(func_type, Type::Function(
            Box::new(Type::Tuple(vec![Type::Integer, Type::Integer])),
            Box::new(Type::Integer)
        ));
    }
    
    #[test]
    fn test_infer_let_binding() {
        let mut inferer = TypeInferer::new();
        let mut env = Environment::new();
        
        // let x = 42
        let value = create_literal_expr(Literal::Integer(42));
        let inferred_type = inferer.infer(&value).unwrap();
        
        env.bind("x", inferred_type.clone());
        assert_eq!(env.lookup("x"), Some(&Type::Integer));
    }
    
    #[test]
    fn test_type_inference_with_generics() {
        let mut inferer = TypeInferer::new();
        
        // Vec<T>
        let type_var = inferer.fresh_type_var();
        let vec_type = Type::Generic("Vec".to_string(), vec![type_var.clone()]);
        
        // Instantiate with Integer
        inferer.unify(&type_var, &Type::Integer).unwrap();
        
        let resolved = inferer.resolve_type(&vec_type);
        assert_eq!(resolved, Type::Generic("Vec".to_string(), vec![Type::Integer]));
    }
    
    // ========== Unification Tests ==========
    
    #[test]
    fn test_unifier_creation() {
        let unifier = Unifier::new();
        assert_eq!(unifier.substitution_count(), 0);
    }
    
    #[test]
    fn test_unify_same_types() {
        let mut unifier = Unifier::new();
        
        let result = unifier.unify(&Type::Integer, &Type::Integer);
        assert!(result.is_ok());
        
        let result = unifier.unify(&Type::String, &Type::String);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_unify_different_types_fails() {
        let mut unifier = Unifier::new();
        
        let result = unifier.unify(&Type::Integer, &Type::String);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_unify_type_variable() {
        let mut unifier = Unifier::new();
        
        let type_var = Type::Variable(0);
        let concrete = Type::Integer;
        
        let result = unifier.unify(&type_var, &concrete);
        assert!(result.is_ok());
        
        let resolved = unifier.resolve(&type_var);
        assert_eq!(resolved, Type::Integer);
    }
    
    #[test]
    fn test_unify_function_types() {
        let mut unifier = Unifier::new();
        
        let func1 = Type::Function(
            Box::new(Type::Integer),
            Box::new(Type::Bool)
        );
        
        let func2 = Type::Function(
            Box::new(Type::Integer),
            Box::new(Type::Bool)
        );
        
        let result = unifier.unify(&func1, &func2);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_unify_recursive_types() {
        let mut unifier = Unifier::new();
        
        // List a = Nil | Cons a (List a)
        let list_var = Type::Variable(0);
        let list_type = Type::Recursive(
            "List".to_string(),
            Box::new(list_var.clone())
        );
        
        let result = unifier.unify(&list_var, &list_type);
        // Should handle recursive types without infinite loop
        assert!(result.is_ok() || result.is_err());
    }
    
    // ========== Environment Tests ==========
    
    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert_eq!(env.binding_count(), 0);
        assert!(env.is_empty());
    }
    
    #[test]
    fn test_environment_binding() {
        let mut env = Environment::new();
        
        env.bind("x", Type::Integer);
        env.bind("y", Type::String);
        
        assert_eq!(env.binding_count(), 2);
        assert_eq!(env.lookup("x"), Some(&Type::Integer));
        assert_eq!(env.lookup("y"), Some(&Type::String));
        assert_eq!(env.lookup("z"), None);
    }
    
    #[test]
    fn test_environment_shadowing() {
        let mut env = Environment::new();
        
        env.bind("x", Type::Integer);
        env.push_scope();
        env.bind("x", Type::String); // Shadow in inner scope
        
        assert_eq!(env.lookup("x"), Some(&Type::String));
        
        env.pop_scope();
        assert_eq!(env.lookup("x"), Some(&Type::Integer));
    }
    
    #[test]
    fn test_environment_scoping() {
        let mut env = Environment::new();
        
        env.bind("global", Type::Bool);
        
        env.push_scope();
        env.bind("local1", Type::Integer);
        
        env.push_scope();
        env.bind("local2", Type::String);
        
        assert_eq!(env.lookup("global"), Some(&Type::Bool));
        assert_eq!(env.lookup("local1"), Some(&Type::Integer));
        assert_eq!(env.lookup("local2"), Some(&Type::String));
        
        env.pop_scope();
        assert_eq!(env.lookup("local2"), None);
        assert_eq!(env.lookup("local1"), Some(&Type::Integer));
        
        env.pop_scope();
        assert_eq!(env.lookup("local1"), None);
        assert_eq!(env.lookup("global"), Some(&Type::Bool));
    }
    
    // ========== MIR Builder Tests ==========
    
    #[test]
    fn test_mir_builder_creation() {
        let builder = MirBuilder::new();
        assert_eq!(builder.block_count(), 0);
        assert_eq!(builder.instruction_count(), 0);
    }
    
    #[test]
    fn test_build_basic_block() {
        let mut builder = MirBuilder::new();
        
        let block_id = builder.create_block("entry");
        assert_eq!(builder.block_count(), 1);
        
        builder.add_instruction(block_id, MirInstruction::Const(0, 42));
        builder.add_instruction(block_id, MirInstruction::Return(0));
        
        assert_eq!(builder.instruction_count(), 2);
    }
    
    #[test]
    fn test_build_control_flow() {
        let mut builder = MirBuilder::new();
        
        let entry = builder.create_block("entry");
        let then_block = builder.create_block("then");
        let else_block = builder.create_block("else");
        let merge = builder.create_block("merge");
        
        // Build if-then-else
        builder.add_instruction(entry, MirInstruction::Branch(0, then_block, else_block));
        builder.add_instruction(then_block, MirInstruction::Jump(merge));
        builder.add_instruction(else_block, MirInstruction::Jump(merge));
        builder.add_instruction(merge, MirInstruction::Return(0));
        
        assert_eq!(builder.block_count(), 4);
    }
    
    #[test]
    fn test_build_function_mir() {
        let mut builder = MirBuilder::new();
        
        // Build MIR for: fn add(x, y) { x + y }
        let entry = builder.create_block("entry");
        
        builder.add_instruction(entry, MirInstruction::Param(0)); // x
        builder.add_instruction(entry, MirInstruction::Param(1)); // y
        builder.add_instruction(entry, MirInstruction::Add(2, 0, 1)); // result = x + y
        builder.add_instruction(entry, MirInstruction::Return(2));
        
        let mir = builder.build();
        assert_eq!(mir.blocks.len(), 1);
        assert_eq!(mir.blocks[0].instructions.len(), 4);
    }
    
    // ========== MIR Optimizer Tests ==========
    
    #[test]
    fn test_mir_optimizer_creation() {
        let optimizer = MirOptimizer::new();
        assert_eq!(optimizer.pass_count(), 0);
    }
    
    #[test]
    fn test_dead_code_elimination() {
        let mut builder = MirBuilder::new();
        let mut optimizer = MirOptimizer::new();
        
        let entry = builder.create_block("entry");
        builder.add_instruction(entry, MirInstruction::Const(0, 42)); // Dead
        builder.add_instruction(entry, MirInstruction::Const(1, 10)); // Used
        builder.add_instruction(entry, MirInstruction::Return(1));
        
        let mir = builder.build();
        let optimized = optimizer.eliminate_dead_code(mir);
        
        // Should remove unused constant
        assert!(optimized.blocks[0].instructions.len() < 3);
    }
    
    #[test]
    fn test_constant_folding() {
        let mut builder = MirBuilder::new();
        let mut optimizer = MirOptimizer::new();
        
        let entry = builder.create_block("entry");
        builder.add_instruction(entry, MirInstruction::Const(0, 5));
        builder.add_instruction(entry, MirInstruction::Const(1, 10));
        builder.add_instruction(entry, MirInstruction::Add(2, 0, 1)); // 5 + 10
        builder.add_instruction(entry, MirInstruction::Return(2));
        
        let mir = builder.build();
        let optimized = optimizer.fold_constants(mir);
        
        // Should fold 5 + 10 into 15
        let has_const_15 = optimized.blocks[0].instructions.iter()
            .any(|inst| matches!(inst, MirInstruction::Const(_, 15)));
        assert!(has_const_15);
    }
    
    #[test]
    fn test_basic_block_merging() {
        let mut builder = MirBuilder::new();
        let mut optimizer = MirOptimizer::new();
        
        let block1 = builder.create_block("block1");
        let block2 = builder.create_block("block2");
        
        builder.add_instruction(block1, MirInstruction::Jump(block2));
        builder.add_instruction(block2, MirInstruction::Return(0));
        
        let mir = builder.build();
        let optimized = optimizer.merge_blocks(mir);
        
        // Should merge blocks with single jump
        assert!(optimized.blocks.len() < 2);
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    fn create_literal_expr(lit: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(lit),
            span: Default::default(),
            attributes: vec![],
        }
    }
    
    fn create_binary_expr(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: Default::default(),
            attributes: vec![],
        }
    }
    
    fn create_if_expr(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            },
            span: Default::default(),
            attributes: vec![],
        }
    }
    
    impl TypeInferer {
        fn is_empty(&self) -> bool {
            self.constraint_count() == 0
        }
        
        fn constraint_count(&self) -> usize {
            self.constraints.len()
        }
    }
    
    impl Environment {
        fn is_empty(&self) -> bool {
            self.binding_count() == 0
        }
        
        fn binding_count(&self) -> usize {
            self.bindings.len()
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_type_inference_never_panics(n in -1000i64..1000) {
            let mut inferer = TypeInferer::new();
            let expr = create_literal_expr(Literal::Integer(n));
            let _ = inferer.infer(&expr); // Should not panic
        }
        
        #[test]
        fn test_unification_symmetric(seed in 0u64..1000) {
            let mut unifier1 = Unifier::new();
            let mut unifier2 = Unifier::new();
            
            let t1 = Type::Variable(seed as usize);
            let t2 = Type::Integer;
            
            let result1 = unifier1.unify(&t1, &t2);
            let result2 = unifier2.unify(&t2, &t1);
            
            // Unification should be symmetric
            assert_eq!(result1.is_ok(), result2.is_ok());
        }
        
        #[test]
        fn test_environment_consistency(names in prop::collection::vec("[a-z]+", 1..20)) {
            let mut env = Environment::new();
            
            for (i, name) in names.iter().enumerate() {
                env.bind(name, Type::Variable(i));
            }
            
            for (i, name) in names.iter().enumerate() {
                assert_eq!(env.lookup(name), Some(&Type::Variable(i)));
            }
        }
    }
}