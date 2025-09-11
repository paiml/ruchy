fn main() {
    use wasm_encoder::*;
    
    let mut module = Module::new();
    
    // Type section
    let mut types = TypeSection::new();
    types.function(vec![], vec![ValType::I32]);
    module.section(&types);
    
    // Function section
    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);
    
    // Export section
    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 0);
    module.section(&exports);
    
    // Code section
    let mut code = CodeSection::new();
    let mut function = Function::new(vec![]);
    function.instruction(&Instruction::I32Const(42));
    function.instruction(&Instruction::End);
    code.function(&function);
    module.section(&code);
    
    let bytes = module.finish();
    
    // Write to file for inspection
    std::fs::write("test.wasm", &bytes).unwrap();
    
    // Try to validate with wasmtime
    match wasmtime::Module::new(&wasmtime::Engine::default(), &bytes) {
        Ok(_) => println!("Module is valid!"),
        Err(e) => println!("Module validation failed: {}", e),
    }
}
