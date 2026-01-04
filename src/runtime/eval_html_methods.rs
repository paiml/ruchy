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

    // ============================================================================
    // COVERAGE TESTS - Error paths and edge cases
    // ============================================================================

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_wrong_arg_count() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");

        // No arguments
        let result = eval_html_document_select(&html, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires exactly 1 argument"));

        // Too many arguments
        let selector1 = Value::from_string("div".to_string());
        let selector2 = Value::from_string("span".to_string());
        let result = eval_html_document_select(&html, &[selector1, selector2]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_wrong_type() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let non_string = Value::Integer(42);

        let result = eval_html_document_select(&html, &[non_string]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects selector as string"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_query_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div id='main'>Content</div>");
        let selector = Value::from_string("#main".to_string());

        let result = eval_html_document_query_selector(&html, &[selector]);
        assert!(result.is_ok());

        // Should return single element, not array
        match result.unwrap() {
            Value::HtmlElement(_) => {} // Good
            _ => panic!("Expected HtmlElement"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_query_selector_not_found() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let selector = Value::from_string("#nonexistent".to_string());

        let result = eval_html_document_query_selector(&html, &[selector]);
        assert!(result.is_ok());

        // Should return Nil when not found
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_query_selector_wrong_args() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");

        // No arguments
        let result = eval_html_document_query_selector(&html, &[]);
        assert!(result.is_err());

        // Wrong type
        let non_string = Value::Bool(true);
        let result = eval_html_document_query_selector(&html, &[non_string]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_query_selector_all() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>One</div><div>Two</div>");
        let selector = Value::from_string("div".to_string());

        let result = eval_html_document_query_selector_all(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 2),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_method_unknown() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");

        let result = eval_html_document_method(&html, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown HtmlDocument method"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_html() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span>Content</span></div>");
        let elements = html.select("div").unwrap();
        assert!(!elements.is_empty());

        let result = eval_html_element_html(&elements[0], &[]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => assert!(s.contains("span")),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_html_with_args_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_html(&elements[0], &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("takes no arguments"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_text_with_args_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_text(&elements[0], &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("takes no arguments"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<a href='https://example.com'>Link</a>");
        let elements = html.select("a").unwrap();
        assert!(!elements.is_empty());

        let attr_name = Value::from_string("href".to_string());
        let result = eval_html_element_attr(&elements[0], &[attr_name]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "https://example.com"),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr_not_found() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let attr_name = Value::from_string("nonexistent".to_string());
        let result = eval_html_element_attr(&elements[0], &[attr_name]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr_wrong_args() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        // No arguments
        let result = eval_html_element_attr(&elements[0], &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires exactly 1 argument"));

        // Wrong type
        let result = eval_html_element_attr(&elements[0], &[Value::Integer(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects attribute name as string"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_method_unknown() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_method(&elements[0], "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown HtmlElement method"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_multiple_elements() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<ul><li>One</li><li>Two</li><li>Three</li></ul>");
        let selector = Value::from_string("li".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array with 3 elements"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_empty_result() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let selector = Value::from_string("span".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected empty array"),
        }
    }
}
