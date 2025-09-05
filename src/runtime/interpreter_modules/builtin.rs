//! Built-in functions for the interpreter
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use super::error::{InterpreterError, InterpreterResult};
use std::rc::Rc;

/// Built-in function type
pub type BuiltinFunction = fn(&[Value]) -> InterpreterResult<Value>;

/// Get all built-in functions
pub fn get_builtins() -> Vec<(&'static str, BuiltinFunction)> {
    vec![
        ("print", builtin_print),
        ("println", builtin_println),
        ("len", builtin_len),
        ("type", builtin_type),
        ("str", builtin_str),
        ("int", builtin_int),
        ("float", builtin_float),
        ("bool", builtin_bool),
        ("abs", builtin_abs),
        ("min", builtin_min),
        ("max", builtin_max),
        ("sum", builtin_sum),
        ("range", builtin_range),
        ("enumerate", builtin_enumerate),
        ("zip", builtin_zip),
    ]
}

// Built-in function implementations (complexity: ≤10)

fn builtin_print(args: &[Value]) -> InterpreterResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    Ok(Value::Nil)
}

fn builtin_println(args: &[Value]) -> InterpreterResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

fn builtin_len(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::Tuple(tup) => Ok(Value::Integer(tup.len() as i64)),
        _ => Err(InterpreterError::type_mismatch(
            "string, array, or tuple",
            args[0].type_name(),
        )),
    }
}

fn builtin_type(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    Ok(Value::from_string(args[0].type_name().to_string()))
}

fn builtin_str(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    Ok(Value::from_string(format!("{}", args[0])))
}

fn builtin_int(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(*i)),
        Value::Float(f) => Ok(Value::Integer(*f as i64)),
        Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
        Value::String(s) => {
            s.parse::<i64>()
                .map(Value::Integer)
                .map_err(|_| InterpreterError::runtime(format!("Cannot convert '{}' to int", s)))
        }
        _ => Err(InterpreterError::type_mismatch(
            "number, bool, or string",
            args[0].type_name(),
        )),
    }
}

fn builtin_float(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Integer(i) => Ok(Value::Float(*i as f64)),
        Value::String(s) => {
            s.parse::<f64>()
                .map(Value::Float)
                .map_err(|_| InterpreterError::runtime(format!("Cannot convert '{}' to float", s)))
        }
        _ => Err(InterpreterError::type_mismatch(
            "number or string",
            args[0].type_name(),
        )),
    }
}

fn builtin_bool(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    Ok(Value::Bool(args[0].is_truthy()))
}

fn builtin_abs(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Integer(i) => Ok(Value::Integer(i.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(InterpreterError::type_mismatch(
            "number",
            args[0].type_name(),
        )),
    }
}

fn builtin_min(args: &[Value]) -> InterpreterResult<Value> {
    if args.is_empty() {
        return Err(InterpreterError::runtime("min() requires at least one argument"));
    }

    let mut min_val = &args[0];
    for arg in &args[1..] {
        if let Some(ord) = arg.compare(min_val) {
            if ord == std::cmp::Ordering::Less {
                min_val = arg;
            }
        } else {
            return Err(InterpreterError::runtime("Cannot compare values in min()"));
        }
    }

    Ok(min_val.clone())
}

fn builtin_max(args: &[Value]) -> InterpreterResult<Value> {
    if args.is_empty() {
        return Err(InterpreterError::runtime("max() requires at least one argument"));
    }

    let mut max_val = &args[0];
    for arg in &args[1..] {
        if let Some(ord) = arg.compare(max_val) {
            if ord == std::cmp::Ordering::Greater {
                max_val = arg;
            }
        } else {
            return Err(InterpreterError::runtime("Cannot compare values in max()"));
        }
    }

    Ok(max_val.clone())
}

fn builtin_sum(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut sum = Value::Integer(0);
            for val in arr.iter() {
                sum = sum.add(val).map_err(InterpreterError::runtime)?;
            }
            Ok(sum)
        }
        _ => Err(InterpreterError::type_mismatch("array", args[0].type_name())),
    }
}

