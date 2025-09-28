//! Lexical analysis and tokenization
use crate::frontend::ast::Span;
use logos::{Lexer, Logos};
/// Process a basic escape character
fn process_basic_escape(ch: char) -> Option<char> {
    match ch {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        '\\' => Some('\\'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '0' => Some('\0'),
        _ => None,
    }
}
/// Process a Unicode escape sequence
fn process_unicode_escape(chars: &mut std::str::Chars) -> String {
    chars.next(); // consume '{'
                  // Most Unicode escapes are 4-6 chars
    let mut hex = String::with_capacity(6);
    for hex_char in chars.by_ref() {
        if hex_char == '}' {
            break;
        }
        hex.push(hex_char);
    }
    if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
        if let Some(unicode_char) = char::from_u32(code_point) {
            return unicode_char.to_string();
        }
    }
    // Invalid code point or hex, keep as literal
    format!("\\u{{{hex}}}")
}
/// Process escape sequences in a string literal
fn process_escapes(s: &str) -> String {
    // Pre-allocate based on input size for common case
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                None => result.push('\\'), // End of string
                Some('u') if chars.as_str().starts_with('{') => {
                    result.push_str(&process_unicode_escape(&mut chars));
                }
                Some(escape_ch) => {
                    if let Some(escaped) = process_basic_escape(escape_ch) {
                        result.push(escaped);
                    } else {
                        // Unknown escape sequence, keep as literal
                        result.push('\\');
                        result.push(escape_ch);
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // Literals
    #[regex(r"[0-9]+(?:i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?", |lex| {
        let slice = lex.slice();
        // Strip type suffix for parsing the numeric value
        let (num_part, _type_suffix) = if let Some(pos) = slice.find(|c: char| c.is_alphabetic()) {
            (&slice[..pos], Some(&slice[pos..]))
        } else {
            (slice, None)
        };
        num_part.parse::<i64>().ok()
    })]
    Integer(i64),
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        Some(process_escapes(inner))
    })]
    String(String),
    #[regex(r#"f"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Remove f" prefix and " suffix
        let inner = &s[2..s.len()-1];
        Some(process_escapes(inner))
    })]
    FString(String),
    #[regex(r#"r"([^"])*""#, |lex| {
        let s = lex.slice();
        // Remove r" prefix and " suffix - no escape processing for raw strings
        Some(s[2..s.len()-1].to_string())
    })]
    RawString(String),
    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        if inner.len() == 1 {
            inner.chars().next()
        } else if inner.starts_with('\\') && inner.len() == 2 {
            match inner.chars().nth(1) {
                Some('n') => Some('\n'),
                Some('t') => Some('\t'),
                Some('r') => Some('\r'),
                Some('\\') => Some('\\'),
                Some('\'') => Some('\''),
                Some('0') => Some('\0'),
                _ => None,
            }
        } else {
            None
        }
    })]
    Char(char),
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Bool(bool),
    // Keywords
    #[token("fun")]
    Fun,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("var")]
    Var,
    #[token("mod")]
    Mod,
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
    #[token("loop")]
    Loop,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("throw")]
    Throw,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("finally")]
    Finally,
    #[token("return")]
    Return,
    #[token("command")]
    Command,
    #[token("Ok")]
    Ok,
    #[token("Err")]
    Err,
    #[token("Some")]
    Some,
    #[token("None")]
    None,
    #[token("null")]
    Null,
    #[token("Result")]
    Result,
    #[token("Option")]
    Option,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("impl")]
    Impl,
    #[token("trait")]
    Trait,
    #[token("extend")]
    Extend,
    #[token("actor")]
    Actor,
    #[token("spawn")]
    Spawn,
    #[token("state")]
    State,
    #[token("property")]
    Property,
    #[token("receive")]
    Receive,
    #[token("send")]
    Send,
    #[token("ask")]
    Ask,
    #[token("type")]
    Type,
    #[token("where")]
    Where,
    #[token("const", priority = 2)]
    Const,
    #[token("unsafe", priority = 2)]
    Unsafe,
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
    #[token("from")]
    From,
    #[token("module")]
    Module,
    #[token("export")]
    Export,
    #[token("default")]
    Default,
    #[token("class")]
    Class,
    #[token("self")]
    Self_,
    #[token("super")]
    Super,
    #[token("crate")]
    Crate,
    #[token("df", priority = 2)]
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
    #[token("<?")]
    ActorQuery,
    #[token("<-")]
    LeftArrow,
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
    #[token("@")]
    At,
    #[token("~")]
    Tilde,
    #[token("\\")]
    Backslash,
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
    #[token("%=")]
    PercentEqual,
    #[token("**=")]
    PowerEqual,
    #[token("&=")]
    AmpersandEqual,
    #[token("|=")]
    PipeEqual,
    #[token("^=")]
    CaretEqual,
    #[token("<<=")]
    LeftShiftEqual,
    #[token("++")]
    Increment,
    #[token("--")]
    Decrement,
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
    #[token("...")]
    DotDotDot,
    #[token("??")]
    NullCoalesce,
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
        )
    }
    #[must_use]
    pub fn is_unary_op(&self) -> bool {
        matches!(
            self,
            Token::Bang | Token::Minus | Token::Tilde | Token::Ampersand
        )
    }
    #[must_use]
    pub fn is_assignment_op(&self) -> bool {
        matches!(
            self,
            Token::Equal
                | Token::PlusEqual
                | Token::MinusEqual
                | Token::StarEqual
                | Token::SlashEqual
                | Token::PercentEqual
                | Token::PowerEqual
                | Token::AmpersandEqual
                | Token::PipeEqual
                | Token::CaretEqual
                | Token::LeftShiftEqual
        )
    }
}
pub struct TokenStream<'a> {
    lexer: Lexer<'a, Token>,
    peeked: Option<(Token, Span)>,
    input: &'a str,
    current_position: usize,
}
/// Saved position in the token stream for backtracking
#[derive(Clone)]
pub struct TokenStreamPosition<'a> {
    lexer: Lexer<'a, Token>,
    peeked: Option<(Token, Span)>,
    current_position: usize,
}
impl<'a> TokenStream<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: Token::lexer(input),
            peeked: None,
            input,
            current_position: 0,
        }
    }

    /// Get the current line and column position
    pub fn current_position(&self) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in self.input.chars().enumerate() {
            if i >= self.current_position {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// Get a string showing the context around the current position
    pub fn get_context_string(&self) -> String {
        let start = self.current_position.saturating_sub(20);
        let end = (self.current_position + 20).min(self.input.len());
        let context = &self.input[start..end];
        format!("...{context}...")
    }
    /// Save the current position for later restoration
    #[must_use]
    pub fn position(&self) -> TokenStreamPosition<'a> {
        TokenStreamPosition {
            lexer: self.lexer.clone(),
            peeked: self.peeked.clone(),
            current_position: self.current_position,
        }
    }
    /// Restore a previously saved position
    pub fn set_position(&mut self, pos: TokenStreamPosition<'a>) {
        self.lexer = pos.lexer;
        self.peeked = pos.peeked;
        self.current_position = pos.current_position;
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<(Token, Span)> {
        if let Some(peeked) = self.peeked.take() {
            self.current_position = peeked.1.end;
            return Some(peeked);
        }
        self.lexer.next().map(|result| {
            let token = result.unwrap_or(Token::Bang); // Error recovery
            let span = Span::new(self.lexer.span().start, self.lexer.span().end);
            self.current_position = span.end;
            (token, span)
        })
    }
    pub fn peek(&mut self) -> Option<&(Token, Span)> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        self.peeked.as_ref()
    }
    /// Look ahead n tokens in the stream
    pub fn peek_ahead(&mut self, n: usize) -> Option<(Token, Span)> {
        self.peek_nth(n)
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
    pub fn peek_nth_is_colon(&mut self, n: usize) -> bool {
        if n == 0 {
            self.peek().is_some_and(|(t, _)| matches!(t, Token::Colon))
        } else {
            self.peek_nth(n)
                .is_some_and(|(t, _)| matches!(t, Token::Colon))
        }
    }
    /// Expect a specific token and return its span
    ///
    /// # Errors
    ///
    /// Returns an error if the next token doesn't match the expected token or if we reached EOF
    pub fn expect(&mut self, expected: &Token) -> anyhow::Result<Span> {
        match self.next() {
            Some((token, span)) if token == *expected => Ok(span),
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
#[allow(clippy::unwrap_used)]
#[allow(clippy::panic)]
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
        let mut stream = TokenStream::new("[1, 2, 3] >> map(|x| x * 2)");
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
    #[test]
    fn test_process_basic_escape() {
        assert_eq!(process_basic_escape('n'), Some('\n'));
        assert_eq!(process_basic_escape('t'), Some('\t'));
        assert_eq!(process_basic_escape('r'), Some('\r'));
        assert_eq!(process_basic_escape('\\'), Some('\\'));
        assert_eq!(process_basic_escape('"'), Some('"'));
        assert_eq!(process_basic_escape('\''), Some('\''));
        assert_eq!(process_basic_escape('0'), Some('\0'));
        assert_eq!(process_basic_escape('x'), None); // Invalid escape
    }

    #[test]
    fn test_process_unicode_escape() {
        let mut chars = "{41}".chars();
        assert_eq!(process_unicode_escape(&mut chars), "A");

        let mut chars = "{1F600}".chars();
        assert_eq!(process_unicode_escape(&mut chars), "😀");

        let mut chars = "{INVALID}".chars();
        assert_eq!(process_unicode_escape(&mut chars), "\\u{INVALID}");
    }

    #[test]
    fn test_process_escapes() {
        assert_eq!(process_escapes("Hello\\nWorld"), "Hello\nWorld");
        assert_eq!(process_escapes("Tab\\tHere"), "Tab\tHere");
        assert_eq!(process_escapes("Quote\\\"Here"), "Quote\"Here");
        assert_eq!(process_escapes("Unicode\\u{41}"), "UnicodeA");
        assert_eq!(process_escapes("Invalid\\x"), "Invalid\\x");
        assert_eq!(process_escapes("Backslash\\\\"), "Backslash\\");
    }

    #[test]
    fn test_tokenize_strings() {
        let mut stream = TokenStream::new(r#""Hello, World!""#);
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::String("Hello, World!".to_string()))
        );

        let mut stream = TokenStream::new(r"'c'");
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Char('c')));
    }

    #[test]
    fn test_tokenize_keywords() {
        let keywords = vec![
            ("let", Token::Let),
            ("var", Token::Var),
            ("fun", Token::Fun),
            ("fn", Token::Fn),
            ("if", Token::If),
            ("else", Token::Else),
            ("match", Token::Match),
            ("for", Token::For),
            ("while", Token::While),
            ("loop", Token::Loop),
            ("return", Token::Return),
            ("break", Token::Break),
            ("continue", Token::Continue),
            ("true", Token::Bool(true)),
            ("false", Token::Bool(false)),
            ("null", Token::Null),
        ];

        for (keyword_str, expected_token) in keywords {
            let mut stream = TokenStream::new(keyword_str);
            assert_eq!(
                stream.next().map(|(t, _)| t),
                Some(expected_token),
                "Failed to tokenize keyword: {keyword_str}"
            );
        }
    }

    #[test]
    fn test_tokenize_operators() {
        let operators = vec![
            ("+", Token::Plus),
            ("-", Token::Minus),
            ("*", Token::Star),
            ("/", Token::Slash),
            ("%", Token::Percent),
            ("**", Token::Power),
            ("==", Token::EqualEqual),
            ("!=", Token::NotEqual),
            ("<", Token::Less),
            ("<=", Token::LessEqual),
            (">", Token::Greater),
            (">=", Token::GreaterEqual),
            ("&&", Token::AndAnd),
            ("||", Token::OrOr),
            ("!", Token::Bang),
            ("=", Token::Equal),
            ("|>", Token::Pipeline),
            // (">>" is parsed differently)
            ("<<", Token::LeftShift),
        ];

        for (op_str, expected_token) in operators {
            let mut stream = TokenStream::new(op_str);
            assert_eq!(
                stream.next().map(|(t, _)| t),
                Some(expected_token),
                "Failed to tokenize operator: {op_str}"
            );
        }
    }

    #[test]
    fn test_tokenize_punctuation() {
        let punctuation = vec![
            ("(", Token::LeftParen),
            (")", Token::RightParen),
            ("[", Token::LeftBracket),
            ("]", Token::RightBracket),
            ("{", Token::LeftBrace),
            ("}", Token::RightBrace),
            (",", Token::Comma),
            (".", Token::Dot),
            (":", Token::Colon),
            ("::", Token::ColonColon),
            (";", Token::Semicolon),
            ("->", Token::Arrow),
            ("=>", Token::FatArrow),
            // ("...", Token::Ellipsis), // Doesn't exist
        ];

        for (punct_str, expected_token) in punctuation {
            let mut stream = TokenStream::new(punct_str);
            assert_eq!(
                stream.next().map(|(t, _)| t),
                Some(expected_token),
                "Failed to tokenize punctuation: {punct_str}"
            );
        }
    }

    #[test]
    fn test_tokenize_floats() {
        let floats = vec!["3.14", "0.0", "1.0", "999.999", "0.001"];

        for float_str in floats {
            let mut stream = TokenStream::new(float_str);
            match stream.next() {
                Some((Token::Float(_), _)) => {}
                _ => panic!("Failed to tokenize float: {float_str}"),
            }
        }
    }

    #[test]
    fn test_tokenize_complex_expression() {
        let mut stream = TokenStream::new("fun add(x: i32, y: i32) -> i32 { x + y }");
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Fun));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("add".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::LeftParen));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("x".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Colon));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("i32".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Comma));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("y".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Colon));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("i32".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::RightParen));
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Arrow));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("i32".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::LeftBrace));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("x".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Plus));
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("y".to_string()))
        );
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::RightBrace));
    }

    #[test]
    fn test_token_stream_peek() {
        let mut stream = TokenStream::new("let x = 42");

        // Peek should not consume
        let peeked = stream.peek().map(|(t, _)| t.clone());
        assert_eq!(peeked, Some(Token::Let));

        // Peek again should return same
        let peeked2 = stream.peek().map(|(t, _)| t.clone());
        assert_eq!(peeked2, Some(Token::Let));

        // Next should consume
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Let));

        // Now peek should show next token
        let peeked3 = stream.peek().map(|(t, _)| t.clone());
        assert_eq!(peeked3, Some(Token::Identifier("x".to_string())));
    }

    #[test]
    fn test_token_stream_position() {
        let mut stream = TokenStream::new("a + b");

        // Save position at start
        let pos = stream.position();

        // Advance
        stream.advance();
        stream.advance();
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("b".to_string()))
        );

        // Restore position
        stream.set_position(pos);
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("a".to_string()))
        );
    }

    #[test]
    fn test_token_stream_expect() {
        let mut stream = TokenStream::new("let x");

        // Expect correct token
        let span = stream.expect(&Token::Let);
        assert!(span.is_ok());

        // Expect wrong token
        let result = stream.expect(&Token::If);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_interpolated_string() {
        let mut stream = TokenStream::new(r#"f"Hello {name}!""#);
        match stream.next() {
            Some((Token::FString(s), _)) => {
                assert!(s.contains("Hello"));
            }
            _ => panic!("Failed to tokenize interpolated string"),
        }
    }

    #[test]
    fn test_tokenize_special_tokens() {
        let mut stream = TokenStream::new("_");
        assert_eq!(stream.next().map(|(t, _)| t), Some(Token::Underscore));
    }

    #[test]
    fn test_peek_nth() {
        let mut stream = TokenStream::new("a b c");

        // Peek at second token
        let second = stream.peek_nth(1).map(|(t, _)| t);
        assert_eq!(second, Some(Token::Identifier("b".to_string())));

        // First token should still be unconsumed
        assert_eq!(
            stream.next().map(|(t, _)| t),
            Some(Token::Identifier("a".to_string()))
        );
    }

    #[test]
    fn test_peek_nth_is_colon() {
        let mut stream = TokenStream::new(": x");
        assert!(stream.peek_nth_is_colon(0));

        let mut stream = TokenStream::new("x : y");
        assert!(!stream.peek_nth_is_colon(0));
        assert!(stream.peek_nth_is_colon(1));
    }

    proptest! {
        #[test]
        fn test_tokenize_identifiers(s in "[a-zA-Z_][a-zA-Z0-9_]{0,100}") {
            // Skip reserved keywords that should tokenize as their respective tokens
            let reserved_keywords = [
                "true", "false", "fun", "fn", "let", "var", "mod", "if", "else", "match",
                "for", "in", "while", "loop", "async", "await", "throw", "try", "catch",
                "return", "command", "Ok", "Err", "Some", "None", "null", "Result", "Option",
                "break", "continue", "struct", "enum", "impl", "trait", "extend", "actor",
                "state", "receive", "send", "ask", "type", "where", "const", "static",
                "mut", "pub", "import", "use", "as", "module", "export", "df"
            ];
            if reserved_keywords.contains(&s.as_str()) {
                return Ok(()); // Skip test for reserved keywords
            }
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

        #[test]
        fn test_process_escapes_never_panics(s: String) {
            // Should never panic on any input
            let _ = process_escapes(&s);
        }

        #[test]
        fn test_tokenize_never_panics(s: String) {
            // Should never panic on any input
            let mut stream = TokenStream::new(&s);
            // Consume all tokens
            while stream.next().is_some() {}
        }
    }
}
