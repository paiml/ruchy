//! WASM Helper Functions
//!
//! Platform-agnostic utilities for WASM REPL.

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        // Use browser's crypto API for UUID
        format!("session-{}", js_sys::Date::now())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use system time for non-WASM builds
        format!(
            "session-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("SystemTime should be after UNIX_EPOCH")
                .as_millis()
        )
    }
}

/// Get current timestamp in milliseconds
pub fn get_timestamp() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("SystemTime should be after UNIX_EPOCH")
            .as_millis() as f64
    }
}

/// JsValue stub for non-WASM builds
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct JsValue;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::prelude::JsValue;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_session_id() {
        let id = generate_session_id();
        assert!(id.starts_with("session-"));
    }

    #[test]
    fn test_generate_session_id_unique() {
        let id1 = generate_session_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = generate_session_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_get_timestamp() {
        let ts = get_timestamp();
        assert!(ts > 0.0);
    }

    #[test]
    fn test_get_timestamp_monotonic() {
        let ts1 = get_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let ts2 = get_timestamp();
        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_get_timestamp_reasonable_value() {
        let ts = get_timestamp();
        // Should be after year 2020 (1577836800000 ms)
        assert!(ts > 1_577_836_800_000.0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_jsvalue_stub_debug() {
        let js = JsValue;
        let debug = format!("{:?}", js);
        assert!(debug.contains("JsValue"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_jsvalue_stub_clone() {
        let js = JsValue;
        let _cloned = js.clone();
    }

    // ===== Property Tests =====

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            /// Property: Session IDs always start with "session-"
            #[test]
            fn prop_session_id_format(_dummy: u8) {
                let id = generate_session_id();
                prop_assert!(id.starts_with("session-"));
            }

            /// Property: Timestamps are always positive
            #[test]
            fn prop_timestamp_positive(_dummy: u8) {
                let ts = get_timestamp();
                prop_assert!(ts > 0.0);
            }

            /// Property: Timestamps are monotonically increasing
            #[test]
            fn prop_timestamp_monotonic(_dummy: u8) {
                let ts1 = get_timestamp();
                let ts2 = get_timestamp();
                prop_assert!(ts2 >= ts1);
            }
        }
    }
}
