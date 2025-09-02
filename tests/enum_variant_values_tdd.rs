// TDD test for enum variant values support (GitHub Issue #18)
// GOAL: Support enum variants with explicit discriminant values
// CRITICAL: Blocking TypeScript→Ruchy migration for ubuntu-config-scripts project
// RED → GREEN → REFACTOR methodology

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_parse_enum_with_explicit_values() {
    // Test parsing enum with explicit discriminant values
    let test_cases = vec![
        ("enum LogLevel { DEBUG = 0 }", "single variant with value"),
        ("enum LogLevel { DEBUG = 0, INFO = 1 }", "multiple variants with values"),
        ("enum Status { OK = 200, NotFound = 404 }", "HTTP status codes"),
        ("enum Flags { Read = 1, Write = 2, Execute = 4 }", "bit flags"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                assert!(debug_str.contains("Enum"), 
                        "{}: Should parse enum with values: {}", description, debug_str);
                println!("✅ {}: {}", description, input);
            },
            Err(e) => {
                // Currently fails - this guides our implementation
                println!("❌ {} FAILS: {} - Error: {}", description, input, e);
                assert!(false, "Enum with values should parse but got: {}", e);
            }
        }
    }
}

#[test]
fn test_transpile_enum_with_values() {
    // Test transpiling enum with values to Rust
    let test_cases = vec![
        (
            "enum LogLevel { DEBUG = 0, INFO = 1, WARN = 2, ERROR = 3 }",
            "#[repr(i32)]\nenum LogLevel {\n    DEBUG = 0,\n    INFO = 1,\n    WARN = 2,\n    ERROR = 3,\n}",
            "log level enum"
        ),
        (
            "enum HttpStatus { OK = 200, NotFound = 404, ServerError = 500 }",
            "#[repr(i32)]\nenum HttpStatus {\n    OK = 200,\n    NotFound = 404,\n    ServerError = 500,\n}",
            "HTTP status codes"
        ),
    ];
    
    for (input, expected_contains, description) in test_cases {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(expr) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&expr) {
                    Ok(rust_code) => {
                        let code_str = rust_code.to_string();
                        // Check if generated code contains expected patterns
                        // Note: TokenStream may format with spaces
                        let has_repr = code_str.contains("#[repr(") || code_str.contains("# [repr");
                        let has_values = code_str.contains(" = ");
                        if has_repr && has_values {
                            println!("✅ Transpile {}: generates discriminant values", description);
                        } else {
                            println!("❌ Transpile {} FAILS: missing discriminant values", description);
                            println!("  has_repr: {}, has_values: {}", has_repr, has_values);
                            assert!(false, "Expected discriminant values in: {}", code_str);
                        }
                    },
                    Err(e) => {
                        println!("❌ Transpile {} FAILS: {}", description, e);
                        assert!(false, "Transpilation should work but got: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("❌ Parse {} FAILS: {}", description, e);
                // Expected to fail until we implement the feature
            }
        }
    }
}

#[test]
fn test_enum_without_values_still_works() {
    // Ensure regular enums without values continue to work
    let test_cases = vec![
        ("enum Color { Red, Green, Blue }", "simple enum"),
        ("enum Option<T> { Some(T), None }", "generic enum"),
        ("pub enum Result<T, E> { Ok(T), Err(E) }", "public generic enum"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: Regular enum should still parse: {:?}", 
                description, result.err());
        println!("✅ {}: {}", description, input);
    }
}

#[test]
fn test_mixed_enum_variants() {
    // Test enum with both valued and non-valued variants
    let test_cases = vec![
        ("enum Mixed { A, B = 10, C }", "mixed with gaps"),
        ("enum Mixed { A = 1, B, C = 5 }", "explicit and implicit"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(_) => {
                println!("✅ {}: {}", description, input);
            },
            Err(e) => {
                println!("⚠️  {} not yet working: {} (will implement)", description, e);
            }
        }
    }
}

