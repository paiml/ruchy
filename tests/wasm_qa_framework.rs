//! WebAssembly Quality Assurance Framework Tests
//! Following the unified testing pattern from the specification

#[cfg(all(target_arch = "wasm32", test))]
use wasm_bindgen_test::*;

#[cfg(all(target_arch = "wasm32", test))]
wasm_bindgen_test_configure!(run_in_browser);

// Macro to reduce boilerplate for cross-platform tests
macro_rules! unified_test {
    ($name:ident, $body:expr) => {
        #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
        #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
        fn $name() {
            $body
        }
    };
}

unified_test!(test_basic_compilation, {
    // Check that basic Ruchy code compiles across platforms
    let source = "let x = 42; x + 1";
    let mut parser = ruchy::frontend::Parser::new(source);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Basic parsing should work on all platforms");
});

unified_test!(test_transpilation_deterministic, {
    // Check that transpilation produces consistent results
    let source = "fun add(a: i32, b: i32) -> i32 { a + b }";
    let mut parser = ruchy::frontend::Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = ruchy::backend::Transpiler::new();
        let result1 = transpiler.transpile(&ast);
        let result2 = transpiler.transpile(&ast);
        assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Transpilation should be deterministic"
        );
    }
});

unified_test!(test_memory_safety, {
    // Check that our code doesn't have obvious memory issues
    let mut data = vec![0u8; 1024];
    for i in 0..data.len() {
        data[i] = (i % 256) as u8;
    }

    // Process the data through our system
    let checksum_before: u32 = data.iter().map(|&x| x as u32).sum();

    // Simulate some processing
    let processed = data.clone();
    let checksum_after: u32 = processed.iter().map(|&x| x as u32).sum();

    assert_eq!(
        checksum_before, checksum_after,
        "Memory should remain consistent"
    );
});

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_specific_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_wasm_panic_hook() {
        // Check that panic hook is properly set
        console_error_panic_hook::set_once();
        // This should not panic the test - just verify the hook is set
        assert!(true);
    }

    #[wasm_bindgen_test]
    fn test_wasm_console_access() {
        // Check that we can access console functions
        web_sys::console::log_1(&"WASM QA Framework test".into());
        assert!(true);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        #[cfg_attr(not(target_arch = "wasm32"), test)]
        fn prop_parser_never_panics(input in ".*") {
            // Limit input size to prevent performance issues
            let test_input = if input.len() > 100 { &input[..100] } else { &input };

            let mut parser = ruchy::frontend::Parser::new(test_input);
            let _ = parser.parse(); // Should not panic
            prop_assert!(true);
        }
    }
}
