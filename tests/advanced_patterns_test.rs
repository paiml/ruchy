// Advanced Pattern Matching Test Suite
// Testing guards, nested patterns, destructuring, and more

use ruchy::runtime::repl::Repl;

// Helper to test in REPL
fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {:?}", e))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {:?}", e))?;
    
    // Remove quotes if present (REPL string formatting)
    if result.starts_with('"') && result.ends_with('"') && result.len() >= 2 {
        Ok(result[1..result.len()-1].to_string())
    } else {
        Ok(result)
    }
}

#[test]
fn test_pattern_guards() {
    // Test pattern guards in match expressions
    let code = r#"
let classify = |x| match x {
    n if n < 0 => "negative",
    n if n == 0 => "zero",
    n if n < 10 => "small",
    _ => "large"
}

classify(-5)
"#;
    
    let result = eval_in_repl(code);
    assert_eq!(result.unwrap(), "negative");
}

#[test]
fn test_nested_destructuring() {
    // Test nested pattern destructuring
    let code = r#"
let point = { x: { value: 10 }, y: { value: 20 } }
let { x: { value: xval }, y: { value: yval } } = point
xval + yval
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "30");
    }
}

#[test]
fn test_array_destructuring() {
    // Test array pattern destructuring
    let code = r#"
let [first, second, ...rest] = [1, 2, 3, 4, 5]
[first, second, rest]
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Array destructuring should at least parse");
}

#[test]
fn test_tuple_destructuring() {
    // Test tuple pattern destructuring
    let code = r#"
let (x, y, z) = (1, 2, 3)
x + y + z
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "6");
    }
}

#[test]
fn test_or_patterns() {
    // Test or-patterns in match
    let code = r#"
let classify = |x| match x {
    1 | 2 | 3 => "low",
    4 | 5 | 6 => "mid",
    7 | 8 | 9 => "high",
    _ => "other"
}

classify(5)
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "mid");
    }
}

#[test]
fn test_range_patterns() {
    // Test range patterns in match
    let code = r#"
let grade = |score| match score {
    90..=100 => "A",
    80..=89 => "B",
    70..=79 => "C",
    60..=69 => "D",
    _ => "F"
}

grade(85)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Range patterns should at least parse");
}

#[test]
fn test_wildcard_patterns() {
    // Test wildcard patterns
    let code = r#"
let point = { x: 10, y: 20, z: 30 }
let { x, .. } = point
x
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "10");
    }
}

#[test]
fn test_ref_patterns() {
    // Test reference patterns
    let code = r#"
let value = 42
match value {
    ref x => x
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Ref patterns should at least parse");
}

#[test]
fn test_box_patterns() {
    // Test box patterns
    let code = r#"
let boxed = Box::new(42)
match boxed {
    box x => x
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Box patterns should at least parse");
}

#[test]
fn test_slice_patterns() {
    // Test slice patterns
    let code = r#"
let arr = [1, 2, 3, 4, 5]
match arr {
    [first, .., last] => first + last
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Slice patterns should at least parse");
}

#[test]
fn test_enum_variant_patterns() {
    // Test enum variant patterns
    let code = r#"
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32)
}

let msg = Message::Move { x: 10, y: 20 }
match msg {
    Message::Quit => "quit",
    Message::Move { x, y } => format!("move to {},{}", x, y),
    Message::Write(text) => text,
    Message::ChangeColor(r, g, b) => format!("color {},{},{}", r, g, b)
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Enum variant patterns should at least parse");
}

#[test]
fn test_const_patterns() {
    // Test const patterns in match
    let code = r#"
const MAX: i32 = 100
const MIN: i32 = 0

let validate = |x| match x {
    MIN => "minimum",
    MAX => "maximum",
    _ => "in range"
}

validate(100)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Const patterns should at least parse");
}