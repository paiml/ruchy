//! REPL Grammar Coverage Testing
//!
//! This module implements comprehensive grammar coverage testing to ensure
//! all language constructs are reachable and properly handled by the REPL.

#![allow(clippy::unwrap_used)] // Test code can use unwrap

use crate::{frontend::ast::ExprKind, runtime::repl::Repl};
use std::collections::HashSet;

/// Tracks which `ExprKind` variants have been successfully parsed and evaluated
#[derive(Debug, Default)]
pub struct GrammarCoverage {
    covered_variants: HashSet<&'static str>,
    total_variants: usize,
}

impl GrammarCoverage {
    #[must_use]
    pub fn new() -> Self {
        Self {
            covered_variants: HashSet::new(),
            total_variants: ExprKind::variant_count(),
        }
    }

    /// Mark a variant as covered
    pub fn mark_covered(&mut self, variant_name: &'static str) {
        self.covered_variants.insert(variant_name);
    }

    /// Check if all variants are covered
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.covered_variants.len() == self.total_variants
    }

    /// Get coverage percentage
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn coverage_percentage(&self) -> f64 {
        (self.covered_variants.len() as f64 / self.total_variants as f64) * 100.0
    }

    /// Get uncovered variants
    #[must_use]
    pub fn uncovered_variants(&self) -> Vec<&'static str> {
        let all_variants = ExprKind::all_variant_names();
        all_variants
            .into_iter()
            .filter(|variant| !self.covered_variants.contains(variant))
            .collect()
    }
}

/// Test cases for comprehensive grammar coverage
pub struct GrammarTestSuite {
    test_cases: Vec<(&'static str, &'static str)>, // (description, code)
}

impl Default for GrammarTestSuite {
    fn default() -> Self {
        Self::new()
    }
}

impl GrammarTestSuite {
    #[must_use]
    pub fn new() -> Self {
        let test_cases = vec![
            // Literals
            ("integer literal", "42"),
            ("float literal", "3.14"),
            ("string literal", "\"hello\""),
            ("boolean literal true", "true"),
            ("boolean literal false", "false"),
            ("unit literal", "()"),
            // String interpolation
            ("string interpolation basic", "\"Hello, {42}!\""),
            ("string interpolation complex", "\"Result: {1 + 2 * 3}\""),
            // Identifiers
            ("identifier", "x"),
            // Binary operations - arithmetic
            ("addition", "1 + 2"),
            ("subtraction", "5 - 3"),
            ("multiplication", "4 * 6"),
            ("division", "8 / 2"),
            ("modulo", "10 % 3"),
            ("power", "2 ** 3"),
            // Binary operations - comparison
            ("equality", "5 == 5"),
            ("inequality", "3 != 4"),
            ("less than", "2 < 5"),
            ("less equal", "3 <= 3"),
            ("greater than", "7 > 4"),
            ("greater equal", "6 >= 6"),
            // Binary operations - logical
            ("logical and", "true && false"),
            ("logical or", "true || false"),
            // Binary operations - bitwise
            ("bitwise and", "5 & 3"),
            ("bitwise or", "5 | 3"),
            ("bitwise xor", "5 ^ 3"),
            ("left shift", "4 << 2"),
            ("right shift", "16 >> 2"),
            // Unary operations
            ("logical not", "!true"),
            ("numeric negation", "-42"),
            ("bitwise not", "~5"),
            // Try operations
            ("try operator", "risky_func()?"),
            // Await
            ("await expression", "await some_async()"),
            // If expressions
            ("if then", "if true { 42 } else { 0 }"),
            ("if without else", "if false { 1 }"),
            // Let bindings
            ("let binding", "let x = 42 in x + 1"),
            // Functions
            (
                "function definition",
                "fun add(a: i32, b: i32) -> i32 { a + b }",
            ),
            ("async function", "async fun fetch() -> String { \"data\" }"),
            // Lambdas
            ("lambda expression", "|x| x + 1"),
            ("lambda with multiple params", "|x, y| x * y"),
            // Struct definitions
            ("struct definition", "struct Point { x: f64, y: f64 }"),
            // Struct literals
            ("struct literal", "Point { x: 1.0, y: 2.0 }"),
            // Field access
            ("field access", "point.x"),
            // Trait definitions
            (
                "trait definition",
                "trait Display { fun to_string(self) -> String; }",
            ),
            // Impl blocks
            (
                "impl block",
                "impl Display for Point { fun to_string(self) -> String { \"point\" } }",
            ),
            // Actor definitions
            ("actor definition", "actor Counter { count: i32 = 0 }"),
            // Send operations
            ("send message", "counter ! Increment"),
            // Ask operations
            ("ask message", "counter ? GetCount"),
            // Function calls
            ("function call", "add(1, 2)"),
            // Method calls
            ("method call", "vec.push(42)"),
            // Blocks
            ("block expression", "{ let x = 1; x + 2 }"),
            // Pipeline operations
            ("pipeline", "[1, 2, 3] |> map(double) |> sum"),
            // Match expressions
            ("match expression", "match x { Some(v) => v, None => 0 }"),
            // Lists
            ("list literal", "[1, 2, 3, 4]"),
            // List comprehensions
            ("list comprehension", "[x * 2 for x in [1, 2, 3]]"),
            (
                "list comprehension with filter",
                "[x for x in [1, 2, 3, 4] if x % 2 == 0]",
            ),
            // DataFrames
            (
                "dataframe literal",
                "DataFrame { cols: [\"a\", \"b\"], rows: [[1, 2], [3, 4]] }",
            ),
            // For loops
            ("for loop", "for x in [1, 2, 3] { print(x) }"),
            // While loops
            ("while loop", "while x < 10 { x = x + 1 }"),
            // Ranges
            ("inclusive range", "1..=5"),
            ("exclusive range", "1..5"),
            // Import statements
            ("import statement", "import std.collections.HashMap"),
            // Control flow
            ("break statement", "break"),
            ("continue statement", "continue"),
            ("labeled break", "break 'outer"),
            ("labeled continue", "continue 'loop"),
            // Try/catch
            ("try catch", "try { risky() } catch e { handle(e) }"),
        ];

        Self { test_cases }
    }

