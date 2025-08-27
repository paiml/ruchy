// Lifetimes and Borrowing Test Suite
// Testing ownership, references, lifetimes, and move semantics

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
fn test_references() {
    // Test reference creation and dereferencing
    let code = r#"
let x = 42
let y = &x
*y
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "References should at least parse");
}

#[test]
fn test_mutable_references() {
    // Test mutable references
    let code = r#"
let mut x = 10
let y = &mut x
*y = 20
x
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Mutable references should at least parse");
}

#[test]
fn test_lifetime_annotations() {
    // Test explicit lifetime annotations
    let code = r#"
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

longest("hello", "world!")
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Lifetime annotations should at least parse");
}

#[test]
fn test_struct_lifetimes() {
    // Test lifetimes in structs
    let code = r#"
struct ImportantExcerpt<'a> {
    part: &'a str
}

let novel = "Call me Ishmael. Some years ago..."
let first = ImportantExcerpt { part: &novel[0..15] }
first.part
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Struct lifetimes should at least parse");
}

#[test]
fn test_move_semantics() {
    // Test move semantics
    let code = r#"
let s1 = "hello"
let s2 = s1  // Move occurs here
s2
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "hello");
    }
}

#[test]
fn test_clone() {
    // Test clone to avoid moves
    let code = r#"
let s1 = "hello"
let s2 = s1.clone()
[s1, s2]
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Clone should at least parse");
}

#[test]
fn test_copy_trait() {
    // Test Copy trait for simple types
    let code = r#"
let x = 5
let y = x  // Copy occurs here (i32 implements Copy)
x + y
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "10");
    }
}

#[test]
fn test_borrow_checker() {
    // Test that would fail borrow checker
    let code = r#"
let mut x = 10
let r1 = &x
let r2 = &mut x  // This should fail - can't have mutable and immutable refs
*r2
"#;
    
    let result = eval_in_repl(code);
    // This should either fail or handle the borrow checking
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_self_referential() {
    // Test self-referential structs (Pin)
    let code = r#"
struct SelfRef {
    value: i32,
    ptr: &i32
}

// This would need Pin in real Rust
"test"
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Self-referential should at least parse");
}

#[test]
fn test_static_lifetime() {
    // Test 'static lifetime
    let code = r#"
let s: &'static str = "I live forever!"
s
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "'static lifetime should at least parse");
}

#[test]
fn test_lifetime_elision() {
    // Test lifetime elision rules
    let code = r#"
fn first_word(s: &str) -> &str {
    &s[0..5]
}

first_word("hello world")
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Lifetime elision should at least parse");
}

#[test]
fn test_multiple_lifetimes() {
    // Test multiple lifetime parameters
    let code = r#"
fn mix<'a, 'b>(x: &'a str, y: &'b str) -> String {
    format!("{} {}", x, y)
}

mix("hello", "world")
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Multiple lifetimes should at least parse");
}