// TDD Functional Tests for REPL based on SPECIFICATION and demos
// These tests define what SHOULD work based on the language spec

use ruchy::runtime::Repl;
use std::env;

#[test]
fn test_basic_arithmetic_from_demos() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // From demo_01_arithmetic.repl
    assert_eq!(repl.eval("2 + 2").unwrap(), "4");
    assert_eq!(repl.eval("10 * 5").unwrap(), "50");
    assert_eq!(repl.eval("100 - 25").unwrap(), "75");
    assert_eq!(repl.eval("50 / 2").unwrap(), "25");
    assert_eq!(repl.eval("17 % 5").unwrap(), "2");
    assert_eq!(repl.eval("2 ** 8").unwrap(), "256"); // Power operator
}

#[test]
fn test_variable_assignment() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Basic variable assignment
    assert!(repl.eval("let x = 5").is_ok());
    assert_eq!(repl.eval("x").unwrap(), "5");
    
    // Using variables in expressions
    assert!(repl.eval("let y = x * 2").is_ok());
    assert_eq!(repl.eval("y").unwrap(), "10");
    
    // String variables
    assert!(repl.eval("let name = \"Ruchy\"").is_ok());
    assert_eq!(repl.eval("name").unwrap(), "\"Ruchy\"");
}

#[test]
fn test_mutable_variables() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Mutable variable declaration
    assert!(repl.eval("let mut x = 5").is_ok());
    assert_eq!(repl.eval("x").unwrap(), "5");
    
    // Mutation
    assert!(repl.eval("x = 10").is_ok());
    assert_eq!(repl.eval("x").unwrap(), "10");
}

#[test]
fn test_string_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // String concatenation
    assert_eq!(repl.eval("\"Hello\" + \" World\"").unwrap(), "\"Hello World\"");
    
    // String methods
    assert_eq!(repl.eval("\"hello\".length()").unwrap(), "5");
    assert_eq!(repl.eval("\"hello\".to_upper()").unwrap(), "\"HELLO\"");
    assert_eq!(repl.eval("\"HELLO\".to_lower()").unwrap(), "\"hello\"");
}

#[test]
fn test_arrays() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Array literal
    assert_eq!(repl.eval("[1, 2, 3, 4, 5]").unwrap(), "[1, 2, 3, 4, 5]");
    
    // Array indexing
    assert!(repl.eval("let arr = [10, 20, 30]").is_ok());
    assert_eq!(repl.eval("arr[0]").unwrap(), "10");
    assert_eq!(repl.eval("arr[2]").unwrap(), "30");
    
    // Array methods
    assert_eq!(repl.eval("arr.length()").unwrap(), "3");
}

#[test]
fn test_boolean_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Boolean literals
    assert_eq!(repl.eval("true").unwrap(), "true");
    assert_eq!(repl.eval("false").unwrap(), "false");
    
    // Boolean operators
    assert_eq!(repl.eval("true && false").unwrap(), "false");
    assert_eq!(repl.eval("true || false").unwrap(), "true");
    assert_eq!(repl.eval("!true").unwrap(), "false");
    assert_eq!(repl.eval("!false").unwrap(), "true");
}

#[test]
fn test_comparison_operators() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("5 > 3").unwrap(), "true");
    assert_eq!(repl.eval("5 < 3").unwrap(), "false");
    assert_eq!(repl.eval("5 >= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 <= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 == 5").unwrap(), "true");
    assert_eq!(repl.eval("5 != 3").unwrap(), "true");
}

#[test]
fn test_if_else() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(repl.eval("if true { 10 } else { 20 }").unwrap(), "10");
    assert_eq!(repl.eval("if false { 10 } else { 20 }").unwrap(), "20");
    
    // With conditions
    assert_eq!(repl.eval("if 5 > 3 { \"yes\" } else { \"no\" }").unwrap(), "\"yes\"");
}

