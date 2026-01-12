//! Function evaluation implementation module
//!
//! This module handles function definitions, lambda expressions,
//! and function calls.
//! Extracted from interpreter.rs for maintainability.

#![allow(clippy::unused_self)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_possible_truncation)]

use super::eval_func;
use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::Param;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

impl Interpreter {
    /// Evaluate function definition
    pub(crate) fn eval_function(
        &mut self,
        name: &str,
        params: &[Param],
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
        let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
            .iter()
            .map(|p| {
                (
                    p.name(),
                    p.default_value
                        .clone()
                        .map(|expr| Arc::new((*expr).clone())),
                )
            })
            .collect();

        let closure = Value::Closure {
            params: params_with_defaults,
            body: Arc::new(body.clone()),
            env: self.current_env().clone(), // ISSUE-119: Rc::clone (shallow copy, already wrapped)
        };

        // Bind function name in environment for recursion
        self.env_set(name.to_string(), closure.clone());
        Ok(closure)
    }

    /// Evaluate lambda expression
    pub(crate) fn eval_lambda(
        &mut self,
        params: &[Param],
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        eval_func::eval_lambda(params, body, self.current_env())
    }

    /// Extract named arguments from assignment expressions and reorder them
    /// to match function parameter order.
    ///
    /// Named arguments use the syntax `param_name = value` and can be in any order.
    /// This function detects such assignment expressions and reorders them to match
    /// the function's parameter order.
    ///
    /// # Arguments
    /// * `args` - The argument expressions from the call site
    /// * `param_names` - The parameter names from the function definition
    ///
    /// # Returns
    /// A reordered list of argument expressions matching parameter order
    pub(crate) fn reorder_named_args<'a>(
        &self,
        args: &'a [Expr],
        param_names: &[String],
    ) -> Vec<&'a Expr> {
        // Extract named args: (name, expr) for Assign expressions, (None, expr) for positional
        let mut named_args: Vec<(Option<String>, &Expr)> = Vec::new();

        for arg in args {
            if let ExprKind::Assign { target, value } = &arg.kind {
                if let ExprKind::Identifier(name) = &target.kind {
                    // This is a named argument: name = value
                    named_args.push((Some(name.clone()), value.as_ref()));
                } else {
                    // Assignment to non-identifier, treat as positional
                    named_args.push((None, arg));
                }
            } else {
                // Positional argument
                named_args.push((None, arg));
            }
        }

        // Check if we have any named arguments
        let has_named = named_args.iter().any(|(name, _)| name.is_some());
        if !has_named {
            // No named args, return original order
            return args.iter().collect();
        }

        // Reorder arguments to match parameter order
        // Only include slots up to the number of provided args
        let mut result: Vec<Option<&Expr>> = vec![None; param_names.len()];
        let mut positional_idx = 0;

        for (name, expr) in &named_args {
            if let Some(param_name) = name {
                // Named argument - find its position
                if let Some(pos) = param_names.iter().position(|p| p == param_name) {
                    result[pos] = Some(*expr);
                } else {
                    // Unknown parameter name - this will be an error at call time
                    // For now, just append to results
                    if positional_idx < result.len() {
                        while positional_idx < result.len() && result[positional_idx].is_some() {
                            positional_idx += 1;
                        }
                        if positional_idx < result.len() {
                            result[positional_idx] = Some(*expr);
                        }
                    }
                }
            } else {
                // Positional argument - find next empty slot
                while positional_idx < result.len() && result[positional_idx].is_some() {
                    positional_idx += 1;
                }
                if positional_idx < result.len() {
                    result[positional_idx] = Some(*expr);
                    positional_idx += 1;
                }
            }
        }

        // Count how many args were actually provided
        let provided_count = named_args.len();

        // Only return up to the number of provided args, in parameter order
        // This allows default params to kick in for any trailing None slots
        result.into_iter().take(provided_count).flatten().collect()
    }

    /// Evaluate function call
    pub(crate) fn eval_function_call(
        &mut self,
        func: &Expr,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        // Handle static method calls: Type::method(args)
        // Parser represents these as Call { func: FieldAccess { object: Identifier("Type"), field: "method" }, args }
        if let ExprKind::FieldAccess { object, field } = &func.kind {
            if let ExprKind::Identifier(type_name) = &object.kind {
                // Detect Box::new(value) static method
                if type_name == "Box" && field == "new" {
                    if args.len() != 1 {
                        return Err(InterpreterError::RuntimeError(format!(
                            "Box::new() requires exactly 1 argument, got {}",
                            args.len()
                        )));
                    }
                    // Box::new(value) → just return the value (Box is transparent in Ruchy)
                    return self.eval_expr(&args[0]);
                }
                // Detect Vec::new() static method
                if type_name == "Vec" && field == "new" {
                    if !args.is_empty() {
                        return Err(InterpreterError::RuntimeError(format!(
                            "Vec::new() takes no arguments, got {}",
                            args.len()
                        )));
                    }
                    // Vec::new() → empty array
                    return Ok(Value::Array(Arc::from([])));
                }

                // ISSUE-117: JSON.parse() and JSON.stringify() methods
                if type_name == "JSON" {
                    if field == "parse" {
                        if args.len() != 1 {
                            return Err(InterpreterError::RuntimeError(format!(
                                "JSON.parse() requires exactly 1 argument, got {}",
                                args.len()
                            )));
                        }
                        let json_str_val = self.eval_expr(&args[0])?;
                        let json_str = json_str_val.to_string();
                        return self.json_parse(&json_str);
                    } else if field == "stringify" {
                        if args.len() != 1 {
                            return Err(InterpreterError::RuntimeError(format!(
                                "JSON.stringify() requires exactly 1 argument, got {}",
                                args.len()
                            )));
                        }
                        let value = self.eval_expr(&args[0])?;
                        return self.json_stringify(&value);
                    }
                }

                // ISSUE-116: File.open() method
                if type_name == "File" && field == "open" {
                    if args.len() != 1 {
                        return Err(InterpreterError::RuntimeError(format!(
                            "File.open() requires exactly 1 argument, got {}",
                            args.len()
                        )));
                    }
                    let path_val = self.eval_expr(&args[0])?;
                    // Call builtin File_open through eval_builtin_function
                    return crate::runtime::eval_builtin::eval_builtin_function(
                        "File_open",
                        &[path_val],
                    )?
                    .ok_or_else(|| {
                        InterpreterError::RuntimeError("File_open builtin not found".to_string())
                    });
                }

                // REGRESSION-077: Check for user-defined struct impl methods
                // impl methods are stored with qualified names like "Logger::new_with_options"
                let qualified_method = format!("{}::{}", type_name, field);
                if let Ok(method_value) = self.lookup_variable(&qualified_method) {
                    // Found impl method - evaluate args and call it
                    let arg_vals: Result<Vec<Value>, InterpreterError> =
                        args.iter().map(|arg| self.eval_expr(arg)).collect();
                    let arg_vals = arg_vals?;
                    return self.call_function(method_value, &arg_vals);
                }
            }
        }

        // NAMED-PARAMS-FIX: Check for named arguments (assignment expressions) and reorder
        // Named arguments allow: greet(greeting = "Hi", name = "Bob") -> greet("Bob", "Hi")
        let has_named_args = args.iter().any(|arg| {
            matches!(
                &arg.kind,
                ExprKind::Assign { target, .. } if matches!(&target.kind, ExprKind::Identifier(_))
            )
        });

        // If we have named arguments, try to look up the function first to get parameter names
        let reordered_args: Vec<&Expr> = if has_named_args {
            // Try to get function's parameter names
            if let ExprKind::Identifier(func_name) = &func.kind {
                if let Ok(func_val) = self.lookup_variable(func_name) {
                    if let Value::Closure { params, .. } = &func_val {
                        let param_names: Vec<String> =
                            params.iter().map(|(name, _)| name.clone()).collect();
                        self.reorder_named_args(args, &param_names)
                    } else {
                        args.iter().collect()
                    }
                } else {
                    args.iter().collect()
                }
            } else {
                args.iter().collect()
            }
        } else {
            args.iter().collect()
        };

        // ISSUE-119 FIX: Evaluate args ONCE at the start to prevent double-evaluation
        // This ensures that side-effects (like counter++) only happen once
        let arg_vals: Vec<Value> = reordered_args
            .iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<Vec<_>, _>>()?;

        // ISSUE-117: Check builtin functions BEFORE variable lookup
        // This ensures parse_json(), stringify_json(), open(), etc. work as functions
        if let ExprKind::Identifier(name) = &func.kind {
            // ISSUE-119 FIX: Convert name to builtin marker format (__builtin_NAME__)
            // to match eval_builtin::try_eval_io_function expectations
            let builtin_name = format!("__builtin_{}__", name);

            // RUNTIME-BUG-002: Propagate builtin function errors instead of falling back to Message objects
            match crate::runtime::eval_builtin::eval_builtin_function(&builtin_name, &arg_vals) {
                Ok(Some(result)) => return Ok(result),
                Ok(None) => {}           // Fall through to normal function evaluation
                Err(e) => return Err(e), // Propagate error (parse_int/parse_float errors, etc.)
            }
        }

        // Try to evaluate the function normally
        let func_val_result = self.eval_expr(func);

        // If function lookup fails and it's an identifier, treat it as a message constructor
        let func_val = match func_val_result {
            Ok(val) => val,
            Err(InterpreterError::RuntimeError(msg)) if msg.starts_with("Undefined variable:") => {
                // Check if this is an identifier that could be a message constructor
                if let ExprKind::Identifier(name) = &func.kind {
                    // Create a message object - args already evaluated above
                    let mut message = HashMap::new();
                    message.insert(
                        "__type".to_string(),
                        Value::from_string("Message".to_string()),
                    );
                    message.insert("type".to_string(), Value::from_string(name.clone()));
                    message.insert("data".to_string(), Value::Array(Arc::from(arg_vals)));

                    return Ok(Value::Object(Arc::new(message)));
                }
                return Err(InterpreterError::RuntimeError(msg));
            }
            Err(e) => return Err(e),
        };

        // arg_vals already evaluated at the start - no need to re-evaluate

        // Special handling for enum variant construction with arguments (tuple variants)
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data: _,
        } = func_val
        {
            // This is a tuple variant constructor: Response::Error("msg")
            return Ok(Value::EnumVariant {
                enum_name,
                variant_name,
                data: Some(arg_vals),
            });
        }

        // DEBUGGER-014 Phase 1.3: Extract function name for tracing
        let func_name = match &func.kind {
            ExprKind::Identifier(name) => name.clone(),
            _ => "anonymous".to_string(),
        };

        // DEBUGGER-014 Phase 3: Type-aware tracing with argument/return values and types
        let trace_enabled = std::env::var("RUCHY_TRACE").is_ok();
        if trace_enabled {
            // Format argument values with type annotations for trace output
            let args_str = arg_vals
                .iter()
                .map(|v| {
                    // Format value with proper string quoting + type annotation
                    let value_str = match v {
                        Value::String(s) => format!("\"{}\"", s),
                        other => other.to_string(),
                    };
                    format!("{}: {}", value_str, v.type_name())
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!("TRACE: → {}({})", func_name, args_str);
        }

        let result = self.call_function(func_val, &arg_vals)?;

        // DEBUGGER-014 Phase 3: Trace function exit with return value and type
        if trace_enabled {
            let result_str = match &result {
                Value::String(s) => format!("\"{}\"", s),
                other => other.to_string(),
            };
            println!(
                "TRACE: ← {} = {}: {}",
                func_name,
                result_str,
                result.type_name()
            );
        }

        // Collect type feedback for function call
        let site_id = func.span.start; // Use func span start as site ID
        self.record_function_call_feedback(site_id, &func_name, &arg_vals, &result);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Pattern, Span, Type, TypeKind};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

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

    // eval_function tests
    #[test]
    fn test_eval_function_simple() {
        let mut interp = make_interpreter();
        let params = vec![make_param("x")];
        let body = make_expr(ExprKind::Identifier("x".to_string()));

        let result = interp.eval_function("identity", &params, &body).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    #[test]
    fn test_eval_function_no_params() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(42, None)));

        let result = interp.eval_function("constant", &[], &body).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    #[test]
    fn test_eval_function_multiple_params() {
        let mut interp = make_interpreter();
        let params = vec![make_param("a"), make_param("b"), make_param("c")];
        let body = make_expr(ExprKind::Identifier("a".to_string()));

        let result = interp.eval_function("multi", &params, &body).unwrap();
        if let Value::Closure { params: p, .. } = result {
            assert_eq!(p.len(), 3);
        } else {
            panic!("Expected Closure");
        }
    }

    #[test]
    fn test_eval_function_registered_in_env() {
        let mut interp = make_interpreter();
        let params = vec![make_param("x")];
        let body = make_expr(ExprKind::Identifier("x".to_string()));

        interp.eval_function("my_func", &params, &body).unwrap();

        let lookup = interp.lookup_variable("my_func");
        assert!(lookup.is_ok());
        assert!(matches!(lookup.unwrap(), Value::Closure { .. }));
    }

    // eval_lambda tests
    #[test]
    fn test_eval_lambda_simple() {
        let mut interp = make_interpreter();
        let params = vec![make_param("x")];
        let body = make_expr(ExprKind::Identifier("x".to_string()));

        let result = interp.eval_lambda(&params, &body).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    #[test]
    fn test_eval_lambda_no_params() {
        let mut interp = make_interpreter();
        let body = make_expr(ExprKind::Literal(Literal::Integer(100, None)));

        let result = interp.eval_lambda(&[], &body).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    // reorder_named_args tests
    #[test]
    fn test_reorder_named_args_no_named() {
        let interp = make_interpreter();
        let args = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ];
        let param_names = vec!["a".to_string(), "b".to_string()];

        let result = interp.reorder_named_args(&args, &param_names);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_reorder_named_args_with_named() {
        let interp = make_interpreter();
        // Create: b = 2, a = 1
        let args = vec![
            make_expr(ExprKind::Assign {
                target: Box::new(make_expr(ExprKind::Identifier("b".to_string()))),
                value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            }),
            make_expr(ExprKind::Assign {
                target: Box::new(make_expr(ExprKind::Identifier("a".to_string()))),
                value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            }),
        ];
        let param_names = vec!["a".to_string(), "b".to_string()];

        let result = interp.reorder_named_args(&args, &param_names);
        // Should reorder to match param order
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_reorder_named_args_mixed() {
        let interp = make_interpreter();
        // Create: 1, b = 2
        let args = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Assign {
                target: Box::new(make_expr(ExprKind::Identifier("b".to_string()))),
                value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            }),
        ];
        let param_names = vec!["a".to_string(), "b".to_string()];

        let result = interp.reorder_named_args(&args, &param_names);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_reorder_named_args_unknown_param() {
        let interp = make_interpreter();
        // Create: unknown = 1
        let args = vec![make_expr(ExprKind::Assign {
            target: Box::new(make_expr(ExprKind::Identifier("unknown".to_string()))),
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        })];
        let param_names = vec!["a".to_string(), "b".to_string()];

        let result = interp.reorder_named_args(&args, &param_names);
        // Unknown param gets placed in first available slot
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_reorder_named_args_assign_to_non_identifier() {
        let interp = make_interpreter();
        // Create: 1[0] = 2 (assignment to index, not named arg)
        let args = vec![make_expr(ExprKind::Assign {
            target: Box::new(make_expr(ExprKind::IndexAccess {
                object: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
                index: Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
            })),
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
        })];
        let param_names = vec!["a".to_string()];

        let result = interp.reorder_named_args(&args, &param_names);
        assert_eq!(result.len(), 1);
    }

    // eval_function_call tests
    #[test]
    fn test_eval_function_call_box_new() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("Box".to_string()))),
            field: "new".to_string(),
        });
        let args = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];

        let result = interp.eval_function_call(&func, &args).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_function_call_box_new_wrong_args() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("Box".to_string()))),
            field: "new".to_string(),
        });
        let args = vec![]; // No args - error

        let result = interp.eval_function_call(&func, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1"));
    }

    #[test]
    fn test_eval_function_call_vec_new() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("Vec".to_string()))),
            field: "new".to_string(),
        });
        let args = vec![];

        let result = interp.eval_function_call(&func, &args).unwrap();
        if let Value::Array(arr) = result {
            assert!(arr.is_empty());
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_function_call_vec_new_wrong_args() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("Vec".to_string()))),
            field: "new".to_string(),
        });
        let args = vec![make_expr(ExprKind::Literal(Literal::Integer(1, None)))];

        let result = interp.eval_function_call(&func, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("takes no arguments"));
    }

    #[test]
    fn test_eval_function_call_json_parse() {
        let interp = make_interpreter();
        // Test via the interpreter's json_parse directly (since static method goes through to_string)
        let result = interp.json_parse(r#"{"a": 1}"#).unwrap();
        // Should return an object
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_eval_function_call_json_static_parse_wrong_args() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("JSON".to_string()))),
            field: "parse".to_string(),
        });
        // 2 args - error
        let args = vec![
            make_expr(ExprKind::Literal(Literal::String("{}".to_string()))),
            make_expr(ExprKind::Literal(Literal::String("{}".to_string()))),
        ];

        let result = interp.eval_function_call(&func, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1"));
    }

    #[test]
    fn test_eval_function_call_json_parse_wrong_args() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("JSON".to_string()))),
            field: "parse".to_string(),
        });
        let args = vec![];

        let result = interp.eval_function_call(&func, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1"));
    }

    #[test]
    fn test_eval_function_call_json_stringify() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("JSON".to_string()))),
            field: "stringify".to_string(),
        });
        let args = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];

        let result = interp.eval_function_call(&func, &args).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "42");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_eval_function_call_json_stringify_wrong_args() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("JSON".to_string()))),
            field: "stringify".to_string(),
        });
        let args = vec![];

        let result = interp.eval_function_call(&func, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1"));
    }

    #[test]
    fn test_eval_function_call_file_open_wrong_args() {
        let mut interp = make_interpreter();
        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("File".to_string()))),
            field: "open".to_string(),
        });
        let args = vec![];

        let result = interp.eval_function_call(&func, &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1"));
    }

    #[test]
    fn test_eval_function_call_message_constructor() {
        let mut interp = make_interpreter();
        // Call undefined function - becomes message constructor
        let func = make_expr(ExprKind::Identifier("CustomMessage".to_string()));
        let args = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];

        let result = interp.eval_function_call(&func, &args).unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Message".to_string()))
            );
            assert_eq!(
                obj.get("type"),
                Some(&Value::from_string("CustomMessage".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_function_call_closure() {
        let mut interp = make_interpreter();
        // Define a function first
        let params = vec![make_param("x")];
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        interp.eval_function("identity", &params, &body).unwrap();

        // Call it
        let func = make_expr(ExprKind::Identifier("identity".to_string()));
        let args = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];

        let result = interp.eval_function_call(&func, &args).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_function_call_enum_variant() {
        let mut interp = make_interpreter();
        // Set up an enum variant value
        interp.set_variable(
            "Result_Error",
            Value::EnumVariant {
                enum_name: "Result".to_string(),
                variant_name: "Error".to_string(),
                data: None,
            },
        );

        let func = make_expr(ExprKind::Identifier("Result_Error".to_string()));
        let args = vec![make_expr(ExprKind::Literal(Literal::String(
            "error message".to_string(),
        )))];

        let result = interp.eval_function_call(&func, &args).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Result");
            assert_eq!(variant_name, "Error");
            assert!(data.is_some());
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_eval_function_call_impl_method() {
        let mut interp = make_interpreter();
        // Set up a struct impl method
        let env = Rc::new(RefCell::new(HashMap::new()));
        interp.set_variable(
            "Point::origin",
            Value::Closure {
                params: vec![],
                body: Arc::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
                env,
            },
        );

        let func = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("Point".to_string()))),
            field: "origin".to_string(),
        });
        let args = vec![];

        let result = interp.eval_function_call(&func, &args).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_eval_function_call_with_named_args() {
        let mut interp = make_interpreter();
        // Define a function with params a, b
        let params = vec![make_param("a"), make_param("b")];
        let body = make_expr(ExprKind::Identifier("a".to_string()));
        interp.eval_function("test_func", &params, &body).unwrap();

        // Call with named args in reverse order: b = 2, a = 1
        let func = make_expr(ExprKind::Identifier("test_func".to_string()));
        let args = vec![
            make_expr(ExprKind::Assign {
                target: Box::new(make_expr(ExprKind::Identifier("b".to_string()))),
                value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            }),
            make_expr(ExprKind::Assign {
                target: Box::new(make_expr(ExprKind::Identifier("a".to_string()))),
                value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            }),
        ];

        let result = interp.eval_function_call(&func, &args).unwrap();
        // Should return 'a' which is 1 (reordered correctly)
        assert_eq!(result, Value::Integer(1));
    }
}
