// TDD Test Suite for InferenceContext::infer_method_call Complexity Reduction
// Current: 41 cyclomatic complexity - NEW HOTSPOT after import_inline fix  
// Target: <20 for all functions
// Strategy: Extract method-category specific handlers (list, string, dataframe)

use ruchy::middleend::infer::InferenceContext;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};

#[cfg(test)]
mod infer_method_call_tdd {
    use super::*;

    fn create_test_context() -> InferenceContext {
        InferenceContext::new()
    }

    fn create_string_literal(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_int_literal(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value)),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_list_expr(elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(elements),
            span: Span::default(),
            attributes: vec![],
        }
    }

    // Test list methods
    #[test]
    fn test_list_length_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1), create_int_literal(2)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "len", &args);
        assert!(result.is_ok());
        // Should return int type for length
    }

    #[test]
    fn test_list_push_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1)]);
        let args = vec![create_int_literal(42)];
        
        let result = context.infer_method_call(&list_expr, "push", &args);
        assert!(result.is_ok());
        // Should return unit type for push
    }

    #[test]
    fn test_list_pop_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "pop", &args);
        assert!(result.is_ok());
        // Should return optional element type
    }

    #[test]
    fn test_list_sorted_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(3), create_int_literal(1)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "sorted", &args);
        assert!(result.is_ok());
        // Should return same list type
    }

    #[test]
    fn test_list_sum_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1), create_int_literal(2)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "sum", &args);
        assert!(result.is_ok());
        // Should return element type
    }

    #[test]
    fn test_list_reversed_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1), create_int_literal(2)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "reversed", &args);
        assert!(result.is_ok());
        // Should return same list type
    }

    #[test]
    fn test_list_unique_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1), create_int_literal(1)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "unique", &args);
        assert!(result.is_ok());
        // Should return same list type
    }

    #[test]
    fn test_list_min_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1), create_int_literal(2)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "min", &args);
        assert!(result.is_ok());
        // Should return optional element type
    }

    #[test]
    fn test_list_max_method() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1), create_int_literal(2)]);
        let args = vec![];
        
        let result = context.infer_method_call(&list_expr, "max", &args);
        assert!(result.is_ok());
        // Should return optional element type
    }

    // Test string methods
    #[test]
    fn test_string_length_method() {
        let mut context = create_test_context();
        let string_expr = create_string_literal("hello");
        let args = vec![];
        
        let result = context.infer_method_call(&string_expr, "len", &args);
        assert!(result.is_ok());
        // Should return int type for length
    }

    #[test]
    fn test_string_chars_method() {
        let mut context = create_test_context();
        let string_expr = create_string_literal("hello");
        let args = vec![];
        
        let result = context.infer_method_call(&string_expr, "chars", &args);
        assert!(result.is_ok());
        // Should return list of strings
    }

    // Test method argument validation
    #[test]
    fn test_method_with_wrong_args_count() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1)]);
        let args = vec![create_int_literal(42)]; // len should take no args
        
        let result = context.infer_method_call(&list_expr, "len", &args);
        assert!(result.is_err());
        // Should error on wrong argument count
    }

    #[test]
    fn test_push_method_wrong_args() {
        let mut context = create_test_context();
        let list_expr = create_list_expr(vec![create_int_literal(1)]);
        let args = vec![]; // push should take exactly one arg
        
        let result = context.infer_method_call(&list_expr, "push", &args);
        assert!(result.is_err());
        // Should error on wrong argument count
    }

    // Test unknown method handling
    #[test]
    fn test_unknown_method() {
        let mut context = create_test_context();
        let string_expr = create_string_literal("hello");
        let args = vec![];
        
        let result = context.infer_method_call(&string_expr, "unknown_method", &args);
        assert!(result.is_ok());
        // Should return a type variable for unknown methods
    }

    // Test dataframe methods (simplified)
    #[test]
    fn test_dataframe_filter_method() {
        let mut context = create_test_context();
        let df_expr = create_string_literal("fake_df"); // Placeholder 
        let args = vec![create_string_literal("condition")];
        
        // This test would need a proper DataFrame expression type
        // For now just ensure it doesn't crash
        let result = context.infer_method_call(&df_expr, "filter", &args);
        assert!(result.is_ok());
    }

    // Tests for refactored helper methods (to be implemented)
    mod refactored_helpers {
        use super::*;

        #[test]
        fn test_list_method_dispatcher() {
            // Test that list methods are properly dispatched
            let mut context = create_test_context();
            let list_expr = create_list_expr(vec![create_int_literal(1)]);
            
            // This would test the extracted infer_list_method once implemented
            // let result = context.infer_list_method(&list_expr, "len", &[]);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_string_method_dispatcher() {
            // Test extracted string method handler
            let mut context = create_test_context();
            let string_expr = create_string_literal("hello");
            
            // This would test the extracted infer_string_method once implemented  
            // let result = context.infer_string_method(&string_expr, "len", &[]);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_dataframe_method_dispatcher() {
            // Test extracted dataframe method handler
            let mut context = create_test_context();
            let df_expr = create_string_literal("fake_df");
            
            // This would test the extracted infer_dataframe_method once implemented
            // let result = context.infer_dataframe_method(&df_expr, "filter", &args);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_generic_method_dispatcher() {
            // Test extracted generic method handler
            let mut context = create_test_context();
            let expr = create_string_literal("test");
            
            // This would test the extracted infer_generic_method once implemented
            // let result = context.infer_generic_method(&expr, "unknown", &[]);
            // assert!(result.is_ok());
        }
    }
}

