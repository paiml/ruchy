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
/// Extract hex digits from Unicode escape sequence
fn extract_unicode_hex(chars: &mut std::str::Chars) -> String {
    chars.next(); // consume '{'
    let mut hex = String::with_capacity(6); // Most Unicode escapes are 4-6 chars
    for hex_char in chars.by_ref() {
        if hex_char == '}' {
            break;
        }
        hex.push(hex_char);
    }
    hex
}

/// Process a Unicode escape sequence
fn process_unicode_escape(chars: &mut std::str::Chars) -> String {
    let hex = extract_unicode_hex(chars);

    // Try to parse as valid Unicode code point
    u32::from_str_radix(&hex, 16)
        .ok()
        .and_then(char::from_u32)
        .map_or_else(|| format!("\\u{{{hex}}}"), |c| c.to_string())
}

/// Handle a backslash escape sequence
fn process_backslash_escape(chars: &mut std::str::Chars, result: &mut String) {
    match chars.next() {
        None => result.push('\\'), // End of string
        Some('u') if chars.as_str().starts_with('{') => {
            result.push_str(&process_unicode_escape(chars));
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
}

/// Process escape sequences in a string literal
fn process_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            process_backslash_escape(&mut chars, &mut result);
        } else {
            result.push(ch);
        }
    }
    result
}

/// Lex nested block comments with depth tracking
/// Handles Rust-style nested comments: /* outer /* inner */ still outer */
fn lex_nested_block_comment(lex: &mut Lexer<Token>) -> Option<String> {
    let remainder = lex.remainder();
    let bytes = remainder.as_bytes();
    let mut depth = 1; // We've already seen the opening /*
    let mut content = String::new();
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Found nested opening /*
            depth += 1;
            content.push('/');
            content.push('*');
            i += 2;
        } else if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
            // Found closing */
            depth -= 1;
            if depth == 0 {
                // Found matching close - advance lexer and return content
                lex.bump(i + 2);
                return Some(content);
            }
            content.push('*');
            content.push('/');
            i += 2;
        } else {
            // Regular character - handle UTF-8
            let ch = remainder[i..].chars().next()?;
            content.push(ch);
            i += ch.len_utf8();
        }
    }

    // Reached end of input without finding matching close
    // For error recovery, consume remainder
    lex.bump(remainder.len());
    Some(content)
}

#[derive(Logos, Debug, PartialEq, Clone)]
// Issue #163: Include \r for Windows line ending (CRLF) support
#[logos(skip r"[ \t\n\r\f]+")]
pub enum Token {
    // Comments (NEW: Track instead of skip)
    // Preserve exact text including whitespace for perfect formatting
    #[regex(r"///[^\n]*", |lex| lex.slice()[3..].to_string())]
    DocComment(String),

    #[regex(r"//[^\n]*", |lex| lex.slice()[2..].to_string())]
    LineComment(String),

    // PARSER-075: Nested block comments with depth tracking (Rust-style)
    #[token("/*", lex_nested_block_comment)]
    BlockComment(String),

