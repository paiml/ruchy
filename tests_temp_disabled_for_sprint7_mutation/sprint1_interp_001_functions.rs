// SPRINT 1 (INTERP-001-C): Function Call Tests
// Goal: 60 failing tests for function calls
// Complexity: All implementations must be ≤10
// Performance: O(n) or better
// EXTREME TDD: Write all tests FIRST, then implement

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

#[cfg(test)]
mod basic_function_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_function_no_params() {
        let code = "fun get_answer() { 42 }; get_answer()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_function_one_param() {
        let code = "fun double(x) { x * 2 }; double(21)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_function_two_params() {
        let code = "fun add(a, b) { a + b }; add(10, 32)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_function_many_params() {
        let code = "fun sum5(a, b, c, d, e) { a + b + c + d + e }; sum5(1, 2, 3, 4, 5)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_function_with_local_vars() {
        let code = "fun compute() { let x = 10; let y = 32; x + y }; compute()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_function_param_shadowing() {
        let code = "let x = 100; fun test(x) { x }; test(42)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_function_return_types() {
        // Return integer
        assert_eq!(
            eval_expr("fun get_int() { 42 }; get_int()").unwrap(),
            Value::Integer(42)
        );
        // Return float
        assert_eq!(
            eval_expr("fun get_float() { 3.14 }; get_float()").unwrap(),
            Value::Float(3.14)
        );
        // Return string
        assert_eq!(
            eval_expr("fun get_str() { \"hello\" }; get_str()").unwrap(),
            Value::String(Rc::from("hello"))
        );
        // Return boolean
        assert_eq!(
            eval_expr("fun get_bool() { true }; get_bool()").unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_function_side_effects() {
        let code = "
            let mut global = 0;
            fun increment() { global = global + 1 };
            increment();
            increment();
            increment();
            global
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(3));
    }
}

#[cfg(test)]
mod recursive_function_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_factorial() {
        let code = "
            fun factorial(n) {
                if n <= 1 { 1 } else { n * factorial(n - 1) }
            };
            factorial(5)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(120));
    }

    #[test]
    fn test_fibonacci() {
        let code = "
            fun fib(n) {
                if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
            };
            fib(10)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(55));
    }

    #[test]
    fn test_mutual_recursion() {
        let code = "
            fun is_even(n) {
                if n == 0 { true } else { is_odd(n - 1) }
            };
            fun is_odd(n) {
                if n == 0 { false } else { is_even(n - 1) }
            };
            is_even(10)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_tail_recursion() {
        let code = "
            fun sum_tail(n, acc) {
                if n == 0 { acc } else { sum_tail(n - 1, acc + n) }
            };
            sum_tail(100, 0)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(5050));
    }

    #[test]
    fn test_deep_recursion() {
        // Should handle reasonably deep recursion
        let code = "
            fun countdown(n) {
                if n == 0 { \"done\" } else { countdown(n - 1) }
            };
            countdown(100)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::String(Rc::from("done")));
    }
}

#[cfg(test)]
mod closure_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_closure_captures_variable() {
        let code = "
            let x = 10;
            fun add_x(y) { x + y };
            add_x(32)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_closure_captures_multiple() {
        let code = "
            let a = 10;
            let b = 20;
            fun use_both() { a + b };
            use_both()
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(30));
    }

    #[test]
    fn test_closure_nested() {
        let code = "
            fun outer(x) {
                fun inner(y) {
                    x + y
                };
                inner
            };
            let add_10 = outer(10);
            add_10(32)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_closure_mutation() {
        let code = "
            let mut counter = 0;
            fun increment() {
                counter = counter + 1;
                counter
            };
            increment();
            increment();
            increment()
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_closure_in_loop() {
        let code = "
            let mut fns = [];
            for i in 1..4 {
                fns.push(fun() { i })
            };
            fns[0]() + fns[1]() + fns[2]()
        ";
        // Each closure should capture its own value of i
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(6)); // 1 + 2 + 3
    }
}

