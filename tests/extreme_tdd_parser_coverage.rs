// EXTREME TDD: Parser Coverage Boost
// Target: imports.rs and types.rs with 0 tests
// Complexity: <10 per test
// Single responsibility, zero technical debt

use ruchy::frontend::parser::Parser;
// ExprKind and Type imports removed - not used

#[cfg(test)]
mod parser_imports_tests {
    use super::*;

    #[test]
    fn test_use_statement() {
        let mut parser = Parser::new("use std::io");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_with_alias() {
        let mut parser = Parser::new("use std::collections::HashMap as HM");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_multiple_items() {
        let mut parser = Parser::new("use std::{io, fs, path}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_nested() {
        let mut parser = Parser::new("use std::collections::{HashMap, HashSet}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_wildcard() {
        let mut parser = Parser::new("use std::prelude::*");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_statement() {
        let mut parser = Parser::new("import math");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_import() {
        let mut parser = Parser::new("from math import sqrt");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_import_multiple() {
        let mut parser = Parser::new("from math import sqrt, sin, cos");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_import_as() {
        let mut parser = Parser::new("from math import sqrt as square_root");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_statement() {
        let mut parser = Parser::new("export fn public_func() { 42 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_module_declaration() {
        let mut parser = Parser::new("mod utils");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_pub_use() {
        let mut parser = Parser::new("pub use crate::types::*");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_extern_crate() {
        let mut parser = Parser::new("extern crate serde");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_self() {
        let mut parser = Parser::new("use self::module::Type");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_super() {
        let mut parser = Parser::new("use super::parent_module");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod parser_types_tests {
    use super::*;

    #[test]
    fn test_type_annotation_basic() {
        let mut parser = Parser::new("let x: i32 = 42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_float() {
        let mut parser = Parser::new("let x: f64 = 3.14");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_string() {
        let mut parser = Parser::new("let x: str = \"hello\"");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_bool() {
        let mut parser = Parser::new("let x: bool = true");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_array() {
        let mut parser = Parser::new("let x: [i32] = [1, 2, 3]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_tuple() {
        let mut parser = Parser::new("let x: (i32, str) = (42, \"hello\")");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_option() {
        let mut parser = Parser::new("let x: Option<i32> = Some(42)");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_result() {
        let mut parser = Parser::new("let x: Result<i32, str> = Ok(42)");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_vec() {
        let mut parser = Parser::new("let x: Vec<i32> = vec![1, 2, 3]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_annotation_hashmap() {
        let mut parser = Parser::new("let x: HashMap<str, i32> = HashMap::new()");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_return_type() {
        let mut parser = Parser::new("fn add(x: i32, y: i32) -> i32 { x + y }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_function() {
        let mut parser = Parser::new("fn identity<T>(x: T) -> T { x }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_impl_block() {
        let mut parser = Parser::new("impl MyStruct { fn new() -> Self { Self } }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_definition() {
        let mut parser = Parser::new("trait Display { fn fmt(&self) -> str }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_alias() {
        let mut parser = Parser::new("type MyInt = i32");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_const_type() {
        let mut parser = Parser::new("const MAX: i32 = 100");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_static_type() {
        let mut parser = Parser::new("static COUNTER: i32 = 0");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_type() {
        let mut parser = Parser::new("let x: &i32 = &42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mutable_reference() {
        let mut parser = Parser::new("let x: &mut i32 = &mut 42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_lifetime_annotation() {
        let mut parser = Parser::new("fn longest<'a>(x: &'a str, y: &'a str) -> &'a str { x }");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod parser_edge_cases {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut parser = Parser::new("");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_whitespace() {
        let mut parser = Parser::new("   \n\t  \n  ");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_comments() {
        let mut parser = Parser::new("// just a comment\n/* block comment */");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_identifiers() {
        let mut parser = Parser::new("let π = 3.14159");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_emoji_in_string() {
        let mut parser = Parser::new("let msg = \"Hello 🌍\"");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_very_long_identifier() {
        let long_name = "a".repeat(100);
        let code = format!("let {} = 42", long_name);
        let mut parser = Parser::new(&code);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_deeply_nested_parens() {
        let mut parser = Parser::new("((((((((((42))))))))))");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_many_operators() {
        let mut parser = Parser::new("1 + 2 * 3 - 4 / 5 % 6 ** 2");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_method_calls() {
        let mut parser = Parser::new("x.foo().bar().baz()");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_field_access() {
        let mut parser = Parser::new("obj.field1.field2.field3");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_brackets() {
        let mut parser = Parser::new("arr[obj.field[index]]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_lambda_in_call() {
        let mut parser = Parser::new("map(|x| x * 2, [1, 2, 3])");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_negative_numbers() {
        let mut parser = Parser::new("-42 + -3.14");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_scientific_notation() {
        let mut parser = Parser::new("1.23e45 + 6.78E-90");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hex_octal_binary() {
        let mut parser = Parser::new("0xFF + 0o777 + 0b1010");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_underscore_numbers() {
        let mut parser = Parser::new("1_000_000 + 0xFF_FF");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_raw_string() {
        let mut parser = Parser::new(r#"r"raw\nstring""#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_byte_string() {
        let mut parser = Parser::new(r#"b"bytes""#);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_char_literal() {
        let mut parser = Parser::new("'a' + 'b'");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_escape_sequences() {
        let mut parser = Parser::new(r#""escaped: \n \t \r \\ \"quote\"" "#);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
