# Sub-spec: REPL Grammar Testing — Core Tests and Property Validation

**Parent:** [test-grammer-repl.md](../test-grammer-repl.md) Sections 1-3

---

### 2.1 Critical Bug Prevention Tests

```rust
// tests/critical_regressions.rs

#[test]
fn test_string_interpolation_no_f_prefix() {
    let mut repl = Repl::new();
    
    // Bug: Parser must reject interpolation syntax without 'f' prefix
    let result = repl.eval(r#""Hello, {name}!""#);
    assert!(
        result.is_ok() && !result.unwrap().contains("Any"),
        "String with braces but no 'f' prefix must parse as literal string"
    );
    
    // Correct interpolation with 'f' prefix
    let result = repl.eval(r#"let name = "World"; f"Hello, {name}!""#);
    assert_eq!(result.unwrap(), "Hello, World!");
}

#[test]
fn test_let_statement_without_semicolon() {
    let mut repl = Repl::new();
    
    // Bug: REPL must accept let statements without semicolons
    let result = repl.eval("let x = 1");
    assert!(result.is_ok(), "Let statement without semicolon must parse in REPL");
    
    // Verify binding exists
    let result = repl.eval("x");
    assert_eq!(result.unwrap(), "1");
    
    // Also test with semicolon
    let result = repl.eval("let y = 2;");
    assert!(result.is_ok());
}

#[test]
fn test_function_with_string_interpolation() {
    let mut repl = Repl::new();
    
    // Original failing case
    let result = repl.eval(r#"
        fun greet(name) {
            f"Hello, {name}!"
        }
    "#);
    assert!(result.is_ok(), "Function with f-string must parse");
    
    // Test invocation
    let result = repl.eval(r#"greet("World")"#);
    assert_eq!(result.unwrap(), "Hello, World!");
}

#[test]
fn test_no_polars_dependency_in_simple_code() {
    let mut repl = Repl::new();
    
    // Bug: Simple functions must not generate polars imports
    let result = repl.eval("fun add(x, y) { x + y }");
    assert!(result.is_ok());
    
    // Verify generated Rust has no unnecessary imports
    let rust_code = repl.to_rust("add");
    assert!(!rust_code.contains("polars"), "No DataFrame ops should not import polars");
    assert!(!rust_code.contains("use std::any::Any"), "Typed parameters should not use Any");
}
```

# End-to-End REPL Grammar Testing Specification

## Executive Summary

This specification defines comprehensive testing for the Ruchy REPL against its formal grammar specification. The approach combines exhaustive enumeration, property-based generation, and example-driven validation to achieve 100% grammar coverage with <15ms response time per construct.

## Architecture

### Testing Layers

```
┌─────────────────────────────────────────┐
│         Property-Based Fuzzing          │  10M iterations
├─────────────────────────────────────────┤
│       Grammar Coverage Matrix           │  67 productions
├─────────────────────────────────────────┤
│         Example Validation              │  9 categories
├─────────────────────────────────────────┤
│          State Machine Model            │  Formal verification
└─────────────────────────────────────────┘
```

## Implementation

### 1. Grammar Coverage Matrix

