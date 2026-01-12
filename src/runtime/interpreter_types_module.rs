//! Module expression evaluation
//!
//! Extracted from interpreter_types_impl.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]

use crate::frontend::ast::{Expr, ExprKind};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::cell::RefCell;
use std::rc::Rc;

impl Interpreter {
    /// ISSUE-106: Evaluate module expression
    /// Creates a namespace object containing all functions defined in the module body
    /// MODULE-001 FIX: Two-pass approach so intra-module calls work
    /// Complexity: 8
    pub(crate) fn eval_module_expr(
        &mut self,
        name: &str,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::sync::Arc;

        // Extract function definitions from the module body
        let exprs = match &body.kind {
            ExprKind::Block(exprs) => exprs.clone(),
            _ => vec![body.clone()],
        };

        // PASS 1: Create a module-scoped environment with all functions
        // This allows intra-module function calls to work (MODULE-001 fix)
        // Start with a copy of parent environment
        let parent_env = self.env_stack.last().unwrap_or(&self.env_stack[0]);
        let module_env_map: HashMap<String, Value> = parent_env.borrow().clone();
        let module_env = Rc::new(RefCell::new(module_env_map));

        // Collect all functions and nested modules for module-internal access
        for expr in &exprs {
            match &expr.kind {
                // Handle function definitions
                ExprKind::Function {
                    name: fn_name,
                    params,
                    body: fn_body,
                    ..
                } => {
                    let closure_params: Vec<(String, Option<Arc<Expr>>)> = params
                        .iter()
                        .map(|p| {
                            let param_name = p.name();
                            let default = p
                                .default_value
                                .as_ref()
                                .map(|d| Arc::new(d.as_ref().clone()));
                            (param_name, default)
                        })
                        .collect();
                    // Create closure with module-scoped environment
                    let closure = Value::Closure {
                        params: closure_params,
                        body: Arc::new(fn_body.as_ref().clone()),
                        env: Rc::clone(&module_env),
                    };
                    // Add to module environment so sibling functions can call each other
                    module_env.borrow_mut().insert(fn_name.clone(), closure);
                }
                // MODULE-002 FIX: Handle nested module definitions
                ExprKind::Module {
                    name: nested_name,
                    body: nested_body,
                } => {
                    // Push module environment for nested module evaluation
                    self.env_stack.push(Rc::clone(&module_env));
                    // Recursively evaluate nested module
                    let nested_module = self.eval_module_expr(nested_name, nested_body)?;
                    self.env_stack.pop();
                    // Add nested module to parent module's environment
                    module_env
                        .borrow_mut()
                        .insert(nested_name.clone(), nested_module);
                }
                _ => {}
            }
        }

        // PASS 2: Build the public namespace for external access
        let mut module_namespace: HashMap<String, Value> = HashMap::new();
        module_namespace.insert(
            "__type".to_string(),
            Value::from_string("Module".to_string()),
        );
        module_namespace.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Only expose public functions and modules in the module namespace
        for expr in &exprs {
            match &expr.kind {
                // Public functions
                ExprKind::Function {
                    name: fn_name,
                    is_pub: true,
                    ..
                } => {
                    // Get the closure from module_env (already created with correct scope)
                    if let Some(closure) = module_env.borrow().get(fn_name) {
                        module_namespace.insert(fn_name.clone(), closure.clone());
                    }
                }
                // MODULE-002 FIX: Public nested modules
                ExprKind::Module {
                    name: nested_name, ..
                } => {
                    // Check if nested module is public (currently all modules are public)
                    if let Some(nested_mod) = module_env.borrow().get(nested_name) {
                        module_namespace.insert(nested_name.clone(), nested_mod.clone());
                    }
                }
                _ => {}
            }
        }

        // Register the module in the global environment
        let module_value = Value::Object(Arc::new(module_namespace));
        self.set_variable(name, module_value.clone());

        Ok(module_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Param, Pattern, Span, Type, TypeKind};

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    fn make_param(name: &str) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: make_type("Any"),
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    #[test]
    fn test_eval_module_expr_empty() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Block(vec![]));
        let result = interp.eval_module_expr("empty_mod", &body).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Module".to_string()))
            );
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("empty_mod".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_expr_with_public_function() {
        let mut interp = make_interpreter();

        // Create: mod math { pub fn add(a, b) { a + b } }
        let fn_body = make_expr(ExprKind::Binary {
            op: crate::frontend::ast::BinaryOp::Add,
            left: Box::new(make_expr(ExprKind::Identifier("a".to_string()))),
            right: Box::new(make_expr(ExprKind::Identifier("b".to_string()))),
        });

        let func = make_expr(ExprKind::Function {
            name: "add".to_string(),
            params: vec![make_param("a"), make_param("b")],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("math", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("add"));
            assert!(matches!(obj.get("add"), Some(Value::Closure { .. })));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_expr_private_function_not_exposed() {
        let mut interp = make_interpreter();

        // Create: mod util { fn helper() { 1 } }
        let fn_body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let func = make_expr(ExprKind::Function {
            name: "helper".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: false, // Private
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("util", &body).unwrap();

        if let Value::Object(obj) = result {
            // Private function should NOT be exposed
            assert!(!obj.contains_key("helper"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_expr_nested_module() {
        let mut interp = make_interpreter();

        // Create: mod outer { mod inner { pub fn foo() { 42 } } }
        let fn_body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let inner_func = make_expr(ExprKind::Function {
            name: "foo".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let inner_body = make_expr(ExprKind::Block(vec![inner_func]));

        let inner_module = make_expr(ExprKind::Module {
            name: "inner".to_string(),
            body: Box::new(inner_body),
        });

        let outer_body = make_expr(ExprKind::Block(vec![inner_module]));
        let result = interp.eval_module_expr("outer", &outer_body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("inner"));
            if let Some(Value::Object(inner)) = obj.get("inner") {
                assert!(inner.contains_key("foo"));
            } else {
                panic!("Expected inner module");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_expr_non_block_body() {
        let mut interp = make_interpreter();

        // Module with single expression instead of block
        let single = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let result = interp.eval_module_expr("single", &single).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Module".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_registered_in_environment() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Block(vec![]));

        interp.eval_module_expr("my_mod", &body).unwrap();

        // Verify module is accessible in environment
        let lookup = interp.lookup_variable("my_mod");
        assert!(lookup.is_ok());
        if let Value::Object(obj) = lookup.unwrap() {
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("my_mod".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_multiple_functions() {
        let mut interp = make_interpreter();

        // Create: mod lib { pub fn foo() { 1 } pub fn bar() { 2 } }
        let foo_body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let foo = make_expr(ExprKind::Function {
            name: "foo".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(foo_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let bar_body = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let bar = make_expr(ExprKind::Function {
            name: "bar".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(bar_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![foo, bar]));
        let result = interp.eval_module_expr("lib", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("foo"));
            assert!(obj.contains_key("bar"));
        } else {
            panic!("Expected Object");
        }
    }
}
