use super::*;
use crate::Transpiler;


/// Test AST builder literal methods
#[test]
fn test_int_literal() {
    let builder = AstBuilder::new();
    let expr = builder.int(42);

    // Verify structure
    if let ExprKind::Literal(Literal::Integer(value, None)) = expr.kind {
        assert_eq!(value, 42);
    } else {
        panic!("Expected integer literal");
    }

    // Verify attributes and span
    assert!(expr.attributes.is_empty());
    assert_eq!(expr.span, Span::default());
}

#[test]
fn test_float_literal() {
    let builder = AstBuilder::new();
    let expr = builder.float(3.15);

    if let ExprKind::Literal(Literal::Float(value)) = expr.kind {
        assert!((value - 3.15).abs() < f64::EPSILON);
    } else {
        panic!("Expected float literal");
    }
}

#[test]
fn test_string_literal() {
    let builder = AstBuilder::new();
    let expr = builder.string("hello world");

    if let ExprKind::Literal(Literal::String(value)) = expr.kind {
        assert_eq!(value, "hello world");
    } else {
        panic!("Expected string literal");
    }
}

#[test]
fn test_bool_literal() {
    let builder = AstBuilder::new();

    let true_expr = builder.bool(true);
    if let ExprKind::Literal(Literal::Bool(value)) = true_expr.kind {
        assert!(value);
    } else {
        panic!("Expected boolean true literal");
    }

    let false_expr = builder.bool(false);
    if let ExprKind::Literal(Literal::Bool(value)) = false_expr.kind {
        assert!(!value);
    } else {
        panic!("Expected boolean false literal");
    }
}

#[test]
fn test_identifier() {
    let builder = AstBuilder::new();
    let expr = builder.ident("variable_name");

    if let ExprKind::Identifier(name) = expr.kind {
        assert_eq!(name, "variable_name");
    } else {
        panic!("Expected identifier");
    }
}

/// Test AST builder binary operations
#[test]
fn test_binary_operations() {
    let builder = AstBuilder::new();

    // Test addition
    let add_expr = builder.binary(builder.int(1), BinaryOp::Add, builder.int(2));

    if let ExprKind::Binary { left, op, right } = add_expr.kind {
        assert!(matches!(op, BinaryOp::Add));
        if let ExprKind::Literal(Literal::Integer(val, None)) = left.kind {
            assert_eq!(val, 1);
        } else {
            panic!("Expected left operand to be 1");
        }
        if let ExprKind::Literal(Literal::Integer(val, None)) = right.kind {
            assert_eq!(val, 2);
        } else {
            panic!("Expected right operand to be 2");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_unary_operations() {
    let builder = AstBuilder::new();

    // Test negation
    let neg_expr = builder.unary(UnaryOp::Negate, builder.int(42));

    if let ExprKind::Unary { op, operand } = neg_expr.kind {
        assert!(matches!(op, UnaryOp::Negate));
        if let ExprKind::Literal(Literal::Integer(val, None)) = operand.kind {
            assert_eq!(val, 42);
        } else {
            panic!("Expected operand to be 42");
        }
    } else {
        panic!("Expected unary expression");
    }
}

#[test]
fn test_if_expression() {
    let builder = AstBuilder::new();

    // Test if-else
    let if_expr = builder.if_expr(
        builder.bool(true),
        builder.string("then"),
        Some(builder.string("else")),
    );

    if let ExprKind::If {
        condition,
        then_branch,
        else_branch,
    } = if_expr.kind
    {
        // Verify condition
        if let ExprKind::Literal(Literal::Bool(val)) = condition.kind {
            assert!(val);
        } else {
            panic!("Expected boolean condition");
        }

        // Verify then branch
        if let ExprKind::Literal(Literal::String(val)) = then_branch.kind {
            assert_eq!(val, "then");
        } else {
            panic!("Expected string 'then'");
        }

        // Verify else branch
        assert!(else_branch.is_some());
        if let Some(else_box) = else_branch {
            if let ExprKind::Literal(Literal::String(val)) = else_box.kind {
                assert_eq!(val, "else");
            } else {
                panic!("Expected string 'else'");
            }
        }
    } else {
        panic!("Expected if expression");
    }
}

#[test]
fn test_if_expression_without_else() {
    let builder = AstBuilder::new();

    let if_expr = builder.if_expr(builder.bool(true), builder.string("then"), None);

    if let ExprKind::If { else_branch, .. } = if_expr.kind {
        assert!(else_branch.is_none());
    } else {
        panic!("Expected if expression");
    }
}

#[test]
fn test_function_call() {
    let builder = AstBuilder::new();

    let call_expr = builder.call(
        builder.ident("function_name"),
        vec![builder.int(1), builder.string("arg")],
    );

    if let ExprKind::Call { func, args } = call_expr.kind {
        // Verify function name
        if let ExprKind::Identifier(name) = func.kind {
            assert_eq!(name, "function_name");
        } else {
            panic!("Expected function identifier");
        }

        // Verify arguments
        assert_eq!(args.len(), 2);
        if let ExprKind::Literal(Literal::Integer(val, None)) = args[0].kind {
            assert_eq!(val, 1);
        } else {
            panic!("Expected first argument to be 1");
        }
        if let ExprKind::Literal(Literal::String(val)) = &args[1].kind {
            assert_eq!(val, "arg");
        } else {
            panic!("Expected second argument to be 'arg'");
        }
    } else {
        panic!("Expected call expression");
    }
}

#[test]
fn test_lambda_expression() {
    let builder = AstBuilder::new();

    let lambda = builder.lambda(
        vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("unknown".to_string()),
                span: Span::new(0, 1),
            }, // Fixed: create proper Type struct
            span: Span::new(0, 1),
            is_mutable: false,
            default_value: None,
        }],
        builder.ident("x"),
    );

    if let ExprKind::Lambda { params, body } = lambda.kind {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name(), "x");
        assert!(matches!(params[0].ty.kind, TypeKind::Named(ref name) if name == "unknown")); // Fixed: match TypeKind properly

        if let ExprKind::Identifier(name) = body.kind {
            assert_eq!(name, "x");
        } else {
            panic!("Expected body to be identifier 'x'");
        }
    } else {
        panic!("Expected lambda expression");
    }
}

