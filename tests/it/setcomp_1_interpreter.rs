#![allow(missing_docs)]
// SETCOMP-1: the interpreter evaluates {x * 2 for x in xs}.
// Set representation: the runtime has no set Value, so a set comprehension
// yields a Value::Array deduplicated by Value equality, first-occurrence order.

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

fn eval(code: &str) -> Value {
    let ast = Parser::new(code).parse().expect("parse failed");
    Interpreter::new().eval_expr(&ast).expect("eval failed")
}

fn ints(v: &Value) -> Vec<i64> {
    match v {
        Value::Array(a) => a
            .iter()
            .map(|x| match x {
                Value::Integer(i) => *i,
                other => panic!("not an integer: {other:?}"),
            })
            .collect(),
        other => panic!("not an array: {other:?}"),
    }
}

#[test]
fn test_setcomp1_dedups_in_first_occurrence_order() {
    assert_eq!(
        ints(&eval("{x * 2 for x in [1, 2, 2, 3, 1]}")),
        vec![2, 4, 6]
    );
}

#[test]
fn test_setcomp1_len_matches_issue_repro() {
    let v = eval("let s = {x * 2 for x in [1, 2, 2]}\ns.len()");
    assert_eq!(v, Value::Integer(2));
}

#[test]
fn test_setcomp1_condition_filters() {
    assert_eq!(
        ints(&eval("{x for x in [1, 2, 3, 4, 4, 2] if x > 1}")),
        vec![2, 3, 4]
    );
}

#[test]
fn test_setcomp1_empty_source() {
    assert!(ints(&eval("{x for x in []}")).is_empty());
}

#[test]
fn test_setcomp1_many_to_one_collapses() {
    assert_eq!(ints(&eval("{x % 2 for x in [1, 2, 3, 4, 5]}")), vec![1, 0]);
}
