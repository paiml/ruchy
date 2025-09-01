//! P0-BOOK-006: Advanced Patterns Test Suite
//! 
//! Tests for advanced pattern matching and destructuring features including:
//! - Complex pattern matching with guards
//! - Destructuring assignment
//! - Tuple and array patterns
//! - Object destructuring
//! - Advanced match expressions
//! - Pattern guards and conditions

use ruchy::runtime::Repl;

#[test]
fn test_tuple_destructuring() {
    let code = r#"
let (a, b, c) = (1, 2, 3)
println(f"a={a}, b={b}, c={c}")
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Tuple destructuring should work: {result:?}");
    // The println returns "()" but the actual output goes to stdout
}

#[test]
fn test_array_pattern_matching() {
    let code = r#"
let arr = [1, 2, 3, 4]
match arr {
    [first] => println(f"Single element: {first}")
    _ => println("Multiple elements")
}
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Array pattern matching should work: {result:?}");
}

#[test]
fn test_object_destructuring() {
    let code = r#"
let person = {name: "Alice", age: 30, city: "Paris"}
let {name, age} = person
println(f"Name: {name}, Age: {age}")
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Object destructuring should work: {result:?}");
}

#[test]
fn test_nested_pattern_matching() {
    let code = r#"
let data = {users: [{name: "Bob", score: 85}, {name: "Carol", score: 92}]}
match data {
    {users: users_list} => {
        println(f"Found users: {users_list}")
    }
    _ => println("No users found")
}
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Nested pattern matching should work: {result:?}");
}

#[test]
fn test_pattern_guards() {
    let code = r#"
let value = 42
match value {
    x if x > 50 => println("Large number")
    x if x > 20 => println("Medium number")
    x if x > 0 => println("Small positive number")
    _ => println("Zero or negative")
}
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Pattern guards should work: {result:?}");
}

#[test]
fn test_advanced_match_expressions() {
    let code = r#"
let value = 30
let result = match value {
    x if x > 25 => "Large"
    x if x > 10 => "Medium"
    _ => "Small"
}
println(result)
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Advanced match expressions should work: {result:?}");
}

#[test]
fn test_range_patterns() {
    let code = r#"
let grade = 85
let category = match grade {
    90..=100 => "A"
    80..=89 => "B"
    70..=79 => "C"
    60..=69 => "D"
    _ => "F"
}
println(f"Grade: {category}")
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Range patterns should work: {result:?}");
}

#[test]
fn test_or_patterns() {
    let code = r#"
let day = "Saturday"
let day_type = match day {
    "Monday" | "Tuesday" | "Wednesday" | "Thursday" | "Friday" => "Weekday"
    "Saturday" | "Sunday" => "Weekend"
    _ => "Unknown"
}
println(f"Day type: {day_type}")
"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(code);
    assert!(result.is_ok(), "Or patterns should work: {result:?}");
}