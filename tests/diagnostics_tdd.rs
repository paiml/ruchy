//! Comprehensive TDD test suite for diagnostics
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every diagnostic path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::frontend::diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel, DiagnosticBuilder};
use ruchy::frontend::Span;

// ==================== DIAGNOSTIC CREATION TESTS ====================

#[test]
fn test_diagnostic_new() {
    let diagnostic = Diagnostic::new(
        DiagnosticLevel::Error,
        "Test error",
        Span::new(0, 10)
    );
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
    assert_eq!(diagnostic.message(), "Test error");
    assert_eq!(diagnostic.span(), Span::new(0, 10));
}

#[test]
fn test_diagnostic_warning() {
    let diagnostic = Diagnostic::warning("Unused variable", Span::new(5, 15));
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Warning);
    assert_eq!(diagnostic.message(), "Unused variable");
}

#[test]
fn test_diagnostic_error() {
    let diagnostic = Diagnostic::error("Syntax error", Span::new(0, 5));
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
    assert_eq!(diagnostic.message(), "Syntax error");
}

#[test]
fn test_diagnostic_info() {
    let diagnostic = Diagnostic::info("Consider using", Span::new(10, 20));
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Info);
    assert_eq!(diagnostic.message(), "Consider using");
}

#[test]
fn test_diagnostic_hint() {
    let diagnostic = Diagnostic::hint("Try adding semicolon", Span::new(30, 31));
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Hint);
    assert_eq!(diagnostic.message(), "Try adding semicolon");
}

// ==================== DIAGNOSTIC BUILDER TESTS ====================

#[test]
fn test_diagnostic_builder_basic() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Type mismatch")
        .span(Span::new(0, 10))
        .build();
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Error);
    assert_eq!(diagnostic.message(), "Type mismatch");
}

#[test]
fn test_diagnostic_builder_with_note() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Undefined variable")
        .span(Span::new(5, 10))
        .note("Did you mean 'x'?")
        .build();
    
    assert!(diagnostic.has_note());
    assert_eq!(diagnostic.note(), Some("Did you mean 'x'?"));
}

#[test]
fn test_diagnostic_builder_with_help() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Missing semicolon")
        .span(Span::new(20, 21))
        .help("Add ';' at the end of the statement")
        .build();
    
    assert!(diagnostic.has_help());
    assert_eq!(diagnostic.help(), Some("Add ';' at the end of the statement"));
}

#[test]
fn test_diagnostic_builder_with_code() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Type error")
        .span(Span::new(0, 5))
        .code("E0308")
        .build();
    
    assert_eq!(diagnostic.code(), Some("E0308"));
}

#[test]
fn test_diagnostic_builder_complete() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Warning)
        .message("Unused import")
        .span(Span::new(0, 20))
        .code("W0001")
        .note("This import is never used")
        .help("Remove the unused import")
        .build();
    
    assert_eq!(diagnostic.level(), DiagnosticLevel::Warning);
    assert_eq!(diagnostic.code(), Some("W0001"));
    assert!(diagnostic.has_note());
    assert!(diagnostic.has_help());
}

// ==================== DIAGNOSTIC LEVEL TESTS ====================

#[test]
fn test_diagnostic_level_severity() {
    assert!(DiagnosticLevel::Error.is_error());
    assert!(!DiagnosticLevel::Warning.is_error());
    assert!(!DiagnosticLevel::Info.is_error());
    assert!(!DiagnosticLevel::Hint.is_error());
}

#[test]
fn test_diagnostic_level_ordering() {
    assert!(DiagnosticLevel::Error > DiagnosticLevel::Warning);
    assert!(DiagnosticLevel::Warning > DiagnosticLevel::Info);
    assert!(DiagnosticLevel::Info > DiagnosticLevel::Hint);
}

#[test]
fn test_diagnostic_level_display() {
    assert_eq!(format!("{}", DiagnosticLevel::Error), "error");
    assert_eq!(format!("{}", DiagnosticLevel::Warning), "warning");
    assert_eq!(format!("{}", DiagnosticLevel::Info), "info");
    assert_eq!(format!("{}", DiagnosticLevel::Hint), "hint");
}

// ==================== DIAGNOSTIC KIND TESTS ====================

