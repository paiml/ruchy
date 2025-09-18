// Example: Control flow in Ruchy REPL
// Run with: cargo run --example repl_control_flow

use ruchy::runtime::Repl;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut repl = Repl::new(std::env::temp_dir())?;
    
    println!("=== Control Flow Demo ===\n");
    
    println!("If-Else Expressions:");
    let if_exprs = vec![
        "if true { \"yes\" } else { \"no\" }",
        "if 5 > 3 { \"bigger\" } else { \"smaller\" }",
        "let x = 10",
        "if x > 0 { \"positive\" } else if x < 0 { \"negative\" } else { \"zero\" }",
    ];
    
    for expr in if_exprs {
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}"),
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    
    println!("\nLoops:");
    let loop_exprs = vec![
        "let sum = 0",
        "for i in [1, 2, 3, 4, 5] { sum = sum + i }",
        "sum",
        "let count = 0",
        "while count < 5 { count = count + 1 }",
        "count",
    ];
    
    for expr in loop_exprs {
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}"),
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    
    println!("\nPattern Matching:");
    let match_exprs = vec![
        "let value = 2",
        "match value { 1 => \"one\", 2 => \"two\", 3 => \"three\", _ => \"other\" }",
        "let score = 85",
        "match score { s if s >= 90 => \"A\", s if s >= 80 => \"B\", s if s >= 70 => \"C\", _ => \"F\" }",
    ];
    
    for expr in match_exprs {
        println!("  > {expr}");
        match repl.eval(expr) {
            Ok(result) => println!("  {result}"),
            Err(e) => println!("  ERROR: {e}"),
        }
    }
    
    Ok(())
}