//! Collection transpilation helpers (list, set, tuple, range, objects, structs)
#![allow(clippy::missing_errors_doc)]

use super::super::Transpiler;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

impl Transpiler {
    pub fn transpile_list(&self, elements: &[Expr]) -> Result<TokenStream> {
        // Check if any elements are spread expressions
        let has_spread = elements
            .iter()
            .any(|e| matches!(e.kind, crate::frontend::ast::ExprKind::Spread { .. }));
        if has_spread {
            // Handle spread expressions by building vector with extends
            let mut statements = Vec::new();
            statements.push(quote! { let mut __temp_vec = Vec::new(); });
            for element in elements {
                if let crate::frontend::ast::ExprKind::Spread { expr } = &element.kind {
                    let expr_tokens = self.transpile_expr(expr)?;
                    statements.push(quote! { __temp_vec.extend(#expr_tokens); });
                } else {
                    let expr_tokens = self.transpile_expr(element)?;
                    statements.push(quote! { __temp_vec.push(#expr_tokens); });
                }
            }
            statements.push(quote! { __temp_vec });
            Ok(quote! { { #(#statements)* } })
        } else if elements.is_empty() {
            // Empty arrays need vec![] to avoid type ambiguity ([] requires type annotation)
            // TRANSPILER-007: Use turbofish syntax to enable Rust type inference from context
            Ok(quote! { vec![] })
        } else {
            // Default: Use array literal syntax [elem1, elem2, ...]
            // This works with explicitly typed [T; N] parameters
            // For Vec<T> params with inferred types, call_helpers adds .to_vec() conversion
            let element_tokens: Result<Vec<_>> =
                elements.iter().map(|e| self.transpile_expr(e)).collect();
            let element_tokens = element_tokens?;
            Ok(quote! { [#(#element_tokens),*] })
        }
    }

    /// Transpiles set literals into `HashSet`
    pub fn transpile_set(&self, elements: &[Expr]) -> Result<TokenStream> {
        // Check if any elements are spread expressions
        let has_spread = elements
            .iter()
            .any(|e| matches!(e.kind, crate::frontend::ast::ExprKind::Spread { .. }));

        if has_spread {
            // Handle spread expressions by building hashset with extends
            let mut statements = Vec::new();
            statements.push(quote! { let mut __temp_set = std::collections::HashSet::new(); });

            for element in elements {
                if let crate::frontend::ast::ExprKind::Spread { expr } = &element.kind {
                    let expr_tokens = self.transpile_expr(expr)?;
                    statements.push(quote! { __temp_set.extend(#expr_tokens); });
                } else {
                    let expr_tokens = self.transpile_expr(element)?;
                    statements.push(quote! { __temp_set.insert(#expr_tokens); });
                }
            }

            statements.push(quote! { __temp_set });
            Ok(quote! { { #(#statements)* } })
        } else if elements.is_empty() {
            // Empty set literal
            Ok(quote! { std::collections::HashSet::new() })
        } else {
            // No spread expressions, build HashSet with inserts
            let mut statements = Vec::new();
            statements.push(quote! { let mut __temp_set = std::collections::HashSet::new(); });

            for element in elements {
                let expr_tokens = self.transpile_expr(element)?;
                statements.push(quote! { __temp_set.insert(#expr_tokens); });
            }

            statements.push(quote! { __temp_set });
            Ok(quote! { { #(#statements)* } })
        }
    }

    /// Transpiles tuple literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_tuple;
    ///
    /// let result = transpile_tuple(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_tuple(&self, elements: &[Expr]) -> Result<TokenStream> {
        let element_tokens: Result<Vec<_>> =
            elements.iter().map(|e| self.transpile_expr(e)).collect();
        let element_tokens = element_tokens?;
        Ok(quote! { (#(#element_tokens),*) })
    }
    /// Transpiles range expressions
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_range;
    ///
    /// let result = transpile_range(true);
    /// assert_eq!(result, Ok(true));
    /// ```
    pub fn transpile_range(
        &self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<TokenStream> {
        let start_tokens = self.transpile_expr(start)?;
        let end_tokens = self.transpile_expr(end)?;
        if inclusive {
            Ok(quote! { #start_tokens..=#end_tokens })
        } else {
            Ok(quote! { #start_tokens..#end_tokens })
        }
    }
    /// Transpiles object literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_object_literal;
    ///
    /// let result = transpile_object_literal(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_object_literal(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<TokenStream> {
        let field_tokens = self.collect_hashmap_field_tokens(fields)?;
        // DEFECT-DICT-DETERMINISM FIX: Use BTreeMap for deterministic key ordering
        // BTreeMap maintains sorted order, HashMap has non-deterministic iteration order
        Ok(quote! {
            {
                let mut map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
                #(#field_tokens)*
                map
            }
        })
    }
    fn collect_hashmap_field_tokens(
        &self,
        fields: &[crate::frontend::ast::ObjectField],
    ) -> Result<Vec<TokenStream>> {
        use crate::frontend::ast::ObjectField;
        let mut field_tokens = Vec::new();
        for field in fields {
            let token = match field {
                ObjectField::KeyValue { key, value } => {
                    let value_tokens = self.transpile_expr(value)?;
                    quote! { map.insert(#key.to_string(), (#value_tokens).to_string()); }
                }
                ObjectField::Spread { expr } => {
                    let expr_tokens = self.transpile_expr(expr)?;
                    // For spread syntax, merge the other map into this one
                    quote! {
                        for (k, v) in #expr_tokens {
                            map.insert(k, v);
                        }
                    }
                }
            };
            field_tokens.push(token);
        }
        Ok(field_tokens)
    }
    /// Transpiles struct literals
    /// # Examples
    ///
    /// ```
    /// use ruchy::backend::transpiler::expressions::transpile_struct_literal;
    ///
    /// let result = transpile_struct_literal("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn transpile_struct_literal(
        &self,
        name: &str,
        fields: &[(String, Expr)],
        base: Option<&Expr>,
    ) -> Result<TokenStream> {
        // DEFECT-020 FIX: Handle enum variant paths like Shape::Circle
        // format_ident! only works for simple identifiers, not paths with ::
        let struct_name: TokenStream = if name.contains("::") {
            // Parse as path (e.g., Shape::Circle)
            let path: syn::Path =
                syn::parse_str(name).map_err(|e| anyhow::anyhow!("Invalid path '{name}': {e}"))?;
            quote! { #path }
        } else {
            // Simple identifier (e.g., Person)
            let ident = format_ident!("{}", name);
            quote! { #ident }
        };
        let mut field_tokens = Vec::new();
        // BOOK-COMPAT-002: Get struct field types for proper string conversion
        let field_types = self.struct_field_types.borrow();
        // Extract base struct name (handle enum variants like Shape::Circle)
        let base_struct_name = if name.contains("::") {
            name.split("::").next().unwrap_or(name)
        } else {
            name
        };
        // BOOK-COMPAT-007B: Check for auto-boxed recursive types
        let auto_boxed = self.auto_boxed_fields.borrow();
        for (field_name, value) in fields {
            let field_ident = format_ident!("{}", field_name);
            // BOOK-COMPAT-002 FIX: Add .to_string() for String fields with string literals
            // When a struct field is typed as String and the value is a string literal,
            // we need to add .to_string() for the Rust code to compile correctly.
            let field_type = field_types.get(&(base_struct_name.to_string(), field_name.clone()));
            let needs_to_string = matches!(field_type, Some(t) if t == "String")
                && matches!(&value.kind, ExprKind::Literal(Literal::String(_)));

            // BOOK-COMPAT-007B: Check if this field needs Box wrapping for recursive types
            // When the value is Some(x) and the field is an auto-boxed Option<Box<T>>,
            // we need to transform Some(x) to Some(Box::new(x))
            let needs_box_wrap = if let ExprKind::Some { value: inner_value } = &value.kind {
                // Check if this struct has an auto-boxed field for any recursive type
                auto_boxed.keys().any(|(s, _)| s == base_struct_name)
                    && matches!(&inner_value.kind, ExprKind::Identifier(_))
            } else {
                false
            };

            if needs_box_wrap {
                if let ExprKind::Some { value: inner_value } = &value.kind {
                    let inner_tokens = self.transpile_expr(inner_value)?;
                    field_tokens.push(quote! { #field_ident: Some(Box::new(#inner_tokens)) });
                }
            } else {
                let value_tokens = self.transpile_expr(value)?;
                if needs_to_string {
                    field_tokens.push(quote! { #field_ident: #value_tokens.to_string() });
                } else {
                    field_tokens.push(quote! { #field_ident: #value_tokens });
                }
            }
        }
        drop(field_types); // Release borrow before potential recursive calls
        drop(auto_boxed);

        // Handle struct update syntax
        if let Some(base_expr) = base {
            let base_tokens = self.transpile_expr(base_expr)?;
            Ok(quote! {
                #struct_name {
                    #(#field_tokens,)*
                    ..#base_tokens
                }
            })
        } else if field_tokens.is_empty() {
            // BOOK-COMPAT-005: Empty struct literal `Settings {}` becomes `Settings::default()`
            // This works for structs with Default impl (from default field values)
            Ok(quote! {
                #struct_name::default()
            })
        } else {
            // BOOK-COMPAT-006: For partial struct literals, check if struct might have defaults
            // If struct has registered field types (meaning it has a definition we've seen),
            // and we're not providing all fields, use struct update syntax with Default
            let field_types = self.struct_field_types.borrow();
            let struct_has_fields = field_types.keys().any(|(s, _)| s == base_struct_name);
            let provided_count = fields.len();
            let total_fields = field_types
                .keys()
                .filter(|(s, _)| s == base_struct_name)
                .count();
            drop(field_types);

            if struct_has_fields && provided_count < total_fields {
                // Partial struct literal - use struct update syntax with Default
                Ok(quote! {
                    #struct_name {
                        #(#field_tokens,)*
                        ..Default::default()
                    }
                })
            } else {
                Ok(quote! {
                    #struct_name {
                        #(#field_tokens,)*
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, ObjectField, Span};

    // Helper to create test transpiler
    fn test_transpiler() -> Transpiler {
        Transpiler::new()
    }

    // Helper to create integer literal expression
    fn int_expr(value: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(value, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Helper to create string literal expression
    fn string_expr(value: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(value.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test 1: transpile_list - empty list
    #[test]
    fn test_transpile_list_empty() {
        let transpiler = test_transpiler();
        let result = transpiler
            .transpile_list(&[])
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "vec ! []");
    }

    // Test 2: transpile_list - single element
    #[test]
    fn test_transpile_list_single() {
        let transpiler = test_transpiler();
        let elements = vec![int_expr(42)];
        let result = transpiler
            .transpile_list(&elements)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
    }

    // Test 3: transpile_list - multiple elements
    #[test]
    fn test_transpile_list_multiple() {
        let transpiler = test_transpiler();
        let elements = vec![int_expr(1), int_expr(2), int_expr(3)];
        let result = transpiler
            .transpile_list(&elements)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(result_str.contains('1') && result_str.contains('2') && result_str.contains('3'));
    }

    // Test 4: transpile_set - empty set
    #[test]
    fn test_transpile_set_empty() {
        let transpiler = test_transpiler();
        let result = transpiler
            .transpile_set(&[])
            .expect("operation should succeed in test");
        assert_eq!(
            result.to_string(),
            "std :: collections :: HashSet :: new ()"
        );
    }

    // Test 5: transpile_set - single element
    #[test]
    fn test_transpile_set_single() {
        let transpiler = test_transpiler();
        let elements = vec![int_expr(42)];
        let result = transpiler
            .transpile_set(&elements)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(
            result_str.contains("HashSet")
                && result_str.contains("insert")
                && result_str.contains("42")
        );
    }

    // Test 6: transpile_set - multiple elements
    #[test]
    fn test_transpile_set_multiple() {
        let transpiler = test_transpiler();
        let elements = vec![int_expr(1), int_expr(2), int_expr(3)];
        let result = transpiler
            .transpile_set(&elements)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(
            result_str.contains("HashSet") && result_str.contains('1') && result_str.contains('2')
        );
    }

    // Test 7: transpile_tuple - empty tuple
    #[test]
    fn test_transpile_tuple_empty() {
        let transpiler = test_transpiler();
        let result = transpiler
            .transpile_tuple(&[])
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "()");
    }

    // Test 8: transpile_tuple - single element
    #[test]
    fn test_transpile_tuple_single() {
        let transpiler = test_transpiler();
        let elements = vec![int_expr(42)];
        let result = transpiler
            .transpile_tuple(&elements)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "(42)");
    }

    // Test 9: transpile_tuple - multiple elements
    #[test]
    fn test_transpile_tuple_multiple() {
        let transpiler = test_transpiler();
        let elements = vec![int_expr(1), string_expr("hello")];
        let result = transpiler
            .transpile_tuple(&elements)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(
            result_str.contains('(') && result_str.contains('1') && result_str.contains("hello")
        );
    }

    // Test 10: transpile_range - exclusive range
    #[test]
    fn test_transpile_range_exclusive() {
        let transpiler = test_transpiler();
        let start = int_expr(0);
        let end = int_expr(10);
        let result = transpiler
            .transpile_range(&start, &end, false)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "0 .. 10");
    }

    // Test 11: transpile_range - inclusive range
    #[test]
    fn test_transpile_range_inclusive() {
        let transpiler = test_transpiler();
        let start = int_expr(0);
        let end = int_expr(10);
        let result = transpiler
            .transpile_range(&start, &end, true)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "0 ..= 10");
    }

    // Test 12: transpile_object_literal - empty object
    #[test]
    fn test_transpile_object_literal_empty() {
        let transpiler = test_transpiler();
        let result = transpiler
            .transpile_object_literal(&[])
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(result_str.contains("BTreeMap") && result_str.contains("new"));
    }

    // Test 13: transpile_object_literal - single field
    #[test]
    fn test_transpile_object_literal_single() {
        let transpiler = test_transpiler();
        let fields = vec![ObjectField::KeyValue {
            key: "name".to_string(),
            value: string_expr("Alice"),
        }];
        let result = transpiler
            .transpile_object_literal(&fields)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(
            result_str.contains("BTreeMap")
                && result_str.contains("insert")
                && result_str.contains("name")
        );
    }

    // Test 14: transpile_object_literal - multiple fields
    #[test]
    fn test_transpile_object_literal_multiple() {
        let transpiler = test_transpiler();
        let fields = vec![
            ObjectField::KeyValue {
                key: "name".to_string(),
                value: string_expr("Alice"),
            },
            ObjectField::KeyValue {
                key: "age".to_string(),
                value: int_expr(30),
            },
        ];
        let result = transpiler
            .transpile_object_literal(&fields)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(result_str.contains("name") && result_str.contains("age"));
    }

    // Test 15: transpile_struct_literal - simple struct
    #[test]
    fn test_transpile_struct_literal_simple() {
        let transpiler = test_transpiler();
        let fields = vec![
            ("name".to_string(), string_expr("Alice")),
            ("age".to_string(), int_expr(30)),
        ];
        let result = transpiler
            .transpile_struct_literal("Person", &fields, None)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(
            result_str.contains("Person")
                && result_str.contains("name")
                && result_str.contains("age")
        );
    }

    // Test 16: transpile_struct_literal - with base (struct update syntax)
    #[test]
    fn test_transpile_struct_literal_with_base() {
        let transpiler = test_transpiler();
        let fields = vec![("name".to_string(), string_expr("Bob"))];
        let base = Expr {
            kind: ExprKind::Identifier("old_person".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler
            .transpile_struct_literal("Person", &fields, Some(&base))
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(
            result_str.contains("Person")
                && result_str.contains("..")
                && result_str.contains("old_person")
        );
    }

    // Test 17: transpile_struct_literal - empty struct
    #[test]
    fn test_transpile_struct_literal_empty() {
        // BOOK-COMPAT-005: Empty struct literals use ::default()
        let transpiler = test_transpiler();
        let result = transpiler
            .transpile_struct_literal("EmptyStruct", &[], None)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(result_str.contains("EmptyStruct") && result_str.contains("default"));
    }

    // Test 18: collect_hashmap_field_tokens - key-value field
    #[test]
    fn test_collect_hashmap_field_tokens_keyvalue() {
        let transpiler = test_transpiler();
        let fields = vec![ObjectField::KeyValue {
            key: "test".to_string(),
            value: int_expr(42),
        }];
        let result = transpiler
            .collect_hashmap_field_tokens(&fields)
            .expect("operation should succeed in test");
        assert_eq!(result.len(), 1);
        let token_str = result[0].to_string();
        assert!(token_str.contains("insert") && token_str.contains("test"));
    }

    // Test 19: transpile_range - with expressions
    #[test]
    fn test_transpile_range_complex() {
        let transpiler = test_transpiler();
        let start = Expr {
            kind: ExprKind::Identifier("start".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let end = Expr {
            kind: ExprKind::Identifier("end".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = transpiler
            .transpile_range(&start, &end, false)
            .expect("operation should succeed in test");
        assert_eq!(result.to_string(), "start .. end");
    }

    // Test 20: transpile_struct_literal - complex field values
    #[test]
    fn test_transpile_struct_literal_complex_values() {
        let transpiler = test_transpiler();
        let nested_list = Expr {
            kind: ExprKind::List(vec![int_expr(1), int_expr(2)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let fields = vec![("numbers".to_string(), nested_list)];
        let result = transpiler
            .transpile_struct_literal("Data", &fields, None)
            .expect("operation should succeed in test");
        let result_str = result.to_string();
        assert!(result_str.contains("Data") && result_str.contains("numbers"));
    }

    // ===== EXTREME TDD Round 142 - Spread and Edge Case Tests =====

    // Test 21: transpile_list with spread expression
    #[test]
    fn test_transpile_list_with_spread() {
        let transpiler = test_transpiler();
        let inner = Expr {
            kind: ExprKind::Identifier("other_vec".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let spread = Expr {
            kind: ExprKind::Spread {
                expr: Box::new(inner),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let elements = vec![spread];
        let result = transpiler
            .transpile_list(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        // Spread generates Vec with extend
        assert!(result_str.contains("Vec") || result_str.contains("vec"));
        assert!(result_str.contains("extend"));
    }

    // Test 22: transpile_list with spread and regular elements
    #[test]
    fn test_transpile_list_mixed_spread() {
        let transpiler = test_transpiler();
        let inner = Expr {
            kind: ExprKind::Identifier("arr".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let spread = Expr {
            kind: ExprKind::Spread {
                expr: Box::new(inner),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let elements = vec![int_expr(1), spread, int_expr(3)];
        let result = transpiler
            .transpile_list(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains("push"));
        assert!(result_str.contains("extend"));
    }

    // Test 23: transpile_set with spread expression
    #[test]
    fn test_transpile_set_with_spread() {
        let transpiler = test_transpiler();
        let inner = Expr {
            kind: ExprKind::Identifier("other_set".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let spread = Expr {
            kind: ExprKind::Spread {
                expr: Box::new(inner),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let elements = vec![spread];
        let result = transpiler
            .transpile_set(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        // Spread generates HashSet with extend
        assert!(result_str.contains("HashSet"));
        assert!(result_str.contains("extend"));
    }

    // Test 24: transpile_set with mixed spread and regular
    #[test]
    fn test_transpile_set_mixed_spread() {
        let transpiler = test_transpiler();
        let inner = Expr {
            kind: ExprKind::Identifier("s".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let spread = Expr {
            kind: ExprKind::Spread {
                expr: Box::new(inner),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let elements = vec![int_expr(1), spread];
        let result = transpiler
            .transpile_set(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains("insert"));
        assert!(result_str.contains("extend"));
    }

    // Test 25: transpile_object_literal with spread
    #[test]
    fn test_transpile_object_literal_spread() {
        let transpiler = test_transpiler();
        let other_map = Expr {
            kind: ExprKind::Identifier("defaults".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let fields = vec![ObjectField::Spread { expr: other_map }];
        let result = transpiler
            .transpile_object_literal(&fields)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains("BTreeMap"));
        assert!(result_str.contains("defaults"));
    }

    // Test 26: transpile_struct_literal with enum path
    #[test]
    fn test_transpile_struct_literal_enum_path() {
        let transpiler = test_transpiler();
        let fields = vec![("radius".to_string(), int_expr(10))];
        let result = transpiler
            .transpile_struct_literal("Shape::Circle", &fields, None)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains("Shape"));
        assert!(result_str.contains("Circle"));
    }

    // Test 27: transpile_tuple with many elements
    #[test]
    fn test_transpile_tuple_many_elements() {
        let transpiler = test_transpiler();
        let elements = vec![
            int_expr(1),
            int_expr(2),
            int_expr(3),
            int_expr(4),
            int_expr(5),
        ];
        let result = transpiler
            .transpile_tuple(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains('1'));
        assert!(result_str.contains('5'));
    }

    // Test 28: transpile_range with negative numbers
    #[test]
    fn test_transpile_range_negative() {
        let transpiler = test_transpiler();
        let start = Expr {
            kind: ExprKind::Literal(Literal::Integer(-5, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let end = int_expr(5);
        let result = transpiler
            .transpile_range(&start, &end, false)
            .expect("operation should succeed");
        let result_str = result.to_string();
        // Negative numbers may be represented with or without spaces
        assert!(result_str.contains("- 5") || result_str.contains("-5"));
        assert!(result_str.contains(".."));
    }

    // Test 29: transpile_list with strings
    #[test]
    fn test_transpile_list_strings() {
        let transpiler = test_transpiler();
        let elements = vec![string_expr("hello"), string_expr("world")];
        let result = transpiler
            .transpile_list(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains("hello"));
        assert!(result_str.contains("world"));
    }

    // Test 30: transpile_set with strings
    #[test]
    fn test_transpile_set_strings() {
        let transpiler = test_transpiler();
        let elements = vec![string_expr("a"), string_expr("b")];
        let result = transpiler
            .transpile_set(&elements)
            .expect("operation should succeed");
        let result_str = result.to_string();
        assert!(result_str.contains("HashSet"));
        assert!(result_str.contains("insert"));
    }
}