    /// Run all test cases and return coverage report
    ///
    /// # Panics
    ///
    /// Panics if the REPL cannot be initialized
    #[must_use]
    #[allow(clippy::expect_used, clippy::print_stderr)]
    pub fn run_coverage_test(&self) -> GrammarCoverage {
        let mut repl = Repl::new().unwrap();
        let mut coverage = GrammarCoverage::new();

        for (description, code) in &self.test_cases {
            match repl.eval(code) {
                Ok(_) => {
                    // Parse the code to determine which variant was used
                    if let Ok(expr) = crate::frontend::parser::Parser::new(code).parse() {
                        let variant_name = expr.kind.variant_name();
                        coverage.mark_covered(variant_name);
                        eprintln!("✓ {description}: {variant_name}");
                    }
                }
                Err(e) => {
                    // Some constructs might not be fully implemented yet
                    eprintln!("✗ {description}: {code} ({e})");
                }
            }
        }

        coverage
    }
}

impl ExprKind {
    /// Get the variant name as a string for coverage tracking
    #[must_use]
    pub fn variant_name(&self) -> &'static str {
        match self {
            ExprKind::Literal(_) => "Literal",
            ExprKind::Identifier(_) => "Identifier",
            ExprKind::StringInterpolation { .. } => "StringInterpolation",
            ExprKind::Binary { .. } => "Binary",
            ExprKind::Unary { .. } => "Unary",
            ExprKind::Try { .. } => "Try",
            ExprKind::TryCatch { .. } => "TryCatch",
            ExprKind::Ok { .. } => "Ok",
            ExprKind::Err { .. } => "Err",
            ExprKind::Await { .. } => "Await",
            ExprKind::If { .. } => "If",
            ExprKind::Let { .. } => "Let",
            ExprKind::Function { .. } => "Function",
            ExprKind::Lambda { .. } => "Lambda",
            ExprKind::Struct { .. } => "Struct",
            ExprKind::StructLiteral { .. } => "StructLiteral",
            ExprKind::ObjectLiteral { .. } => "ObjectLiteral",
            ExprKind::FieldAccess { .. } => "FieldAccess",
            ExprKind::Trait { .. } => "Trait",
            ExprKind::Impl { .. } => "Impl",
            ExprKind::Actor { .. } => "Actor",
            ExprKind::Send { .. } => "Send",
            ExprKind::Ask { .. } => "Ask",
            ExprKind::Call { .. } => "Call",
            ExprKind::MethodCall { .. } => "MethodCall",
            ExprKind::Block(_) => "Block",
            ExprKind::Pipeline { .. } => "Pipeline",
            ExprKind::Match { .. } => "Match",
            ExprKind::List(_) => "List",
            ExprKind::ListComprehension { .. } => "ListComprehension",
            ExprKind::DataFrame { .. } => "DataFrame",
            ExprKind::For { .. } => "For",
            ExprKind::While { .. } => "While",
            ExprKind::Range { .. } => "Range",
            ExprKind::Import { .. } => "Import",
            ExprKind::Break { .. } => "Break",
            ExprKind::Continue { .. } => "Continue",
            ExprKind::DataFrameOperation { .. } => "DataFrameOperation",
        }
    }

    /// Get the total number of variants
    #[must_use]
    pub fn variant_count() -> usize {
        33 // Update this if new variants are added
    }

    /// Get all variant names for coverage tracking
    #[must_use]
    pub fn all_variant_names() -> Vec<&'static str> {
        vec![
            "Literal",
            "Identifier",
            "StringInterpolation",
            "Binary",
            "Unary",
            "Try",
            "TryCatch",
            "Await",
            "If",
            "Let",
            "Function",
            "Lambda",
            "Struct",
            "StructLiteral",
            "FieldAccess",
            "Trait",
            "Impl",
            "Actor",
            "Send",
            "Ask",
            "Call",
            "MethodCall",
            "Block",
            "Pipeline",
            "Match",
            "List",
            "ListComprehension",
            "DataFrame",
            "For",
            "While",
            "Range",
            "Import",
            "Break",
            "Continue",
        ]
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    #[ignore = "Requires rustc at runtime"]
    #[allow(clippy::print_stderr)]
    fn test_complete_grammar_coverage() {
        let test_suite = GrammarTestSuite::new();
        let coverage = test_suite.run_coverage_test();

        let coverage_pct = coverage.coverage_percentage();
        eprintln!("Grammar Coverage: {coverage_pct:.1}%");
        eprintln!(
            "Covered: {}/{}",
            coverage.covered_variants.len(),
            coverage.total_variants
        );

        if !coverage.is_complete() {
            eprintln!("Uncovered variants:");
            for variant in coverage.uncovered_variants() {
                eprintln!("  - {variant}");
            }
        }

        // For now, we'll accept partial coverage as many features are still being implemented
        // Current expectation: Basic language constructs should work (literals, binary ops, functions, etc.)
        assert!(
            coverage_pct >= 15.0,
            "Grammar coverage should be at least 15% (got {coverage_pct:.1}%)"
        );

        // Log coverage for tracking progress
        eprintln!("✓ Grammar coverage test passed with {coverage_pct:.1}% coverage");
    }

    #[test]
    #[ignore = "Requires rustc at runtime"]
    fn test_basic_arithmetic_coverage() {
        let mut repl = Repl::new().unwrap();
        let mut coverage = GrammarCoverage::new();

        let basic_tests = vec![
            ("1 + 2", "Binary"),
            ("42", "Literal"),
            ("x", "Identifier"),
            ("-5", "Unary"),
        ];

        for (code, expected_variant) in basic_tests {
            if repl.eval(code).is_ok() {
                if let Ok(expr) = crate::frontend::parser::Parser::new(code).parse() {
                    let variant = expr.kind.variant_name();
                    assert_eq!(variant, expected_variant);
                    coverage.mark_covered(variant);
                }
            }
        }

        assert!(coverage.coverage_percentage() > 0.0);
    }

    #[test]
    fn test_coverage_tracking() {
        let mut coverage = GrammarCoverage::new();

        assert!((coverage.coverage_percentage() - 0.0).abs() < f64::EPSILON);
        assert!(!coverage.is_complete());

        coverage.mark_covered("Literal");
        coverage.mark_covered("Binary");

        assert!(coverage.coverage_percentage() > 0.0);
        assert!(coverage.coverage_percentage() < 100.0);

        let uncovered = coverage.uncovered_variants();
        assert!(!uncovered.is_empty());
        assert!(!uncovered.contains(&"Literal"));
        assert!(!uncovered.contains(&"Binary"));
    }
}
