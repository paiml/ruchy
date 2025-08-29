#![cfg(feature = "mcp")]
#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Tests for LSP semantic analyzer functionality
//!
//! This module tests the Language Server Protocol analyzer for semantic analysis,
//! targeting the 0% coverage lsp/analyzer.rs module.

use ruchy::lsp::SemanticAnalyzer;
use tower_lsp::lsp_types::{CompletionItemKind, DiagnosticSeverity, Position};

/// Test basic analyzer creation
#[test]
fn test_analyzer_creation() {
    let _analyzer = SemanticAnalyzer::new();
    // Should create successfully
    // Analyzer created successfully
    
    // Test Default trait
    let _default_analyzer = SemanticAnalyzer::default();
    // Analyzer created successfully
}

/// Test getting completions for basic keywords
#[test]
fn test_get_completions_keywords() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Should have keyword completions
    assert!(!completions.is_empty());
    
    // Check for specific keywords
    let has_fun = completions.iter().any(|c| c.label == "fun" && c.kind == Some(CompletionItemKind::KEYWORD));
    let has_let = completions.iter().any(|c| c.label == "let" && c.kind == Some(CompletionItemKind::KEYWORD));
    let has_if = completions.iter().any(|c| c.label == "if" && c.kind == Some(CompletionItemKind::KEYWORD));
    
    assert!(has_fun);
    assert!(has_let);
    assert!(has_if);
}

/// Test getting completions for built-in types
#[test]
fn test_get_completions_types() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Check for specific built-in types
    let has_i32 = completions.iter().any(|c| c.label == "i32" && c.kind == Some(CompletionItemKind::TYPE_PARAMETER));
    let has_string = completions.iter().any(|c| c.label == "String" && c.kind == Some(CompletionItemKind::TYPE_PARAMETER));
    let has_bool = completions.iter().any(|c| c.label == "bool" && c.kind == Some(CompletionItemKind::TYPE_PARAMETER));
    
    assert!(has_i32);
    assert!(has_string);
    assert!(has_bool);
}

/// Test completions include keyword details
#[test]
fn test_completions_keyword_details() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Find a keyword completion and check its details
    let fun_completion = completions.iter().find(|c| c.label == "fun").unwrap();
    assert!(fun_completion.detail.as_ref().unwrap().contains("Ruchy keyword"));
}

/// Test completions include type details
#[test]
fn test_completions_type_details() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Find a type completion and check its details
    let i32_completion = completions.iter().find(|c| c.label == "i32").unwrap();
    assert!(i32_completion.detail.as_ref().unwrap().contains("Built-in type"));
}

/// Test hover info on empty document
#[test]
fn test_get_hover_info_empty() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let hover = analyzer.get_hover_info("", position).unwrap();
    
    // Should return None for empty document (parsing fails)
    assert!(hover.is_none());
    
    // Empty document returns None, so no content to check
}

/// Test hover info on valid document
#[test]
fn test_get_hover_info_valid_document() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    let document = "fun test() { 42 }";
    
    let hover = analyzer.get_hover_info(document, position).unwrap();
    
    // Should return hover info for valid document
    assert!(hover.is_some());
}

/// Test hover info on invalid document
#[test]
fn test_get_hover_info_invalid_document() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    let invalid_document = "invalid syntax here @@##";
    
    let hover = analyzer.get_hover_info(invalid_document, position).unwrap();
    
    // Should return None for invalid document that fails parsing
    assert!(hover.is_none());
}

/// Test get definition (currently returns None)
#[test]
fn test_get_definition() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    let document = "fun test() { 42 }";
    
    let definition = analyzer.get_definition(document, position).unwrap();
    
    // Currently returns None
    assert!(definition.is_none());
}

/// Test diagnostics on valid document
#[test]
fn test_get_diagnostics_valid_document() {
    let mut analyzer = SemanticAnalyzer::new();
    let document = "fun test() { 42 }";
    
    let diagnostics = analyzer.get_diagnostics(document).unwrap();
    
    // Valid document should have no diagnostics
    assert!(diagnostics.is_empty());
}

/// Test diagnostics on invalid document
#[test]
fn test_get_diagnostics_invalid_document() {
    let mut analyzer = SemanticAnalyzer::new();
    let invalid_document = "invalid syntax @@##";
    
    let diagnostics = analyzer.get_diagnostics(invalid_document).unwrap();
    
    // Should have parse error diagnostic
    assert!(!diagnostics.is_empty());
    
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::ERROR));
    assert!(diagnostic.message.contains("Parse error"));
    assert_eq!(diagnostic.source.as_ref().unwrap(), "ruchy");
}

