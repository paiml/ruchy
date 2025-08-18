#![allow(clippy::print_stdout)] // Tests can print debug info
#![allow(clippy::unwrap_used)] // Tests need unwrap

use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{ExprKind, ImportItem};

#[test]
fn test_empty_import_path() {
    let result = Parser::new("import").parse();
    assert!(result.is_err(), "Should fail on empty import");
}

#[test]
fn test_import_with_trailing_coloncolon() {
    let result = Parser::new("import std::").parse();
    assert!(result.is_err(), "Should fail on trailing ::");
}

#[test]
fn test_import_with_empty_braces() {
    let result = Parser::new("import std::collections::{}").parse();
    // This might be valid (importing nothing) or invalid - let's see
    if let Ok(expr) = result {
        if let ExprKind::Import { items, .. } = &expr.kind {
            assert_eq!(items.len(), 0, "Empty braces should import nothing");
        }
    } else {
        // Also acceptable - empty imports might be rejected
    }
}

#[test]
fn test_import_with_duplicate_items() {
    let input = "import std::collections::{HashMap, HashMap}";
    let result = Parser::new(input).parse();
    // Should parse successfully even with duplicates
    assert!(result.is_ok(), "Should handle duplicate imports");
    if let Ok(expr) = result {
        if let ExprKind::Import { items, .. } = &expr.kind {
            assert_eq!(items.len(), 2); // Both duplicates are kept
        }
    }
}

#[test]
fn test_import_with_nested_braces() {
    // This should fail - no nested braces in imports
    let result = Parser::new("import std::{collections::{HashMap}}").parse();
    assert!(result.is_err(), "Should fail on nested braces");
}

#[test]
fn test_import_wildcard_with_other_items() {
    // Can't mix wildcard with specific items
    let result = Parser::new("import std::collections::{*, HashMap}").parse();
    // This should probably fail or just import wildcard
    match result {
        Ok(expr) => {
            if let ExprKind::Import { items, .. } = &expr.kind {
                // Check how it's handled
                println!("Mixed wildcard import resulted in {} items", items.len());
            }
        }
        Err(e) => {
            println!("Mixed wildcard import failed: {e}");
        }
    }
}

#[test]
fn test_very_long_import_path() {
    let long_path = "a::b::c::d::e::f::g::h::i::j::k::l::m::n::o::p::q::r::s::t::u::v::w::x::y::z";
    let input = format!("import {long_path}");
    let result = Parser::new(&input).parse();
    assert!(result.is_ok(), "Should handle long paths");
}

#[test]
fn test_import_with_unicode() {
    // Identifiers with unicode (if supported)
    let result = Parser::new("import 日本::語").parse();
    // This will likely fail as most parsers don't support unicode identifiers
    assert!(result.is_err(), "Unicode identifiers should probably fail");
}

#[test]
fn test_import_with_numbers_in_path() {
    let result = Parser::new("import std2::collections3").parse();
    assert!(result.is_ok(), "Should handle numbers in identifiers");
}

#[test]
fn test_import_keyword_as_alias() {
    // Using a keyword as an alias should fail
    let result = Parser::new("import std::collections::HashMap as fn").parse();
    assert!(result.is_err(), "Should not allow keyword as alias");
}

#[test]
fn test_multiple_wildcards() {
    let result = Parser::new("import std::collections::{*, *}").parse();
    // Should this be allowed?
    if let Ok(expr) = result {
        if let ExprKind::Import { items, .. } = &expr.kind {
            // Check if multiple wildcards are collapsed or kept
            let wildcard_count = items.iter().filter(|i| matches!(i, ImportItem::Wildcard)).count();
            println!("Multiple wildcards resulted in {wildcard_count} wildcard items");
        }
    } else {
        // Also reasonable to reject
    }
}

#[test]
fn test_import_with_special_characters() {
    // Various invalid characters
    assert!(Parser::new("import std::collec-tions").parse().is_err());
    assert!(Parser::new("import std::collec+tions").parse().is_err());
    assert!(Parser::new("import std::collec.tions").parse().is_err());
    assert!(Parser::new("import std::collec@tions").parse().is_err());
}

#[test]
fn test_export_with_duplicates() {
    let result = Parser::new("export { foo, foo, bar }").parse();
    assert!(result.is_ok(), "Should handle duplicate exports");
}

#[test]
fn test_module_with_keyword_name() {
    let result = Parser::new("module fn { }").parse();
    assert!(result.is_err(), "Should not allow keyword as module name");
}

#[test]
fn test_import_alias_to_same_name() {
    // Aliasing to the same name - pointless but should work
    let result = Parser::new("import std::HashMap as HashMap").parse();
    assert!(result.is_ok(), "Should allow redundant aliasing");
}

#[test]
fn test_import_with_only_commas() {
    let result = Parser::new("import std::collections::{,,,}").parse();
    // Should fail on empty items between commas
    assert!(result.is_err(), "Should fail on only commas");
}

#[test]
fn test_import_missing_closing_brace() {
    let result = Parser::new("import std::collections::{HashMap, Vec").parse();
    assert!(result.is_err(), "Should fail on missing closing brace");
}

#[test]
fn test_module_missing_closing_brace() {
    let result = Parser::new("module Test { 42").parse();
    assert!(result.is_err(), "Should fail on missing closing brace");
}

#[test]
fn test_export_missing_closing_brace() {
    let result = Parser::new("export { foo, bar").parse();
    assert!(result.is_err(), "Should fail on missing closing brace");
}