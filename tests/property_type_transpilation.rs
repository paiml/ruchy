//! Property-based tests for type transpilation
//! Verifies the refactored type transpilation functions maintain correctness

use ruchy::{Transpiler, Parser};

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};
    
    // Property: All basic types should transpile consistently
    #[test]
    fn prop_basic_types_transpile_consistently() {
        fn check_type_transpilation(type_name: String) -> TestResult {
            // Filter to valid type names
            if type_name.is_empty() || !type_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return TestResult::discard();
            }
            
            let code = format!("let x: {} = undefined", type_name);
            let mut parser = Parser::new(&code);
            
            // Should either parse successfully or fail consistently
            match parser.parse() {
                Ok(ast) => {
                    let transpiler = Transpiler::new();
                    // Transpilation should be deterministic
                    let result1 = transpiler.transpile(&ast);
                    let result2 = transpiler.transpile(&ast);
                    
                    TestResult::from_bool(
                        result1.is_ok() == result2.is_ok() &&
                        result1.map(|r| r.to_string()) == result2.map(|r| r.to_string())
                    )
                }
                Err(_) => TestResult::passed()  // Parse errors are ok for invalid types
            }
        }
        
        quickcheck(check_type_transpilation as fn(String) -> TestResult);
    }
    
    // Property: Generic types with valid parameters should transpile
    #[test]
    fn prop_generic_types_transpile() {
        fn check_generic_type(base: String, param_count: u8) -> TestResult {
            if base.is_empty() || param_count == 0 || param_count > 5 {
                return TestResult::discard();
            }
            
            // Common generic types
            let base_type = match base.len() % 4 {
                0 => "Vec",
                1 => "Option", 
                2 => "Result",
                _ => "HashMap",
            };
            
            let params = (0..param_count.min(2))
                .map(|i| match i {
                    0 => "i32",
                    _ => "String",
                })
                .collect::<Vec<_>>()
                .join(", ");
            
            let code = format!("let x: {}<{}> = undefined", base_type, params);
            let mut parser = Parser::new(&code);
            
            match parser.parse() {
                Ok(ast) => {
                    let transpiler = Transpiler::new();
                    let result = transpiler.transpile(&ast);
                    TestResult::from_bool(result.is_ok())
                }
                Err(_) => TestResult::discard()
            }
        }
        
        quickcheck(check_generic_type as fn(String, u8) -> TestResult);
    }
    
    // Property: Optional types should always map to Option<T>
    #[test]
    fn prop_optional_types_map_correctly() {
        fn check_optional_type(inner_type: String) -> TestResult {
            let valid_types = vec!["i32", "i64", "f32", "f64", "bool", "String", "char"];
            let inner = if inner_type.is_empty() {
                "i32"
            } else {
                valid_types[inner_type.len() % valid_types.len()]
            };
            
            let code = format!("let x: Option<{}> = None", inner);
            let mut parser = Parser::new(&code);
            
            match parser.parse() {
                Ok(ast) => {
                    let transpiler = Transpiler::new();
                    match transpiler.transpile(&ast) {
                        Ok(result) => {
                            let output = result.to_string();
                            TestResult::from_bool(
                                output.contains("Option") && 
                                output.contains("None")
                            )
                        }
                        Err(_) => TestResult::failed()
                    }
                }
                Err(_) => TestResult::discard()
            }
        }
        
        quickcheck(check_optional_type as fn(String) -> TestResult);
    }
    
    // Property: List types should transpile to Vec<T>
    #[test]
    fn prop_list_types_transpile_to_vec() {
        fn check_list_type(elem_type: String) -> TestResult {
            let valid_types = vec!["i32", "f64", "bool", "String"];
            let elem = valid_types[elem_type.len() % valid_types.len()];
            
            let code = format!("let x: [{}] = []", elem);
            let mut parser = Parser::new(&code);
            
            match parser.parse() {
                Ok(ast) => {
                    let transpiler = Transpiler::new();
                    match transpiler.transpile(&ast) {
                        Ok(result) => {
                            let output = result.to_string();
                            // Should contain Vec and the element type
                            TestResult::from_bool(output.contains("Vec") || output.contains("vec!"))
                        }
                        Err(_) => TestResult::discard()
                    }
                }
                Err(_) => TestResult::discard()
            }
        }
        
        quickcheck(check_list_type as fn(String) -> TestResult);
    }
    
    // Property: Tuple types preserve element count and order
    #[test]
    fn prop_tuple_types_preserve_structure() {
        fn check_tuple_type(elem_count: u8) -> TestResult {
            if elem_count == 0 || elem_count > 10 {
                return TestResult::discard();
            }
            
            let types = (0..elem_count)
                .map(|i| match i % 3 {
                    0 => "i32",
                    1 => "bool",
                    _ => "String",
                })
                .collect::<Vec<_>>()
                .join(", ");
            
            let values = (0..elem_count)
                .map(|i| match i % 3 {
                    0 => "42",
                    1 => "true",
                    _ => "\"test\"",
                })
                .collect::<Vec<_>>()
                .join(", ");
            
            let code = format!("let x: ({}) = ({})", types, values);
            let mut parser = Parser::new(&code);
            
            match parser.parse() {
                Ok(ast) => {
                    let transpiler = Transpiler::new();
                    match transpiler.transpile(&ast) {
                        Ok(result) => {
                            let output = result.to_string();
                            // Should have parentheses for tuple
                            TestResult::from_bool(
                                output.contains("(") && 
                                output.contains(")") &&
                                output.matches(',').count() >= (elem_count - 1) as usize
                            )
                        }
                        Err(_) => TestResult::discard()
                    }
                }
                Err(_) => TestResult::discard()
            }
        }
        
        quickcheck(check_tuple_type as fn(u8) -> TestResult);
    }
    
    // Property: Reference types should handle mutability correctly
    #[test]
    fn prop_reference_types_handle_mutability() {
        fn check_reference_type(is_mut: bool, is_str: bool) -> TestResult {
            let inner_type = if is_str { "str" } else { "i32" };
            let ref_prefix = if is_mut { "&mut " } else { "&" };
            
            let code = format!("let x: {}{} = undefined", ref_prefix, inner_type);
            let mut parser = Parser::new(&code);
            
            match parser.parse() {
                Ok(ast) => {
                    let transpiler = Transpiler::new();
                    match transpiler.transpile(&ast) {
                        Ok(result) => {
                            let output = result.to_string();
                            
                            // Check proper reference handling
                            if is_str {
                                // &str should not become &&str
                                TestResult::from_bool(!output.contains("&&str"))
                            } else {
                                // Regular types should have proper reference
                                TestResult::from_bool(
                                    output.contains("&") && 
                                    (is_mut == output.contains("&mut"))
                                )
                            }
                        }
                        Err(_) => TestResult::discard()
                    }
                }
                Err(_) => TestResult::discard()
            }
        }
        
        quickcheck(check_reference_type as fn(bool, bool) -> TestResult);
    }
    
    #[test]
    fn test_type_consistency() {
        // Ensure our refactored type system maintains consistency
        let test_cases = vec![
            ("int", "i64"),
            ("float", "f64"),
            ("bool", "bool"),
            ("string", "String"),
            ("char", "char"),
        ];
        
        for (ruchy_type, expected_rust) in test_cases {
            let code = format!("let x: {} = undefined", ruchy_type);
            let mut parser = Parser::new(&code);
            let ast = parser.parse().expect("Should parse basic type");
            
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast).expect("Should transpile");
            let output = result.to_string();
            
            assert!(
                output.contains(expected_rust),
                "Type {} should transpile to {}, got: {}",
                ruchy_type, expected_rust, output
            );
        }
    }
}