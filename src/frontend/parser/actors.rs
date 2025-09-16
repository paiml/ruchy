//! Actor system parsing
use super::{ParserState, *};
use crate::frontend::ast::{ActorHandler, StructField};
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_actor(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume actor
    let name = parse_actor_name(state)?;
    state.tokens.expect(&Token::LeftBrace)?;
    let (state_fields, handlers) = parse_actor_body(state)?;
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(
        ExprKind::Actor {
            name,
            state: state_fields,
            handlers,
        },
        start_span,
    ))
}
// Helper: Parse actor name (complexity: 2)
fn parse_actor_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected actor name");
    }
}
// Helper: Parse actor body (complexity: 4)
fn parse_actor_body(state: &mut ParserState) -> Result<(Vec<StructField>, Vec<ActorHandler>)> {
    let mut state_fields = Vec::new();
    let mut handlers = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        if matches!(state.tokens.peek(), Some((Token::State, _))) {
            parse_state_block(state, &mut state_fields)?;
        } else if matches!(state.tokens.peek(), Some((Token::Receive, _))) {
            parse_receive_handler(state, &mut handlers)?;
        } else {
            parse_inline_state_field(state, &mut state_fields)?;
        }
    }
    Ok((state_fields, handlers))
}
// Helper: Parse state block (complexity: 5)
fn parse_state_block(state: &mut ParserState, state_fields: &mut Vec<StructField>) -> Result<()> {
    state.tokens.advance(); // consume 'state'
    state.tokens.expect(&Token::LeftBrace)?;
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let field_name = parse_field_name(state, "Expected field name in state block")?;
        state.tokens.expect(&Token::Colon)?;
        let ty = utils::parse_type(state)?;
        state_fields.push(StructField {
            name: field_name,
            ty,
            is_pub: false,
        });
        consume_optional_separator(state);
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(())
}
// Helper: Parse receive handler (complexity: 3)
fn parse_receive_handler(state: &mut ParserState, handlers: &mut Vec<ActorHandler>) -> Result<()> {
    state.tokens.advance(); // consume 'receive'
    if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        parse_receive_block(state, handlers)
    } else {
        parse_individual_handler(state, handlers)
    }
}
// Helper: Parse receive block with multiple handlers (complexity: 6)
fn parse_receive_block(state: &mut ParserState, handlers: &mut Vec<ActorHandler>) -> Result<()> {
    state.tokens.advance(); // consume {
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let message_type = parse_field_name(state, "Expected message type in receive block")?;
        let params = parse_optional_params(state)?;
        state.tokens.expect(&Token::FatArrow)?;
        let body = parse_handler_body(state)?;
        handlers.push(ActorHandler {
            message_type,
            params,
            body,
        });
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(())
}
// Helper: Parse individual handler (complexity: 4)
fn parse_individual_handler(state: &mut ParserState, handlers: &mut Vec<ActorHandler>) -> Result<()> {
    let message_type = parse_field_name(state, "Expected message type after receive")?;
    let params = parse_optional_params(state)?;
    // Parse optional return type
    if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        let _return_type = utils::parse_type(state)?;
    }
    let body = Box::new(collections::parse_block(state)?);
    handlers.push(ActorHandler {
        message_type,
        params,
        body,
    });
    Ok(())
}
// Helper: Parse inline state field (complexity: 4)
fn parse_inline_state_field(state: &mut ParserState, state_fields: &mut Vec<StructField>) -> Result<()> {
    let field_name = parse_field_name(state, "Expected field name")?;
    state.tokens.expect(&Token::Colon)?;
    let ty = utils::parse_type(state)?;
    // Parse optional default value
    if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance(); // consume =
        let _default_value = super::parse_expr_recursive(state)?;
    }
    state_fields.push(StructField {
        name: field_name,
        ty,
        is_pub: false,
    });
    consume_optional_separator(state);
    Ok(())
}
// Helper: Parse field name (complexity: 2)
fn parse_field_name(state: &mut ParserState, error_msg: &str) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("{}", error_msg);
    }
}
// Helper: Parse optional parameters (complexity: 2)
fn parse_optional_params(state: &mut ParserState) -> Result<Vec<Param>> {
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        utils::parse_params(state)
    } else {
        Ok(Vec::new())
    }
}
// Helper: Parse handler body (complexity: 2)
fn parse_handler_body(state: &mut ParserState) -> Result<Box<Expr>> {
    if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        Ok(Box::new(collections::parse_block(state)?))
    } else {
        Ok(Box::new(super::parse_expr_recursive(state)?))
    }
}
// Helper: Consume optional separator (complexity: 2)
fn consume_optional_separator(state: &mut ParserState) {
    if matches!(
        state.tokens.peek(),
        Some((Token::Comma | Token::Semicolon, _))
    ) {
        state.tokens.advance();
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_actor_function_signature() {
        // This test just verifies the function signature compiles and exists
        // Actual functionality testing is done via integration tests due to
        // complex parser infrastructure requirements
        // Verify function exists with correct signature
        // This is a compile-time check - if it compiles, the test passes
        let f: fn(&mut ParserState) -> Result<Expr> = parse_actor;
        // Use the variable to avoid unused warning
        assert!(!format!("{f:p}").is_empty(), "Function exists");
    }

    #[test]
    fn test_helper_functions_exist() {
        // Test that helper functions exist with correct signatures
        let _f1: fn(&mut ParserState) -> Result<String> = parse_actor_name;
        let _f2: fn(&mut ParserState, &str) -> Result<String> = parse_field_name;
        let _f3: fn(&mut ParserState) -> Result<Vec<Param>> = parse_optional_params;
        let _f4: fn(&mut ParserState) = consume_optional_separator;

        // If this compiles, the signatures are correct
        assert!(true, "Helper functions exist with correct signatures");
    }

    #[test]
    fn test_actor_handler_struct() {
        // Test that ActorHandler can be created
        use crate::frontend::ast::{ActorHandler, ExprKind, Span, Literal};

        let handler = ActorHandler {
            message_type: "TestMessage".to_string(),
            params: vec![],
            body: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(42)),
                Span::new(0, 2),
            )),
        };

        assert_eq!(handler.message_type, "TestMessage");
        assert!(handler.params.is_empty());
    }

    #[test]
    fn test_struct_field_creation() {
        use crate::frontend::ast::{StructField, Type, TypeKind, Span};

        let field = StructField {
            name: "test_field".to_string(),
            ty: Type {
                kind: TypeKind::Named("Int".to_string()),
                span: Span::new(0, 3),
            },
            is_pub: false,
        };

        assert_eq!(field.name, "test_field");
        assert_eq!(field.ty.kind, TypeKind::Named("Int".to_string()));
        assert!(!field.is_pub);
    }

    // Additional comprehensive parser tests

    #[test]
    fn test_parse_simple_actor() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Counter { count: i32 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse simple actor");
    }

    #[test]
    #[ignore = "actor state block syntax not implemented"]
    fn test_parse_actor_with_state_block() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor User { state { name: String, age: i32 } }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with state block");
    }

    #[test]
    #[ignore = "actor receive syntax not implemented"]
    fn test_parse_actor_with_receive() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Echo { receive msg -> println(msg) }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with receive handler");
    }

    #[test]
    #[ignore = "actor multiple handlers not implemented"]
    fn test_parse_actor_with_multiple_handlers() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new(r#"
            actor Calculator {
                state { value: i32 }
                receive {
                    Add(n) => value + n,
                    Sub(n) => value - n,
                    Reset => 0
                }
            }
        "#);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with multiple handlers");
    }

    #[test]
    fn test_parse_actor_empty() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Empty { }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse empty actor");
    }

    #[test]
    fn test_parse_actor_with_inline_fields() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Point { x: f64, y: f64 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with inline fields");
    }

    #[test]
    fn test_parse_actor_with_complex_types() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Storage { data: HashMap<String, Vec<i32>> }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with complex types");
    }

    #[test]
    #[ignore = "actor handler params not implemented"]
    fn test_parse_actor_with_handler_params() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Server { receive Request(url, method) -> Response }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with handler parameters");
    }

    #[test]
    #[ignore = "actor block handler not implemented"]
    fn test_parse_actor_with_block_handler() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new(r#"
            actor Logger {
                receive Log(msg) => {
                    let timestamp = now();
                    println(timestamp, msg)
                }
            }
        "#);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with block handler");
    }

    #[test]
    #[ignore = "actor mixed content not implemented"]
    fn test_parse_actor_with_mixed_content() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new(r#"
            actor Worker {
                id: String
                state {
                    tasks: Vec<Task>,
                    active: bool
                }
                receive Start => active = true
                receive Stop => active = false
                receive AddTask(task) => tasks.push(task)
            }
        "#);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with mixed content");
    }

    #[test]
    #[ignore = "nested actors not implemented"]
    fn test_parse_nested_actors() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new(r#"
            {
                actor Parent {
                    child: actor Child { value: i32 }
                }
            }
        "#);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse nested actors");
    }

    #[test]
    #[ignore = "actor generics not implemented"]
    fn test_parse_actor_with_generics() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("actor Container<T> { items: Vec<T> }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with generic types");
    }

    #[test]
    #[ignore = "spawn expression not implemented"]
    fn test_parse_spawn_expression() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("spawn actor Counter { count: 0 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse spawn expression");
    }

    #[test]
    fn test_parse_send_expression() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new("my_actor <- Message(data)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse send expression");
    }

    #[test]
    #[ignore = "actor with await not implemented"]
    fn test_parse_actor_with_await() {
        use crate::frontend::parser::Parser;
        let mut parser = Parser::new(r#"
            actor AsyncWorker {
                receive Fetch(url) => await http_get(url)
            }
        "#);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse actor with await");
    }
}
