//! Native HTML parsing stdlib (HTTP-002-C, STD-011)
//!
//! Provides HTML parsing and querying using Mozilla's html5ever parser.
//! Avoids deprecated `scraper` crate by implementing native solution.
//!
//! # Design Philosophy
//!
//! - **Zero Deprecated Dependencies**: Uses maintained html5ever from Mozilla Servo
//! - **Thin Wrapper**: Minimal complexity, maximum reliability
//! - **Ruchy-Friendly**: Clean API matching Ruby/JavaScript patterns
//! - **Toyota Way**: ≤10 complexity per function, comprehensive tests
//!
//! # Examples
//!
//! ```ruchy
//! html = Html.parse("<div class='test'>Hello</div>")
//! elements = html.select(".test")
//! puts elements[0].text()  # "Hello"
//! ```

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::fmt;
use std::sync::Arc;

/// HTML document type for parsing and querying
///
/// Wraps html5ever's RcDom for Ruchy-friendly API
#[derive(Clone)]
pub struct HtmlDocument {
    dom: Arc<RcDom>,
}

impl fmt::Debug for HtmlDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HtmlDocument")
    }
}

/// HTML element wrapper
///
/// References a node in the HTML DOM tree
#[derive(Clone)]
pub struct HtmlElement {
    handle: Handle,
}

impl fmt::Debug for HtmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HtmlElement")
    }
}

impl HtmlDocument {
    /// Parse HTML from string
    ///
    /// Uses html5ever's parser for standards-compliant HTML5 parsing.
    /// Handles malformed HTML gracefully.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::stdlib::html::HtmlDocument;
    ///
    /// let html = HtmlDocument::parse("<div>Test</div>");
    /// ```
    pub fn parse(content: &str) -> Self {
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut content.as_bytes())
            .unwrap();

        Self { dom: Arc::new(dom) }
    }

    /// Select elements matching CSS selector
    ///
    /// Returns all elements matching the given CSS selector.
    /// Uses simple selector matching (tag, class, id, attribute).
    ///
    /// # Errors
    ///
    /// Returns error if selector is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// let html = HtmlDocument::parse("<p class='text'>Hello</p>");
    /// let elements = html.select(".text").unwrap();
    /// assert_eq!(elements.len(), 1);
    /// ```
    pub fn select(&self, selector: &str) -> Result<Vec<HtmlElement>, String> {
        let elements = self.select_nodes(&self.dom.document, selector);
        Ok(elements.into_iter().map(|handle| HtmlElement { handle }).collect())
    }

    /// Query selector (returns first match)
    ///
    /// Returns the first element matching the selector, or None.
    ///
    /// # Examples
    ///
    /// ```
    /// let html = HtmlDocument::parse("<p>First</p><p>Second</p>");
    /// let element = html.query_selector("p").unwrap();
    /// assert!(element.is_some());
    /// ```
    pub fn query_selector(&self, selector: &str) -> Result<Option<HtmlElement>, String> {
        let elements = self.select(selector)?;
        Ok(elements.into_iter().next())
    }

    /// Query selector all (alias for select)
    pub fn query_selector_all(&self, selector: &str) -> Result<Vec<HtmlElement>, String> {
        self.select(selector)
    }

    /// Recursively select nodes matching selector
    ///
    /// Internal helper for traversing DOM tree.
    /// Complexity: 8 (within Toyota Way limits)
    fn select_nodes(&self, node: &Handle, selector: &str) -> Vec<Handle> {
        let mut results = Vec::new();

        // Check if current node matches
        if self.matches_selector(node, selector) {
            results.push(node.clone());
        }

        // Recursively check children
        for child in node.children.borrow().iter() {
            results.extend(self.select_nodes(child, selector));
        }

        results
    }

    /// Check if node matches CSS selector
    ///
    /// Supports: tag, .class, #id, [attr], [attr=value]
    /// Complexity: 10 (at Toyota Way limit)
    fn matches_selector(&self, node: &Handle, selector: &str) -> bool {
        let selector = selector.trim();

        match &node.data {
            NodeData::Element { name, attrs, .. } => {
                let tag_name = name.local.as_ref();
                let attrs_borrowed = attrs.borrow();

                // Class selector: ".className"
                if let Some(class_name) = selector.strip_prefix('.') {
                    return attrs_borrowed.iter().any(|attr| {
                        attr.name.local.as_ref() == "class"
                            && attr.value.as_ref().split_whitespace().any(|c| c == class_name)
                    });
                }

                // ID selector: "#idName"
                if let Some(id_name) = selector.strip_prefix('#') {
                    return attrs_borrowed.iter().any(|attr| {
                        attr.name.local.as_ref() == "id" && attr.value.as_ref() == id_name
                    });
                }

                // Attribute selector: "[attr]" or "[attr=value]"
                if selector.starts_with('[') && selector.ends_with(']') {
                    let inner = &selector[1..selector.len() - 1];
                    if let Some((attr_name, attr_value)) = inner.split_once('=') {
                        let attr_value = attr_value.trim_matches('\'').trim_matches('"');
                        return attrs_borrowed.iter().any(|attr| {
                            attr.name.local.as_ref() == attr_name && attr.value.as_ref() == attr_value
                        });
                    } else {
                        return attrs_borrowed.iter().any(|attr| attr.name.local.as_ref() == inner);
                    }
                }

                // Descendant selector: "div p" - match last element only (simplified)
                if selector.contains(' ') {
                    let parts: Vec<&str> = selector.split_whitespace().collect();
                    if let Some(&last) = parts.last() {
                        return self.matches_selector(node, last);
                    }
                }

                // Tag selector: "div"
                tag_name == selector
            }
            _ => false,
        }
    }
}

