// EXTREME TDD: Interpreter Coverage Boost
// Target: interpreter.rs - boost from 68.5% to 80%+
// Complexity: <10 per test
// Single responsibility, zero technical debt

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::value::Value;
use std::collections::HashMap;

#[cfg(test)]
mod interpreter_basic_tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_whitespace_only() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("   \n\t  \n  ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_comment_only() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("// just a comment");
        assert!(result.is_ok());
    }

    #[test]
    fn test_integer_literal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_float_literal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("3.14");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(3.14));
    }

    #[test]
    fn test_string_literal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"hello\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_boolean_true() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_boolean_false() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("false");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_unit_value() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("()");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_list_literal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[1, 2, 3]");
        assert!(result.is_ok());
        let list = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        assert_eq!(result.unwrap(), Value::List(list));
    }

    #[test]
    fn test_tuple_literal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(1, \"hello\", true)");
        assert!(result.is_ok());
        let tuple = vec![
            Value::Integer(1),
            Value::String("hello".to_string()),
            Value::Boolean(true),
        ];
        assert_eq!(result.unwrap(), Value::Tuple(tuple));
    }

    #[test]
    fn test_object_literal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("{a: 1, b: 2}");
        assert!(result.is_ok());
        if let Value::Object(map) = result.unwrap() {
            assert_eq!(map.get("a"), Some(&Value::Integer(1)));
            assert_eq!(map.get("b"), Some(&Value::Integer(2)));
        } else {
            panic!("Expected object");
        }
    }
}

#[cfg(test)]
mod interpreter_arithmetic_tests {
    use super::*;

    #[test]
    fn test_addition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("2 + 3");
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_subtraction() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("10 - 4");
        assert_eq!(result.unwrap(), Value::Integer(6));
    }

    #[test]
    fn test_multiplication() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("3 * 7");
        assert_eq!(result.unwrap(), Value::Integer(21));
    }

    #[test]
    fn test_division() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("15 / 3");
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_modulo() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("17 % 5");
        assert_eq!(result.unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_power() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("2 ** 8");
        assert_eq!(result.unwrap(), Value::Integer(256));
    }

    #[test]
    fn test_negative_number() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("-42");
        assert_eq!(result.unwrap(), Value::Integer(-42));
    }

    #[test]
    fn test_complex_arithmetic() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("2 + 3 * 4 - 5");
        assert_eq!(result.unwrap(), Value::Integer(9));
    }

    #[test]
    fn test_parenthesized() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(2 + 3) * 4");
        assert_eq!(result.unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_float_arithmetic() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("3.5 + 2.5");
        assert_eq!(result.unwrap(), Value::Float(6.0));
    }
}

#[cfg(test)]
mod interpreter_comparison_tests {
    use super::*;

    #[test]
    fn test_equal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("5 == 5");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_not_equal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("5 != 3");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_less_than() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("3 < 5");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_less_equal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("5 <= 5");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_greater_than() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("7 > 3");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_greater_equal() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("5 >= 5");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_string_comparison() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"apple\" < \"banana\"");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_chained_comparison() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("1 < 2 && 2 < 3");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }
}

#[cfg(test)]
mod interpreter_logical_tests {
    use super::*;

    #[test]
    fn test_logical_and() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("true && true");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_logical_or() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("true || false");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_logical_not() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("!false");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_short_circuit_and() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("false && (1/0)");
        assert_eq!(result.unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_short_circuit_or() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("true || (1/0)");
        assert_eq!(result.unwrap(), Value::Boolean(true));
    }
}

#[cfg(test)]
mod interpreter_variable_tests {
    use super::*;

    #[test]
    fn test_let_binding() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let x = 42").unwrap();
        let result = interpreter.eval("x");
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_let_mut_binding() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut x = 10").unwrap();
        interpreter.eval("x = 20").unwrap();
        let result = interpreter.eval("x");
        assert_eq!(result.unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_const_binding() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("const PI = 3.14159").unwrap();
        let result = interpreter.eval("PI");
        assert_eq!(result.unwrap(), Value::Float(3.14159));
    }