/// Test AST builder pattern methods
#[test]
fn test_pattern_wildcard() {
    let builder = AstBuilder::new();
    let pattern = builder.pattern_wildcard();
    assert!(matches!(pattern, Pattern::Wildcard));
}

#[test]
fn test_pattern_identifier() {
    let builder = AstBuilder::new();
    let pattern = builder.pattern_ident("var_name");

    if let Pattern::Identifier(name) = pattern {
        assert_eq!(name, "var_name");
    } else {
        panic!("Expected identifier pattern");
    }
}

#[test]
fn test_pattern_literal() {
    let builder = AstBuilder::new();
    let pattern = builder.pattern_literal(Literal::Integer(42, None));

    if let Pattern::Literal(Literal::Integer(val, None)) = pattern {
        assert_eq!(val, 42);
    } else {
        panic!("Expected literal pattern");
    }
}

#[test]
fn test_pattern_tuple() {
    let builder = AstBuilder::new();
    let pattern =
        builder.pattern_tuple(vec![builder.pattern_ident("x"), builder.pattern_ident("y")]);

    if let Pattern::Tuple(patterns) = pattern {
        assert_eq!(patterns.len(), 2);
        assert!(matches!(patterns[0], Pattern::Identifier(_)));
        assert!(matches!(patterns[1], Pattern::Identifier(_)));
    } else {
        panic!("Expected tuple pattern");
    }
}

#[test]
fn test_pattern_or() {
    let builder = AstBuilder::new();
    let pattern = builder.pattern_or(vec![
        builder.pattern_literal(Literal::Integer(1, None)),
        builder.pattern_literal(Literal::Integer(2, None)),
    ]);

    if let Pattern::Or(patterns) = pattern {
        assert_eq!(patterns.len(), 2);
        assert!(matches!(
            patterns[0],
            Pattern::Literal(Literal::Integer(1, None))
        ));
        assert!(matches!(
            patterns[1],
            Pattern::Literal(Literal::Integer(2, None))
        ));
    } else {
        panic!("Expected or pattern");
    }
}

