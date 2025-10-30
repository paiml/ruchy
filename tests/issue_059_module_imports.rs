#![allow(missing_docs)]
//! GitHub Issue #59: Module/Import Syntax Tests
//!
//! Tests for multi-file project support with various import syntaxes.
//! Ticket: DOCS-059 (Documentation: Clarify module/import syntax)
//!
//! This test file demonstrates and validates ALL supported import syntaxes in Ruchy.

use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::ExprKind;

/// Test naming convention: `test_issue059`_<section>_<syntax>_<case>
/// Traceability: issue059 → GitHub issue #59

#[test]
fn test_issue059_01_use_simple() {
    // Rust-style use statement: use module::item
    let source = "use std::collections::HashMap";
    let ast = Parser::new(source).parse().expect("Should parse");

    // Should create Import expression
    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. }),
        "Expected Import or ImportAll"
    );
}

#[test]
fn test_issue059_02_use_wildcard() {
    // Wildcard import: use module::*
    let source = "use std::collections::*";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_03_use_aliased() {
    // Aliased import: use module::item as alias
    let source = "use std::collections::HashMap as Map";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_04_use_grouped() {
    // Grouped imports: use module::{item1, item2}
    let source = "use std::collections::{HashMap, BTreeMap}";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_05_import_simple() {
    // Python-style import: import module.item
    let source = "import std";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_06_import_nested() {
    // Nested module import: import module.submodule.item
    let source = "import std.collections.HashMap";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_07_from_import() {
    // From import: from module import item
    let source = "from std import println";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_08_from_import_multiple() {
    // From import multiple: from module import item1, item2
    let source = "from std.collections import HashMap, HashSet";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_09_from_import_wildcard() {
    // From import wildcard: from module import *
    let source = "from std.collections import *";
    let ast = Parser::new(source).parse().expect("Should parse");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_10_struct_with_imports() {
    // Real-world scenario from GitHub issue: struct with imports
    let source = r"
use types::DiscoveryFramework

struct TestRunner {
    framework: DiscoveryFramework,
}
";
    let ast = Parser::new(source).parse().expect("Should parse struct with imports");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_11_multiline_imports() {
    // Multiple import statements in sequence
    let source = r"
use std::collections::HashMap
use std::io::Read
import fs.readFile
from utils import helper

let x = 42
";
    let ast = Parser::new(source).parse().expect("Should parse multiple imports");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

#[test]
fn test_issue059_12_nested_grouped_imports() {
    // Nested grouped imports: use module::{sub1::{item1, item2}, sub2::item3}
    let source = "use std::{collections::{HashMap, HashSet}, io::Read}";
    let ast = Parser::new(source).parse().expect("Should parse nested grouped imports");

    assert!(
        matches!(ast.kind, ExprKind::Import { .. } | ExprKind::ImportAll { .. } | ExprKind::Block(_)),
        "Expected Import, ImportAll, or Block"
    );
}

// Property tests: Generate random valid import statements
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Generate valid module path: identifier(::identifier)*
    prop_compose! {
        fn arb_module_path()(
            parts in prop::collection::vec("[a-z][a-z0-9_]*", 1..5)
        ) -> String {
            parts.join("::")
        }
    }

    // Generate valid identifier
    prop_compose! {
        fn arb_identifier()(
            name in "[a-z][a-z0-9_]*"
        ) -> String {
            name
        }
    }

    proptest! {
        #[test]
        fn prop_use_statement_always_parses(
            module in arb_module_path(),
            item in arb_identifier()
        ) {
            let source = format!("use {module}::{item}");
            let result = Parser::new(&source).parse();
            prop_assert!(result.is_ok(), "use statement should always parse: {}", source);
        }

        #[test]
        fn prop_import_statement_always_parses(
            parts in prop::collection::vec("[a-z][a-z0-9_]*", 1..5)
        ) {
            let module = parts.join(".");
            let source = format!("import {module}");
            let result = Parser::new(&source).parse();
            prop_assert!(result.is_ok(), "import statement should always parse: {}", source);
        }

        #[test]
        fn prop_from_import_always_parses(
            module in arb_module_path(),
            items in prop::collection::vec(arb_identifier(), 1..4)
        ) {
            let module_dots = module.replace("::", ".");
            let items_str = items.join(", ");
            let source = format!("from {module_dots} import {items_str}");
            let result = Parser::new(&source).parse();
            prop_assert!(result.is_ok(), "from import should always parse: {}", source);
        }
    }
}

// Runtime execution tests
// EXTREME TDD: Test that import statements execute without errors (even if no-op for now)

#[test]
fn test_issue059_runtime_01_use_statement_executes() {
    use ruchy::runtime::interpreter::Interpreter;

    let source = r"
use std::collections::HashMap

let x = 42
x
";

    let ast = Parser::new(source).parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    // Should NOT error with "Expression type not yet implemented"
    assert!(
        result.is_ok(),
        "use statement should execute without error, got: {result:?}"
    );

    // Result should be 42 (the last expression)
    assert_eq!(result.unwrap().to_string(), "42");
}

#[test]
fn test_issue059_runtime_02_import_statement_executes() {
    use ruchy::runtime::interpreter::Interpreter;

    let source = r"
import std.collections

let x = 42
x
";

    let ast = Parser::new(source).parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    assert!(
        result.is_ok(),
        "import statement should execute without error, got: {result:?}"
    );

    assert_eq!(result.unwrap().to_string(), "42");
}

#[test]
fn test_issue059_runtime_03_from_import_executes() {
    use ruchy::runtime::interpreter::Interpreter;

    let source = r"
from std import println

let x = 42
x
";

    let ast = Parser::new(source).parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    assert!(
        result.is_ok(),
        "from import statement should execute without error, got: {result:?}"
    );

    assert_eq!(result.unwrap().to_string(), "42");
}

#[test]
fn test_issue059_runtime_04_wildcard_import_executes() {
    use ruchy::runtime::interpreter::Interpreter;

    let source = r"
use std::*

let x = 42
x
";

    let ast = Parser::new(source).parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    assert!(
        result.is_ok(),
        "wildcard import should execute without error, got: {result:?}"
    );

    assert_eq!(result.unwrap().to_string(), "42");
}

#[test]
fn test_issue059_runtime_05_multiple_imports_execute() {
    use ruchy::runtime::interpreter::Interpreter;

    let source = r"
use std::collections::HashMap
use std::io::Read
import fs.readFile
from utils import helper

let x = 42
x
";

    let ast = Parser::new(source).parse().expect("Should parse");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    assert!(
        result.is_ok(),
        "multiple import statements should execute without error, got: {result:?}"
    );

    assert_eq!(result.unwrap().to_string(), "42");
}
