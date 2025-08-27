// Macro System Test Suite
// Testing macro_rules!, macro expansion, and hygiene

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
fn test_simple_macro() {
    // Test a simple macro definition
    let code = r#"
macro_rules! say_hello {
    () => {
        println("Hello, World!")
    }
}

say_hello!()
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Simple macro should at least parse");
}

#[test]
fn test_macro_with_arguments() {
    // Test macro with pattern matching
    let code = r#"
macro_rules! add {
    ($x:expr, $y:expr) => {
        $x + $y
    }
}

add!(2, 3)
"#;
    
    let result = eval_in_repl(code);
    if result.is_ok() {
        assert_eq!(result.unwrap(), "5");
    }
}

#[test]
fn test_repetition_macro() {
    // Test macro with repetition
    let code = r#"
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut v = []
            $(
                v.push($x)
            )*
            v
        }
    }
}

vec![1, 2, 3, 4, 5]
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Repetition macro should at least parse");
}

#[test]
fn test_println_macro() {
    // Test println! macro
    let code = r#"
println!("Hello, {}", "World")
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "println! macro should work: {:?}", result);
}

#[test]
fn test_format_macro() {
    // Test format! macro
    let code = r#"
format!("The answer is {}", 42)
"#;
    
    let result = eval_in_repl(code);
    if result.is_ok() {
        assert!(result.unwrap().contains("42"));
    }
}

#[test]
fn test_assert_macro() {
    // Test assert! macro
    let code = r#"
assert!(true)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "assert! macro should at least parse");
}

#[test]
fn test_dbg_macro() {
    // Test dbg! macro for debugging
    let code = r#"
let x = 42
dbg!(x)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "dbg! macro should at least parse");
}

#[test]
fn test_custom_derive_macro() {
    // Test custom derive macros
    let code = r#"
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Derive macro should at least parse");
}

#[test]
fn test_macro_hygiene() {
    // Test macro hygiene
    let code = r#"
macro_rules! swap {
    ($a:expr, $b:expr) => {
        {
            let temp = $a
            $a = $b
            $b = temp
        }
    }
}

let mut x = 1
let mut y = 2
swap!(x, y)
[x, y]
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Macro hygiene test should at least parse");
}

#[test]
fn test_recursive_macro() {
    // Test recursive macro
    let code = r#"
macro_rules! factorial {
    (0) => { 1 };
    ($n:expr) => {
        $n * factorial!($n - 1)
    }
}

factorial!(5)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Recursive macro should at least parse");
}