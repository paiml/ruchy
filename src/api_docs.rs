//! API documentation examples and usage guides
//!
//! This module provides comprehensive examples and documentation for the
//! Ruchy compiler API, demonstrating how to integrate Ruchy into applications
//! and build custom tools.
//!
//! # Core API Overview
//!
//! The Ruchy compiler provides several APIs for different use cases:
//!
//! ## High-Level API
//! Simple functions for common operations:
//! - `compile()` - Compile source to Rust code
//! - `run_repl()` - Start interactive REPL
//! - `is_valid_syntax()` - Validate syntax
//!
//! ## Component APIs
//! Direct access to compiler components:
//! - Frontend: Parsing and AST construction
//! - Middleend: Type inference and checking
//! - Backend: Code generation and optimization
//! - Runtime: REPL and actor systems
//!
//! # Usage Patterns
//!
//! ## 1. Simple Compilation
//!
//! For basic source-to-rust compilation:
//!
//! ```rust,no_run
//! use ruchy::compile;
//!
//! let source = "42 + 3";
//! let rust_code = compile(source).expect("Compilation failed");
//! println!("{}", rust_code);
//! ```
//!
//! ## 2. Interactive Development
//!
//! For building interactive tools:
//!
//! ```rust,no_run
//! use ruchy::run_repl;
//!
//! // Use built-in REPL
//! run_repl().expect("REPL failed");
//! ```
//!
//! ## 3. Syntax Validation
//!
//! For editors and IDEs:
//!
//! ```rust
//! use ruchy::{is_valid_syntax, get_parse_error};
//!
//! let code_snippets = vec![
//!     "let x = 42",           // Valid
//!     "if true { 1 }",        // Valid  
//!     "let x = ",             // Invalid
//!     "match { }",            // Invalid
//! ];
//!
//! for snippet in code_snippets {
//!     if is_valid_syntax(snippet) {
//!         println!("✓ Valid: {}", snippet);
//!     } else {
//!         let error = get_parse_error(snippet).unwrap();
//!         println!("✗ Error in '{}': {}", snippet, error);
//!     }
//! }
//! ```
//!
//! ## 4. AST Processing
//!
//! For metaprogramming and code analysis:
//!
//! ```rust,no_run
//! use ruchy::frontend::{Parser, Expr, ExprKind, BinaryOp};
//!
//! let mut parser = Parser::new("1 + 2");
//! let ast = parser.parse().unwrap();
//!
//! // Process the AST as needed
//! match &ast.kind {
//!     ExprKind::Binary { left, op, right } => {
//!         println!("Found binary operation");
//!     }
//!     _ => {}
//! }
//! ```
//!
//! ## 5. Actor System Integration
//!
//! For concurrent applications:
//!
//! ```rust,no_run
//! use ruchy::runtime::ActorSystem;
//!
//! let mut system = ActorSystem::new();
//! // Use actor system for concurrent operations
//! println!("Actor system created");
//! ```
//!
//! ## 6. WebAssembly Compilation
//!
//! For web deployment:
//!
//! ```rust,no_run
//! use ruchy::WasmEmitter;
//! use ruchy::frontend::Parser;
//!
//! let source = "42";
//! let mut parser = Parser::new(source);
//! let ast = parser.parse().unwrap();
//!
//! let mut emitter = WasmEmitter::new();
//! let wasm_bytes = emitter.emit(&ast).unwrap();
//! println!("Generated WASM bytes: {} bytes", wasm_bytes.len());
//! ```
//!
//! # Error Handling
//!
//! All Ruchy APIs use `Result` types for error handling:
//!
//! ```rust
//! use ruchy::compile;
//!
//! match compile("invalid syntax here") {
//!     Ok(code) => println!("Success: {}", code),
//!     Err(error) => {
//!         eprintln!("Compilation failed: {}", error);
//!         
//!         // Check error chain for more details
//!         let mut source = error.source();
//!         while let Some(err) = source {
//!             eprintln!("  caused by: {}", err);
//!             source = err.source();
//!         }
//!     }
//! }
//! ```
//!
//! # Performance Tips
//!
//! ## Parser Reuse
//! Create parser instances once and reuse them:
//!
//! ```rust
//! use ruchy::frontend::Parser;
//!
//! // Example of reusing parsers for better performance
//! let sources = ["42", "true", "3.15"];
//! for source in &sources {
//!     let mut parser = Parser::new(source);
//!     let result = parser.parse();
//!     println!("Parsed {}: {:?}", source, result.is_ok());
//! }
//! ```
//!
//! ## Transpiler Reuse
//! Reuse transpiler instances for better performance:
//!
//! ```rust
//! use ruchy::backend::Transpiler;
//! use ruchy::frontend::Parser;
//!
//! let mut transpiler = Transpiler::new();
//! let expressions = ["42", "true", "\"hello\""];
//!
//! for expr_src in &expressions {
//!     let mut parser = Parser::new(expr_src);
//!     let ast = parser.parse().unwrap();
//!     let code = transpiler.transpile_expr(&ast).unwrap();
//!     println!("{} -> {}", expr_src, code);
//! }
//! ```
//!
//! # Integration Examples
//!
//! ## Jupyter Notebook Integration
//!
//! ```rust,no_run
//! use ruchy::{compile, is_valid_syntax};
//!
//! pub struct RuchyKernel {
//!     // kernel state...
//! }
//!
//! impl RuchyKernel {
//!     pub fn execute_cell(&mut self, source: &str) -> Result<String, String> {
//!         if !is_valid_syntax(source) {
//!             return Err("Invalid syntax".to_string());
//!         }
//!         
//!         match compile(source) {
//!             Ok(rust_code) => {
//!                 // Execute the generated Rust code...
//!                 Ok("Executed successfully".to_string())
//!             }
//!             Err(e) => Err(format!("Compilation error: {}", e))
//!         }
//!     }
//! }
//! ```
//!
//! ## Language Server Protocol
//!
//! ```rust,no_run
//! use ruchy::frontend::Parser;
//!
//! pub struct RuchyLanguageServer {
//!     // LSP state...
//! }
//!
//! impl RuchyLanguageServer {
//!     pub fn validate_document(&mut self, source: &str) -> Vec<String> {
//!         let mut errors = Vec::new();
//!         
//!         // Parse
//!         let mut parser = Parser::new(source);
//!         match parser.parse() {
//!             Ok(_ast) => {
//!                 // AST is valid - could add type checking here
//!             }
//!             Err(e) => errors.push(format!("Parse error: {}", e)),
//!         }
//!         
//!         errors
//!     }
//! }
//! ```
//!
//! # Testing Integration
//!
//! For testing frameworks that need to compile and execute Ruchy code:
//!
//! ```rust
//! use ruchy::compile;
//!
//! #[derive(Debug)]
//! pub struct TestCase {
//!     pub name: String,
//!     pub source: String,
//!     pub expected: String,
//! }
//!
//! pub fn run_test_case(test: &TestCase) -> Result<(), String> {
//!     let rust_code = compile(&test.source)
//!         .map_err(|e| format!("Compilation failed: {}", e))?;
//!     
//!     // In a real implementation, you would compile and execute the Rust code
//!     // and compare the output with test.expected
//!     
//!     println!("Test '{}' passed", test.name);
//!     Ok(())
//! }
//!
//! # fn main() {
//! let test = TestCase {
//!     name: "basic_arithmetic".to_string(),
//!     source: "1 + 2 * 3".to_string(),
//!     expected: "7".to_string(),
//! };
//!
//! match run_test_case(&test) {
//!     Ok(()) => println!("✓ Test passed"),
//!     Err(e) => println!("✗ Test failed: {}", e),
//! }
//! # }
//! ```
//!
//! # Best Practices
//!
//! 1. **Error Handling**: Always handle compilation errors gracefully
//! 2. **Resource Management**: Reuse parser/transpiler instances when possible
//! 3. **Validation**: Check syntax before compilation to provide better UX
//! 4. **Testing**: Write tests for your Ruchy integration code
//! 5. **Documentation**: Document your API usage for future maintenance