#[test]
fn test_diagnostic_kind_syntax() {
    let kind = DiagnosticKind::SyntaxError;
    assert!(matches!(kind, DiagnosticKind::SyntaxError));
}

#[test]
fn test_diagnostic_kind_type() {
    let kind = DiagnosticKind::TypeError;
    assert!(matches!(kind, DiagnosticKind::TypeError));
}

#[test]
fn test_diagnostic_kind_undefined() {
    let kind = DiagnosticKind::UndefinedVariable("x".to_string());
    assert!(matches!(kind, DiagnosticKind::UndefinedVariable(_)));
}

#[test]
fn test_diagnostic_kind_unused() {
    let kind = DiagnosticKind::UnusedVariable("y".to_string());
    assert!(matches!(kind, DiagnosticKind::UnusedVariable(_)));
}

// ==================== SPAN TESTS ====================

#[test]
fn test_span_creation() {
    let span = Span::new(10, 20);
    assert_eq!(span.start(), 10);
    assert_eq!(span.end(), 20);
    assert_eq!(span.len(), 10);
}

#[test]
fn test_span_empty() {
    let span = Span::new(5, 5);
    assert!(span.is_empty());
    assert_eq!(span.len(), 0);
}

#[test]
fn test_span_contains() {
    let span = Span::new(10, 20);
    assert!(span.contains(15));
    assert!(!span.contains(5));
    assert!(!span.contains(25));
}

#[test]
fn test_span_merge() {
    let span1 = Span::new(10, 20);
    let span2 = Span::new(15, 25);
    let merged = span1.merge(span2);
    
    assert_eq!(merged.start(), 10);
    assert_eq!(merged.end(), 25);
}

// ==================== DIAGNOSTIC COLLECTION TESTS ====================

#[test]
fn test_diagnostic_collection_new() {
    let mut collection = DiagnosticCollection::new();
    assert!(collection.is_empty());
    assert_eq!(collection.len(), 0);
}

#[test]
fn test_diagnostic_collection_add() {
    let mut collection = DiagnosticCollection::new();
    
    collection.add(Diagnostic::error("Error 1", Span::new(0, 5)));
    collection.add(Diagnostic::warning("Warning 1", Span::new(10, 15)));
    
    assert_eq!(collection.len(), 2);
    assert!(!collection.is_empty());
}

#[test]
fn test_diagnostic_collection_has_errors() {
    let mut collection = DiagnosticCollection::new();
    
    collection.add(Diagnostic::warning("Warning", Span::new(0, 5)));
    assert!(!collection.has_errors());
    
    collection.add(Diagnostic::error("Error", Span::new(10, 15)));
    assert!(collection.has_errors());
}

#[test]
fn test_diagnostic_collection_filter_errors() {
    let mut collection = DiagnosticCollection::new();
    
    collection.add(Diagnostic::error("Error 1", Span::new(0, 5)));
    collection.add(Diagnostic::warning("Warning", Span::new(10, 15)));
    collection.add(Diagnostic::error("Error 2", Span::new(20, 25)));
    
    let errors = collection.errors();
    assert_eq!(errors.len(), 2);
}

#[test]
fn test_diagnostic_collection_filter_warnings() {
    let mut collection = DiagnosticCollection::new();
    
    collection.add(Diagnostic::error("Error", Span::new(0, 5)));
    collection.add(Diagnostic::warning("Warning 1", Span::new(10, 15)));
    collection.add(Diagnostic::warning("Warning 2", Span::new(20, 25)));
    
    let warnings = collection.warnings();
    assert_eq!(warnings.len(), 2);
}

// ==================== DIAGNOSTIC FORMATTING TESTS ====================

#[test]
fn test_diagnostic_format_simple() {
    let diagnostic = Diagnostic::error("Syntax error", Span::new(0, 5));
    let formatted = format!("{}", diagnostic);
    
    assert!(formatted.contains("error"));
    assert!(formatted.contains("Syntax error"));
}

#[test]
fn test_diagnostic_format_with_code() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Type mismatch")
        .span(Span::new(0, 10))
        .code("E0308")
        .build();
    
    let formatted = format!("{}", diagnostic);
    assert!(formatted.contains("E0308"));
}

