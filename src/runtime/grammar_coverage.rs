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
        input: &str,
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
                let error_msg = format!("{input}: {e}");
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
            ExprKind::StructLiteral { .. } => "StructLiteral",
            ExprKind::ObjectLiteral { .. } => "ObjectLiteral",
            ExprKind::FieldAccess { .. } => "FieldAccess",
            ExprKind::Trait { .. } => "Trait",
            ExprKind::Impl { .. } => "Impl",
            ExprKind::Try { .. } => "Try",
            ExprKind::TryCatch { .. } => "TryCatch",
            ExprKind::Await { .. } => "Await",
            ExprKind::List(_) => "List",
            ExprKind::ListComprehension { .. } => "ListComprehension",
            ExprKind::StringInterpolation { .. } => "StringInterpolation",
            ExprKind::QualifiedName { .. } => "QualifiedName",
            ExprKind::Send { .. } => "Send",
            ExprKind::Ask { .. } => "Ask",
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

    /// Generate a coverage report
    pub fn report(&self) -> String {
        use std::fmt::Write;

        let mut report = String::new();
        report.push_str("Grammar Coverage Report\n");
        report.push_str("=======================\n\n");

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
    ("literal_float", "3.14"),
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
    ("df_filter", "df |> filter(col(\"age\") > 18)"),
    ("df_select", "df |> select([\"name\", \"age\"])"),
    ("df_groupby", "df |> groupby(\"dept\")"),
    ("df_agg", "df |> agg([mean(\"salary\"), count()])"),
    ("df_join", "df1 |> join(df2, on: \"id\")"),
    // Pipeline operators (3)
    ("pipe_simple", "data |> filter(|x| x > 0)"),
    ("pipe_method", "text |> trim() |> uppercase()"),
    ("pipe_nested", "x |> (|y| y |> double() |> square())"),
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
];
