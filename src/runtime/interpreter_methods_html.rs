//! HTML document and element method dispatch
//!
//! Extracted from interpreter_methods.rs for coverage attribution.

#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]

use crate::runtime::interpreter::Interpreter;
use crate::runtime::{InterpreterError, Value};

impl Interpreter {
    /// Evaluate `HtmlDocument` methods (HTTP-002-C)
    /// Complexity: 4 (within Toyota Way limits)
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn eval_html_document_method(
        &self,
        doc: &crate::stdlib::html::HtmlDocument,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "select" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "select() expects 1 argument (selector)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(selector) => {
                        let elements = doc.select(selector.as_ref()).map_err(|e| {
                            InterpreterError::RuntimeError(format!("select() failed: {e}"))
                        })?;
                        let values: Vec<Value> =
                            elements.into_iter().map(Value::HtmlElement).collect();
                        Ok(Value::Array(values.into()))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "select() expects a string selector".to_string(),
                    )),
                }
            }
            "query_selector" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "query_selector() expects 1 argument (selector)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(selector) => {
                        let element = doc.query_selector(selector.as_ref()).map_err(|e| {
                            InterpreterError::RuntimeError(format!("query_selector() failed: {e}"))
                        })?;
                        Ok(element.map_or(Value::Nil, Value::HtmlElement))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "query_selector() expects a string selector".to_string(),
                    )),
                }
            }
            "query_selector_all" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "query_selector_all() expects 1 argument (selector)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(selector) => {
                        let elements = doc.query_selector_all(selector.as_ref()).map_err(|e| {
                            InterpreterError::RuntimeError(format!(
                                "query_selector_all() failed: {e}"
                            ))
                        })?;
                        let values: Vec<Value> =
                            elements.into_iter().map(Value::HtmlElement).collect();
                        Ok(Value::Array(values.into()))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "query_selector_all() expects a string selector".to_string(),
                    )),
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown method '{}' on HtmlDocument",
                method
            ))),
        }
    }

    /// Evaluate `HtmlElement` methods (HTTP-002-C)
    /// Complexity: 4 (within Toyota Way limits)
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn eval_html_element_method(
        &self,
        elem: &crate::stdlib::html::HtmlElement,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "text" => {
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "text() expects no arguments".to_string(),
                    ));
                }
                Ok(Value::from_string(elem.text()))
            }
            "attr" => {
                if arg_values.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "attr() expects 1 argument (attribute name)".to_string(),
                    ));
                }
                match &arg_values[0] {
                    Value::String(attr_name) => {
                        let value = elem.attr(attr_name.as_ref());
                        Ok(value.map_or(Value::Nil, Value::from_string))
                    }
                    _ => Err(InterpreterError::RuntimeError(
                        "attr() expects a string attribute name".to_string(),
                    )),
                }
            }
            "html" => {
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "html() expects no arguments".to_string(),
                    ));
                }
                Ok(Value::from_string(elem.html()))
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown method '{}' on HtmlElement",
                method
            ))),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::stdlib::html::HtmlDocument;

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // HtmlDocument tests
    #[test]
    fn test_html_document_select() {
        let interp = make_interpreter();
        let html = "<html><body><p>Hello</p><p>World</p></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(
            &doc,
            "select",
            &[Value::from_string("p".to_string())],
        ).unwrap();

        if let Value::Array(elements) = result {
            assert_eq!(elements.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_html_document_select_wrong_args() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "select", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects 1 argument"));
    }

    #[test]
    fn test_html_document_select_wrong_type() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "select", &[Value::Integer(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("string selector"));
    }

    #[test]
    fn test_html_document_query_selector() {
        let interp = make_interpreter();
        let html = "<html><body><div id='main'>Content</div></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(
            &doc,
            "query_selector",
            &[Value::from_string("#main".to_string())],
        ).unwrap();

        assert!(matches!(result, Value::HtmlElement(_)));
    }

    #[test]
    fn test_html_document_query_selector_not_found() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(
            &doc,
            "query_selector",
            &[Value::from_string("#nonexistent".to_string())],
        ).unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_html_document_query_selector_wrong_args() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "query_selector", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_html_document_query_selector_wrong_type() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "query_selector", &[Value::Bool(true)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_html_document_query_selector_all() {
        let interp = make_interpreter();
        let html = "<html><body><span>A</span><span>B</span><span>C</span></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(
            &doc,
            "query_selector_all",
            &[Value::from_string("span".to_string())],
        ).unwrap();

        if let Value::Array(elements) = result {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_html_document_query_selector_all_wrong_args() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "query_selector_all", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_html_document_query_selector_all_wrong_type() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "query_selector_all", &[Value::Float(1.5)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_html_document_unknown_method() {
        let interp = make_interpreter();
        let html = "<html><body></body></html>";
        let doc = HtmlDocument::parse(html);

        let result = interp.eval_html_document_method(&doc, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown method"));
    }

    // HtmlElement tests
    #[test]
    fn test_html_element_text() {
        let interp = make_interpreter();
        let html = "<html><body><p id='greeting'>Hello World</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("#greeting").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "text", &[]).unwrap();
        assert_eq!(result, Value::from_string("Hello World".to_string()));
    }

    #[test]
    fn test_html_element_text_wrong_args() {
        let interp = make_interpreter();
        let html = "<html><body><p>Test</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("p").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "text", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    #[test]
    fn test_html_element_attr() {
        let interp = make_interpreter();
        let html = "<html><body><a href='https://example.com'>Link</a></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("a").unwrap().unwrap();

        let result = interp.eval_html_element_method(
            &elem,
            "attr",
            &[Value::from_string("href".to_string())],
        ).unwrap();

        assert_eq!(result, Value::from_string("https://example.com".to_string()));
    }

    #[test]
    fn test_html_element_attr_not_found() {
        let interp = make_interpreter();
        let html = "<html><body><p>Test</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("p").unwrap().unwrap();

        let result = interp.eval_html_element_method(
            &elem,
            "attr",
            &[Value::from_string("nonexistent".to_string())],
        ).unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_html_element_attr_wrong_args() {
        let interp = make_interpreter();
        let html = "<html><body><p>Test</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("p").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "attr", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects 1 argument"));
    }

    #[test]
    fn test_html_element_attr_wrong_type() {
        let interp = make_interpreter();
        let html = "<html><body><p>Test</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("p").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "attr", &[Value::Integer(42)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("string attribute name"));
    }

    #[test]
    fn test_html_element_html() {
        let interp = make_interpreter();
        let html = "<html><body><div><span>Inner</span></div></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("div").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "html", &[]).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("span"));
            assert!(s.contains("Inner"));
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_html_element_html_wrong_args() {
        let interp = make_interpreter();
        let html = "<html><body><p>Test</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("p").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "html", &[Value::Bool(false)]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    #[test]
    fn test_html_element_unknown_method() {
        let interp = make_interpreter();
        let html = "<html><body><p>Test</p></body></html>";
        let doc = HtmlDocument::parse(html);
        let elem = doc.query_selector("p").unwrap().unwrap();

        let result = interp.eval_html_element_method(&elem, "unknown", &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown method"));
    }
}
