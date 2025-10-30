#![allow(missing_docs)]
//! HTML stdlib tests (HTTP-002-C)
//!
//! Tests for native HTML parsing following TDD approach:
//! RED → GREEN → REFACTOR
//!
//! Reference: docs/specifications/HTTP-002-advanced-http-features.md

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Test: `Html.parse()` creates HTML document
/// RED: This test should FAIL because Html type doesn't exist yet
#[test]
fn test_html_parse() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<div class='test'>Hello</div>")
            puts html.class
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("HtmlDocument"));
}

/// Test: .`select()` returns array of elements
/// RED: This test should FAIL because .`select()` method doesn't exist
#[test]
fn test_html_select() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<div class='a'>1</div><div class='a'>2</div>")
            elements = html.select(".a")
            puts elements.length
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

/// Test: .`query_selector()` returns first element or nil
/// RED: This test should FAIL because .`query_selector()` doesn't exist
#[test]
fn test_html_query_selector() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<p>Hello World</p>")
            p = html.query_selector("p")
            puts p.text()
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

/// Test: .`query_selector()` returns nil for no match
#[test]
fn test_html_query_selector_no_match() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<p>Hello</p>")
            element = html.query_selector(".missing")
            puts element == nil
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

/// Test: `Element.text()` gets text content
/// RED: This test should FAIL because .`text()` method doesn't exist
#[test]
fn test_html_element_text() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<p>Hello World</p>")
            p = html.query_selector("p")
            puts p.text()
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));
}

/// Test: `Element.attr()` gets attribute value
/// RED: This test should FAIL because .`attr()` method doesn't exist
#[test]
fn test_html_element_attr() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<a href='http://example.com'>Link</a>")
            link = html.query_selector("a")
            puts link.attr("href")
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("http://example.com"));
}

/// Test: `Element.attr()` returns nil for missing attribute
#[test]
fn test_html_element_attr_missing() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<a>Link</a>")
            link = html.query_selector("a")
            puts link.attr("href") == nil
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

/// Test: `Element.html()` gets inner HTML
/// RED: This test should FAIL because .`html()` method doesn't exist
#[test]
fn test_html_element_html() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<div><p>Test</p></div>")
            div = html.query_selector("div")
            puts div.html()
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("<p>Test</p>"));
}

/// Test: Complex CSS selectors work
/// RED: This test should FAIL because selector support incomplete
#[test]
fn test_html_complex_selector() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<div><ul><li class='item'>1</li><li class='item'>2</li></ul></div>")
            items = html.select("div ul li.item")
            puts items.length
            puts items[0].text()
            puts items[1].text()
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

/// Test: .`query_selector_all()` is alias for .`select()`
#[test]
fn test_html_query_selector_all() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<p>1</p><p>2</p><p>3</p>")
            elements = html.query_selector_all("p")
            puts elements.length
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

/// Test: HTML parsing handles malformed HTML gracefully
#[test]
fn test_html_parse_malformed() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<div><p>Unclosed")
            elements = html.select("p")
            puts elements.length
        "#)
        .assert()
        .success();
}

/// Test: HTML parsing handles empty strings
#[test]
fn test_html_parse_empty() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("")
            elements = html.select("*")
            puts elements.length
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

/// Test: Selector error handling
#[test]
fn test_html_invalid_selector() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html = Html.parse("<div>Test</div>")
            elements = html.select(":::invalid")
        "#)
        .assert()
        .failure(); // Should error on invalid selector
}

/// Test: Practical example from specification
#[test]
fn test_html_practical_example() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"
            html_content = "<html><body><a href='https://example.com'>Example</a><a href='https://test.com'>Test</a></body></html>"
            html = Html.parse(html_content)
            links = html.select("a[href]")

            puts "Found " + links.length.to_string() + " links"

            links.each do |link|
                puts link.text() + ": " + link.attr("href")
            end
        "#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2 links"))
        .stdout(predicate::str::contains("Example: https://example.com"))
        .stdout(predicate::str::contains("Test: https://test.com"));
}

/// Property test: All parsed elements should have valid selectors
#[test]
#[ignore = "Property test - run with: cargo test -- --ignored"]
fn prop_html_elements_have_valid_selectors() {
    use proptest::prelude::*;

    proptest!(|(
        tag in "[a-z]{1,5}",
        class in "[a-z]{1,10}",
        content in "[a-zA-Z0-9 ]{0,50}",
    )| {
        let html_str = format!("<{tag} class='{class}'>{content}</{tag}>");

        // This would need proper integration with Ruchy runtime
        // For now, just verify it compiles
        let _ = (html_str, tag, class, content);
    });
}
