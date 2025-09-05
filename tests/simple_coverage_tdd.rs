//! Extremely simple TDD tests for quick coverage wins
//! Target: Boost coverage with minimal complexity (≤5)

#[cfg(test)]
mod tests {
    use ruchy::frontend::parser::Parser;
    
    // Test group 1: Numbers (complexity: 2 each)
    #[test]
    fn test_int_0() {
        assert!(Parser::new("0").parse_expr().is_ok());
    }
    
    #[test]
    fn test_int_negative() {
        assert!(Parser::new("-42").parse_expr().is_ok());
    }
    
    #[test]
    fn test_float_simple() {
        assert!(Parser::new("1.0").parse_expr().is_ok());
    }
    
    #[test]
    fn test_float_scientific() {
        assert!(Parser::new("1e10").parse_expr().is_ok());
    }
    
    // Test group 2: Strings (complexity: 2 each)
    #[test]
    fn test_string_empty() {
        assert!(Parser::new(r#""""#).parse_expr().is_ok());
    }
    
    #[test]
    fn test_string_single_char() {
        assert!(Parser::new(r#""a""#).parse_expr().is_ok());
    }
    
    #[test]
    fn test_string_multichar() {
        assert!(Parser::new(r#""hello""#).parse_expr().is_ok());
    }
    
    // Test group 3: Booleans (complexity: 2 each)
    #[test]
    fn test_true() {
        assert!(Parser::new("true").parse_expr().is_ok());
    }
    
    #[test]
    fn test_false() {
        assert!(Parser::new("false").parse_expr().is_ok());
    }
    
    // Test group 4: Variables (complexity: 2 each)
    #[test]
    fn test_var_simple() {
        assert!(Parser::new("x").parse_expr().is_ok());
    }
    
    #[test]
    fn test_var_underscore() {
        assert!(Parser::new("_var").parse_expr().is_ok());
    }
    
    #[test]
    fn test_var_with_numbers() {
        assert!(Parser::new("var123").parse_expr().is_ok());
    }
    
    // Test group 5: Binary ops (complexity: 3 each)
    #[test]
    fn test_add() {
        assert!(Parser::new("1 + 2").parse_expr().is_ok());
    }
    
    #[test]
    fn test_subtract() {
        assert!(Parser::new("3 - 1").parse_expr().is_ok());
    }
    
    #[test]
    fn test_multiply() {
        assert!(Parser::new("2 * 3").parse_expr().is_ok());
    }
    
    #[test]
    fn test_divide() {
        assert!(Parser::new("6 / 2").parse_expr().is_ok());
    }
    
    #[test]
    fn test_modulo() {
        assert!(Parser::new("5 % 2").parse_expr().is_ok());
    }
    
    // Test group 6: Comparisons (complexity: 3 each)
    #[test]
    fn test_eq() {
        assert!(Parser::new("1 == 1").parse_expr().is_ok());
    }
    
    #[test]
    fn test_ne() {
        assert!(Parser::new("1 != 2").parse_expr().is_ok());
    }
    
    #[test]
    fn test_lt() {
        assert!(Parser::new("1 < 2").parse_expr().is_ok());
    }
    
    #[test]
    fn test_gt() {
        assert!(Parser::new("2 > 1").parse_expr().is_ok());
    }
    
    #[test]
    fn test_le() {
        assert!(Parser::new("1 <= 2").parse_expr().is_ok());
    }
    
    #[test]
    fn test_ge() {
        assert!(Parser::new("2 >= 1").parse_expr().is_ok());
    }
    
    // Test group 7: Logical ops (complexity: 3 each)
    #[test]
    fn test_and() {
        assert!(Parser::new("true && false").parse_expr().is_ok());
    }
    
    #[test]
    fn test_or() {
        assert!(Parser::new("true || false").parse_expr().is_ok());
    }
    
    #[test]
    fn test_not() {
        assert!(Parser::new("!true").parse_expr().is_ok());
    }
    
    // Test group 8: Grouping (complexity: 3 each)
    #[test]
    fn test_parens() {
        assert!(Parser::new("(42)").parse_expr().is_ok());
    }
    
    #[test]
    fn test_nested_parens() {
        assert!(Parser::new("((1))").parse_expr().is_ok());
    }
    
    #[test]
    fn test_parens_expr() {
        assert!(Parser::new("(1 + 2)").parse_expr().is_ok());
    }
    
    // Test group 9: Lists (complexity: 3 each)
    #[test]
    fn test_list_empty() {
        assert!(Parser::new("[]").parse_expr().is_ok());
    }
    
    #[test]
    fn test_list_single() {
        assert!(Parser::new("[1]").parse_expr().is_ok());
    }
    
    #[test]
    fn test_list_multiple() {
        assert!(Parser::new("[1, 2, 3]").parse_expr().is_ok());
    }
    
    // Test group 10: Objects (complexity: 3 each)
    #[test]
    fn test_object_empty() {
        assert!(Parser::new("{}").parse_expr().is_ok());
    }
    
    #[test]
    fn test_object_single() {
        assert!(Parser::new("{x: 1}").parse_expr().is_ok());
    }
    
    #[test]
    fn test_object_multiple() {
        assert!(Parser::new("{x: 1, y: 2}").parse_expr().is_ok());
    }
    
    // Test group 11: Calls (complexity: 3 each)
    #[test]
    fn test_call_no_args() {
        assert!(Parser::new("func()").parse_expr().is_ok());
    }
    
    #[test]
    fn test_call_one_arg() {
        assert!(Parser::new("func(1)").parse_expr().is_ok());
    }
    
    #[test]
    fn test_call_multiple_args() {
        assert!(Parser::new("func(1, 2, 3)").parse_expr().is_ok());
    }
    
    // Test group 12: Member access (complexity: 3 each)
    #[test]
    fn test_dot_access() {
        assert!(Parser::new("obj.field").parse_expr().is_ok());
    }
    
    #[test]
    fn test_dot_chain() {
        assert!(Parser::new("obj.field.nested").parse_expr().is_ok());
    }
    
    #[test]
    fn test_index_access() {
        assert!(Parser::new("arr[0]").parse_expr().is_ok());
    }
    
    // Test group 13: Let bindings (complexity: 3 each)
    #[test]
    fn test_let_simple() {
        assert!(Parser::new("let x = 1").parse_expr().is_ok());
    }
    
    #[test]
    fn test_let_with_type() {
        assert!(Parser::new("let x: Int = 1").parse_expr().is_ok());
    }
    
    #[test]
    fn test_let_destructure() {
        assert!(Parser::new("let [a, b] = [1, 2]").parse_expr().is_ok());
    }
    
    // Test group 14: If expressions (complexity: 4)
    #[test]
    fn test_if_simple() {
        assert!(Parser::new("if true { 1 }").parse_expr().is_ok());
    }
    
    #[test]
    fn test_if_else() {
        assert!(Parser::new("if true { 1 } else { 2 }").parse_expr().is_ok());
    }
    
    #[test]
    fn test_if_elif() {
        assert!(Parser::new("if true { 1 } elif false { 2 } else { 3 }").parse_expr().is_ok());
    }
    
    // Test group 15: Loops (complexity: 3 each)
    #[test]
    fn test_while_loop() {
        assert!(Parser::new("while true { 1 }").parse_expr().is_ok());
    }
    
    #[test]
    fn test_for_loop() {
        assert!(Parser::new("for x in [1, 2, 3] { x }").parse_expr().is_ok());
    }
    
    // Test group 16: Functions (complexity: 4)
    #[test]
    fn test_fun_no_params() {
        assert!(Parser::new("fun f() { 42 }").parse_expr().is_ok());
    }
    
    #[test]
    fn test_fun_one_param() {
        assert!(Parser::new("fun f(x) { x }").parse_expr().is_ok());
    }
    
    #[test]
    fn test_fun_multiple_params() {
        assert!(Parser::new("fun f(x, y, z) { x + y + z }").parse_expr().is_ok());
    }
    
    // Test group 17: Lambdas (complexity: 3)
    #[test]
    fn test_lambda_simple() {
        assert!(Parser::new("|x| x").parse_expr().is_ok());
    }
    
    #[test]
    fn test_lambda_multiple_params() {
        assert!(Parser::new("|x, y| x + y").parse_expr().is_ok());
    }
    
    #[test]
    fn test_lambda_no_params() {
        assert!(Parser::new("|| 42").parse_expr().is_ok());
    }
    
    // Test group 18: Match (complexity: 4)
    #[test]
    fn test_match_simple() {
        assert!(Parser::new("match x { 1 => true, _ => false }").parse_expr().is_ok());
    }
    
    #[test]
    fn test_match_multiple() {
        assert!(Parser::new("match x { 1 => a, 2 => b, _ => c }").parse_expr().is_ok());
    }
}