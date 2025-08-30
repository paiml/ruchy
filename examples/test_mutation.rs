use ruchy::runtime::Repl;

fn main() {
    let mut repl = Repl::new().unwrap();
    
    println!("Testing mutation in while loops:\n");
    
    // Test immutable variable (should fail)
    println!("1. Testing immutable variable:");
    println!("   > let x = 0");
    match repl.eval("let x = 0") {
        Ok(v) => println!("   {v}"),
        Err(e) => println!("   ERROR: {e}"),
    }
    
    println!("   > while x < 5 {{ x = x + 1 }}");
    match repl.eval("while x < 5 { x = x + 1 }") {
        Ok(v) => println!("   {v}"),
        Err(e) => println!("   ERROR: {e}"),
    }
    
    // Test mutable variable (should work)
    println!("\n2. Testing mutable variable:");
    println!("   > let mut y = 0");
    match repl.eval("let mut y = 0") {
        Ok(v) => println!("   {v}"),
        Err(e) => println!("   ERROR: {e}"),
    }
    
    println!("   > while y < 5 {{ y = y + 1 }}");
    match repl.eval("while y < 5 { y = y + 1 }") {
        Ok(v) => println!("   {v}"),
        Err(e) => println!("   ERROR: {e}"),
    }
    
    println!("   > y");
    match repl.eval("y") {
        Ok(v) => println!("   {v}"),
        Err(e) => println!("   ERROR: {e}"),
    }
}