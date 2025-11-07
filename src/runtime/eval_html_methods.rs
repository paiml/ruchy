//! HTML method evaluation module
//!
//! This module handles evaluation of HTML parsing methods.
//! Provides method dispatching for `HtmlDocument` and `HtmlElement` types.
//! Following Toyota Way principles - complexity ≤10 per function.

use crate::runtime::{InterpreterError, Value};
#[cfg(not(target_arch = "wasm32"))]
use crate::stdlib::html::{HtmlDocument, HtmlElement};

/// Evaluate `HtmlDocument` methods
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Errors
/// Returns error if method not found or arguments invalid
#[cfg(not(target_arch = "wasm32"))]
pub fn eval_html_document_method(
    doc: &HtmlDocument,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "select" => eval_html_document_select(doc, arg_values),
        "query_selector" => eval_html_document_query_selector(doc, arg_values),
        "query_selector_all" => eval_html_document_query_selector_all(doc, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown HtmlDocument method: {method}"
        ))),
    }
}

/// Evaluate `HtmlElement` methods
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
///
/// # Errors
/// Returns error if method not found or arguments invalid
#[cfg(not(target_arch = "wasm32"))]
pub fn eval_html_element_method(
    element: &HtmlElement,
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "text" => eval_html_element_text(element, arg_values),
        "html" => eval_html_element_html(element, arg_values),
        "attr" => eval_html_element_attr(element, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown HtmlElement method: {method}"
        ))),
    }
}

// HtmlDocument method implementations

#[cfg(not(target_arch = "wasm32"))]
fn eval_html_document_select(
    doc: &HtmlDocument,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "HtmlDocument.select() requires exactly 1 argument (selector)".to_string(),
        ));
    }

    if let Value::String(selector) = &arg_values[0] {
        match doc.select(selector) {
            Ok(elements) => {
                let values: Vec<Value> = elements.into_iter().map(Value::HtmlElement).collect();
                Ok(Value::from_array(values))
            }
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "HTML selector error: {e}"
            ))),
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "HtmlDocument.select() expects selector as string".to_string(),
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn eval_html_document_query_selector(
    doc: &HtmlDocument,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "HtmlDocument.query_selector() requires exactly 1 argument (selector)".to_string(),
        ));
    }

    if let Value::String(selector) = &arg_values[0] {
        match doc.query_selector(selector) {
            Ok(Some(element)) => Ok(Value::HtmlElement(element)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "HTML selector error: {e}"
            ))),
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "HtmlDocument.query_selector() expects selector as string".to_string(),
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn eval_html_document_query_selector_all(
    doc: &HtmlDocument,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    // Same as select() - returns all matches
    eval_html_document_select(doc, arg_values)
}

// HtmlElement method implementations

#[cfg(not(target_arch = "wasm32"))]
fn eval_html_element_text(
    element: &HtmlElement,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "HtmlElement.text() takes no arguments".to_string(),
        ));
    }

    Ok(Value::from_string(element.text()))
}

#[cfg(not(target_arch = "wasm32"))]
fn eval_html_element_html(
    element: &HtmlElement,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "HtmlElement.html() takes no arguments".to_string(),
        ));
    }

    Ok(Value::from_string(element.html()))
}

#[cfg(not(target_arch = "wasm32"))]
fn eval_html_element_attr(
    element: &HtmlElement,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "HtmlElement.attr() requires exactly 1 argument (attribute_name)".to_string(),
        ));
    }

    if let Value::String(attr_name) = &arg_values[0] {
        match element.attr(attr_name) {
            Some(value) => Ok(Value::from_string(value)),
            None => Ok(Value::Nil),
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "HtmlElement.attr() expects attribute name as string".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div class='test'>Content</div>");
        let selector = Value::from_string("div".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok(), "select() should succeed");

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_text() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Hello World</div>");
        let elements = html.select("div").unwrap();
        assert!(!elements.is_empty());

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok(), "text() should succeed");

        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "Hello World"),
            _ => panic!("Expected string"),
        }
    }
}
