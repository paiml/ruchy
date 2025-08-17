//! Core parser implementation with main entry points

use super::{ParserState, *};

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
    /// let ast = parser.parse().unwrap();
    /// ```
    ///
    /// ```
    /// use ruchy::Parser;
    ///
    /// let mut parser = Parser::new("let x = 10 in x + 1");
    /// let ast = parser.parse().unwrap();
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
    /// let expr = parser.parse_expr().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the input cannot be parsed as a valid expression
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
    /// let expr = parser.parse_expr_with_precedence(0).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the expression cannot be parsed or contains syntax errors
    pub fn parse_expr_with_precedence(&mut self, min_prec: i32) -> Result<Expr> {
        super::parse_expr_with_precedence_recursive(&mut self.state, min_prec)
    }
}