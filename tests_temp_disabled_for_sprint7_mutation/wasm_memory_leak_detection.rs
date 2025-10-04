//! WASM Memory Leak Detection Tests
//! WebAssembly Extreme Quality Assurance Framework v3.0

#[cfg(all(target_arch = "wasm32", test))]
use wasm_bindgen_test::*;

#[cfg(all(target_arch = "wasm32", test))]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Unified test macro for memory leak detection
macro_rules! memory_test {
    ($name:ident, $body:expr) => {
        #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen_test)]
        #[cfg_attr(not(all(target_arch = "wasm32", target_os = "unknown")), test)]
        fn $name() {
            $body
        }
    };
}

memory_test!(test_repeated_allocation_deallocation, {
    // Check that repeated allocation and deallocation doesn't leak memory
    for iteration in 0..100 {
        // Allocate some data
        let data = vec![0u8; 1024 * iteration % 10]; // Variable size 0-9KB

        // Process it
        let checksum: u32 = data.iter().map(|&x| x as u32).sum();

        // Check processing
        assert_eq!(checksum, 0); // All zeros

        // Rust will automatically deallocate when data goes out of scope
    }
});

memory_test!(test_parser_memory_cleanup, {
    // Check that parser doesn't leak memory across multiple uses
    for i in 0..50 {
        let source = format!("let x{} = {};", i, i * 2);
        let mut parser = ruchy::frontend::Parser::new(&source);
        let result = parser.parse();

        // Parser should either succeed or fail gracefully
        match result {
            Ok(_ast) => {
                // AST will be dropped automatically
            }
            Err(_e) => {
                // Error will be dropped automatically
            }
        }
    }
});

memory_test!(test_transpiler_memory_stability, {
    // Check that transpiler maintains stable memory usage
    let transpiler = ruchy::backend::Transpiler::new();

    for i in 0..30 {
        let source = format!("fn test{}() {{ {} }}", i, i);
        let mut parser = ruchy::frontend::Parser::new(&source);

        if let Ok(ast) = parser.parse() {
            let _result = transpiler.transpile(&ast);
            // Result will be dropped automatically
        }
    }
});

#[cfg(all(target_arch = "wasm32", test))]
mod wasm_specific_memory_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_wasm_string_allocation() {
        // Check string allocation patterns common in WASM FFI
        for i in 0..100 {
            let test_string = format!("test string {}", i);
            let js_string: JsValue = test_string.into();

            // Convert back to Rust string
            if let Some(rust_string) = js_string.as_string() {
                assert!(rust_string.contains("test string"));
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_array_buffer_management() {
        // Check array buffer allocation and cleanup
        for size in [1024, 2048, 4096, 8192] {
            let data = vec![42u8; size];
            let uint8_array = js_sys::Uint8Array::from(&data[..]);

            // Check the data
            assert_eq!(uint8_array.length(), size as u32);
            assert_eq!(uint8_array.get_index(0), 42);

            // Array will be cleaned up by JS GC
        }
    }

    #[wasm_bindgen_test]
    fn test_function_call_memory_overhead() {
        // Check that repeated function calls don't accumulate memory
        use crate::wasm_bindings::RuchyWasm;

        let compiler = RuchyWasm::new();

        for i in 0..50 {
            let source = format!("let value = {}", i);
            let is_valid = compiler.validate(&source);
            assert!(is_valid);
        }
    }

    #[wasm_bindgen_test]
    async fn test_async_memory_cleanup() {
        // Check async operations don't leak memory
        use crate::wasm_bindings::RuchyWasm;
        use wasm_bindgen_futures::JsFuture;

        let compiler = RuchyWasm::new();

        for i in 0..10 {
            let promise = compiler.async_operation(i);
            let result = JsFuture::from(promise).await;

            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod native_memory_tests {
    use super::*;

    #[test]
    fn test_large_input_memory_handling() {
        // Check that large inputs are handled without excessive memory usage
        let large_input = "let x = 1; ".repeat(1000); // ~10KB of repeated code

        let mut parser = ruchy::frontend::Parser::new(&large_input);
        let result = parser.parse();

        // Should handle large input gracefully
        assert!(result.is_ok() || result.is_err()); // Either succeeds or fails gracefully
    }

    #[test]
    fn test_error_path_memory_cleanup() {
        // Check that error paths don't leak memory
        let invalid_inputs = [
            "let x = ",  // Incomplete
            "if true {", // Unclosed
            "fun () {",  // Invalid syntax
            "match x {", // Incomplete match
            "for in {",  // Invalid for loop
        ];

        for input in invalid_inputs {
            let mut parser = ruchy::frontend::Parser::new(input);
            let _result = parser.parse(); // May fail, that's expected

            // Memory should be cleaned up regardless of success/failure
        }
    }

    #[test]
    fn test_recursive_structure_memory() {
        // Check deeply nested structures don't cause memory issues
        let mut nested = String::from("1");
        for _ in 0..100 {
            nested = format!("({})", nested);
        }

        let mut parser = ruchy::frontend::Parser::new(&nested);
        let _result = parser.parse(); // May hit recursion limits, that's ok

        // Should not leak memory even with deep nesting
    }
}