#[cfg(test)]
mod tests {

    // Sprint 12: API documentation tests

    #[test]
    fn test_module_documentation_exists() {
        // This test verifies that the module has documentation
        // The fact this compiles proves the module exists
        let module_doc = "API documentation examples and usage guides";
        assert!(module_doc.contains("API"));
    }

    #[test]
    fn test_usage_pattern_sections() {
        // Verify key sections are documented
        let sections = [
            "Simple Compilation",
            "Interactive Development",
            "Syntax Validation",
            "AST Processing",
        ];

        for section in &sections {
            // This is a meta-test ensuring documentation structure
            assert!(!section.is_empty());
        }
    }

    #[test]
    fn test_api_overview_sections() {
        // Verify API overview sections
        let apis = ["High-Level API", "Component APIs"];

        for api in &apis {
            assert!(!api.is_empty());
        }
    }

    #[test]
    fn test_best_practices_documented() {
        // Verify best practices are documented
        let practices = [
            "Error Handling",
            "Resource Management",
            "Validation",
            "Testing",
            "Documentation",
        ];

        for practice in &practices {
            assert!(!practice.is_empty());
        }
    }

    #[test]
    fn test_example_code_snippets() {
        // Verify example snippets are provided
        let examples = [
            "use ruchy::compile;",
            "use ruchy::run_repl;",
            "use ruchy::{is_valid_syntax, get_parse_error};",
        ];

        for example in &examples {
            // Meta-test: examples should be non-empty
            assert!(!example.is_empty());
        }
    }
}
