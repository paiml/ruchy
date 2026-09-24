//! OPTMETHODS-1: `Option`/`Result` methods in the interpreter.
//!
//! The interpreter keeps nil-for-absent library results (`pop`, `first`,
//! `get` return the element or nil), so these methods accept both forms:
//! `Some(v)`/`Ok(v)` and a plain non-nil value are present; `None`, `Err(e)`
//! and nil are absent. `v.pop().unwrap()` therefore yields the element, as
//! the transpiled Rust does.
//!
//! A plain value only reaches these methods after its own type has reported
//! the method as unknown (see [`is_unknown_method_error`]), so an existing
//! string/array/integer/object method is never shadowed.

use crate::runtime::{InterpreterError, Value};

/// Which constructor a present value is wrapped in when a method rebuilds it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Option,
    Result,
    Plain,
}

/// A receiver seen as an `Option`/`Result`.
enum Shape {
    /// `Some(v)`, `Ok(v)` or a plain non-nil value.
    Present(Family, Value),
    /// `None` or nil (payload `Nil`), or `Err(e)` (payload `e`).
    Absent(Family, Value),
}

/// Result of an `Option` method dispatch; `None` means "not an `Option` method".
pub type OptionMethodResult = Option<Result<Value, InterpreterError>>;

/// True if `receiver` is `Some`/`None`/`Ok`/`Err` or nil: these dispatch here
/// before any other method table.
///
/// # Complexity
/// Cyclomatic complexity: 2
pub fn is_option_receiver(receiver: &Value) -> bool {
    matches!(receiver, Value::Nil) || variant_payload(receiver).is_some()
}

/// True if `err` is a type's "no such method" error for `method`.
///
/// # Complexity
/// Cyclomatic complexity: 4
pub fn is_unknown_method_error(err: &InterpreterError, method: &str) -> bool {
    let InterpreterError::RuntimeError(msg) = err else {
        return false;
    };
    let names_method =
        msg.ends_with(&format!(": {method}")) || msg.contains(&format!("'{method}'"));
    names_method && (msg.contains("Unknown") || msg.contains("not found"))
}

/// The variant name and payload of a `Some`/`None`/`Ok`/`Err` value.
///
/// # Complexity
/// Cyclomatic complexity: 3
fn variant_payload(receiver: &Value) -> Option<(&str, Option<&Value>)> {
    let Value::EnumVariant {
        variant_name, data, ..
    } = receiver
    else {
        return None;
    };
    let payload = data.as_ref().and_then(|d| d.first());
    match variant_name.as_str() {
        "Some" | "None" | "Ok" | "Err" => Some((variant_name.as_str(), payload)),
        _ => None,
    }
}

/// # Complexity
/// Cyclomatic complexity: 7
fn classify(receiver: &Value) -> Shape {
    let payload = |p: Option<&Value>| p.cloned().unwrap_or(Value::Nil);
    match variant_payload(receiver) {
        Some(("Some", p)) => Shape::Present(Family::Option, payload(p)),
        Some(("Ok", p)) => Shape::Present(Family::Result, payload(p)),
        Some(("Err", p)) => Shape::Absent(Family::Result, payload(p)),
        Some(_) => Shape::Absent(Family::Option, Value::Nil),
        None if matches!(receiver, Value::Nil) => Shape::Absent(Family::Plain, Value::Nil),
        None => Shape::Present(Family::Plain, receiver.clone()),
    }
}

fn variant(enum_name: &str, variant_name: &str, data: Option<Value>) -> Value {
    Value::EnumVariant {
        enum_name: enum_name.to_string(),
        variant_name: variant_name.to_string(),
        data: data.map(|v| vec![v]),
    }
}

fn some(v: Value) -> Value {
    variant("Option", "Some", Some(v))
}

fn none() -> Value {
    variant("Option", "None", None)
}

/// Rebuild a present value in its family's constructor.
fn wrap(family: Family, v: Value) -> Value {
    match family {
        Family::Option => some(v),
        Family::Result => variant("Result", "Ok", Some(v)),
        Family::Plain => v,
    }
}

/// The absent value of a family that has no payload to keep.
fn empty(family: Family) -> Value {
    match family {
        Family::Plain => Value::Nil,
        _ => none(),
    }
}

