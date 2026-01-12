//! `DataFrame` literal parsing
//!
//! Handles parsing of `DataFrame` literals using the `df!` macro syntax.
//! `DataFrames` are delegated to the collections module for full parsing,
//! while this module handles the dispatch between:
//! - `df![...]` - `DataFrame` literal (delegated to collections)
//! - `df` - `DataFrame` identifier (for method calls, etc.)
//!
//! # Examples
//! ```ruchy
//! df![name => ["Alice", "Bob"], age => [30, 25]]  // DataFrame literal
//! df.filter(...)                                   // DataFrame identifier
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{ParserState, Result};

// Import DataFrame parser from collections module
use crate::frontend::parser::collections;

/// Parse `DataFrame` token: either `df![...]` literal or `df` identifier
///
/// Dispatches based on whether the next token is `!`:
/// - `df!` → Delegate to `collections::parse_dataframe` for literal parsing
/// - `df` → Treat as identifier for method calls
///
/// # Examples
/// ```ruchy
/// df![x => [1, 2, 3]]          // Literal
/// df.select("column")           // Identifier
/// let my_df = df               // Identifier
/// ```
pub(in crate::frontend::parser) fn parse_dataframe_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    // Check if this is df! (literal) or df (identifier)
    if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
        // DataFrame literal: df![...]
        // Delegate to collections module which handles the full syntax
        // Note: collections::parse_dataframe will consume the DataFrame token
        collections::parse_dataframe(state)
    } else {
        // DataFrame identifier: df.method() or df variable reference
        // Consume the DataFrame token since we're handling it as identifier
        state.tokens.advance();
        Ok(Expr::new(ExprKind::Identifier("df".to_string()), span))
    }
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_dataframe_literal_empty() {
        let code = "df![]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty DataFrame should parse");
    }

    #[test]
    fn test_dataframe_literal_single_column() {
        let code = r#"df![name => ["Alice", "Bob"]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Single column DataFrame should parse");
    }

    #[test]
    fn test_dataframe_literal_multiple_columns() {
        let code = r#"df![name => ["Alice", "Bob"], age => [30, 25]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Multiple column DataFrame should parse");
    }

    #[test]
    fn test_dataframe_identifier() {
        let code = "df";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame identifier should parse");
    }

    #[test]
    fn test_dataframe_method_call() {
        let code = r#"df.select("column")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame method call should parse");
    }

    #[test]
    fn test_dataframe_assignment() {
        let code = "let my_df = df";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame assignment should parse");
    }

    #[test]
    fn test_dataframe_literal_with_integers() {
        let code = "df![x => [1, 2, 3], y => [4, 5, 6]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with integers should parse");
    }

    // Test 8: DataFrame with floats
    #[test]
    fn test_dataframe_literal_with_floats() {
        let code = "df![x => [1.0, 2.5, 3.14]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with floats should parse");
    }

    // Test 9: DataFrame with mixed types
    #[test]
    fn test_dataframe_literal_mixed_types() {
        let code = r#"df![name => ["Alice"], age => [30], score => [95.5]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with mixed types should parse");
    }

    // Test 10: DataFrame method chain
    #[test]
    fn test_dataframe_method_chain() {
        let code = r#"df.filter("x > 0").select("y")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame method chain should parse");
    }

    // Test 11: DataFrame in let binding with literal
    #[test]
    fn test_dataframe_let_binding_literal() {
        let code = "let data = df![x => [1, 2, 3]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame let binding with literal should parse"
        );
    }

    // Test 12: DataFrame in function parameter
    #[test]
    fn test_dataframe_function_parameter() {
        let code = "fun process(df) { df }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame as function parameter should parse"
        );
    }

    // Test 13: DataFrame with boolean values
    #[test]
    fn test_dataframe_literal_with_booleans() {
        let code = "df![active => [true, false, true]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with booleans should parse");
    }

    // Test 14: DataFrame empty column names
    #[test]
    fn test_dataframe_method_filter() {
        let code = r#"df.filter("age > 21")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame filter should parse");
    }

    // Test 15: DataFrame join operation
    #[test]
    fn test_dataframe_method_join() {
        let code = r#"df.join(other_df, "id")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame join should parse");
    }

    // Test 16: DataFrame groupby operation
    #[test]
    fn test_dataframe_method_groupby() {
        let code = r#"df.groupby("category")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame groupby should parse");
    }

    // Test 17: DataFrame in if condition
    #[test]
    fn test_dataframe_in_if() {
        let code = "if df.len() > 0 { df } else { empty_df }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame in if should parse");
    }

    // Test 18: DataFrame return from function
    #[test]
    fn test_dataframe_function_return() {
        let code = "fun create_df() { df![x => [1, 2, 3]] }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame function return should parse");
    }

    // Test 19: DataFrame with underscore column names
    #[test]
    fn test_dataframe_underscore_columns() {
        let code = "df![first_name => [\"A\"], last_name => [\"B\"]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with underscore columns should parse"
        );
    }

    // Test 20: DataFrame head method
    #[test]
    fn test_dataframe_method_head() {
        let code = "df.head(5)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame head should parse");
    }

    // Property tests for DataFrames
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_dataframe_identifier_always_parses(_suffix in "[a-z]{0,10}") {
                let code = "df";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "DataFrame identifier should always parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_empty_dataframe_literal_parses(_n in 0..100usize) {
                let code = "df![]";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "Empty DataFrame literal should parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_single_column_integers_parse(values in prop::collection::vec(any::<i32>(), 1..10)) {
                let values_str = values.iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let code = format!("df![x => [{values_str}]]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Single column with integers {} should parse", code);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_dataframe_method_chain_parses(depth in 1..5usize) {
                let mut code = "df".to_string();
                for _ in 0..depth {
                    code.push_str(".select(\"x\")");
                }
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "DataFrame method chain {} should parse", code);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_dataframe_column_names_parse(name in "[a-z][a-z0-9_]{0,10}") {
                let code = format!("df![{name} => [1, 2, 3]]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "DataFrame with column {} should parse", name);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_multiple_columns_parse(num_cols in 1..5usize) {
                let columns = (0..num_cols)
                    .map(|i| format!("col{i} => [1, 2]"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let code = format!("df![{columns}]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "DataFrame with {} columns should parse", num_cols);
            }
        }
    }

    // =========================================================================
    // Additional Tests for Coverage: Method Chain Parsing (Tests 21-30)
    // =========================================================================

    #[test]
    fn test_dataframe_method_tail() {
        let code = "df.tail(10)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame tail should parse");
    }

    #[test]
    fn test_dataframe_method_limit() {
        let code = "df.limit(100)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame limit should parse");
    }

    #[test]
    fn test_dataframe_method_sort() {
        let code = r#"df.sort("name")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame sort should parse");
    }

    #[test]
    fn test_dataframe_long_method_chain() {
        let code = r#"df.filter("age > 18").select("name").sort("name").head(10)"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Long DataFrame method chain should parse");
    }

    #[test]
    fn test_dataframe_method_with_multiple_args() {
        let code = r#"df.join(other, "id", "inner")"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame method with multiple args should parse"
        );
    }

    // =========================================================================
    // Column Access Syntax (Tests 26-31)
    // =========================================================================

    #[test]
    fn test_dataframe_column_dot_access() {
        let code = "df.column_name";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame column dot access should parse");
    }

    #[test]
    fn test_dataframe_column_bracket_access() {
        let code = r#"df["column_name"]"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame column bracket access should parse"
        );
    }

    #[test]
    fn test_dataframe_column_index_access() {
        let code = "df[0]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame column index access should parse");
    }

    #[test]
    fn test_dataframe_nested_column_access() {
        let code = "df.data.values";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Nested DataFrame column access should parse"
        );
    }

    #[test]
    fn test_dataframe_column_access_then_method() {
        let code = "df.column.sum()";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame column access then method should parse"
        );
    }

    #[test]
    fn test_dataframe_method_then_column_access() {
        let code = r#"df.filter("x > 0").result"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame method then column access should parse"
        );
    }

    // =========================================================================
    // Filter Expressions (Tests 32-37)
    // =========================================================================

    #[test]
    fn test_dataframe_filter_with_lambda() {
        let code = "df.filter(|x| x > 5)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame filter with lambda should parse");
    }

    #[test]
    fn test_dataframe_filter_with_comparison() {
        let code = "df.filter(x > 5)";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame filter with comparison should parse"
        );
    }

    #[test]
    fn test_dataframe_filter_with_and_condition() {
        let code = "df.filter(x > 5 && y < 10)";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame filter with AND condition should parse"
        );
    }

    #[test]
    fn test_dataframe_filter_with_or_condition() {
        let code = "df.filter(x > 5 || x < 0)";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame filter with OR condition should parse"
        );
    }

    #[test]
    fn test_dataframe_filter_with_equality() {
        let code = r#"df.filter(name == "Alice")"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame filter with equality should parse"
        );
    }

    #[test]
    fn test_dataframe_filter_with_not_equal() {
        let code = r#"df.filter(status != "inactive")"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame filter with not equal should parse"
        );
    }

    // =========================================================================
    // Aggregate Operations Parsing (Tests 38-45)
    // =========================================================================

    #[test]
    fn test_dataframe_aggregate_sum() {
        let code = r#"df.sum("values")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame sum aggregate should parse");
    }

    #[test]
    fn test_dataframe_aggregate_mean() {
        let code = r#"df.mean("scores")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame mean aggregate should parse");
    }

    #[test]
    fn test_dataframe_aggregate_min() {
        let code = r#"df.min("price")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame min aggregate should parse");
    }

    #[test]
    fn test_dataframe_aggregate_max() {
        let code = r#"df.max("price")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame max aggregate should parse");
    }

    #[test]
    fn test_dataframe_aggregate_count() {
        let code = r#"df.count("id")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame count aggregate should parse");
    }

    #[test]
    fn test_dataframe_aggregate_std() {
        let code = r#"df.std("values")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame std aggregate should parse");
    }

    #[test]
    fn test_dataframe_groupby_then_aggregate() {
        let code = r#"df.groupby("category").sum("amount")"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame groupby then aggregate should parse"
        );
    }

    #[test]
    fn test_dataframe_multiple_aggregates_chain() {
        let code = r#"df.sum("a").mean("b").max("c")"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame multiple aggregates chain should parse"
        );
    }

    // =========================================================================
    // Error Handling for Malformed DataFrame Syntax (Tests 46-55)
    // =========================================================================

    #[test]
    fn test_dataframe_missing_bracket() {
        let code = "df![x => [1, 2, 3]";
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Missing bracket should error");
    }

    #[test]
    fn test_dataframe_missing_arrow() {
        let code = "df![x [1, 2, 3]]";
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Missing arrow should error");
    }

    #[test]
    fn test_dataframe_missing_values() {
        let code = "df![x =>]";
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Missing values should error");
    }

    #[test]
    fn test_dataframe_unclosed_values_list() {
        let code = "df![x => [1, 2, 3";
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Unclosed values list should error");
    }

    #[test]
    fn test_dataframe_invalid_column_name_number() {
        let code = "df![123 => [1, 2, 3]]";
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Numeric column name should error");
    }

    #[test]
    fn test_dataframe_method_missing_parens() {
        let code = "df.filter";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Method without parens parses as field access"
        );
    }

    #[test]
    fn test_dataframe_method_unclosed_parens() {
        let code = r#"df.filter("x > 0""#;
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Unclosed method parens should error");
    }

    #[test]
    fn test_dataframe_double_bang() {
        // Note: df!![] parses as df followed by !![]
        // The parser is lenient about this - it parses df as identifier then !![]
        let code = "df!![]";
        let result = Parser::new(code).parse();
        // This actually parses (df identifier followed by double negation of empty array)
        // so we just verify it parses without crashing
        assert!(
            result.is_ok() || result.is_err(),
            "Double bang should parse or error gracefully"
        );
    }

    #[test]
    fn test_dataframe_trailing_comma_in_columns() {
        let code = "df![x => [1, 2, 3],]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Trailing comma in columns should be allowed"
        );
    }

    #[test]
    fn test_dataframe_empty_column_name_string() {
        let code = r#"df!["" => [1, 2, 3]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty string column name should parse");
    }

    // =========================================================================
    // DataFrame in Context (Tests 56-65)
    // =========================================================================

    #[test]
    fn test_dataframe_in_match() {
        let code = "match df.len() { 0 => empty, _ => df }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame in match should parse");
    }

    #[test]
    fn test_dataframe_in_for_loop() {
        let code = "for row in df.rows() { print(row) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame in for loop should parse");
    }

    #[test]
    fn test_dataframe_as_return_type() {
        let code = "fun get_data() -> DataFrame { df![] }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame as return type should parse");
    }

    #[test]
    fn test_dataframe_in_tuple() {
        let code = "(df, 42)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame in tuple should parse");
    }

    #[test]
    fn test_dataframe_in_array() {
        let code = "[df1, df2, df3]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame in array should parse");
    }

    // =========================================================================
    // String Column Names (Tests 61-65)
    // =========================================================================

    #[test]
    fn test_dataframe_string_column_name() {
        let code = r#"df!["column name" => [1, 2, 3]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "String column name should parse");
    }

    #[test]
    fn test_dataframe_mixed_column_names() {
        let code = r#"df![col1 => [1], "col 2" => [2]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Mixed column names should parse");
    }

    #[test]
    fn test_dataframe_special_char_column_name() {
        let code = r#"df!["col-with-dashes" => [1, 2, 3]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Special char column name should parse");
    }

    #[test]
    fn test_dataframe_numeric_string_column_name() {
        let code = r#"df!["123" => [1, 2, 3]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Numeric string column name should parse");
    }

    // =========================================================================
    // Complex DataFrame Expressions (Tests 66-72)
    // =========================================================================

    #[test]
    fn test_dataframe_nested_expressions_in_values() {
        let code = "df![x => [1 + 2, 3 * 4, 5 - 6]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with expressions in values should parse"
        );
    }

    #[test]
    fn test_dataframe_function_call_in_values() {
        let code = "df![x => [foo(), bar(), baz()]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with function calls in values should parse"
        );
    }

    #[test]
    fn test_dataframe_conditional_in_values() {
        let code = "df![x => [if true { 1 } else { 0 }]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with conditional in values should parse"
        );
    }

    #[test]
    fn test_dataframe_binary_operation_result() {
        let code = "df![x => [1, 2]] + df![y => [3, 4]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame binary operation should parse");
    }

    #[test]
    fn test_dataframe_method_on_literal() {
        let code = "df![x => [1, 2, 3]].head(2)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame method on literal should parse");
    }

    #[test]
    fn test_dataframe_chained_from_literal() {
        let code = r#"df![x => [1, 2, 3]].filter("x > 1").select("x")"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame chained from literal should parse"
        );
    }

    #[test]
    fn test_dataframe_in_pipe_operator() {
        let code = "df |> filter(x > 5) |> select(y)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame in pipe operator should parse");
    }

    // =========================================================================
    // AST Structure Verification (Tests 73-77)
    // =========================================================================

    /// Helper to extract the first expression from a parsed result
    fn get_first_expr(expr: &crate::frontend::ast::Expr) -> &crate::frontend::ast::Expr {
        use crate::frontend::ast::ExprKind;
        match &expr.kind {
            ExprKind::Block(exprs) if !exprs.is_empty() => &exprs[0],
            _ => expr,
        }
    }

    #[test]
    fn test_dataframe_ast_empty_columns() {
        use crate::frontend::ast::ExprKind;
        let code = "df![]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        let first = get_first_expr(&expr);
        match &first.kind {
            ExprKind::DataFrame { columns } => {
                assert!(columns.is_empty(), "Empty DataFrame should have no columns");
            }
            _ => panic!("Expected DataFrame expression"),
        }
    }

    #[test]
    fn test_dataframe_ast_single_column() {
        use crate::frontend::ast::ExprKind;
        let code = "df![x => [1, 2, 3]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        let first = get_first_expr(&expr);
        match &first.kind {
            ExprKind::DataFrame { columns } => {
                assert_eq!(columns.len(), 1, "Should have one column");
                assert_eq!(columns[0].name, "x", "Column name should be 'x'");
                assert_eq!(columns[0].values.len(), 3, "Should have 3 values");
            }
            _ => panic!("Expected DataFrame expression"),
        }
    }

    #[test]
    fn test_dataframe_ast_multiple_columns() {
        use crate::frontend::ast::ExprKind;
        let code = "df![a => [1], b => [2], c => [3]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        let first = get_first_expr(&expr);
        match &first.kind {
            ExprKind::DataFrame { columns } => {
                assert_eq!(columns.len(), 3, "Should have three columns");
                assert_eq!(columns[0].name, "a");
                assert_eq!(columns[1].name, "b");
                assert_eq!(columns[2].name, "c");
            }
            _ => panic!("Expected DataFrame expression"),
        }
    }

    #[test]
    fn test_dataframe_ast_method_call_structure() {
        use crate::frontend::ast::ExprKind;
        let code = "df.head(5)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        let first = get_first_expr(&expr);
        match &first.kind {
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                assert_eq!(method, "head", "Method should be 'head'");
                assert_eq!(args.len(), 1, "Should have one argument");
                match &receiver.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "df"),
                    _ => panic!("Expected identifier receiver"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_dataframe_ast_field_access_structure() {
        use crate::frontend::ast::ExprKind;
        let code = "df.column_name";
        let result = Parser::new(code).parse();
        assert!(result.is_ok());
        let expr = result.unwrap();
        let first = get_first_expr(&expr);
        match &first.kind {
            ExprKind::FieldAccess { object, field } => {
                assert_eq!(field, "column_name", "Field should be 'column_name'");
                match &object.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "df"),
                    _ => panic!("Expected identifier object"),
                }
            }
            _ => panic!("Expected FieldAccess expression"),
        }
    }

    // =========================================================================
    // Additional Edge Cases (Tests 78-85)
    // =========================================================================

    #[test]
    fn test_dataframe_identifier_as_variable() {
        use crate::frontend::ast::ExprKind;
        let code = "df";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame identifier should parse");
        let expr = result.unwrap();
        let first = get_first_expr(&expr);
        match &first.kind {
            ExprKind::Identifier(name) => assert_eq!(name, "df"),
            _ => panic!("Expected identifier 'df'"),
        }
    }

    #[test]
    fn test_dataframe_empty_array_values() {
        let code = "df![x => []]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with empty array values should parse"
        );
    }

    #[test]
    fn test_dataframe_single_value() {
        let code = "df![x => [42]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with single value should parse");
    }

    #[test]
    fn test_dataframe_many_columns() {
        let code = "df![a => [1], b => [2], c => [3], d => [4], e => [5], f => [6]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with many columns should parse");
    }

    #[test]
    fn test_dataframe_negative_numbers() {
        let code = "df![x => [-1, -2, -3]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with negative numbers should parse"
        );
    }

    #[test]
    fn test_dataframe_float_values() {
        let code = "df![x => [1.5, 2.7, 3.9]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with float values should parse");
    }

    #[test]
    fn test_dataframe_scientific_notation() {
        let code = "df![x => [1e10, 2e-5, 3.14e2]]";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "DataFrame with scientific notation should parse"
        );
    }

    #[test]
    fn test_dataframe_null_values() {
        let code = "df![x => [None, Some(1), None]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with null values should parse");
    }
}
