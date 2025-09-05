//! TDD tests for types.rs to boost coverage from 34.92% to 50%+
//! Target: Test all type transpilation functions with complexity ≤10

#[cfg(test)]
mod tests {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;
    
    // Helper function (complexity: 3)
    fn transpile_str(input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        let transpiler = Transpiler::default();
        Ok(transpiler.transpile_to_string(&expr)?)
    }
    
    // Basic type tests (complexity: 2 each)
    #[test]
    fn test_transpile_type_i32() {
        let result = transpile_str("let x: i32 = 42");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("i32"));
        assert!(output.contains("42"));
    }
    
    #[test]
    fn test_transpile_type_f64() {
        let result = transpile_str("let x: f64 = 3.14");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("f64"));
        assert!(output.contains("3.14"));
    }
    
    #[test]
    fn test_transpile_type_bool() {
        let result = transpile_str("let x: bool = true");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("bool"));
        assert!(output.contains("true"));
    }
    
    #[test]
    fn test_transpile_type_string() {
        let result = transpile_str("let x: String = \"hello\"");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("String"));
        assert!(output.contains("hello"));
    }
    
    // Generic type tests (complexity: 3 each)
    #[test]
    fn test_transpile_type_vec() {
        let result = transpile_str("let v: Vec<i32> = vec![1, 2, 3]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Vec"));
        assert!(output.contains("i32"));
    }
    
    #[test]
    fn test_transpile_type_option() {
        let result = transpile_str("let opt: Option<String> = Some(\"test\")");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Option"));
        assert!(output.contains("String"));
    }
    
    #[test]
    fn test_transpile_type_result() {
        let result = transpile_str("let res: Result<i32, String> = Ok(42)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Result"));
        assert!(output.contains("i32"));
        assert!(output.contains("String"));
    }
    
    #[test]
    fn test_transpile_type_hashmap() {
        let result = transpile_str("let map: HashMap<String, i32> = HashMap::new()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("HashMap"));
    }
    
    // Nested generic types (complexity: 4 each)
    #[test]
    fn test_transpile_type_nested_vec() {
        let result = transpile_str("let matrix: Vec<Vec<i32>> = vec![vec![1, 2], vec![3, 4]]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Vec"));
    }
    
    #[test]
    fn test_transpile_type_option_vec() {
        let result = transpile_str("let opt_vec: Option<Vec<String>> = Some(vec![\"a\".to_string()])");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Option") && output.contains("Vec"));
    }
    
    // Struct tests (complexity: 4 each)
    #[test]
    fn test_transpile_struct_simple() {
        let result = transpile_str("struct Point { x: i32, y: i32 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("struct Point"));
        assert!(output.contains("x:") && output.contains("y:"));
        assert!(output.contains("i32"));
    }
    
    #[test]
    fn test_transpile_struct_with_generics() {
        let result = transpile_str("struct Container<T> { value: T }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("struct Container"));
        assert!(output.contains("<T>"));
        assert!(output.contains("value:"));
    }
    
    #[test]
    fn test_transpile_struct_public() {
        let result = transpile_str("pub struct PublicPoint { pub x: i32, pub y: i32 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("pub struct"));
        assert!(output.contains("pub x:"));
        assert!(output.contains("pub y:"));
    }
    
    #[test]
    fn test_transpile_struct_with_methods() {
        let result = transpile_str("struct Point { x: i32, y: i32 } impl Point { fn new(x: i32, y: i32) -> Point { Point { x, y } } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("struct Point"));
        assert!(output.contains("impl Point"));
        assert!(output.contains("fn new"));
    }
    
    // Enum tests (complexity: 4 each)
    #[test]
    fn test_transpile_enum_simple() {
        let result = transpile_str("enum Color { Red, Green, Blue }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("enum Color"));
        assert!(output.contains("Red") && output.contains("Green") && output.contains("Blue"));
    }
    
    #[test]
    fn test_transpile_enum_with_data() {
        let result = transpile_str("enum Message { Text(String), Number(i32) }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("enum Message"));
        assert!(output.contains("Text(String)"));
        assert!(output.contains("Number(i32)"));
    }
    
    #[test]
    fn test_transpile_enum_public() {
        let result = transpile_str("pub enum Status { Active, Inactive }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("pub enum Status"));
        assert!(output.contains("Active") && output.contains("Inactive"));
    }
    
    #[test]
    fn test_transpile_enum_with_generics() {
        let result = transpile_str("enum Option<T> { Some(T), None }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("enum Option"));
        assert!(output.contains("<T>"));
        assert!(output.contains("Some(T)") && output.contains("None"));
    }
    
    // Trait tests (complexity: 4 each)
    #[test]
    fn test_transpile_trait_simple() {
        let result = transpile_str("trait Display { fn fmt(&self) -> String; }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("trait Display"));
        assert!(output.contains("fn fmt"));
        assert!(output.contains("&self"));
        assert!(output.contains("String"));
    }
    
    #[test]
    fn test_transpile_trait_with_default() {
        let result = transpile_str("trait Debug { fn debug(&self) -> String { \"Debug\".to_string() } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("trait Debug"));
        assert!(output.contains("fn debug"));
    }
    
    #[test]
    fn test_transpile_trait_public() {
        let result = transpile_str("pub trait Clone { fn clone(&self) -> Self; }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("pub trait Clone"));
        assert!(output.contains("fn clone"));
        assert!(output.contains("Self"));
    }
    
    #[test]
    fn test_transpile_trait_with_generics() {
        let result = transpile_str("trait Into<T> { fn into(self) -> T; }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("trait Into"));
        assert!(output.contains("<T>"));
        assert!(output.contains("fn into"));
    }
    
    // Impl tests (complexity: 4 each)
    #[test]
    fn test_transpile_impl_basic() {
        let result = transpile_str("impl Point { fn new() -> Point { Point { x: 0, y: 0 } } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("impl Point"));
        assert!(output.contains("fn new"));
    }
    
    #[test]
    fn test_transpile_impl_for_trait() {
        let result = transpile_str("impl Display for Point { fn fmt(&self) -> String { format!(\"({}, {})\", self.x, self.y) } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("impl Display for Point"));
        assert!(output.contains("fn fmt"));
    }
    
    #[test]
    fn test_transpile_impl_with_generics() {
        let result = transpile_str("impl<T> Container<T> { fn new(value: T) -> Container<T> { Container { value } } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("impl<T> Container<T>"));
        assert!(output.contains("fn new"));
    }
    
    #[test]
    fn test_transpile_impl_methods() {
        let result = transpile_str("impl Point { fn x(&self) -> i32 { self.x } fn y(&self) -> i32 { self.y } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn x"));
        assert!(output.contains("fn y"));
        assert!(output.contains("&self"));
    }
    
    // Function type tests (complexity: 3 each)
    #[test]
    fn test_transpile_function_type() {
        let result = transpile_str("let f: fn(i32) -> i32 = |x| x + 1");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn(i32) -> i32"));
    }
    
    #[test]
    fn test_transpile_closure_type() {
        let result = transpile_str("let closure: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 2)");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Fn(i32) -> i32"));
    }
    
    // Array and slice type tests (complexity: 3 each)
    #[test]
    fn test_transpile_array_type() {
        let result = transpile_str("let arr: [i32; 5] = [1, 2, 3, 4, 5]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("[i32; 5]"));
    }
    
    #[test]
    fn test_transpile_slice_type() {
        let result = transpile_str("let slice: &[i32] = &[1, 2, 3]");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("&[i32]"));
    }
    
    // Tuple type tests (complexity: 3 each)
    #[test]
    fn test_transpile_tuple_type() {
        let result = transpile_str("let tuple: (i32, String) = (42, \"hello\".to_string())");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("(i32, String)"));
    }
    
    #[test]
    fn test_transpile_unit_type() {
        let result = transpile_str("let unit: () = ()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("()"));
    }
    
    // Reference type tests (complexity: 3 each)
    #[test]
    fn test_transpile_reference_type() {
        let result = transpile_str("let r: &i32 = &42");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("&i32"));
    }
    
    #[test]
    fn test_transpile_mutable_reference_type() {
        let result = transpile_str("let r: &mut i32 = &mut 42");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("&mut i32"));
    }
    
    // Pointer type tests (complexity: 3 each)
    #[test]
    fn test_transpile_raw_pointer_type() {
        let result = transpile_str("let ptr: *const i32 = std::ptr::null()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("*const i32"));
    }
    
    #[test]
    fn test_transpile_mutable_raw_pointer_type() {
        let result = transpile_str("let ptr: *mut i32 = std::ptr::null_mut()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("*mut i32"));
    }
    
    // Lifetime tests (complexity: 4 each)
    #[test]
    fn test_transpile_lifetime_reference() {
        let result = transpile_str("fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { if x.len() > y.len() { x } else { y } }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("'a"));
    }
    
    #[test]
    fn test_transpile_struct_with_lifetime() {
        let result = transpile_str("struct StringRef<'a> { s: &'a str }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("'a"));
        assert!(output.contains("&'a str"));
    }
    
    // Complex nested type tests (complexity: 5 each)
    #[test]
    fn test_transpile_complex_nested_type() {
        let result = transpile_str("let complex: HashMap<String, Vec<Option<Result<i32, String>>>> = HashMap::new()");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("HashMap"));
        assert!(output.contains("String"));
        assert!(output.contains("Vec"));
        assert!(output.contains("Option"));
        assert!(output.contains("Result"));
    }
    
    #[test]
    fn test_transpile_function_with_complex_types() {
        let result = transpile_str("fn process(data: Vec<HashMap<String, Option<i32>>>) -> Result<Vec<String>, Box<dyn std::error::Error>> { Ok(vec![]) }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("fn process"));
        assert!(output.contains("Vec"));
        assert!(output.contains("HashMap"));
        assert!(output.contains("Option"));
        assert!(output.contains("Result"));
    }
    
    // Property test / attribute tests (complexity: 3 each)
    #[test]
    fn test_transpile_derive_attribute() {
        let result = transpile_str("#[derive(Debug, Clone)] struct Point { x: i32, y: i32 }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("derive") || output.contains("Debug") || output.contains("Clone"));
    }
    
    #[test]
    fn test_transpile_cfg_attribute() {
        let result = transpile_str("#[cfg(test)] fn test_function() { }");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("cfg") || output.contains("test"));
    }
    
    // Error handling tests (complexity: 2 each)
    #[test]
    fn test_transpile_invalid_type_syntax() {
        let result = transpile_str("let x: << = 42"); // Invalid type syntax
        assert!(result.is_err());
    }
    
    #[test]
    fn test_transpile_incomplete_type() {
        let result = transpile_str("let x: Vec< = vec![]"); // Incomplete generic
        assert!(result.is_err());
    }
}