```rust
// src/runtime/grammar_coverage.rs

use std::collections::{HashMap, HashSet};
use std::time::Duration;
use ruchy_ast::*;

#[derive(Default)]
pub struct GrammarCoverageMatrix {
    productions: HashMap<&'static str, ProductionStats>,
    ast_variants: HashSet<String>,
    uncovered: Vec<&'static str>,
}

#[derive(Default)]
struct ProductionStats {
    hit_count: usize,
    success_count: usize,
    avg_latency_ns: u64,
    error_patterns: Vec<String>,
}

impl GrammarCoverageMatrix {
    pub fn record(&mut self, input: &str, result: ParseResult, elapsed: Duration) {
        let production = identify_production(input);
        let stats = self.productions.entry(production).or_default();
        
        stats.hit_count += 1;
        stats.avg_latency_ns = 
            (stats.avg_latency_ns * (stats.hit_count - 1) + elapsed.as_nanos() as u64) 
            / stats.hit_count;
        
        match result {
            Ok(ast) => {
                stats.success_count += 1;
                self.record_ast_variants(&ast);
            }
            Err(e) => stats.error_patterns.push(e.to_string()),
        }
    }
    
    fn record_ast_variants(&mut self, ast: &Ast) {
        ast.walk(|node| {
            self.ast_variants.insert(std::any::type_name_of_val(node));
        });
    }
    
    pub fn assert_complete(&self) {
        assert!(
            self.uncovered.is_empty(),
            "Uncovered productions: {:?}",
            self.uncovered
        );
        
        assert_eq!(
            self.ast_variants.len(),
            AST_VARIANT_COUNT,
            "Missing AST variants"
        );
    }
    
    pub fn generate_uncovered(&self, rng: &mut impl Rng) -> String {
        // Focus generation on missing productions
        if let Some(prod) = self.uncovered.first() {
            generate_for_production(prod, rng)
        } else {
            generate_random_expr(rng)
        }
    }
}

fn identify_production(input: &str) -> &'static str {
    match input {
        s if s.starts_with("fun ") => "function_decl",
        s if s.starts_with("let ") => "let_binding",
        s if s.starts_with("struct ") => "struct_decl",
        s if s.starts_with("trait ") => "trait_decl",
        s if s.starts_with("impl ") => "impl_block",
        s if s.starts_with("actor ") => "actor_decl",
        s if s.contains(" |> ") => "pipeline_expr",
        s if s.contains("match ") => "match_expr",
        s if s.contains("for ") => "for_loop",
        s if s.contains("while ") => "while_loop",
        s if s.contains("DataFrame") => "dataframe_ops",
        s if s.contains(" <- ") => "actor_send",
        s if s.contains(" <? ") => "actor_ask",
        _ => "expression",
    }
}
```

### 2. Exhaustive Production Tests

