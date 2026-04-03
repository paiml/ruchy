# Sub-spec: Self-Hosting Compiler — Phase 1: Lexer Implementation

**Parent:** [ruchy-self-hosting-spec.md](../ruchy-self-hosting-spec.md) Phase 1

---

## Phase 1: Lexer Implementation (COMPLETED - Week 1)

### Deterministic Mode Support
```ruchy
struct LexerConfig {
  deterministic: bool,  // Force reproducible output
  intern_strings: bool, // Use string interning for memory efficiency
  parallel: bool,       // Allow parallel tokenization (disabled in deterministic mode)
}
```

### Architecture
```ruchy
module ruchy::lexer {
  use std::io::Read;
  use std::collections::HashMap;
  
  enum Token {
    // Literals
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Bool(bool),
    
    // Keywords
    Let, Mut, Fn, If, Else, Match, For, While,
    Struct, Enum, Trait, Impl, Import, Export,
    Async, Await, Actor, Send, Ask,
    
    // Operators
    Plus, Minus, Star, Slash, Percent, Power,
    Equal, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    And, Or, Not, BitAnd, BitOr, BitXor, BitNot,
    Assign, PlusAssign, MinusAssign,
    
    // Delimiters
    LeftParen, RightParen,
    LeftBracket, RightBracket,
    LeftBrace, RightBrace,
    
    // Special
    Arrow, FatArrow, Pipe, ColonColon, Dot, Comma, Semicolon,
    Colon, Question, At, Hash, Dollar, Ampersand,
    
    // Identifiers and comments
    Identifier(String),
    Comment(String),
    
    // Control
    Newline,
    Eof,
  }
  
  struct Span {
    start: usize,
    end: usize,
    file_id: u32,
  }
  
  struct TokenStream {
    tokens: Vec<(Token, Span)>,
    position: usize,
    source: String,
  }
  
  impl TokenStream {
    fn new(source: String) -> Result<TokenStream, LexError> {
      let tokens = tokenize(&source)?;
      Ok(TokenStream { tokens, position: 0, source })
    }
    
    fn peek(&self) -> Option<&Token> {
      self.tokens.get(self.position).map(|(t, _)| t)
    }
    
    fn advance(&mut self) -> Option<Token> {
      if self.position < self.tokens.len() {
        let token = self.tokens[self.position].0.clone();
        self.position += 1;
        Some(token)
      } else {
        None
      }
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
      match self.advance() {
        Some(token) if token == expected => Ok(()),
        Some(token) => Err(ParseError::UnexpectedToken { expected, found: token }),
        None => Err(ParseError::UnexpectedEof),
      }
    }
  }
  
  fn tokenize(input: &str) -> Result<Vec<(Token, Span)>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();
    
    while let Some((pos, ch)) = chars.next() {
      let start = pos;
      let token = match ch {
        // Whitespace
        ' ' | '\t' | '\r' => continue,
        '\n' => Token::Newline,
        
        // Single-character tokens
        '(' => Token::LeftParen,
        ')' => Token::RightParen,
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ';' => Token::Semicolon,
        '@' => Token::At,
        '#' => Token::Hash,
        '$' => Token::Dollar,
        '?' => Token::Question,
        
        // Multi-character operators
        '+' => match chars.peek() {
          Some((_, '=')) => { chars.next(); Token::PlusAssign }
          Some((_, '+')) => { chars.next(); Token::Increment }
          _ => Token::Plus
        },
        '-' => match chars.peek() {
          Some((_, '=')) => { chars.next(); Token::MinusAssign }
          Some((_, '-')) => { chars.next(); Token::Decrement }
          Some((_, '>')) => { chars.next(); Token::Arrow }
          _ => Token::Minus
        },
        '*' => match chars.peek() {
          Some((_, '*')) => { chars.next(); Token::Power }
          Some((_, '=')) => { chars.next(); Token::StarAssign }
          _ => Token::Star
        },
        '/' => match chars.peek() {
          Some((_, '/')) => {
            chars.next();
            let comment = consume_line(&mut chars);
            Token::Comment(comment)
          }
          Some((_, '=')) => { chars.next(); Token::SlashAssign }
          _ => Token::Slash
        },
        '=' => match chars.peek() {
          Some((_, '=')) => { chars.next(); Token::Equal }
          Some((_, '>')) => { chars.next(); Token::FatArrow }
          _ => Token::Assign
        },
        '!' => match chars.peek() {
          Some((_, '=')) => { chars.next(); Token::NotEqual }
          _ => Token::Not
        },
        '<' => match chars.peek() {
          Some((_, '=')) => { chars.next(); Token::LessEqual }
          Some((_, '<')) => { chars.next(); Token::LeftShift }
          _ => Token::Less
        },
        '>' => match chars.peek() {
          Some((_, '=')) => { chars.next(); Token::GreaterEqual }
          Some((_, '>')) => { chars.next(); Token::RightShift }
          _ => Token::Greater
        },
        '&' => match chars.peek() {
          Some((_, '&')) => { chars.next(); Token::And }
          _ => Token::Ampersand
        },
        '|' => match chars.peek() {
          Some((_, '|')) => { chars.next(); Token::Or }
          Some((_, '>')) => { chars.next(); Token::Pipe }
          _ => Token::BitOr
        },
        ':' => match chars.peek() {
          Some((_, ':')) => { chars.next(); Token::ColonColon }
          _ => Token::Colon
        },
        '.' => match chars.peek() {
          Some((_, '.')) => {
            chars.next();
            match chars.peek() {
              Some((_, '.')) => { chars.next(); Token::Ellipsis }
              _ => Token::Range
            }
          }
          _ => Token::Dot
        },
        
        // String literals
        '"' => {
          let string = consume_string(&mut chars)?;
          Token::String(string)
        },
        
        // Character literals  
        '\'' => {
          let ch = consume_char(&mut chars)?;
          Token::Char(ch)
        },
        
        // Numbers
        '0'..='9' => {
          let number = consume_number(ch, &mut chars);
          parse_number(&number)?
        },
        
        // Identifiers and keywords
        'a'..='z' | 'A'..='Z' | '_' => {
          let ident = consume_identifier(ch, &mut chars);
          match_keyword_or_ident(&ident)
        },
        
        _ => return Err(LexError::UnexpectedChar(ch, start)),
      };
      
      let end = chars.peek().map(|(p, _)| *p).unwrap_or(input.len());
      tokens.push((token, Span { start, end, file_id: 0 }));
    }
    
    tokens.push((Token::Eof, Span { start: input.len(), end: input.len(), file_id: 0 }));
    Ok(tokens)
  }
  
  fn match_keyword_or_ident(s: &str) -> Token {
    match s {
      "let" => Token::Let,
      "mut" => Token::Mut,
      "fn" => Token::Fn,
      "if" => Token::If,
      "else" => Token::Else,
      "match" => Token::Match,
      "for" => Token::For,
      "while" => Token::While,
      "loop" => Token::Loop,
      "break" => Token::Break,
      "continue" => Token::Continue,
      "return" => Token::Return,
      "struct" => Token::Struct,
      "enum" => Token::Enum,
      "trait" => Token::Trait,
      "impl" => Token::Impl,
      "import" => Token::Import,
      "export" => Token::Export,
      "module" => Token::Module,
      "async" => Token::Async,
      "await" => Token::Await,
      "actor" => Token::Actor,
      "send" => Token::Send,
      "ask" => Token::Ask,
      "spawn" => Token::Spawn,
      "true" => Token::Bool(true),
      "false" => Token::Bool(false),
      "Ok" => Token::Ok,
      "Err" => Token::Err,
      "Some" => Token::Some,
      "None" => Token::None,
      _ => Token::Identifier(s.to_string()),
    }
  }
}
```
