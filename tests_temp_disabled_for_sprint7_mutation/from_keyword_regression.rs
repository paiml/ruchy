//! Parser regression test for GitHub issue #23
//!
//! Issue: 'from' is now a reserved keyword
//! Status: BREAKING CHANGE - needs deprecation strategy
//!
//! This test documents the breaking change where 'from' became a reserved
//! keyword, preventing its use as an identifier (parameter names, variables, fields).

use ruchy::frontend::Parser;

/// Helper: Parse code and return Ok if successful
fn parse_ok(code: &str) -> Result<(), String> {
    let mut parser = Parser::new(code);
    parser.parse().map_err(|e| e.to_string())?;
    Ok(())
}

/// Helper: Check if parsing fails with specific error
fn parse_fails_with(code: &str, expected_error: &str) -> bool {
    let mut parser = Parser::new(code);
    match parser.parse() {
        Err(e) => e.to_string().contains(expected_error),
        Ok(_) => false,
    }
}

#[test]
fn test_from_as_parameter_name_fails() {
    // This used to work in v1.89.0 but is now a breaking change
    let code = "fun shortest_path(from, to) { from }";

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "'from' should not be allowed as parameter name (reserved keyword), got: {result:?}"
    );

    // Verify it fails with some error (various possible error messages)
    let mut parser = Parser::new(code);
    let err = parser.parse().unwrap_err();
    let err_str = err.to_string();

    // Accept any of these error patterns
    let is_keyword_error = err_str.contains("reserved")
        || err_str.contains("keyword")
        || err_str.contains("Expected")
        || err_str.contains("parameters")
        || err_str.contains("identifier");

    assert!(
        is_keyword_error,
        "Should fail with keyword-related error, got: {err_str}"
    );
}

#[test]
fn test_from_as_variable_name_fails() {
    let code = "let from = \"NYC\"";

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "'from' should not be allowed as variable name (reserved keyword)"
    );
}

#[test]
fn test_from_in_struct_field_fails() {
    let code = r"
        struct Edge {
            from: String,
            to: String
        }
    ";

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "'from' should not be allowed as struct field name (reserved keyword)"
    );
}

#[test]
fn test_from_in_object_literal_fails() {
    let code = r#"
        let edge = {
            from: "A",
            to: "B"
        }
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "'from' should not be allowed as object field name (reserved keyword)"
    );
}

/// Test workarounds for the breaking change
#[test]
fn test_workaround_from_vertex() {
    // Recommended workaround: use from_vertex instead of from
    let code = "fun shortest_path(from_vertex, to_vertex) { from_vertex }";

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Workaround 'from_vertex' should work: {result:?}"
    );
}

#[test]
fn test_workaround_source() {
    // Alternative workaround: use source instead of from
    let code = "fun shortest_path(source, target) { source }";

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Workaround 'source' should work: {result:?}"
    );
}

#[test]
fn test_workaround_start() {
    // Another alternative: use start instead of from
    let code = "fun date_range(start, end) { start }";

    let result = parse_ok(code);
    assert!(result.is_ok(), "Workaround 'start' should work: {result:?}");
}

/// Test that 'from' might be intended for import syntax
#[test]
#[ignore] // FIXME: Import syntax not yet fully implemented
fn test_from_in_import_statement() {
    // This might be the intended use for the 'from' keyword
    let code = "from math import sqrt, pow";

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "'from' should work in import statements (intended use)"
    );
}

/// Test common patterns that are now broken
#[test]
fn test_graph_algorithm_pattern_broken() {
    // Graph algorithms commonly use 'from' and 'to'
    let code = r"
        fun dijkstra(graph, from, to) {
            // Dijkstra's algorithm implementation
            from
        }
    ";

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "Graph algorithms using 'from' parameter are now broken"
    );
}

#[test]
fn test_networking_pattern_broken() {
    // Networking code commonly uses 'from'
    let code = r#"
        fun send_packet(from, to, data) {
            println(f"Sending from {from} to {to}")
        }
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "Networking code using 'from' parameter is now broken"
    );
}

#[test]
fn test_date_range_pattern_broken() {
    // Date ranges commonly use 'from' and 'to'
    let code = r"
        struct DateRange {
            from: String,
            to: String
        }
    ";

    let result = parse_ok(code);
    assert!(
        result.is_err(),
        "Date range structs using 'from' field are now broken"
    );
}

/// Verify workarounds for all common patterns
#[test]
fn test_graph_algorithm_with_workaround() {
    let code = r"
        fun dijkstra(graph, source, target) {
            // Dijkstra's algorithm implementation
            source
        }
    ";

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Graph algorithms with 'source/target' workaround should work"
    );
}

#[test]
fn test_networking_with_workaround() {
    let code = r#"
        fun send_packet(sender, receiver, data) {
            println(f"Sending from {sender} to {receiver}")
        }
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Networking code with 'sender/receiver' workaround should work"
    );
}

#[test]
fn test_date_range_with_workaround() {
    let code = r"
        struct DateRange {
            start_date: String,
            end_date: String
        }
    ";

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Date ranges with 'start_date/end_date' workaround should work"
    );
}
