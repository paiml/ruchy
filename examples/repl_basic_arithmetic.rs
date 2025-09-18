// Example: Basic arithmetic operations in Ruchy REPL
// Run with: cargo run --example repl_basic_arithmetic

use ruchy::runtime::Repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new(std::env::temp_dir())?;
    
    println!("=== Basic Arithmetic Demo ===\n");
    
    // Basic math operations
    let expressions = vec![
        ("2 + 2", "Addition"),
        ("10 * 5", "Multiplication"),
        ("100 - 25", "Subtraction"),
        ("50 / 2", "Division"),
        ("17 % 5", "Modulo"),
        ("2 ** 8", "Exponentiation"),
        ("(10 + 5) * 2", "Parentheses"),
    ];
    
    for (expr, description) in expressions {
        println!("{description}:");
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}\n"),
            Err(e) => println!("  ERROR: {e}\n"),
        }
    }
    
    Ok(())
}