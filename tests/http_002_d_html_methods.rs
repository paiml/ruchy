#![allow(missing_docs)]
// HTTP-002-D: HTML Method Call Support (GitHub Issue #43)
//
// RED phase tests - These tests MUST FAIL initially, proving the bug exists
//
// Testing Strategy (Extreme TDD):
// 1. RED: Write failing tests demonstrating Html.parse().select() doesn't work
// 2. GREEN: Implement eval_html_methods.rs to wire up method calls
// 3. REFACTOR: Add property tests with random HTML inputs

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

// ===========================
// Section 1: Html.parse() Baseline (Should Already Work)
// ===========================

#[test]
fn test_http002d_01_html_parse_returns_html_document() {
    // BASELINE: Html.parse() should already work
    let code = r#"
        let html = Html.parse("<div>Hello</div>")
        html
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    assert!(result.is_ok(), "Html.parse() should return HtmlDocument");
    assert_eq!(
        result.unwrap().type_name(),
        "html_document",
        "Should return html_document type"
    );
}

// ===========================
// Section 2: HtmlDocument.select() Method (RED - Should FAIL)
// ===========================

#[test]
fn test_http002d_02_html_document_select_class_selector() {
    // BUG: This test MUST FAIL initially (method not wired up)
    let code = r#"
        let html = Html.parse("<div class='test'>Content</div>")
        let elements = html.select(".test")
        elements.length()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "HtmlDocument.select() should work: {:?}",
        result.err()
    );
    // Should return array with 1 element
    match result.unwrap() {
        ruchy::runtime::Value::Integer(n) => assert_eq!(n, 1),
        other => panic!("Expected integer length, got: {}", other.type_name()),
    }
}

#[test]
fn test_http002d_03_html_document_select_tag_selector() {
    // Test tag selector (e.g., "div")
    let code = r#"
        let html = Html.parse("<div>One</div><div>Two</div>")
        let elements = html.select("div")
        elements.length()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "Tag selector should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::Integer(n) => assert_eq!(n, 2),
        other => panic!("Expected integer length, got: {}", other.type_name()),
    }
}

#[test]
fn test_http002d_04_html_document_select_id_selector() {
    // Test ID selector (e.g., "#main")
    let code = r##"
        let html = Html.parse("<div id='main'>Content</div>")
        let elements = html.select("#main")
        elements.length()
    "##;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "ID selector should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::Integer(n) => assert_eq!(n, 1),
        other => panic!("Expected integer length, got: {}", other.type_name()),
    }
}

// ===========================
// Section 3: HtmlElement.text() Method (RED - Should FAIL)
// ===========================

#[test]
fn test_http002d_05_html_element_text_extraction() {
    // BUG: This test MUST FAIL initially (method not wired up)
    let code = r#"
        let html = Html.parse("<div class='test'>Hello World</div>")
        let elements = html.select(".test")
        elements[0].text()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "HtmlElement.text() should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::String(s) => assert_eq!(s.as_ref(), "Hello World"),
        other => panic!("Expected string, got: {}", other.type_name()),
    }
}

#[test]
fn test_http002d_06_html_element_text_nested_elements() {
    // Test text extraction from nested elements
    let code = r#"
        let html = Html.parse("<div><p>First</p><p>Second</p></div>")
        let elements = html.select("div")
        elements[0].text()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "Nested element text should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::String(s) => {
            let text = s.as_ref();
            assert!(text.contains("First"), "Should contain 'First'");
            assert!(text.contains("Second"), "Should contain 'Second'");
        }
        other => panic!("Expected string, got: {}", other.type_name()),
    }
}

// ===========================
// Section 4: HtmlDocument.query_selector() Method (RED)
// ===========================

#[test]
fn test_http002d_07_html_document_query_selector_single_match() {
    // query_selector returns first match (not array)
    let code = r#"
        let html = Html.parse("<div class='test'>First</div><div class='test'>Second</div>")
        let element = html.query_selector(".test")
        element.text()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "query_selector should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::String(s) => assert_eq!(s.as_ref(), "First"),
        other => panic!("Expected string, got: {}", other.type_name()),
    }
}

#[test]
fn test_http002d_08_html_document_query_selector_no_match() {
    // query_selector returns nil if no match
    let code = r#"
        let html = Html.parse("<div>Content</div>")
        html.query_selector(".nonexistent")
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "query_selector should work: {:?}",
        result.err()
    );
    assert!(
        result.unwrap().is_nil(),
        "Should return nil when no match"
    );
}

