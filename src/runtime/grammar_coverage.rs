//! Grammar coverage matrix for REPL testing
//!
//! Tracks coverage of all grammar productions to ensure complete testing
use crate::frontend::ast::Expr;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
/// Statistics for a single grammar production
#[derive(Default, Debug)]
pub struct ProductionStats {
    pub hit_count: usize,
    pub success_count: usize,
    pub avg_latency_ns: u64,
    pub error_patterns: Vec<String>,
}
/// Grammar coverage tracking matrix
#[derive(Default)]
pub struct GrammarCoverageMatrix {
    pub productions: HashMap<&'static str, ProductionStats>,
    pub ast_variants: HashSet<String>,
    pub uncovered: Vec<&'static str>,
}
impl GrammarCoverageMatrix {
    /// Create a new coverage matrix
    pub fn new() -> Self {
        Self::default()
    }
    /// Record a parse attempt
    pub fn record(
        &mut self,
        production: &'static str,
        _input: &str,
        result: Result<Expr>,
        elapsed: Duration,
    ) {
        let stats = self.productions.entry(production).or_default();
        stats.hit_count += 1;
        let elapsed_ns =
            u64::try_from(elapsed.as_nanos().min(u128::from(u64::MAX))).unwrap_or(u64::MAX);
        stats.avg_latency_ns = if stats.hit_count == 1 {
            elapsed_ns
        } else {
            (stats.avg_latency_ns * (stats.hit_count as u64 - 1) + elapsed_ns)
                / stats.hit_count as u64
        };
        match result {
            Ok(ast) => {
                stats.success_count += 1;
                self.record_ast_variants(&ast);
            }
            Err(e) => {
                let error_msg = e.to_string();
                if !stats.error_patterns.contains(&error_msg) {
                    stats.error_patterns.push(error_msg);
                }
            }
        }
    }
    /// Record AST variant usage
    fn record_ast_variants(&mut self, expr: &Expr) {
        use crate::frontend::ast::ExprKind;
        let variant_name = match &expr.kind {
            ExprKind::Literal(_) => "Literal",
            ExprKind::Identifier(_) => "Identifier",
            ExprKind::Binary { .. } => "Binary",
            ExprKind::Unary { .. } => "Unary",
            ExprKind::Call { .. } => "Call",
            ExprKind::MethodCall { .. } => "MethodCall",
            ExprKind::If { .. } => "If",
            ExprKind::Match { .. } => "Match",
            ExprKind::Block(_) => "Block",
            ExprKind::Let { .. } => "Let",
            ExprKind::Function { .. } => "Function",
            ExprKind::Lambda { .. } => "Lambda",
            ExprKind::Throw { .. } => "Throw",
            ExprKind::Ok { .. } => "Ok",
            ExprKind::Err { .. } => "Err",
            ExprKind::While { .. } => "While",
            ExprKind::For { .. } => "For",
            ExprKind::Struct { .. } => "Struct",
            ExprKind::TupleStruct { .. } => "TupleStruct",
            ExprKind::StructLiteral { .. } => "StructLiteral",
            ExprKind::ObjectLiteral { .. } => "ObjectLiteral",
            ExprKind::FieldAccess { .. } => "FieldAccess",
            ExprKind::Trait { .. } => "Trait",
            ExprKind::Impl { .. } => "Impl",
            ExprKind::Extension { .. } => "Extension",
            ExprKind::Await { .. } => "Await",
            ExprKind::List(_) => "List",
            ExprKind::ListComprehension { .. } => "ListComprehension",
            ExprKind::StringInterpolation { .. } => "StringInterpolation",
            ExprKind::QualifiedName { .. } => "QualifiedName",
            ExprKind::Send { .. } => "Send",
            ExprKind::Ask { .. } => "Ask",
            ExprKind::Command { .. } => "Command",
            ExprKind::Macro { .. } => "Macro",
            ExprKind::Actor { .. } => "Actor",
            ExprKind::DataFrame { .. } => "DataFrame",
            ExprKind::DataFrameOperation { .. } => "DataFrameOperation",
            ExprKind::Pipeline { .. } => "Pipeline",
            ExprKind::Import { .. } => "Import",
            ExprKind::Export { .. } => "Export",
            ExprKind::Module { .. } => "Module",
            ExprKind::Range { .. } => "Range",
            ExprKind::Break { .. } => "Break",
            ExprKind::Continue { .. } => "Continue",
            ExprKind::Assign { .. } => "Assign",
            ExprKind::CompoundAssign { .. } => "CompoundAssign",
            _ => "Other",
        };
        self.ast_variants.insert(variant_name.to_string());
    }
    /// Check if coverage is complete
    pub fn is_complete(&self, required_productions: usize) -> bool {
        self.productions.len() >= required_productions && self.uncovered.is_empty()
    }
    /// Assert that coverage is complete
    ///
    /// # Panics
    ///
    /// Panics if there are uncovered productions or if the number of covered
    /// productions is less than the required amount.
    pub fn assert_complete(&self, required_productions: usize) {
        assert!(
            self.uncovered.is_empty(),
            "Uncovered productions: {:?}",
            self.uncovered
        );
        assert!(
            self.productions.len() >= required_productions,
            "Only {} of {} productions covered",
            self.productions.len(),
            required_productions
        );
    }
    /// Get coverage percentage
    pub fn get_coverage_percentage(&self) -> f64 {
        if self.uncovered.is_empty() && self.productions.is_empty() {
            return 0.0;
        }
        // Count uncovered productions that haven't been covered
        let uncovered_count = self
            .uncovered
            .iter()
            .filter(|prod| !self.productions.contains_key(**prod))
            .count();
        let total = self.productions.len() + uncovered_count;
        if total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let percentage = (self.productions.len() as f64 / total as f64) * 100.0;
        percentage
    }
    /// Generate a coverage report (alias for `report()`)
    pub fn generate_report(&self) -> String {
        self.report()
    }
    /// Generate a coverage report
    pub fn report(&self) -> String {
        use std::fmt::Write;
        let mut report = String::new();
        report.push_str("Grammar Coverage Report\n");
        report.push_str("=======================\n\n");
        let coverage_percentage = self.get_coverage_percentage();
        let _ = writeln!(&mut report, "Coverage: {coverage_percentage:.1}%");
        let _ = writeln!(
            &mut report,
            "Productions covered: {}",
            self.productions.len()
        );
        let _ = writeln!(
            &mut report,
            "AST variants seen: {}",
            self.ast_variants.len()
        );
        let total_hits: usize = self.productions.values().map(|s| s.hit_count).sum();
        let total_success: usize = self.productions.values().map(|s| s.success_count).sum();
        let _ = writeln!(&mut report, "Total attempts: {total_hits}");
        let success_rate = if total_hits > 0 {
            #[allow(clippy::cast_precision_loss)]
            let rate = (total_success as f64 / total_hits as f64) * 100.0;
            rate
        } else {
            0.0
        };
        let _ = writeln!(&mut report, "Success rate: {success_rate:.2}%");
        // Find slowest productions
        let mut slowest: Vec<_> = self.productions.iter().collect();
        slowest.sort_by_key(|(_, stats)| stats.avg_latency_ns);
        slowest.reverse();
        if !slowest.is_empty() {
            report.push_str("\nSlowest productions:\n");
            for (name, stats) in slowest.iter().take(5) {
                #[allow(clippy::cast_precision_loss)]
                let ms = stats.avg_latency_ns as f64 / 1_000_000.0;
                let _ = writeln!(&mut report, "  {name}: {ms:.2}ms");
            }
        }
        if !self.uncovered.is_empty() {
            report.push_str("\nUncovered productions:\n");
            for prod in &self.uncovered {
                let _ = writeln!(&mut report, "  - {prod}");
            }
        }
        report
    }
}
/// All grammar productions that need coverage
pub const GRAMMAR_PRODUCTIONS: &[(&str, &str)] = &[
    // Core literals (5)
    ("literal_int", "42"),
    ("literal_float", "3.15"),
    ("literal_string", r#""hello""#),
    ("literal_bool", "true"),
    ("literal_unit", "()"),
    // Binary operators (12)
    ("op_assign", "x = 5"),
    ("op_logical_or", "a || b"),
    ("op_logical_and", "a && b"),
    ("op_equality", "x == y"),
    ("op_comparison", "x < y"),
    ("op_bitwise_or", "a | b"),
    ("op_bitwise_xor", "a ^ b"),
    ("op_bitwise_and", "a & b"),
    ("op_shift", "x << 2"),
    ("op_range", "0..10"),
    ("op_add", "x + y"),
    ("op_mul", "x * y"),
    // Unary operators (3)
    ("op_neg", "-x"),
    ("op_not", "!x"),
    ("op_ref", "&value"),
    // Control flow (5)
    ("if_expr", "if x > 0 { 1 } else { -1 }"),
    ("match_expr", "match x { Some(y) => y, None => 0 }"),
    ("for_loop", "for x in 0..10 { print(x) }"),
    ("while_loop", "while x > 0 { x = x - 1 }"),
    ("loop_expr", "loop { break 42 }"),
    // Function calls (5) - CRITICAL: These were missing!
    ("call_simple", "println(42)"),
    ("call_args", "println(\"Hello\", \"World\")"),
    ("call_expr", "add(2 + 3, 4 * 5)"),
    ("call_nested", "println(add(1, 2))"),
    ("call_builtin", "print(\"test\")"),
    // Functions (4)
    ("fun_decl", "fun add(a: Int, b: Int) -> Int { a + b }"),
    ("fun_generic", "fun id<T>(x: T) -> T { x }"),
    ("lambda", "|x| x * 2"),
    ("lambda_typed", "|x: Int| -> Int { x * 2 }"),
    // Pattern matching (6)
    ("pattern_bind", "let x = 5"),
    ("pattern_tuple", "let (x, y) = (1, 2)"),
    ("pattern_struct", "let Point { x, y } = p"),
    ("pattern_enum", "let Some(x) = opt"),
    ("pattern_slice", "let [head, ..tail] = list"),
    ("pattern_guard", "match x { n if n > 0 => n }"),
    // Type system (5)
    ("type_simple", "let x: Int = 5"),
    ("type_generic", "let v: Vec<Int> = vec![1,2,3]"),
    ("type_function", "let f: Fn(Int) -> Int = |x| x"),
    ("type_tuple", "let t: (Int, String) = (1, \"hi\")"),
    ("type_option", "let opt: Option<Int> = Some(5)"),
    // Structs/Traits/Impls (3)
    ("struct_decl", "struct Point { x: Float, y: Float }"),
    ("trait_decl", "trait Show { fun show(self) -> String }"),
    (
        "impl_block",
        "impl Show for Point { fun show(self) -> String { \"...\" } }",
    ),
    // Actor system (4)
    ("actor_decl", "actor Counter { state count: Int = 0 }"),
    ("actor_handler", "on Increment { self.count += 1 }"),
    ("send_op", "counter <- Increment"),
    ("ask_op", "let n = counter <? GetCount"),
    // DataFrame operations (6)
    ("df_literal", "df![a = [1,2,3], b = [4,5,6]]"),
    ("df_filter", "df >> filter(col(\"age\") > 18)"),
    ("df_select", "df >> select([\"name\", \"age\"])"),
    ("df_groupby", "df >> groupby(\"dept\")"),
    ("df_agg", "df >> agg([mean(\"salary\"), count()])"),
    ("df_join", "df1 >> join(df2, on: \"id\")"),
    // Pipeline operators (3)
    ("pipe_simple", "data >> filter(|x| x > 0)"),
    ("pipe_method", "text >> trim() >> uppercase()"),
    ("pipe_nested", "x >> (|y| y >> double() >> square())"),
    // String interpolation (2)
    ("string_interp", r#"f"Hello {name}""#),
    ("string_complex", r#"f"Result: {compute(x):.2f}""#),
    // Import/Export (3)
    ("import_simple", "import std::fs"),
    (
        "import_multi",
        "import std::collections::{HashMap, HashSet}",
    ),
    ("export", "export { Point, distance }"),
    // Macros (2)
    ("macro_println", "println!(\"Hello\", \"World\")"),
    ("macro_vec", "vec![1, 2, 3]"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use crate::BinaryOp;
    use crate::UnaryOp;
    use std::time::Duration;

    // Sprint 14: Grammar coverage tests

    #[test]
    fn test_coverage_matrix_creation() {
        let matrix = GrammarCoverageMatrix::new();
        assert!(matrix.productions.is_empty());
        assert!(matrix.ast_variants.is_empty());
        assert!(matrix.uncovered.is_empty());
    }

    #[test]
    fn test_production_stats_default() {
        let stats = ProductionStats::default();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.avg_latency_ns, 0);
        assert!(stats.error_patterns.is_empty());
    }

    #[test]
    fn test_record_success() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record("literal", "42", Ok(expr), Duration::from_millis(10));

        assert_eq!(matrix.productions.len(), 1);
        let stats = &matrix.productions["literal"];
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.success_count, 1);
        assert!(stats.avg_latency_ns > 0);
        assert!(stats.error_patterns.is_empty());
    }

    #[test]
    fn test_record_failure() {
        let mut matrix = GrammarCoverageMatrix::new();
        let error = anyhow::anyhow!("Parse error");

        matrix.record("invalid", "bad input", Err(error), Duration::from_millis(5));

        assert_eq!(matrix.productions.len(), 1);
        let stats = &matrix.productions["invalid"];
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.success_count, 0);
        assert!(stats.avg_latency_ns > 0);
        assert_eq!(stats.error_patterns.len(), 1);
        assert!(stats.error_patterns[0].contains("Parse error"));
    }

    #[test]
    fn test_record_multiple_attempts() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record("literal", "42", Ok(expr.clone()), Duration::from_millis(10));
        matrix.record("literal", "43", Ok(expr), Duration::from_millis(20));

        let stats = &matrix.productions["literal"];
        assert_eq!(stats.hit_count, 2);
        assert_eq!(stats.success_count, 2);
        // Average should be around 15ms in nanoseconds
        assert!(stats.avg_latency_ns > 10_000_000 && stats.avg_latency_ns < 20_000_000);
    }

    #[test]
    fn test_ast_variant_recording() {
        let mut matrix = GrammarCoverageMatrix::new();

        let literal_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        let identifier_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span { start: 0, end: 1 },
        );

        matrix.record("literal", "42", Ok(literal_expr), Duration::from_millis(1));
        matrix.record(
            "identifier",
            "x",
            Ok(identifier_expr),
            Duration::from_millis(1),
        );

        assert!(matrix.ast_variants.contains("Literal"));
        assert!(matrix.ast_variants.contains("Identifier"));
    }

    #[test]
    fn test_error_pattern_deduplication() {
        let mut matrix = GrammarCoverageMatrix::new();

        let error1 = anyhow::anyhow!("Same error");
        let error2 = anyhow::anyhow!("Same error");
        let error3 = anyhow::anyhow!("Different error");

        matrix.record("test", "input1", Err(error1), Duration::from_millis(1));
        matrix.record("test", "input2", Err(error2), Duration::from_millis(1));
        matrix.record("test", "input3", Err(error3), Duration::from_millis(1));

        let stats = &matrix.productions["test"];
        assert_eq!(stats.hit_count, 3);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.error_patterns.len(), 2); // Only unique errors
    }

    /* Commented out - API mismatch
    #[test]
    fn test_calculate_stats() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        // Add some successes and failures
        matrix.record("test1", "input", Ok(expr.clone()), Duration::from_millis(1));
        matrix.record("test1", "input", Ok(expr.clone()), Duration::from_millis(1));
        matrix.record("test1", "input", Err(anyhow::anyhow!("error")), Duration::from_millis(1));

        matrix.record("test2", "input", Ok(expr), Duration::from_millis(1));

        let (total, successful, coverage_pct) = matrix.calculate_stats();

        assert_eq!(total, 4);
        assert_eq!(successful, 3);
        assert_eq!(coverage_pct, 75.0);
    }
    */

    #[test]
    fn test_generate_report() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record("literal", "42", Ok(expr), Duration::from_millis(10));

        let report = matrix.generate_report();

        assert!(report.contains("Grammar Coverage Report"));
        assert!(report.contains("literal"));
        assert!(report.contains("100.00%"));
    }

    /* Commented out - API mismatch
    #[test]
    fn test_check_coverage_threshold_success() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record("test", "input", Ok(expr), Duration::from_millis(1));

        let result = matrix.check_coverage_threshold(50.0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100.0);
    }

    #[test]
    fn test_check_coverage_threshold_failure() {
        let mut matrix = GrammarCoverageMatrix::new();

        matrix.record("test", "input", Err(anyhow::anyhow!("error")), Duration::from_millis(1));

        let result = matrix.check_coverage_threshold(50.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("below threshold"));
    }
    */

    #[test]
    fn test_grammar_productions_array() {
        // Test that GRAMMAR_PRODUCTIONS array is properly defined
        // GRAMMAR_PRODUCTIONS is a const array, so this is a compile-time guarantee

        // Check a few samples
        let first = super::GRAMMAR_PRODUCTIONS[0];
        assert!(!first.0.is_empty());
        assert!(!first.1.is_empty());
    }

    // Test 13: is_complete returns true when coverage is sufficient
    #[test]
    fn test_is_complete_true() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record(
            "prod1",
            "input1",
            Ok(expr.clone()),
            Duration::from_millis(1),
        );
        matrix.record("prod2", "input2", Ok(expr), Duration::from_millis(1));

        assert!(matrix.is_complete(2));
        assert!(!matrix.is_complete(3)); // Not enough productions
    }

    // Test 14: is_complete returns false when there are uncovered productions
    #[test]
    fn test_is_complete_false_uncovered() {
        let mut matrix = GrammarCoverageMatrix::new();
        matrix.uncovered.push("uncovered_prod");

        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );
        matrix.record("prod1", "input1", Ok(expr), Duration::from_millis(1));

        assert!(!matrix.is_complete(1));
    }

    // Test 15: get_coverage_percentage with empty matrix
    #[test]
    fn test_get_coverage_percentage_empty() {
        let matrix = GrammarCoverageMatrix::new();
        assert!((matrix.get_coverage_percentage() - 0.0).abs() < f64::EPSILON);
    }

    // Test 16: get_coverage_percentage with 100% coverage
    #[test]
    fn test_get_coverage_percentage_full() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record(
            "prod1",
            "input1",
            Ok(expr.clone()),
            Duration::from_millis(1),
        );
        matrix.record("prod2", "input2", Ok(expr), Duration::from_millis(1));

        assert!((matrix.get_coverage_percentage() - 100.0).abs() < 0.01);
    }

    // Test 17: get_coverage_percentage with partial coverage
    #[test]
    fn test_get_coverage_percentage_partial() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        matrix.record("prod1", "input1", Ok(expr), Duration::from_millis(1));
        matrix.uncovered.push("uncovered1");
        matrix.uncovered.push("uncovered2");

        // 1 covered, 2 uncovered = 33.33%
        let pct = matrix.get_coverage_percentage();
        assert!(pct > 30.0 && pct < 40.0);
    }

    // Test 18: record_ast_variants Binary
    #[test]
    fn test_ast_variant_binary() {
        let mut matrix = GrammarCoverageMatrix::new();

        let binary_expr = Expr::new(
            ExprKind::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span { start: 0, end: 1 },
                )),
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span { start: 2, end: 3 },
                )),
            },
            Span { start: 0, end: 3 },
        );

        matrix.record("binary", "1 + 2", Ok(binary_expr), Duration::from_millis(1));

        assert!(matrix.ast_variants.contains("Binary"));
    }

    // Test 19: record_ast_variants Unary
    #[test]
    fn test_ast_variant_unary() {
        let mut matrix = GrammarCoverageMatrix::new();

        let unary_expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span { start: 1, end: 3 },
                )),
            },
            Span { start: 0, end: 3 },
        );

        matrix.record("unary", "-42", Ok(unary_expr), Duration::from_millis(1));

        assert!(matrix.ast_variants.contains("Unary"));
    }

    // Test 20: record_ast_variants If
    #[test]
    fn test_ast_variant_if() {
        let mut matrix = GrammarCoverageMatrix::new();

        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span { start: 0, end: 4 },
                )),
                then_branch: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span { start: 7, end: 8 },
                )),
                else_branch: None,
            },
            Span { start: 0, end: 10 },
        );

        matrix.record("if", "if true { 1 }", Ok(if_expr), Duration::from_millis(1));

        assert!(matrix.ast_variants.contains("If"));
    }

    // Test 21: record_ast_variants Block
    #[test]
    fn test_ast_variant_block() {
        let mut matrix = GrammarCoverageMatrix::new();

        let block_expr = Expr::new(
            ExprKind::Block(vec![Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Span { start: 1, end: 3 },
            )]),
            Span { start: 0, end: 4 },
        );

        matrix.record("block", "{ 42 }", Ok(block_expr), Duration::from_millis(1));

        assert!(matrix.ast_variants.contains("Block"));
    }

    // Test 22: record_ast_variants Let
    #[test]
    fn test_ast_variant_let() {
        let mut matrix = GrammarCoverageMatrix::new();

        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span { start: 8, end: 10 },
                )),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span { start: 14, end: 15 },
                )),
                is_mutable: false,
                else_block: None,
            },
            Span { start: 0, end: 15 },
        );

        matrix.record(
            "let",
            "let x = 42 in x",
            Ok(let_expr),
            Duration::from_millis(1),
        );

        assert!(matrix.ast_variants.contains("Let"));
    }

    // Test 23: record_ast_variants Call
    #[test]
    fn test_ast_variant_call() {
        let mut matrix = GrammarCoverageMatrix::new();

        let call_expr = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("foo".to_string()),
                    Span { start: 0, end: 3 },
                )),
                args: vec![],
            },
            Span { start: 0, end: 5 },
        );

        matrix.record("call", "foo()", Ok(call_expr), Duration::from_millis(1));

        assert!(matrix.ast_variants.contains("Call"));
    }

    // Test 24: record_ast_variants List
    #[test]
    fn test_ast_variant_list() {
        let mut matrix = GrammarCoverageMatrix::new();

        let list_expr = Expr::new(
            ExprKind::List(vec![Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span { start: 1, end: 2 },
            )]),
            Span { start: 0, end: 3 },
        );

        matrix.record("list", "[1]", Ok(list_expr), Duration::from_millis(1));

        assert!(matrix.ast_variants.contains("List"));
    }

    // Test 25: report includes all expected sections
    #[test]
    fn test_report_comprehensive() {
        let mut matrix = GrammarCoverageMatrix::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span { start: 0, end: 1 },
        );

        // Add some successes and failures
        matrix.record("literal", "42", Ok(expr.clone()), Duration::from_millis(10));
        matrix.record("literal", "43", Ok(expr), Duration::from_millis(20));
        matrix.record(
            "other",
            "bad",
            Err(anyhow::anyhow!("error")),
            Duration::from_millis(5),
        );

        let report = matrix.report();

        assert!(report.contains("Grammar Coverage Report"));
        assert!(report.contains("Coverage:"));
        assert!(report.contains("Productions covered:"));
        assert!(report.contains("AST variants seen:"));
        assert!(report.contains("Total attempts:"));
        assert!(report.contains("Success rate:"));
        assert!(report.contains("Slowest productions:"));
    }

    // Test 26: GrammarCoverageMatrix default
    #[test]
    fn test_grammar_coverage_matrix_default() {
        let matrix = GrammarCoverageMatrix::default();
        assert!(matrix.productions.is_empty());
        assert!(matrix.ast_variants.is_empty());
        assert!(matrix.uncovered.is_empty());
    }
}
