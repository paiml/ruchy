#![no_main]

use libfuzzer_sys::fuzz_target;
use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{ExprKind, ImportItem};
use ruchy::backend::transpiler::Transpiler;

fuzz_target!(|data: &[u8]| {
    // Convert fuzzer input to string
    if let Ok(input) = std::str::from_utf8(data) {
        // Test 1: Try parsing as import statement
        let import_input = format!("import {}", input);
        if let Ok(expr) = Parser::new(&import_input).parse() {
            // Verify it parsed as an import
            if let ExprKind::Import { path, items } = &expr.kind {
                // Validate path is non-empty
                assert!(!path.is_empty() || !items.is_empty());
                
                // Try transpiling it
                let transpiler = Transpiler::new();
                let _ = transpiler.transpile(&expr);
            }
        }
        
        // Test 2: Try parsing as import with braces
        if input.contains("::") {
            let parts: Vec<&str> = input.rsplitn(2, "::").collect();
            if parts.len() == 2 {
                let path = parts[1];
                let items = parts[0];
                let import_input = format!("import {}::{{{}}}", path, items);
                if let Ok(expr) = Parser::new(&import_input).parse() {
                    // Try transpiling it
                    let transpiler = Transpiler::new();
                    let _ = transpiler.transpile(&expr);
                }
            }
        }
        
        // Test 3: Try parsing as module declaration
        let module_input = format!("module TestModule {{ {} }}", input);
        if let Ok(expr) = Parser::new(&module_input).parse() {
            if let ExprKind::Module { name, body } = &expr.kind {
                assert_eq!(name, "TestModule");
                // Body should be valid
                assert!(body.span.start <= body.span.end);
            }
        }
        
        // Test 4: Try parsing as export statement
        let export_input = format!("export {}", input);
        if let Ok(expr) = Parser::new(&export_input).parse() {
            if let ExprKind::Export { items } = &expr.kind {
                // Items should be non-empty for successful parse
                assert!(!items.is_empty());
            }
        }
        
        // Test 5: Try parsing wildcard import
        let wildcard_input = format!("import {}::*", input);
        if let Ok(expr) = Parser::new(&wildcard_input).parse() {
            if let ExprKind::Import { items, .. } = &expr.kind {
                // Should have exactly one wildcard item
                assert!(items.len() == 1);
                assert!(matches!(items[0], ImportItem::Wildcard));
            }
        }
        
        // Test 6: Try parsing aliased import
        if !input.is_empty() && !input.contains(char::is_whitespace) {
            let alias_input = format!("import {} as MyAlias", input);
            if let Ok(expr) = Parser::new(&alias_input).parse() {
                if let ExprKind::Import { items, .. } = &expr.kind {
                    assert!(items.len() == 1);
                    if let ImportItem::Aliased { alias, .. } = &items[0] {
                        assert_eq!(alias, "MyAlias");
                    }
                }
            }
        }
        
        // Test 7: Round-trip test for valid imports
        if input.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':') 
           && !input.is_empty() 
           && input.chars().next().unwrap().is_alphabetic() {
            let simple_import = format!("import {}", input);
            if let Ok(expr) = Parser::new(&simple_import).parse() {
                // Try to transpile and ensure no panic
                let transpiler = Transpiler::new();
                if let Ok(tokens) = transpiler.transpile(&expr) {
                    let output = tokens.to_string();
                    // Output should contain "use"
                    assert!(output.contains("use"));
                }
            }
        }
    }
});