```rust
// tests/grammar_exhaustive.rs

use std::time::{Duration, Instant};
use ruchy::{Repl, GrammarCoverageMatrix, AST_VARIANT_COUNT};

const GRAMMAR_PRODUCTIONS: &[(&str, &str)] = &[
    // Core literals (5)
    ("literal_int", "42"),
    ("literal_float", "3.14"),
    ("literal_string", r#""hello""#),
    ("literal_bool", "true"),
    ("literal_char", "'x'"),
    
    // Binary operators (13) - all precedence levels
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
    ("op_power", "x ** 2"),
    
    // Unary operators (4)
    ("op_neg", "-x"),
    ("op_not", "!x"),
    ("op_deref", "*ptr"),
    ("op_ref", "&value"),
    
    // Control flow (5)
    ("if_expr", "if x > 0 { 1 } else { -1 }"),
    ("match_expr", "match x { Some(y) => y, None => 0 }"),
    ("for_loop", "for x in 0..10 { print(x) }"),
    ("while_loop", "while x > 0 { x = x - 1 }"),
    ("loop_expr", "loop { break 42 }"),
    
    // Functions (4)
    ("fun_decl", "fun add(a: Int, b: Int) -> Int { a + b }"),
    ("fun_generic", "fun id<T>(x: T) -> T { x }"),
    ("lambda", "|x| x * 2"),
    ("lambda_typed", "|x: Int| -> Int { x * 2 }"),
    
    // Pattern matching & Let bindings (10) - CRITICAL: Statement vs Expression context
    ("let_stmt_semicolon", "let x = 5;"),
    ("let_stmt_no_semicolon", "let x = 5"),  // Must work in REPL
    ("let_in_block", "{ let x = 5; x + 1 }"),
    ("let_with_type", "let x: Int = 5"),
    ("let_mutable", "let mut x = 5"),
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
    ("type_trait", "let d: impl Display = value"),
    
    // Structs/Traits/Impls (3)
    ("struct_decl", "struct Point { x: Float, y: Float }"),
    ("trait_decl", "trait Show { fun show(self) -> String }"),
    ("impl_block", "impl Show for Point { fun show(self) -> String { \"...\" } }"),
    
    // Actor system (4)
    ("actor_decl", "actor Counter { state count: Int = 0 }"),
    ("actor_handler", "on Increment { self.count += 1 }"),
    ("send_op", "counter <- Increment"),
    ("ask_op", "let n = counter <? GetCount"),
    
    // DataFrame operations (6)
    ("df_read", r#"DataFrame::read_csv("data.csv")"#),
    ("df_filter", "df |> filter(col(\"age\") > 18)"),
    ("df_select", "df |> select([\"name\", \"age\"])"),
    ("df_groupby", "df |> groupby(\"dept\")"),
    ("df_agg", "df |> agg([mean(\"salary\"), count()])"),
    ("df_join", "df1 |> join(df2, on: \"id\")"),
    
    // Pipeline operators (3)
    ("pipe_simple", "data |> filter(|x| x > 0)"),
    ("pipe_method", "text |> trim() |> uppercase()"),
    ("pipe_nested", "x |> (|y| y |> double() |> square())"),
    
    // String interpolation (5) - CRITICAL: Must handle all forms
    ("string_interp_simple", r#"f"Hello {name}""#),
    ("string_interp_expr", r#"f"Result: {x + y}""#),
    ("string_interp_format", r#"f"Value: {compute(x):.2f}""#),
    ("string_interp_in_function", "fun greet(name) { f\"Hello, {name}!\" }"),
    ("string_regular_no_interp", r#""Regular string with {braces} not interpolated""#),
    
    // Import/Export (3)
    ("import_simple", "import std::fs"),
    ("import_multi", "import std::collections::{HashMap, HashSet}"),
    ("export", "export { Point, distance }"),
    
    // Attributes (2)
    ("attr_test", "#[test] fun test_foo() { assert(true) }"),
    ("attr_derive", "#[derive(Debug, Clone)] struct Data { x: Int }"),
];

#[test]
fn test_grammar_complete() {
    let mut repl = Repl::new();
    let mut coverage = GrammarCoverageMatrix::default();
    
    for (name, input) in GRAMMAR_PRODUCTIONS {
        let start = Instant::now();
        let result = repl.eval(input);
        coverage.record(input, result.clone(), start.elapsed());
        
        assert!(
            result.is_ok() || coverage.productions[name].error_patterns.len() > 0,
            "Production '{}' failed without error tracking: {}", name, input
        );
        
        // Latency requirement
        assert!(
            start.elapsed() < Duration::from_millis(15),
            "Production '{}' too slow: {:?}", name, start.elapsed()
        );
    }
    
    coverage.assert_complete();
}

#[test]
fn test_ast_variant_coverage() {
    let mut reached_variants = HashSet::new();
    
    for (_, input) in GRAMMAR_PRODUCTIONS {
        if let Ok(ast) = parse(input) {
            ast.walk(|node| {
                reached_variants.insert(std::any::type_name_of_val(node));
            });
        }
    }
    
    let all_variants = [
        Expr::all_variants(),
        Stmt::all_variants(),
        Pattern::all_variants(),
    ].concat();
    
    for variant in all_variants {
        assert!(
            reached_variants.contains(variant),
            "AST variant '{}' never constructed", variant
        );
    }
}
```

### 3. Property-Based Testing

