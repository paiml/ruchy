//! Struct definition and instantiation
//!
//! Extracted from interpreter_types_impl.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]

use crate::frontend::ast::Expr;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

impl Interpreter {
    /// Evaluate struct definition
    /// Creates a struct type descriptor that can be used for instantiation
    /// Complexity: 7
    pub(crate) fn eval_struct_definition(
        &mut self,
        name: &str,
        _type_params: &[String], // Generic type parameters (not yet used in runtime)
        fields: &[crate::frontend::ast::StructField],
        methods: &[crate::frontend::ast::ClassMethod],
        _is_pub: bool,
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;
        use std::sync::Arc;

        // Create a struct type object
        let mut struct_type = HashMap::new();

        // Store struct metadata
        struct_type.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        struct_type.insert("__name".to_string(), Value::from_string(name.to_string()));

        // Store field definitions
        let mut field_defs = HashMap::new();
        for field in fields {
            // Store field type information
            let type_name = match &field.ty.kind {
                crate::frontend::ast::TypeKind::Named(n) => n.clone(),
                crate::frontend::ast::TypeKind::Array { .. } => "Array".to_string(),
                crate::frontend::ast::TypeKind::Optional(_) => "Option".to_string(),
                crate::frontend::ast::TypeKind::List(_) => "List".to_string(),
                crate::frontend::ast::TypeKind::Tuple(_) => "Tuple".to_string(),
                _ => "Any".to_string(),
            };

            let mut field_info = HashMap::new();
            field_info.insert("type".to_string(), Value::from_string(type_name));
            field_info.insert(
                "is_pub".to_string(),
                Value::from_bool(field.visibility.is_public()),
            );
            field_info.insert("is_mut".to_string(), Value::from_bool(field.is_mut));
            // Store visibility for access control
            let visibility_str = match field.visibility {
                crate::frontend::ast::Visibility::Public => "pub",
                crate::frontend::ast::Visibility::PubCrate => "pub(crate)",
                crate::frontend::ast::Visibility::PubSuper => "pub(super)",
                crate::frontend::ast::Visibility::Private => "private",
                crate::frontend::ast::Visibility::Protected => "protected",
            };
            field_info.insert(
                "visibility".to_string(),
                Value::from_string(visibility_str.to_string()),
            );

            // Store default value if present
            if let Some(default_expr) = &field.default_value {
                let default_val = self.eval_expr(default_expr)?;
                field_info.insert("default".to_string(), default_val);
            }

            field_defs.insert(
                field.name.clone(),
                Value::Object(std::sync::Arc::new(field_info)),
            );
        }

        struct_type.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(field_defs)),
        );

        // Store methods as separate variables with qualified names (same as impl blocks)
        // This allows runtime method dispatch via eval_struct_instance_method
        for method in methods {
            // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
            let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = method
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

            // Create a closure for the method
            let method_closure = Value::Closure {
                params: params_with_defaults,
                body: Arc::new((*method.body).clone()),
                env: Rc::new(RefCell::new(HashMap::new())), // Empty environment
            };

            // Store method with qualified name in environment (e.g., "Rectangle::area")
            let qualified_name = format!("{}::{}", name, method.name);
            self.set_variable(&qualified_name, method_closure);
        }

        // Register this struct type in the environment
        let struct_obj = Value::Object(std::sync::Arc::new(struct_type));
        self.set_variable(name, struct_obj.clone());

        Ok(struct_obj)
    }

    /// Evaluate struct literal (instantiation)
    /// Creates an instance of a struct with provided field values
    /// Complexity: 8
    pub(crate) fn eval_struct_literal(
        &mut self,
        name: &str,
        fields: &[(String, crate::frontend::ast::Expr)],
    ) -> Result<Value, InterpreterError> {
        use std::collections::HashMap;

        // Look up the struct type definition
        let struct_type = self.lookup_variable(name).map_err(|_| {
            InterpreterError::RuntimeError(format!("Undefined struct type: {name}"))
        })?;

        // Verify it's actually a struct type
        let struct_type_obj = if let Value::Object(obj) = &struct_type {
            obj
        } else {
            return Err(InterpreterError::RuntimeError(format!(
                "{name} is not a struct type"
            )));
        };

        // Verify it's a struct type (not actor or other type)
        let type_name = struct_type_obj
            .get("__type")
            .and_then(|v| {
                if let Value::String(s) = v {
                    Some(s.as_ref())
                } else {
                    None
                }
            })
            .unwrap_or("");

        // Handle Actor types differently
        if type_name == "Actor" {
            // Convert field expressions to values for actor instantiation
            let mut field_values = Vec::new();
            for (field_name, field_expr) in fields {
                let value = self.eval_expr(field_expr)?;
                field_values.push((field_name.clone(), value));
            }

            // Create an object with the named fields to pass to actor instantiation
            let mut args_obj = HashMap::new();
            for (name, value) in field_values {
                args_obj.insert(name, value);
            }

            // Call the actor instantiation function
            return self.instantiate_actor_with_args(name, &[Value::Object(Arc::new(args_obj))]);
        }

        // Allow both Struct and Class types (both use same instantiation syntax)
        if type_name != "Struct" && type_name != "Class" {
            return Err(InterpreterError::RuntimeError(format!(
                "{name} is not a struct or class type (it's a {type_name})"
            )));
        }

        // Get field definitions
        let field_defs = struct_type_obj
            .get("__fields")
            .and_then(|v| {
                if let Value::Object(obj) = v {
                    Some(obj)
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                InterpreterError::RuntimeError(format!("Invalid struct type definition for {name}"))
            })?;

        // Create struct instance fields (without metadata)
        let mut instance_fields = HashMap::new();

        // Evaluate and set field values
        for (field_name, field_expr) in fields {
            // Verify field exists in struct definition
            if !field_defs.contains_key(field_name) {
                return Err(InterpreterError::RuntimeError(format!(
                    "Struct {name} does not have field '{field_name}'"
                )));
            }

            // Evaluate field value
            let field_value = self.eval_expr(field_expr)?;
            instance_fields.insert(field_name.clone(), field_value);
        }

        // Check that all required fields are provided or have defaults
        for (field_name, field_def_value) in field_defs.iter() {
            if !instance_fields.contains_key(field_name) {
                // Check if this field has a default value
                if let Value::Object(field_info) = field_def_value {
                    if let Some(default_val) = field_info.get("default") {
                        // Use default value
                        instance_fields.insert(field_name.clone(), default_val.clone());
                    } else {
                        // No default, field is required
                        return Err(InterpreterError::RuntimeError(format!(
                            "Missing required field '{field_name}' for struct {name}"
                        )));
                    }
                } else {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Invalid field definition for '{field_name}' in struct {name}"
                    )));
                }
            }
        }

        // Return different value types based on __type
        if type_name == "Struct" {
            // Pure struct: return Value::Struct (methods stored separately via qualified names)
            Ok(Value::Struct {
                name: name.to_string(),
                fields: Arc::new(instance_fields),
            })
        } else {
            // Class: return Value::Object (includes metadata like __type, __class, __methods)
            let mut class_instance = instance_fields;

            // Add metadata to instance
            class_instance.insert(
                "__type".to_string(),
                Value::from_string("instance".to_string()),
            );
            class_instance.insert("__class".to_string(), Value::from_string(name.to_string()));

            // Copy methods from class definition to instance
            if let Some(Value::Object(methods)) = struct_type_obj.get("__methods") {
                class_instance.insert("__methods".to_string(), Value::Object(Arc::clone(methods)));
            }

            Ok(Value::Object(Arc::new(class_instance)))
        }
    }

    pub(crate) fn instantiate_struct_with_args(
        &mut self,
        struct_name: &str,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        // Look up the struct definition
        let struct_def = self.lookup_variable(struct_name)?;

        if let Value::Object(ref struct_info) = struct_def {
            // Verify this is a struct
            if let Some(Value::String(ref type_str)) = struct_info.get("__type") {
                if type_str.as_ref() != "Struct" {
                    return Err(InterpreterError::RuntimeError(format!(
                        "{} is not a struct",
                        struct_name
                    )));
                }
            }

            // For structs with positional arguments, we need to map them to fields
            // This is a simplified version - real implementation would need parameter names
            // For now, create an empty struct instance
            let mut instance = HashMap::new();
            instance.insert(
                "__struct".to_string(),
                Value::from_string(struct_name.to_string()),
            );

            // Initialize fields with default values
            if let Some(Value::Object(ref fields)) = struct_info.get("__fields") {
                // Map positional arguments to fields (assuming order matches definition)
                for (i, (field_name, field_info)) in fields.iter().enumerate() {
                    if i < args.len() {
                        instance.insert(field_name.clone(), args[i].clone());
                    } else if let Value::Object(ref field_meta) = field_info {
                        // Use default value if present
                        if let Some(default) = field_meta.get("default") {
                            instance.insert(field_name.clone(), default.clone());
                        } else {
                            // Initialize with default for type
                            instance.insert(field_name.clone(), Value::Nil);
                        }
                    }
                }
            }

            Ok(Value::Object(Arc::new(instance)))
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "{} is not a struct definition",
                struct_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Span, StructField, Type, TypeKind, Visibility};

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_type(name: &str) -> Type {
        Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::default(),
        }
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

    #[test]
    fn test_eval_struct_definition_empty() {
        let mut interp = make_interpreter();
        let result = interp.eval_struct_definition("Empty", &[], &[], &[], false).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(obj.get("__type"), Some(&Value::from_string("Struct".to_string())));
            assert_eq!(obj.get("__name"), Some(&Value::from_string("Empty".to_string())));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_struct_definition_with_fields() {
        let mut interp = make_interpreter();
        let fields = vec![
            make_struct_field("x", make_type("i32")),
            make_struct_field("y", make_type("i32")),
        ];

        let result = interp.eval_struct_definition("Point", &[], &fields, &[], false).unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Object(fields_obj)) = obj.get("__fields") {
                assert!(fields_obj.contains_key("x"));
                assert!(fields_obj.contains_key("y"));
            } else {
                panic!("Expected __fields Object");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_eval_struct_literal_undefined() {
        let mut interp = make_interpreter();
        let result = interp.eval_struct_literal("UndefinedStruct", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Undefined struct type"));
    }

    #[test]
    fn test_eval_struct_literal_not_struct() {
        let mut interp = make_interpreter();
        interp.set_variable("NotStruct", Value::Integer(42));
        let result = interp.eval_struct_literal("NotStruct", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a struct type"));
    }

    #[test]
    fn test_eval_struct_literal_wrong_type() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Other".to_string()));
        interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

        let result = interp.eval_struct_literal("WrongType", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a struct or class type"));
    }

    #[test]
    fn test_instantiate_struct_undefined() {
        let mut interp = make_interpreter();
        let result = interp.instantiate_struct_with_args("UndefinedStruct", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_instantiate_struct_not_struct() {
        let mut interp = make_interpreter();
        interp.set_variable("NotStruct", Value::Integer(42));
        let result = interp.instantiate_struct_with_args("NotStruct", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a struct definition"));
    }

    #[test]
    fn test_instantiate_struct_wrong_type() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Class".to_string()));
        interp.set_variable("WrongType", Value::Object(Arc::new(obj)));

        let result = interp.instantiate_struct_with_args("WrongType", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a struct"));
    }

    #[test]
    fn test_instantiate_struct_with_args() {
        let mut interp = make_interpreter();

        // Create struct definition
        let fields = vec![
            make_struct_field("x", make_type("i32")),
        ];
        interp.eval_struct_definition("Point", &[], &fields, &[], false).unwrap();

        // Instantiate with args
        let result = interp.instantiate_struct_with_args("Point", &[Value::Integer(10)]).unwrap();

        if let Value::Object(obj) = result {
            assert_eq!(obj.get("__struct"), Some(&Value::from_string("Point".to_string())));
        } else {
            panic!("Expected Object");
        }
    }
}
