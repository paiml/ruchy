// Example: Variables and functions in Ruchy REPL
// Run with: cargo run --example repl_variables_and_functions

use ruchy::runtime::Repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new(std::env::temp_dir())?;
    
    println!("=== Variables and Functions Demo ===\n");
    
    // Variable examples
    println!("Variables:");
    let variable_exprs = vec![
        "let x = 42",
        "let name = \"Ruchy\"",
        "let numbers = [1, 2, 3, 4, 5]",
        "x",
        "name",
        "numbers",
    ];
    
    for expr in variable_exprs {
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}"),
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    
    println!("\nFunctions:");
    let function_exprs = vec![
        "fn add(a, b) { a + b }",
        "add(3, 4)",
        "fn greet(name) { \"Hello, \" + name }",
        "greet(\"World\")",
        "let double = fn(x) { x * 2 }",
        "double(21)",
    ];
    
    for expr in function_exprs {
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}"),
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    
    println!("\nRecursive Functions:");
    let recursive_exprs = vec![
        "fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }",
        "factorial(5)",
        "fn fib(n) { if n <= 1 { n } else { fib(n - 1) + fib(n - 2) } }",
        "fib(10)",
    ];
    
    for expr in recursive_exprs {
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}"),
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    
    Ok(())
}