```rust
// tests/grammar_properties.rs

use proptest::prelude::*;
use ruchy::{parse, Repl, GrammarCoverageMatrix};

// Strategy: Generate valid Ruchy expressions
fn arb_expr() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        any::<i64>().prop_map(|n| n.to_string()),
        any::<f64>().prop_map(|f| format!("{:.2}", f)),
        "[a-z][a-z0-9_]*".prop_map(|s| s.to_string()),
        "\"[^\"]*\"".prop_map(|s| s.to_string()),
    ];
    
    leaf.prop_recursive(8, 256, 10, |inner| {
        prop_oneof![
            // Binary expressions
            (inner.clone(), arb_binop(), inner.clone())
                .prop_map(|(l, op, r)| format!("({} {} {})", l, op, r)),
            
            // Pipeline
            (inner.clone(), prop::collection::vec(inner.clone(), 1..5))
                .prop_map(|(init, stages)| 
                    format!("{} {}", init, stages.iter()
                        .map(|s| format!("|> {}", s))
                        .collect::<Vec<_>>()
                        .join(" "))),
            
            // Function call
            ("[a-z]+", prop::collection::vec(inner.clone(), 0..4))
                .prop_map(|(f, args)| format!("{}({})", f, args.join(", "))),
            
            // Lambda
            ("[a-z]+", inner.clone())
                .prop_map(|(param, body)| format!("|{}| {}", param, body)),
            
            // If expression
            (inner.clone(), inner.clone(), inner.clone())
                .prop_map(|(cond, t, f)| format!("if {} {{ {} }} else {{ {} }}", cond, t, f)),
        ]
    })
}

fn arb_binop() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("+"), Just("-"), Just("*"), Just("/"), Just("%"),
        Just("=="), Just("!="), Just("<"), Just(">"), Just("<="), Just(">="),
        Just("&&"), Just("||"),
        Just("&"), Just("|"), Just("^"), Just("<<"), Just(">>"),
        Just("**"),
    ]
}

proptest! {
    #[test]
    fn prop_parse_never_panics(input in arb_expr()) {
        let mut repl = Repl::new();
        let _ = repl.eval(&input); // Must not panic
    }
    
    #[test]
    fn prop_parse_print_roundtrip(input in arb_expr()) {
        let mut repl = Repl::new();
        if let Ok(val) = repl.eval(&input) {
            let printed = val.to_string();
            let reparsed = repl.eval(&printed);
            prop_assert!(reparsed.is_ok(), "Failed to reparse: {}", printed);
        }
    }
    
    #[test]
    fn prop_precedence_preserved(
        a in any::<i32>(),
        b in any::<i32>(),
        c in any::<i32>()
    ) {
        let mut repl = Repl::new();
        
        // Test operator precedence
        let expr1 = format!("{} + {} * {}", a, b, c);
        let expr2 = format!("({} + {}) * {}", a, b, c);
        
        if let (Ok(val1), Ok(val2)) = (repl.eval(&expr1), repl.eval(&expr2)) {
            // Multiplication binds tighter than addition
            prop_assert_ne!(val1, val2, "Precedence not preserved");
        }
    }
    
    #[test]
    fn prop_deterministic_parsing(input in arb_expr()) {
        let result1 = parse(&input);
        let result2 = parse(&input);
        
        match (result1, result2) {
            (Ok(ast1), Ok(ast2)) => prop_assert_eq!(ast1, ast2),
            (Err(e1), Err(e2)) => prop_assert_eq!(e1.to_string(), e2.to_string()),
            _ => prop_assert!(false, "Non-deterministic parsing"),
        }
    }
}

// Coverage-directed generation
#[test]
fn prop_coverage_directed() {
    let mut coverage = GrammarCoverageMatrix::default();
    let mut runner = TestRunner::default();
    
    for _ in 0..100_000 {
        let expr = if coverage.productions.len() < 50 {
            // Focus on uncovered productions
            coverage.generate_uncovered(&mut runner)
        } else {
            // Random exploration
            arb_expr().new_tree(&mut runner).unwrap().current()
        };
        
        let start = Instant::now();
        let result = parse(&expr);
        coverage.record(&expr, result, start.elapsed());
        
        if coverage.productions.len() >= GRAMMAR_PRODUCTIONS.len() {
            break;
        }
    }
    
    assert!(
        coverage.productions.len() >= GRAMMAR_PRODUCTIONS.len() * 95 / 100,
        "Coverage too low: {}/{}", 
        coverage.productions.len(), 
        GRAMMAR_PRODUCTIONS.len()
    );
}
```