#[test]
fn test_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Function declaration
    assert!(repl.eval("fn add(a, b) { a + b }").is_ok());
    assert_eq!(repl.eval("add(3, 4)").unwrap(), "7");
    
    // Function with string
    assert!(repl.eval("fn greet(name) { \"Hello, \" + name }").is_ok());
    assert_eq!(repl.eval("greet(\"World\")").unwrap(), "\"Hello, World\"");
}

#[test]
fn test_lambda_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Lambda assignment
    assert!(repl.eval("let add = fn(a, b) { a + b }").is_ok());
    assert_eq!(repl.eval("add(10, 20)").unwrap(), "30");
    
    // Arrow function syntax
    assert!(repl.eval("let double = x => x * 2").is_ok());
    assert_eq!(repl.eval("double(5)").unwrap(), "10");
}

#[test]
fn test_for_loops() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let mut sum = 0").is_ok());
    assert!(repl.eval("for i in [1, 2, 3, 4, 5] { sum = sum + i }").is_ok());
    assert_eq!(repl.eval("sum").unwrap(), "15");
    
    // Range-based for loop
    assert!(repl.eval("let mut count = 0").is_ok());
    assert!(repl.eval("for i in 1..5 { count = count + 1 }").is_ok());
    assert_eq!(repl.eval("count").unwrap(), "4"); // 1..5 is exclusive
}

#[test]
fn test_while_loops() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let mut x = 0").is_ok());
    assert!(repl.eval("while x < 5 { x = x + 1 }").is_ok());
    assert_eq!(repl.eval("x").unwrap(), "5");
}

#[test]
fn test_match_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert_eq!(
        repl.eval("match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }").unwrap(),
        "\"two\""
    );
    
    // Pattern matching with guards
    assert!(repl.eval("let x = 10").is_ok());
    assert_eq!(
        repl.eval("match x { n if n > 5 => \"big\", _ => \"small\" }").unwrap(),
        "\"big\""
    );
}

#[test]
fn test_objects() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Object literal
    assert!(repl.eval("let person = { name: \"Alice\", age: 30 }").is_ok());
    assert_eq!(repl.eval("person.name").unwrap(), "\"Alice\"");
    assert_eq!(repl.eval("person.age").unwrap(), "30");
}

#[test]
fn test_destructuring() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Array destructuring
    assert!(repl.eval("let [a, b, c] = [1, 2, 3]").is_ok());
    assert_eq!(repl.eval("a").unwrap(), "1");
    assert_eq!(repl.eval("b").unwrap(), "2");
    assert_eq!(repl.eval("c").unwrap(), "3");
    
    // Object destructuring
    assert!(repl.eval("let { x, y } = { x: 10, y: 20 }").is_ok());
    assert_eq!(repl.eval("x").unwrap(), "10");
    assert_eq!(repl.eval("y").unwrap(), "20");
}

#[test]
fn test_spread_operator() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let arr1 = [1, 2, 3]").is_ok());
    assert!(repl.eval("let arr2 = [0, ...arr1, 4]").is_ok());
    assert_eq!(repl.eval("arr2").unwrap(), "[0, 1, 2, 3, 4]");
}

#[test]
fn test_pipe_operator() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("fn double(x) { x * 2 }").is_ok());
    assert!(repl.eval("fn add_one(x) { x + 1 }").is_ok());
    
    // Pipe operator chains
    assert_eq!(repl.eval("5 |> double").unwrap(), "10");
    assert_eq!(repl.eval("5 |> double |> add_one").unwrap(), "11");
}

#[test]
fn test_string_interpolation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let name = \"World\"").is_ok());
    assert_eq!(repl.eval("f\"Hello {name}\"").unwrap(), "\"Hello World\"");
    
    assert!(repl.eval("let x = 42").is_ok());
    assert_eq!(repl.eval("f\"The answer is {x}\"").unwrap(), "\"The answer is 42\"");
}

#[test]
fn test_try_catch() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Try-catch for division by zero
    assert_eq!(
        repl.eval("try { 10 / 0 } catch { \"error\" }").unwrap(),
        "\"error\""
    );
    
    // Successful try
    assert_eq!(
        repl.eval("try { 10 / 2 } catch { \"error\" }").unwrap(),
        "5"
    );
}

