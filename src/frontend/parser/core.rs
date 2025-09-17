//! Core parser implementation with main entry points
use super::{ParserState, Result, Expr, bail, Token, ExprKind, Span, ErrorNode, utils};
pub struct Parser<'a> {
    state: ParserState<'a>,
}
impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            state: ParserState::new(input),
        }
    }
    /// Get all errors encountered during parsing
    #[must_use]
    pub fn get_errors(&self) -> &[ErrorNode] {
        self.state.get_errors()
    }
    /// Parse the input into an expression or block of expressions
    ///
    /// Parse a complete program or expression
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::Parser;
    ///
    /// let mut parser = Parser::new("42");
    /// let ast = parser.parse().expect("Failed to parse");
    /// ```
    ///
    /// ```
    /// use ruchy::Parser;
    ///
    /// let mut parser = Parser::new("let x = 10 in x + 1");
    /// let ast = parser.parse().expect("Failed to parse");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input is empty
    /// - Syntax errors are encountered
    /// - Unexpected tokens are found
    ///
    /// # Panics
    ///
    /// Should not panic in normal operation. Uses `expect` on verified conditions.
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn parse(&mut self) -> Result<Expr> {
        // Parse multiple top-level expressions/statements as a block
        let mut exprs = Vec::new();
        while self.state.tokens.peek().is_some() {
            let attributes = utils::parse_attributes(&mut self.state)?;
            let mut expr = super::parse_expr_recursive(&mut self.state)?;
            expr.attributes = attributes;
            exprs.push(expr);
            // Skip optional semicolons
            if let Some((Token::Semicolon, _)) = self.state.tokens.peek() {
                self.state.tokens.advance();
            }
        }
        if exprs.is_empty() {
            bail!("Empty program");
        } else if exprs.len() == 1 {
            Ok(exprs.into_iter().next().expect("checked: non-empty vec"))
        } else {
            Ok(Expr {
                kind: ExprKind::Block(exprs),
                span: Span { start: 0, end: 0 }, // Simplified span for now
                attributes: Vec::new(),
            })
        }
    }
    /// Parse a single expression
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::Parser;
    ///
    /// let mut parser = Parser::new("1 + 2 * 3");
    /// let expr = parser.parse_expr().expect("Failed to parse expression");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the input cannot be parsed as a valid expression
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn parse_expr(&mut self) -> Result<Expr> {
        super::parse_expr_recursive(&mut self.state)
    }
    /// Parse an expression with operator precedence
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::Parser;
    ///
    /// let mut parser = Parser::new("1 + 2 * 3");
    /// // Parse with minimum precedence 0 to get all operators
    /// let expr = parser.parse_expr_with_precedence(0).expect("Failed to parse expression");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be parsed or contains syntax errors
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn parse_expr_with_precedence(&mut self, min_prec: i32) -> Result<Expr> {
        super::parse_expr_with_precedence_recursive(&mut self.state, min_prec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new("42");
        assert!(parser.get_errors().is_empty());
    }

    #[test]
    fn test_parser_new_empty_input() {
        let parser = Parser::new("");
        assert!(parser.get_errors().is_empty());
    }

    #[test]
    fn test_parse_simple_integer() {
        let mut parser = Parser::new("42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_float() {
        let mut parser = Parser::new("3.14");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_string() {
        let mut parser = Parser::new("\"hello\"");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_boolean_true() {
        let mut parser = Parser::new("true");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_boolean_false() {
        let mut parser = Parser::new("false");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_simple_variable() {
        let mut parser = Parser::new("x");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_addition() {
        let mut parser = Parser::new("1 + 2");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_subtraction() {
        let mut parser = Parser::new("5 - 3");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_multiplication() {
        let mut parser = Parser::new("2 * 3");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_binary_division() {
        let mut parser = Parser::new("10 / 2");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_complex_arithmetic() {
        let mut parser = Parser::new("1 + 2 * 3");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let mut parser = Parser::new("(1 + 2) * 3");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_expression() {
        let mut parser = Parser::new("let x = 42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_in_expression() {
        let mut parser = Parser::new("let x = 10 in x + 1");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function_call() {
        let mut parser = Parser::new("f(1, 2, 3)");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_program_error() {
        let mut parser = Parser::new("");
        let result = parser.parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty program"));
    }

    #[test]
    fn test_parse_multiple_expressions_as_block() {
        let mut parser = Parser::new("let x = 1; let y = 2; x + y");
        let result = parser.parse();
        assert!(result.is_ok());

        if let Ok(expr) = result {
            match expr.kind {
                ExprKind::Block(exprs) => {
                    assert_eq!(exprs.len(), 3);
                },
                _ => {
                    // Single expression is also valid
                }
            }
        }
    }

    #[test]
    fn test_parse_expr_single() {
        let mut parser = Parser::new("42 + 24");
        let result = parser.parse_expr();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expr_complex() {
        let mut parser = Parser::new("(x + y) * (a - b)");
        let result = parser.parse_expr();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expr_with_precedence_low() {
        let mut parser = Parser::new("1 + 2 * 3");
        let result = parser.parse_expr_with_precedence(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_expr_with_precedence_high() {
        let mut parser = Parser::new("1 + 2 * 3");
        let result = parser.parse_expr_with_precedence(10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_comparison_operators() {
        let expressions = vec![
            "1 == 2",
            "1 != 2",
            "1 < 2",
            "1 <= 2",
            "1 > 2",
            "1 >= 2",
        ];

        for expr in expressions {
            let mut parser = Parser::new(expr);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {}", expr);
        }
    }

    #[test]
    fn test_parse_logical_operators() {
        let expressions = vec![
            "true && false",
            "true || false",
            "!true",
        ];

        for expr in expressions {
            let mut parser = Parser::new(expr);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {}", expr);
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let mut parser = Parser::new("[1, 2, 3]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_array() {
        let mut parser = Parser::new("[]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_object_literal() {
        let mut parser = Parser::new("{x: 1, y: 2}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_empty_object() {
        let mut parser = Parser::new("{}");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_expression() {
        let mut parser = Parser::new("if true { 1 } else { 2 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_without_else() {
        let mut parser = Parser::new("if condition { action }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_match_expression() {
        let mut parser = Parser::new("match x { 1 => \"one\", _ => \"other\" }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda_expression() {
        let mut parser = Parser::new("\\x -> x + 1");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda_multiple_params() {
        let mut parser = Parser::new("\\x, y -> x + y");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_method_call() {
        let mut parser = Parser::new("obj.method(arg)");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_chained_method_calls() {
        let mut parser = Parser::new("obj.method1().method2().method3()");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_index_access() {
        let mut parser = Parser::new("arr[0]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_index_access() {
        let mut parser = Parser::new("matrix[i][j]");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_assignment() {
        let mut parser = Parser::new("x = 42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_compound_assignment() {
        let expressions = vec![
            "x += 1",
            "x -= 1",
            "x *= 2",
            "x /= 2",
        ];

        for expr in expressions {
            let mut parser = Parser::new(expr);
            let result = parser.parse();
            assert!(result.is_ok(), "Failed to parse: {}", expr);
        }
    }

    #[test]
    fn test_parse_with_semicolons() {
        let mut parser = Parser::new("let x = 1; let y = 2;");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let mut parser = Parser::new("((1 + 2) * (3 + 4))");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_errors_empty_on_valid_parse() {
        let mut parser = Parser::new("42");
        let _ = parser.parse();
        assert!(parser.get_errors().is_empty());
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let mut parser = Parser::new("  1   +   2  ");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_comments() {
        let mut parser = Parser::new("// This is a comment\n42");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_multiline_expression() {
        let mut parser = Parser::new("1 +\n2 *\n3");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
