// Simple standalone test for WASM generation
fn main() {
    use wasm_encoder::*;
    
    // Create the simplest valid WASM module
    let mut module = Module::new();
    
    // Add empty type section
    let types = TypeSection::new();
    module.section(&types);
    
    let bytes = module.finish();
    
    println!("Minimal module: {} bytes", bytes.len());
    for b in &bytes {
        print!("{:02x} ", b);
    }
    println!();
    
    // Now with a function
    let mut module2 = Module::new();
    
    // Type section with one function type
    let mut types2 = TypeSection::new();
    types2.function(vec![], vec![]);
    module2.section(&types2);
    
    // Function section
    let mut functions = FunctionSection::new();
    functions.function(0);
    module2.section(&functions);
    
    // Code section
    let mut codes = CodeSection::new();
    let func = Function::new(vec![]);
    codes.function(&func);
    module2.section(&codes);
    
    let bytes2 = module2.finish();
    
    println!("\nWith function: {} bytes", bytes2.len());
    for b in &bytes2 {
        print!("{:02x} ", b);
    }
    println!();
}