#[cfg(test)]
mod lambda_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_lambda_basic() {
        let code = "let double = |x| x * 2; double(21)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_lambda_multiple_params() {
        let code = "let add = |a, b| a + b; add(10, 32)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_lambda_no_params() {
        let code = "let get_answer = || 42; get_answer()";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_lambda_with_block() {
        let code = "let compute = |x| { let y = x * 2; y + 2 }; compute(20)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_lambda_closure() {
        let code = "
            let x = 10;
            let add_x = |y| x + y;
            add_x(32)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_lambda_as_argument() {
        let code = "
            fun apply(f, x) { f(x) };
            apply(|x| x * 2, 21)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_lambda_returned() {
        let code = "
            fun make_adder(x) {
                |y| x + y
            };
            let add_10 = make_adder(10);
            add_10(32)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }
}

#[cfg(test)]
mod higher_order_function_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_map_function() {
        let code = "
            fun map(f, list) {
                let mut result = [];
                for item in list {
                    result.push(f(item))
                };
                result
            };
            map(|x| x * 2, [1, 2, 3, 4, 5])
        ";
        // Result should be [2, 4, 6, 8, 10]
        assert!(eval_expr(code).is_ok());
    }

    #[test]
    fn test_filter_function() {
        let code = "
            fun filter(predicate, list) {
                let mut result = [];
                for item in list {
                    if predicate(item) {
                        result.push(item)
                    }
                };
                result
            };
            filter(|x| x > 3, [1, 2, 3, 4, 5])
        ";
        // Result should be [4, 5]
        assert!(eval_expr(code).is_ok());
    }

    #[test]
    fn test_reduce_function() {
        let code = "
            fun reduce(f, list, initial) {
                let mut acc = initial;
                for item in list {
                    acc = f(acc, item)
                };
                acc
            };
            reduce(|acc, x| acc + x, [1, 2, 3, 4, 5], 0)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_compose_functions() {
        let code = "
            fun compose(f, g) {
                |x| f(g(x))
            };
            let double = |x| x * 2;
            let add_one = |x| x + 1;
            let double_then_add = compose(add_one, double);
            double_then_add(20)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(41));
    }

    #[test]
    fn test_partial_application() {
        let code = "
            fun add(a, b) { a + b };
            fun partial(f, a) {
                |b| f(a, b)
            };
            let add_10 = partial(add, 10);
            add_10(32)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_currying() {
        let code = "
            fun curry_add(a) {
                |b| a + b
            };
            curry_add(10)(32)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }
}

#[cfg(test)]
mod generic_function_tests {
    use super::*;

    fn eval_expr(code: &str) -> Result<Value, String> {
        let mut interpreter = Interpreter::new();
        let mut parser = Parser::new(code);
        let expr = parser.parse().map_err(|e| e.to_string())?;
        interpreter.eval_expr(&expr).map_err(|e| e.to_string())
    }

    #[test]
    fn test_identity_function() {
        let code = "fun identity<T>(x: T) -> T { x }; identity(42)";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_swap_function() {
        let code =
            "fun swap<T, U>(pair: (T, U)) -> (U, T) { (pair.1, pair.0) }; swap((1, \"hello\"))";
        // Should return ("hello", 1)
        assert!(eval_expr(code).is_ok());
    }

    #[test]
    fn test_generic_map() {
        let code = "
            fun map<T, U>(f: fun(T) -> U, list: [T]) -> [U] {
                let mut result = [];
                for item in list {
                    result.push(f(item))
                };
                result
            };
            map(|x| x.to_string(), [1, 2, 3])
        ";
        // Should return ["1", "2", "3"]
        assert!(eval_expr(code).is_ok());
    }

    #[test]
    fn test_generic_option() {
        let code = "
            fun unwrap_or<T>(opt: Option<T>, default: T) -> T {
                match opt {
                    Some(x) => x,
                    None => default
                }
            };
            unwrap_or(Some(42), 0)
        ";
        assert_eq!(eval_expr(code).unwrap(), Value::Integer(42));
    }
}
