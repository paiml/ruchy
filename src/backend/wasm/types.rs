//! WASM Type Definitions
//!
//! Core type definitions for WASM code generation.

/// WASM value types for type inference
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    /// 32-bit integer
    #[default]
    I32,
    /// 32-bit float
    F32,
    /// 64-bit integer
    I64,
    /// 64-bit float
    F64,
}

impl WasmType {
    /// Check if this type is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, WasmType::F32 | WasmType::F64)
    }

    /// Check if this type is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(self, WasmType::I32 | WasmType::I64)
    }

    /// Check if this type is 64-bit
    pub fn is_64bit(&self) -> bool {
        matches!(self, WasmType::I64 | WasmType::F64)
    }

    /// Check if this type is 32-bit
    pub fn is_32bit(&self) -> bool {
        matches!(self, WasmType::I32 | WasmType::F32)
    }

    /// Get the byte size of this type
    pub fn byte_size(&self) -> usize {
        match self {
            WasmType::I32 | WasmType::F32 => 4,
            WasmType::I64 | WasmType::F64 => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_type_is_float() {
        assert!(WasmType::F32.is_float());
        assert!(WasmType::F64.is_float());
        assert!(!WasmType::I32.is_float());
        assert!(!WasmType::I64.is_float());
    }

    #[test]
    fn test_wasm_type_is_integer() {
        assert!(WasmType::I32.is_integer());
        assert!(WasmType::I64.is_integer());
        assert!(!WasmType::F32.is_integer());
        assert!(!WasmType::F64.is_integer());
    }

    #[test]
    fn test_wasm_type_is_64bit() {
        assert!(WasmType::I64.is_64bit());
        assert!(WasmType::F64.is_64bit());
        assert!(!WasmType::I32.is_64bit());
        assert!(!WasmType::F32.is_64bit());
    }

    #[test]
    fn test_wasm_type_is_32bit() {
        assert!(WasmType::I32.is_32bit());
        assert!(WasmType::F32.is_32bit());
        assert!(!WasmType::I64.is_32bit());
        assert!(!WasmType::F64.is_32bit());
    }

    #[test]
    fn test_wasm_type_byte_size() {
        assert_eq!(WasmType::I32.byte_size(), 4);
        assert_eq!(WasmType::F32.byte_size(), 4);
        assert_eq!(WasmType::I64.byte_size(), 8);
        assert_eq!(WasmType::F64.byte_size(), 8);
    }

    #[test]
    fn test_wasm_type_default() {
        assert_eq!(WasmType::default(), WasmType::I32);
    }

    #[test]
    fn test_wasm_type_debug() {
        let debug = format!("{:?}", WasmType::I32);
        assert!(debug.contains("I32"));
    }

    #[test]
    fn test_wasm_type_clone() {
        let ty = WasmType::F64;
        let cloned = ty;
        assert_eq!(ty, cloned);
    }

    #[test]
    fn test_wasm_type_eq() {
        assert_eq!(WasmType::I32, WasmType::I32);
        assert_ne!(WasmType::I32, WasmType::F32);
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        fn arb_wasm_type() -> impl Strategy<Value = WasmType> {
            prop_oneof![
                Just(WasmType::I32),
                Just(WasmType::F32),
                Just(WasmType::I64),
                Just(WasmType::F64),
            ]
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            #[test]
            fn prop_float_xor_integer(ty in arb_wasm_type()) {
                // A type is either float or integer, not both
                prop_assert!(ty.is_float() != ty.is_integer());
            }

            #[test]
            fn prop_32bit_xor_64bit(ty in arb_wasm_type()) {
                // A type is either 32-bit or 64-bit, not both
                prop_assert!(ty.is_32bit() != ty.is_64bit());
            }

            #[test]
            fn prop_byte_size_consistent(ty in arb_wasm_type()) {
                let size = ty.byte_size();
                if ty.is_32bit() {
                    prop_assert_eq!(size, 4);
                } else {
                    prop_assert_eq!(size, 8);
                }
            }
        }
    }
}
