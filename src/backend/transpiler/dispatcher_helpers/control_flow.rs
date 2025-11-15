//! Control flow expression transpilation helpers

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::{bail, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    fn transpile_operator_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Binary { left, op, right } => self.transpile_binary(left, *op, right),
            ExprKind::Unary { op, operand } => self.transpile_unary(*op, operand),
            ExprKind::Assign { target, value } => self.transpile_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.transpile_compound_assign(target, *op, value)
            }
            ExprKind::PreIncrement { target } => self.transpile_pre_increment(target),
            ExprKind::PostIncrement { target } => self.transpile_post_increment(target),
            ExprKind::PreDecrement { target } => self.transpile_pre_decrement(target),
            ExprKind::PostDecrement { target } => self.transpile_post_decrement(target),
            ExprKind::Await { expr } => self.transpile_await(expr),
            ExprKind::Spawn { actor } => self.transpile_spawn(actor),
            ExprKind::AsyncBlock { body } => self.transpile_async_block(body),
            ExprKind::AsyncLambda { params, body } => self.transpile_async_lambda(params, body),
            _ => unreachable!(),
        }
    }
    fn transpile_control_flow_only_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.transpile_if(condition, then_branch, else_branch.as_deref()),
            ExprKind::Match { expr, arms } => self.transpile_match(expr, arms),
            ExprKind::For {
                var,
                pattern,
                iter,
                body,
                ..
            } => self.transpile_for(var, pattern.as_ref(), iter, body),
            ExprKind::While {
                condition, body, ..
            } => self.transpile_while(condition, body),
            ExprKind::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
            } => self.transpile_if_let(pattern, expr, then_branch, else_branch.as_deref()),
            ExprKind::WhileLet {
                pattern,
                expr,
                body,
                ..
            } => self.transpile_while_let(pattern, expr, body),
            ExprKind::Loop { body, .. } => self.transpile_loop(body),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => self.transpile_try_catch(try_block, catch_clauses, finally_block.as_deref()),
            _ => unreachable!(),
        }
    }
    /// Transpile function-related expressions
    pub(super) fn transpile_function_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Function {
                name,
                type_params,
                params,
                body,
                is_async,
                return_type,
                is_pub,
            } => self.transpile_function(
                name,
                type_params,
                params,
                body,
                *is_async,
                return_type.as_ref(),
                *is_pub,
                &expr.attributes,
            ),
            ExprKind::Lambda { params, body } => self.transpile_lambda(params, body),
            ExprKind::Call { func, args } => self.transpile_call(func, args),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.transpile_method_call(receiver, method, args),
            ExprKind::Macro { name, args } => self.transpile_macro(name, args),
            _ => unreachable!("Non-function expression in transpile_function_expr"),
        }
    }
    /// Transpile macro expressions with clean dispatch pattern
    ///
    /// This function uses specialized handlers for different macro categories:
    /// - Print macros: `println!`, `print!`, `panic!` (string formatting)
    /// - Collection macros: `vec!` (simple element transpilation)
    /// - Assertion macros: `assert!`, `assert_eq!`, `assert_ne!` (validation + transpilation)
    ///
    /// # Example Usage
    /// This method dispatches to specific macro handlers based on the macro name.
    /// For example, `println` calls `transpile_println_macro`, `vec` calls `transpile_vec_macro`, etc.
    pub(super) fn transpile_macro(&self, name: &str, args: &[Expr]) -> Result<TokenStream> {
        match name {
            // Print macros (string formatting)
            "println" => self.transpile_println_macro(args),
            "print" => self.transpile_print_macro(args),
            "panic" => self.transpile_panic_macro(args),
            // Collection macros (simple transpilation)
            "vec" => self.transpile_vec_macro(args),
            // Assertion macros (validation + transpilation)
            "assert" => self.transpile_assert_macro(args),
            "assert_eq" => self.transpile_assert_eq_macro(args),
            "assert_ne" => self.transpile_assert_ne_macro(args),
            // External macros (pass through)
            "json" | "sql" | "format" | "dbg" | "include_str" | "include_bytes" | "todo"
            | "unimplemented" | "unreachable" | "compile_error" | "concat" | "env"
            | "option_env" | "cfg" | "column" | "file" | "line" | "module_path" | "stringify"
            | "write" | "writeln" | "eprintln" | "eprint" => {
                self.transpile_passthrough_macro(name, args)
            }
            _ => bail!("Unknown macro: {}", name),
        }
    }
    /// Transpile structure-related expressions
    pub(super) fn transpile_struct_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::Struct {
                name,
                type_params,
                fields,
                derives,
                is_pub,
            } => self.transpile_struct(name, type_params, fields, derives, *is_pub),
            ExprKind::TupleStruct {
                name,
                type_params,
                fields,
                derives,
                is_pub,
            } => self.transpile_tuple_struct(name, type_params, fields, derives, *is_pub),
            ExprKind::Class {
                name,
                type_params,
                superclass: _, // Inheritance not yet transpiled
                traits,
                fields,
                constructors,
                methods,
                constants,
                properties: _, // Properties not yet transpiled
                derives,
                is_pub,
                is_sealed: _,   // Sealed classes not yet transpiled
                is_abstract: _, // Abstract classes not yet transpiled
                decorators: _,  // Decorators not yet transpiled
            } => self.transpile_class(
                name,
                type_params,
                traits,
                fields,
                constructors,
                methods,
                constants,
                derives,
                *is_pub,
            ),
            ExprKind::StructLiteral { name, fields, base } => {
                self.transpile_struct_literal(name, fields, base.as_deref())
            }
            ExprKind::ObjectLiteral { fields } => self.transpile_object_literal(fields),
            ExprKind::FieldAccess { object, field } => self.transpile_field_access(object, field),
            ExprKind::IndexAccess { object, index } => self.transpile_index_access(object, index),
            ExprKind::Slice { object, start, end } => {
                self.transpile_slice(object, start.as_deref(), end.as_deref())
            }
            _ => unreachable!("Non-struct expression in transpile_struct_expr"),
        }
    }
    /// Transpile data and error handling expressions (split for complexity)
    pub(super) fn transpile_data_error_expr(&self, expr: &Expr) -> Result<TokenStream> {
        match &expr.kind {
            ExprKind::DataFrame { .. }
            | ExprKind::DataFrameOperation { .. }
            | ExprKind::List(_)
            | ExprKind::Set(_)
            | ExprKind::ArrayInit { .. }
            | ExprKind::Tuple(_)
            | ExprKind::ListComprehension { .. }
            | ExprKind::SetComprehension { .. }
            | ExprKind::DictComprehension { .. }
            | ExprKind::Range { .. } => self.transpile_data_only_expr(expr),
            ExprKind::Throw { .. }
            | ExprKind::Ok { .. }
            | ExprKind::Err { .. }
            | ExprKind::Some { .. }
            | ExprKind::None
            | ExprKind::Try { .. } => self.transpile_error_only_expr(expr),
            _ => unreachable!("Non-data/error expression in transpile_data_error_expr"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, MatchArm, Pattern, Span, UnaryOp};

    // Helper: Create test transpiler instance
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper: Create integer literal expression
    fn int_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create identifier expression
    fn ident_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper: Create block expression
    fn block_expr(exprs: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Block(exprs),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: transpile_control_flow_only_expr - if expression
    #[test]
    fn test_transpile_control_flow_only_expr_if() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(ident_expr("x")),
                then_branch: Box::new(int_expr(1)),
                else_branch: Some(Box::new(int_expr(2))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 2: transpile_control_flow_only_expr - match expression
    #[test]
    fn test_transpile_control_flow_only_expr_match() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Match {
                expr: Box::new(int_expr(1)),
                arms: vec![MatchArm {
                    pattern: Pattern::Literal(Literal::Integer(1, None)),
                    guard: None,
                    body: int_expr(10),
                    span: Span::default(),
                }],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 3: transpile_control_flow_only_expr - for loop
    #[test]
    fn test_transpile_control_flow_only_expr_for() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::For {
                var: "i".to_string(),
                pattern: None,
                iter: Box::new(ident_expr("items")),
                body: Box::new(block_expr(vec![ident_expr("i")])),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 4: transpile_control_flow_only_expr - while loop
    #[test]
    fn test_transpile_control_flow_only_expr_while() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::While {
                condition: Box::new(ident_expr("running")),
                body: Box::new(block_expr(vec![ident_expr("step")])),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 5: transpile_control_flow_only_expr - loop
    #[test]
    fn test_transpile_control_flow_only_expr_loop() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Loop {
                body: Box::new(block_expr(vec![ident_expr("work")])),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 6: transpile_function_expr - function definition
    #[test]
    fn test_transpile_function_expr_function() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Function {
                name: "add".to_string(),
                type_params: vec![],
                params: vec![("a".to_string(), None), ("b".to_string(), None)],
                body: Box::new(block_expr(vec![int_expr(0)])),
                is_async: false,
                return_type: None,
                is_pub: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_function_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 7: transpile_function_expr - lambda
    #[test]
    fn test_transpile_function_expr_lambda() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Lambda {
                params: vec!["x".to_string()],
                body: Box::new(ident_expr("x")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_function_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 8: transpile_function_expr - function call
    #[test]
    fn test_transpile_function_expr_call() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Call {
                func: Box::new(ident_expr("print")),
                args: vec![int_expr(42)],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_function_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 9: transpile_function_expr - method call
    #[test]
    fn test_transpile_function_expr_method_call() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::MethodCall {
                receiver: Box::new(ident_expr("obj")),
                method: "to_string".to_string(),
                args: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_function_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 10: transpile_macro - println
    #[test]
    fn test_transpile_macro_println() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("println", &[int_expr(42)]);
        assert!(result.is_ok());
    }

    // Test 11: transpile_macro - vec
    #[test]
    fn test_transpile_macro_vec() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("vec", &[int_expr(1), int_expr(2)]);
        assert!(result.is_ok());
    }

    // Test 12: transpile_macro - assert
    #[test]
    fn test_transpile_macro_assert() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("assert", &[ident_expr("true")]);
        assert!(result.is_ok());
    }

    // Test 13: transpile_macro - passthrough (json)
    #[test]
    fn test_transpile_macro_passthrough() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("json", &[]);
        assert!(result.is_ok());
    }

    // Test 14: transpile_macro - unknown macro (error path)
    #[test]
    fn test_transpile_macro_unknown() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("unknown_macro", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown macro"));
    }

    // Test 15: transpile_struct_expr - struct literal
    #[test]
    fn test_transpile_struct_expr_struct_literal() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::StructLiteral {
                name: "Point".to_string(),
                fields: vec![("x".to_string(), int_expr(1)), ("y".to_string(), int_expr(2))],
                base: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 16: transpile_struct_expr - field access
    #[test]
    fn test_transpile_struct_expr_field_access() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::FieldAccess {
                object: Box::new(ident_expr("point")),
                field: "x".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 17: transpile_struct_expr - index access
    #[test]
    fn test_transpile_struct_expr_index_access() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::IndexAccess {
                object: Box::new(ident_expr("arr")),
                index: Box::new(int_expr(0)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 18: transpile_data_error_expr - list
    #[test]
    fn test_transpile_data_error_expr_list() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::List(vec![int_expr(1), int_expr(2)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_data_error_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 19: transpile_data_error_expr - tuple
    #[test]
    fn test_transpile_data_error_expr_tuple() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Tuple(vec![int_expr(1), int_expr(2)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_data_error_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 20: transpile_data_error_expr - None
    #[test]
    fn test_transpile_data_error_expr_none() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::None,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_data_error_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 21: transpile_operator_only_expr - Binary (addition)
    #[test]
    fn test_transpile_operator_only_expr_binary() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(int_expr(1)),
                op: BinaryOp::Add,
                right: Box::new(int_expr(2)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 22: transpile_operator_only_expr - Unary (negation)
    #[test]
    fn test_transpile_operator_only_expr_unary() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Minus,
                operand: Box::new(int_expr(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 23: transpile_operator_only_expr - Assign
    #[test]
    fn test_transpile_operator_only_expr_assign() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Assign {
                target: Box::new(ident_expr("x")),
                value: Box::new(int_expr(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 24: transpile_operator_only_expr - CompoundAssign
    #[test]
    fn test_transpile_operator_only_expr_compound_assign() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::CompoundAssign {
                target: Box::new(ident_expr("count")),
                op: BinaryOp::Add,
                value: Box::new(int_expr(1)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 25: transpile_operator_only_expr - PreIncrement
    #[test]
    fn test_transpile_operator_only_expr_pre_increment() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::PreIncrement {
                target: Box::new(ident_expr("i")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 26: transpile_operator_only_expr - PostIncrement
    #[test]
    fn test_transpile_operator_only_expr_post_increment() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::PostIncrement {
                target: Box::new(ident_expr("j")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 27: transpile_control_flow_only_expr - IfLet
    #[test]
    fn test_transpile_control_flow_only_expr_if_let() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::IfLet {
                pattern: Pattern::Identifier("x".to_string()),
                expr: Box::new(ident_expr("opt")),
                then_branch: Box::new(ident_expr("x")),
                else_branch: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 28: transpile_control_flow_only_expr - WhileLet
    #[test]
    fn test_transpile_control_flow_only_expr_while_let() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::WhileLet {
                pattern: Pattern::Identifier("item".to_string()),
                expr: Box::new(ident_expr("iter")),
                body: Box::new(block_expr(vec![ident_expr("item")])),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 29: transpile_control_flow_only_expr - TryCatch
    #[test]
    fn test_transpile_control_flow_only_expr_try_catch() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::TryCatch {
                try_block: Box::new(block_expr(vec![ident_expr("work")])),
                catch_clauses: vec![],
                finally_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_control_flow_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 30: transpile_macro - print
    #[test]
    fn test_transpile_macro_print() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("print", &[int_expr(42)]);
        assert!(result.is_ok());
    }

    // Test 31: transpile_macro - panic
    #[test]
    fn test_transpile_macro_panic() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("panic", &[]);
        assert!(result.is_ok());
    }

    // Test 32: transpile_macro - assert_eq
    #[test]
    fn test_transpile_macro_assert_eq() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("assert_eq", &[int_expr(1), int_expr(1)]);
        assert!(result.is_ok());
    }

    // Test 33: transpile_struct_expr - ObjectLiteral
    #[test]
    fn test_transpile_struct_expr_object_literal() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ObjectLiteral {
                fields: vec![("name".to_string(), ident_expr("value"))],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 34: transpile_struct_expr - Slice
    #[test]
    fn test_transpile_struct_expr_slice() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Slice {
                object: Box::new(ident_expr("arr")),
                start: Some(Box::new(int_expr(0))),
                end: Some(Box::new(int_expr(5))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 35: transpile_data_error_expr - Range
    #[test]
    fn test_transpile_data_error_expr_range() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Range {
                start: Some(Box::new(int_expr(1))),
                end: Some(Box::new(int_expr(10))),
                inclusive: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_data_error_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 36: transpile_operator_only_expr - PreDecrement
    #[test]
    fn test_transpile_operator_only_expr_pre_decrement() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::PreDecrement {
                target: Box::new(ident_expr("counter")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 37: transpile_operator_only_expr - PostDecrement
    #[test]
    fn test_transpile_operator_only_expr_post_decrement() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::PostDecrement {
                target: Box::new(ident_expr("value")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 38: transpile_operator_only_expr - Await
    #[test]
    fn test_transpile_operator_only_expr_await() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Await {
                expr: Box::new(ident_expr("future")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 39: transpile_operator_only_expr - Spawn
    #[test]
    fn test_transpile_operator_only_expr_spawn() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Spawn {
                actor: Box::new(ident_expr("worker")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 40: transpile_operator_only_expr - AsyncBlock
    #[test]
    fn test_transpile_operator_only_expr_async_block() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::AsyncBlock {
                body: Box::new(block_expr(vec![ident_expr("task")])),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 41: transpile_operator_only_expr - AsyncLambda
    #[test]
    fn test_transpile_operator_only_expr_async_lambda() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::AsyncLambda {
                params: vec!["x".to_string()],
                body: Box::new(ident_expr("x")),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_operator_only_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 42: transpile_macro - assert_ne
    #[test]
    fn test_transpile_macro_assert_ne() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("assert_ne", &[int_expr(1), int_expr(2)]);
        assert!(result.is_ok());
    }

    // Test 43: transpile_macro - format (passthrough)
    #[test]
    fn test_transpile_macro_format() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("format", &[]);
        assert!(result.is_ok());
    }

    // Test 44: transpile_macro - dbg (passthrough)
    #[test]
    fn test_transpile_macro_dbg() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("dbg", &[ident_expr("value")]);
        assert!(result.is_ok());
    }

    // Test 45: transpile_macro - todo (passthrough)
    #[test]
    fn test_transpile_macro_todo() {
        let transpiler = test_transpiler();
        let result = transpiler.transpile_macro("todo", &[]);
        assert!(result.is_ok());
    }

    // Test 46: transpile_struct_expr - Struct definition
    #[test]
    fn test_transpile_struct_expr_struct() {
        use crate::frontend::ast::StructField;
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Struct {
                name: "Point".to_string(),
                type_params: vec![],
                fields: vec![
                    StructField {
                        name: "x".to_string(),
                        field_type: "i32".to_string(),
                        is_pub: true,
                        default_value: None,
                    },
                ],
                derives: vec![],
                is_pub: true,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 47: transpile_struct_expr - TupleStruct definition
    #[test]
    fn test_transpile_struct_expr_tuple_struct() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::TupleStruct {
                name: "Wrapper".to_string(),
                type_params: vec![],
                fields: vec!["i32".to_string()],
                derives: vec![],
                is_pub: true,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 48: transpile_struct_expr - Class definition
    #[test]
    fn test_transpile_struct_expr_class() {
        use crate::frontend::ast::StructField;
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Class {
                name: "MyClass".to_string(),
                type_params: vec![],
                superclass: None,
                traits: vec![],
                fields: vec![
                    StructField {
                        name: "value".to_string(),
                        field_type: "i32".to_string(),
                        is_pub: true,
                        default_value: None,
                    },
                ],
                constructors: vec![],
                methods: vec![],
                constants: vec![],
                properties: vec![],
                derives: vec![],
                is_pub: true,
                is_sealed: false,
                is_abstract: false,
                decorators: vec![],
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_struct_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 49: transpile_data_error_expr - Set
    #[test]
    fn test_transpile_data_error_expr_set() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::Set(vec![int_expr(1), int_expr(2), int_expr(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_data_error_expr(&expr);
        assert!(result.is_ok());
    }

    // Test 50: transpile_data_error_expr - ArrayInit
    #[test]
    fn test_transpile_data_error_expr_array_init() {
        let transpiler = test_transpiler();
        let expr = Expr {
            kind: ExprKind::ArrayInit {
                elem: Box::new(int_expr(0)),
                count: Box::new(int_expr(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler.transpile_data_error_expr(&expr);
        assert!(result.is_ok());
    }
}
