use ruchy::compile;

fn main() {
    // Test actor send
    if let Ok(result) = compile("myactor ! message") {
        println!("Send result: {}", result);
    } else {
        println!("Send failed");
    }
    
    // Test actor ask  
    if let Ok(result) = compile("myactor ? request") {
        println!("Ask result: {}", result);
    } else {
        println!("Ask failed");
    }
    
    // Test let statement
    if let Ok(result) = compile("let x = 10") {
        println!("Let result: {}", result);
    } else {
        println!("Let failed");
    }
    
    // Test trait
    if let Ok(result) = compile("trait Show { fun show(&self) -> String }") {
        println!("Trait result: {}", result);
    } else {
        println!("Trait failed");
    }
}