/// How an absent receiver is named in error messages.
fn describe_absent(family: Family, payload: &Value) -> String {
    match family {
        Family::Option => "None".to_string(),
        Family::Result => format!("Err({payload})"),
        Family::Plain => "nil".to_string(),
    }
}

fn arg(method: &str, args: &[Value], count: usize) -> Result<(), InterpreterError> {
    if args.len() == count {
        return Ok(());
    }
    Err(InterpreterError::RuntimeError(format!(
        "{method}() takes {count} argument(s), got {}",
        args.len()
    )))
}

/// Evaluate an `Option`/`Result` method on `receiver`, or `None` if `method`
/// is not one of them. `call` invokes a closure argument.
///
/// # Complexity
/// Cyclomatic complexity: 3
pub fn eval_option_method<F>(
    receiver: &Value,
    method: &str,
    args: &[Value],
    call: &mut F,
) -> OptionMethodResult
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    let shape = classify(receiver);
    if let Some(result) = eval_query_method(&shape, method, args) {
        return Some(result);
    }
    eval_combinator_method(receiver, shape, method, args, call)
}

/// Predicates and extractors that never call a closure.
///
/// # Complexity
/// Cyclomatic complexity: 9
fn eval_query_method(shape: &Shape, method: &str, args: &[Value]) -> OptionMethodResult {
    let present = matches!(shape, Shape::Present(..));
    let result = match method {
        "is_some" | "is_ok" => arg(method, args, 0).map(|()| Value::Bool(present)),
        "is_none" | "is_err" => arg(method, args, 0).map(|()| Value::Bool(!present)),
        "unwrap" => arg(method, args, 0).and_then(|()| unwrap(shape, method, None)),
        "expect" => arg(method, args, 1).and_then(|()| unwrap(shape, method, Some(&args[0]))),
        "unwrap_or_default" => arg(method, args, 0).and_then(|()| unwrap_or_default(shape)),
        "unwrap_or" => arg(method, args, 1).map(|()| present_or(shape, &args[0])),
        "ok" => arg(method, args, 0).map(|()| ok(shape)),
        "ok_or" => arg(method, args, 1).map(|()| ok_or(shape, &args[0])),
        _ => return None,
    };
    Some(result)
}

/// `unwrap`/`expect`: the present value, or an error naming the absent one.
fn unwrap(shape: &Shape, method: &str, msg: Option<&Value>) -> Result<Value, InterpreterError> {
    match (shape, msg) {
        (Shape::Present(_, v), _) => Ok(v.clone()),
        (Shape::Absent(family, p), None) => Err(InterpreterError::RuntimeError(format!(
            "called `{method}()` on {}",
            describe_absent(*family, p)
        ))),
        (Shape::Absent(family, p), Some(msg)) => Err(InterpreterError::RuntimeError(format!(
            "{}: {}",
            display_text(msg),
            describe_absent(*family, p)
        ))),
    }
}

fn display_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.to_string(),
        other => other.to_string(),
    }
}

/// The default of an absent value's type is not known at runtime, so an
/// absent receiver is an error rather than a guessed default.
fn unwrap_or_default(shape: &Shape) -> Result<Value, InterpreterError> {
    match shape {
        Shape::Present(_, v) => Ok(v.clone()),
        Shape::Absent(family, p) => Err(InterpreterError::RuntimeError(format!(
            "called `unwrap_or_default()` on {}: the default value's type is unknown at runtime",
            describe_absent(*family, p)
        ))),
    }
}

fn present_or(shape: &Shape, fallback: &Value) -> Value {
    match shape {
        Shape::Present(_, v) => v.clone(),
        Shape::Absent(..) => fallback.clone(),
    }
}

fn ok(shape: &Shape) -> Value {
    match shape {
        Shape::Present(_, v) => some(v.clone()),
        Shape::Absent(..) => none(),
    }
}

fn ok_or(shape: &Shape, err: &Value) -> Value {
    match shape {
        Shape::Present(_, v) => variant("Result", "Ok", Some(v.clone())),
        Shape::Absent(..) => variant("Result", "Err", Some(err.clone())),
    }
}

