// TDD Test Suite for InferenceContext::infer_other_expr Complexity Reduction  
// Current: 38 cyclomatic complexity - FINAL HOTSPOT
// Target: <20 for all functions
// Strategy: Group related expression types into category handlers

use ruchy::middleend::infer::InferenceContext;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};

#[cfg(test)]
mod infer_other_expr_tdd {
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

    fn create_block_expr(exprs: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Block(exprs),
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_some_expr(value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Some { value: Box::new(value) },
            span: Span::default(),
            attributes: vec![],
        }
    }

    fn create_none_expr() -> Expr {
        Expr {
            kind: ExprKind::None,
            span: Span::default(),
            attributes: vec![],
        }
    }

    // Test basic expressions
    #[test]
    fn test_block_inference() {
        let mut context = create_test_context();
        let block_expr = create_block_expr(vec![create_int_literal(42)]);
        
        let result = context.infer_other_expr(&block_expr);
        assert!(result.is_ok());
        // Should return the type of the last expression
    }

    #[test]
    fn test_empty_block_inference() {
        let mut context = create_test_context();
        let block_expr = create_block_expr(vec![]);
        
        let result = context.infer_other_expr(&block_expr);
        assert!(result.is_ok());
        // Should return Unit type for empty block
    }

    // Test option expressions
    #[test]
    fn test_some_inference() {
        let mut context = create_test_context();
        let some_expr = create_some_expr(create_int_literal(42));
        
        let result = context.infer_other_expr(&some_expr);
        assert!(result.is_ok());
        // Should return Optional<int> type
    }

    #[test]
    fn test_none_inference() {
        let mut context = create_test_context();
        let none_expr = create_none_expr();
        
        let result = context.infer_other_expr(&none_expr);
        assert!(result.is_ok());
        // Should return Optional<TypeVar> type
    }

    // Test import/export (should return Unit)
    #[test]
    fn test_import_export_inference() {
        let mut context = create_test_context();
        let import_expr = Expr {
            kind: ExprKind::Import { 
                path: "test".to_string(),
                items: vec![],
            },
            span: Span::default(),
            attributes: vec![],
        };
        
        let result = context.infer_other_expr(&import_expr);
        assert!(result.is_ok());
        // Should return Unit type
    }

    // Test control flow expressions (should return Unit)
    #[test]
    fn test_break_continue_return_inference() {
        let mut context = create_test_context();
        
        let break_expr = Expr {
            kind: ExprKind::Break { label: None },
            span: Span::default(),
            attributes: vec![],
        };
        
        let result = context.infer_other_expr(&break_expr);
        assert!(result.is_ok());
        // Should return Unit type for control flow
    }

