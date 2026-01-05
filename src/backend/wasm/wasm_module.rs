//! WASM Module Representation
//!
//! Represents a compiled WASM module ready for execution.

/// Represents a compiled WASM module
#[derive(Debug, Clone)]
pub struct WasmModule {
    bytes: Vec<u8>,
}

impl WasmModule {
    /// Create a new WASM module from compiled bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Get the raw WASM bytes
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get the size of the module in bytes
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    /// Check if this is a valid WASM module (has magic number)
    pub fn is_valid(&self) -> bool {
        self.bytes.len() >= 8 && &self.bytes[0..4] == b"\0asm"
    }

    /// Get the WASM version (should be 1 for current spec)
    pub fn version(&self) -> Option<u32> {
        if self.bytes.len() >= 8 {
            Some(u32::from_le_bytes([
                self.bytes[4],
                self.bytes[5],
                self.bytes[6],
                self.bytes[7],
            ]))
        } else {
            None
        }
    }
}

impl Default for WasmModule {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_module_new() {
        let module = WasmModule::new(vec![0, 1, 2, 3]);
        assert_eq!(module.bytes(), &[0, 1, 2, 3]);
    }

    #[test]
    fn test_wasm_module_size() {
        let module = WasmModule::new(vec![0; 100]);
        assert_eq!(module.size(), 100);
    }

    #[test]
    fn test_wasm_module_default() {
        let module = WasmModule::default();
        assert!(module.bytes().is_empty());
        assert_eq!(module.size(), 0);
    }

    #[test]
    fn test_wasm_module_is_valid() {
        // Valid WASM header
        let valid = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
        assert!(valid.is_valid());

        // Invalid - too short
        let invalid_short = WasmModule::new(vec![0x00, 0x61]);
        assert!(!invalid_short.is_valid());

        // Invalid - wrong magic
        let invalid_magic = WasmModule::new(vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]);
        assert!(!invalid_magic.is_valid());
    }

    #[test]
    fn test_wasm_module_version() {
        let module = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
        assert_eq!(module.version(), Some(1));

        let empty = WasmModule::new(vec![]);
        assert_eq!(empty.version(), None);
    }

    #[test]
    fn test_wasm_module_clone() {
        let module = WasmModule::new(vec![1, 2, 3]);
        let cloned = module.clone();
        assert_eq!(module.bytes(), cloned.bytes());
    }

    #[test]
    fn test_wasm_module_debug() {
        let module = WasmModule::new(vec![1, 2, 3]);
        let debug = format!("{:?}", module);
        assert!(debug.contains("WasmModule"));
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_size_equals_bytes_len(data in prop::collection::vec(any::<u8>(), 0..100)) {
                let module = WasmModule::new(data.clone());
                prop_assert_eq!(module.size(), data.len());
            }

            #[test]
            fn prop_bytes_roundtrip(data in prop::collection::vec(any::<u8>(), 0..100)) {
                let module = WasmModule::new(data.clone());
                prop_assert_eq!(module.bytes(), data.as_slice());
            }

            #[test]
            fn prop_valid_wasm_header(_dummy: u8) {
                let valid = WasmModule::new(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
                prop_assert!(valid.is_valid());
                prop_assert_eq!(valid.version(), Some(1));
            }
        }
    }
}