/// Methods that take a closure or return the receiver itself.
///
/// # Complexity
/// Cyclomatic complexity: 8
fn eval_combinator_method<F>(
    receiver: &Value,
    shape: Shape,
    method: &str,
    args: &[Value],
    call: &mut F,
) -> OptionMethodResult
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    let result = match method {
        "or" => arg(method, args, 1).map(|()| or(receiver, &shape, &args[0])),
        "unwrap_or_else" => {
            arg(method, args, 1).and_then(|()| unwrap_or_else(shape, &args[0], call))
        }
        "map" => arg(method, args, 1).and_then(|()| map(receiver, shape, &args[0], call)),
        "and_then" => arg(method, args, 1).and_then(|()| and_then(receiver, shape, &args[0], call)),
        "filter" => arg(method, args, 1).and_then(|()| filter(receiver, shape, &args[0], call)),
        _ => return None,
    };
    Some(result)
}

fn or(receiver: &Value, shape: &Shape, fallback: &Value) -> Value {
    match shape {
        Shape::Present(..) => receiver.clone(),
        Shape::Absent(..) => fallback.clone(),
    }
}

/// `Err(e).unwrap_or_else(f)` passes `e` to `f`; `None`/nil pass nothing.
fn unwrap_or_else<F>(shape: Shape, f: &Value, call: &mut F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    match shape {
        Shape::Present(_, v) => Ok(v),
        Shape::Absent(Family::Result, e) => call(f, &[e]),
        Shape::Absent(..) => call(f, &[]),
    }
}

fn map<F>(
    receiver: &Value,
    shape: Shape,
    f: &Value,
    call: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    match shape {
        Shape::Present(family, v) => Ok(wrap(family, call(f, &[v])?)),
        Shape::Absent(..) => Ok(receiver.clone()),
    }
}

fn and_then<F>(
    receiver: &Value,
    shape: Shape,
    f: &Value,
    call: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    match shape {
        Shape::Present(_, v) => call(f, &[v]),
        Shape::Absent(..) => Ok(receiver.clone()),
    }
}

fn filter<F>(
    receiver: &Value,
    shape: Shape,
    pred: &Value,
    call: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    match shape {
        Shape::Present(family, v) => {
            let keep = call(pred, &[v])?.is_truthy();
            Ok(if keep {
                receiver.clone()
            } else {
                empty(family)
            })
        }
        Shape::Absent(..) => Ok(receiver.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_call(_: &Value, _: &[Value]) -> Result<Value, InterpreterError> {
        Err(InterpreterError::RuntimeError("no closure".to_string()))
    }

    fn eval(receiver: &Value, method: &str, args: &[Value]) -> Result<Value, InterpreterError> {
        eval_option_method(receiver, method, args, &mut no_call).expect("an option method")
    }

    #[test]
    fn test_optmethods_1_unit_present_and_absent_forms() {
        let some3 = some(Value::Integer(3));
        assert_eq!(eval(&some3, "unwrap", &[]).unwrap(), Value::Integer(3));
        assert_eq!(
            eval(&Value::Integer(4), "unwrap", &[]).unwrap(),
            Value::Integer(4)
        );
        assert_eq!(eval(&none(), "is_none", &[]).unwrap(), Value::Bool(true));
        assert_eq!(
            eval(&Value::Nil, "is_some", &[]).unwrap(),
            Value::Bool(false)
        );
        let fallback = [Value::Integer(5)];
        assert_eq!(
            eval(&Value::Nil, "unwrap_or", &fallback).unwrap(),
            Value::Integer(5)
        );
    }

    #[test]
    fn test_optmethods_1_unit_absent_unwrap_errors_name_the_value() {
        let err = variant(
            "Result",
            "Err",
            Some(Value::from_string("boom".to_string())),
        );
        let msg = eval(&err, "unwrap", &[]).unwrap_err().to_string();
        assert!(msg.contains("boom"), "{msg}");
        let msg = eval(&Value::Nil, "unwrap", &[]).unwrap_err().to_string();
        assert!(msg.contains("nil"), "{msg}");
        assert!(eval(&none(), "unwrap_or_default", &[]).is_err());
    }

    #[test]
    fn test_optmethods_1_unit_non_option_method_and_receiver() {
        assert!(eval_option_method(&none(), "len", &[], &mut no_call).is_none());
        assert!(is_option_receiver(&Value::Nil));
        assert!(!is_option_receiver(&Value::Integer(1)));
        let unknown = InterpreterError::RuntimeError("Unknown integer method: unwrap".to_string());
        assert!(is_unknown_method_error(&unknown, "unwrap"));
        let other = InterpreterError::RuntimeError("Unknown integer method: map_x".to_string());
        assert!(!is_unknown_method_error(&other, "map"));
    }
}