#[test]
fn test_pattern_struct() {
    let builder = AstBuilder::new();
    let pattern = builder.pattern_struct(
        "Point".to_string(),
        vec![
            ("x".to_string(), builder.pattern_ident("x_val")),
            ("y".to_string(), builder.pattern_ident("y_val")),
        ],
    );

    if let Pattern::Struct {
        name,
        fields,
        has_rest,
    } = pattern
    {
        assert_eq!(name, "Point");
        assert_eq!(fields.len(), 2);
        assert!(!has_rest);

        assert_eq!(fields[0].name, "x");
        assert!(fields[0].pattern.is_some());
        assert_eq!(fields[1].name, "y");
        assert!(fields[1].pattern.is_some());
    } else {
        panic!("Expected struct pattern");
    }
}

#[test]
fn test_pattern_rest() {
    let builder = AstBuilder::new();
    let pattern = builder.pattern_rest();
    assert!(matches!(pattern, Pattern::Rest));
}

/// Test AST builder collection methods
#[test]
fn test_list_literal() {
    let builder = AstBuilder::new();
    let list = builder.list(vec![builder.int(1), builder.int(2), builder.int(3)]);

    if let ExprKind::List(elements) = list.kind {
        assert_eq!(elements.len(), 3);
        for (i, element) in elements.iter().enumerate() {
            if let ExprKind::Literal(Literal::Integer(val, None)) = element.kind {
                assert_eq!(val, (i + 1) as i64);
            } else {
                panic!("Expected integer literal at index {i}");
            }
        }
    } else {
        panic!("Expected list expression");
    }
}

#[test]
fn test_tuple_literal() {
    let builder = AstBuilder::new();
    let tuple = builder.tuple(vec![builder.string("first"), builder.int(42)]);

    if let ExprKind::Tuple(elements) = tuple.kind {
        assert_eq!(elements.len(), 2);

        if let ExprKind::Literal(Literal::String(val)) = &elements[0].kind {
            assert_eq!(val, "first");
        } else {
            panic!("Expected string literal as first element");
        }

        if let ExprKind::Literal(Literal::Integer(val, None)) = elements[1].kind {
            assert_eq!(val, 42);
        } else {
            panic!("Expected integer literal as second element");
        }
    } else {
        panic!("Expected tuple expression");
    }
}

/// Test AST builder control flow methods
#[test]
fn test_for_loop() {
    let builder = AstBuilder::new();
    let for_loop = builder.for_loop(
        "i".to_string(),
        builder.list(vec![builder.int(1), builder.int(2)]),
        builder.ident("i"),
    );

    if let ExprKind::For {
        label: _,
        var,
        iter,
        body,
        pattern,
    } = for_loop.kind
    {
        assert_eq!(var, "i");
        assert!(pattern.is_none());

        if let ExprKind::List(_) = iter.kind {
            // Valid iterator
        } else {
            panic!("Expected list as iterator");
        }

        if let ExprKind::Identifier(name) = body.kind {
            assert_eq!(name, "i");
        } else {
            panic!("Expected identifier body");
        }
    } else {
        panic!("Expected for loop");
    }
}

#[test]
fn test_while_loop() {
    let builder = AstBuilder::new();
    let while_loop = builder.while_loop(builder.bool(true), builder.string("body"));

    if let ExprKind::While {
        condition, body, ..
    } = while_loop.kind
    {
        if let ExprKind::Literal(Literal::Bool(val)) = condition.kind {
            assert!(val);
        } else {
            panic!("Expected boolean condition");
        }

        if let ExprKind::Literal(Literal::String(val)) = body.kind {
            assert_eq!(val, "body");
        } else {
            panic!("Expected string body");
        }
    } else {
        panic!("Expected while loop");
    }
}

#[test]
fn test_loop_expression() {
    let builder = AstBuilder::new();
    let loop_expr = builder.loop_expr(builder.string("infinite"));

    if let ExprKind::Loop { body, .. } = loop_expr.kind {
        if let ExprKind::Literal(Literal::String(val)) = body.kind {
            assert_eq!(val, "infinite");
        } else {
            panic!("Expected string body");
        }
    } else {
        panic!("Expected loop expression");
    }
}

#[test]
fn test_break_expression() {
    let builder = AstBuilder::new();

    // Break without label
    let break_expr = builder.break_expr(None);
    if let ExprKind::Break { label, .. } = break_expr.kind {
        assert!(label.is_none());
    } else {
        panic!("Expected break expression");
    }

    // Break with label
    let labeled_break = builder.break_expr(Some("outer".to_string()));
    if let ExprKind::Break { label, .. } = labeled_break.kind {
        assert_eq!(label, Some("outer".to_string()));
    } else {
        panic!("Expected labeled break expression");
    }
}

