//! Class definition and instantiation
//!
//! Extracted from interpreter_types_impl.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]

use crate::frontend::ast::{Expr, ExprKind};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

impl Interpreter {
    /// Evaluate class definition
    ///
    /// Supports:
    /// - Class fields with types and defaults
    /// - Multiple constructors (including named constructors)
    /// - Instance methods with self binding
    /// - Static methods (no self binding)
    /// - Inheritance metadata (superclass stored but not fully implemented)
    ///
    /// Complexity: 8
    pub(crate) fn eval_class_definition(
        &mut self,
        name: &str,
        _type_params: &[String],
        superclass: Option<&String>,
        _traits: &[String],
        fields: &[crate::frontend::ast::StructField],
        constructors: &[crate::frontend::ast::Constructor],
        methods: &[crate::frontend::ast::ClassMethod],
        constants: &[crate::frontend::ast::ClassConstant],
        _derives: &[String],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::sync::Arc;

        // Create class metadata object
        let mut class_info = HashMap::new();

        // Mark as class type
        class_info.insert(
            "__type".to_string(),
            Value::from_string("Class".to_string()),
        );
        class_info.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store superclass if present
        if let Some(parent) = superclass {
            class_info.insert(
                "__superclass".to_string(),
                Value::from_string(parent.clone()),
            );
        }

        // Store field definitions (similar to struct)
        let mut field_defs = HashMap::new();
        for field in fields {
            let mut field_info = HashMap::new();

            // Store field type
            let type_str = format!("{:?}", field.ty);
            field_info.insert("type".to_string(), Value::from_string(type_str));

            // Store visibility
            field_info.insert(
                "is_pub".to_string(),
                Value::Bool(field.visibility.is_public()),
            );
            field_info.insert("is_mut".to_string(), Value::Bool(field.is_mut));

            // Store default value if present
            if let Some(ref default) = field.default_value {
                // Evaluate default value
                let default_val = self.eval_expr(default)?;
                field_info.insert("default".to_string(), default_val);
            }

            field_defs.insert(
                field.name.clone(),
                Value::Object(std::sync::Arc::new(field_info)),
            );
        }
        class_info.insert("__fields".to_string(), Value::Object(Arc::new(field_defs)));

        // Store constructors as closures
        let mut constructor_info = HashMap::new();
        for constructor in constructors {
            // Store constructor by name (default name is "new")
            let ctor_name = constructor
                .name
                .as_ref()
                .unwrap_or(&"new".to_string())
                .clone();

            // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
            let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = constructor
                .params
                .iter()
                .map(|p| {
                    let name = match &p.pattern {
                        crate::frontend::ast::Pattern::Identifier(n) => n.clone(),
                        _ => "_".to_string(),
                    };
                    let default = p
                        .default_value
                        .clone()
                        .map(|expr| Arc::new((*expr).clone()));
                    (name, default)
                })
                .collect();

            // Create a closure for the constructor
            let ctor_closure = Value::Closure {
                params: params_with_defaults,
                body: Arc::new((*constructor.body).clone()),
                env: Rc::new(RefCell::new(HashMap::new())), // ISSUE-119: Empty env for now
            };

            constructor_info.insert(ctor_name, ctor_closure);
        }

        // If no constructors defined, create a default "new" constructor
        if constructor_info.is_empty() {
            // Create a default constructor that initializes fields with defaults
            let default_body = Expr::new(
                ExprKind::Block(Vec::new()), // Empty block - fields get initialized with defaults
                crate::frontend::ast::Span::new(0, 0),
            );

            let default_constructor = Value::Closure {
                params: Vec::new(), // No parameters
                body: Arc::new(default_body),
                env: Rc::new(RefCell::new(HashMap::new())), // ISSUE-119: Empty environment
            };

            constructor_info.insert("new".to_string(), default_constructor);
        }

        class_info.insert(
            "__constructors".to_string(),
            Value::Object(Arc::new(constructor_info)),
        );

        // Store methods as closures with metadata
        let mut method_info = HashMap::new();
        for method in methods {
            // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values (excluding 'self')
            let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = method
                .params
                .iter()
                .filter_map(|p| match &p.pattern {
                    crate::frontend::ast::Pattern::Identifier(name) if name != "self" => {
                        let default = p
                            .default_value
                            .clone()
                            .map(|expr| Arc::new((*expr).clone()));
                        Some((name.clone(), default))
                    }
                    crate::frontend::ast::Pattern::Identifier(_) => None, // Skip 'self'
                    _ => {
                        let default = p
                            .default_value
                            .clone()
                            .map(|expr| Arc::new((*expr).clone()));
                        Some(("_".to_string(), default))
                    }
                })
                .collect();

            // Create a closure for the method
            let method_closure = Value::Closure {
                params: params_with_defaults,
                body: Arc::new((*method.body).clone()),
                env: Rc::new(RefCell::new(HashMap::new())), // ISSUE-119: Empty environment
            };

            // Store method with metadata
            let mut method_meta = HashMap::new();
            method_meta.insert("closure".to_string(), method_closure);
            method_meta.insert("is_static".to_string(), Value::Bool(method.is_static));
            method_meta.insert("is_override".to_string(), Value::Bool(method.is_override));

            method_info.insert(method.name.clone(), Value::Object(Arc::new(method_meta)));
        }
        class_info.insert(
            "__methods".to_string(),
            Value::Object(Arc::new(method_info)),
        );

        // Store class constants
        let mut constants_info = HashMap::new();
        for constant in constants {
            // Evaluate the constant value
            let const_value = self.eval_expr(&constant.value)?;

            // Store constant with metadata
            let mut const_meta = HashMap::new();
            const_meta.insert("value".to_string(), const_value.clone());
            const_meta.insert(
                "type".to_string(),
                Value::from_string(format!("{:?}", constant.ty)),
            );
            const_meta.insert("is_pub".to_string(), Value::Bool(constant.is_pub));

            constants_info.insert(constant.name.clone(), Value::Object(Arc::new(const_meta)));

            // Also store the constant directly on the class for easy access
            // e.g., MyClass::CONSTANT_NAME
            let qualified_name = format!("{}::{}", name, constant.name);
            self.set_variable(&qualified_name, const_value);
        }
        class_info.insert(
            "__constants".to_string(),
            Value::Object(Arc::new(constants_info)),
        );

        // Store the class definition in the environment
        let class_value = Value::Object(Arc::new(class_info));
        self.set_variable(name, class_value.clone());

        Ok(class_value)
    }

