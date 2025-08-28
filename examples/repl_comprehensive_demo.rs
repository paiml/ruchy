// Example: Comprehensive REPL demo showing all working features
// Run with: cargo run --example repl_comprehensive_demo

use ruchy::runtime::Repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new()?;
    
    println!("=== Ruchy REPL Comprehensive Demo ===\n");
    println!("This demo shows all the working features of the Ruchy REPL.\n");
    
    // Section 1: Basic Operations
    println!("1. BASIC OPERATIONS");
    println!("===================");
    run_section(&mut repl, vec![
        ("2 + 2", "Addition"),
        ("10 * 5", "Multiplication"),
        ("100 - 25", "Subtraction"),
        ("50 / 2", "Division"),
        ("17 % 5", "Modulo"),
        ("2 ** 8", "Power"),
        ("true && false", "Boolean AND"),
        ("true || false", "Boolean OR"),
        ("!true", "Boolean NOT"),
        ("5 > 3", "Greater than"),
        ("5 == 5", "Equality"),
        ("5 != 3", "Inequality"),
    ]);
    
    // Section 2: Variables
    println!("\n2. VARIABLES");
    println!("============");
    run_section(&mut repl, vec![
        ("let x = 42", "Variable declaration"),
        ("x", "Variable access"),
        ("let name = \"Ruchy\"", "String variable"),
        ("name", "String access"),
        ("let numbers = [1, 2, 3, 4, 5]", "Array variable"),
        ("numbers", "Array access"),
        ("numbers[0]", "Array indexing"),
    ]);
    
    // Section 3: Control Flow
    println!("\n3. CONTROL FLOW");
    println!("===============");
    run_section(&mut repl, vec![
        ("if true { \"yes\" } else { \"no\" }", "If-else true"),
        ("if false { \"yes\" } else { \"no\" }", "If-else false"),
        ("if 5 > 3 { \"bigger\" } else { \"smaller\" }", "If with condition"),
        ("match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }", "Pattern matching"),
    ]);
    
    // Section 4: Functions
    println!("\n4. FUNCTIONS");
    println!("============");
    run_section(&mut repl, vec![
        ("fn add(a, b) { a + b }", "Function definition"),
        ("add(3, 4)", "Function call"),
        ("fn greet(name) { \"Hello, \" + name }", "String function"),
        ("greet(\"World\")", "String function call"),
        ("let double = fn(x) { x * 2 }", "Lambda function"),
        ("double(21)", "Lambda call"),
        ("let triple = x => x * 3", "Arrow function"),
        ("triple(7)", "Arrow function call"),
    ]);
    
    // Section 5: Recursion
    println!("\n5. RECURSION");
    println!("============");
    run_section(&mut repl, vec![
        ("fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }", "Factorial definition"),
        ("factorial(5)", "5! = 120"),
        ("factorial(0)", "0! = 1"),
        ("fn fib(n) { if n <= 1 { n } else { fib(n - 1) + fib(n - 2) } }", "Fibonacci definition"),
        ("fib(10)", "10th Fibonacci number"),
    ]);
    
    // Section 6: Data Structures
    println!("\n6. DATA STRUCTURES");
    println!("==================");
    run_section(&mut repl, vec![
        ("[1, 2, 3, 4, 5]", "Array literal"),
        ("let obj = { x: 10, y: 20 }", "Object literal"),
        ("obj", "Object display"),
        ("obj.x", "Object field access"),
        ("[1, 2, 3].length()", "Array length"),
    ]);
    
    // Section 7: Loops
    println!("\n7. LOOPS");
    println!("========");
    run_section(&mut repl, vec![
        ("let sum = 0", "Initialize sum"),
        ("for i in [1, 2, 3] { sum = sum + i }", "For loop"),
        ("sum", "Sum after for loop"),
        ("let count = 0", "Initialize count"),
        ("while count < 3 { count = count + 1 }", "While loop"),
        ("count", "Count after while"),
    ]);
    
    // Section 8: Enums and Structs
    println!("\n8. ENUMS AND STRUCTS");
    println!("====================");
    run_section(&mut repl, vec![
        ("enum Color { Red, Green, Blue }", "Enum definition"),
        ("Color::Red", "Enum variant"),
        ("struct Point { x, y }", "Struct definition"),
        ("Point { x: 10, y: 20 }", "Struct instantiation"),
    ]);
    
    // Section 9: Advanced Features
    println!("\n9. ADVANCED FEATURES");
    println!("====================");
    run_section(&mut repl, vec![
        ("5 |> double", "Pipe operator"),
        ("let [a, b] = [1, 2]", "Array destructuring"),
        ("a", "First destructured value"),
        ("b", "Second destructured value"),
        ("1..5", "Range"),
    ]);
    
    println!("\n=== Demo Complete ===");
    println!("This demonstrates the current working features of the Ruchy REPL.");
    println!("Some advanced features like async/await, generics, and spread operator");
    println!("are still being implemented.");
    
    Ok(())
}

fn run_section(repl: &mut Repl, expressions: Vec<(&str, &str)>) {
    for (expr, description) in expressions {
        println!("\n  {}", description);
        println!("  > {}", expr);
        match repl.eval(expr) {
            Ok(result) => {
                // Limit output for readability
                if result.len() > 100 {
                    println!("  {}...", &result[..100]);
                } else {
                    println!("  {}", result);
                }
            }
            Err(e) => println!("  ERROR: {}", e),
        }
    }
}