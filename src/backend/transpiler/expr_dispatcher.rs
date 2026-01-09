//! Expression Dispatcher and Utilities
//!
//! This module handles the main expression transpilation dispatcher:
//! - Routes expressions to specialized handlers based on `ExprKind`
//! - Rust keyword detection utility
//!
//! **EXTREME TDD Round 69**: Extracted from mod.rs for modularization.

#![allow(clippy::doc_markdown)]

use super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind};
use anyhow::Result;
use proc_macro2::TokenStream;

impl Transpiler {
    /// Check if a name is a Rust reserved keyword
    /// Complexity: 1 (within Toyota Way limits)
    pub(crate) fn is_rust_reserved_keyword(name: &str) -> bool {
        matches!(
            name,
            "as" | "break"
                | "const"
                | "continue"
                | "crate"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
                | "async"
                | "await"
                | "dyn"
                | "final"
                | "try"
                | "abstract"
                | "become"
                | "box"
                | "do"
                | "macro"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "yield"
        )
    }

    /// Main expression transpilation dispatcher
    ///
    /// Routes expressions to specialized handlers based on ExprKind.
    /// This keeps the main dispatch logic centralized while delegating
    /// complex transpilation to focused sub-modules.
    ///
    /// # Panics
    ///
    /// Panics if label names cannot be parsed as valid Rust tokens
    ///
    /// Complexity: 6 (within Toyota Way limits)
    pub fn transpile_expr(&self, expr: &Expr) -> Result<TokenStream> {
        use ExprKind::{
            Actor, ActorQuery, ActorSend, ArrayInit, Ask, Assign, AsyncBlock, AsyncLambda, Await,
            Binary, Call, Class, Command, CompoundAssign, DataFrame, DataFrameOperation,
            DictComprehension, Effect, Err, FieldAccess, For, Function, Handle, Identifier, If,
            IfLet, IndexAccess, Lambda, List, ListComprehension, Literal, Loop, Macro, Match,
            MethodCall, None, ObjectLiteral, Ok, PostDecrement, PostIncrement, PreDecrement,
            PreIncrement, QualifiedName, Range, Send, Set, SetComprehension, Slice, Some, Spawn,
            StringInterpolation, Struct, StructLiteral, Throw, Try, TryCatch, Tuple, TupleStruct,
            TypeCast, Unary, While, WhileLet,
        };

        // Dispatch to specialized handlers to keep complexity below 10
        match &expr.kind {
            // Basic expressions
            Literal(_)
            | Identifier(_)
            | QualifiedName { .. }
            | StringInterpolation { .. }
            | TypeCast { .. } => self.transpile_basic_expr(expr),

            // Operators and control flow
            Binary { .. }
            | Unary { .. }
            | Assign { .. }
            | CompoundAssign { .. }
            | PreIncrement { .. }
            | PostIncrement { .. }
            | PreDecrement { .. }
            | PostDecrement { .. }
            | Await { .. }
            | Spawn { .. }
            | AsyncBlock { .. }
            | AsyncLambda { .. }
            | If { .. }
            | IfLet { .. }
            | Match { .. }
            | For { .. }
            | While { .. }
            | WhileLet { .. }
            | Loop { .. }
            | TryCatch { .. } => self.transpile_operator_control_expr(expr),

            // Functions
            Function { .. } | Lambda { .. } | Call { .. } | MethodCall { .. } | Macro { .. } => {
                self.transpile_function_expr(expr)
            }

            // Structures
            Struct { .. }
            | TupleStruct { .. }
            | Class { .. }
            | StructLiteral { .. }
            | ObjectLiteral { .. }
            | FieldAccess { .. }
            | IndexAccess { .. }
            | Slice { .. } => self.transpile_struct_expr(expr),

            // Data and error handling
            DataFrame { .. }
            | DataFrameOperation { .. }
            | List(_)
            | Set(_)
            | ArrayInit { .. }
            | Tuple(_)
            | ListComprehension { .. }
            | SetComprehension { .. }
            | DictComprehension { .. }
            | Range { .. }
            | Throw { .. }
            | Ok { .. }
            | Err { .. }
            | Some { .. }
            | None
            | Try { .. } => self.transpile_data_error_expr(expr),

            // Actor system and process execution
            Actor { .. }
            | Effect { .. }
            | Handle { .. }
            | Send { .. }
            | Ask { .. }
            | ActorSend { .. }
            | ActorQuery { .. }
            | Command { .. } => self.transpile_actor_expr(expr),

            // Everything else
            _ => self.transpile_misc_expr(expr),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn int_expr(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn float_expr(f: f64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Float(f)))
    }

    fn string_expr(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn bool_expr(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    fn ident_expr(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    // ========================================================================
    // is_rust_reserved_keyword tests
    // ========================================================================

    #[test]
    fn test_is_rust_reserved_keyword_common() {
        assert!(Transpiler::is_rust_reserved_keyword("fn"));
        assert!(Transpiler::is_rust_reserved_keyword("let"));
        assert!(Transpiler::is_rust_reserved_keyword("if"));
        assert!(Transpiler::is_rust_reserved_keyword("else"));
        assert!(Transpiler::is_rust_reserved_keyword("for"));
        assert!(Transpiler::is_rust_reserved_keyword("while"));
        assert!(Transpiler::is_rust_reserved_keyword("loop"));
        assert!(Transpiler::is_rust_reserved_keyword("match"));
    }

    #[test]
    fn test_is_rust_reserved_keyword_types() {
        assert!(Transpiler::is_rust_reserved_keyword("struct"));
        assert!(Transpiler::is_rust_reserved_keyword("enum"));
        assert!(Transpiler::is_rust_reserved_keyword("trait"));
        assert!(Transpiler::is_rust_reserved_keyword("impl"));
        assert!(Transpiler::is_rust_reserved_keyword("type"));
    }

    #[test]
    fn test_is_rust_reserved_keyword_modifiers() {
        assert!(Transpiler::is_rust_reserved_keyword("pub"));
        assert!(Transpiler::is_rust_reserved_keyword("mut"));
        assert!(Transpiler::is_rust_reserved_keyword("const"));
        assert!(Transpiler::is_rust_reserved_keyword("static"));
        assert!(Transpiler::is_rust_reserved_keyword("unsafe"));
    }

    #[test]
    fn test_is_rust_reserved_keyword_async() {
        assert!(Transpiler::is_rust_reserved_keyword("async"));
        assert!(Transpiler::is_rust_reserved_keyword("await"));
    }

    #[test]
    fn test_is_rust_reserved_keyword_future() {
        assert!(Transpiler::is_rust_reserved_keyword("abstract"));
        assert!(Transpiler::is_rust_reserved_keyword("become"));
        assert!(Transpiler::is_rust_reserved_keyword("box"));
        assert!(Transpiler::is_rust_reserved_keyword("do"));
        assert!(Transpiler::is_rust_reserved_keyword("final"));
        assert!(Transpiler::is_rust_reserved_keyword("macro"));
        assert!(Transpiler::is_rust_reserved_keyword("override"));
        assert!(Transpiler::is_rust_reserved_keyword("priv"));
        assert!(Transpiler::is_rust_reserved_keyword("typeof"));
        assert!(Transpiler::is_rust_reserved_keyword("unsized"));
        assert!(Transpiler::is_rust_reserved_keyword("virtual"));
        assert!(Transpiler::is_rust_reserved_keyword("yield"));
    }

    #[test]
    fn test_is_rust_reserved_keyword_not_reserved() {
        assert!(!Transpiler::is_rust_reserved_keyword("foo"));
        assert!(!Transpiler::is_rust_reserved_keyword("bar"));
        assert!(!Transpiler::is_rust_reserved_keyword("my_func"));
        assert!(!Transpiler::is_rust_reserved_keyword("MyStruct"));
        assert!(!Transpiler::is_rust_reserved_keyword("i32"));
        assert!(!Transpiler::is_rust_reserved_keyword("String"));
    }

    #[test]
    fn test_is_rust_reserved_keyword_special() {
        assert!(Transpiler::is_rust_reserved_keyword("self"));
        assert!(Transpiler::is_rust_reserved_keyword("Self"));
        assert!(Transpiler::is_rust_reserved_keyword("super"));
        assert!(Transpiler::is_rust_reserved_keyword("crate"));
    }

    // ========================================================================
    // transpile_expr dispatcher tests
    // ========================================================================

    #[test]
    fn test_transpile_expr_literal_int() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&int_expr(42));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("42"));
    }

    #[test]
    fn test_transpile_expr_literal_float() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&float_expr(3.14));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("3.14"));
    }

    #[test]
    fn test_transpile_expr_literal_string() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&string_expr("hello"));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("hello"));
    }

    #[test]
    fn test_transpile_expr_literal_bool() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&bool_expr(true));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("true"));
    }

    #[test]
    fn test_transpile_expr_identifier() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&ident_expr("my_var"));
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("my_var"));
    }

    #[test]
    fn test_transpile_expr_list() {
        let transpiler = Transpiler::new();
        let list = make_expr(ExprKind::List(vec![int_expr(1), int_expr(2), int_expr(3)]));
        let result = transpiler.transpile_expr(&list);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        // Non-empty lists transpile to array syntax [1, 2, 3]
        assert!(code.contains('[') && code.contains(']'));
    }

    #[test]
    fn test_transpile_expr_tuple() {
        let transpiler = Transpiler::new();
        let tuple = make_expr(ExprKind::Tuple(vec![int_expr(1), int_expr(2)]));
        let result = transpiler.transpile_expr(&tuple);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_expr_none() {
        let transpiler = Transpiler::new();
        let none = make_expr(ExprKind::None);
        let result = transpiler.transpile_expr(&none);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("None"));
    }

    #[test]
    fn test_transpile_expr_some() {
        let transpiler = Transpiler::new();
        let some = make_expr(ExprKind::Some {
            value: Box::new(int_expr(42)),
        });
        let result = transpiler.transpile_expr(&some);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Some"));
    }

    #[test]
    fn test_transpile_expr_ok() {
        let transpiler = Transpiler::new();
        let ok = make_expr(ExprKind::Ok {
            value: Box::new(int_expr(42)),
        });
        let result = transpiler.transpile_expr(&ok);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Ok"));
    }

    #[test]
    fn test_transpile_expr_err() {
        let transpiler = Transpiler::new();
        let err = make_expr(ExprKind::Err {
            error: Box::new(string_expr("error")),
        });
        let result = transpiler.transpile_expr(&err);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("Err"));
    }

    #[test]
    fn test_transpile_expr_range() {
        let transpiler = Transpiler::new();
        let range = make_expr(ExprKind::Range {
            start: Box::new(int_expr(0)),
            end: Box::new(int_expr(10)),
            inclusive: false,
        });
        let result = transpiler.transpile_expr(&range);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpile_expr_range_inclusive() {
        let transpiler = Transpiler::new();
        let range = make_expr(ExprKind::Range {
            start: Box::new(int_expr(0)),
            end: Box::new(int_expr(10)),
            inclusive: true,
        });
        let result = transpiler.transpile_expr(&range);
        assert!(result.is_ok());
    }
}
