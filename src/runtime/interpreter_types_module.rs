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

    #[test]
    fn test_eval_module_function_with_default_param() {
        let mut interp = make_interpreter();

        // Create: mod defaults { pub fn greet(name = "World") { name } }
        let fn_body = make_expr(ExprKind::Identifier("name".to_string()));
        let default_expr = make_expr(ExprKind::Literal(Literal::String("World".to_string())));

        let param_with_default = Param {
            pattern: Pattern::Identifier("name".to_string()),
            ty: make_type("String"),
            span: Span::default(),
            is_mutable: false,
            default_value: Some(Box::new(default_expr)),
        };

        let func = make_expr(ExprKind::Function {
            name: "greet".to_string(),
            params: vec![param_with_default],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("defaults", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("greet"));
            // Verify closure captured default value
            if let Some(Value::Closure { params, .. }) = obj.get("greet") {
                assert_eq!(params.len(), 1);
                assert!(params[0].1.is_some()); // Has default value
            } else {
                panic!("Expected Closure");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_async_function() {
        let mut interp = make_interpreter();

        // Create: mod async_mod { pub async fn fetch() { 1 } }
        let fn_body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));

        let func = make_expr(ExprKind::Function {
            name: "fetch".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: true,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("async_mod", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("fetch"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_mixed_expressions() {
        let mut interp = make_interpreter();

        // Module with function AND non-function expressions
        let fn_body = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let func = make_expr(ExprKind::Function {
            name: "process".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        // Add a let binding - should be ignored
        let let_expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
            body: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            is_mutable: false,
            else_block: None,
        });

        let body = make_expr(ExprKind::Block(vec![let_expr, func]));
        let result = interp.eval_module_expr("mixed", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("process"));
            // x should not be exposed in module namespace
            assert!(!obj.contains_key("x"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_deeply_nested() {
        let mut interp = make_interpreter();

        // Create: mod a { mod b { mod c { pub fn deep() { 99 } } } }
        let fn_body = make_expr(ExprKind::Literal(Literal::Integer(99, None)));
        let deep_fn = make_expr(ExprKind::Function {
            name: "deep".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let c_body = make_expr(ExprKind::Block(vec![deep_fn]));
        let c_module = make_expr(ExprKind::Module {
            name: "c".to_string(),
            body: Box::new(c_body),
        });

        let b_body = make_expr(ExprKind::Block(vec![c_module]));
        let b_module = make_expr(ExprKind::Module {
            name: "b".to_string(),
            body: Box::new(b_body),
        });

        let a_body = make_expr(ExprKind::Block(vec![b_module]));
        let result = interp.eval_module_expr("a", &a_body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("b"));
            if let Some(Value::Object(b_obj)) = obj.get("b") {
                assert!(b_obj.contains_key("c"));
                if let Some(Value::Object(c_obj)) = b_obj.get("c") {
                    assert!(c_obj.contains_key("deep"));
                } else {
                    panic!("Expected c module");
                }
            } else {
                panic!("Expected b module");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_mutable_param() {
        let mut interp = make_interpreter();

        // Create: mod mutables { pub fn modify(mut x) { x } }
        let fn_body = make_expr(ExprKind::Identifier("x".to_string()));

        let mutable_param = Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: make_type("i32"),
            span: Span::default(),
            is_mutable: true,
            default_value: None,
        };

        let func = make_expr(ExprKind::Function {
            name: "modify".to_string(),
            params: vec![mutable_param],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("mutables", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("modify"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_with_type_params() {
        let mut interp = make_interpreter();

        // Create: mod generics { pub fn identity<T>(x: T) { x } }
        let fn_body = make_expr(ExprKind::Identifier("x".to_string()));

        let func = make_expr(ExprKind::Function {
            name: "identity".to_string(),
            params: vec![make_param("x")],
            return_type: None,
            body: Box::new(fn_body),
            type_params: vec!["T".to_string()],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("generics", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("identity"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_with_return_type() {
        let mut interp = make_interpreter();

        // Create: mod typed { pub fn get_number() -> i32 { 42 } }
        let fn_body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let func = make_expr(ExprKind::Function {
            name: "get_number".to_string(),
            params: vec![],
            return_type: Some(make_type("i32")),
            body: Box::new(fn_body),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func]));
        let result = interp.eval_module_expr("typed", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("get_number"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_mixed_visibility() {
        let mut interp = make_interpreter();

        // Create: mod mixed_vis { pub fn public() { 1 } fn private() { 2 } pub fn also_public() { 3 } }
        let pub1 = make_expr(ExprKind::Function {
            name: "public".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let private = make_expr(ExprKind::Function {
            name: "private".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            type_params: vec![],
            is_pub: false,
            is_async: false,
        });

        let pub2 = make_expr(ExprKind::Function {
            name: "also_public".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(3, None)))),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![pub1, private, pub2]));
        let result = interp.eval_module_expr("mixed_vis", &body).unwrap();

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("public"));
            assert!(obj.contains_key("also_public"));
            assert!(!obj.contains_key("private"));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_module_sibling_functions_environment() {
        let mut interp = make_interpreter();

        // Test that sibling functions share module environment
        // mod siblings { pub fn a() { 1 } pub fn b() { 2 } }
        let func_a = make_expr(ExprKind::Function {
            name: "a".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let func_b = make_expr(ExprKind::Function {
            name: "b".to_string(),
            params: vec![],
            return_type: None,
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            type_params: vec![],
            is_pub: true,
            is_async: false,
        });

        let body = make_expr(ExprKind::Block(vec![func_a, func_b]));
        let result = interp.eval_module_expr("siblings", &body).unwrap();

        if let Value::Object(obj) = result {
            // Both functions should have the same environment (module_env)
            if let (
                Some(Value::Closure { env: env_a, .. }),
                Some(Value::Closure { env: env_b, .. }),
            ) = (obj.get("a"), obj.get("b"))
            {
                // Both closures should have "a" and "b" in their env
                let env_a_ref = env_a.borrow();
                let env_b_ref = env_b.borrow();
                assert!(env_a_ref.contains_key("a"));
                assert!(env_a_ref.contains_key("b"));
                assert!(env_b_ref.contains_key("a"));
                assert!(env_b_ref.contains_key("b"));
            } else {
                panic!("Expected Closures");
            }
        } else {
            panic!("Expected Object");
        }
    }
}
