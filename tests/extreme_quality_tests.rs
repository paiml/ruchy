// EXTREME QUALITY TDD Tests - Following Toyota Way Principles
// Every test has single responsibility and complexity ≤10
// No shortcuts, no hacks, only quality

use ruchy::compile;

#[cfg(test)]
mod parser_fundamentals {
    use super::*;

    // Single Responsibility: Test integer parsing
    #[test]
    fn test_parse_positive_integer() {
        let code = "fn main() { let x = 42; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse positive integer");
    }

    #[test]
    fn test_parse_zero() {
        let code = "fn main() { let x = 0; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse zero");
    }

    #[test]
    fn test_parse_negative_integer() {
        let code = "fn main() { let x = -42; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse negative integer");
    }

    // Single Responsibility: Test float parsing
    #[test]
    fn test_parse_simple_float() {
        let code = "fn main() { let x = 3.14; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse simple float");
    }

    #[test]
    fn test_parse_float_no_decimal() {
        let code = "fn main() { let x = 1.0; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse float with .0");
    }

    // Single Responsibility: Test boolean parsing
    #[test]
    fn test_parse_true() {
        let code = "fn main() { let x = true; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse true");
    }

    #[test]
    fn test_parse_false() {
        let code = "fn main() { let x = false; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should parse false");
    }
}

#[cfg(test)]
mod binary_operations {
    use super::*;

    // Single Responsibility: Test addition
    #[test]
    fn test_add_two_numbers() {
        let code = "fn main() { let x = 1 + 2; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile addition");
    }

    // Single Responsibility: Test subtraction
    #[test]
    fn test_subtract_two_numbers() {
        let code = "fn main() { let x = 5 - 3; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile subtraction");
    }

    // Single Responsibility: Test multiplication
    #[test]
    fn test_multiply_two_numbers() {
        let code = "fn main() { let x = 3 * 4; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile multiplication");
    }

    // Single Responsibility: Test division
    #[test]
    fn test_divide_two_numbers() {
        let code = "fn main() { let x = 10 / 2; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile division");
    }

    // Single Responsibility: Test modulo
    #[test]
    fn test_modulo_operation() {
        let code = "fn main() { let x = 10 % 3; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile modulo");
    }
}

#[cfg(test)]
mod comparison_operations {
    use super::*;

    #[test]
    fn test_less_than() {
        let code = "fn main() { let x = 1 < 2; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile less than");
    }

    #[test]
    fn test_greater_than() {
        let code = "fn main() { let x = 2 > 1; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile greater than");
    }

    #[test]
    fn test_less_than_or_equal() {
        let code = "fn main() { let x = 1 <= 2; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile less than or equal");
    }

    #[test]
    fn test_greater_than_or_equal() {
        let code = "fn main() { let x = 2 >= 1; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile greater than or equal");
    }

    #[test]
    fn test_equality() {
        let code = "fn main() { let x = 1 == 1; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile equality");
    }

    #[test]
    fn test_inequality() {
        let code = "fn main() { let x = 1 != 2; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile inequality");
    }
}

#[cfg(test)]
mod logical_operations {
    use super::*;

    #[test]
    fn test_logical_and() {
        let code = "fn main() { let x = true && false; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile logical AND");
    }

    #[test]
    fn test_logical_or() {
        let code = "fn main() { let x = true || false; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile logical OR");
    }

    #[test]
    fn test_logical_not() {
        let code = "fn main() { let x = !true; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile logical NOT");
    }
}

#[cfg(test)]
mod variable_declarations {
    use super::*;

    #[test]
    fn test_simple_let_binding() {
        let code = "fn main() { let x = 1; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile let binding");
    }

    #[test]
    fn test_let_with_type_annotation() {
        let code = "fn main() { let x: i32 = 1; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile typed let binding");
    }

    #[test]
    fn test_mutable_let_binding() {
        let code = "fn main() { let mut x = 1; }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile mutable binding");
    }

    #[test]
    #[ignore = "const keyword not yet supported"]
    fn test_const_declaration() {
        let code = "const PI: f64 = 3.14159;";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile const declaration");
    }

    #[test]
    #[ignore = "static keyword not yet supported"]
    fn test_static_declaration() {
        let code = "static COUNT: i32 = 0;";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile static declaration");
    }
}

#[cfg(test)]
mod function_definitions {
    use super::*;

    #[test]
    fn test_simple_function() {
        let code = "fn hello() {}";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile simple function");
    }

    #[test]
    fn test_function_with_params() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile function with parameters");
    }

    #[test]
    fn test_function_with_return() {
        let code = "fn get_value() -> i32 { 42 }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile function with return type");
    }

    #[test]
    fn test_function_with_explicit_return() {
        let code = "fn get_value() -> i32 { return 42; }";
        let result = compile(code);
        assert!(
            result.is_ok(),
            "Should compile function with return statement"
        );
    }

    #[test]
    fn test_recursive_function() {
        let code = "fn factorial(n: u32) -> u32 { if n <= 1 { 1 } else { n * factorial(n - 1) } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile recursive function");
    }
}

#[cfg(test)]
mod control_flow {
    use super::*;

    #[test]
    fn test_simple_if() {
        let code = "fn main() { if true { } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile simple if");
    }

    #[test]
    fn test_if_else() {
        let code = "fn main() { if true { } else { } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile if-else");
    }

    #[test]
    fn test_if_else_if() {
        let code = "fn main() { if true { } else if false { } else { } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile if-else-if");
    }

    #[test]
    fn test_match_statement() {
        let code = "fn main() { match 1 { 1 => {}, _ => {} } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile match statement");
    }

    #[test]
    fn test_for_loop() {
        let code = "fn main() { for i in 0..10 { } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile for loop");
    }

    #[test]
    fn test_while_loop() {
        let code = "fn main() { while true { break; } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile while loop");
    }

    #[test]
    fn test_loop() {
        let code = "fn main() { loop { break; } }";
        let result = compile(code);
        assert!(result.is_ok(), "Should compile loop");
    }
}
