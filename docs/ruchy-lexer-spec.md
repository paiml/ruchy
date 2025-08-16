# Ruchy Lexer Specification

## Design Constraints

The lexer transforms UTF-8 source text into a token stream in a single pass with O(1) lookahead. No parser feedback. No backtracking. No heap allocation per token.

## Token Categories

### Keywords (31 total)

```
fun     let     if      then    else    match   with
for     in      while   loop    break   continue return
import  export  pub     actor   receive spawn   send
async   await   yield   type    alias   trait   impl
true    false   null    _
```

Keywords take precedence over identifiers via maximal munch.

### Operators

```
Arithmetic:  +  -  *  /  %  **
Comparison:  == != <  >  <= >=
Logical:     && || !
Bitwise:     &  |  ^  ~  << >>
Assignment:  =  += -= *= /= %= **= &= |= ^= <<= >>=
Pipeline:    |> <|
Composition: >> <<
Arrow:       -> =>
Range:       .. ...
Access:      .  ?.  ::
```

### Delimiters

```
( )  [ ]  { }  < >
,  ;  :  @  #  $
```

### Literals

#### Numbers

```rust
integer  = decimal | hexadecimal | octal | binary
decimal  = [0-9][0-9_]*
hex      = 0x[0-9a-fA-F][0-9a-fA-F_]*
octal    = 0o[0-7][0-7_]*
binary   = 0b[01][01_]*

float    = [0-9][0-9_]* '.' [0-9][0-9_]* ([eE][+-]?[0-9]+)?
         | [0-9][0-9_]* [eE][+-]?[0-9]+
```

Underscores allowed for readability except at boundaries.

#### Strings

```rust
string        = '"' string_content* '"'
string_content = escape_seq | interpolation_start | [^"\{]
escape_seq    = '\' [nrt\"{]
interpolation_start = '{'  // Emits InterpolationStart token
```

String lexing yields interleaved tokens:
```
"Hello, {name}!" → StringStart("Hello, ") InterpolationStart Ident("name") InterpolationEnd StringEnd("!")
```

Raw strings for regexes:
```rust
raw_string = 'r"' [^"]* '"' | 'r#"' .* '"#' | 'r##"' .* '"##' ...
```

#### Characters

```rust
char = '\'' (escape_seq | [^'\]) '\''
```

### Identifiers

```rust
ident = [a-zA-Z_][a-zA-Z0-9_]*
```

Python-style naming conventions enforced by linter, not lexer.

## Comment Handling

Comments are lexed but emitted via a secondary channel:

```rust
pub struct LexerOutput<'src> {
    tokens: TokenStream,
    comments: CommentStream,
}

pub struct Comment {
    kind: CommentKind,
    span: Span,
    attaches_to: Option<u32>, // Token index
}

pub enum CommentKind {
    Line,      // //
    Block,     // /* */
    Doc,       // ///
    ModDoc,    // //!
}
```

The parser consumes both streams, attaching comments to AST nodes for documentation generation and formatting preservation.

## Implementation

### Token Structure

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub start: u32,  // Byte offset
    pub end: u32,    // Byte offset
}

#[repr(u8)]  // Single byte discriminant
pub enum TokenKind {
    // Keywords (31)
    Fun, Let, If, Then, Else, Match, With,
    For, In, While, Loop, Break, Continue, Return,
    Import, Export, Pub, Actor, Receive, Spawn, Send,
    Async, Await, Yield, Type, Alias, Trait, Impl,
    True, False, Null, Underscore,
    
    // Identifiers and Literals
    Ident,
    Integer,
    Float,
    StringStart,      // Beginning of string literal
    StringFragment,   // String content between interpolations
    StringEnd,        // End of string literal
    InterpolationStart, // '{' inside string
    InterpolationEnd,   // '}' closing interpolation
    Char,
    
    // Operators (fixed-size array indexing)
    Plus, Minus, Star, Slash, Percent, Power,
    EqEq, NotEq, Less, Greater, LessEq, GreaterEq,
    AndAnd, OrOr, Not,
    And, Or, Xor, Tilde, Shl, Shr,
    Eq, PlusEq, MinusEq, StarEq, SlashEq, PercentEq,
    PowerEq, AndEq, OrEq, XorEq, ShlEq, ShrEq,
    Pipe, PipeArrow, ArrowPipe,
    ComposeRight, ComposeLeft,
    Arrow, FatArrow,
    DotDot, DotDotDot,
    Dot, QuestionDot, ColonColon,
    
    // Delimiters
    LParen, RParen, LBracket, RBracket, LBrace, RBrace,
    LAngle, RAngle,
    Comma, Semi, Colon, At, Hash, Dollar,
    
    // Special
    Newline,  // Significant for implicit semicolons
    Eof,
}
```

Size: 9 bytes per token (1 byte kind + 8 bytes span).

### Lexer State Machine

```rust
pub struct Lexer<'src> {
    input: &'src str,
    bytes: &'src [u8],
    pos: usize,
    
    // String interner for identifiers
    interner: &'src mut Interner,
    
    // String interpolation depth tracking
    interpolation_depth: u32,
}

impl<'src> Lexer<'src> {
    #[inline(always)]
    fn current(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
    
    #[inline(always)]
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }
    
    #[inline(always)]
    fn advance(&mut self) -> Option<u8> {
        let ch = self.current();
        self.pos += 1;
        ch
    }
    