    #[test]
    fn test_shadow_variable() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let x = 10").unwrap();
        interpreter.eval("let x = 20").unwrap();
        let result = interpreter.eval("x");
        assert_eq!(result.unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_compound_assignment() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut x = 10").unwrap();
        interpreter.eval("x += 5").unwrap();
        let result = interpreter.eval("x");
        assert_eq!(result.unwrap(), Value::Integer(15));
    }
}

#[cfg(test)]
mod interpreter_function_tests {
    use super::*;

    #[test]
    fn test_function_definition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("fun add(a, b) { a + b }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("fun add(a, b) { a + b }").unwrap();
        let result = interpreter.eval("add(3, 4)");
        assert_eq!(result.unwrap(), Value::Integer(7));
    }

    #[test]
    fn test_recursive_function() {
        let mut interpreter = Interpreter::new();
        interpreter
            .eval("fun factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }")
            .unwrap();
        let result = interpreter.eval("factorial(5)");
        assert_eq!(result.unwrap(), Value::Integer(120));
    }

    #[test]
    fn test_closure() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let x = 10").unwrap();
        interpreter.eval("fun add_x(y) { x + y }").unwrap();
        let result = interpreter.eval("add_x(5)");
        assert_eq!(result.unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_lambda() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let add = |a, b| a + b").unwrap();
        let result = interpreter.eval("add(2, 3)");
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_higher_order_function() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("fun apply(f, x) { f(x) }").unwrap();
        interpreter.eval("fun double(x) { x * 2 }").unwrap();
        let result = interpreter.eval("apply(double, 5)");
        assert_eq!(result.unwrap(), Value::Integer(10));
    }
}

#[cfg(test)]
mod interpreter_control_flow_tests {
    use super::*;

    #[test]
    fn test_if_true() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("if true { 42 } else { 0 }");
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_if_false() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("if false { 42 } else { 0 }");
        assert_eq!(result.unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_if_without_else() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("if false { 42 }");
        assert_eq!(result.unwrap(), Value::Unit);
    }

    #[test]
    fn test_while_loop() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut x = 0").unwrap();
        interpreter.eval("while x < 5 { x += 1 }").unwrap();
        let result = interpreter.eval("x");
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_for_loop() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut sum = 0").unwrap();
        interpreter.eval("for i in 1..5 { sum += i }").unwrap();
        let result = interpreter.eval("sum");
        assert_eq!(result.unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_match_expression() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
        assert_eq!(result.unwrap(), Value::String("two".to_string()));
    }

    #[test]
    fn test_break_in_loop() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut x = 0").unwrap();
        interpreter
            .eval("while true { x += 1; if x == 3 { break } }")
            .unwrap();
        let result = interpreter.eval("x");
        assert_eq!(result.unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_continue_in_loop() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut sum = 0").unwrap();
        interpreter
            .eval("for i in 1..6 { if i == 3 { continue }; sum += i }")
            .unwrap();
        let result = interpreter.eval("sum");
        assert_eq!(result.unwrap(), Value::Integer(12)); // 1+2+4+5
    }

    #[test]
    fn test_nested_if() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("if true { if false { 1 } else { 2 } } else { 3 }");
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
}

#[cfg(test)]
mod interpreter_string_tests {
    use super::*;

    #[test]
    fn test_string_concatenation() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"hello\" + \" world\"");
        assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
    }

    #[test]
    fn test_string_interpolation() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let name = \"Ruchy\"").unwrap();
        let result = interpreter.eval("f\"Hello {name}\"");
        assert_eq!(result.unwrap(), Value::String("Hello Ruchy".to_string()));
    }

    #[test]
    fn test_string_length() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"hello\".len()");
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_string_uppercase() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"hello\".upper()");
        assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_split() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"a,b,c\".split(\",\")");
        assert!(result.is_ok());
        if let Value::List(items) = result.unwrap() {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_string_replace() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"hello world\".replace(\"world\", \"ruchy\")");
        assert_eq!(result.unwrap(), Value::String("hello ruchy".to_string()));
    }
}