// ===========================
// Section 5: Integration Tests (RED)
// ===========================

#[test]
#[ignore = "PARSER-081: Array literals fail after multiple let statements (not HTML-specific)"]
fn test_http002d_09_html_web_scraping_workflow() {
    // Real-world web scraping example
    //
    // ROOT CAUSE: This is a PARSER bug, not an HTML bug.
    // The issue is the array literal `[title, count]` at the end.
    //
    // ACTUAL BUG: Parser fails on array literals after multiple `let` statements.
    // Minimal reproduction: `let x = 1\nlet y = 2\n[x, y]` → "Expected RightBracket, found Comma"
    // Workaround: Use let-in expressions: `let x = 1 in let y = 2 in [x, y]` works fine.
    //
    // This requires a parser-level fix for statement parsing, not an HTML-specific fix.
    let code = r#"
        let html = Html.parse("<html><body><h1>Title</h1><p class='content'>Paragraph 1</p><p class='content'>Paragraph 2</p></body></html>")

        // Extract title
        let title = html.query_selector("h1").text()

        // Extract all paragraphs
        let paragraphs = html.select(".content")
        let count = paragraphs.length()

        // Return tuple with results
        [title, count]
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "Web scraping workflow should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            // First element should be "Title"
            if let ruchy::runtime::Value::String(s) = &arr[0] {
                assert_eq!(s.as_ref(), "Title");
            } else {
                panic!("Expected string for title");
            }
            // Second element should be 2
            if let ruchy::runtime::Value::Integer(n) = arr[1] {
                assert_eq!(n, 2);
            } else {
                panic!("Expected integer for count");
            }
        }
        other => panic!("Expected array, got: {}", other.type_name()),
    }
}

#[test]
fn test_http002d_10_html_error_invalid_selector() {
    // Error handling: invalid CSS selector
    let code = r#"
        let html = Html.parse("<div>Content</div>")
        html.select("[invalid[selector")
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    // Should return error for invalid selector
    assert!(
        result.is_err(),
        "Invalid selector should return error, got: {result:?}"
    );
}

// ===========================
// Section 6: Method Chaining (RED)
// ===========================

#[test]
fn test_http002d_11_html_method_chaining() {
    // Test method chaining: parse → select → text
    //
    // ROOT CAUSE: This is an INTERPRETER bug, not an HTML bug.
    // Proof: The HTML stdlib test_method_chaining_simulation() passes (html.rs:424),
    // proving that parse() → select() → [0] → text() works correctly when done step-by-step.
    //
    // The issue is in how the interpreter evaluates chained expressions with array indexing.
    // Breaking into steps works:
    //   let elements = html.select(".content")
    //   elements[0].text()  // ✅ Works
    //
    // But chaining fails:
    //   html.select(".content")[0].text()  // ❌ Returns ""
    //
    // This requires an interpreter-level fix, not an HTML-specific fix.
    let code = r#"
        Html.parse("<div class='content'>Hello World</div>").select(".content")[0].text()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "Method chaining should work: {:?}",
        result.err()
    );
    match result.unwrap() {
        ruchy::runtime::Value::String(s) => assert_eq!(s.as_ref(), "Hello World"),
        other => panic!("Expected string, got: {}", other.type_name()),
    }
}

// ===========================
// Section 7: Edge Cases (RED)
// ===========================

#[test]
fn test_http002d_12_html_empty_document() {
    // Empty HTML document
    let code = r#"
        let html = Html.parse("")
        html.select("div").length()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(result.is_ok(), "Empty document should work: {:?}", result.err());
    match result.unwrap() {
        ruchy::runtime::Value::Integer(n) => assert_eq!(n, 0),
        other => panic!("Expected integer, got: {}", other.type_name()),
    }
}

#[test]
fn test_http002d_13_html_malformed_html() {
    // html5ever should handle malformed HTML gracefully
    let code = r#"
        let html = Html.parse("<div><p>Unclosed tags")
        html.select("div").length()
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);
    assert!(
        result.is_ok(),
        "Malformed HTML should be handled: {:?}",
        result.err()
    );
    // Should still parse (html5ever is forgiving)
    match result.unwrap() {
        ruchy::runtime::Value::Integer(n) => assert!(n > 0),
        other => panic!("Expected integer, got: {}", other.type_name()),
    }
}