#[test]
fn test_continue_expression() {
    let builder = AstBuilder::new();

    // Continue without label
    let continue_expr = builder.continue_expr(None);
    if let ExprKind::Continue { label } = continue_expr.kind {
        assert!(label.is_none());
    } else {
        panic!("Expected continue expression");
    }

    // Continue with label
    let labeled_continue = builder.continue_expr(Some("loop1".to_string()));
    if let ExprKind::Continue { label } = labeled_continue.kind {
        assert_eq!(label, Some("loop1".to_string()));
    } else {
        panic!("Expected labeled continue expression");
    }
}

#[test]
fn test_return_expression() {
    let builder = AstBuilder::new();

    // Return without value
    let return_expr = builder.return_expr(None);
    if let ExprKind::Return { value } = return_expr.kind {
        assert!(value.is_none());
    } else {
        panic!("Expected return expression");
    }

    // Return with value
    let return_with_value = builder.return_expr(Some(builder.int(42)));
    if let ExprKind::Return { value } = return_with_value.kind {
        assert!(value.is_some());
        if let Some(val) = value {
            if let ExprKind::Literal(Literal::Integer(num, None)) = val.kind {
                assert_eq!(num, 42);
            } else {
                panic!("Expected integer return value");
            }
        }
    } else {
        panic!("Expected return expression with value");
    }
}

/// Test AST builder utility methods
#[test]
fn test_result_variants() {
    let builder = AstBuilder::new();

    // Test Ok variant
    let ok_expr = builder.ok(builder.int(42));
    if let ExprKind::Call { func, args } = ok_expr.kind {
        if let ExprKind::Identifier(name) = func.kind {
            assert_eq!(name, "Ok");
        } else {
            panic!("Expected Ok function");
        }
        assert_eq!(args.len(), 1);
    } else {
        panic!("Expected call expression for Ok");
    }

    // Test Err variant
    let err_expr = builder.err(builder.string("error"));
    if let ExprKind::Call { func, args } = err_expr.kind {
        if let ExprKind::Identifier(name) = func.kind {
            assert_eq!(name, "Err");
        } else {
            panic!("Expected Err function");
        }
        assert_eq!(args.len(), 1);
    } else {
        panic!("Expected call expression for Err");
    }
}

#[test]
fn test_option_variants() {
    let builder = AstBuilder::new();

    // Test Some variant
    let some_expr = builder.some(builder.string("value"));
    if let ExprKind::Call { func, args } = some_expr.kind {
        if let ExprKind::Identifier(name) = func.kind {
            assert_eq!(name, "Some");
        } else {
            panic!("Expected Some function");
        }
        assert_eq!(args.len(), 1);
    } else {
        panic!("Expected call expression for Some");
    }

    // Test None variant
    let none_expr = builder.none();
    if let ExprKind::Identifier(name) = none_expr.kind {
        assert_eq!(name, "None");
    } else {
        panic!("Expected None identifier");
    }
}

#[test]
fn test_block_expression() {
    let builder = AstBuilder::new();
    let block = builder.block(vec![
        builder.let_expr("x".to_string(), builder.int(1)),
        builder.ident("x"),
    ]);

    if let ExprKind::Block(statements) = block.kind {
        assert_eq!(statements.len(), 2);

        // First statement should be let
        if let ExprKind::Let { name, .. } = &statements[0].kind {
            assert_eq!(name, "x");
        } else {
            panic!("Expected let expression as first statement");
        }

        // Second statement should be identifier
        if let ExprKind::Identifier(name) = &statements[1].kind {
            assert_eq!(name, "x");
        } else {
            panic!("Expected identifier as second statement");
        }
    } else {
        panic!("Expected block expression");
    }
}

#[test]
fn test_let_expression() {
    let builder = AstBuilder::new();
    let let_expr = builder.let_expr("variable".to_string(), builder.int(42));

    if let ExprKind::Let {
        name,
        value,
        type_annotation,
        is_mutable,
        body,
        ..
    } = let_expr.kind
    {
        assert_eq!(name, "variable");
        assert!(type_annotation.is_none());
        assert!(!is_mutable);

        // Verify value
        if let ExprKind::Literal(Literal::Integer(val, None)) = value.kind {
            assert_eq!(val, 42);
        } else {
            panic!("Expected integer value");
        }

        // Verify body is unit
        if let ExprKind::Literal(Literal::Unit) = body.kind {
            // Expected unit body
        } else {
            panic!("Expected unit body");
        }
    } else {
        panic!("Expected let expression");
    }
}