#[cfg(test)]
mod interpreter_list_tests {
    use super::*;

    #[test]
    fn test_list_index() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[10, 20, 30][1]");
        assert_eq!(result.unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_list_append() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let mut list = [1, 2]").unwrap();
        interpreter.eval("list.push(3)").unwrap();
        let result = interpreter.eval("list");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(items.len(), 3);
            assert_eq!(items[2], Value::Integer(3));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_comprehension() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[x * 2 for x in 1..4]");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(
                items,
                vec![Value::Integer(2), Value::Integer(4), Value::Integer(6),]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_slice() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[1, 2, 3, 4, 5][1..3]");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(items, vec![Value::Integer(2), Value::Integer(3)]);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_map() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[1, 2, 3].map(|x| x * 2)");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(
                items,
                vec![Value::Integer(2), Value::Integer(4), Value::Integer(6),]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_filter() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[1, 2, 3, 4, 5].filter(|x| x > 2)");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(
                items,
                vec![Value::Integer(3), Value::Integer(4), Value::Integer(5),]
            );
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_list_reduce() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[1, 2, 3, 4].reduce(0, |acc, x| acc + x)");
        assert_eq!(result.unwrap(), Value::Integer(10));
    }
}

#[cfg(test)]
mod interpreter_error_handling_tests {
    use super::*;

    #[test]
    fn test_undefined_variable() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("undefined_var");
        assert!(result.is_err());
    }

    #[test]
    fn test_division_by_zero() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("1 / 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_type_error() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"hello\" + 42");
        assert!(result.is_err());
    }

    #[test]
    fn test_index_out_of_bounds() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("[1, 2, 3][10]");
        assert!(result.is_err());
    }

    #[test]
    fn test_function_arity_mismatch() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("fun add(a, b) { a + b }").unwrap();
        let result = interpreter.eval("add(1)");
        assert!(result.is_err());
    }

    #[test]
    fn test_try_catch() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("try { 1 / 0 } catch { 42 }");
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_option_unwrap() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("Some(42).unwrap()");
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_result_ok() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("Ok(42)");
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod interpreter_advanced_tests {
    use super::*;

    #[test]
    fn test_pipeline_operator() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("5 |> |x| x * 2 |> |x| x + 1");
        assert_eq!(result.unwrap(), Value::Integer(11));
    }

    #[test]
    fn test_async_function() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("async fun fetch_data() { 42 }").unwrap();
        let result = interpreter.eval("await fetch_data()");
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_generator() {
        let mut interpreter = Interpreter::new();
        interpreter
            .eval("fun* count() { yield 1; yield 2; yield 3 }")
            .unwrap();
        let result = interpreter.eval("count().collect()");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_destructuring() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let [a, b, c] = [1, 2, 3]").unwrap();
        let result = interpreter.eval("b");
        assert_eq!(result.unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_spread_operator() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let arr1 = [1, 2]").unwrap();
        let result = interpreter.eval("[...arr1, 3, 4]");
        if let Value::List(items) = result.unwrap() {
            assert_eq!(items.len(), 4);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_object_spread() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("let obj1 = {a: 1, b: 2}").unwrap();
        let result = interpreter.eval("{...obj1, c: 3}");
        if let Value::Object(map) = result.unwrap() {
            assert_eq!(map.len(), 3);
            assert_eq!(map.get("c"), Some(&Value::Integer(3)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_class_definition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("class Point { x: i32, y: i32 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_definition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("enum Color { Red, Green, Blue }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_guard() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("match 5 { x if x > 3 => \"big\", _ => \"small\" }");
        assert_eq!(result.unwrap(), Value::String("big".to_string()));
    }

    #[test]
    fn test_range_patterns() {
        let mut interpreter = Interpreter::new();
        let result = interpreter
            .eval("match 15 { 1..10 => \"small\", 10..20 => \"medium\", _ => \"large\" }");
        assert_eq!(result.unwrap(), Value::String("medium".to_string()));
    }
}