    /// Instantiates a class by calling its constructor.
    pub(crate) fn instantiate_class_with_constructor(
        &mut self,
        class_name: &str,
        constructor_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Verify this is a class
            if let Some(Value::String(ref type_str)) = class_info.get("__type") {
                if type_str.as_ref() != "Class" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not a class",
                        class_name
                    )));
                }
            }

            // Create instance object
            let mut instance = HashMap::new();
            instance.insert(
                "__class".to_string(),
                Value::from_string(class_name.to_string()),
            );

            // Helper function to collect fields from class and its parents
            fn collect_all_fields(
                class_info: &HashMap<String, Value>,
                interpreter: &Interpreter,
            ) -> HashMap<String, Value> {
                let mut all_fields = HashMap::new();

                // First, get parent fields if there's a superclass
                if let Some(Value::String(ref parent_name)) = class_info.get("__superclass") {
                    if let Ok(Value::Object(ref parent_info)) =
                        interpreter.lookup_variable(parent_name)
                    {
                        let parent_fields = collect_all_fields(parent_info, interpreter);
                        all_fields.extend(parent_fields);
                    }
                }

                // Then add this class's fields (overriding parent fields if they exist)
                if let Some(Value::Object(ref fields)) = class_info.get("__fields") {
                    for (field_name, field_info) in fields.iter() {
                        if let Value::Object(ref field_meta) = field_info {
                            // Use default value if present
                            if let Some(default) = field_meta.get("default") {
                                all_fields.insert(field_name.clone(), default.clone());
                            } else {
                                // Initialize with nil
                                all_fields.insert(field_name.clone(), Value::Nil);
                            }
                        }
                    }
                }

                all_fields
            }

            // Initialize fields with default values from this class and all parent classes
            let all_fields = collect_all_fields(class_info, self);
            for (field_name, field_value) in all_fields {
                instance.insert(field_name, field_value);
            }

            // Execute the constructor if present
            if let Some(Value::Object(ref constructors)) = class_info.get("__constructors") {
                // Look for the specified constructor
                if let Some(constructor) = constructors.get(constructor_name) {
                    if let Value::Closure {
                        params,
                        body,
                        env: _,
                    } = constructor
                    {
                        // Check argument count
                        if args.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "constructor expects {} arguments, got {}",
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create environment for constructor
                        let mut ctor_env = HashMap::new();

                        // Bind 'self' to mutable instance for constructor
                        ctor_env.insert(
                            "self".to_string(),
                            Value::Object(Arc::new(instance.clone())),
                        );

                        // RUNTIME-DEFAULT-PARAMS: Bind constructor parameters
                        for ((param_name, _default_value), arg) in params.iter().zip(args) {
                            ctor_env.insert(param_name.clone(), arg.clone());
                        }

                        // Push constructor environment
                        self.env_stack.push(Rc::new(RefCell::new(ctor_env))); // ISSUE-119: Wrap in Rc<RefCell>

                        // Execute constructor body
                        // RUNTIME-098: Constructor may return explicit value (e.g., Counter { count: 0 })
                        let result = self.eval_expr(body)?;

                        // Pop environment before checking result
                        self.env_stack.pop();

                        // RUNTIME-098: Check if constructor returned an explicit struct instance
                        // If so, use that instead of manually building from fields
                        if let Value::Object(ref returned_obj) = result {
                            // Check if it's a struct instance with the correct class
                            if let Some(Value::String(ref class)) = returned_obj.get("__class") {
                                if class.as_ref() == class_name {
                                    // Constructor explicitly returned an instance - convert to ObjectMut for mutability
                                    let obj_map = returned_obj.as_ref().clone();
                                    return Ok(crate::runtime::object_helpers::new_mutable_object(
                                        obj_map,
                                    ));
                                }
                            }
                            // Also handle struct literals (without __class)
                            if !returned_obj.contains_key("__class") {
                                // This is a plain struct literal, add class metadata
                                let mut obj_with_class = returned_obj.as_ref().clone();
                                obj_with_class.insert(
                                    "__class".to_string(),
                                    Value::from_string(class_name.to_string()),
                                );
                                return Ok(crate::runtime::object_helpers::new_mutable_object(
                                    obj_with_class,
                                ));
                            }
                        }

                        // RUNTIME-098: For field-assignment constructors (self.x = value),
                        // extract updated self from environment after constructor execution
                        let updated_self = self.lookup_variable("self")?;
                        if let Value::Object(ref updated_instance) = updated_self {
                            // Copy all non-metadata fields from updated self back to instance
                            for (key, value) in updated_instance.iter() {
                                if !key.starts_with("__") {
                                    instance.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                }
            }

            // Return ObjectMut for mutable class instances (support &mut self methods)
            Ok(crate::runtime::object_helpers::new_mutable_object(instance))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class definition",
                class_name
            )))
        }
    }

    /// Instantiate a class with arguments (calls init constructor)
    /// Returns `Value::Class` with reference semantics
    pub(crate) fn instantiate_class_with_args(
        &mut self,
        class_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        use std::sync::RwLock;

        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Verify this is a class
            if let Some(Value::String(ref type_str)) = class_info.get("__type") {
                if type_str.as_ref() != "Class" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not a class",
                        class_name
                    )));
                }
            }

            // Collect methods from the class definition
            let mut methods_map = HashMap::new();
            if let Some(Value::Object(ref methods_obj)) = class_info.get("__methods") {
                for (method_name, method_value) in methods_obj.iter() {
                    // Extract the closure from method metadata
                    if let Value::Object(ref method_meta) = method_value {
                        if let Some(closure) = method_meta.get("closure") {
                            methods_map.insert(method_name.clone(), closure.clone());
                        }
                    }
                }
            }

            // Create instance fields with default values
            let mut instance_fields = HashMap::new();
            if let Some(Value::Object(ref fields)) = class_info.get("__fields") {
                for (field_name, field_info) in fields.iter() {
                    if let Value::Object(ref field_meta) = field_info {
                        // Use default value if present
                        if let Some(default) = field_meta.get("default") {
                            instance_fields.insert(field_name.clone(), default.clone());
                        } else {
                            // Initialize with nil
                            instance_fields.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            // Create the Class instance
            let class_instance = Value::Class {
                class_name: class_name.to_string(),
                fields: Arc::new(RwLock::new(instance_fields.clone())),
                methods: Arc::new(methods_map),
            };

            // Execute the init constructor if present
            if let Some(Value::Object(ref constructors)) = class_info.get("__constructors") {
                // Look for "init" or "new" constructor
                let constructor = constructors.get("init").or_else(|| constructors.get("new"));

                if let Some(constructor) = constructor {
                    if let Value::Closure {
                        params,
                        body,
                        env: _,
                    } = constructor
                    {
                        // Check argument count
                        if args.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "constructor expects {} arguments, got {}",
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create environment for constructor
                        let mut ctor_env = HashMap::new();

                        // Bind 'self' to the class instance
                        ctor_env.insert("self".to_string(), class_instance.clone());

                        // RUNTIME-DEFAULT-PARAMS: Bind constructor parameters
                        for ((param_name, _default_value), arg) in params.iter().zip(args) {
                            ctor_env.insert(param_name.clone(), arg.clone());
                        }

                        // Push constructor environment
                        self.env_stack.push(Rc::new(RefCell::new(ctor_env))); // ISSUE-119: Wrap in Rc<RefCell>

                        // Execute constructor body
                        let _result = self.eval_expr(body)?;

                        // Pop environment
                        self.env_stack.pop();
                    }
                }
            }

            Ok(class_instance)
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class definition",
                class_name
            )))
        }
    }

    pub(crate) fn eval_class_instance_method(
        &mut self,
        instance: &HashMap<String, Value>,
        class_name: &str,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Look for the method in the class definition
            if let Some(Value::Object(ref methods)) = class_info.get("__methods") {
                if let Some(Value::Object(ref method_meta)) = methods.get(method) {
                    // Get the method closure
                    if let Some(Value::Closure { params, body, .. }) = method_meta.get("closure") {
                        // Check if it's a static method
                        let is_static = method_meta
                            .get("is_static")
                            .and_then(|v| {
                                if let Value::Bool(b) = v {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(false);

                        if is_static {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Cannot call static method {} on instance",
                                method
                            )));
                        }

                        // Create environment for method execution
                        let mut method_env = HashMap::new();

                        // Add 'self' to the environment
                        method_env.insert(
                            "self".to_string(),
                            Value::Object(Arc::new(instance.clone())),
                        );

                        // Bind method parameters to arguments
                        // Note: We're not including 'self' in params count here
                        if arg_values.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Method {} expects {} arguments, got {}",
                                method,
                                params.len(),
                                arg_values.len()
                            )));
                        }

                        // RUNTIME-DEFAULT-PARAMS: Extract param name from tuple (name, default_value)
                        for ((param_name, _default_value), arg) in params.iter().zip(arg_values) {
                            method_env.insert(param_name.clone(), arg.clone());
                        }

                        // Push method environment
                        self.env_push(method_env);

                        // Execute method body
                        let result = self.eval_expr(body)?;

                        // Pop environment
                        self.env_pop();

                        return Ok(result);
                    }
                }
            }

            // Method not found
            Err(InterpreterError::RuntimeError(format!(
                "Class {} has no method named {}",
                class_name, method
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class",
                class_name
            )))
        }
    }

    pub(crate) fn call_static_method(
        &mut self,
        class_name: &str,
        method_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the class definition
        let class_def = self.lookup_variable(class_name)?;

        if let Value::Object(ref class_info) = class_def {
            // Look for the method in the class definition
            if let Some(Value::Object(ref methods)) = class_info.get("__methods") {
                if let Some(Value::Object(ref method_meta)) = methods.get(method_name) {
                    // Verify it's a static method
                    let is_static = method_meta
                        .get("is_static")
                        .and_then(|v| {
                            if let Value::Bool(b) = v {
                                Some(*b)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(false);

                    if !is_static {
                        return Err(InterpreterError::RuntimeError(format!(
                            "{} is not a static method",
                            method_name
                        )));
                    }

                    // Get the method closure
                    if let Some(Value::Closure { params, body, .. }) = method_meta.get("closure") {
                        // Check parameter count
                        if args.len() != params.len() {
                            return Err(InterpreterError::RuntimeError(format!(
                                "Static method {} expects {} arguments, got {}",
                                method_name,
                                params.len(),
                                args.len()
                            )));
                        }

                        // Create environment for static method execution
                        let mut method_env = HashMap::new();

                        // RUNTIME-DEFAULT-PARAMS: Bind parameters to arguments (no self for static methods)
                        for (i, (param_name, _default_value)) in params.iter().enumerate() {
                            method_env.insert(param_name.clone(), args[i].clone());
                        }

                        // Push the method environment
                        self.env_stack.push(Rc::new(RefCell::new(method_env))); // ISSUE-119: Wrap in Rc<RefCell>

                        // Execute the method body
                        let result = self.eval_expr(body);

                        // Pop the method environment
                        self.env_stack.pop();

                        return result;
                    }
                }
            }

            Err(InterpreterError::RuntimeError(format!(
                "Static method {} not found in class {}",
                method_name, class_name
            )))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a class",
                class_name
            )))
        }
    }
}

#[cfg(test)]
#[path = "interpreter_types_class_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "interpreter_types_class_tests_part2.rs"]
mod tests_part2;