#[test]
fn test_assignment() {
    let builder = AstBuilder::new();
    let assign = builder.assign(builder.ident("variable"), builder.int(100));

    if let ExprKind::Assign { target, value } = assign.kind {
        // Verify target
        if let ExprKind::Identifier(name) = target.kind {
            assert_eq!(name, "variable");
        } else {
            panic!("Expected identifier target");
        }

        // Verify value
        if let ExprKind::Literal(Literal::Integer(val, None)) = value.kind {
            assert_eq!(val, 100);
        } else {
            panic!("Expected integer value");
        }
    } else {
        panic!("Expected assignment expression");
    }
}

/// Test AST builder type methods
#[test]
fn test_type_int() {
    let builder = AstBuilder::new();
    let int_type = builder.type_int();

    if let TypeKind::Named(name) = int_type.kind {
        assert_eq!(name, "i32");
    } else {
        panic!("Expected named type 'i32'");
    }

    assert_eq!(int_type.span, Span::default());
}

#[test]
fn test_type_result() {
    let builder = AstBuilder::new();
    let result_type = builder.type_result(
        builder.type_int(),
        Type {
            kind: TypeKind::Named("String".to_string()),
            span: Span::default(),
        },
    );

    if let TypeKind::Generic { base, params } = result_type.kind {
        assert_eq!(base, "Result");
        assert_eq!(params.len(), 2);

        // Check Ok type
        if let TypeKind::Named(name) = &params[0].kind {
            assert_eq!(name, "i32");
        } else {
            panic!("Expected i32 as Ok type");
        }

        // Check Err type
        if let TypeKind::Named(name) = &params[1].kind {
            assert_eq!(name, "String");
        } else {
            panic!("Expected String as Err type");
        }
    } else {
        panic!("Expected generic Result type");
    }
}

#[test]
fn test_type_option() {
    let builder = AstBuilder::new();
    let option_type = builder.type_option(builder.type_int());

    if let TypeKind::Generic { base, params } = option_type.kind {
        assert_eq!(base, "Option");
        assert_eq!(params.len(), 1);

        // Check inner type
        if let TypeKind::Named(name) = &params[0].kind {
            assert_eq!(name, "i32");
        } else {
            panic!("Expected i32 as Option inner type");
        }
    } else {
        panic!("Expected generic Option type");
    }
}

#[test]
fn test_ast_builder_basic() {
    let builder = AstBuilder::new();
    // Create: if x > 0 { "positive" } else { "negative" }
    let ast = builder.if_expr(
        builder.binary(builder.ident("x"), BinaryOp::Greater, builder.int(0)),
        builder.string("positive"),
        Some(builder.string("negative")),
    );
    // Should be able to transpile
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}
#[test]
fn test_ast_builder_match_with_guard() {
    let builder = AstBuilder::new();
    // Create match with pattern guard (parser doesn't support this)
    let ast = builder.match_expr(
        builder.ident("x"),
        vec![
            builder.match_arm(
                builder.pattern_ident("n"),
                Some(builder.binary(builder.ident("n"), BinaryOp::Greater, builder.int(0))),
                builder.string("positive"),
            ),
            builder.match_arm(builder.pattern_wildcard(), None, builder.string("other")),
        ],
    );
    // Should be able to transpile even though parser can't parse this
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}
#[test]
fn test_ast_builder_or_pattern() {
    let builder = AstBuilder::new();
    // Create or-pattern (parser doesn't support this)
    let ast = builder.match_expr(
        builder.ident("x"),
        vec![
            builder.match_arm(
                builder.pattern_or(vec![
                    builder.pattern_literal(Literal::Integer(1, None)),
                    builder.pattern_literal(Literal::Integer(2, None)),
                    builder.pattern_literal(Literal::Integer(3, None)),
                ]),
                None,
                builder.string("small"),
            ),
            builder.match_arm(builder.pattern_wildcard(), None, builder.string("other")),
        ],
    );
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
}

