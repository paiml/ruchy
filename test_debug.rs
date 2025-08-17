use ruchy::{compile, get_parse_error};

fn main() {
    // Test trait parsing
    match compile("trait Show { fun show(&self) -> String }") {
        Ok(result) => println!("Trait success: {}", result),
        Err(e) => println!("Trait error: {}", e),
    }
    
    // Test parse error
    if let Some(error) = get_parse_error("trait Show { fun show(&self) -> String }") {
        println!("Parse error: {}", error);
    }
}