// Demonstration of how the refactoring would work
// These would be the extracted helper methods to reduce complexity
/*
impl InferenceContext {
    // Main method becomes a dispatcher (complexity ~5)
    fn infer_method_call(&mut self, receiver: &Expr, method: &str, args: &[Expr]) -> Result<MonoType> {
        let receiver_ty = self.infer_expr(receiver)?;
        self.add_method_constraint(&receiver_ty, method, args)?;
        
        // Dispatch based on receiver type category
        match &receiver_ty {
            MonoType::List(_) => self.infer_list_method(&receiver_ty, method, args),
            MonoType::String => self.infer_string_method(&receiver_ty, method, args),
            MonoType::DataFrame(_) | MonoType::Series(_) => self.infer_dataframe_method(&receiver_ty, method, args),
            MonoType::Named(name) if name == "DataFrame" || name == "Series" => {
                self.infer_dataframe_method(&receiver_ty, method, args)
            }
            _ => self.infer_generic_method(&receiver_ty, method, args),
        }
    }

    // Extract list method handling (complexity ~10)
    fn infer_list_method(&mut self, receiver_ty: &MonoType, method: &str, args: &[Expr]) -> Result<MonoType> {
        if let MonoType::List(elem_ty) = receiver_ty {
            match method {
                "len" | "length" => {
                    self.validate_no_args(method, args)?;
                    Ok(MonoType::Int)
                }
                "push" => {
                    self.validate_single_arg(method, args)?;
                    let arg_ty = self.infer_expr(&args[0])?;
                    self.unifier.unify(&arg_ty, elem_ty)?;
                    Ok(MonoType::Unit)
                }
                "pop" => {
                    self.validate_no_args(method, args)?;
                    Ok(MonoType::Optional(elem_ty.clone()))
                }
                "sorted" | "reversed" | "unique" => {
                    self.validate_no_args(method, args)?;
                    Ok(MonoType::List(elem_ty.clone()))
                }
                "sum" => {
                    self.validate_no_args(method, args)?;
                    Ok(*elem_ty.clone())
                }
                "min" | "max" => {
                    self.validate_no_args(method, args)?;
                    Ok(MonoType::Optional(elem_ty.clone()))
                }
                _ => self.infer_generic_method(receiver_ty, method, args),
            }
        } else {
            bail!("Expected list type for list method")
        }
    }

    // Extract string method handling (complexity ~5)
    fn infer_string_method(&mut self, receiver_ty: &MonoType, method: &str, args: &[Expr]) -> Result<MonoType> {
        match method {
            "len" | "length" => {
                self.validate_no_args(method, args)?;
                Ok(MonoType::Int)
            }
            "chars" => {
                self.validate_no_args(method, args)?;
                Ok(MonoType::List(Box::new(MonoType::String)))
            }
            _ => self.infer_generic_method(receiver_ty, method, args),
        }
    }

    // Extract dataframe method handling (complexity ~8)
    fn infer_dataframe_method(&mut self, receiver_ty: &MonoType, method: &str, args: &[Expr]) -> Result<MonoType> {
        match method {
            "filter" | "groupby" | "agg" | "select" => {
                if let MonoType::DataFrame(columns) = receiver_ty {
                    Ok(MonoType::DataFrame(columns.clone()))
                } else {
                    Ok(MonoType::Named("DataFrame".to_string()))
                }
            }
            "mean" | "std" | "sum" | "count" => Ok(MonoType::Float),
            "col" => self.infer_column_selection(receiver_ty, args),
            _ => self.infer_generic_method(receiver_ty, method, args),
        }
    }

    // Extract generic method handling (complexity ~8)
    fn infer_generic_method(&mut self, receiver_ty: &MonoType, method: &str, args: &[Expr]) -> Result<MonoType> {
        if let Some(scheme) = self.env.lookup(method) {
            let method_ty = self.env.instantiate(scheme, &mut self.gen);
            let result_ty = MonoType::Var(self.gen.fresh());
            let expected_func_ty = self.build_method_function_type(receiver_ty, args, result_ty.clone())?;
            self.unifier.unify(&method_ty, &expected_func_ty)?;
            Ok(self.unifier.apply(&result_ty))
        } else {
            Ok(MonoType::Var(self.gen.fresh()))
        }
    }

    // Helper methods for validation and construction (complexity ~3 each)
    fn validate_no_args(&self, method: &str, args: &[Expr]) -> Result<()> {
        if !args.is_empty() {
            bail!("Method {} takes no arguments", method);
        }
        Ok(())
    }

    fn validate_single_arg(&self, method: &str, args: &[Expr]) -> Result<()> {
        if args.len() != 1 {
            bail!("Method {} takes exactly one argument", method);
        }
        Ok(())
    }

    fn add_method_constraint(&mut self, receiver_ty: &MonoType, method: &str, args: &[Expr]) -> Result<()> {
        let arg_types: Result<Vec<_>> = args.iter().map(|arg| self.infer_expr(arg)).collect();
        let arg_types = arg_types?;
        
        self.type_constraints.push(TypeConstraint::MethodCall(
            receiver_ty.clone(),
            method.to_string(),
            arg_types,
        ));
        Ok(())
    }
}
*/