/// Test symbol table extraction from function
#[test]
fn test_symbol_extraction_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let document = "fun add(x: i32, y: i32) -> i32 { x + y }";
    
    let _diagnostics = analyzer.get_diagnostics(document).unwrap();
    
    // Now get completions to see if symbols were added
    let position = Position { line: 0, character: 0 };
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Should include function and parameter symbols
    let has_add_function = completions.iter().any(|c| c.label == "add" && c.kind == Some(CompletionItemKind::VARIABLE));
    
    // Function should be found (even if reported as variable in current implementation)
    assert!(has_add_function);
}

/// Test symbol table extraction from let binding
#[test]
fn test_symbol_extraction_let_binding() {
    let mut analyzer = SemanticAnalyzer::new();
    let document = "let x = 42";
    
    let _diagnostics = analyzer.get_diagnostics(document).unwrap();
    
    // Now get completions to see if variable was added
    let position = Position { line: 0, character: 0 };
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Should include variable symbol
    let has_x_variable = completions.iter().any(|c| c.label == "x" && c.kind == Some(CompletionItemKind::VARIABLE));
    assert!(has_x_variable);
}

/// Test symbol table extraction from struct definition
#[test]
fn test_symbol_extraction_struct() {
    let mut analyzer = SemanticAnalyzer::new();
    let document = "struct Point { x: i32, y: i32 }";
    
    let _diagnostics = analyzer.get_diagnostics(document).unwrap();
    
    // Now get completions to see if struct was added
    let position = Position { line: 0, character: 0 };
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Should include struct symbol
    let has_point_struct = completions.iter().any(|c| c.label == "Point" && c.kind == Some(CompletionItemKind::VARIABLE));
    assert!(has_point_struct);
}

/// Test completions with symbol documentation
#[test]
fn test_completions_with_symbol_documentation() {
    let mut analyzer = SemanticAnalyzer::new();
    let document = "fun test() { 42 }";
    
    // Process document to populate symbol table
    let _diagnostics = analyzer.get_diagnostics(document).unwrap();
    
    let position = Position { line: 0, character: 0 };
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Find function symbol and check it has kind detail
    let function_completion = completions.iter().find(|c| c.label == "test");
    if let Some(completion) = function_completion {
        assert!(completion.detail.is_some());
        assert_eq!(completion.detail.as_ref().unwrap(), "function");
    }
}

/// Test diagnostic code and source fields
#[test]
fn test_diagnostic_fields() {
    let mut analyzer = SemanticAnalyzer::new();
    let invalid_document = "@@## invalid syntax!!!";
    
    let diagnostics = analyzer.get_diagnostics(invalid_document).unwrap();
    assert!(!diagnostics.is_empty());
    
    let diagnostic = &diagnostics[0];
    assert!(diagnostic.code.is_some());
    if let Some(tower_lsp::lsp_types::NumberOrString::String(code)) = &diagnostic.code {
        assert_eq!(code, "parse_error");
    }
    assert_eq!(diagnostic.source.as_ref().unwrap(), "ruchy");
}

/// Test diagnostic range
#[test]
fn test_diagnostic_range() {
    let mut analyzer = SemanticAnalyzer::new();
    let invalid_document = "@@## bad syntax";
    
    let diagnostics = analyzer.get_diagnostics(invalid_document).unwrap();
    assert!(!diagnostics.is_empty());
    
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.range.start.line, 0);
    assert_eq!(diagnostic.range.start.character, 0);
    assert_eq!(diagnostic.range.end.line, 0);
    assert_eq!(diagnostic.range.end.character, 10);
}

/// Test all LSP keyword completions
#[test]
fn test_all_keyword_completions() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Check all expected keywords are present
    let expected_keywords = vec![
        "fun", "let", "if", "else", "match", "struct", "trait", "impl", "actor", "import",
        "for", "while", "break", "continue", "true", "false",
    ];
    
    for keyword in expected_keywords {
        let has_keyword = completions.iter().any(|c| c.label == keyword && c.kind == Some(CompletionItemKind::KEYWORD));
        assert!(has_keyword, "Missing keyword: {keyword}");
    }
}

/// Test all built-in type completions
#[test]
fn test_all_builtin_type_completions() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Check all expected types are present
    let expected_types = vec!["i32", "i64", "f32", "f64", "String", "bool", "()"];
    
    for type_name in expected_types {
        let has_type = completions.iter().any(|c| c.label == type_name && c.kind == Some(CompletionItemKind::TYPE_PARAMETER));
        assert!(has_type, "Missing type: {type_name}");
    }
}

/// Test completions count
#[test]
fn test_completions_count() {
    let analyzer = SemanticAnalyzer::new();
    let position = Position { line: 0, character: 0 };
    
    let completions = analyzer.get_completions("", position).unwrap();
    
    // Should have 16 keywords + 7 types = 23 base completions
    // (plus any symbol table entries)
    assert!(completions.len() >= 23);
}