    fn lex_string(&mut self) -> Token {
        let start = self.pos;
        self.advance(); // Skip opening "
        
        let mut fragment_start = self.pos;
        
        while let Some(ch) = self.current() {
            match ch {
                b'"' if self.interpolation_depth == 0 => {
                    // End of string
                    let span = Span { start: fragment_start, end: self.pos };
                    self.advance();
                    return Token { kind: StringEnd, span };
                }
                b'{' => {
                    // Start interpolation - emit fragment and delimiter
                    let span = Span { start: fragment_start, end: self.pos };
                    self.advance();
                    self.interpolation_depth += 1;
                    return Token { kind: InterpolationStart, span };
                }
                b'\\' => {
                    // Handle escape sequence
                    self.advance(); // Skip backslash
                    self.advance(); // Skip escaped char
                }
                _ => self.advance(),
            }
        }
        
        // Unterminated string
        Token { kind: Error, span: Span { start, end: self.pos } }
    }
}
```

### Maximal Munch

Longest match wins. Implementation via sorted operator table:

```rust
const OPERATORS: &[(&&[u8], TokenKind)] = &[
    (b"**=", PowerEq),
    (b"<<=", ShlEq),
    (b">>=", ShrEq),
    (b"...", DotDotDot),
    (b"**", Power),
    (b"<<", Shl),
    (b">>", Shr),
    (b"<=", LessEq),
    (b">=", GreaterEq),
    (b"==", EqEq),
    (b"!=", NotEq),
    (b"&&", AndAnd),
    (b"||", OrOr),
    (b"|>", PipeArrow),
    (b"<|", ArrowPipe),
    (b"..", DotDot),
    (b"::", ColonColon),
    (b"->", Arrow),
    (b"=>", FatArrow),
    // ... single-char operators last
];
```

### Keyword Recognition

Perfect hash via compile-time generation:

```rust
fn classify_keyword(ident: &str) -> Option<TokenKind> {
    // Generated via build.rs
    match ident.len() {
        2 => match ident {
            "if" => Some(If),
            "in" => Some(In),
            _ => None,
        },
        3 => match ident {
            "fun" => Some(Fun),
            "let" => Some(Let),
            "for" => Some(For),
            "pub" => Some(Pub),
            _ => None,
        },
        // ...
    }
}
```

### Error Recovery

Invalid tokens produce `Token::Error` with span, enabling parser recovery:

```rust
fn lex_number(&mut self) -> Token {
    let start = self.pos;
    
    if self.current() == b'0' {
        match self.peek() {
            Some(b'x') => return self.lex_hex(),
            Some(b'o') => return self.lex_octal(),
            Some(b'b') => return self.lex_binary(),
            _ => {}
        }
    }
    
    while self.current().map_or(false, |c| c.is_ascii_digit() || c == b'_') {
        self.advance();
    }
    
    // Validate no trailing underscore
    if self.bytes[self.pos - 1] == b'_' {
        return Token {
            kind: Error,
            span: Span { start, end: self.pos },
        };
    }
    
    // Lexer only validates form, not value
    // Parser handles overflow detection
    Token {
        kind: Integer,
        span: Span { start, end: self.pos },
    }
}
```

## Performance Characteristics

- **Throughput**: >100MB/s on modern hardware
- **Memory**: O(1) per token, O(n) for string interning
- **Latency**: <1μs per token
- **Cache locality**: Sequential memory access

## Invariants

1. **Span Coverage**: Every byte of input belongs to exactly one token span
2. **Roundtrip**: `tokens.to_source() == original` (modulo whitespace)
3. **Determinism**: Same input produces identical token stream
4. **Streaming**: Token generation requires only local context

## Testing Strategy

### Property Tests

```rust
#[proptest]
fn prop_lex_never_panics(input: String) {
    let _ = lex(&input);  // Must not panic
}

#[proptest]
fn prop_span_coverage(input: String) {
    let tokens = lex(&input);
    let covered = tokens.iter()
        .map(|t| t.span.end - t.span.start)
        .sum();
    assert_eq!(covered, input.len());
}
```

### Fuzzing

```rust
#[fuzz]
fn fuzz_lexer(data: &[u8]) {
    if let Ok(input) = str::from_utf8(data) {
        let tokens = lex(input);
        assert!(tokens.last().kind == Eof);
    }
}
```

### Benchmarks

```rust
#[bench]
fn bench_lex_10k_loc(b: &mut Bencher) {
    let input = include_str!("../corpus/10k.ruchy");
    b.iter(|| lex(black_box(input)));
    assert!(b.throughput() > 100_000_000);  // 100MB/s
}
```

## Edge Cases

1. **UTF-8 boundaries**: Never split multi-byte characters
2. **Number literal validation**: Lexer validates form (`123_456` valid, `123_` invalid), parser validates value (overflow detection)
3. **Nested comments**: `/* /* */ */` correctly handled via depth tracking
4. **String interpolation**: Lexer emits delimiter tokens, parser handles expression parsing
5. **Operator ambiguity**: `x<y>z` lexes as `x < y > z` (parser handles generics)

## Non-Goals

- **Syntax validation**: Parser's responsibility
- **Semantic analysis**: Type checker's responsibility  
- **Error messages**: Parser provides context
- **Incremental lexing**: Separate module

The lexer does one thing well: byte stream to token stream transformation. Fast. Correct. Predictable.