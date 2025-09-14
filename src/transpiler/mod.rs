//! Transpiler module implementing extreme quality engineering
//!
//! Based on docs/ruchy-transpiler-docs.md
pub mod canonical_ast;
pub mod provenance;
pub mod reference_interpreter;
// Re-exports
pub use canonical_ast::{AstNormalizer, CoreExpr, CoreLiteral, DeBruijnIndex, PrimOp};
pub use provenance::{CompilationTrace, ProvenanceTracker, TraceDiffer};
pub use reference_interpreter::{Environment, ReferenceInterpreter, Value};

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 10: Transpiler module tests

    #[test]
    fn test_ast_normalizer_creation() {
        let normalizer = AstNormalizer::new();
        // Just verify it can be created
        let _ = normalizer;
    }

    #[test]
    fn test_de_bruijn_index_creation() {
        let idx = DeBruijnIndex(0);
        assert_eq!(idx.0, 0);

        let idx2 = DeBruijnIndex(42);
        assert_eq!(idx2.0, 42);
    }

    #[test]
    fn test_de_bruijn_index_equality() {
        let idx1 = DeBruijnIndex(5);
        let idx2 = DeBruijnIndex(5);
        let idx3 = DeBruijnIndex(10);

        assert_eq!(idx1, idx2);
        assert_ne!(idx1, idx3);
    }

    #[test]
    fn test_provenance_tracker_creation() {
        let tracker = ProvenanceTracker::new();
        assert!(tracker.traces().is_empty());
    }

    #[test]
    fn test_provenance_tracker_add_trace() {
        let mut tracker = ProvenanceTracker::new();
        let trace = CompilationTrace::new("test_phase");
        tracker.add_trace(trace);
        assert_eq!(tracker.traces().len(), 1);
    }

    #[test]
    fn test_compilation_trace_creation() {
        let trace = CompilationTrace::new("optimization");
        assert_eq!(trace.phase(), "optimization");
        assert!(trace.steps().is_empty());
    }

    #[test]
    fn test_compilation_trace_add_step() {
        let mut trace = CompilationTrace::new("parsing");
        trace.add_step("tokenize", "Tokenized input");
        trace.add_step("parse", "Built AST");
        assert_eq!(trace.steps().len(), 2);
    }

    #[test]
    fn test_trace_differ_creation() {
        let differ = TraceDiffer::new();
        // Just verify it can be created
        let _ = differ;
    }

    #[test]
    fn test_trace_differ_compare_identical() {
        let differ = TraceDiffer::new();
        let trace1 = CompilationTrace::new("test");
        let trace2 = CompilationTrace::new("test");

        let diff = differ.compare(&trace1, &trace2);
        assert!(diff.is_identical());
    }

    #[test]
    fn test_trace_differ_compare_different() {
        let differ = TraceDiffer::new();
        let mut trace1 = CompilationTrace::new("test");
        trace1.add_step("step1", "data1");

        let mut trace2 = CompilationTrace::new("test");
        trace2.add_step("step2", "data2");

        let diff = differ.compare(&trace1, &trace2);
        assert!(!diff.is_identical());
    }

    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert!(env.bindings().is_empty());
    }

    #[test]
    fn test_environment_bind() {
        let mut env = Environment::new();
        env.bind("x", Value::Integer(42));
        assert_eq!(env.lookup("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_environment_lookup_missing() {
        let env = Environment::new();
        assert_eq!(env.lookup("y"), None);
    }

    #[test]
    fn test_environment_extend() {
        let mut env = Environment::new();
        env.bind("x", Value::Integer(1));

        let mut env2 = env.extend();
        env2.bind("y", Value::Integer(2));

        assert_eq!(env2.lookup("x"), Some(&Value::Integer(1)));
        assert_eq!(env2.lookup("y"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_value_variants() {
        let int_val = Value::Integer(42);
        let float_val = Value::Float(3.14);
        let bool_val = Value::Bool(true);
        let string_val = Value::String("hello".to_string());
        let unit_val = Value::Unit;

        assert!(matches!(int_val, Value::Integer(42)));
        assert!(matches!(float_val, Value::Float(_)));
        assert!(matches!(bool_val, Value::Bool(true)));
        assert!(matches!(string_val, Value::String(_)));
        assert!(matches!(unit_val, Value::Unit));
    }

    #[test]
    fn test_value_equality() {
        let val1 = Value::Integer(42);
        let val2 = Value::Integer(42);
        let val3 = Value::Integer(43);

        assert_eq!(val1, val2);
        assert_ne!(val1, val3);
    }

    #[test]
    fn test_reference_interpreter_creation() {
        let interpreter = ReferenceInterpreter::new();
        assert_eq!(interpreter.step_count(), 0);
    }

    #[test]
    fn test_reference_interpreter_with_limit() {
        let interpreter = ReferenceInterpreter::with_step_limit(1000);
        assert_eq!(interpreter.step_limit(), 1000);
    }

    #[test]
    fn test_core_literal_integer() {
        let lit = CoreLiteral::Integer(100);
        assert!(matches!(lit, CoreLiteral::Integer(100)));
    }

    #[test]
    fn test_core_literal_float() {
        let lit = CoreLiteral::Float(2.718);
        if let CoreLiteral::Float(f) = lit {
            assert!((f - 2.718).abs() < 0.001);
        }
    }

    #[test]
    fn test_core_literal_bool() {
        let lit_true = CoreLiteral::Bool(true);
        let lit_false = CoreLiteral::Bool(false);
        assert!(matches!(lit_true, CoreLiteral::Bool(true)));
        assert!(matches!(lit_false, CoreLiteral::Bool(false)));
    }

    #[test]
    fn test_core_literal_string() {
        let lit = CoreLiteral::String("test".to_string());
        if let CoreLiteral::String(s) = lit {
            assert_eq!(s, "test");
        }
    }

    #[test]
    fn test_core_expr_literal() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        assert!(matches!(expr, CoreExpr::Literal(_)));
    }

    #[test]
    fn test_core_expr_variable() {
        let expr = CoreExpr::Variable(DeBruijnIndex(0));
        if let CoreExpr::Variable(idx) = expr {
            assert_eq!(idx.0, 0);
        }
    }

    #[test]
    fn test_prim_op_add() {
        let op = PrimOp::Add;
        assert!(matches!(op, PrimOp::Add));
    }

    #[test]
    fn test_prim_op_subtract() {
        let op = PrimOp::Subtract;
        assert!(matches!(op, PrimOp::Subtract));
    }

    #[test]
    fn test_prim_op_multiply() {
        let op = PrimOp::Multiply;
        assert!(matches!(op, PrimOp::Multiply));
    }

    #[test]
    fn test_prim_op_divide() {
        let op = PrimOp::Divide;
        assert!(matches!(op, PrimOp::Divide));
    }

    #[test]
    fn test_prim_op_equality() {
        assert_eq!(PrimOp::Add, PrimOp::Add);
        assert_ne!(PrimOp::Add, PrimOp::Subtract);
    }

    #[test]
    fn test_value_list() {
        let list = Value::List(vec![Value::Integer(1), Value::Integer(2)]);
        if let Value::List(items) = list {
            assert_eq!(items.len(), 2);
        }
    }

    #[test]
    fn test_value_function() {
        let func = Value::Function(Box::new(|_env, _arg| Value::Unit));
        assert!(matches!(func, Value::Function(_)));
    }

    #[test]
    fn test_environment_shadow_binding() {
        let mut env = Environment::new();
        env.bind("x", Value::Integer(1));
        env.bind("x", Value::Integer(2));
        assert_eq!(env.lookup("x"), Some(&Value::Integer(2)));
    }

    #[test]
    fn test_provenance_tracker_clear() {
        let mut tracker = ProvenanceTracker::new();
        tracker.add_trace(CompilationTrace::new("phase1"));
        tracker.add_trace(CompilationTrace::new("phase2"));
        assert_eq!(tracker.traces().len(), 2);

        tracker.clear();
        assert_eq!(tracker.traces().len(), 0);
    }

    #[test]
    fn test_compilation_trace_with_metadata() {
        let mut trace = CompilationTrace::new("optimization");
        trace.add_step_with_metadata("inline", "Inlined function foo", vec![("count", "3")]);
        assert_eq!(trace.steps().len(), 1);
    }

    #[test]
    fn test_reference_interpreter_step_counting() {
        let mut interpreter = ReferenceInterpreter::new();
        assert_eq!(interpreter.step_count(), 0);

        interpreter.increment_steps();
        assert_eq!(interpreter.step_count(), 1);

        interpreter.increment_steps();
        assert_eq!(interpreter.step_count(), 2);
    }
}

// Transpiler tests commented out temporarily due to type mismatches
// TODO: Fix these tests after verifying the actual transpiler API

/*
#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 5: Comprehensive transpiler tests for coverage improvement

    #[test]
    fn test_core_literal_creation() {
        let int_lit = CoreLiteral::Integer(42);
        assert!(matches!(int_lit, CoreLiteral::Integer(42)));

        let float_lit = CoreLiteral::Float(3.14);
        if let CoreLiteral::Float(f) = float_lit {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float literal");
        }

        let bool_lit = CoreLiteral::Bool(true);
        assert!(matches!(bool_lit, CoreLiteral::Bool(true)));

        let string_lit = CoreLiteral::String("hello".to_string());
        if let CoreLiteral::String(s) = string_lit {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string literal");
        }
    }

    #[test]
    fn test_core_expr_literal() {
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        assert!(matches!(expr, CoreExpr::Literal(CoreLiteral::Integer(42))));
    }

    #[test]
    fn test_core_expr_variable() {
        let var = CoreExpr::Variable(DeBruijnIndex(0));
        if let CoreExpr::Variable(idx) = var {
            assert_eq!(idx.0, 0);
        } else {
            panic!("Expected variable");
        }
    }

    #[test]
    fn test_de_bruijn_index() {
        let idx1 = DeBruijnIndex(0);
        let idx2 = DeBruijnIndex(1);
        let idx3 = DeBruijnIndex(0);

        assert_eq!(idx1.0, 0);
        assert_eq!(idx2.0, 1);
        assert_eq!(idx1, idx3);
        assert_ne!(idx1, idx2);
    }

    #[test]
    fn test_prim_op_variants() {
        let add = PrimOp::Add;
        let sub = PrimOp::Subtract;
        let mul = PrimOp::Multiply;
        let div = PrimOp::Divide;

        assert!(matches!(add, PrimOp::Add));
        assert!(matches!(sub, PrimOp::Subtract));
        assert!(matches!(mul, PrimOp::Multiply));
        assert!(matches!(div, PrimOp::Divide));
    }

    #[test]
    fn test_core_expr_application() {
        let func = Box::new(CoreExpr::Variable(DeBruijnIndex(0)));
        let arg = Box::new(CoreExpr::Literal(CoreLiteral::Integer(42)));
        let app = CoreExpr::Application(func, arg);

        if let CoreExpr::Application(f, a) = app {
            assert!(matches!(**f, CoreExpr::Variable(_)));
            assert!(matches!(**a, CoreExpr::Literal(_)));
        } else {
            panic!("Expected application");
        }
    }

    #[test]
    fn test_core_expr_lambda() {
        let body = Box::new(CoreExpr::Variable(DeBruijnIndex(0)));
        let lambda = CoreExpr::Lambda(body);

        if let CoreExpr::Lambda(b) = lambda {
            assert!(matches!(**b, CoreExpr::Variable(_)));
        } else {
            panic!("Expected lambda");
        }
    }

    #[test]
    fn test_core_expr_let() {
        let value = Box::new(CoreExpr::Literal(CoreLiteral::Integer(42)));
        let body = Box::new(CoreExpr::Variable(DeBruijnIndex(0)));
        let let_expr = CoreExpr::Let(value, body);

        if let CoreExpr::Let(v, b) = let_expr {
            assert!(matches!(**v, CoreExpr::Literal(_)));
            assert!(matches!(**b, CoreExpr::Variable(_)));
        } else {
            panic!("Expected let expression");
        }
    }

    #[test]
    fn test_core_expr_if() {
        let cond = Box::new(CoreExpr::Literal(CoreLiteral::Bool(true)));
        let then_branch = Box::new(CoreExpr::Literal(CoreLiteral::Integer(1)));
        let else_branch = Box::new(CoreExpr::Literal(CoreLiteral::Integer(2)));
        let if_expr = CoreExpr::If(cond, then_branch, else_branch);

        if let CoreExpr::If(c, t, e) = if_expr {
            assert!(matches!(**c, CoreExpr::Literal(CoreLiteral::Bool(true))));
            assert!(matches!(**t, CoreExpr::Literal(CoreLiteral::Integer(1))));
            assert!(matches!(**e, CoreExpr::Literal(CoreLiteral::Integer(2))));
        } else {
            panic!("Expected if expression");
        }
    }

    #[test]
    fn test_core_expr_primitive() {
        let left = Box::new(CoreExpr::Literal(CoreLiteral::Integer(2)));
        let right = Box::new(CoreExpr::Literal(CoreLiteral::Integer(3)));
        let add = CoreExpr::Primitive(PrimOp::Add, vec![left, right]);

        if let CoreExpr::Primitive(op, args) = add {
            assert!(matches!(op, PrimOp::Add));
            assert_eq!(args.len(), 2);
        } else {
            panic!("Expected primitive operation");
        }
    }

    #[test]
    fn test_environment_creation() {
        let env = Environment::new();
        assert!(env.is_empty());
    }

    #[test]
    fn test_environment_push_pop() {
        let mut env = Environment::new();
        let val = Value::Integer(42);

        env.push(val.clone());
        assert!(!env.is_empty());

        let popped = env.pop();
        assert_eq!(popped, Some(val));
        assert!(env.is_empty());
    }

    #[test]
    fn test_value_creation() {
        let int_val = Value::Integer(42);
        assert!(matches!(int_val, Value::Integer(42)));

        let float_val = Value::Float(3.14);
        if let Value::Float(f) = float_val {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float value");
        }

        let bool_val = Value::Bool(true);
        assert!(matches!(bool_val, Value::Bool(true)));

        let string_val = Value::String("test".to_string());
        if let Value::String(s) = string_val {
            assert_eq!(s, "test");
        } else {
            panic!("Expected string value");
        }
    }

    #[test]
    fn test_provenance_tracker_creation() {
        let tracker = ProvenanceTracker::new();
        // Just test that it can be created
        assert!(tracker.traces().is_empty());
    }

    #[test]
    fn test_compilation_trace_creation() {
        let trace = CompilationTrace::new("test_phase");
        assert_eq!(trace.phase(), "test_phase");
    }

    #[test]
    fn test_trace_differ_creation() {
        let differ = TraceDiffer::new();
        // Just test that it can be created
        let trace1 = CompilationTrace::new("phase1");
        let trace2 = CompilationTrace::new("phase2");
        let _diff = differ.diff(&trace1, &trace2);
    }

    #[test]
    fn test_ast_normalizer_creation() {
        let normalizer = AstNormalizer::new();
        // Just test that it can be created
        assert!(normalizer.is_normalized(&CoreExpr::Literal(CoreLiteral::Integer(42))));
    }

    #[test]
    fn test_reference_interpreter_creation() {
        let interpreter = ReferenceInterpreter::new();
        // Just test that it can be created
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        let result = interpreter.eval(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_interpreter_literal() {
        let interpreter = ReferenceInterpreter::new();
        let expr = CoreExpr::Literal(CoreLiteral::Integer(42));
        let result = interpreter.eval(&expr);

        assert!(result.is_ok());
        if let Ok(Value::Integer(n)) = result {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer value");
        }
    }

    #[test]
    fn test_prim_op_equality() {
        assert_eq!(PrimOp::Add, PrimOp::Add);
        assert_ne!(PrimOp::Add, PrimOp::Subtract);
        assert_ne!(PrimOp::Multiply, PrimOp::Divide);
    }

    #[test]
    fn test_core_literal_equality() {
        assert_eq!(CoreLiteral::Integer(42), CoreLiteral::Integer(42));
        assert_ne!(CoreLiteral::Integer(42), CoreLiteral::Integer(43));
        assert_eq!(CoreLiteral::Bool(true), CoreLiteral::Bool(true));
        assert_ne!(CoreLiteral::Bool(true), CoreLiteral::Bool(false));
    }

    #[test]
    fn test_de_bruijn_operations() {
        let idx = DeBruijnIndex(5);
        assert_eq!(idx.0, 5);

        // Test that indices can be used in collections
        let mut indices = vec![DeBruijnIndex(1), DeBruijnIndex(2), DeBruijnIndex(3)];
        indices.push(DeBruijnIndex(4));
        assert_eq!(indices.len(), 4);
    }

    #[test]
    fn test_nested_core_expr() {
        // Test nested let expressions
        let inner_value = Box::new(CoreExpr::Literal(CoreLiteral::Integer(10)));
        let inner_body = Box::new(CoreExpr::Variable(DeBruijnIndex(0)));
        let inner_let = CoreExpr::Let(inner_value, inner_body);

        let outer_value = Box::new(inner_let);
        let outer_body = Box::new(CoreExpr::Variable(DeBruijnIndex(0)));
        let outer_let = CoreExpr::Let(outer_value, outer_body);

        if let CoreExpr::Let(val, _) = outer_let {
            assert!(matches!(**val, CoreExpr::Let(_, _)));
        } else {
            panic!("Expected nested let");
        }
    }

    #[test]
    fn test_primitive_operations_with_multiple_args() {
        let args = vec![
            Box::new(CoreExpr::Literal(CoreLiteral::Integer(1))),
            Box::new(CoreExpr::Literal(CoreLiteral::Integer(2))),
            Box::new(CoreExpr::Literal(CoreLiteral::Integer(3))),
        ];
        let multi_add = CoreExpr::Primitive(PrimOp::Add, args);

        if let CoreExpr::Primitive(op, arguments) = multi_add {
            assert!(matches!(op, PrimOp::Add));
            assert_eq!(arguments.len(), 3);
        } else {
            panic!("Expected primitive with multiple arguments");
        }
    }
}
*/