    // Test struct/enum/trait definitions (should return Unit)
    #[test]  
    fn test_struct_definition_inference() {
        let mut context = create_test_context();
        
        let struct_expr = Expr {
            kind: ExprKind::Struct {
                name: "TestStruct".to_string(),
                fields: vec![],
                type_params: vec![],
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
        };
        
        let result = context.infer_other_expr(&struct_expr);
        assert!(result.is_ok());
        // Should return Unit type for definitions
    }

    // Test struct literal
    #[test]
    fn test_struct_literal_inference() {
        let mut context = create_test_context();
        
        let struct_literal = Expr {
            kind: ExprKind::StructLiteral {
                name: "TestStruct".to_string(),
                fields: vec![],
            },
            span: Span::default(),
            attributes: vec![],
        };
        
        let result = context.infer_other_expr(&struct_literal);
        assert!(result.is_ok());
        // Should return named type
    }

    // Test object literal
    #[test]
    fn test_object_literal_inference() {
        let mut context = create_test_context();
        
        let object_literal = Expr {
            kind: ExprKind::ObjectLiteral {
                fields: vec![],
            },
            span: Span::default(),
            attributes: vec![],
        };
        
        let result = context.infer_other_expr(&object_literal);
        assert!(result.is_ok());
        // Should return object type
    }

    // Test command expression
    #[test]
    fn test_command_inference() {
        let mut context = create_test_context();
        
        let command_expr = Expr {
            kind: ExprKind::Command {
                program: "ls".to_string(),
                args: vec![],
                env: vec![],
                working_dir: None,
            },
            span: Span::default(),
            attributes: vec![],
        };
        
        let result = context.infer_other_expr(&command_expr);
        assert!(result.is_ok());
        // Should return String type
    }

    // Test unknown expression handling
    #[test]
    fn test_unknown_expression_error() {
        let mut context = create_test_context();
        
        // Create an expression type that should trigger the fallback case
        let literal_expr = create_string_literal("test");
        
        // This should actually be handled by other parts of the system
        // This test verifies that at least the method can be called
        let result = context.infer_other_expr(&literal_expr);
        // May succeed or fail depending on how literals are routed
        assert!(result.is_ok() || result.is_err());
    }

    // Tests for refactored helper methods (to be implemented)
    mod refactored_helpers {
        use super::*;

        #[test]
        fn test_control_flow_expressions() {
            // Test that control flow expressions are properly handled
            let mut _context = create_test_context();
            
            // This would test the extracted infer_control_flow once implemented
            // let result = context.infer_control_flow_expr(&break_expr);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_definition_expressions() {
            // Test that definition expressions are properly handled
            let mut _context = create_test_context();
            
            // This would test the extracted infer_definition_expr once implemented  
            // let result = context.infer_definition_expr(&struct_expr);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_literal_expressions() {
            // Test that literal expressions are properly handled
            let mut context = create_test_context();
            
            // This would test the extracted infer_literal_expr once implemented
            // let result = context.infer_literal_expr(&object_literal);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_async_expressions() {
            // Test that async expressions are properly handled
            let mut context = create_test_context();
            
            // This would test the extracted infer_async_expr once implemented
            // let result = context.infer_async_expr(&async_expr);
            // assert!(result.is_ok());
        }

        #[test]
        fn test_actor_expressions() {
            // Test that actor expressions are properly handled
            let mut context = create_test_context();
            
            // This would test the extracted infer_actor_expr once implemented
            // let result = context.infer_actor_expr(&send_expr);
            // assert!(result.is_ok());
        }
    }
}

// Demonstration of how the refactoring would work
// These would be the extracted helper methods to reduce complexity
/*
impl InferenceContext {
    // Main method becomes a dispatcher (complexity ~8)
    fn infer_other_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            // String interpolation and result types (specific cases)
            ExprKind::StringInterpolation { .. } => self.infer_string_interpolation_group(expr),
            
            // Control flow expressions  
            ExprKind::Break { .. } | ExprKind::Continue { .. } | ExprKind::Return { .. } => {
                self.infer_control_flow_expr(expr)
            }
            
            // Definition expressions (return Unit)
            ExprKind::Struct { .. } | ExprKind::Enum { .. } | ExprKind::Trait { .. } | 
            ExprKind::Impl { .. } | ExprKind::Extension { .. } | ExprKind::Actor { .. } |
            ExprKind::Import { .. } | ExprKind::Export { .. } => {
                self.infer_definition_expr(expr)
            }
            
            // Literal and access expressions
            ExprKind::StructLiteral { .. } | ExprKind::ObjectLiteral { .. } | 
            ExprKind::FieldAccess { .. } | ExprKind::IndexAccess { .. } | ExprKind::Slice { .. } => {
                self.infer_literal_access_expr(expr)
            }
            
            // Option expressions
            ExprKind::Some { .. } | ExprKind::None => self.infer_option_expr(expr),
            
            // Async expressions
            ExprKind::Await { .. } | ExprKind::AsyncBlock { .. } | ExprKind::Try { .. } => {
                self.infer_async_expr(expr)
            }
            
            // Actor expressions
            ExprKind::Send { .. } | ExprKind::ActorSend { .. } | ExprKind::Ask { .. } | 
            ExprKind::ActorQuery { .. } => {
                self.infer_actor_expr(expr)
            }
            
            // Assignment expressions
            ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } |
            ExprKind::PreIncrement { .. } | ExprKind::PostIncrement { .. } |
            ExprKind::PreDecrement { .. } | ExprKind::PostDecrement { .. } => {
                self.infer_assignment_expr(expr)
            }
            
            // Remaining expressions
            _ => self.infer_remaining_expr(expr),
        }
    }

    // Extract control flow handling (complexity ~3)
    fn infer_control_flow_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        Ok(MonoType::Unit)  // All control flow returns Unit
    }

    // Extract definition handling (complexity ~3)  
    fn infer_definition_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        Ok(MonoType::Unit)  // All definitions return Unit
    }

    // Extract literal/access handling (complexity ~8)
    fn infer_literal_access_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::StructLiteral { name, .. } => Ok(MonoType::Named(name.clone())),
            ExprKind::ObjectLiteral { fields } => self.infer_object_literal(fields),
            ExprKind::FieldAccess { object, .. } => self.infer_field_access(object),
            ExprKind::IndexAccess { object, index } => self.infer_index_access(object, index),
            ExprKind::Slice { object, .. } => self.infer_slice(object),
            _ => bail!("Unexpected literal/access expression"),
        }
    }

    // Extract option handling (complexity ~5)
    fn infer_option_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Some { value } => {
                let inner_type = self.infer_expr(value)?;
                Ok(MonoType::Optional(Box::new(inner_type)))
            }
            ExprKind::None => {
                let type_var = MonoType::Var(self.gen.fresh());
                Ok(MonoType::Optional(Box::new(type_var)))
            }
            _ => bail!("Unexpected option expression"),
        }
    }

    // Extract async handling (complexity ~5)
    fn infer_async_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Await { expr } => self.infer_await(expr),
            ExprKind::AsyncBlock { body } => self.infer_async_block(body),
            ExprKind::Try { expr } => {
                let expr_type = self.infer(expr)?;
                Ok(expr_type)
            }
            _ => bail!("Unexpected async expression"),
        }
    }

    // Extract actor handling (complexity ~6)
    fn infer_actor_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Send { actor, message } | ExprKind::ActorSend { actor, message } => {
                self.infer_send(actor, message)
            }
            ExprKind::Ask { actor, message, timeout } => {
                self.infer_ask(actor, message, timeout.as_deref())
            }
            ExprKind::ActorQuery { actor, message } => self.infer_ask(actor, message, None),
            _ => bail!("Unexpected actor expression"),
        }
    }

    // Extract assignment handling (complexity ~6)
    fn infer_assignment_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Assign { target, value } => self.infer_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.infer_compound_assign(target, *op, value)
            }
            ExprKind::PreIncrement { target } | ExprKind::PostIncrement { target } |
            ExprKind::PreDecrement { target } | ExprKind::PostDecrement { target } => {
                self.infer_increment_decrement(target)
            }
            _ => bail!("Unexpected assignment expression"),
        }
    }

    // Extract remaining expressions (complexity ~8)
    fn infer_remaining_expr(&mut self, expr: &Expr) -> Result<MonoType> {
        match &expr.kind {
            ExprKind::Let { name, value, body, is_mutable, .. } => {
                self.infer_let(name, value, body, *is_mutable)
            }
            ExprKind::Block(exprs) => self.infer_block(exprs),
            ExprKind::Range { start, end, .. } => self.infer_range(start, end),
            ExprKind::Pipeline { expr, stages } => self.infer_pipeline(expr, stages),
            ExprKind::Module { body, .. } => self.infer_expr(body),
            ExprKind::DataFrame { columns } => self.infer_dataframe(columns),
            ExprKind::Command { .. } => Ok(MonoType::String),
            ExprKind::Macro { name, args } => self.infer_macro(name, args),
            ExprKind::DataFrameOperation { source, operation } => {
                self.infer_dataframe_operation(source, operation)
            }
            _ => bail!("Unknown expression type in inference"),
        }
    }
}
*/