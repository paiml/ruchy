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

    // ============================================================================
    // EXTREME TDD Round 132: Additional comprehensive tests
    // Target: 18 → 40+ tests
    // ============================================================================

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_class_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div class='foo'>A</div><div class='bar'>B</div>");
        let selector = Value::from_string(".foo".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_id_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div id='unique'>Content</div>");
        let selector = Value::from_string("#unique".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_nested_elements() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span><p>Nested</p></span></div>");
        let selector = Value::from_string("p".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_query_selector_first_match() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>First</div><div>Second</div>");
        let selector = Value::from_string("div".to_string());

        let result = eval_html_document_query_selector(&html, &[selector]);
        assert!(result.is_ok());

        // Should return first element only
        match result.unwrap() {
            Value::HtmlElement(el) => {
                assert!(el.text().contains("First"));
            }
            _ => panic!("Expected HtmlElement"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_text_multiple_children() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span>Hello</span> <span>World</span></div>");
        let elements = html.select("div").unwrap();
        assert!(!elements.is_empty());

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => {
                assert!(s.contains("Hello"));
                assert!(s.contains("World"));
            }
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_text_empty_element() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div></div>");
        let elements = html.select("div").unwrap();
        assert!(!elements.is_empty());

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => assert!(s.is_empty() || s.trim().is_empty()),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_html_self_closing() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><br/><hr/></div>");
        let elements = html.select("div").unwrap();
        assert!(!elements.is_empty());

        let result = eval_html_element_html(&elements[0], &[]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr_multiple_attrs() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<a href='link' class='btn' id='mylink'>Text</a>");
        let elements = html.select("a").unwrap();
        assert!(!elements.is_empty());

        // Check href
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("href".to_string())]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "link"),
            _ => panic!("Expected string"),
        }

        // Check class
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("class".to_string())]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "btn"),
            _ => panic!("Expected string"),
        }

        // Check id
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("id".to_string())]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "mylink"),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_method_select() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let selector = Value::from_string("div".to_string());

        let result = eval_html_document_method(&html, "select", &[selector]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_method_query_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let selector = Value::from_string("div".to_string());

        let result = eval_html_document_method(&html, "query_selector", &[selector]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_method_query_selector_all() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>A</div><div>B</div>");
        let selector = Value::from_string("div".to_string());

        let result = eval_html_document_method(&html, "query_selector_all", &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 2),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_method_text() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Hello</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_method(&elements[0], "text", &[]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_method_html() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><b>Bold</b></div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_method(&elements[0], "html", &[]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => assert!(s.contains("Bold")),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_method_attr() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<a href='test'>Link</a>");
        let elements = html.select("a").unwrap();

        let result = eval_html_element_method(&elements[0], "attr", &[Value::from_string("href".to_string())]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_text_whitespace_handling() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>   Spaces   </div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => assert!(s.contains("Spaces")),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_attribute_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div data-id='123'>Content</div><div>Other</div>");
        let selector = Value::from_string("[data-id]".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_query_selector_wrong_type() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let non_string = Value::Float(3.14);

        let result = eval_html_document_query_selector(&html, &[non_string]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr_empty_value() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<input disabled value=''>");
        let elements = html.select("input").unwrap();
        assert!(!elements.is_empty());

        let result = eval_html_element_attr(&elements[0], &[Value::from_string("value".to_string())]);
        assert!(result.is_ok());

        match result.unwrap() {
            Value::String(s) => assert!(s.is_empty()),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_document_select_with_float_arg_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let float_val = Value::Float(1.5);

        let result = eval_html_document_select(&html, &[float_val]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects selector as string"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr_with_bool_arg_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_attr(&elements[0], &[Value::Bool(true)]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_html_element_attr_too_many_args() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_attr(
            &elements[0],
            &[Value::from_string("a".to_string()), Value::from_string("b".to_string())]
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires exactly 1 argument"));
    }
}

// ============================================================================
// EXTREME TDD Round 134: Additional comprehensive tests
// Target: 39 → 60+ tests
// ============================================================================
#[cfg(test)]
mod round_134_tests {
    use super::*;

    // --- Document select edge cases ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_select_descendant_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><ul><li>Item</li></ul></div>");
        let selector = Value::from_string("div li".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_select_child_combinator() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span>Direct</span></div>");
        let selector = Value::from_string("div > span".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_select_multiple_matches_same_tag() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<p>1</p><p>2</p><p>3</p><p>4</p><p>5</p>");
        let selector = Value::from_string("p".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 5),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_select_multiple_classes() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div class='a b'>Content</div>");
        let selector = Value::from_string(".a.b".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());
    }

    // --- Query selector edge cases ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_query_selector_deeply_nested() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><div><div><div><span>Deep</span></div></div></div></div>");
        let selector = Value::from_string("span".to_string());

        let result = eval_html_document_query_selector(&html, &[selector]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::HtmlElement(el) => assert!(el.text().contains("Deep")),
            _ => panic!("Expected HtmlElement"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_query_selector_with_array_arg_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let arr = Value::from_array(vec![Value::Integer(1)]);

        let result = eval_html_document_query_selector(&html, &[arr]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_query_selector_too_many_args() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let sel1 = Value::from_string("div".to_string());
        let sel2 = Value::from_string("span".to_string());

        let result = eval_html_document_query_selector(&html, &[sel1, sel2]);
        assert!(result.is_err());
    }

    // --- Element text edge cases ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_text_with_nested_tags() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Start <b>bold</b> middle <i>italic</i> end</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => {
                assert!(s.contains("Start"));
                assert!(s.contains("bold"));
                assert!(s.contains("italic"));
                assert!(s.contains("end"));
            }
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_text_special_chars() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>&lt;script&gt;</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());
    }

    // --- Element html edge cases ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_html_preserves_structure() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><p>Para</p><ul><li>Item</li></ul></div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_html(&elements[0], &[]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => {
                assert!(s.contains("<p>") || s.contains("p>"));
                assert!(s.contains("<ul>") || s.contains("ul>"));
            }
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_html_empty() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<span></span>");
        let elements = html.select("span").unwrap();

        let result = eval_html_element_html(&elements[0], &[]);
        assert!(result.is_ok());
    }

    // --- Element attr edge cases ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_attr_data_attribute() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div data-value='42' data-name='test'>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_attr(&elements[0], &[Value::from_string("data-value".to_string())]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "42"),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_attr_boolean_attribute() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<input type='checkbox' checked>");
        let elements = html.select("input").unwrap();

        // Type attribute should be accessible
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("type".to_string())]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::String(s) => assert_eq!(s.as_ref(), "checkbox"),
            _ => panic!("Expected string"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_attr_with_nil_arg_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_attr(&elements[0], &[Value::Nil]);
        assert!(result.is_err());
    }

    // --- Document method dispatch ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_method_dispatch_select_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");

        // Wrong argument type
        let result = eval_html_document_method(&html, "select", &[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_method_dispatch_query_selector_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");

        // No arguments
        let result = eval_html_document_method(&html, "query_selector", &[]);
        assert!(result.is_err());
    }

    // --- Element method dispatch ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_method_dispatch_text_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        // Should error with arguments
        let result = eval_html_element_method(&elements[0], "text", &[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_method_dispatch_html_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        // Should error with arguments
        let result = eval_html_element_method(&elements[0], "html", &[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_method_dispatch_attr_error() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();

        // Should error with no arguments
        let result = eval_html_element_method(&elements[0], "attr", &[]);
        assert!(result.is_err());
    }

    // --- Additional coverage tests ---
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_select_sibling_elements() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span>A</span><span>B</span><span>C</span></div>");
        let selector = Value::from_string("span".to_string());

        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_select_with_universal_selector() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span>Content</span></div>");
        let selector = Value::from_string("*".to_string());

        // Universal selector should find all elements
        let result = eval_html_document_select(&html, &[selector]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_query_selector_all_single_match() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div id='only'>Unique</div>");
        let selector = Value::from_string("#only".to_string());

        let result = eval_html_document_query_selector_all(&html, &[selector]);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Array(arr) => assert_eq!(arr.len(), 1),
            _ => panic!("Expected array"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_text_with_newlines() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<pre>Line1\nLine2\nLine3</pre>");
        let elements = html.select("pre").unwrap();

        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_html_with_comments() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><!-- comment -->Content</div>");
        let elements = html.select("div").unwrap();

        let result = eval_html_element_html(&elements[0], &[]);
        assert!(result.is_ok());
    }

    // === EXTREME TDD Round 138 tests ===

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_select_empty_result() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let result = eval_html_document_select(&html, &[Value::from_string("span".to_string())]);
        assert!(result.is_ok());
        if let Ok(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 0);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_select_multiple_matches() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<ul><li>A</li><li>B</li><li>C</li></ul>");
        let result = eval_html_document_select(&html, &[Value::from_string("li".to_string())]);
        assert!(result.is_ok());
        if let Ok(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 3);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_query_selector_first_match() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<p>First</p><p>Second</p>");
        let result = eval_html_document_query_selector(&html, &[Value::from_string("p".to_string())]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_query_selector_no_match() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let result = eval_html_document_query_selector(&html, &[Value::from_string("span".to_string())]);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, Value::Nil);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_query_selector_all_nested() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><span>A</span><div><span>B</span></div></div>");
        let result = eval_html_document_query_selector_all(&html, &[Value::from_string("span".to_string())]);
        assert!(result.is_ok());
        if let Ok(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 2);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_text_whitespace_only() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<span>   </span>");
        let elements = html.select("span").unwrap();
        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_html_self_closing() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div><br/></div>");
        let elements = html.select("div").unwrap();
        let result = eval_html_element_html(&elements[0], &[]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_attr_data_id_attribute() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div data-id=\"123\">Content</div>");
        let elements = html.select("div").unwrap();
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("data-id".to_string())]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_attr_missing_attribute() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div>Content</div>");
        let elements = html.select("div").unwrap();
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("nonexistent".to_string())]);
        assert!(result.is_ok());
        if let Ok(val) = result {
            assert_eq!(val, Value::Nil);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_method_wrong_arg_count() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div></div>");
        let result = eval_html_document_select(&html, &[]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_document_method_wrong_arg_type() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<div></div>");
        let result = eval_html_document_select(&html, &[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_text_with_entities() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<span>&amp; &lt; &gt;</span>");
        let elements = html.select("span").unwrap();
        let result = eval_html_element_text(&elements[0], &[]);
        assert!(result.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_element_attr_empty_value() {
        use crate::stdlib::html::HtmlDocument;

        let html = HtmlDocument::parse("<input disabled=\"\"/>");
        let elements = html.select("input").unwrap();
        let result = eval_html_element_attr(&elements[0], &[Value::from_string("disabled".to_string())]);
        assert!(result.is_ok());
    }
}
