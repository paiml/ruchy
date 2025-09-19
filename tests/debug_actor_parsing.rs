// Debug test to understand actor parsing issues

use ruchy::frontend::parser::Parser;

#[test]
fn debug_simple_actor_parsing() {
    let input = r#"
        actor TestActor {
            value: i32
        }
    "#;
    let mut parser = Parser::new(input);
    let result = parser.parse();

    match result {
        Ok(expr) => {
            println!("Parsed successfully: {:?}", expr.kind);
        }
        Err(err) => {
            println!("Parse error: {:?}", err);
            panic!("Failed to parse: {:?}", err);
        }
    }
}

#[test]
fn debug_state_block_actor() {
    let input = r#"
        actor CounterActor {
            state {
                count: i32,
                max: i32
            }
        }
    "#;
    let mut parser = Parser::new(input);
    let result = parser.parse();

    match result {
        Ok(expr) => {
            println!("State block actor parsed: {:?}", expr.kind);
        }
        Err(err) => {
            println!("State block error: {:?}", err);
            panic!("Failed to parse state block: {:?}", err);
        }
    }
}