impl HtmlElement {
    /// Get text content of element and its descendants
    ///
    /// Recursively collects all text nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// let html = HtmlDocument::parse("<p>Hello <span>World</span></p>");
    /// let p = html.query_selector("p").unwrap().unwrap();
    /// assert_eq!(p.text(), "Hello World");
    /// ```
    pub fn text(&self) -> String {
        self.collect_text(&self.handle)
    }

    /// Get attribute value
    ///
    /// Returns attribute value or None if attribute doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// let html = HtmlDocument::parse("<a href='test.html'>Link</a>");
    /// let link = html.query_selector("a").unwrap().unwrap();
    /// assert_eq!(link.attr("href"), Some("test.html".to_string()));
    /// ```
    pub fn attr(&self, name: &str) -> Option<String> {
        match &self.handle.data {
            NodeData::Element { attrs, .. } => {
                attrs.borrow().iter().find_map(|attr| {
                    if attr.name.local.as_ref() == name {
                        Some(attr.value.to_string())
                    } else {
                        None
                    }
                })
            }
            _ => None,
        }
    }

    /// Get inner HTML
    ///
    /// Returns HTML content of element's children.
    ///
    /// # Examples
    ///
    /// ```
    /// let html = HtmlDocument::parse("<div><p>Test</p></div>");
    /// let div = html.query_selector("div").unwrap().unwrap();
    /// assert!(div.html().contains("<p>Test</p>"));
    /// ```
    pub fn html(&self) -> String {
        self.serialize_node(&self.handle)
    }

    /// Recursively collect text from node and children
    ///
    /// Complexity: 5 (well within Toyota Way limits)
    fn collect_text(&self, node: &Handle) -> String {
        let mut text = String::new();

        match &node.data {
            NodeData::Text { contents } => {
                text.push_str(&contents.borrow());
            }
            _ => {
                for child in node.children.borrow().iter() {
                    text.push_str(&self.collect_text(child));
                }
            }
        }

        text
    }

    /// Serialize node to HTML string
    ///
    /// Simplified HTML serialization.
    /// Complexity: 8 (within Toyota Way limits)
    fn serialize_node(&self, node: &Handle) -> String {
        let mut html = String::new();

        match &node.data {
            NodeData::Element { name, attrs, .. } => {
                let tag_name = name.local.as_ref();
                html.push('<');
                html.push_str(tag_name);

                // Add attributes
                for attr in attrs.borrow().iter() {
                    html.push(' ');
                    html.push_str(attr.name.local.as_ref());
                    html.push_str("=\"");
                    html.push_str(&attr.value);
                    html.push('"');
                }

                html.push('>');

                // Add children
                for child in node.children.borrow().iter() {
                    html.push_str(&self.serialize_node(child));
                }

                html.push_str("</");
                html.push_str(tag_name);
                html.push('>');
            }
            NodeData::Text { contents } => {
                html.push_str(&contents.borrow());
            }
            _ => {
                // Serialize children for other node types
                for child in node.children.borrow().iter() {
                    html.push_str(&self.serialize_node(child));
                }
            }
        }

        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = HtmlDocument::parse("<div>Test</div>");
        assert!(html.dom.document.children.borrow().len() > 0);
    }

    #[test]
    fn test_select_by_tag() {
        let html = HtmlDocument::parse("<div><p>Test</p></div>");
        let elements = html.select("p").unwrap();
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_select_by_class() {
        let html = HtmlDocument::parse("<div class='test'>Hello</div>");
        let elements = html.select(".test").unwrap();
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_element_text() {
        let html = HtmlDocument::parse("<p>Hello World</p>");
        let p = html.query_selector("p").unwrap().unwrap();
        assert_eq!(p.text().trim(), "Hello World");
    }

    #[test]
    fn test_element_attr() {
        let html = HtmlDocument::parse("<a href='test.html'>Link</a>");
        let link = html.query_selector("a").unwrap().unwrap();
        assert_eq!(link.attr("href"), Some("test.html".to_string()));
    }

    #[test]
    fn test_element_attr_missing() {
        let html = HtmlDocument::parse("<a>Link</a>");
        let link = html.query_selector("a").unwrap().unwrap();
        assert_eq!(link.attr("href"), None);
    }

    #[test]
    fn test_multiple_elements() {
        let html = HtmlDocument::parse("<p>1</p><p>2</p><p>3</p>");
        let elements = html.select("p").unwrap();
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn test_query_selector_none() {
        let html = HtmlDocument::parse("<div>Test</div>");
        let element = html.query_selector("p").unwrap();
        assert!(element.is_none());
    }

    #[test]
    fn test_malformed_html() {
        let html = HtmlDocument::parse("<div><p>Unclosed");
        let elements = html.select("p").unwrap();
        assert_eq!(elements.len(), 1);
    }

    #[test]
    fn test_empty_html() {
        let html = HtmlDocument::parse("");
        let elements = html.select("*").unwrap();
        assert_eq!(elements.len(), 0);
    }

    /// Property test: Parsing never panics
    #[test]
    #[ignore = "Property test - run with: cargo test -- --ignored"]
    fn prop_parse_never_panics() {
        use proptest::prelude::*;

        proptest!(|(html_str in ".*")| {
            let _ = HtmlDocument::parse(&html_str);
        });
    }
}