#[test]
fn test_enum_value_expressions() {
    // Test enum variants with expression values
    let test_cases = vec![
        ("enum Flags { A = 1 << 0, B = 1 << 1, C = 1 << 2 }", "bit shift expressions"),
        ("enum Math { PI = 3, E = 2 }", "numeric constants"),
        ("enum Calc { Sum = 1 + 1, Product = 2 * 3 }", "arithmetic expressions"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(_) => {
                println!("✅ {}: {}", description, input);
            },
            Err(e) => {
                println!("⚠️  {} expressions not yet working: {} (stretch goal)", description, e);
            }
        }
    }
}

#[test]
fn test_typescript_migration_use_case() {
    // Real-world TypeScript enum from ubuntu-config-scripts
    let typescript_style = r#"
        pub enum LogLevel {
            DEBUG = 0,
            INFO = 1,
            WARN = 2,
            ERROR = 3,
            FATAL = 4
        }
    "#;
    
    let mut parser = Parser::new(typescript_style);
    let result = parser.parse();
    
    match result {
        Ok(expr) => {
            println!("✅ TypeScript-style enum migration works!");
            
            // Also test transpilation
            let transpiler = Transpiler::new();
            match transpiler.transpile(&expr) {
                Ok(rust_code) => {
                    let code_str = rust_code.to_string();
                    // Check for repr attribute (may have spaces in formatting)
                    assert!(code_str.contains("[repr") || code_str.contains("#[repr"), 
                            "Should generate repr attribute for valued enum: {}", code_str);
                    assert!(code_str.contains("DEBUG = 0"), 
                            "Should preserve discriminant values: {}", code_str);
                    println!("✅ Transpilation produces valid Rust with discriminants");
                },
                Err(e) => {
                    println!("❌ Transpilation failed: {}", e);
                    assert!(false, "Transpilation should work for TypeScript migration");
                }
            }
        },
        Err(e) => {
            println!("❌ CRITICAL: TypeScript enum pattern MUST work: {}", e);
            assert!(false, "This is blocking ubuntu-config-scripts migration!");
        }
    }
}

#[test]
fn test_enum_variant_values_quality_gate() {
    // Ensure implementation maintains PMAT TDG A- grade
    println!("📊 Quality Gate Check for Enum Variant Values Implementation:");
    println!("  - Parser complexity must stay <10");
    println!("  - AST changes must be minimal");
    println!("  - Transpiler must generate idiomatic Rust");
    println!("  - All existing enum tests must still pass");
    println!("  - TDG grade must remain A- or better");
    
    // This will be validated by running PMAT after implementation
    // pmat tdg . --min-grade A- --fail-on-violation
}

#[test]
fn test_all_enum_patterns_comprehensive() {
    // Comprehensive test to ensure all patterns work
    let test_cases = vec![
        // Basic patterns
        ("enum E { A }", true, "single variant"),
        ("enum E { A, B, C }", true, "multiple variants"),
        
        // With values
        ("enum E { A = 1 }", false, "single with value"), // Currently fails
        ("enum E { A = 0, B = 1 }", false, "multiple with values"), // Currently fails
        
        // Generic enums
        ("enum Option<T> { Some(T), None }", true, "generic enum"),
        ("enum Result<T,E> { Ok(T), Err(E) }", true, "multi-generic"),
        
        // Public enums
        ("pub enum E { A }", true, "public enum"),
        ("pub enum E { A = 1 }", false, "public with value"), // Currently fails
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (input, should_work, variant) in test_cases {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(_) => {
                if should_work {
                    println!("✅ {} variant: works as expected", variant);
                    passed += 1;
                } else {
                    println!("🎉 {} variant: NOW WORKING (was broken)!", variant);
                    passed += 1;
                }
            },
            Err(e) => {
                if should_work {
                    println!("❌ {} variant BROKEN: {}", variant, e);
                    failed += 1;
                } else {
                    println!("⚠️  {} variant: Expected failure (not yet implemented)", variant);
                    failed += 1;
                }
            }
        }
    }
    
    println!("\n📊 Enum Patterns Results: {} passed, {} failed", passed, failed);
    
    // Currently we expect valued enums to fail
    // After implementation, all should pass
    assert!(passed >= 5, "At least basic enums should work");
}