/// Test match expressions and arms
#[test]
fn test_match_expression() {
    let builder = AstBuilder::new();
    let match_expr = builder.match_expr(
        builder.ident("value"),
        vec![
            builder.match_arm(
                builder.pattern_literal(Literal::Integer(1, None)),
                None,
                builder.string("one"),
            ),
            builder.match_arm(builder.pattern_wildcard(), None, builder.string("other")),
        ],
    );

    if let ExprKind::Match { expr, arms } = match_expr.kind {
        // Verify match expression
        if let ExprKind::Identifier(name) = expr.kind {
            assert_eq!(name, "value");
        } else {
            panic!("Expected identifier in match expression");
        }

        // Verify arms
        assert_eq!(arms.len(), 2);

        // First arm
        assert!(matches!(
            arms[0].pattern,
            Pattern::Literal(Literal::Integer(1, None))
        ));
        assert!(arms[0].guard.is_none());
        if let ExprKind::Literal(Literal::String(val)) = &arms[0].body.kind {
            assert_eq!(val, "one");
        } else {
            panic!("Expected string body in first arm");
        }

        // Second arm
        assert!(matches!(arms[1].pattern, Pattern::Wildcard));
        assert!(arms[1].guard.is_none());
        if let ExprKind::Literal(Literal::String(val)) = &arms[1].body.kind {
            assert_eq!(val, "other");
        } else {
            panic!("Expected string body in second arm");
        }
    } else {
        panic!("Expected match expression");
    }
}

#[test]
fn test_match_arm_with_guard() {
    let builder = AstBuilder::new();
    let arm = builder.match_arm(
        builder.pattern_ident("n"),
        Some(builder.binary(builder.ident("n"), BinaryOp::Greater, builder.int(0))),
        builder.string("positive"),
    );

    if let Pattern::Identifier(name) = arm.pattern {
        assert_eq!(name, "n");
    } else {
        panic!("Expected identifier pattern");
    }

    assert!(arm.guard.is_some());
    if let Some(guard) = arm.guard {
        if let ExprKind::Binary { op, .. } = guard.kind {
            assert!(matches!(op, BinaryOp::Greater));
        } else {
            panic!("Expected binary guard expression");
        }
    }

    if let ExprKind::Literal(Literal::String(val)) = arm.body.kind {
        assert_eq!(val, "positive");
    } else {
        panic!("Expected string body");
    }
}

/// Test string interpolation
#[test]
fn test_string_interpolation() {
    let builder = AstBuilder::new();

    // Create string parts for "Hello {name}!"
    let parts = vec![
        StringPart::Text("Hello ".to_string()),
        StringPart::Expr(Box::new(builder.ident("name"))),
        StringPart::Text("!".to_string()),
    ];

    let interpolation = builder.string_interpolation(parts);

    if let ExprKind::StringInterpolation { parts } = interpolation.kind {
        assert_eq!(parts.len(), 3);

        // First part: "Hello "
        if let StringPart::Text(text) = &parts[0] {
            assert_eq!(text, "Hello ");
        } else {
            panic!("Expected text part");
        }

        // Second part: {name}
        if let StringPart::Expr(expr) = &parts[1] {
            if let ExprKind::Identifier(name) = &expr.kind {
                assert_eq!(name, "name");
            } else {
                panic!("Expected identifier expression");
            }
        } else {
            panic!("Expected expression part");
        }

        // Third part: "!"
        if let StringPart::Text(text) = &parts[2] {
            assert_eq!(text, "!");
        } else {
            panic!("Expected text part");
        }
    } else {
        panic!("Expected string interpolation");
    }
}

/// Test edge cases and boundary conditions
#[test]
fn test_empty_collections() {
    let builder = AstBuilder::new();

    // Empty list
    let empty_list = builder.list(vec![]);
    if let ExprKind::List(elements) = empty_list.kind {
        assert!(elements.is_empty());
    } else {
        panic!("Expected empty list");
    }

    // Empty tuple
    let empty_tuple = builder.tuple(vec![]);
    if let ExprKind::Tuple(elements) = empty_tuple.kind {
        assert!(elements.is_empty());
    } else {
        panic!("Expected empty tuple");
    }

    // Empty block
    let empty_block = builder.block(vec![]);
    if let ExprKind::Block(statements) = empty_block.kind {
        assert!(statements.is_empty());
    } else {
        panic!("Expected empty block");
    }
}

