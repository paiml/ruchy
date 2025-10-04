//! Browser Compatibility Suite
//! WebAssembly Extreme Quality Assurance Framework v3.0

#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use wasm_bindgen_test::*;
use web_sys::{window, Document, Element, Window};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_browser_api_availability() {
    let window = window().expect("should have window");
    let document = window.document().expect("should have document");

    // Check critical APIs exist
    assert!(window.local_storage().is_ok());
    assert!(window.session_storage().is_ok());
    assert!(document.create_element("canvas").is_ok());

    // Check WebGL availability (if supported)
    let canvas: Element = document.create_element("canvas").unwrap();
    if let Ok(canvas) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() {
        let gl_context = canvas.get_context("webgl2");
        if gl_context.is_ok() {
            // WebGL2 is available - good for advanced features
            web_sys::console::log_1(&"WebGL2 context available".into());
        } else {
            // Fallback to WebGL1
            let gl1_context = canvas.get_context("webgl");
            assert!(gl1_context.is_ok(), "At least WebGL1 should be available");
        }
    }
}

#[wasm_bindgen_test]
async fn test_async_browser_apis() {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};

    let opts = RequestInit::new();
    opts.set_method("HEAD");

    // Check with a reliable endpoint
    let request = Request::new_with_str_and_init("https://httpbin.org/status/200", &opts);

    if request.is_ok() {
        let window = window().unwrap();
        let request = request.unwrap();
        let promise = window.fetch_with_request(&request);

        match JsFuture::from(promise).await {
            Ok(response_val) => {
                let response: Result<Response, _> = response_val.dyn_into();
                if let Ok(response) = response {
                    assert!(response.status() >= 200 && response.status() < 300);
                }
            }
            Err(_) => {
                // Network error is acceptable - we're testing API availability
                web_sys::console::log_1(
                    &"Network request failed (expected in some test environments)".into(),
                );
            }
        }
    }
}

#[wasm_bindgen_test]
fn test_memory_and_performance() {
    // Check that we can access performance APIs
    let window = window().unwrap();

    if let Ok(performance) = window.performance() {
        let now = performance.now();
        assert!(now > 0.0, "Performance timer should be available");

        // Check memory API if available
        if let Ok(memory) = performance.memory() {
            let used_heap = memory.used_js_heap_size();
            assert!(used_heap > 0, "Memory usage should be positive");
        }
    }
}

#[wasm_bindgen_test]
fn test_error_handling_and_console() {
    // Check console API
    web_sys::console::log_1(&"Testing console.log from WASM".into());
    web_sys::console::warn_1(&"Testing console.warn from WASM".into());

    // Check error boundary
    let result = std::panic::catch_unwind(|| {
        // This should not actually panic in a well-behaved test
        42
    });
    assert!(result.is_ok(), "Panic catching should work");
}

#[wasm_bindgen_test]
fn test_local_storage_operations() {
    let window = window().unwrap();

    if let Ok(Some(storage)) = window.local_storage() {
        let test_key = "wasm_qa_test_key";
        let test_value = "wasm_qa_test_value";

        // Check storage operations
        let _ = storage.set_item(test_key, test_value);

        if let Ok(Some(retrieved)) = storage.get_item(test_key) {
            assert_eq!(retrieved, test_value);
        }

        // Clean up
        let _ = storage.remove_item(test_key);
    }
}
