/// ERROR-004: TDD test for while loop followed by let statement
///
/// This isolates the specific parser issue causing "Expected RightBrace, found Let"
use ruchy::frontend::Parser;

fn parse(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    parser
        .parse()
        .map(|_| "OK".to_string())
        .map_err(|e| e.to_string())
}

#[test]
fn test_while_alone_in_function() {
    let code = r#"
        fun test() -> i32 {
            let mut i = 0;
            while i < 5 {
                i = i + 1;
            }
            i
        }
        test()
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "While loop alone should parse: {:?}",
        result
    );
}

#[test]
fn test_while_followed_by_let() {
    let code = r#"
        fun test() -> i32 {
            let mut i = 0;
            while i < 5 {
                i = i + 1;
            }
            let result = i;
            result
        }
        test()
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "While followed by let should parse: {:?}",
        result
    );
}

#[test]
fn test_while_with_let_inside() {
    let code = r#"
        fun test() -> i32 {
            let mut i = 0;
            while i < 5 {
                let temp = i + 1;
                i = temp;
            }
            i
        }
        test()
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "While with let inside should parse: {:?}",
        result
    );
}

#[test]
fn test_while_with_let_inside_and_after() {
    let code = r#"
        fun test() -> i32 {
            let mut result = 0;
            let mut i = 0;
            while i < 5 {
                let temp = i * 2;
                result = result + temp;
                i = i + 1;
            }
            let final_value = result;
            final_value
        }
        test()
    "#;

    let result = parse(code);
    assert!(
        result.is_ok(),
        "While with let inside and after should parse: {:?}",
        result
    );
}
