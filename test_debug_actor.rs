use ruchy::compile;

fn main() {
    println!("=== Actor Send Debug ===");
    if let Ok(result) = compile("myactor ! message") {
        println!("Send output:\n{}", result);
        println!("Contains '.send('? {}", result.contains(".send("));
    }
    
    println!("\n=== Actor Ask Debug ===");
    if let Ok(result) = compile("myactor ? request") {
        println!("Ask output:\n{}", result);
        println!("Contains '.ask('? {}", result.contains(".ask("));
    }
}