// Property-based testing for language invariants that must NEVER be violated
// Uses mathematical properties to verify correctness

use ruchy::runtime::Repl;
use std::env;

#[test]
fn test_while_loop_iteration_invariant() {
    // Property: while i < N should execute exactly N iterations
    for n in 1..=10 {
        let code = format!(
            "var count = 0; var i = 0; while i < {n} {{ count = count + 1; i = i + 1 }}; count"
        );

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let result = repl.eval(&code).unwrap();

        assert_eq!(
            result.to_string(),
            n.to_string(),
            "while i < {n} should execute exactly {n} times"
        );
    }
}

#[test]
fn test_while_loop_never_returns_body_value() {
    // Property: while loops always return Unit, never body values
    let test_cases = vec![
        "var i = 0; while i < 1 { i = i + 1; 42 }", // Body returns 42
        "var i = 0; while i < 2 { i = i + 1; \"hello\" }", // Body returns string
        "var i = 0; while i < 1 { i = i + 1; [1, 2, 3] }", // Body returns array
    ];

    for code in test_cases {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let result = repl.eval(code).unwrap();

        assert_eq!(
            result.to_string(),
            "",
            "While loop should return Unit, not body value. Code: {code}"
        );
    }
}

#[test]
fn test_object_items_consistency() {
    // Property: obj.items().len() == obj.keys().len() == obj.values().len()
    let test_cases = vec![
        r#"{"a": 1}"#,
        r#"{"x": 1, "y": 2}"#,
        r#"{"name": "test", "value": 42, "active": true}"#,
    ];

    for obj_literal in test_cases {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        // Get lengths
        let items_len = repl
            .eval(&format!("let obj = {obj_literal}; obj.items().len()"))
            .unwrap()
            .to_string();
        let keys_len = repl
            .eval(&format!("let obj = {obj_literal}; obj.keys().len()"))
            .unwrap()
            .to_string();
        let values_len = repl
            .eval(&format!("let obj = {obj_literal}; obj.values().len()"))
            .unwrap()
            .to_string();

        assert_eq!(
            items_len, keys_len,
            "items.len() != keys.len() for {obj_literal}"
        );
        assert_eq!(
            keys_len, values_len,
            "keys.len() != values.len() for {obj_literal}"
        );
    }
}

#[test]
fn test_for_loop_tuple_destructuring_consistency() {
    // Property: for (a, b) in tuples should bind correctly
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Test that tuple destructuring works correctly
    repl.eval(r#"let obj = {"x": 1, "y": 2}"#).unwrap();
    repl.eval(r"var keys = []").unwrap();
    repl.eval(r"var values = []").unwrap();
    repl.eval(
        r"for key, value in obj.items() { keys = keys.push(key); values = values.push(value) }",
    )
    .unwrap();
    let result = repl.eval(r"[keys, values]").unwrap();

    let result_str = result;
    // Result should be [["x", "y"], [1, 2]] or [["y", "x"], [2, 1]] (order may vary)
    assert!(
        result_str.contains("\"x\"") && result_str.contains("\"y\""),
        "Keys should be extracted correctly"
    );
    assert!(
        result_str.contains('1') && result_str.contains('2'),
        "Values should be extracted correctly"
    );
}

#[test]
fn test_arithmetic_associativity() {
    // Property: (a + b) + c == a + (b + c)
    let test_cases = vec![(1, 2, 3), (10, 20, 30), (0, 5, -5)];

    for (a, b, c) in test_cases {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let left = repl
            .eval(&format!("({a} + {b}) + {c}"))
            .unwrap()
            .to_string();
        let right = repl
            .eval(&format!("{a} + ({b} + {c})"))
            .unwrap()
            .to_string();

        assert_eq!(
            left, right,
            "Addition should be associative: ({a} + {b}) + {c} != {a} + ({b} + {c})"
        );
    }
}

#[test]
fn test_string_concatenation_identity() {
    // Property: s + "" == "" + s == s
    let test_cases = vec!["hello", "world", "test123", ""];

    for s in test_cases {
        let mut repl = Repl::new(std::env::temp_dir()).unwrap();

        let original = repl.eval(&format!(r#""{s}""#)).unwrap().to_string();
        let left_identity = repl.eval(&format!(r#""{s}" + """#)).unwrap().to_string();
        let right_identity = repl.eval(&format!(r#""" + "{s}""#)).unwrap().to_string();

        assert_eq!(
            original, left_identity,
            "Left identity failed for string: {s}"
        );
        assert_eq!(
            original, right_identity,
            "Right identity failed for string: {s}"
        );
    }
}

#[test]
fn test_function_determinism() {
    // Property: Functions with same inputs always produce same outputs
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Define a function
    repl.eval("fn compute(x) { x * 2 + 1 }").unwrap();

    // Call it multiple times with same input
    let inputs = vec![0, 1, 5, -3, 100];
    for input in inputs {
        let result1 = repl.eval(&format!("compute({input})")).unwrap().to_string();
        let result2 = repl.eval(&format!("compute({input})")).unwrap().to_string();
        let result3 = repl.eval(&format!("compute({input})")).unwrap().to_string();

        assert_eq!(
            result1, result2,
            "Function not deterministic for input {input}"
        );
        assert_eq!(
            result2, result3,
            "Function not deterministic for input {input}"
        );
    }
}
