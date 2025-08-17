//! Lexical analysis and tokenization

use crate::frontend::ast::Span;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    String(String),

    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Bool(bool),

    // Keywords
    #[token("fun")]
    Fun,
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("while")]
    While,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("return")]
    Return,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("struct")]
    Struct,
    #[token("impl")]
    Impl,
    #[token("trait")]
    Trait,
    #[token("actor")]
    Actor,
    #[token("receive")]
    Receive,
    #[token("send")]
    Send,
    #[token("ask")]
    Ask,
    #[token("type")]
    Type,
    #[token("const")]
    Const,
    #[token("static")]
    Static,
    #[token("mut")]
    Mut,
    #[token("pub")]
    Pub,
    #[token("import")]
    Import,
    #[token("use")]
    Use,
    #[token("as")]
    As,
    #[token("df")]
    DataFrame,

    // Identifiers (lower priority than keywords)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Identifier(String),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("**")]
    Power,

    #[token("==")]
    EqualEqual,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("!")]
    Bang,

    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,
    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,

    #[token("=")]
    Equal,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    MinusEqual,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,

    #[token("|>")]
    Pipeline,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("..")]
    DotDot,
    #[token("..=")]
    DotDotEqual,
    #[token("?")]
    Question,
    #[token("?.")]
    SafeNav,

    // Delimiters
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,

    // Punctuation
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token(";")]
    Semicolon,
    #[token("_", priority = 2)]
    Underscore,

    // Attribute support
    #[token("#")]
    Hash,
}

impl Token {
    #[must_use]
    pub fn is_binary_op(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::Power
                | Token::EqualEqual
                | Token::NotEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
                | Token::AndAnd
                | Token::OrOr
                | Token::Ampersand
                | Token::Pipe
                | Token::Caret
                | Token::LeftShift
                | Token::RightShift
        )
    }

    #[must_use]
    pub fn is_unary_op(&self) -> bool {
        matches!(self, Token::Bang | Token::Minus | Token::Tilde)
    }
}

pub struct TokenStream<'a> {
    lexer: Lexer<'a, Token>,
    peeked: Option<(Token, Span)>,
}

impl<'a> TokenStream<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Token::lexer(input),
            peeked: None,
        }
    }

    /// Get the current position (line, column) in the source
    #[must_use]
    pub fn position(&self) -> (usize, usize) {
        // For now, return a simple position based on current span
        // In a real implementation, we'd track line/column properly
        let span = self.lexer.span();
        (1, span.start) // Simplified: all on line 1, column is byte offset
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(Token, Span)> {
        if let Some(peeked) = self.peeked.take() {
            return Some(peeked);
        }

        self.lexer.next().map(|result| {
            let token = result.unwrap_or(Token::Bang); // Error recovery
            let span = Span::new(self.lexer.span().start, self.lexer.span().end);
            (token, span)
        })
    }

    pub fn peek(&mut self) -> Option<&(Token, Span)> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<(Token, Span)> {
        // For simplicity, we'll only support peek_nth(1) by cloning the lexer
        if n == 1 {
            let saved_peeked = self.peeked.clone();
            let saved_lexer = self.lexer.clone();

            // Get first token
            let _first = self.peek();
            self.advance();

            // Get second token
            let result = self.peek().cloned();

            // Restore state
            self.lexer = saved_lexer;
            self.peeked = saved_peeked;

            result
        } else {
            None // Not supported for n > 1
        }
    }

    pub fn expect(&mut self, expected: Token) -> anyhow::Result<Span> {
        match self.next() {
            Some((token, span)) if token == expected => Ok(span),
            Some((token, _)) => anyhow::bail!("Expected {:?}, found {:?}", expected, token),
            None => anyhow::bail!("Expected {:?}, found EOF", expected),
        }
    }

    // Alias for next() to avoid clippy warning about Iterator trait
    pub fn advance(&mut self) -> Option<(Token, Span)> {
        self.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    #[allow(clippy::approx_constant)] // Intentional literal for test
    fn test_tokenize_basic() {
        let mut stream = TokenStream::new("let x = 42 + 3.14");

        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Let));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("x".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Equal));
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Integer(42)));
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Plus));
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Float(3.14))); // Intentional literal for test
        assert_eq!(stream.next().map(|(t, _)| t), None);
    }

    #[test]
    fn test_tokenize_pipeline() {
        let mut stream = TokenStream::new("[1, 2, 3] |> map(x => x * 2)");

        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::LeftBracket));
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Integer(1)));
        // ... rest of tokens
    }

    #[test]
    fn test_tokenize_comments() {
        let mut stream = TokenStream::new("x // comment\n+ /* block */ y");

        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("x".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Plus));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("y".to_string()))
        );
    }

    proptest! {
        #[test]
        fn test_tokenize_identifiers(s in "[a-zA-Z_][a-zA-Z0-9_]{0,100}") {
            let mut stream = TokenStream::new(&s);
            match stream.advance() {
                Some((Token::Identifier(id), _)) => prop_assert_eq!(id, s),
                Some((Token::Underscore, _)) if s == "_" => {}, // Special case for underscore
                _ => panic!("Failed to tokenize identifier: {s}"),
            }
        }

        #[test]
        fn test_tokenize_integers(n in 0i64..1_000_000) {
            let s = n.to_string();
            let mut stream = TokenStream::new(&s);
            match stream.advance() {
                Some((Token::Integer(i), _)) => prop_assert_eq!(i, n),
                _ => panic!("Failed to tokenize integer"),
            }
        }
    }
}
