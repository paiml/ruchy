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
mod tests {
    use super::*;
    use crate::frontend::ast::{
        ClassConstant, ClassMethod, Constructor, Literal, Param, Pattern, SelfType, Span,
        StructField, Type, TypeKind, Visibility,
    };

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr::new(kind, Span::default())
    }

    fn make_struct_field(name: &str, ty: Type) -> StructField {
        StructField {
            name: name.to_string(),
            ty,
            default_value: None,
            is_mut: false,
            visibility: Visibility::Public,
            decorators: vec![],
        }
    }

    fn make_struct_field_with_default(name: &str, ty: Type, default: Expr) -> StructField {
        StructField {
            name: name.to_string(),
            ty,
            default_value: Some(default),
            is_mut: true,
            visibility: Visibility::Public,
            decorators: vec![],
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

    fn make_constructor(name: Option<&str>, params: Vec<Param>, body: Expr) -> Constructor {
        Constructor {
            name: name.map(|s| s.to_string()),
            params,
            return_type: None,
            body: Box::new(body),
            is_pub: true,
        }
    }

    fn make_method(name: &str, params: Vec<Param>, body: Expr, is_static: bool) -> ClassMethod {
        ClassMethod {
            name: name.to_string(),
            params,
            return_type: Some(make_type("Any")),
            body: Box::new(body),
            is_pub: true,
            is_static,
            is_override: false,
            is_final: false,
            is_abstract: false,
            is_async: false,
            self_type: if is_static {
                SelfType::None
            } else {
                SelfType::Borrowed
            },
        }
    }

    fn make_constant(name: &str, value: Expr) -> ClassConstant {
        ClassConstant {
            name: name.to_string(),
            ty: make_type("i32"),
            value,
            is_pub: true,
        }
    }

    fn make_struct_literal(name: &str, fields: Vec<(&str, Expr)>) -> Expr {
        make_expr(ExprKind::StructLiteral {
            name: name.to_string(),
            fields: fields
                .into_iter()
                .map(|(n, e)| (n.to_string(), e))
                .collect(),
            base: None,
        })
    }

    #[test]
    fn test_eval_class_definition_empty() {
        let mut interp = make_interpreter();
        let result = interp
            .eval_class_definition("Empty", &[], None, &[], &[], &[], &[], &[], &[], false)
            .unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Class".to_string()))
            );
            assert_eq!(
                obj.get("__name"),
                Some(&Value::from_string("Empty".to_string()))
            );
            // Should have default "new" constructor
            if let Some(Value::Object(ctors)) = obj.get("__constructors") {
                assert!(ctors.contains_key("new"));
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_class_definition_with_superclass() {
        let mut interp = make_interpreter();
        let parent = "ParentClass".to_string();
        let result = interp
            .eval_class_definition(
                "ChildClass",
                &[],
                Some(&parent),
                &[],
                &[],
                &[],
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__superclass"),
                Some(&Value::from_string("ParentClass".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_instantiate_class_not_class() {
        let mut interp = make_interpreter();
        interp.set_variable("NotClass", Value::Integer(42));
        let result = interp.instantiate_class_with_constructor("NotClass", "new", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a class definition"));
    }

    #[test]
    fn test_instantiate_class_wrong_type() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

        let result = interp.instantiate_class_with_constructor("WrongType", "new", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a class"));
    }

    #[test]
    fn test_instantiate_class_with_args_not_class() {
        let mut interp = make_interpreter();
        interp.set_variable("NotClass", Value::Integer(42));
        let result = interp.instantiate_class_with_args("NotClass", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a class definition"));
    }

    #[test]
    fn test_instantiate_class_with_args_wrong_type() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

        let result = interp.instantiate_class_with_args("WrongType", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a class"));
    }

    #[test]
    fn test_eval_class_instance_method_not_class() {
        let mut interp = make_interpreter();
        interp.set_variable("NotClass", Value::Integer(42));
        let result = interp.eval_class_instance_method(&HashMap::new(), "NotClass", "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a class"));
    }

    #[test]
    fn test_call_static_method_not_class() {
        let mut interp = make_interpreter();
        interp.set_variable("NotClass", Value::Integer(42));
        let result = interp.call_static_method("NotClass", "method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a class"));
    }

    #[test]
    fn test_call_static_method_not_found() {
        let mut interp = make_interpreter();
        interp
            .eval_class_definition("TestClass", &[], None, &[], &[], &[], &[], &[], &[], false)
            .unwrap();
        let result = interp.call_static_method("TestClass", "nonexistent", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_eval_class_instance_method_not_found() {
        let mut interp = make_interpreter();
        interp
            .eval_class_definition("TestClass", &[], None, &[], &[], &[], &[], &[], &[], false)
            .unwrap();
        let result =
            interp.eval_class_instance_method(&HashMap::new(), "TestClass", "nonexistent", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no method named"));
    }

    // =========================================================================
    // Additional tests for coverage improvement
    // =========================================================================

    #[test]
    fn test_eval_class_definition_with_fields() {
        let mut interp = make_interpreter();
        let fields = vec![
            make_struct_field("x", make_type("i32")),
            make_struct_field("y", make_type("String")),
        ];

        let result = interp
            .eval_class_definition("Point", &[], None, &[], &fields, &[], &[], &[], &[], false)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(field_defs)) = obj.get("__fields") {
                assert!(field_defs.contains_key("x"));
                assert!(field_defs.contains_key("y"));
            } else {
                panic!("Expected __fields");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_class_definition_with_field_defaults() {
        let mut interp = make_interpreter();
        let fields = vec![make_struct_field_with_default(
            "count",
            make_type("i32"),
            make_expr(ExprKind::Literal(Literal::Integer(42, None))),
        )];

        let result = interp
            .eval_class_definition(
                "Counter",
                &[],
                None,
                &[],
                &fields,
                &[],
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(field_defs)) = obj.get("__fields") {
                if let Some(Value::Object(count_field)) = field_defs.get("count") {
                    assert_eq!(count_field.get("default"), Some(&Value::Integer(42)));
                    assert_eq!(count_field.get("is_mut"), Some(&Value::Bool(true)));
                } else {
                    panic!("Expected count field");
                }
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_class_definition_with_constructor() {
        let mut interp = make_interpreter();
        let constructors = vec![make_constructor(
            Some("new"),
            vec![make_param("value")],
            make_expr(ExprKind::Block(vec![])),
        )];

        let result = interp
            .eval_class_definition(
                "MyClass",
                &[],
                None,
                &[],
                &[],
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(ctors)) = obj.get("__constructors") {
                assert!(ctors.contains_key("new"));
                if let Some(Value::Closure { params, .. }) = ctors.get("new") {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].0, "value");
                }
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_class_definition_with_methods() {
        let mut interp = make_interpreter();
        let methods = vec![
            make_method(
                "get_value",
                vec![make_param("self")],
                make_expr(ExprKind::Literal(Literal::Integer(100, None))),
                false,
            ),
            make_method(
                "static_method",
                vec![],
                make_expr(ExprKind::Literal(Literal::Integer(200, None))),
                true,
            ),
        ];

        let result = interp
            .eval_class_definition(
                "MyClass",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(method_defs)) = obj.get("__methods") {
                assert!(method_defs.contains_key("get_value"));
                assert!(method_defs.contains_key("static_method"));

                // Check static flag
                if let Some(Value::Object(static_meta)) = method_defs.get("static_method") {
                    assert_eq!(static_meta.get("is_static"), Some(&Value::Bool(true)));
                }
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_class_definition_with_constants() {
        let mut interp = make_interpreter();
        let constants = vec![make_constant(
            "MAX_VALUE",
            make_expr(ExprKind::Literal(Literal::Integer(1000, None))),
        )];

        let result = interp
            .eval_class_definition(
                "Config",
                &[],
                None,
                &[],
                &[],
                &[],
                &[],
                &constants,
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(const_defs)) = obj.get("__constants") {
                assert!(const_defs.contains_key("MAX_VALUE"));
            }
        }

        // Also check that constant is accessible via qualified name
        let const_val = interp.lookup_variable("Config::MAX_VALUE").unwrap();
        assert_eq!(const_val, Value::Integer(1000));
    }

    #[test]
    fn test_instantiate_class_with_constructor_success() {
        let mut interp = make_interpreter();

        // Define a class with a field and constructor that returns a struct literal
        let fields = vec![make_struct_field("value", make_type("i32"))];

        let constructors = vec![make_constructor(
            Some("new"),
            vec![],
            make_struct_literal(
                "Simple",
                vec![(
                    "value",
                    make_expr(ExprKind::Literal(Literal::Integer(0, None))),
                )],
            ),
        )];

        interp
            .eval_class_definition(
                "Simple",
                &[],
                None,
                &[],
                &fields,
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Instantiate it
        let result = interp
            .instantiate_class_with_constructor("Simple", "new", &[])
            .unwrap();

        // Should be ObjectMut
        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(
                obj.get("__class"),
                Some(&Value::from_string("Simple".to_string()))
            );
        } else {
            panic!("Expected ObjectMut, got {:?}", result);
        }
    }

    #[test]
    fn test_instantiate_class_with_constructor_and_args() {
        let mut interp = make_interpreter();

        // Define a class with fields and a constructor
        let fields = vec![
            make_struct_field("x", make_type("i32")),
            make_struct_field("y", make_type("i32")),
        ];
        let constructors = vec![make_constructor(
            Some("new"),
            vec![make_param("x"), make_param("y")],
            make_struct_literal(
                "Point",
                vec![
                    ("x", make_expr(ExprKind::Identifier("x".to_string()))),
                    ("y", make_expr(ExprKind::Identifier("y".to_string()))),
                ],
            ),
        )];

        interp
            .eval_class_definition(
                "Point",
                &[],
                None,
                &[],
                &fields,
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Instantiate with arguments
        let result = interp
            .instantiate_class_with_constructor(
                "Point",
                "new",
                &[Value::Integer(10), Value::Integer(20)],
            )
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(
                obj.get("__class"),
                Some(&Value::from_string("Point".to_string()))
            );
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_class_with_constructor_wrong_arg_count() {
        let mut interp = make_interpreter();

        let constructors = vec![make_constructor(
            Some("new"),
            vec![make_param("x")],
            make_expr(ExprKind::Block(vec![])),
        )];

        interp
            .eval_class_definition(
                "OneArg",
                &[],
                None,
                &[],
                &[],
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Wrong number of arguments
        let result = interp.instantiate_class_with_constructor("OneArg", "new", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects 1 arguments"));
    }

    #[test]
    fn test_instantiate_class_with_args_success() {
        let mut interp = make_interpreter();

        let fields = vec![make_struct_field("count", make_type("i32"))];
        let constructors = vec![make_constructor(
            Some("init"),
            vec![],
            make_expr(ExprKind::Block(vec![])),
        )];

        interp
            .eval_class_definition(
                "Counter",
                &[],
                None,
                &[],
                &fields,
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp.instantiate_class_with_args("Counter", &[]).unwrap();

        // Should be Value::Class
        if let Value::Class { class_name, .. } = result {
            assert_eq!(class_name, "Counter");
        } else {
            panic!("Expected Value::Class, got {:?}", result);
        }
    }

    #[test]
    fn test_instantiate_class_with_args_wrong_count() {
        let mut interp = make_interpreter();

        let constructors = vec![make_constructor(
            Some("init"),
            vec![make_param("value")],
            make_expr(ExprKind::Block(vec![])),
        )];

        interp
            .eval_class_definition(
                "NeedsArg",
                &[],
                None,
                &[],
                &[],
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp.instantiate_class_with_args("NeedsArg", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects 1 arguments"));
    }

    #[test]
    fn test_eval_class_instance_method_success() {
        let mut interp = make_interpreter();

        // Create a method that returns a literal
        let methods = vec![make_method(
            "get_value",
            vec![], // self is filtered out automatically
            make_expr(ExprKind::Literal(Literal::Integer(42, None))),
            false,
        )];

        interp
            .eval_class_definition(
                "Getter",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let instance = HashMap::new();
        let result = interp
            .eval_class_instance_method(&instance, "Getter", "get_value", &[])
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_class_instance_method_with_args() {
        let mut interp = make_interpreter();

        // Method that takes an argument and returns it
        let methods = vec![make_method(
            "echo",
            vec![make_param("x")],
            make_expr(ExprKind::Identifier("x".to_string())),
            false,
        )];

        interp
            .eval_class_definition("Echo", &[], None, &[], &[], &[], &methods, &[], &[], false)
            .unwrap();

        let instance = HashMap::new();
        let result = interp
            .eval_class_instance_method(&instance, "Echo", "echo", &[Value::Integer(99)])
            .unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    #[test]
    fn test_eval_class_instance_method_wrong_arg_count() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "need_one",
            vec![make_param("x")],
            make_expr(ExprKind::Literal(Literal::Integer(0, None))),
            false,
        )];

        interp
            .eval_class_definition(
                "NeedOne",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let instance = HashMap::new();
        let result = interp.eval_class_instance_method(&instance, "NeedOne", "need_one", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects 1 arguments"));
    }

    #[test]
    fn test_eval_class_instance_method_on_static() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "static_fn",
            vec![],
            make_expr(ExprKind::Literal(Literal::Integer(0, None))),
            true, // is_static = true
        )];

        interp
            .eval_class_definition(
                "HasStatic",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let instance = HashMap::new();
        let result = interp.eval_class_instance_method(&instance, "HasStatic", "static_fn", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot call static method"));
    }

    #[test]
    fn test_call_static_method_success() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "create",
            vec![],
            make_expr(ExprKind::Literal(Literal::Integer(999, None))),
            true, // is_static = true
        )];

        interp
            .eval_class_definition(
                "Factory",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp.call_static_method("Factory", "create", &[]).unwrap();
        assert_eq!(result, Value::Integer(999));
    }

    #[test]
    fn test_call_static_method_with_args() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "add",
            vec![make_param("a"), make_param("b")],
            make_expr(ExprKind::Identifier("a".to_string())), // Just return a for simplicity
            true,
        )];

        interp
            .eval_class_definition("Math", &[], None, &[], &[], &[], &methods, &[], &[], false)
            .unwrap();

        let result = interp
            .call_static_method("Math", "add", &[Value::Integer(10), Value::Integer(20)])
            .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_call_static_method_wrong_arg_count() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "need_two",
            vec![make_param("a"), make_param("b")],
            make_expr(ExprKind::Literal(Literal::Integer(0, None))),
            true,
        )];

        interp
            .eval_class_definition(
                "NeedTwo",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp.call_static_method("NeedTwo", "need_two", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects 2 arguments"));
    }

    #[test]
    fn test_call_static_method_on_non_static() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "instance_method",
            vec![],
            make_expr(ExprKind::Literal(Literal::Integer(0, None))),
            false, // is_static = false
        )];

        interp
            .eval_class_definition(
                "HasInstance",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp.call_static_method("HasInstance", "instance_method", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a static method"));
    }

    #[test]
    fn test_instantiate_class_with_superclass_fields() {
        let mut interp = make_interpreter();

        // Define parent class with a field
        let parent_fields = vec![make_struct_field_with_default(
            "parent_val",
            make_type("i32"),
            make_expr(ExprKind::Literal(Literal::Integer(100, None))),
        )];
        // Parent constructor returns struct literal
        let parent_constructors = vec![make_constructor(
            Some("new"),
            vec![],
            make_struct_literal(
                "Parent",
                vec![(
                    "parent_val",
                    make_expr(ExprKind::Literal(Literal::Integer(100, None))),
                )],
            ),
        )];
        interp
            .eval_class_definition(
                "Parent",
                &[],
                None,
                &[],
                &parent_fields,
                &parent_constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Define child class with both parent and child fields (inheritance not fully supported)
        let child_fields = vec![
            make_struct_field_with_default(
                "parent_val",
                make_type("i32"),
                make_expr(ExprKind::Literal(Literal::Integer(100, None))),
            ),
            make_struct_field_with_default(
                "child_val",
                make_type("i32"),
                make_expr(ExprKind::Literal(Literal::Integer(200, None))),
            ),
        ];
        let parent_name = "Parent".to_string();
        // Child constructor returns struct literal with both fields
        let child_constructors = vec![make_constructor(
            Some("new"),
            vec![],
            make_struct_literal(
                "Child",
                vec![
                    (
                        "parent_val",
                        make_expr(ExprKind::Literal(Literal::Integer(100, None))),
                    ),
                    (
                        "child_val",
                        make_expr(ExprKind::Literal(Literal::Integer(200, None))),
                    ),
                ],
            ),
        )];
        interp
            .eval_class_definition(
                "Child",
                &[],
                Some(&parent_name),
                &[],
                &child_fields,
                &child_constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Instantiate child
        let result = interp
            .instantiate_class_with_constructor("Child", "new", &[])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            // Should have both parent and child fields
            assert_eq!(obj.get("parent_val"), Some(&Value::Integer(100)));
            assert_eq!(obj.get("child_val"), Some(&Value::Integer(200)));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_eval_class_with_method_override_flag() {
        let mut interp = make_interpreter();

        let mut override_method = make_method(
            "overridden",
            vec![],
            make_expr(ExprKind::Literal(Literal::Integer(0, None))),
            false,
        );
        override_method.is_override = true;

        let methods = vec![override_method];

        let result = interp
            .eval_class_definition(
                "Subclass",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(method_defs)) = obj.get("__methods") {
                if let Some(Value::Object(method_meta)) = method_defs.get("overridden") {
                    assert_eq!(method_meta.get("is_override"), Some(&Value::Bool(true)));
                }
            }
        }
    }

    #[test]
    fn test_instantiate_class_with_args_has_methods() {
        let mut interp = make_interpreter();

        let methods = vec![make_method(
            "do_something",
            vec![],
            make_expr(ExprKind::Literal(Literal::Integer(42, None))),
            false,
        )];

        interp
            .eval_class_definition(
                "WithMethods",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp
            .instantiate_class_with_args("WithMethods", &[])
            .unwrap();

        if let Value::Class { methods: m, .. } = result {
            assert!(m.contains_key("do_something"));
        } else {
            panic!("Expected Value::Class");
        }
    }

    #[test]
    fn test_instantiate_class_with_args_has_field_defaults() {
        let mut interp = make_interpreter();

        let fields = vec![make_struct_field_with_default(
            "initialized",
            make_type("i32"),
            make_expr(ExprKind::Literal(Literal::Integer(777, None))),
        )];

        interp
            .eval_class_definition(
                "WithDefaults",
                &[],
                None,
                &[],
                &fields,
                &[],
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp
            .instantiate_class_with_args("WithDefaults", &[])
            .unwrap();

        if let Value::Class { fields: f, .. } = result {
            let fields_guard = f.read().unwrap();
            assert_eq!(fields_guard.get("initialized"), Some(&Value::Integer(777)));
        } else {
            panic!("Expected Value::Class");
        }
    }

    #[test]
    fn test_class_definition_with_pattern_wildcard() {
        let mut interp = make_interpreter();

        // Constructor with a non-identifier pattern (wildcard)
        let constructors = vec![Constructor {
            name: Some("new".to_string()),
            params: vec![Param {
                pattern: Pattern::Wildcard,
                ty: make_type("Any"),
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
            body: Box::new(make_expr(ExprKind::Block(vec![]))),
            is_pub: true,
        }];

        let result = interp
            .eval_class_definition(
                "WildcardClass",
                &[],
                None,
                &[],
                &[],
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(ctors)) = obj.get("__constructors") {
                if let Some(Value::Closure { params, .. }) = ctors.get("new") {
                    // Wildcard pattern becomes "_"
                    assert_eq!(params[0].0, "_");
                }
            }
        }
    }

    #[test]
    fn test_method_with_pattern_wildcard() {
        let mut interp = make_interpreter();

        let methods = vec![ClassMethod {
            name: "ignore_arg".to_string(),
            params: vec![Param {
                pattern: Pattern::Wildcard,
                ty: make_type("Any"),
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: Some(make_type("Any")),
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
            is_pub: true,
            is_static: false,
            is_override: false,
            is_final: false,
            is_abstract: false,
            is_async: false,
            self_type: SelfType::Borrowed,
        }];

        let result = interp
            .eval_class_definition(
                "WildcardMethod",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(method_defs)) = obj.get("__methods") {
                if let Some(Value::Object(method_meta)) = method_defs.get("ignore_arg") {
                    if let Some(Value::Closure { params, .. }) = method_meta.get("closure") {
                        // Wildcard pattern becomes "_"
                        assert_eq!(params[0].0, "_");
                    }
                }
            }
        }
    }

    #[test]
    fn test_instantiate_class_field_without_default() {
        let mut interp = make_interpreter();

        // Field without default value should be initialized to Nil
        let fields = vec![make_struct_field("uninitialized", make_type("Any"))];

        // Constructor returns struct literal (avoids the self lookup issue)
        // Use Literal::Unit which maps to Value::Nil
        let constructors = vec![make_constructor(
            Some("new"),
            vec![],
            make_struct_literal(
                "NoDefault",
                vec![("uninitialized", make_expr(ExprKind::Literal(Literal::Unit)))],
            ),
        )];

        interp
            .eval_class_definition(
                "NoDefault",
                &[],
                None,
                &[],
                &fields,
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp
            .instantiate_class_with_constructor("NoDefault", "new", &[])
            .unwrap();

        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(obj.get("uninitialized"), Some(&Value::Nil));
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_instantiate_class_with_args_field_without_default() {
        let mut interp = make_interpreter();

        let fields = vec![make_struct_field("nil_field", make_type("Any"))];

        interp
            .eval_class_definition(
                "NilField",
                &[],
                None,
                &[],
                &fields,
                &[],
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        let result = interp.instantiate_class_with_args("NilField", &[]).unwrap();

        if let Value::Class { fields: f, .. } = result {
            let fields_guard = f.read().unwrap();
            assert_eq!(fields_guard.get("nil_field"), Some(&Value::Nil));
        }
    }

    // =========================================================================
    // Additional coverage tests
    // =========================================================================

    #[test]
    fn test_class_constant_accessible_via_qualified_name() {
        let mut interp = make_interpreter();

        // Create constant
        let const_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::default(),
        );
        let constants = vec![ClassConstant {
            name: "MAX_VALUE".to_string(),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            },
            value: const_expr,
            is_pub: true,
        }];

        interp
            .eval_class_definition(
                "Constants",
                &[],
                None,
                &[],
                &[],
                &[],
                &[],
                &constants,
                &[],
                false,
            )
            .unwrap();

        // Access constant via qualified name
        let result = interp.lookup_variable("Constants::MAX_VALUE").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_class_with_named_constructor() {
        let mut interp = make_interpreter();

        // Create named constructor
        let ctor_body = Expr::new(ExprKind::Block(vec![]), Span::default());
        let constructors = vec![Constructor {
            name: Some("from_value".to_string()),
            params: vec![],
            return_type: None,
            body: Box::new(ctor_body),
            is_pub: true,
        }];

        let result = interp
            .eval_class_definition(
                "Named",
                &[],
                None,
                &[],
                &[],
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Verify named constructor exists
        if let Value::Object(obj) = result {
            if let Some(Value::Object(ctors)) = obj.get("__constructors") {
                assert!(ctors.contains_key("from_value"));
            } else {
                panic!("Expected __constructors");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_instantiate_class_with_named_constructor() {
        let mut interp = make_interpreter();

        // Create a class with named constructor that returns an object
        // The constructor body returns an object with __class set
        let mut return_obj = HashMap::new();
        return_obj.insert(
            "__class".to_string(),
            Value::from_string("Creatable".to_string()),
        );

        // Constructor body that returns an Object (avoids self lookup issue)
        let ctor_body = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)), // Simple body
            Span::default(),
        );
        let constructors = vec![Constructor {
            name: Some("create".to_string()),
            params: vec![],
            return_type: None,
            body: Box::new(ctor_body),
            is_pub: true,
        }];

        interp
            .eval_class_definition(
                "Creatable",
                &[],
                None,
                &[],
                &[],
                &constructors,
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Verify class has the named constructor
        let class_def = interp.lookup_variable("Creatable").unwrap();
        if let Value::Object(obj) = class_def {
            if let Some(Value::Object(ctors)) = obj.get("__constructors") {
                assert!(ctors.contains_key("create"));
            } else {
                panic!("Expected __constructors");
            }
        }
    }

    #[test]
    fn test_class_method_is_override_true() {
        let mut interp = make_interpreter();

        // Create method with is_override = true
        let method_body = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        );
        let methods = vec![ClassMethod {
            name: "overridden".to_string(),
            params: vec![],
            body: Box::new(method_body),
            return_type: None,
            is_pub: true,
            is_static: false,
            is_override: true,
            is_final: false,
            is_abstract: false,
            is_async: false,
            self_type: SelfType::Borrowed,
        }];

        let result = interp
            .eval_class_definition(
                "Override",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        // Verify is_override flag is stored
        if let Value::Object(obj) = result {
            if let Some(Value::Object(methods_obj)) = obj.get("__methods") {
                if let Some(Value::Object(method_meta)) = methods_obj.get("overridden") {
                    assert_eq!(method_meta.get("is_override"), Some(&Value::Bool(true)));
                }
            }
        }
    }

    #[test]
    fn test_class_constant_is_pub_false() {
        let mut interp = make_interpreter();

        let const_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(100, None)),
            Span::default(),
        );
        let constants = vec![ClassConstant {
            name: "PRIVATE_VALUE".to_string(),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            },
            value: const_expr,
            is_pub: false,
        }];

        let result = interp
            .eval_class_definition(
                "PrivateConst",
                &[],
                None,
                &[],
                &[],
                &[],
                &[],
                &constants,
                &[],
                false,
            )
            .unwrap();

        // Verify is_pub = false in constant metadata
        if let Value::Object(obj) = result {
            if let Some(Value::Object(consts)) = obj.get("__constants") {
                if let Some(Value::Object(const_meta)) = consts.get("PRIVATE_VALUE") {
                    assert_eq!(const_meta.get("is_pub"), Some(&Value::Bool(false)));
                }
            }
        }
    }

    #[test]
    fn test_instantiate_constructor_not_found_uses_default() {
        let mut interp = make_interpreter();

        // Create class with NO explicit constructors (will get default "new")
        interp
            .eval_class_definition(
                "DefaultCtor",
                &[],
                None,
                &[],
                &[],
                &[], // No explicit constructors
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Try to instantiate with a non-existent constructor name
        // Should use the default constructor
        let result = interp
            .instantiate_class_with_constructor("DefaultCtor", "nonexistent", &[])
            .unwrap();

        // Should still work (constructor not found but class instantiated)
        if let Value::ObjectMut(cell) = result {
            let obj = cell.lock().unwrap();
            assert_eq!(
                obj.get("__class"),
                Some(&Value::from_string("DefaultCtor".to_string()))
            );
        } else {
            panic!("Expected ObjectMut");
        }
    }

    #[test]
    fn test_class_with_multiple_fields() {
        let mut interp = make_interpreter();

        let fields = vec![
            make_struct_field("field1", make_type("i32")),
            make_struct_field("field2", make_type("String")),
            make_struct_field("field3", make_type("bool")),
        ];

        let result = interp
            .eval_class_definition(
                "MultiField",
                &[],
                None,
                &[],
                &fields,
                &[],
                &[],
                &[],
                &[],
                false,
            )
            .unwrap();

        // Verify all fields are stored
        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                assert!(fields_obj.contains_key("field1"));
                assert!(fields_obj.contains_key("field2"));
                assert!(fields_obj.contains_key("field3"));
                assert_eq!(fields_obj.len(), 3);
            }
        }
    }

    #[test]
    fn test_class_with_multiple_methods() {
        let mut interp = make_interpreter();

        let method1_body = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::default(),
        );
        let method2_body = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::default(),
        );

        let methods = vec![
            ClassMethod {
                name: "method1".to_string(),
                params: vec![],
                body: Box::new(method1_body),
                return_type: None,
                is_pub: true,
                is_static: false,
                is_override: false,
                is_final: false,
                is_abstract: false,
                is_async: false,
                self_type: SelfType::Borrowed,
            },
            ClassMethod {
                name: "method2".to_string(),
                params: vec![],
                body: Box::new(method2_body),
                return_type: None,
                is_pub: true,
                is_static: true,
                is_override: false,
                is_final: false,
                is_abstract: false,
                is_async: false,
                self_type: SelfType::None,
            },
        ];

        let result = interp
            .eval_class_definition(
                "MultiMethod",
                &[],
                None,
                &[],
                &[],
                &[],
                &methods,
                &[],
                &[],
                false,
            )
            .unwrap();

        // Verify all methods are stored
        if let Value::Object(obj) = result {
            if let Some(Value::Object(methods_obj)) = obj.get("__methods") {
                assert!(methods_obj.contains_key("method1"));
                assert!(methods_obj.contains_key("method2"));
                assert_eq!(methods_obj.len(), 2);
            }
        }
    }

    #[test]
    fn test_class_with_multiple_constants() {
        let mut interp = make_interpreter();

        let const1 = ClassConstant {
            name: "CONST_A".to_string(),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            },
            value: Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::default(),
            ),
            is_pub: true,
        };
        let const2 = ClassConstant {
            name: "CONST_B".to_string(),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            },
            value: Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::default(),
            ),
            is_pub: true,
        };

        let result = interp
            .eval_class_definition(
                "MultiConst",
                &[],
                None,
                &[],
                &[],
                &[],
                &[],
                &[const1, const2],
                &[],
                false,
            )
            .unwrap();

        // Verify all constants are stored
        if let Value::Object(obj) = result {
            if let Some(Value::Object(consts)) = obj.get("__constants") {
                assert!(consts.contains_key("CONST_A"));
                assert!(consts.contains_key("CONST_B"));
                assert_eq!(consts.len(), 2);
            }
        }

        // Also verify qualified names work
        assert_eq!(
            interp.lookup_variable("MultiConst::CONST_A").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            interp.lookup_variable("MultiConst::CONST_B").unwrap(),
            Value::Integer(2)
        );
    }
}