#[test]
fn test_async_await() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Async function definition
    assert!(repl.eval("async fn fetch_data() { 42 }").is_ok());
    assert_eq!(repl.eval("await fetch_data()").unwrap(), "42");
}

#[test]
fn test_generics() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Generic function
    assert!(repl.eval("fn identity<T>(x: T) -> T { x }").is_ok());
    assert_eq!(repl.eval("identity(42)").unwrap(), "42");
    assert_eq!(repl.eval("identity(\"hello\")").unwrap(), "\"hello\"");
}

#[test]
fn test_enums() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("enum Color { Red, Green, Blue }").is_ok());
    assert!(repl.eval("let c = Color::Red").is_ok());
    assert_eq!(repl.eval("c").unwrap(), "Color::Red");
}

#[test]
fn test_structs() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("struct Point { x, y }").is_ok());
    assert!(repl.eval("let p = Point { x: 10, y: 20 }").is_ok());
    assert_eq!(repl.eval("p.x").unwrap(), "10");
    assert_eq!(repl.eval("p.y").unwrap(), "20");
}

#[test]
fn test_factorial_calculation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Recursive factorial
    assert!(repl.eval("fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }").is_ok());
    assert_eq!(repl.eval("factorial(5)").unwrap(), "120");
}

#[test]
fn test_fibonacci() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("fn fib(n) { if n <= 1 { n } else { fib(n - 1) + fib(n - 2) } }").is_ok());
    assert_eq!(repl.eval("fib(0)").unwrap(), "0");
    assert_eq!(repl.eval("fib(1)").unwrap(), "1");
    assert_eq!(repl.eval("fib(5)").unwrap(), "5"); // 0, 1, 1, 2, 3, 5
}

#[test]
fn test_closures() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let x = 10").is_ok());
    assert!(repl.eval("let add_x = fn(y) { x + y }").is_ok());
    assert_eq!(repl.eval("add_x(5)").unwrap(), "15");
}

#[test]
fn test_higher_order_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Map function
    assert!(repl.eval("let nums = [1, 2, 3, 4, 5]").is_ok());
    assert!(repl.eval("let doubled = nums.map(x => x * 2)").is_ok());
    assert_eq!(repl.eval("doubled").unwrap(), "[2, 4, 6, 8, 10]");
    
    // Filter function
    assert!(repl.eval("let evens = nums.filter(x => x % 2 == 0)").is_ok());
    assert_eq!(repl.eval("evens").unwrap(), "[2, 4]");
    
    // Reduce function
    assert!(repl.eval("let sum = nums.reduce((a, b) => a + b, 0)").is_ok());
    assert_eq!(repl.eval("sum").unwrap(), "15");
}

#[test]
fn test_range_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    // Inclusive range
    assert_eq!(repl.eval("[...1..=5]").unwrap(), "[1, 2, 3, 4, 5]");
    
    // Exclusive range
    assert_eq!(repl.eval("[...1..5]").unwrap(), "[1, 2, 3, 4]");
}

#[test]
fn test_tuple_operations() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let t = (1, \"hello\", true)").is_ok());
    assert_eq!(repl.eval("t.0").unwrap(), "1");
    assert_eq!(repl.eval("t.1").unwrap(), "\"hello\"");
    assert_eq!(repl.eval("t.2").unwrap(), "true");
}

#[test]
fn test_optional_chaining() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let obj = { a: { b: { c: 42 } } }").is_ok());
    assert_eq!(repl.eval("obj?.a?.b?.c").unwrap(), "42");
    
    assert!(repl.eval("let null_obj = null").is_ok());
    assert_eq!(repl.eval("null_obj?.a?.b?.c").unwrap(), "null");
}

#[test]
fn test_null_coalescing() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    
    assert!(repl.eval("let x = null").is_ok());
    assert_eq!(repl.eval("x ?? 10").unwrap(), "10");
    
    assert!(repl.eval("let y = 5").is_ok());
    assert_eq!(repl.eval("y ?? 10").unwrap(), "5");
}