fn builtin_range(args: &[Value]) -> InterpreterResult<Value> {
    let (start, end, step) = parse_range_args(args)?;
    validate_range_step(step)?;
    let values = generate_range_values(start, end, step);
    Ok(Value::from_array(values))
}

/// Parse range arguments (complexity: 4)
fn parse_range_args(args: &[Value]) -> InterpreterResult<(i64, i64, i64)> {
    match args.len() {
        1 => parse_single_arg_range(&args[0]),
        2 => parse_two_arg_range(&args[0], &args[1]),
        3 => parse_three_arg_range(&args[0], &args[1], &args[2]),
        _ => Err(InterpreterError::runtime("range() takes 1-3 arguments")),
    }
}

/// Parse single argument range: range(n) -> 0..n (complexity: 2)
fn parse_single_arg_range(arg: &Value) -> InterpreterResult<(i64, i64, i64)> {
    let end = arg.as_i64()
        .ok_or_else(|| InterpreterError::type_mismatch("integer", arg.type_name()))?;
    Ok((0, end, 1))
}

/// Parse two argument range: range(start, end) (complexity: 3)
fn parse_two_arg_range(arg1: &Value, arg2: &Value) -> InterpreterResult<(i64, i64, i64)> {
    let start = arg1.as_i64()
        .ok_or_else(|| InterpreterError::type_mismatch("integer", arg1.type_name()))?;
    let end = arg2.as_i64()
        .ok_or_else(|| InterpreterError::type_mismatch("integer", arg2.type_name()))?;
    Ok((start, end, 1))
}

/// Parse three argument range: range(start, end, step) (complexity: 4)
fn parse_three_arg_range(arg1: &Value, arg2: &Value, arg3: &Value) -> InterpreterResult<(i64, i64, i64)> {
    let start = arg1.as_i64()
        .ok_or_else(|| InterpreterError::type_mismatch("integer", arg1.type_name()))?;
    let end = arg2.as_i64()
        .ok_or_else(|| InterpreterError::type_mismatch("integer", arg2.type_name()))?;
    let step = arg3.as_i64()
        .ok_or_else(|| InterpreterError::type_mismatch("integer", arg3.type_name()))?;
    Ok((start, end, step))
}

/// Validate range step is not zero (complexity: 2)
fn validate_range_step(step: i64) -> InterpreterResult<()> {
    if step == 0 {
        Err(InterpreterError::runtime("range() step cannot be zero"))
    } else {
        Ok(())
    }
}

/// Generate range values (complexity: 3)
fn generate_range_values(start: i64, end: i64, step: i64) -> Vec<Value> {
    let mut values = Vec::new();
    let mut current = start;
    
    if step > 0 {
        while current < end {
            values.push(Value::Integer(current));
            current += step;
        }
    } else {
        while current > end {
            values.push(Value::Integer(current));
            current += step;
        }
    }
    
    values
}

fn builtin_enumerate(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() != 1 {
        return Err(InterpreterError::argument_count_mismatch(1, args.len()));
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for (i, val) in arr.iter().enumerate() {
                let pair = vec![Value::Integer(i as i64), val.clone()];
                result.push(Value::from_tuple(pair));
            }
            Ok(Value::from_array(result))
        }
        _ => Err(InterpreterError::type_mismatch("array", args[0].type_name())),
    }
}

fn builtin_zip(args: &[Value]) -> InterpreterResult<Value> {
    if args.len() < 2 {
        return Err(InterpreterError::runtime("zip() requires at least 2 arguments"));
    }

    let arrays: Result<Vec<_>, _> = args.iter().map(|arg| {
        match arg {
            Value::Array(arr) => Ok(arr.as_ref()),
            _ => Err(InterpreterError::type_mismatch("array", arg.type_name())),
        }
    }).collect();

    let arrays = arrays?;
    let min_len = arrays.iter().map(|a| a.len()).min().unwrap_or(0);
    
    let mut result = Vec::new();
    for i in 0..min_len {
        let mut tuple = Vec::new();
        for arr in &arrays {
            tuple.push(arr[i].clone());
        }
        result.push(Value::from_tuple(tuple));
    }

    Ok(Value::from_array(result))
}