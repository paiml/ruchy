//! EXTREME TDD tests for formatter.rs
//!
//! Coverage target: 95% for formatter module
//! These tests cover ExprKind variants not covered by existing tests.

#[cfg(test)]
mod tests {
    use crate::frontend::ast::{Expr, ExprKind, Literal, UnaryOp};
    use crate::frontend::parser::Parser;
    use crate::quality::formatter::Formatter;

    // Helper to parse and format code
    fn format_code(code: &str) -> String {
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("should parse");
        let formatter = Formatter::new();
        formatter.format(&ast).expect("should format")
    }

    // Helper that returns Result for error testing
    fn try_format(code: &str) -> Result<String, String> {
        let mut parser = Parser::new(code);
        match parser.parse() {
            Ok(ast) => {
                let formatter = Formatter::new();
                formatter.format(&ast).map_err(|e| e.to_string())
            }
            Err(e) => Err(format!("Parse error: {e:?}")),
        }
    }

    // Helper to create simple literal
    fn make_lit(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Default::default(),
        )
    }

    // Helper to create identifier
    fn make_ident(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Default::default())
    }

    include!("formatter_tests_part1.rs");
    include!("formatter_tests_part2.rs");
}