#[test]
fn test_diagnostic_format_with_note_and_help() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Missing semicolon")
        .span(Span::new(20, 21))
        .note("Statement requires semicolon")
        .help("Add ';' at the end")
        .build();
    
    let formatted = format!("{}", diagnostic);
    assert!(formatted.contains("note:") || formatted.contains("Note:"));
    assert!(formatted.contains("help:") || formatted.contains("Help:"));
}

// ==================== SOURCE LOCATION TESTS ====================

#[test]
fn test_diagnostic_with_line_info() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Undefined variable")
        .span(Span::new(45, 50))
        .line(3)
        .column(5)
        .build();
    
    assert_eq!(diagnostic.line(), Some(3));
    assert_eq!(diagnostic.column(), Some(5));
}

#[test]
fn test_diagnostic_with_file_info() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Import error")
        .span(Span::new(0, 20))
        .file("main.ruchy")
        .build();
    
    assert_eq!(diagnostic.file(), Some("main.ruchy"));
}

// ==================== DIAGNOSTIC SEVERITY TESTS ====================

#[test]
fn test_diagnostic_is_error() {
    let error = Diagnostic::error("Error", Span::new(0, 5));
    let warning = Diagnostic::warning("Warning", Span::new(0, 5));
    
    assert!(error.is_error());
    assert!(!warning.is_error());
}

#[test]
fn test_diagnostic_is_warning() {
    let warning = Diagnostic::warning("Warning", Span::new(0, 5));
    let info = Diagnostic::info("Info", Span::new(0, 5));
    
    assert!(warning.is_warning());
    assert!(!info.is_warning());
}

// ==================== RELATED DIAGNOSTICS TESTS ====================

#[test]
fn test_diagnostic_with_related() {
    let main_diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
        .message("Variable already defined")
        .span(Span::new(50, 55))
        .build();
    
    let related = Diagnostic::info("Previous definition here", Span::new(10, 15));
    
    let with_related = main_diagnostic.with_related(vec![related]);
    assert!(with_related.has_related());
    assert_eq!(with_related.related().len(), 1);
}

// ==================== DIAGNOSTIC SUPPRESSION TESTS ====================

#[test]
fn test_diagnostic_suppression() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Warning)
        .message("Unused variable")
        .span(Span::new(0, 10))
        .suppressible(true)
        .build();
    
    assert!(diagnostic.is_suppressible());
}

#[test]
fn test_diagnostic_not_suppressible() {
    let diagnostic = Diagnostic::error("Syntax error", Span::new(0, 5));
    assert!(!diagnostic.is_suppressible());
}

// ==================== DIAGNOSTIC QUICK FIX TESTS ====================

#[test]
fn test_diagnostic_with_quick_fix() {
    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Warning)
        .message("Missing semicolon")
        .span(Span::new(20, 20))
        .quick_fix("Insert ';'", ";")
        .build();
    
    assert!(diagnostic.has_quick_fix());
    assert_eq!(diagnostic.quick_fix_text(), Some(";"));
}

// ==================== DIAGNOSTIC COLLECTION ITERATION TESTS ====================

#[test]
fn test_diagnostic_collection_iter() {
    let mut collection = DiagnosticCollection::new();
    
    collection.add(Diagnostic::error("Error 1", Span::new(0, 5)));
    collection.add(Diagnostic::warning("Warning", Span::new(10, 15)));
    collection.add(Diagnostic::error("Error 2", Span::new(20, 25)));
    
    let mut count = 0;
    for _diagnostic in collection.iter() {
        count += 1;
    }
    assert_eq!(count, 3);
}

#[test]
fn test_diagnostic_collection_clear() {
    let mut collection = DiagnosticCollection::new();
    
    collection.add(Diagnostic::error("Error", Span::new(0, 5)));
    collection.add(Diagnostic::warning("Warning", Span::new(10, 15)));
    
    assert_eq!(collection.len(), 2);
    
    collection.clear();
    assert_eq!(collection.len(), 0);
    assert!(collection.is_empty());
}

// Helper struct for collection tests
struct DiagnosticCollection {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollection {
    fn new() -> Self {
        Self { diagnostics: Vec::new() }
    }
    
    fn add(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
    
    fn len(&self) -> usize {
        self.diagnostics.len()
    }
    
    fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
    
    fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }
    
    fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error()).collect()
    }
    
    fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_warning()).collect()
    }
    
    fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
    
    fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

// Run all tests with: cargo test diagnostics_tdd --test diagnostics_tdd