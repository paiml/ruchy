//! TDD tests for format_detailed_introspection refactoring
//! Original complexity: 255 (WAY over limit!)
//! Target complexity: <20 per function

use ruchy::runtime::repl::Repl;
use ruchy::runtime::Value;
use anyhow::Result;

#[test]
fn test_format_string_value() -> Result<()> {
    let repl = Repl::new()?;
    let value = Value::String("hello".to_string());
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: String"));
    assert!(output.contains("Value: \"hello\""));
    Ok(())
}

#[test]
fn test_format_integer_value() -> Result<()> {
    let repl = Repl::new()?;
    let value = Value::Integer(42);
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: Integer"));
    assert!(output.contains("Value: 42"));
    Ok(())
}

#[test]
fn test_format_function_value() -> Result<()> {
    let repl = Repl::new()?;
    let value = Value::Function {
        name: "add".to_string(),
        params: vec!["x".to_string(), "y".to_string()],
        body: Box::new(ruchy::frontend::ast::Expr::default()),
    };
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: Function"));
    assert!(output.contains("fn add(x, y)"));
    assert!(output.contains("Parameters: x, y"));
    Ok(())
}

#[test]
fn test_format_list_value() -> Result<()> {
    let repl = Repl::new()?;
    let value = Value::List(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]);
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: List"));
    assert!(output.contains("Length: 3"));
    assert!(output.contains("[1, 2, 3]"));
    Ok(())
}

#[test]
fn test_format_dict_value() -> Result<()> {
    use std::collections::HashMap;
    
    let repl = Repl::new()?;
    let mut dict = HashMap::new();
    dict.insert("key".to_string(), Value::String("value".to_string()));
    let value = Value::Dict(dict);
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: Dict"));
    assert!(output.contains("Size: 1"));
    assert!(output.contains("key"));
    assert!(output.contains("value"));
    Ok(())
}

#[test]
fn test_format_none_value() -> Result<()> {
    let repl = Repl::new()?;
    let value = Value::None;
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: None"));
    assert!(output.contains("Value: None"));
    Ok(())
}

#[test]
fn test_format_bool_value() -> Result<()> {
    let repl = Repl::new()?;
    let value = Value::Bool(true);
    
    let output = repl.format_detailed_introspection("test", &value);
    
    assert!(output.contains("Name: test"));
    assert!(output.contains("Type: Bool"));
    assert!(output.contains("Value: true"));
    Ok(())
}