#[test]
fn test_extreme_values() {
    let builder = AstBuilder::new();

    // Test extreme integer values
    let max_int = builder.int(i64::MAX);
    if let ExprKind::Literal(Literal::Integer(val, None)) = max_int.kind {
        assert_eq!(val, i64::MAX);
    } else {
        panic!("Expected max integer");
    }

    let min_int = builder.int(i64::MIN);
    if let ExprKind::Literal(Literal::Integer(val, None)) = min_int.kind {
        assert_eq!(val, i64::MIN);
    } else {
        panic!("Expected min integer");
    }

    // Test extreme float values
    let infinity = builder.float(f64::INFINITY);
    if let ExprKind::Literal(Literal::Float(val)) = infinity.kind {
        assert!(val.is_infinite());
        assert!(val.is_sign_positive());
    } else {
        panic!("Expected positive infinity");
    }

    let neg_infinity = builder.float(f64::NEG_INFINITY);
    if let ExprKind::Literal(Literal::Float(val)) = neg_infinity.kind {
        assert!(val.is_infinite());
        assert!(val.is_sign_negative());
    } else {
        panic!("Expected negative infinity");
    }

    let nan = builder.float(f64::NAN);
    if let ExprKind::Literal(Literal::Float(val)) = nan.kind {
        assert!(val.is_nan());
    } else {
        panic!("Expected NaN");
    }
}

#[test]
fn test_unicode_strings() {
    let builder = AstBuilder::new();

    // Test various Unicode characters
    let unicode_str = builder.string("Hello 世界 🌍 café naïve");
    if let ExprKind::Literal(Literal::String(val)) = unicode_str.kind {
        assert_eq!(val, "Hello 世界 🌍 café naïve");
        assert!(val.contains('世'));
        assert!(val.contains('🌍'));
        assert!(val.contains('ï'));
    } else {
        panic!("Expected Unicode string");
    }

    // Empty string
    let empty_str = builder.string("");
    if let ExprKind::Literal(Literal::String(val)) = empty_str.kind {
        assert!(val.is_empty());
    } else {
        panic!("Expected empty string");
    }

    // String with special characters
    let special_str = builder.string("Line1\nLine2\tTabbed\"Quoted\"");
    if let ExprKind::Literal(Literal::String(val)) = special_str.kind {
        assert!(val.contains('\n'));
        assert!(val.contains('\t'));
        assert!(val.contains('"'));
    } else {
        panic!("Expected string with special characters");
    }
}

#[test]
fn test_nested_expressions() {
    let builder = AstBuilder::new();

    // Deeply nested binary operations: ((1 + 2) * 3) - 4
    let nested = builder.binary(
        builder.binary(
            builder.binary(builder.int(1), BinaryOp::Add, builder.int(2)),
            BinaryOp::Multiply,
            builder.int(3),
        ),
        BinaryOp::Subtract,
        builder.int(4),
    );

    if let ExprKind::Binary { left, op, right } = nested.kind {
        assert!(matches!(op, BinaryOp::Subtract));
        assert!(matches!(left.kind, ExprKind::Binary { .. }));
        if let ExprKind::Literal(Literal::Integer(val, None)) = right.kind {
            assert_eq!(val, 4);
        } else {
            panic!("Expected integer 4");
        }
    } else {
        panic!("Expected nested binary expression");
    }
}

#[test]
fn test_complex_pattern_combinations() {
    let builder = AstBuilder::new();

    // Complex nested pattern: (Some(Point { x, y: 0 }), _)
    let complex_pattern = builder.pattern_tuple(vec![
        builder.pattern_literal(Literal::Integer(1, None)), // Simplified for testing
        builder.pattern_wildcard(),
    ]);

    if let Pattern::Tuple(patterns) = complex_pattern {
        assert_eq!(patterns.len(), 2);
        assert!(matches!(
            patterns[0],
            Pattern::Literal(Literal::Integer(1, None))
        ));
        assert!(matches!(patterns[1], Pattern::Wildcard));
    } else {
        panic!("Expected tuple pattern");
    }
}