    // Python/Ruby-style hash comments (PARSER-053)
    // Match # followed by non-[ character (or end of line)
    // Pattern: # followed by (NOT '[' and NOT newline), then anything until newline
    #[regex(r"#(?:[^\[\n][^\n]*)?", |lex| {
        let s = lex.slice();
        if s.len() > 1 { s[1..].to_string() } else { String::new() }
    })]
    HashComment(String),

    // Literals
    // Issue #168: Hexadecimal literals (0x or 0X prefix)
    #[regex(r"0[xX][0-9a-fA-F]+(?:i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?", |lex| {
        let slice = lex.slice();
        slice.to_string()
    })]
    HexInteger(String),
    #[regex(r"[0-9]+(?:i8|i16|i32|i64|i128|isize|u8|u16|u32|u64|u128|usize)?", |lex| {
        let slice = lex.slice();
        // Parse type suffix and numeric value separately - store as string to preserve suffix
        slice.to_string()
    })]
    Integer(String),
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),
    // Double-quoted strings
    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        Some(process_escapes(inner))
    })]
    // PARSER-072: Single-quoted strings (multi-char only, single-char handled by Char)
    // PARSER-080: Exclude '>' and newlines to prevent matching across lifetime boundaries
    // PARSER-079: Extend exclusions to prevent String pattern from interfering with Lifetime tokens
    //   Exclude: space, ;, }, ), , : to allow lifetime tokens like 'outer: to parse correctly
    //   BUG-FIX: Added : to exclusion list - 'outer: was matching String pattern and failing
    //   Pattern: empty string ('') OR 2+ characters between single quotes
    #[regex(r"'(([^'\\>\n \t;},):]|\\.)([^'\\>\n \t;},):]|\\.)+|)'", |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        // Only match if it's NOT a single character (let Char handle that)
        if inner.len() != 1 && !(inner.starts_with('\\') && inner.len() == 2) {
            Some(process_escapes(inner))
        } else {
            None
        }
    })]
    String(String),
    #[regex(r#"f"([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Remove f" prefix and " suffix
        let inner = &s[2..s.len()-1];
        Some(process_escapes(inner))
    })]
    FString(String),
    // Raw strings with hash delimiters: r#"..."# (allows quotes inside)
    #[regex(r####"r#"([^"]|"[^#])*"#"####, |lex| {
        let s = lex.slice();
        // Remove r#" prefix and "# suffix - no escape processing for raw strings
        Some(s[3..s.len()-2].to_string())
    })]
    // Basic raw strings: r"..." (no hash delimiters)
    #[regex(r#"r"([^"])*""#, |lex| {
        let s = lex.slice();
        // Remove r" prefix and " suffix - no escape processing for raw strings
        Some(s[2..s.len()-1].to_string())
    })]
    RawString(String),
    // Re-enabled - Char matches 'x' format
    // Priority 7: highest among single-quote patterns to match 'a' before String
    #[regex(r"'([^'\\]|\\.)'", priority = 7, callback = |lex| {
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
    #[regex(r"b'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        let inner = &s[2..s.len()-1];  // Skip b' prefix
        if inner.len() == 1 {
            Some(inner.as_bytes()[0])
        } else if inner.starts_with('\\') && inner.len() == 2 {
            match inner.chars().nth(1) {
                Some('n') => Some(b'\n'),
                Some('t') => Some(b'\t'),
                Some('r') => Some(b'\r'),
                Some('\\') => Some(b'\\'),
                Some('\'') => Some(b'\''),
                Some('0') => Some(b'\0'),
                _ => None,
            }
        } else {
            None
        }
    })]
    Byte(u8),
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
    #[token("lazy")]
    Lazy,
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
    // PARSER-089: "command" removed as vestigial keyword - now lexed as normal identifier
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
    #[token("effect")]
    Effect,
    #[token("handle")]
    Handle,
    #[token("handler")]
    Handler,
    // NOTE: 'state' removed as keyword - now context-sensitive in actor parser
    // This fixes DEFECT-PARSER-001 where 'let mut state' failed after if/else chains
    #[token("property")]
    Property,
    #[token("private")]
    Private,
    #[token("protected")]
    Protected,
    #[token("sealed")]
    Sealed,
    #[token("final")]
    Final,
    #[token("abstract")]
    Abstract,
    #[token("mixin")]
    Mixin,
    #[token("operator")]
    Operator,
    #[token("interface")]
    Interface,
    #[token("implements")]
    Implements,
    #[token("override")]
    Override,
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
    // PARSER-081: @label syntax for labeled loops (replaces 'lifetime)
    // Priority 3 ensures @label matches before standalone @ (Token::At used for decorators)
    #[regex("@[a-zA-Z_][a-zA-Z0-9_]*", priority = 3, callback = |lex| lex.slice().to_string())]
    Label(String),
    // PARSER-082: Atoms (:symbol)
    #[regex(r":[a-zA-Z_][a-zA-Z0-9_]*", priority = 3, callback = |lex| lex.slice()[1..].to_string())]
    Atom(String),
    // Deprecated: 'lifetime syntax - kept for backward compatibility during migration
    // Priority 5 ensures 'lifetime matches before single-quoted string patterns
    #[regex(r"'[a-zA-Z_][a-zA-Z0-9_]*", priority = 5, callback = |lex| lex.slice().to_string())]
    Lifetime(String),
    #[token("pub")]
    Pub,
    #[token("import")]
    Import,
    #[token("use")]
    Use,
    #[token("as")]
    As,
    #[token("with")]
    With,
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
    // Attribute support - match #[ specifically to avoid conflict with hash comments
    // Priority 3 ensures #[ is matched before # comments (which default to priority 0)
    #[token("#[", priority = 3)]
    AttributeStart,
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
    /// Get reference to the source code
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.input
    }

    #[must_use]
    pub fn new(input: &'a str) -> Self {
        // Handle shebang: Skip first line if it starts with #!
        // This allows executable scripts like: #!/usr/bin/env ruchy
        let processed_input = if input.starts_with("#!") {
            // Find the end of the first line
            if let Some(newline_pos) = input.find('\n') {
                // Skip the shebang line (including the newline)
                &input[newline_pos + 1..]
            } else {
                // Entire file is just a shebang, treat as empty
                ""
            }
        } else {
            input
        };

        Self {
            lexer: Token::lexer(processed_input),
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
        // DEFECT-026 FIX: Support n=1 and n=2 for where clause lookahead
        if n == 0 {
            return self.peek().cloned();
        }
        let saved_peeked = self.peeked.clone();
        let saved_lexer = self.lexer.clone();

        // Advance n times
        for _ in 0..n {
            let _ = self.peek();
            self.advance();
        }
        // Get the nth token
        let result = self.peek().cloned();

        // Restore state
        self.lexer = saved_lexer;
        self.peeked = saved_peeked;
        result
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
            Some((token, _)) => anyhow::bail!("Expected {expected:?}, found {token:?}"),
            None => anyhow::bail!("Expected {expected:?}, found EOF"),
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
#[path = "lexer_tests.rs"]
mod tests;
