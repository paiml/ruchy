// TDD tests for comprehensive type conversion support
// This tests type casting, conversion functions, and automatic coercion

use ruchy::runtime::repl::Repl;

#[test]
fn test_explicit_type_casting() {
    let mut repl = Repl::new().unwrap();
    
    // Integer to float casting
    let result = repl.eval("42 as float").unwrap();
    assert_eq!(result, "42");
    
    // Float to integer casting
    let result = repl.eval("3.14 as int").unwrap();
    assert_eq!(result, "3");
    
    // Boolean to integer casting
    let result = repl.eval("true as int").unwrap();
    assert_eq!(result, "1");
    
    let result = repl.eval("false as int").unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_string_to_number_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // String to int
    let result = repl.eval(r#"int("42")"#).unwrap();
    assert_eq!(result, "42");
    
    let result = repl.eval(r#"int("-123")"#).unwrap();
    assert_eq!(result, "-123");
    
    // String to float
    let result = repl.eval(r#"float("3.14")"#).unwrap();
    assert_eq!(result, "3.14");
    
    let result = repl.eval(r#"float("-2.5")"#).unwrap();
    assert_eq!(result, "-2.5");
    
    // Invalid conversions should error
    assert!(repl.eval(r#"int("hello")"#).is_err());
    assert!(repl.eval(r#"float("world")"#).is_err());
}

#[test]
fn test_number_to_string_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Int to string
    let result = repl.eval("str(42)").unwrap();
    assert_eq!(result, r#""42""#);
    
    let result = repl.eval("str(-123)").unwrap();
    assert_eq!(result, r#""-123""#);
    
    // Float to string
    let result = repl.eval("str(3.14)").unwrap();
    assert_eq!(result, r#""3.14""#);
    
    // Using to_string method
    let result = repl.eval("42.to_string()").unwrap();
    assert_eq!(result, r#""42""#);
}

#[test]
fn test_boolean_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Bool to int
    let result = repl.eval("int(true)").unwrap();
    assert_eq!(result, "1");
    
    let result = repl.eval("int(false)").unwrap();
    assert_eq!(result, "0");
    
    // Bool to string
    let result = repl.eval("str(true)").unwrap();
    assert_eq!(result, r#""true""#);
    
    let result = repl.eval("str(false)").unwrap();
    assert_eq!(result, r#""false""#);
    
    // String to bool
    let result = repl.eval(r#"bool("true")"#).unwrap();
    assert_eq!(result, "true");
    
    let result = repl.eval(r#"bool("false")"#).unwrap();
    assert_eq!(result, "false");
    
    let result = repl.eval(r#"bool("")"#).unwrap();
    assert_eq!(result, "false");
    
    let result = repl.eval(r#"bool("anything")"#).unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_numeric_coercion_in_operations() {
    let mut repl = Repl::new().unwrap();
    
    // Int + Float should produce Float
    let result = repl.eval("5 + 2.5").unwrap();
    assert_eq!(result, "7.5");
    
    // Float + Int should produce Float
    let result = repl.eval("3.0 + 7").unwrap();
    assert_eq!(result, "10");
    
    // Mixed multiplication
    let result = repl.eval("4 * 2.5").unwrap();
    assert_eq!(result, "10");
    
    // Mixed division always produces float
    let result = repl.eval("10 / 3").unwrap();
    assert!(result.starts_with("3.333"));
}

#[test]
fn test_char_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Char to string
    let result = repl.eval("str('a')").unwrap();
    assert_eq!(result, r#""a""#);
    
    // Char to int (ASCII value)
    let result = repl.eval("'A'.to_int()").unwrap();
    assert_eq!(result, "65");
    
    // Int to char
    let result = repl.eval("char(65)").unwrap();
    assert_eq!(result, "'A'");
    
    let result = repl.eval("char(97)").unwrap();
    assert_eq!(result, "'a'");
}

#[test]
fn test_list_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // List to string representation
    let result = repl.eval("str([1, 2, 3])").unwrap();
    assert_eq!(result, r#""[1, 2, 3]""#);
    
    // String to char list
    let result = repl.eval(r#""abc".chars()"#).unwrap();
    assert_eq!(result, r#"["a", "b", "c"]"#);
    
    // String to byte list
    let result = repl.eval(r#""ABC".bytes()"#).unwrap();
    assert_eq!(result, "[65, 66, 67]");
}

#[test]
fn test_tuple_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Tuple to string
    let result = repl.eval("str((1, 2, 3))").unwrap();
    assert_eq!(result, r#""(1, 2, 3)""#);
    
    // Tuple to list (if supported)
    let result = repl.eval("list((1, 2, 3))").unwrap();
    assert_eq!(result, "[1, 2, 3]");
    
    // List to tuple (if supported)
    let result = repl.eval("tuple([1, 2, 3])").unwrap();
    assert_eq!(result, "(1, 2, 3)");
}

#[test]
fn test_hex_octal_binary_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Hex string to int
    let result = repl.eval(r#"int("0xFF", 16)"#).unwrap();
    assert_eq!(result, "255");
    
    // Binary string to int
    let result = repl.eval(r#"int("1010", 2)"#).unwrap();
    assert_eq!(result, "10");
    
    // Octal string to int
    let result = repl.eval(r#"int("77", 8)"#).unwrap();
    assert_eq!(result, "63");
    
    // Int to hex string
    let result = repl.eval("hex(255)").unwrap();
    assert_eq!(result, r#""0xff""#);
    
    // Int to binary string
    let result = repl.eval("bin(10)").unwrap();
    assert_eq!(result, r#""0b1010""#);
    
    // Int to octal string
    let result = repl.eval("oct(63)").unwrap();
    assert_eq!(result, r#""0o77""#);
}

#[test]
fn test_option_result_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Option to Result
    let result = repl.eval("Option::Some(42).ok_or(\"error\")").unwrap();
    assert_eq!(result, "Result::Ok(42)");
    
    let result = repl.eval("Option::None.ok_or(\"error\")").unwrap();
    assert_eq!(result, r#"Result::Err("error")"#);
    
    // Result to Option
    let result = repl.eval("Result::Ok(42).ok()").unwrap();
    assert_eq!(result, "Option::Some(42)");
    
    let result = repl.eval("Result::Err(\"error\").ok()").unwrap();
    assert_eq!(result, "Option::None");
}

#[test]
fn test_chained_conversions() {
    let mut repl = Repl::new().unwrap();
    
    // Multiple conversions in sequence
    let result = repl.eval(r#"str(int(float("3.14")))"#).unwrap();
    assert_eq!(result, r#""3""#);
    
    // Complex conversion chain
    let result = repl.eval("bool(int(str(true)))").unwrap();
    assert_eq!(result, "true");
}