/// Test builder consistency and invariants
#[test]
fn test_builder_default_values() {
    let builder = AstBuilder::new();
    let default_builder = AstBuilder::default();

    // Both should create identical spans
    assert_eq!(builder.span, default_builder.span);
    assert_eq!(builder.span, Span::default());

    // All expressions should have consistent default values
    let expr = builder.int(42);
    assert!(expr.attributes.is_empty());
    assert_eq!(expr.span, Span::default());
}

#[test]
fn test_span_consistency() {
    let builder = AstBuilder::new();

    // All created expressions should have the same span as builder
    let int_expr = builder.int(1);
    let str_expr = builder.string("test");
    let bool_expr = builder.bool(true);
    let ident_expr = builder.ident("var");

    assert_eq!(int_expr.span, builder.span);
    assert_eq!(str_expr.span, builder.span);
    assert_eq!(bool_expr.span, builder.span);
    assert_eq!(ident_expr.span, builder.span);

    // Types should also have consistent spans
    let int_type = builder.type_int();
    assert_eq!(int_type.span, builder.span);
}

/// Test AST builder transpilation integration
#[test]
fn test_transpilation_integration_basic() {
    let builder = AstBuilder::new();
    let mut transpiler = Transpiler::new();

    // Test that all basic expressions can be transpiled
    let expressions = vec![
        builder.int(42),
        builder.float(3.15),
        builder.string("hello"),
        builder.bool(true),
        builder.ident("variable"),
    ];

    for expr in expressions {
        let result = transpiler.transpile(&expr);
        assert!(result.is_ok(), "Failed to transpile: {:?}", expr.kind);
    }
}

#[test]
fn test_transpilation_integration_complex() {
    let builder = AstBuilder::new();
    let mut transpiler = Transpiler::new();

    // Test complex expression: fibonacci-style recursive structure
    let complex_expr = builder.if_expr(
        builder.binary(builder.ident("n"), BinaryOp::LessEqual, builder.int(1)),
        builder.ident("n"),
        Some(builder.binary(
            builder.call(
                builder.ident("fib"),
                vec![builder.binary(builder.ident("n"), BinaryOp::Subtract, builder.int(1))],
            ),
            BinaryOp::Add,
            builder.call(
                builder.ident("fib"),
                vec![builder.binary(builder.ident("n"), BinaryOp::Subtract, builder.int(2))],
            ),
        )),
    );

    let result = transpiler.transpile(&complex_expr);
    assert!(result.is_ok(), "Failed to transpile complex expression");
}

#[test]
fn test_transpilation_integration_collections() {
    let builder = AstBuilder::new();
    let mut transpiler = Transpiler::new();

    // Test collection transpilation
    let list_expr = builder.list(vec![builder.int(1), builder.int(2), builder.int(3)]);

    let result = transpiler.transpile(&list_expr);
    assert!(result.is_ok(), "Failed to transpile list");

    // Test tuple transpilation
    let tuple_expr = builder.tuple(vec![
        builder.string("first"),
        builder.int(42),
        builder.bool(true),
    ]);

    let result = transpiler.transpile(&tuple_expr);
    assert!(result.is_ok(), "Failed to transpile tuple");
}

#[test]
fn test_all_binary_operators() {
    let builder = AstBuilder::new();

    let operators = vec![
        BinaryOp::Add,
        BinaryOp::Subtract,
        BinaryOp::Multiply,
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
    ];

    for op in operators {
        let expr = builder.binary(builder.int(1), op, builder.int(2));

        if let ExprKind::Binary { op: actual_op, .. } = expr.kind {
            // Using discriminant comparison since BinaryOp doesn't derive PartialEq
            assert_eq!(
                std::mem::discriminant(&actual_op),
                std::mem::discriminant(&op),
                "Operator mismatch"
            );
        } else {
            panic!("Expected binary expression with operator {op:?}");
        }
    }
}

#[test]
fn test_all_unary_operators() {
    let builder = AstBuilder::new();

    let operators = vec![UnaryOp::Negate, UnaryOp::Not];

    for op in operators {
        let expr = builder.unary(op, builder.int(42));

        if let ExprKind::Unary { op: actual_op, .. } = expr.kind {
            assert_eq!(
                std::mem::discriminant(&actual_op),
                std::mem::discriminant(&op),
                "Operator mismatch"
            );
        } else {
            panic!("Expected unary expression with operator {op:?}");
        }
    }
}
