# Ruchy: Complete Language and System Specification

*Version 14.0 - Single source of truth consolidating all 38 specification documents*

## Table of Contents

### Core Language Specifications
1. [Language Specification](#1-language-specification)
2. [Grammar Reference](#2-grammar-reference)
3. [Implementation Specification](#3-implementation-specification)
4. [Parser Specification](#4-parser-specification)
5. [Lexer Specification](#5-lexer-specification)
6. [Script Capability Specification](#6-script-capability-specification)
7. [Classes Specification](#7-classes-specification)
8. [Functional Programming Specification](#8-functional-programming-specification)
9. [Interpreter Specification](#9-interpreter-specification)
10. [Unary Operator Specification](#10-unary-operator-specification)
11. [Systems Operations Specification](#11-systems-operations-specification)
12. [Security Specification](#12-security-specification)

### Architecture Specifications
13. [MCP Message-Passing Architecture](#13-mcp-message-passing-architecture)
14. [LSP Specification](#14-lsp-specification)
15. [Critical Missing Components](#15-critical-missing-components)
16. [Binary Architecture](#16-binary-architecture)
17. [Edge Cases Specification](#17-edge-cases-specification)
18. [REPL Testing Specification](#18-repl-testing-specification)
19. [REPL UX Specification](#19-repl-ux-specification)
20. [Docker Specification](#20-docker-specification)

### Integration Specifications
21. [Cargo Integration](#21-cargo-integration)
22. [Depyler Integration](#22-depyler-integration)
23. [Rust Cargo InterOp](#23-rust-cargo-interop)

### Execution Mode Specifications
24. [One-Liner and Script Execution](#24-one-liner-and-script-execution)
25. [Disassembly Specification](#25-disassembly-specification)
26. [Advanced Mathematical REPL](#26-advanced-mathematical-repl)

### Quality & Testing Specifications
27. [Quality Gates](#27-quality-gates)
28. [Provability](#28-provability)
29. [Lint Specification](#29-lint-specification)
30. [Quality Scoring Specification](#30-quality-scoring-specification)
31. [Language Completeness Validation](#31-language-completeness-validation)

### Project Management
32. [Master TODO](#32-master-todo)
33. [Project Status](#33-project-status)
34. [Deep Context](#34-deep-context)

### External Dependencies
35. [PMAT Integration](#35-pmat-integration)
36. [PDMT Integration](#36-pdmt-integration)
37. [External Tool Dependencies](#37-external-tool-dependencies)

### Appendices
38. [Complete Grammar Definition](#38-complete-grammar-definition)
39. [Meta-Specification](#39-meta-specification)

---

## 1. Language Specification

### 1.1 Design Philosophy

Ruchy combines the ergonomics of Swift, Kotlin, and Elixir with Rust's performance guarantees through mechanical syntax transformation. Core principles:

- **Familiarity First**: Syntax borrowed from successful languages
- **Progressive Complexity**: Simple code looks simple, complex features available on demand
- **Zero Runtime Overhead**: All abstractions compile to efficient Rust
- **Type Inference**: Types required only at module boundaries
- **DataFrame-First**: Polars as primary collection type

### 1.2 Type System

```rust
// Primitive Types
i8, i16, i32, i64, i128
u8, u16, u32, u64, u128
f32, f64
bool, char, String, ()

// Composite Types
[T]                    // Arrays/Lists (maps to Series/Vec)
(T1, T2, ...)         // Tuples  
T1 -> T2              // Functions
Option<T>             // Nullable types
Result<T, E>          // Error handling
&T, &mut T            // References

// Mathematical Types (first-class)
DataFrame             // Tabular data (Polars)
LazyFrame            // Lazy DataFrame evaluation
Series               // Column data
Matrix<T, R, C>      // Linear algebra (nalgebra)
Vector<T, N>         // N-dimensional vector
Array<T, D>          // N-dimensional array (ndarray)
SymExpr              // Symbolic expression
Formula              // Statistical formula (y ~ x1 + x2)
Distribution<T>      // Probability distribution
Complex<T>           // Complex numbers

// Type Aliases
type UserId = i64
type Callback = fun(i32) -> bool
type Point = (x: f64, y: f64)

// Refinement Types (future)
{x: i32 | x > 0}     // Positive integers
{s: String | s.len() < 100}  // Bounded strings
```

### 1.3 Core Language Features

#### Functions
```rust
// Basic function with type inference
fun add(x: i32, y: i32) -> i32 {
    x + y
}

// Single expression function
fun double(x: i32) = x * 2

// Default parameters
fun greet(name: String, greeting = "Hello") {
    println!("{greeting}, {name}!")
}

// Generic functions
fun id<T>(x: T) -> T { x }

fun map<T, U>(list: [T], f: fun(T) -> U) -> [U] {
    list.iter().map(f).collect()
}

// Lambda expressions  
let inc = |x| x + 1
let mul = |x, y| x * y

// Mathematical functions
fun mean(numbers: [f64]) -> f64 {
    numbers.sum() / numbers.len() as f64
}

fun std_dev(data: Series) -> f64 {
    data.std().unwrap()
}
```

#### Pattern Matching
```rust
// Basic match
match value {
    0 => "zero",
    1 | 2 => "small", 
    n if n > 10 => "large",
    _ => "other"
}

// List patterns
match list {
    [] => "empty",
    [x] => "single element: {x}", 
    [x, y] => "pair: {x}, {y}",
    [head, ...tail] => "head: {head}, rest: {tail.len()} items",
    _ => "many"
}

// Tuple patterns
match point {
    (0, 0) => "origin",
    (x, 0) => "on x-axis at {x}",
    (0, y) => "on y-axis at {y}",
    (x, y) if x == y => "on diagonal",
    _ => "arbitrary point"
}

// Enum patterns with guards
match result {
    Ok(value) if value > 0 => process(value),
    Ok(_) => skip(),
    Err(e) if e.is_recoverable() => retry(),
    Err(e) => fail(e)
}
```

#### Control Flow
```rust
// If expressions
let status = if age >= 18 { "adult" } else { "minor" }

// When expressions (Swift-style)
when {
    x < 0 -> "negative",
    x == 0 -> "zero",
    x > 0 -> "positive"
}

// For loops with ranges
for i in 0..10 {
    println!("{i}")
}

// While loops
while condition {
    process()
}

// Loop with break value
let result = loop {
    if done() {
        break value
    }
    iterate()
}

// List comprehensions
let squares = [x * x for x in 1..10]
let evens = [x for x in numbers if x % 2 == 0]
let grid = [(x, y) for x in 0..3 for y in 0..3]
```

#### Error Handling
```rust
// Result type with ? operator
fun read_number(path: String) -> Result<i32, Error> {
    let content = read_file(path)?
    let number = content.parse()?
    Ok(number)
}

// Try-catch blocks
try {
    risky_operation()
} catch FileError(e) {
    handle_file_error(e)
} catch ParseError(e) {
    handle_parse_error(e)
} finally {
    cleanup()
}

// Panic with custom message
panic!("Unexpected state: {state}")

// Assertions
assert!(x > 0, "x must be positive")
assert_eq!(result, expected)
```

### 1.4 Collections and Iterators

```rust
// Arrays/Lists default to Series
let numbers = [1, 2, 3, 4, 5]  // -> Series
let matrix = [[1, 2], [3, 4]]  // -> DataFrame

// Explicit collections
let vec = Vec::from([1, 2, 3])
let map = HashMap::from([("a", 1), ("b", 2)])
let set = HashSet::from([1, 2, 3])

// Iterator chains
numbers
    |> filter(|x| x > 0)
    |> map(|x| x * 2)
    |> fold(0, |acc, x| acc + x)
```

### 1.5 String Interpolation

```rust
// Basic interpolation
let name = "Alice"
let greeting = "Hello, {name}!"

// Expression interpolation
let result = "The answer is {2 + 2}"

// Format specifiers
let pi = 3.14159
let formatted = "Pi to 2 places: {pi:.2}"

// Multi-line strings
let query = """
    SELECT * FROM users
    WHERE age > {min_age}
    ORDER BY name
"""
```

## 2. Grammar Reference

### 2.1 Formal Grammar (EBNF)

```ebnf
program         = item*
item            = function | struct_def | enum_def | trait_def | impl_block
                | actor_def | module_def | import_stmt | type_alias | const_def

// Function definitions
function        = attributes? visibility? 'fun' identifier 
                  generic_params? '(' params? ')' return_type? 
                  where_clause? (block | '=' expr)

params          = param (',' param)*
param           = pattern ':' type default_value?
default_value   = '=' expr
return_type     = '->' type

// Type definitions
struct_def      = visibility? 'struct' identifier generic_params? struct_body
struct_body     = '{' field (',' field)* '}'
field           = visibility? identifier ':' type

enum_def        = visibility? 'enum' identifier generic_params? enum_body
enum_body       = '{' variant (',' variant)* '}'
variant         = identifier ('(' type (',' type)* ')')?

trait_def       = visibility? 'trait' identifier generic_params? trait_body
trait_body      = '{' trait_item* '}'
trait_item      = function_sig | associated_type

impl_block      = 'impl' generic_params? type_path for_clause? impl_body
impl_body       = '{' impl_item* '}'
impl_item       = function | associated_const

// Actor definitions
actor_def       = visibility? 'actor' identifier generic_params? actor_body
actor_body      = '{' actor_state? receive_block '}'
actor_state     = field (',' field)*
receive_block   = 'receive' '{' message_handler (',' message_handler)* '}'
message_handler = pattern '=>' expr

// Expressions
expression      = assignment
assignment      = pipeline ('=' assignment)?
pipeline        = logical_or ('|>' pipeline)*
logical_or      = logical_and ('||' logical_and)*
logical_and     = equality ('&&' equality)*
equality        = comparison (('==' | '!=') comparison)*
comparison      = term (('>' | '>=' | '<' | '<=') term)*
term            = factor (('+' | '-') factor)*
factor          = unary (('*' | '/' | '%' | '**' | '//') unary)*
unary           = ('!' | '-' | 'await')? postfix
postfix         = primary ('.' IDENTIFIER | '[' expression ']' 
                | '(' arguments? ')' | '?' | '!')*

primary         = NUMBER | STRING | BOOLEAN | IDENTIFIER | '(' expression ')'
                | if_expr | match_expr | when_expr | for_expr | while_expr
                | loop_expr | lambda | array_expr | tuple_expr | record_expr
                | dataframe_literal | try_expr | async_block

// Lambda expressions
lambda          = '|' params? '|' (expr | block)
                | params '=>' (expr | block)

// Pattern matching
pattern         = literal_pattern | identifier_pattern | wildcard_pattern
                | tuple_pattern | array_pattern | struct_pattern
                | variant_pattern | range_pattern

match_expr      = 'match' expr '{' match_arm (',' match_arm)* '}'
match_arm       = pattern ('if' expr)? '=>' expr

// DataFrame literals
dataframe_literal = 'df!' '[' column_def (',' column_def)* ']'
column_def      = STRING ':' '[' expr (',' expr)* ']'
```

### 2.2 Operator Precedence

| Precedence | Operators | Associativity | Category |
|------------|-----------|---------------|----------|
| 1 | `.` `?.` `::` | Left | Member access |
| 2 | `()` `[]` | Left | Call, index |
| 3 | `!` `~` `-` (unary) `await` | Right | Unary |
| 4 | `**` | Right | Power |
| 5 | `*` `/` `%` `//` | Left | Multiplicative |
| 6 | `+` `-` | Left | Additive |
| 7 | `<<` `>>` | Left | Shift |
| 8 | `&` | Left | Bitwise AND |
| 9 | `^` | Left | Bitwise XOR |
| 10 | `\|` | Left | Bitwise OR |
| 11 | `==` `!=` `<` `<=` `>` `>=` | Left | Comparison |
| 12 | `is` `in` | Left | Type/membership |
| 13 | `&&` | Left | Logical AND |
| 14 | `\|\|` | Left | Logical OR |
| 15 | `..` `...` `..=` | None | Range |
| 16 | `??` | Right | Null coalescing |
| 17 | `\|>` | Left | Pipeline |
| 18 | `=` `+=` `-=` etc. | Right | Assignment |

### 2.3 Keywords (Reserved)

```
fun let var const if else when match for while loop break continue
return struct enum trait impl actor receive send ask async await
defer guard try catch throw import export module pub priv mut
type alias where in is as true false null
df col mean std quantile filter groupby agg sort select
```

## 3. Implementation Specification

### 3.1 Transpilation Architecture

```rust
// Multi-stage transformation with MIR for optimization
pub struct Transpiler {
    parser: RuchyParser,
    type_checker: TypeChecker,
    mir_gen: MirGenerator,
    optimizer: MirOptimizer,
    codegen: RustCodeGen,
}

impl Transpiler {
    pub fun transpile(source: &str) -> Result<String, Error> {
        // Parse to AST
        let ast = self.parser.parse(source)?;
        
        // Type inference and checking
        let typed_ast = self.type_checker.infer(ast)?;
        
        // Generate MIR for optimization
        let mir = self.mir_gen.lower(typed_ast)?;
        
        // Optimize MIR (DataFrame fusion, dead code elimination)
        let optimized_mir = self.optimizer.optimize(mir)?;
        
        // Generate Rust AST from optimized MIR
        let rust_ast = self.codegen.generate(optimized_mir)?;
        
        // Generate and format Rust source
        let rust_code = quote!(#rust_ast).to_string();
        rustfmt::format(rust_code)
    }
    
    // MIR-based transformation pipeline
    fun transform_mir(&self, mir: MirNode) -> MirNode {
        match mir {
            // DataFrame operations get lazy evaluation
            MirNode::DataFrameOp { op, input } => {
                MirNode::LazyDataFrameOp {
                    op,
                    input: Box::new(self.transform_mir(*input)),
                    fusion_candidates: self.find_fusion_opportunities(&op),
                }
            }
            // Actor messages get flow analysis
            MirNode::ActorSend { actor, msg } => {
                self.analyze_message_flow(actor, msg)
            }
            // Pipeline operators expand to method chains
            MirNode::Pipeline { expr, ops } => {
                ops.fold(expr, |acc, op| {
                    MirNode::MethodCall {
                        receiver: Box::new(acc),
                        method: op,
                    }
                })
            }
            _ => mir,
        }
    }
}

// NOTE: Direct syn::File generation exists for MVP only
// Full implementation requires MIR for optimization
```

### 3.2 DataFrame-First Design

Every collection operation defaults to Polars types:

```rust
// Collection type hierarchy
pub enum CollectionType {
    DataFrame(DataFrame),     // 2D tabular data
    LazyFrame(LazyFrame),    // Lazy evaluation
    Series(Series),          // 1D column data
    Vec(Vec<T>),            // Explicit Vec only
    HashMap(HashMap<K, V>),  // Explicit HashMap only
}

// Transformation rules
impl CollectionTransform {
    fun array_to_series(elements: Vec<Expr>) -> TokenStream {
        // [1, 2, 3] becomes Series
        quote! {
            ::polars::prelude::Series::new("", &[#(#elements),*])
        }
    }
    
    fun nested_array_to_dataframe(rows: Vec<Vec<Expr>>) -> TokenStream {
        // [[1, 2], [3, 4]] becomes DataFrame
        quote! {
            ::polars::prelude::df![
                "col0" => vec![#(#col0_elements),*],
                "col1" => vec![#(#col1_elements),*]
            ]
        }
    }
}
```

## 4. Parser Specification

### 4.1 Parser Architecture

Hand-written recursive descent with Pratt parsing for operators:

```rust
pub struct Parser<'src> {
    tokens: TokenStream<'src>,
    current: Token,
    peek: Token,
    
    // Error recovery
    errors: Vec<ParseError>,
    panic_mode: bool,
    
    // String interpolation context
    interpolation_stack: Vec<InterpolationContext>,
    
    // Comment attachment
    comments: CommentStream<'src>,
}

impl Parser<'_> {
    // Pratt parsing for expressions
    fun parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        
        loop {
            let op = match self.current_binop() {
                Some(op) => op,
                None => break,
            };
            
            let (left_bp, right_bp) = op.binding_power();
            if left_bp < min_bp {
                break;
            }
            
            self.advance();
            let right = self.parse_expr_bp(right_bp)?;
            
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span(left.span.start),
            };
        }
        
        Ok(left)
    }
}
```

### 4.2 Error Recovery

Synchronization points for graceful error recovery:

```rust
impl Parser<'_> {
    fun synchronize(&mut self) {
        self.panic_mode = false;
        
        while !self.is_at_end() {
            if self.previous().kind == Semicolon {
                return;
            }
            
            if self.is_sync_point() {
                return;
            }
            
            self.advance();
        }
    }
    
    fun is_sync_point(&self) -> bool {
        matches!(
            self.current.kind,
            Fun | Let | Type | Import | Export | If | For | Match
        )
    }
}
```

### 4.3 AST Desugaring

Pipeline operators expand during AST transformation:

```rust
impl AstDesugarer {
    fun visit_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Pipeline { left, op, right } => {
                // x |> f becomes f(x)
                *expr = Expr::Call {
                    callee: right.clone(),
                    args: vec![*left.clone()],
                    span: expr.span,
                };
                
                // Recursively desugar
                self.visit_expr(expr);
            }
            _ => self.walk_expr(expr),
        }
    }
}
```

## 5. Lexer Specification

### 5.1 Token Categories

```rust
pub enum TokenKind {
    // Keywords (31 total)
    Fun, Let, If, Else, Match, For, While, Loop,
    Break, Continue, Return, Import, Export,
    Actor, Receive, Send, Ask, Async, Await,
    Trait, Impl, Struct, Enum, Type, Where,
    True, False, Null,
    
    // Operators
    Plus, Minus, Star, Slash, Percent, Power,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not,
    Pipe, Arrow, FatArrow,
    
    // Delimiters
    LParen, RParen, LBracket, RBracket, LBrace, RBrace,
    Comma, Semi, Colon, Dot,
    
    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    
    // Identifiers
    Ident(String),
    
    // Special
    Eof,
}
```

### 5.2 Lexical Rules

```rust
impl Lexer<'src> {
    pub fun new(input: &'src str) -> Self {
        Lexer {
            input: input.as_bytes(),
            position: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    fun next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }
        
        let c = self.advance();
        
        match c {
            // Single-character tokens
            b'(' => self.make_token(LParen),
            b')' => self.make_token(RParen),
            b'[' => self.make_token(LBracket),
            b']' => self.make_token(RBracket),
            
            // Multi-character tokens
            b'=' => {
                if self.match_char(b'=') {
                    self.make_token(Eq)
                } else if self.match_char(b'>') {
                    self.make_token(FatArrow)
                } else {
                    self.make_token(Assign)
                }
            }
            
            // Numbers
            b'0'..=b'9' => self.number(),
            
            // Identifiers and keywords
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(),
            
            // Strings
            b'"' => self.string(),
            
            _ => self.error_token("Unexpected character"),
        }
    }
}
```

### 5.3 String Interpolation Lexing

```rust
impl Lexer<'_> {
    fun string(&mut self) -> Token {
        let mut parts = Vec::new();
        
        while !self.is_at_end() && !self.check(b'"') {
            if self.check(b'{') {
                // Emit string fragment
                if !parts.is_empty() {
                    self.emit_token(StringFragment(parts.clone()));
                }
                
                // Emit interpolation start
                self.emit_token(InterpolationStart);
                
                // Lex expression tokens
                self.lex_interpolation();
                
                // Emit interpolation end
                self.emit_token(InterpolationEnd);
                
                parts.clear();
            } else {
                parts.push(self.advance());
            }
        }
        
        self.consume(b'"', "Unterminated string");
        self.make_token(StringEnd)
    }
}
```

## 6. Script Capability Specification

### 6.1 Execution Modes

```rust
pub enum ExecutionMode {
    Script,      // .ruchy files
    Repl,        // Interactive mode
    Jupyter,     // Notebook cells
    Compiled,    // AOT compilation
    OneLiner,    // -e flag
}

// Mode detection
impl Runtime {
    fun detect_mode(args: &Args) -> ExecutionMode {
        match args {
            Args { eval: Some(_), .. } => ExecutionMode::OneLiner,
            Args { file: Some(f), .. } if f.ends_with(".ruchy") => ExecutionMode::Script,
            Args { repl: true, .. } => ExecutionMode::Repl,
            Args { compile: true, .. } => ExecutionMode::Compiled,
            _ => ExecutionMode::Repl,
        }
    }
}
```

### 6.2 REPL Features

```rust
pub struct Repl {
    interpreter: TreeWalkInterpreter,
    history: Vec<String>,
    bindings: HashMap<String, Value>,
    
    // DataFrame visualization
    df_printer: DataFramePrinter,
    
    // Completion engine
    completer: Completer,
}

impl Repl {
    pub async fun run(&mut self) -> Result<()> {
        loop {
            let input = self.read_line("ruchy> ")?;
            
            match self.parse_command(&input) {
                Command::Expr(expr) => {
                    let result = self.eval(expr)?;
                    self.display(result);
                }
                Command::Import(module) => {
                    self.import_module(module)?;
                }
                Command::Help => self.show_help(),
                Command::Exit => break,
            }
        }
        Ok(())
    }
}
```

## 7. Classes and Structs Specification

### 7.0 Value Types vs Reference Types (Swift Model)

Ruchy follows Swift's model exactly: **structs are value types**, **classes are reference types**.

#### Key Distinctions

| Feature | Struct (Value Type) | Class (Reference Type) |
|---------|--------------------|-----------------------|
| **Semantics** | Copied on assignment | Reference shared on assignment |
| **Mutability** | Requires `mutating` keyword for methods that modify | Methods can modify by default |
| **Inheritance** | ❌ Not supported | ✅ Single inheritance |
| **Initialization** | Automatic memberwise init | Must define init explicitly |
| **Deinitializer** | ❌ Not supported | ✅ `deinit` supported |
| **Memory** | Stack allocated (usually) | Heap allocated |
| **Equality** | Value equality (==) | Reference identity (===) or value equality (==) |
| **Thread Safety** | Inherently safer (copies) | Requires synchronization |
| **Performance** | Faster allocation/deallocation | Reference counting overhead |

#### Struct Semantics (Value Type)
```ruchy
struct Point {
    x: float,
    y: float

    // Methods that modify must be marked 'mutating'
    mutating fun move(dx: float, dy: float) {
        self.x += dx
        self.y += dy
    }

    // Non-mutating methods don't need special marking
    fun distance_from_origin() -> float {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

// Value semantics - assignment creates a copy
var p1 = Point { x: 0.0, y: 0.0 }
var p2 = p1  // p2 is a COPY of p1
p2.x = 10.0
assert(p1.x == 0.0)  // p1 unchanged
assert(p2.x == 10.0) // p2 modified

// Structs get automatic memberwise initializer
var p3 = Point(x: 5.0, y: 3.0)  // Free init
```

#### Class Semantics (Reference Type)
```ruchy
class Person {
    name: String
    age: i32

    // Must define initializer
    init(name: String, age: i32) {
        self.name = name
        self.age = age
    }

    // Methods can modify without 'mutating'
    fun have_birthday() {
        self.age += 1
    }

    // Optional deinitializer
    deinit {
        println(f"Person {self.name} being deallocated")
    }
}

// Reference semantics - assignment shares reference
let person1 = Person(name: "Alice", age: 30)
let person2 = person1  // person2 references SAME object
person2.age = 31
assert(person1.age == 31)  // Both see the change

// Identity comparison
assert(person1 === person2)  // Same instance
let person3 = Person(name: "Alice", age: 31)
assert(person1 !== person3)  // Different instances
```

### 7.0.1 Implementation Status

#### Structs - Runtime Status
**Current Implementation: 73% Complete (19/26 tests passing)**

✅ **Implemented**:
- Struct definitions with fields and types
- Struct instantiation using literal syntax
- Field access (dot notation)
- Nested structs
- String interpolation with struct fields
- Error handling for missing/extra fields

❌ **Not Implemented**:
- Value semantics (currently using reference semantics)
- Mutating keyword for methods
- Automatic memberwise initializer
- Copy-on-write optimization
- Stack allocation

#### Classes - Runtime Status
**Current Implementation: 35% Complete (6/17 tests passing)**

✅ **Implemented**:
- Class definition parsing and metadata storage
- Field declarations with types and defaults
- Constructor and method metadata storage

❌ **Not Implemented**:
- Reference semantics
- Class instantiation with `init`
- Instance methods
- Static methods
- Class inheritance with `super` calls
- Method overriding
- Deinitializers
- Reference identity (===) operator

### 7.1 Design Principles

Following Swift's model provides clear mental models:

1. **Use structs for value semantics**: Points, vectors, colors, most data models
2. **Use classes for reference semantics**: Shared state, inheritance hierarchies, identity matters
3. **Prefer structs by default**: Simpler, safer, more performant
4. **Classes when needed**: Inheritance, shared mutable state, Objective-C interop

### 7.2 Memory Model

#### Struct Memory (Value Types)
- Stack allocated when possible
- Inline storage in containers
- Copy-on-write for large values
- No reference counting

#### Class Memory (Reference Types)
- Always heap allocated
- Reference counted (ARC)
- Weak/unowned references supported
- Deinit called when count reaches zero

### 7.3 Transpilation Strategy

#### Structs → Rust
```rust
// Ruchy struct becomes Rust struct with Clone
#[derive(Clone, Copy)] // if all fields are Copy
#[derive(Clone)]       // otherwise
struct Point {
    x: f64,
    y: f64,
}
```

#### Classes → Rust
```rust
// Ruchy class becomes Rc/Arc wrapped struct
struct PersonData {
    name: String,
    age: i32,
}
type Person = Rc<RefCell<PersonData>>;  // single-threaded
type Person = Arc<Mutex<PersonData>>;   // multi-threaded
```

```rust
// Ruchy class syntax (reference type)
class DataProcessor {
    buffer: Vec<u8>
    capacity: usize

    // Constructor (Swift-style init)
    init(capacity: usize = 1024) {
        self.buffer = Vec::with_capacity(capacity)
        self.capacity = capacity
    }

    // Instance method (can modify without 'mutating')
    fun process(data: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(data)
        Ok(())
    }
    
    // Associated function (static method)
    fun from_file(path: &str) -> Result<Self> {
        let data = std::fs::read(path)?;
        let mut proc = Self::new(data.len());
        proc.process(&data)?;
        Ok(proc)
    }
}

// Transpiles to:
pub struct DataProcessor {
    buffer: Vec<u8>,
    capacity: usize,
}

impl DataProcessor {
    pub fun new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fun process(&mut self, data: &[u8]) -> Result<()> {
        self.buffer.extend_from_slice(data);
        Ok(())
    }
    
    pub fun from_file(path: &str) -> Result<Self> {
        let data = std::fs::read(path)?;
        let mut proc = Self::new(data.len());
        proc.process(&data)?;
        Ok(proc)
    }
}

impl Default for DataProcessor {
    fun default() -> Self {
        Self::new(1024)
    }
}
```

### 7.2 Trait Implementation

```rust
// Ruchy trait syntax - explicit implementation
class Point {
    x: f64,
    y: f64,
    
    impl Display {
        fun fmt(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "({}, {})", self.x, self.y)
        }
    }
    
    impl PartialEq {
        fun eq(&self, other: &Self) -> bool {
            self.x == other.x && self.y == other.y
        }
    }
}

// Auto-derive via attributes
@[derive(Debug, Clone, PartialEq)]
class Vector3 {
    x: f64,
    y: f64,
    z: f64,
}
```

### 7.3 Properties (Getters/Setters)

```rust
class Temperature {
    celsius: f64,
    
    // Property with getter/setter
    property fahrenheit: f64 {
        get => self.celsius * 9.0/5.0 + 32.0,
        set(value) => self.celsius = (value - 32.0) * 5.0/9.0
    }
    
    // Read-only property
    property kelvin: f64 {
        get => self.celsius + 273.15
    }
}

// Transpiles to:
impl Temperature {
    pub fun fahrenheit(&self) -> f64 {
        self.celsius * 9.0/5.0 + 32.0
    }
    
    pub fun set_fahrenheit(&mut self, value: f64) {
        self.celsius = (value - 32.0) * 5.0/9.0
    }
    
    pub fun kelvin(&self) -> f64 {
        self.celsius + 273.15
    }
}
```

### 7.4 Generic Classes

```rust
class Cache<K: Hash + Eq, V> {
    map: HashMap<K, V>,
    capacity: usize,
    
    new(capacity: usize) {
        Self {
            map: HashMap::with_capacity(capacity),
            capacity,
        }
    }
    
    fun get(&self, key: &K) -> Option<&V> {
        self.map.get(key)
    }
    
    fun insert(&mut self, key: K, value: V) where V: Clone {
        if self.map.len() >= self.capacity {
            self.evict_oldest();
        }
        self.map.insert(key, value);
    }
}
```

### 7.5 Extension Methods

```rust
// Extend existing types with new methods
extension String {
    fun is_palindrome(&self) -> bool {
        let clean = self.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_lowercase())
            .collect::<String>();
        clean == clean.chars().rev().collect::<String>()
    }
}

// Transpiles to trait with blanket impl
trait StringExt {
    fun is_palindrome(&self) -> bool;
}

impl StringExt for String {
    fun is_palindrome(&self) -> bool {
        let clean = self.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_lowercase().to_string())
            .collect::<String>();
        clean == clean.chars().rev().collect::<String>()
    }
}
```

### 7.6 Protocols (Trait Aliases)

```rust
// Define protocol as trait combination
protocol Numeric = Add + Sub + Mul + Div + Clone + PartialOrd;

protocol Serializable = Serialize + Deserialize;

class Matrix<T: Numeric> {
    data: Vec<Vec<T>>,
    
    fun multiply(&self, other: &Self) -> Self 
    where T: Default + Sum {
        // Matrix multiplication
    }
}
```

### 7.7 Companion Objects

```rust
class User {
    id: u64,
    name: String,
    
    // Companion object for associated items
    companion {
        const TABLE_NAME: &str = "users";
        let mut id_counter: AtomicU64 = AtomicU64::new(1);
        
        fun next_id() -> u64 {
            Self::id_counter.fetch_add(1, Ordering::SeqCst)
        }
        
        fun from_row(row: DatabaseRow) -> Result<User> {
            Ok(User {
                id: row.get("id")?,
                name: row.get("name")?,
            })
        }
    }
}

// Usage:
let id = User::next_id();
let table = User::TABLE_NAME;
```

### 7.8 Sealed Classes (Sum Types)

```rust
// Algebraic data types with methods
sealed class Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
    
    fun area(&self) -> f64 {
        match self {
            Circle { radius } => PI * radius * radius,
            Rectangle { width, height } => width * height,
            Triangle { base, height } => 0.5 * base * height,
        }
    }
}

// Transpiles to enum with methods
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    fun area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
}
```

### 7.9 Data Classes

```rust
// Automatic implementation of common traits
@[data]
class Point3D {
    x: f64,
    y: f64,
    z: f64,
}

// Generates:
// - Constructor
// - Debug, Clone, PartialEq, Eq, Hash
// - Builder pattern
// - Destructuring support

let p = Point3D { x: 1.0, y: 2.0, z: 3.0 };
let Point3D { x, y, z } = p;  // Destructuring
```

### 7.10 Visibility Modifiers

```rust
class BankAccount {
    pub number: String,        // Public
    balance: f64,              // Private (default)
    pub(crate) branch: String, // Crate-visible
    
    // Public method
    pub fun deposit(&mut self, amount: f64) {
        self.balance += amount;
    }
    
    // Private method
    fun validate_amount(&self, amount: f64) -> bool {
        amount > 0.0 && amount <= 1_000_000.0
    }
}
```

### 7.11 Memory Management

```rust
// Classes use Rust's ownership model
class Resource {
    handle: FileHandle,
    
    // Move semantics by default
    fun transfer(self) -> FileHandle {
        self.handle  // Ownership transferred
    }
    
    // Borrowing
    fun read(&self) -> &[u8] {
        &self.handle.buffer
    }
    
    // Mutable borrowing
    fun write(&mut self, data: &[u8]) {
        self.handle.write(data)
    }
    
    // Automatic cleanup via Drop
    impl Drop {
        fun drop(&mut self) {
            self.handle.close();
        }
    }
}
```

### 7.12 Class Transpilation Rules

```rust
impl ClassTranspiler {
    fun transpile_class(&self, class: &ClassDef) -> TokenStream {
        let struct_def = self.generate_struct(class);
        let impl_blocks = self.generate_impls(class);
        let trait_impls = self.generate_trait_impls(class);
        
        quote! {
            #struct_def
            #(#impl_blocks)*
            #(#trait_impls)*
        }
    }
    
    fun handle_constructor(&self, class: &ClassDef) -> TokenStream {
        // 'new' method becomes associated function
        if let Some(ctor) = class.find_method("new") {
            self.transform_constructor(ctor)
        } else {
            // Generate default constructor
            self.generate_default_new(class)
        }
    }
    
    fun handle_properties(&self, prop: &Property) -> TokenStream {
        let getter = self.generate_getter(prop);
        let setter = prop.setter.as_ref()
            .map(|s| self.generate_setter(prop, s));
        
        quote! {
            #getter
            #setter
        }
    }
}
```

## 8. Functional Programming Specification

### 8.1 Core Functional Primitives

Ruchy provides first-class functions with zero allocation overhead through aggressive inlining and monomorphization.

```rust
// Function types are structural, not nominal
type Predicate<T> = fun(T) -> bool
type Transform<A, B> = fun(A) -> B
type Reducer<T, Acc> = fun(Acc, T) -> Acc

// Higher-order functions with type inference
fun map<T, U>(list: [T], f: fun(T) -> U) -> [U] {
    list.iter().map(f).collect()
}

// Currying via closures - zero cost when inlined
fun add(x: i32) -> fun(i32) -> i32 {
    |y| x + y
}

let add5 = add(5)
let result = add5(10)  // 15
```

### 8.2 Algebraic Data Types

```rust
// Sum types with pattern matching
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Product types via tuples and records
type Point2D = (f64, f64)
type Person = { name: String, age: u32 }

// Recursive types
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

// Pattern matching with guards and bindings
fun length<T>(list: List<T>) -> usize {
    match list {
        Nil => 0,
        Cons(_, tail) => 1 + length(*tail),
    }
}
```

### 8.3 Immutability by Default

```rust
// Immutable bindings
let x = 42       // Immutable
var y = 42       // Mutable (explicit)

// Persistent data structures via Rc/Arc
class PersistentVector<T> {
    root: Arc<Node<T>>,
    
    fun push(&self, value: T) -> Self {
        // Structural sharing - O(log n)
        Self {
            root: self.root.add(value),
        }
    }
}

// Copy-on-write semantics for efficiency
fun update_field(record: Person) -> Person {
    { ...record, age: record.age + 1 }  // Only age cloned
}
```

### 8.4 Lazy Evaluation

```rust
// Lazy sequences via iterators
lazy val fibonacci: Iterator<u64> = {
    Iterator::unfold((0, 1), |(a, b)| {
        Some((*a, (*b, *a + *b)))
    })
}

// Thunks for deferred computation
class Lazy<T> {
    cell: OnceCell<T>,
    init: Box<dyn FnOnce() -> T>,
    
    fun force(&self) -> &T {
        self.cell.get_or_init(|| (self.init)())
    }
}

// Stream processing with lazy transformations
let result = (0..)
    |> filter(|x| x % 2 == 0)
    |> map(|x| x * x)
    |> take(10)
    |> collect()
```

### 8.5 Monadic Composition

```rust
// Option monad
impl<T> Option<T> {
    fun bind<U>(self, f: fun(T) -> Option<U>) -> Option<U> {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }
    
    fun map<U>(self, f: fun(T) -> U) -> Option<U> {
        self.bind(|x| Some(f(x)))
    }
}

// Result monad for error handling
impl<T, E> Result<T, E> {
    fun and_then<U>(self, f: fun(T) -> Result<U, E>) -> Result<U, E> {
        match self {
            Ok(x) => f(x),
            Err(e) => Err(e),
        }
    }
}

// Do-notation via ? operator
fun divide_and_add(x: f64, y: f64, z: f64) -> Result<f64, String> {
    let quotient = divide(x, y)?
    let sum = add(quotient, z)?
    Ok(sum)
}
```

### 8.6 Function Composition

```rust
// Composition operators
infix fun <A, B, C> (>>)(f: fun(A) -> B, g: fun(B) -> C) -> fun(A) -> C {
    |x| g(f(x))
}

infix fun <A, B, C> (<<)(g: fun(B) -> C, f: fun(A) -> B) -> fun(A) -> C {
    |x| g(f(x))
}

// Point-free style
let process = parse >> validate >> transform >> serialize

// Kleisli composition for monadic functions
infix fun <A, B, C> (>=>)(
    f: fun(A) -> Result<B, E>,
    g: fun(B) -> Result<C, E>
) -> fun(A) -> Result<C, E> {
    |x| f(x).and_then(g)
}
```

### 8.7 Partial Application

```rust
// Automatic currying for multi-parameter functions
fun fold<T, Acc>(list: [T], init: Acc, f: fun(Acc, T) -> Acc) -> Acc {
    list.iter().fold(init, f)
}

// Partial application via underscore
let sum = fold(_, 0, |acc, x| acc + x)
let total = sum([1, 2, 3, 4, 5])  // 15

// Operator sections
let increment = (_ + 1)
let doubled = map(numbers, _ * 2)
```

### 8.8 Tail Call Optimization

```rust
// TCO via loop transformation
#[tailrec]
fun factorial(n: u64, acc: u64 = 1) -> u64 {
    if n == 0 {
        acc
    } else {
        factorial(n - 1, n * acc)  // Becomes loop
    }
}

// Trampoline for mutual recursion
enum Trampoline<T> {
    Done(T),
    More(Box<dyn FnOnce() -> Trampoline<T>>),
}

fun run_trampoline<T>(mut t: Trampoline<T>) -> T {
    loop {
        match t {
            Done(x) => return x,
            More(f) => t = f(),
        }
    }
}
```

### 8.9 Effect System (Future)

```rust
// Algebraic effects for pure functional I/O
effect IO {
    fun print(s: String)
    fun read() -> String
}

effect State<S> {
    fun get() -> S
    fun put(s: S)
}

// Effect handlers
fun handle_io<T>(comp: T with IO) -> T {
    handle comp {
        print(s) => { 
            println!("{}", s);
            resume(())
        },
        read() => {
            let input = std::io::stdin().read_line();
            resume(input)
        }
    }
}

// Effect composition
fun program() with IO + State<i32> {
    let name = perform read()
    perform print("Hello, {name}")
    let count = perform get()
    perform put(count + 1)
}
```

### 8.10 Optics (Lenses & Prisms)

```rust
// Lenses for nested record updates
class Lens<S, A> {
    get: fun(&S) -> &A,
    set: fun(S, A) -> S,
    
    fun modify(&self, s: S, f: fun(A) -> A) -> S {
        let value = (self.get)(&s);
        (self.set)(s, f(value.clone()))
    }
    
    // Lens composition
    fun compose<B>(self, other: Lens<A, B>) -> Lens<S, B> {
        Lens {
            get: |s| other.get(self.get(s)),
            set: |s, b| self.set(s, other.set(self.get(s).clone(), b)),
        }
    }
}

// Usage
let address_lens = Lens::new(|p: &Person| &p.address);
let city_lens = Lens::new(|a: &Address| &a.city);
let person_city = address_lens.compose(city_lens);

let updated = person_city.set(person, "New York");
```

### 8.11 Type Classes (via Traits)

```rust
// Functor type class
trait Functor<F<_>> {
    fun map<A, B>(fa: F<A>, f: fun(A) -> B) -> F<B>
}

impl Functor for Option {
    fun map<A, B>(fa: Option<A>, f: fun(A) -> B) -> Option<B> {
        match fa {
            Some(a) => Some(f(a)),
            None => None,
        }
    }
}

// Monad type class
trait Monad<M<_>>: Functor<M> {
    fun pure<A>(a: A) -> M<A>
    fun bind<A, B>(ma: M<A>, f: fun(A) -> M<B>) -> M<B>
}

// Traverse and sequence
trait Traversable<T<_>>: Functor<T> {
    fun traverse<F<_>: Applicative, A, B>(
        ta: T<A>,
        f: fun(A) -> F<B>
    ) -> F<T<B>>
}
```

### 8.12 Memoization

```rust
// Automatic memoization for pure functions
#[memoize]
fun fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Manual memoization with cache control
class Memoized<K: Hash + Eq, V: Clone> {
    cache: DashMap<K, V>,
    compute: Box<dyn Fn(K) -> V>,
    
    fun call(&self, key: K) -> V {
        self.cache.entry(key.clone())
            .or_insert_with(|| (self.compute)(key))
            .clone()
    }
}
```

### 8.13 Functional Data Transformations

```rust
// Transducers for composable transformations
type Transducer<A, B, R> = fun(fun(R, B) -> R) -> fun(R, A) -> R

fun mapping<A, B, R>(f: fun(A) -> B) -> Transducer<A, B, R> {
    |reducer| |acc, item| reducer(acc, f(item))
}

fun filtering<A, R>(pred: fun(&A) -> bool) -> Transducer<A, A, R> {
    |reducer| |acc, item| {
        if pred(&item) {
            reducer(acc, item)
        } else {
            acc
        }
    }
}

// Composition of transducers
let xform = mapping(|x| x * 2) >> filtering(|x| x > 10)
let result = transduce(xform, vec::push, vec![], input)
```

### 8.14 Transpilation Strategy

```rust
impl FunctionalTranspiler {
    fun transpile_closure(&self, closure: &Closure) -> TokenStream {
        match closure.captures.len() {
            0 => {
                // Zero captures: fun pointer
                quote! { #body }
            }
            n if n <= 3 => {
                // Small captures: stack closure
                quote! { move |#params| #body }
            }
            _ => {
                // Large captures: Box<dyn Fn>
                quote! { Box::new(move |#params| #body) }
            }
        }
    }
    
    fun optimize_tail_call(&self, func: &Function) -> TokenStream {
        if func.is_tail_recursive() {
            // Transform to loop
            self.generate_loop_form(func)
        } else {
            self.standard_codegen(func)
        }
    }
    
    fun inline_hof(&self, call: &HigherOrderCall) -> TokenStream {
        if call.is_monomorphic() && call.closure.is_small() {
            // Inline closure at call site
            self.inline_expansion(call)
        } else {
            // Dynamic dispatch
            self.trait_object_call(call)
        }
    }
}
```

## 9. Interpreter Specification

### 9.1 Tree-Walk Interpreter Architecture

The interpreter provides immediate execution for REPL and rapid prototyping, preceding transpilation.

```rust
pub struct Interpreter {
    globals: Environment,
    locals: Stack<Environment>,
    call_stack: Stack<CallFrame>,
    heap: Arena<Value>,
}

pub enum Value {
    // Primitives - stack allocated
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    
    // References - heap allocated
    String(ArenaRef<String>),
    List(ArenaRef<Vec<Value>>),
    DataFrame(ArenaRef<DataFrame>),
    Function(ArenaRef<Closure>),
    Object(ArenaRef<HashMap<String, Value>>),
    
    // Special
    Null,
    NativeFunction(fn(&mut Interpreter, Vec<Value>) -> Result<Value>),
}
```

### 9.2 Execution Model

```rust
impl Interpreter {
    pub fun eval(&mut self, node: &AstNode) -> Result<Value> {
        match node {
            // Direct evaluation - no allocation
            AstNode::Literal(lit) => Ok(self.eval_literal(lit)),
            AstNode::Identifier(name) => self.lookup_variable(name),
            
            // Binary operations - stack-based
            AstNode::Binary { op, left, right } => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                self.apply_binary_op(op, l, r)
            }
            
            // Control flow - maintains stack discipline
            AstNode::If { cond, then, else_ } => {
                if self.eval(cond)?.is_truthy() {
                    self.eval(then)
                } else {
                    else_.map_or(Ok(Value::Null), |e| self.eval(e))
                }
            }
            
            // Function calls - new stack frame
            AstNode::Call { callee, args } => {
                let func = self.eval(callee)?;
                let arg_vals = args.iter()
                    .map(|a| self.eval(a))
                    .collect::<Result<Vec<_>>>()?;
                self.call_function(func, arg_vals)
            }
            
            _ => self.eval_complex(node),
        }
    }
    
    fun call_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value> {
        match func {
            Value::Function(closure_ref) => {
                let closure = self.heap.get(closure_ref);
                
                // Push new call frame
                self.call_stack.push(CallFrame {
                    return_addr: self.pc,
                    locals: Environment::new(),
                });
                
                // Bind parameters
                for (param, arg) in closure.params.iter().zip(args) {
                    self.locals.last_mut().unwrap()
                        .define(param.clone(), arg);
                }
                
                // Execute body
                let result = self.eval(&closure.body)?;
                
                // Pop call frame
                self.call_stack.pop();
                Ok(result)
            }
            Value::NativeFunction(f) => f(self, args),
            _ => Err(RuntimeError::NotCallable),
        }
    }
}
```

### 9.3 Memory Management

```rust
pub struct Arena<T> {
    chunks: Vec<Box<[MaybeUninit<T>; 1024]>>,
    current: usize,
    gc_threshold: usize,
}

impl<T> Arena<T> {
    pub fun alloc(&mut self, value: T) -> ArenaRef<T> {
        if self.should_gc() {
            self.collect_garbage();
        }
        
        let ref_ = self.next_slot();
        self.chunks[ref_.chunk][ref_.index].write(value);
        ref_
    }
    
    fun collect_garbage(&mut self) {
        // Mark phase - trace from roots
        let mut marked = BitSet::new(self.capacity());
        self.mark_roots(&mut marked);
        
        // Sweep phase - compact live objects
        self.compact_chunks(marked);
    }
}
```

### 9.4 Optimization Strategies

```rust
pub struct OptimizingInterpreter {
    base: Interpreter,
    inline_cache: HashMap<CallSite, Value>,
    hot_paths: HashMap<BlockId, u32>,
    jit_threshold: u32,
}

impl OptimizingInterpreter {
    fun eval_with_optimizations(&mut self, node: &AstNode) -> Result<Value> {
        // Inline caching for method dispatch
        if let AstNode::MethodCall { receiver, method, .. } = node {
            if let Some(cached) = self.inline_cache.get(&(receiver, method)) {
                return Ok(cached.clone());
            }
        }
        
        // Hot path detection for JIT candidates
        if let AstNode::Block { id, .. } = node {
            let count = self.hot_paths.entry(*id).or_insert(0);
            *count += 1;
            
            if *count > self.jit_threshold {
                self.compile_to_bytecode(node)?;
            }
        }
        
        self.base.eval(node)
    }
}
```

### 9.5 DataFrame Operations

```rust
impl Interpreter {
    fun eval_dataframe_ops(&mut self, df: &DataFrame, ops: &[DfOp]) -> Result<Value> {
        let mut result = df.clone();
        
        for op in ops {
            result = match op {
                DfOp::Filter(predicate) => {
                    let mask = self.eval_predicate(predicate, &result)?;
                    result.filter(mask)?
                }
                DfOp::Select(columns) => {
                    result.select(columns)?
                }
                DfOp::GroupBy(keys) => {
                    result.group_by(keys)?
                }
                DfOp::Agg(aggregations) => {
                    self.apply_aggregations(result, aggregations)?
                }
                _ => return Err(RuntimeError::UnsupportedOperation),
            };
        }
        
        Ok(Value::DataFrame(self.heap.alloc(result)))
    }
}
```

### 9.6 REPL Integration

```rust
pub struct ReplInterpreter {
    interpreter: Interpreter,
    history: Vec<(String, Value)>,
    bindings_snapshot: Environment,
}

impl ReplInterpreter {
    pub fun eval_line(&mut self, input: &str) -> Result<ReplOutput> {
        // Parse with error recovery
        let ast = match parse_with_recovery(input) {
            Ok(ast) => ast,
            Err(e) => return Ok(ReplOutput::ParseError(e)),
        };
        
        // Type inference for better errors
        let typed_ast = self.infer_types(&ast)?;
        
        // Execute with timing
        let start = Instant::now();
        let result = self.interpreter.eval(&typed_ast)?;
        let elapsed = start.elapsed();
        
        // Update history
        self.history.push((input.to_string(), result.clone()));
        
        Ok(ReplOutput::Success {
            value: result,
            elapsed,
            type_info: self.type_of(&result),
        })
    }
    
    pub fun snapshot(&self) -> Environment {
        self.interpreter.globals.clone()
    }
    
    pub fun restore(&mut self, snapshot: Environment) {
        self.interpreter.globals = snapshot;
    }
}
```

### 9.7 Native Function Interface

```rust
pub fun register_native_functions(interpreter: &mut Interpreter) {
    // Math functions
    interpreter.define_native("sin", |_, args| {
        match args.as_slice() {
            [Value::Float(x)] => Ok(Value::Float(x.sin())),
            _ => Err(RuntimeError::InvalidArguments),
        }
    });
    
    // DataFrame constructors
    interpreter.define_native("df", |interp, args| {
        let df = DataFrame::from_values(args)?;
        Ok(Value::DataFrame(interp.heap.alloc(df)))
    });
    
    // I/O functions
    interpreter.define_native("print", |_, args| {
        for arg in args {
            print!("{}", arg);
        }
        println!();
        Ok(Value::Null)
    });
}
```

### 9.8 Error Handling

```rust
pub enum RuntimeError {
    TypeError { expected: Type, found: Type },
    NameError { name: String },
    IndexError { index: i64, length: usize },
    StackOverflow { depth: usize },
    DivisionByZero,
    
    // With stack trace
    WithTrace {
        error: Box<RuntimeError>,
        trace: Vec<StackFrame>,
    },
}

impl Interpreter {
    fun capture_stack_trace(&self) -> Vec<StackFrame> {
        self.call_stack.iter().map(|frame| {
            StackFrame {
                function: frame.function_name.clone(),
                line: frame.source_location.line,
                column: frame.source_location.column,
            }
        }).collect()
    }
}
```

### 9.9 Performance Characteristics

```rust
// Benchmarked operations per second on reference hardware
pub struct InterpreterPerformance {
    arithmetic_ops: u64,      // 50M ops/sec
    function_calls: u64,      // 5M calls/sec
    object_allocations: u64,  // 10M allocs/sec
    gc_pause_p99: Duration,   // <1ms
    startup_time: Duration,   // <5ms
}

// Memory usage
pub struct MemoryProfile {
    base_overhead: usize,     // 2MB interpreter state
    per_value: usize,        // 24 bytes enum + heap
    gc_overhead: f64,        // 1.2x live set
    max_stack_depth: usize,  // 10,000 frames
}
```

### 9.10 Transition to Compilation

```rust
impl Interpreter {
    pub fun should_compile(&self, metrics: &ExecutionMetrics) -> bool {
        metrics.execution_count > 100 ||
        metrics.total_time > Duration::from_millis(10) ||
        metrics.contains_hot_loop
    }
    
    pub fun extract_type_profile(&self) -> TypeProfile {
        // Collect runtime type information for optimization
        TypeProfile {
            monomorphic_calls: self.collect_monomorphic_sites(),
            common_types: self.histogram_types(),
            allocation_sites: self.heap.hot_allocations(),
        }
    }
}
```

## 10. Unary Operator Specification

### 10.1 Operator Inventory

Three unary operators aligned with high-level scripting semantics:

```rust
pub enum UnaryOp {
    // Arithmetic
    Negate,     // -x → arithmetic negation
    
    // Logical  
    Not,        // !x → logical negation
    
    // Control
    Await,      // await x → suspend until ready
}

// Postfix operators (separate category)
pub enum PostfixOp {
    Try,        // x? → error propagation
    Optional,   // x?.field → optional chaining
}
```

### 10.2 Design Rationale

Memory operators (`&`, `&mut`, `*`) are **compiler-internal** transformations, not surface syntax. Escape analysis determines borrowing automatically:

```rust
// Ruchy source
let y = process(x)

// Transpiler determines based on escape analysis:
let y = process(&x)     // if x escapes
let y = process(x)      // if x moves
```

Bitwise operations use **method syntax** for clarity:

```rust
// Not: ~x (C-style)
// But: x.bit_not() (method)

x.bit_not()
x.bit_and(y)
x.bit_or(y)
```

### 10.3 Precedence and Associativity

Unary operators at precedence level 11, right-associative:

```rust
-!x  parses as  -(!(x))
await -x  parses as  await (-x)
```

Postfix operators at precedence level 12, left-associative:

```rust
x?.field  parses as  (x?).field
x?[0]  parses as  (x?)[0]
```

### 10.4 Type-Directed Semantics

```rust
impl UnaryOp {
    fun type_check(&self, operand: Type) -> Result<Type, TypeError> {
        match (self, operand) {
            // Arithmetic negation
            (Negate, Type::Int(n)) => Ok(Type::Int(n)),
            (Negate, Type::Float(n)) => Ok(Type::Float(n)),
            (Negate, Type::Unsigned(_)) => Err(TypeError::CannotNegateUnsigned),
            
            // Logical negation (bool only)
            (Not, Type::Bool) => Ok(Type::Bool),
            (Not, _) => Err(TypeError::NotRequiresBool),
            
            // Await (futures only)
            (Await, Type::Future(inner)) => Ok(*inner),
            (Await, _) => Err(TypeError::AwaitRequiresFuture),
        }
    }
}
```

### 10.5 Transpilation Rules

```rust
impl UnaryTranspiler {
    fun transpile(&self, op: UnaryOp, operand: &Expr) -> TokenStream {
        let operand_tokens = self.transpile_expr(operand);
        
        match op {
            Negate => quote! { -(#operand_tokens) },
            Not => quote! { !(#operand_tokens) },
            Await => quote! { (#operand_tokens).await },
        }
    }
    
    fun transpile_postfix(&self, op: PostfixOp, operand: &Expr) -> TokenStream {
        let operand_tokens = self.transpile_expr(operand);
        
        match op {
            Try => quote! { (#operand_tokens)? },
            Optional => self.desugar_optional_chain(operand),
        }
    }
}
```

### 10.6 Memory Operation Inference

Compiler internally adds borrowing based on usage analysis:

```rust
impl EscapeAnalyzer {
    fun infer_borrowing(&self, expr: &Expr, context: &Context) -> BorrowKind {
        match self.analyze_lifetime(expr, context) {
            Lifetime::Local => BorrowKind::None,        // Move/copy
            Lifetime::Escapes => BorrowKind::Shared,    // &
            Lifetime::Mutated => BorrowKind::Mutable,   // &mut
        }
    }
}

// Ruchy: data.process()
// Rust: (&data).process()  // if data used later
// Rust: data.process()      // if data consumed
```

### 10.7 Constant Folding

```rust
impl ConstantFolder {
    fun fold_unary(&self, op: UnaryOp, val: &Value) -> Option<Value> {
        match (op, val) {
            (Negate, Value::Int(n)) => Some(Value::Int(-n)),
            (Negate, Value::Float(f)) => Some(Value::Float(-f)),
            (Not, Value::Bool(b)) => Some(Value::Bool(!b)),
            (Await, _) => None, // Runtime only
        }
    }
}
```

### 10.8 Error Messages

```rust
impl UnaryError {
    fun diagnostic(&self) -> Diagnostic {
        match self {
            Self::UnsignedNegation { type_ } => {
                Diagnostic::error("cannot negate unsigned type")
                    .with_note(format!("{} is always non-negative", type_))
                    .with_suggestion("cast to signed type: -(x as i32)")
            }
            Self::NotRequiresBool { type_ } => {
                Diagnostic::error(format!("! operator requires bool, found {}", type_))
                    .with_suggestion("for bitwise NOT, use: x.bit_not()")
            }
            _ => self.default_diagnostic(),
        }
    }
}
```

### 10.9 Optimization

```rust
impl MirOptimizer {
    fun optimize_unary(&mut self, op: UnaryOp, operand: MirExpr) -> MirExpr {
        match (op, &operand) {
            // Double negation elimination
            (Negate, MirExpr::Unary(Negate, inner)) => *inner.clone(),
            (Not, MirExpr::Unary(Not, inner)) => *inner.clone(),
            
            // Constant propagation
            (op, MirExpr::Const(val)) => {
                match self.fold_unary(op, val) {
                    Some(result) => MirExpr::Const(result),
                    None => MirExpr::Unary(op, Box::new(operand)),
                }
            }
            
            _ => MirExpr::Unary(op, Box::new(operand)),
        }
    }
}
```

### 10.10 Performance Characteristics

```rust
pub struct UnaryPerformance {
    // Single CPU instruction
    negate: Duration,    // 1 cycle for int/float
    not: Duration,       // 1 cycle for bool
    
    // Context switch possible
    await: Duration,     // 10-1000ns depending on executor
}

// Memory operations (compiler-inserted) costs:
// & reference: 0 cycles (compile-time)
// * dereference: 1-300 cycles (cache-dependent)
```

## 11. Systems Operations Specification

### 11.1 File I/O Operations

Zero-cost abstractions over Rust's std::fs with automatic resource management:

```rust
// Simple file operations
let content = read_file("data.txt")?
write_file("output.txt", content)?

// Streaming operations
for line in read_lines("large.csv")? {
    process(line?)
}

// Automatic resource cleanup via RAII
with_file("data.bin", "r") { |file|
    let header = file.read_bytes(256)?
    process_header(header)
}  // File closed automatically
```

### 11.2 Process Management

```rust
// Command execution with builder pattern
let output = Command("git")
    .args(["status", "--short"])
    .env("GIT_PAGER", "")
    .capture()?

// Async process spawning
let proc = spawn_async("server") {
    env: { "PORT": "8080" },
    stdout: Pipe,
    stderr: Inherit,
}

let output = proc.wait_with_output().await?

// Process pipelines
let result = pipeline! {
    Command("cat", "data.txt") |
    Command("grep", "error") |
    Command("wc", "-l")
}
```

### 11.3 Network Operations

```rust
// HTTP client (reqwest backend)
let response = http::get("https://api.example.com/data").await?
let json: DataFrame = response.json().await?

// TCP operations
let listener = TcpListener::bind("127.0.0.1:8080")?
for stream in listener.incoming() {
    spawn_actor(handle_connection(stream?))
}

// UDP operations
let socket = UdpSocket::bind("0.0.0.0:34254")?
socket.send_to(b"hello", "127.0.0.1:8080")?
```

### 11.4 Environment and Configuration

```rust
// Environment variables
let path = env::var("PATH").unwrap_or("/usr/bin")
env::set_var("RUST_LOG", "debug")

// Configuration files (TOML/JSON/YAML)
let config = Config::from_file("config.toml")?
let port: u16 = config.get("server.port").unwrap_or(8080)

// Command-line arguments
let args = Args::parse()
if args.verbose {
    set_log_level(LogLevel::Debug)
}
```

### 11.5 Filesystem Operations

```rust
// Path manipulation
let path = Path::new("/usr/local")
    .join("bin")
    .with_extension("exe")

// Directory operations
create_dir_all("output/logs")?
for entry in read_dir(".")? {
    let entry = entry?
    if entry.is_file() && entry.extension() == "txt" {
        process_text_file(entry.path())?
    }
}

// File metadata
let metadata = file_metadata("data.bin")?
println!("Size: {} bytes", metadata.size())
println!("Modified: {}", metadata.modified()?)

// Glob patterns
for path in glob("**/*.rs")? {
    lint_rust_file(path?)?
}
```

### 11.6 Inter-Process Communication

```rust
// Unix sockets
let socket = UnixSocket::bind("/tmp/ruchy.sock")?
socket.listen(128)?

// Named pipes (FIFOs)
create_fifo("/tmp/ruchy_pipe", 0o644)?
let pipe = open_fifo("/tmp/ruchy_pipe", Read)?

// Shared memory
let shm = SharedMemory::create("ruchy_shm", 1024 * 1024)?
let data = shm.as_slice_mut()
data[0..4].copy_from_slice(&[1, 2, 3, 4])

// Message queues via actors
actor LogCollector {
    receive {
        LogMessage(level, text) => {
            append_log(level, text)?
        }
    }
}
```

### 11.7 System Information

```rust
// CPU information
let cpu_count = sys::cpu_count()
let cpu_usage = sys::cpu_usage()  // 0.0 - 1.0

// Memory information
let mem = sys::memory_info()
println!("Total: {} GB", mem.total / (1024 * 1024 * 1024))
println!("Available: {} GB", mem.available / (1024 * 1024 * 1024))

// Disk information
let disk = sys::disk_usage("/")?
println!("Free space: {} GB", disk.free / (1024 * 1024 * 1024))

// Process information
let pid = sys::current_pid()
let memory = sys::process_memory(pid)?
```

### 11.8 Signal Handling

```rust
// Register signal handlers
signal::handle(Signal::SIGINT) { |_sig|
    println!("Interrupted! Cleaning up...")
    cleanup_and_exit(0)
}

signal::handle(Signal::SIGTERM) { |_sig|
    graceful_shutdown()
}

// Send signals
signal::send(pid, Signal::SIGUSR1)?
```

### 11.9 Date and Time Operations

```rust
// Current time
let now = DateTime::now()
let unix_time = now.timestamp()

// Formatting and parsing
let formatted = now.format("%Y-%m-%d %H:%M:%S")
let parsed = DateTime::parse("2025-01-17 10:30:00", "%Y-%m-%d %H:%M:%S")?

// Duration and arithmetic
let duration = Duration::from_secs(3600)
let later = now + duration

// Timezone handling
let utc = now.to_utc()
let tokyo = now.with_timezone("Asia/Tokyo")
```

### 11.10 Logging and Diagnostics

```rust
// Structured logging
log::info!("Server started", { port: 8080, workers: 4 })
log::error!("Connection failed", { error: e, retry_count: 3 })

// Log levels
log::set_level(LogLevel::Debug)
log::debug!("Detailed information for debugging")

// Custom loggers
let logger = Logger::new()
    .with_target(File("app.log"))
    .with_target(Console)
    .with_format(JsonFormat)

logger.info("Application initialized")
```

### 11.11 Cryptography and Security

```rust
// Hashing
let hash = sha256("password")
let verified = verify_sha256("password", hash)

// Encryption (via ring/rustls)
let key = generate_key(256)
let encrypted = encrypt_aes(data, key)?
let decrypted = decrypt_aes(encrypted, key)?

// Random generation
let random_bytes = random::bytes(32)
let random_int = random::range(1, 100)
```

### 11.12 Transpilation Strategy

```rust
impl SystemsTranspiler {
    fun transpile_io(&self, op: IoOp) -> TokenStream {
        match op {
            IoOp::ReadFile(path) => quote! {
                std::fs::read_to_string(#path)
            },
            IoOp::WriteFile(path, content) => quote! {
                std::fs::write(#path, #content)
            },
            IoOp::WithFile(path, mode, body) => {
                // Generate RAII wrapper
                quote! {
                    {
                        let file = std::fs::File::open(#path)?;
                        let result = #body(file);
                        drop(file);
                        result
                    }
                }
            }
        }
    }
    
    fun transpile_command(&self, cmd: Command) -> TokenStream {
        quote! {
            std::process::Command::new(#cmd.program)
                #(.arg(#cmd.args))*
                #(.env(#cmd.env.0, #cmd.env.1))*
                .output()
        }
    }
}
```

### 11.13 Error Handling

All systems operations return `Result<T, SystemError>`:

```rust
pub enum SystemError {
    Io(std::io::Error),
    Network(NetworkError),
    Process(ProcessError),
    Permission(PermissionError),
    Timeout(Duration),
}

impl SystemError {
    fun is_transient(&self) -> bool {
        matches!(self, 
            SystemError::Network(_) | 
            SystemError::Timeout(_)
        )
    }
    
    fun should_retry(&self) -> bool {
        self.is_transient()
    }
}
```

### 11.14 Performance Characteristics

```rust
pub struct SystemsPerformance {
    // I/O operations
    file_read_throughput: BytesPerSec,     // ~2 GB/s (NVMe SSD)
    network_throughput: BytesPerSec,       // ~10 Gb/s (10GbE)
    
    // Process operations
    spawn_overhead: Duration,              // ~1-5ms
    ipc_latency: Duration,                 // ~1-10μs (shared memory)
    
    // Overhead vs raw syscalls
    abstraction_cost: Percent,             // <1% for most operations
}
```

## 12. Security Specification

### 12.1 Memory Safety Guarantees

Ruchy inherits Rust's memory safety through transpilation. No unsafe blocks in generated code unless explicitly marked:

```rust
// Ruchy source - safe by default
fun process_buffer(data: [u8]) {
    data.iter().map(|b| b ^ 0xFF).collect()
}

// Transpiles to safe Rust
fun process_buffer(data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b ^ 0xFF).collect()
}

// Unsafe requires explicit annotation
@[unsafe]
fun raw_pointer_access(ptr: *const u8, len: usize) -> [u8] {
    unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
}
```

### 12.2 Input Validation

Compile-time and runtime validation layers:

```rust
pub struct InputValidator {
    max_string_length: usize,     // 10MB default
    max_array_size: usize,        // 1M elements
    max_recursion_depth: usize,   // 1000 levels
}

impl InputValidator {
    fun validate_string(&self, s: &str) -> Result<(), SecurityError> {
        if s.len() > self.max_string_length {
            return Err(SecurityError::StringTooLarge);
        }
        if !s.is_char_boundary(0) {
            return Err(SecurityError::InvalidUtf8);
        }
        Ok(())
    }
    
    fun validate_regex(&self, pattern: &str) -> Result<(), SecurityError> {
        // Prevent ReDoS attacks
        let complexity = self.calculate_regex_complexity(pattern);
        if complexity > MAX_REGEX_COMPLEXITY {
            return Err(SecurityError::RegexTooComplex);
        }
        Ok(())
    }
}
```

### 12.3 Sandboxing

REPL and script execution sandboxing:

```rust
pub struct SecuritySandbox {
    fs_whitelist: Vec<PathBuf>,
    network_whitelist: Vec<IpAddr>,
    syscall_filter: SeccompFilter,
    resource_limits: ResourceLimits,
}

impl SecuritySandbox {
    pub fun execute<T>(&self, code: impl FnOnce() -> T) -> Result<T> {
        // Apply seccomp filter
        self.syscall_filter.apply()?;
        
        // Set resource limits
        rlimit::setrlimit(Resource::AS, self.resource_limits.memory)?;
        rlimit::setrlimit(Resource::CPU, self.resource_limits.cpu_time)?;
        
        // Drop privileges
        caps::drop(caps::Capability::ALL)?;
        
        // Execute in restricted context
        Ok(code())
    }
}
```

### 12.4 Dependency Scanning

Automated vulnerability detection:

```rust
pub struct DependencyScanner {
    advisory_db: RustSecAdvisoryDb,
    license_checker: LicenseChecker,
}

impl DependencyScanner {
    pub fun scan(&self, manifest: &Cargo.toml) -> SecurityReport {
        let mut vulnerabilities = Vec::new();
        
        for dependency in manifest.dependencies() {
            // Check for known CVEs
            if let Some(advisories) = self.advisory_db.check(dependency) {
                vulnerabilities.extend(advisories);
            }
            
            // License compliance
            if !self.license_checker.is_allowed(dependency.license()) {
                vulnerabilities.push(Vulnerability::License(dependency));
            }
            
            // Version pinning
            if dependency.version().contains("*") {
                vulnerabilities.push(Vulnerability::Unpinned(dependency));
            }
        }
        
        SecurityReport { vulnerabilities }
    }
}
```

### 12.5 Cryptographic Operations

Secure-by-default crypto via ring/rustls:

```rust
// No home-grown crypto - use audited libraries
pub struct CryptoProvider {
    rng: SystemRandom,
}

impl CryptoProvider {
    pub fun generate_key(&self) -> Key {
        let mut key = [0u8; 32];
        self.rng.fill(&mut key).expect("RNG failure");
        Key::from(key)
    }
    
    pub fun constant_time_compare(&self, a: &[u8], b: &[u8]) -> bool {
        ring::constant_time::verify_slices_are_equal(a, b).is_ok()
    }
    
    pub fun hash_password(&self, password: &str) -> String {
        // Argon2id with secure defaults
        argon2::hash_encoded(
            password.as_bytes(),
            &self.generate_salt(),
            &argon2::Config::default()
        ).unwrap()
    }
}
```

### 12.6 Taint Analysis

Track untrusted data through the program:

```rust
#[derive(Clone)]
pub struct TaintedValue<T> {
    value: T,
    taint: TaintLevel,
    origin: TaintOrigin,
}

pub enum TaintLevel {
    Untrusted,     // User input, network data
    Sanitized,     // Validated but not trusted
    Trusted,       // Internal computation
}

impl<T> TaintedValue<T> {
    pub fun propagate<U>(&self, f: impl FnOnce(&T) -> U) -> TaintedValue<U> {
        TaintedValue {
            value: f(&self.value),
            taint: self.taint.clone(),
            origin: self.origin.clone(),
        }
    }
    
    pub fun assert_trusted(self) -> Result<T, SecurityError> {
        match self.taint {
            TaintLevel::Trusted => Ok(self.value),
            _ => Err(SecurityError::UntrustedValue),
        }
    }
}
```

### 12.7 Secure Defaults

```rust
pub struct SecurityDefaults {
    // Network
    tls_min_version: TlsVersion::Tls13,
    cipher_suites: &[CipherSuite::TLS13_AES_256_GCM_SHA384],
    
    // File I/O
    file_permissions: 0o600,  // Owner read/write only
    directory_permissions: 0o700,
    
    // Process
    enable_aslr: true,
    enable_dep: true,
    enable_pie: true,
    
    // Memory
    zero_on_drop: true,
    stack_canaries: true,
}
```

### 12.8 Audit Logging

```rust
pub struct SecurityAuditLog {
    sink: AuditSink,
    encryption: Option<EncryptionKey>,
}

impl SecurityAuditLog {
    pub fun log_event(&self, event: SecurityEvent) {
        let entry = AuditEntry {
            timestamp: SystemTime::now(),
            event,
            context: self.capture_context(),
            hash: self.compute_hash(&event),
        };
        
        let serialized = if let Some(key) = &self.encryption {
            self.encrypt(&entry, key)
        } else {
            entry.serialize()
        };
        
        self.sink.write(serialized);
    }
}

pub enum SecurityEvent {
    AuthenticationFailure { user: String, ip: IpAddr },
    PrivilegeEscalation { from: UserId, to: UserId },
    SuspiciousPattern { pattern: String, score: f64 },
    ResourceExhaustion { resource: Resource, limit: u64 },
}
```

### 12.9 Supply Chain Security

```rust
pub struct SupplyChainValidator {
    allowed_registries: Vec<Url>,
    signing_keys: Vec<PublicKey>,
    reproducible_builds: bool,
}

impl SupplyChainValidator {
    pub fun validate_dependency(&self, dep: &Dependency) -> Result<()> {
        // Registry validation
        if !self.allowed_registries.contains(&dep.registry()) {
            return Err(SecurityError::UntrustedRegistry);
        }
        
        // Signature verification
        if let Some(sig) = dep.signature() {
            self.verify_signature(dep, sig)?;
        } else if self.require_signatures {
            return Err(SecurityError::UnsignedDependency);
        }
        
        // Reproducible build verification
        if self.reproducible_builds {
            self.verify_reproducible_build(dep)?;
        }
        
        Ok(())
    }
}
```

### 12.10 Threat Model

```rust
pub enum ThreatVector {
    // Memory corruption (mitigated by Rust)
    BufferOverflow,      // Prevented by bounds checking
    UseAfterFree,        // Prevented by ownership
    DataRace,            // Prevented by borrow checker
    
    // Logic vulnerabilities (require analysis)
    InjectionAttack,     // SQL, command, path injection
    XSS,                 // Cross-site scripting
    CSRF,                // Cross-site request forgery
    
    // Resource exhaustion
    MemoryExhaustion,    // OOM attacks
    CPUExhaustion,       // Algorithmic complexity attacks
    DiskExhaustion,      // Storage filling
    
    // Supply chain
    MaliciousDependency, // Backdoored packages
    Typosquatting,       // Similar package names
    DependencyConfusion, // Internal/external package confusion
}

impl ThreatModel {
    pub fun assess_risk(&self, code: &Ast) -> RiskAssessment {
        let mut risks = Vec::new();
        
        // Static analysis passes
        risks.extend(self.check_injection_vulnerabilities(code));
        risks.extend(self.check_resource_exhaustion(code));
        risks.extend(self.check_unsafe_patterns(code));
        
        RiskAssessment {
            severity: risks.iter().map(|r| r.severity).max(),
            mitigations: self.suggest_mitigations(&risks),
        }
    }
}
```

### 12.11 Security Configuration

```toml
[security]
# Compilation
forbid_unsafe = true
require_overflow_checks = true
strip_symbols = true

# Runtime
sandbox.enabled = true
sandbox.fs_whitelist = ["./data", "/tmp/ruchy"]
sandbox.network_whitelist = ["127.0.0.1"]
sandbox.max_memory = "1GB"
sandbox.max_cpu_time = "60s"

# Dependencies
dependency_scanning = true
allow_git_dependencies = false
require_signed_packages = true
allowed_registries = ["https://crates.io"]

# Audit
audit_log.enabled = true
audit_log.encryption = true
audit_log.retention_days = 90
```

## 13. MCP Message-Passing Architecture

### 7.1 Actor Model with MCP Integration

```rust
pub trait Actor: Send + Sync {
    type Message: McpSerializable;
    type Response: McpSerializable;
    
    async fun receive(&mut self, msg: Self::Message) -> Option<Self::Response>;
    
    fun spawn(self) -> ActorHandle<Self::Message, Self::Response> {
        let (tx, rx) = mpsc::channel(100);
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                self.receive(msg).await;
            }
        });
        ActorHandle { tx }
    }
}

// MCP protocol support
#[derive(McpSerializable)]
pub struct McpMessage {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: Option<String>,
}

impl Actor for McpActor {
    type Message = McpMessage;
    type Response = McpResponse;
    
    async fun receive(&mut self, msg: McpMessage) -> Option<McpResponse> {
        match msg.method.as_str() {
            "tools/list" => self.list_tools().await,
            "tools/call" => self.call_tool(msg.params).await,
            _ => None,
        }
    }
}
```

### 7.2 Supervision Trees

```rust
pub struct Supervisor<A: Actor> {
    children: Vec<ActorHandle<A::Message, A::Response>>,
    strategy: SupervisionStrategy,
}

pub enum SupervisionStrategy {
    OneForOne,      // Restart failed child
    OneForAll,      // Restart all children
    RestForOne,     // Restart failed and subsequent
}

impl<A: Actor> Supervisor<A> {
    pub fun supervise(&mut self, child: A) {
        let handle = child.spawn();
        self.monitor(handle.clone());
        self.children.push(handle);
    }
    
    async fun monitor(&mut self, handle: ActorHandle<_, _>) {
        loop {
            if handle.is_failed().await {
                match self.strategy {
                    OneForOne => self.restart_child(handle).await,
                    OneForAll => self.restart_all().await,
                    RestForOne => self.restart_from(handle).await,
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
```

## 8. LSP Specification

### 8.1 Language Server Protocol Implementation

```rust
pub struct RuchyLanguageServer {
    workspace: Workspace,
    analyzer: SemanticAnalyzer,
    formatter: Formatter,
}

#[tower_lsp::async_trait]
impl LanguageServer for RuchyLanguageServer {
    async fun initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Full),
                completion_provider: Some(CompletionOptions::default()),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SEMANTIC_TOKEN_LEGEND.clone(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        }
                    )
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    
    async fun completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        let position = params.text_document_position;
        let document = self.workspace.get_document(&position.text_document.uri)?;
        
        let completions = self.analyzer.get_completions(
            &document,
            position.position
        )?;
        
        Ok(CompletionResponse::Array(completions))
    }
}
```

### 8.2 Semantic Analysis

```rust
pub struct SemanticAnalyzer {
    type_checker: TypeChecker,
    symbol_table: SymbolTable,
    diagnostics: Vec<Diagnostic>,
}

impl SemanticAnalyzer {
    pub fun analyze(&mut self, ast: &Ast) -> Result<TypedAst> {
        // Phase 1: Name resolution
        self.resolve_names(ast)?;
        
        // Phase 2: Type inference
        let typed_ast = self.type_checker.infer(ast)?;
        
        // Phase 3: Borrow checking
        self.check_borrows(&typed_ast)?;
        
        // Phase 4: Effect checking
        self.check_effects(&typed_ast)?;
        
        Ok(typed_ast)
    }
}
```

## 9. Critical Missing Components

### 9.1 Component Inventory

```rust
pub enum ComponentStatus {
    Complete,
    InProgress { completion: u8 },
    NotStarted,
    Blocked { by: Vec<String> },
}

pub struct ComponentTracker {
    components: HashMap<String, Component>,
}

pub struct Component {
    name: String,
    status: ComponentStatus,
    priority: Priority,
    estimated_days: u32,
    dependencies: Vec<String>,
}

// Current missing components (CORRECTED)
impl ComponentTracker {
    fun critical_missing() -> Vec<Component> {
        vec![
            Component {
                name: "Type Inference Engine".into(),
                status: InProgress { completion: 40 },
                priority: P0,
                estimated_days: 10,
                dependencies: vec!["Parser".into()],
            },
            Component {
                name: "Ownership Mapping Rules".into(),  // NOT a borrow checker
                status: NotStarted,
                priority: P0,
                estimated_days: 5,  // Reduced from 15
                dependencies: vec!["Type Inference".into()],
                notes: "Map Ruchy patterns to Rust borrowing rules, not implement checker".into(),
            },
            Component {
                name: "MIR Generation".into(),
                status: NotStarted,
                priority: P1,
                estimated_days: 10,
                dependencies: vec!["Type Inference".into()],
            },
            Component {
                name: "DataFrame IR Fusion".into(),
                status: NotStarted,
                priority: P1,
                estimated_days: 7,
                dependencies: vec!["MIR Generation".into()],
            },
        ]
    }
}
```

## 10. Binary Architecture

### 10.1 Compilation Pipeline

```rust
pub struct CompilationPipeline {
    stages: Vec<Box<dyn CompilationStage>>,
}

pub trait CompilationStage {
    fun process(&self, input: CompilerInput) -> Result<CompilerOutput>;
}

impl CompilationPipeline {
    pub fun standard() -> Self {
        CompilationPipeline {
            stages: vec![
                Box::new(Lexer),
                Box::new(Parser),
                Box::new(Desugarer),
                Box::new(TypeChecker),
                Box::new(BorrowChecker),
                Box::new(MirGenerator),
                Box::new(Optimizer),
                Box::new(RustCodeGen),
                Box::new(RustCompiler),
            ],
        }
    }
    
    pub fun compile(&self, source: &str) -> Result<Binary> {
        let mut output = CompilerInput::Source(source.to_string());
        
        for stage in &self.stages {
            output = stage.process(output)?;
        }
        
        match output {
            CompilerOutput::Binary(bin) => Ok(bin),
            _ => Err(Error::InvalidPipeline),
        }
    }
}
```

### 10.2 Binary Format

```rust
pub struct RuchyBinary {
    magic: [u8; 4],  // b"RCHY"
    version: Version,
    metadata: Metadata,
    
    // Sections
    code: Vec<u8>,
    data: Vec<u8>,
    symbols: SymbolTable,
    debug_info: Option<DebugInfo>,
    
    // Embedded runtime (if standalone)
    runtime: Option<EmbeddedRuntime>,
}

impl RuchyBinary {
    pub fun write(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        
        // Write header
        file.write_all(&self.magic)?;
        file.write_all(&self.version.to_bytes())?;
        
        // Write sections with alignment
        self.write_section(&mut file, "CODE", &self.code)?;
        self.write_section(&mut file, "DATA", &self.data)?;
        self.write_section(&mut file, "SYMB", &self.symbols.to_bytes())?;
        
        if let Some(debug) = &self.debug_info {
            self.write_section(&mut file, "DBUG", &debug.to_bytes())?;
        }
        
        Ok(())
    }
}
```

## 11. Edge Cases Specification

### 11.1 Parser Edge Cases

```rust
#[cfg(test)]
mod edge_case_tests {
    #[test]
    fun test_unicode_identifiers() {
        let source = "let 你好 = 42";
        assert!(parse(source).is_ok());
    }
    
    #[test]
    fun test_nested_string_interpolation() {
        let source = r#""outer {"{inner}"} text""#;
        let ast = parse(source).unwrap();
        assert_matches!(ast, Expr::String { parts, .. } if parts.len() == 3);
    }
    
    #[test]
    fun test_pipeline_precedence() {
        let source = "1 + 2 |> f |> g * 3";
        // Should parse as: (g(f(1 + 2))) * 3
        let ast = parse(source).unwrap();
        assert_correct_precedence(ast);
    }
    
    #[test]
    fun test_actor_message_pattern_exhaustiveness() {
        let source = r#"
            actor Counter {
                receive {
                    Inc => count += 1,
                    Dec => count -= 1
                    // Missing Get case - should warn
                }
            }
        "#;
        let diagnostics = analyze(source);
        assert!(diagnostics.has_warning("Non-exhaustive patterns"));
    }
}
```

### 11.2 Type System Edge Cases

```rust
// Variance edge cases
trait Container<T> {
    fun put(&mut self, item: T);
    fun get(&self) -> &T;
}

// Lifetime edge cases
fun complex_lifetimes<'a, 'b>(
    x: &'a str,
    y: &'b str
) -> &'a str where 'b: 'a {
    if x.len() > y.len() { x } else { y }
}

// Higher-kinded types (future)
trait Functor<F<_>> {
    fun map<A, B>(fa: F<A>, f: fun(A) -> B) -> F<B>
}
```

## 12. REPL Testing Specification

### 12.1 Interactive Testing Framework

```rust
pub struct ReplTester {
    repl: Repl,
    transcript: Vec<TestStep>,
}

pub struct TestStep {
    input: String,
    expected_output: String,
    expected_state: Option<HashMap<String, Value>>,
}

impl ReplTester {
    pub fun test_transcript(&mut self, script: &str) -> TestResult {
        let steps = self.parse_transcript(script)?;
        
        for step in steps {
            let output = self.repl.eval(&step.input)?;
            
            assert_eq!(output, step.expected_output);
            
            if let Some(state) = step.expected_state {
                for (var, expected) in state {
                    let actual = self.repl.get_binding(&var)?;
                    assert_eq!(actual, expected);
                }
            }
        }
        
        TestResult::Pass
    }
}
```

### 12.2 Property-Based REPL Tests

```rust
#[proptest]
fun repl_doesnt_crash(input: String) {
    let mut repl = Repl::new();
    let _ = repl.eval(&input); // Must not panic
}

#[proptest]
fun repl_state_consistency(
    commands: Vec<ReplCommand>
) {
    let mut repl = Repl::new();
    let mut model = ReplModel::new();
    
    for cmd in commands {
        let repl_result = repl.execute(&cmd);
        let model_result = model.execute(&cmd);
        
        prop_assert_eq!(repl_result, model_result);
    }
}
```

## 13. REPL UX Specification

### 13.1 Interaction Model

```rust
pub struct ReplUx {
    input_handler: InputHandler,
    display_engine: DisplayEngine,
    history_manager: HistoryManager,
    completion_engine: CompletionEngine,
    help_system: HelpSystem,
}

pub enum ReplMode {
    Expression,     // Default: evaluate expressions
    Command,        // Meta-commands (:help, :type, etc.)
    Multiline,      // Block input mode
    Debug,          // Step-through evaluation
    Notebook,       // Jupyter-style cells
}
```

### 13.2 Display Engine

```rust
pub struct DisplayEngine {
    formatters: HashMap<TypeId, Box<dyn Formatter>>,
    truncation: TruncationPolicy,
    color_scheme: ColorScheme,
}

impl DisplayEngine {
    // Automatic rich display for DataFrames
    fun display_dataframe(&self, df: &DataFrame) -> String {
        let shape = df.shape();
        let preview_rows = 10.min(shape.0);
        
        let mut table = Table::new();
        table.set_header(df.columns());
        
        // Intelligent truncation for wide tables
        let display_cols = if shape.1 > 20 {
            let start_cols = &df.columns()[..10];
            let end_cols = &df.columns()[shape.1-10..];
            format!("{} ... {} (showing 20 of {} columns)", 
                start_cols.join(", "), 
                end_cols.join(", "),
                shape.1)
        } else {
            df.columns().join(", ")
        };
        
        // Type-aware cell formatting
        for row in df.head(preview_rows).iter() {
            table.add_row(row.iter().map(|cell| {
                match cell.dtype() {
                    DataType::Float64 => format!("{:.4}", cell),
                    DataType::Date => cell.format("%Y-%m-%d"),
                    _ => cell.to_string(),
                }
            }));
        }
        
        format!("{}\nShape: {} rows × {} columns", table, shape.0, shape.1)
    }
    
    // Syntax-highlighted code display
    fun display_code(&self, code: &str, lang: Language) -> String {
        let theme = &self.color_scheme;
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        
        let syntax = ps.find_syntax_by_extension(lang.extension())
            .unwrap_or_else(|| ps.find_syntax_plain_text());
        
        let mut h = HighlightLines::new(syntax, &ts.themes[theme.name()]);
        let ranges: Vec<_> = h.highlight(code, &ps);
        styled_ranges_to_string(&ranges)
    }
}
```

### 13.3 Smart Completion

```rust
pub struct CompletionEngine {
    symbol_table: SymbolTable,
    type_inference: TypeInference,
    ml_model: Option<CompletionModel>,
}

impl CompletionEngine {
    fun complete(&self, partial: &str, cursor: usize) -> Vec<Completion> {
        let context = self.extract_context(partial, cursor);
        
        match context {
            Context::MemberAccess { receiver, partial_member } => {
                // Type-aware member completion
                let receiver_type = self.type_inference.infer_type(&receiver);
                self.get_members(receiver_type)
                    .filter(|m| m.starts_with(partial_member))
                    .map(|m| Completion {
                        text: m.name,
                        kind: m.kind,
                        documentation: m.docs,
                        score: self.score_relevance(&m, &context),
                    })
                    .sorted_by_key(|c| -c.score)
                    .take(10)
                    .collect()
            }
            Context::DataFrameOp { df, partial_op } => {
                // DataFrame-specific completions
                vec![
                    Completion::new("filter", "Filter rows based on condition"),
                    Completion::new("groupby", "Group by column(s)"),
                    Completion::new("agg", "Aggregate grouped data"),
                    Completion::new("select", "Select specific columns"),
                    Completion::new("sort", "Sort by column(s)"),
                ]
                .into_iter()
                .filter(|c| c.text.starts_with(partial_op))
                .collect()
            }
            Context::Import { partial_path } => {
                // Crate and module completion
                self.complete_import_path(partial_path)
            }
            _ => self.fallback_completions(partial),
        }
    }
}
```

### 13.4 Interactive Help System

```rust
pub struct HelpSystem {
    documentation: DocIndex,
    examples: ExampleDatabase,
    tutorials: TutorialEngine,
}

impl HelpSystem {
    // Context-aware help
    fun help(&self, query: Option<&str>) -> HelpResponse {
        match query {
            None => self.general_help(),
            Some(topic) => {
                if let Some(symbol) = self.resolve_symbol(topic) {
                    self.symbol_help(symbol)
                } else if let Some(error) = self.error_help(topic) {
                    error
                } else {
                    self.search_help(topic)
                }
            }
        }
    }
    
    // Live examples in REPL
    fun example(&self, topic: &str) -> Option<Example> {
        self.examples.get(topic).map(|ex| Example {
            description: ex.description.clone(),
            code: ex.code.clone(),
            expected_output: ex.output.clone(),
            runnable: true,  // Can execute directly in REPL
        })
    }
    
    // Interactive tutorials
    fun tutorial(&self, name: Option<&str>) -> Tutorial {
        match name {
            None => self.list_tutorials(),
            Some("dataframes") => Tutorial::dataframes(),
            Some("actors") => Tutorial::actors(),
            Some("pipelines") => Tutorial::pipelines(),
            _ => Tutorial::not_found(name),
        }
    }
}
```

### 13.5 History Management

```rust
pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    search_index: SearchIndex,
    persistent_store: Option<PathBuf>,
}

impl HistoryManager {
    // Fuzzy history search
    fun search(&self, pattern: &str) -> Vec<&HistoryEntry> {
        if pattern.starts_with("!") {
            // Bash-style history expansion
            self.expand_history(pattern)
        } else {
            // Fuzzy search with frecency scoring
            self.search_index
                .fuzzy_search(pattern)
                .sorted_by_key(|e| -e.frecency_score())
                .take(10)
                .collect()
        }
    }
    
    // Semantic deduplication
    fun add_entry(&mut self, input: String, output: Value) {
        // Don't add duplicate semantic entries
        if !self.is_semantic_duplicate(&input) {
            let entry = HistoryEntry {
                id: self.next_id(),
                timestamp: Instant::now(),
                input: input.clone(),
                output: Some(output),
                execution_time: self.last_execution_time(),
            };
            
            self.entries.push(entry.clone());
            self.search_index.index(&entry);
            
            if let Some(store) = &self.persistent_store {
                self.persist_entry(store, &entry);
            }
        }
    }
}
```

### 13.6 Error Recovery UX

```rust
pub struct ErrorRecovery {
    error_db: ErrorDatabase,
    suggestion_engine: SuggestionEngine,
}

impl ErrorRecovery {
    fun handle_error(&self, error: ReplError) -> ErrorResponse {
        let suggestions = self.suggestion_engine.suggest(&error);
        let similar_errors = self.error_db.find_similar(&error);
        
        ErrorResponse {
            message: self.format_error_message(&error),
            suggestions,
            similar_errors,
            quick_fixes: self.generate_quick_fixes(&error),
            documentation_links: self.relevant_docs(&error),
        }
    }
    
    // Intelligent error messages
    fun format_error_message(&self, error: &ReplError) -> String {
        match error {
            ReplError::TypeError { expected, found, span } => {
                let context = self.extract_context(span);
                format!(
                    "Type mismatch at line {}:\n\
                     {}\n\
                     Expected: {}\n\
                     Found: {}\n\
                     Hint: {}",
                    span.line,
                    self.highlight_error(context, span),
                    self.format_type(expected),
                    self.format_type(found),
                    self.suggest_type_conversion(found, expected)
                )
            }
            ReplError::UnresolvedImport { module } => {
                format!(
                    "Cannot find module '{}'.\n\
                     Did you mean one of these?\n\
                     {}",
                    module,
                    self.suggest_similar_modules(module)
                        .iter()
                        .map(|m| format!("  - {}", m))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            _ => error.to_string(),
        }
    }
}
```

### 13.7 Multi-line Input

```rust
pub struct MultilineInput {
    buffer: String,
    mode: InputMode,
    bracket_matcher: BracketMatcher,
}

impl MultilineInput {
    fun should_continue(&self, input: &str) -> bool {
        // Smart continuation detection
        self.bracket_matcher.has_unclosed(input) ||
        input.ends_with("\\") ||
        self.is_incomplete_statement(input)
    }
    
    fun is_incomplete_statement(&self, input: &str) -> bool {
        match self.parse_partial(input) {
            Ok(_) => false,
            Err(ParseError::UnexpectedEof) => true,
            Err(_) => false,
        }
    }
    
    // Visual feedback for multi-line mode
    fun prompt(&self, line_number: usize) -> String {
        match self.mode {
            InputMode::Normal => "ruchy> ".to_string(),
            InputMode::Continuation => format!("  ...{} ", line_number),
            InputMode::String => "  str> ".to_string(),
            InputMode::Comment => "  com> ".to_string(),
        }
    }
}
```

### 13.8 Performance Monitoring

```rust
pub struct ReplPerformance {
    metrics: Metrics,
    profiler: MicroProfiler,
}

impl ReplPerformance {
    // Automatic performance warnings
    fun monitor_execution(&mut self, input: &str) -> TimingReport {
        let start = Instant::now();
        
        let parse_time = self.measure(|| parse(input));
        let compile_time = self.measure(|| compile(input));
        let execute_time = self.measure(|| execute(input));
        
        let total = start.elapsed();
        
        // Warn if REPL response exceeds target
        if total > Duration::from_millis(15) {
            self.analyze_bottleneck(parse_time, compile_time, execute_time)
        }
        
        TimingReport {
            total,
            breakdown: vec![
                ("parse", parse_time),
                ("compile", compile_time),
                ("execute", execute_time),
            ],
        }
    }
    
    fun analyze_bottleneck(&self, parse: Duration, compile: Duration, execute: Duration) {
        let total = parse + compile + execute;
        
        if parse > total * 0.5 {
            println!("⚠️ Slow parsing detected. Consider breaking up complex expressions.");
        } else if compile > total * 0.5 {
            println!("⚠️ Slow compilation. Type inference may be struggling.");
        } else if execute > total * 0.5 {
            println!("⚠️ Slow execution. Consider using lazy evaluation for DataFrames.");
        }
    }
}
```

### 13.9 Configuration

```toml
# ~/.ruchy/repl.toml
[display]
max_rows = 20
max_columns = 30
float_precision = 4
color_scheme = "monokai"
unicode_tables = true

[completion]
enable_ml = true
min_chars = 2
max_suggestions = 10
include_docs = true

[history]
max_entries = 10000
persistent = true
deduplicate = true
search_algorithm = "fuzzy"

[performance]
warn_threshold_ms = 15
profile_slow_commands = true
cache_compiled_expressions = true
```

## 14. Docker Specification

### 13.1 Container Architecture

```dockerfile
# Multi-stage build for minimal size
FROM rust:1.75 as builder

WORKDIR /usr/src/ruchy
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build with optimizations
RUN cargo build --release --features "docker"

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ruchy/target/release/ruchy /usr/local/bin/

# Create non-root user
RUN useradd -m -u 1000 ruchy
USER ruchy

ENTRYPOINT ["ruchy"]
CMD ["--repl"]
```

### 13.2 Docker Compose for Development

```yaml
version: '3.8'

services:
  ruchy-dev:
    build:
      context: .
      dockerfile: Dockerfile.dev
    volumes:
      - ./src:/workspace/src
      - ./examples:/workspace/examples
      - cargo-cache:/usr/local/cargo
    environment:
      - RUST_LOG=debug
      - RUCHY_MODE=development
    ports:
      - "8080:8080"  # LSP server
      - "9229:9229"  # Debug port
    
  ruchy-jupyter:
    image: ruchy/jupyter:latest
    ports:
      - "8888:8888"
    volumes:
      - ./notebooks:/home/jovyan/work
    environment:
      - JUPYTER_ENABLE_LAB=yes

volumes:
  cargo-cache:
```

## 14. Cargo Integration

### 14.1 Build Script Integration

```rust
// build.rs
use ruchy_compiler::Transpiler;

fun main() {
    println!("cargo:rerun-if-changed=src/");
    
    let ruchy_files = glob::glob("src/**/*.ruchy")
        .expect("Failed to read glob pattern");
    
    let transpiler = Transpiler::new();
    
    for entry in ruchy_files {
        let path = entry.unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        
        let rust_code = transpiler.transpile(&source).unwrap();
        
        let out_path = path.with_extension("rs");
        std::fs::write(out_path, rust_code).unwrap();
    }
}
```

### 14.2 Cargo.toml Configuration

```toml
[package]
name = "my-ruchy-project"
version = "0.1.0"
edition = "2021"

[dependencies]
ruchy-runtime = "1.0"
polars = { version = "0.35", features = ["lazy"] }
tokio = { version = "1.35", features = ["full"] }

[build-dependencies]
ruchy-compiler = "1.0"
glob = "0.3"

[features]
default = ["polars-backend"]
polars-backend = ["ruchy-runtime/polars"]
ndarray-backend = ["ruchy-runtime/ndarray"]

[[bin]]
name = "main"
path = "src/main.ruchy"
```

## 15. Depyler Integration

### 15.1 Python Pattern Mapping

```rust
// LIMITED SCOPE: Common pattern mapping only, not general transpilation
pub struct DepylerPatternMapper {
    patterns: HashMap<Pattern, Transform>,
}

impl DepylerPatternMapper {
    pub fun new() -> Self {
        let mut patterns = HashMap::new();
        
        // NumPy/Pandas → Polars mappings (~20 patterns)
        patterns.insert(
            Pattern::PandasDataFrame,
            Transform::PolarsDataFrame,
        );
        patterns.insert(
            Pattern::NumpyArray,
            Transform::PolarsSeries,
        );
        patterns.insert(
            Pattern::PandasGroupBy,
            Transform::PolarsGroupBy,
        );
        
        // List comprehension → iterator chains
        patterns.insert(
            Pattern::ListComp,
            Transform::IteratorChain,
        );
        
        // Type annotations → Ruchy types
        patterns.insert(
            Pattern::TypeHint("List[int]"),
            Transform::RuchyType("[i32]"),
        );
        
        DepylerPatternMapper { patterns }
    }
    
    pub fun map_snippet(&self, python: &str) -> Option<String> {
        // Only handles specific patterns, not arbitrary Python
        for (pattern, transform) in &self.patterns {
            if pattern.matches(python) {
                return Some(transform.apply(python));
            }
        }
        None  // Unrecognized pattern
    }
}

// NOT a full Python→Ruchy transpiler
// Just a pattern recognizer for common data science idioms
```

## 16. Rust Cargo InterOp

### 16.1 Direct Crate Usage

```rust
// No FFI needed - direct Rust crate usage
import std::collections::HashMap
import tokio::time::Duration
import serde::{Serialize, Deserialize}
import polars::prelude::*

// Use any Rust crate directly
fun use_external_crate() {
    let client = reqwest::Client::new()
    let response = client
        .get("https://api.example.com/data")
        .timeout(Duration::from_secs(10))
        .send()
        .await?
    
    let data: DataFrame = response.json().await?
    data |> filter(col("value") > 100)
}
```

### 16.2 Ruchy as Rust Library

```rust
// lib.rs - Ruchy code exposed to Rust
#[ruchy::export]
fun process_data(df: DataFrame) -> DataFrame {
    df |> filter(col("age") > 18)
       |> groupby("city")
       |> agg([
           col("salary").mean().alias("avg_salary"),
           col("name").count().alias("count")
       ])
}

// Can be used from Rust:
use my_ruchy_lib::process_data;

fun main() {
    let df = DataFrame::read_csv("data.csv")?;
    let result = process_data(df);
    println!("{}", result);
}
```

## 17. One-Liner and Script Execution

### 17.1 Command-Line Interface

```rust
#[derive(Parser)]
#[command(name = "ruchy")]
pub struct Cli {
    /// Execute a one-liner
    #[arg(short, long)]
    eval: Option<String>,
    
    /// Run a script file
    file: Option<PathBuf>,
    
    /// Start REPL
    #[arg(long)]
    repl: bool,
    
    /// Compile to binary
    #[arg(short, long)]
    compile: bool,
    
    /// Output path for compilation
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Enable JIT compilation
    #[arg(long)]
    jit: bool,
    
    /// Optimization level (0-3)
    #[arg(short = 'O', default_value = "2")]
    opt_level: u8,
}

impl Cli {
    pub fun execute(self) -> Result<()> {
        match self {
            Cli { eval: Some(code), .. } => {
                // One-liner execution
                let result = Runtime::eval_one_liner(&code)?;
                println!("{}", result);
            }
            Cli { file: Some(path), compile: true, .. } => {
                // AOT compilation
                let binary = Compiler::compile_file(&path)?;
                binary.write(self.output.unwrap_or("a.out".into()))?;
            }
            Cli { file: Some(path), .. } => {
                // Script execution
                Runtime::run_script(&path)?;
            }
            Cli { repl: true, .. } => {
                // Interactive REPL
                Repl::new().run()?;
            }
            _ => {
                // Default to REPL
                Repl::new().run()?;
            }
        }
        Ok(())
    }
}
```

### 17.2 Script Examples

```rust
#!/usr/bin/env ruchy

// Data analysis script
import polars::prelude::*
import std::env

fun main() {
    let args = env::args()
    let file = args.get(1).expect("Usage: script.ruchy <file>")
    
    let df = DataFrame::read_csv(file)?
    
    // Analysis pipeline
    let result = df
        |> filter(col("date") >= "2024-01-01")
        |> groupby("category")
        |> agg([
            col("amount").sum().alias("total"),
            col("amount").mean().alias("average"),
            col("id").count().alias("count")
        ])
        |> sort("total", descending=true)
    
    println!("Analysis Results:")
    println!("{}", result)
    
    // Export to multiple formats
    result.write_csv("output.csv")?
    result.write_parquet("output.parquet")?
    result.write_json("output.json")?
}
```

## 18. Disassembly Specification

### 18.1 Multi-Representation Output

```rust
pub trait Disassembler<Input> {
    type Output;
    fun disassemble(&self, input: &Input) -> Self::Output;
}

// Primary representations
impl Disassembler<TypedAst> for JsonAstDisassembler {
    fun disassemble(&self, ast: &TypedAst) -> String {
        serde_json::to_string_pretty(ast).unwrap()
    }
}

impl Disassembler<TypedAst> for BytecodeDisassembler {
    fun disassemble(&self, ast: &TypedAst) -> Vec<Instruction> {
        self.compile_to_bytecode(ast)
    }
}

impl Disassembler<TypedAst> for RustDisassembler {
    fun disassemble(&self, ast: &TypedAst) -> String {
        let rust_ast = self.transform_to_rust(ast);
        quote!(#rust_ast).to_string()
    }
}
```

### 18.2 Bytecode Representation

```rust
#[derive(Debug, Clone)]
pub enum Instruction {
    // Stack operations
    Push(Value),
    Pop,
    Dup,
    Swap,
    
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Pow,
    
    // Comparison
    Eq, Ne, Lt, Le, Gt, Ge,
    
    // Control flow
    Jump(Label),
    JumpIf(Label),
    JumpIfNot(Label),
    Call(FunctionId, u8),
    Return,
    
    // Memory
    Load(LocalId),
    Store(LocalId),
    LoadField(FieldId),
    StoreField(FieldId),
    
    // DataFrame operations
    DfCreate,
    DfFilter,
    DfSelect,
    DfGroupBy,
    DfAgg,
    DfSort,
}

pub fun disassemble_bytecode(instructions: &[Instruction]) -> String {
    instructions.iter().enumerate()
        .map(|(i, inst)| format!("{:04}: {:?}", i, inst))
        .collect::<Vec<_>>()
        .join("\n")
}
```

## 19. Advanced Mathematical REPL

### 19.1 Mathematical Operations

```rust
pub struct MathRepl {
    base_repl: Repl,
    math_engine: MathEngine,
    plot_backend: PlotBackend,
}

impl MathRepl {
    pub fun eval_math(&mut self, expr: &str) -> Result<MathResult> {
        let parsed = self.parse_math_expr(expr)?;
        
        match parsed {
            MathExpr::Symbolic(sym) => {
                // Symbolic computation
                let simplified = self.math_engine.simplify(sym)?;
                Ok(MathResult::Symbolic(simplified))
            }
            MathExpr::Statistical(formula) => {
                // Statistical modeling (R-like)
                let model = self.fit_model(formula)?;
                Ok(MathResult::Model(model))
            }
            MathExpr::LinearAlgebra(matrix_op) => {
                // Matrix operations
                let result = self.compute_matrix(matrix_op)?;
                Ok(MathResult::Matrix(result))
            }
            MathExpr::Plot(plot_spec) => {
                // Visualization
                let chart = self.plot_backend.render(plot_spec)?;
                Ok(MathResult::Plot(chart))
            }
        }
    }
}

// Mathematical syntax extensions
impl MathRepl {
    fun parse_special_syntax(&self, input: &str) -> Option<MathExpr> {
        // LaTeX-like syntax
        if input.starts_with("\\") {
            return Some(self.parse_latex(input));
        }
        
        // R-like formula syntax
        if input.contains("~") {
            return Some(self.parse_formula(input));
        }
        
        // Matrix literal
        if input.starts_with("[[") {
            return Some(self.parse_matrix(input));
        }
        
        None
    }
}
```

### 19.2 Statistical Computing

```rust
// Built-in statistical functions
fun lm(formula: Formula, data: DataFrame) -> LinearModel {
    // Linear regression
    let model = LinearRegression::fit(formula, data)?
    model
}

fun glm(formula: Formula, data: DataFrame, family: Family) -> GLM {
    // Generalized linear models
    let model = GeneralizedLinearModel::fit(formula, data, family)?
    model
}

fun anova(model: LinearModel) -> AnovaTable {
    // Analysis of variance
    model.anova()
}

// Distribution functions
fun rnorm(n: i32, mean: f64 = 0.0, sd: f64 = 1.0) -> Series {
    Normal::new(mean, sd).sample(n)
}

fun qnorm(p: f64, mean: f64 = 0.0, sd: f64 = 1.0) -> f64 {
    Normal::new(mean, sd).quantile(p)
}
```

## 20. Quality Gates

### 20.1 Quality Metrics and Enforcement

```rust
pub struct QualityGates {
    metrics: QualityMetrics,
    thresholds: QualityThresholds,
}

#[derive(Default)]
pub struct QualityMetrics {
    test_coverage: f64,
    cyclomatic_complexity: u32,
    cognitive_complexity: u32,
    satd_count: usize,  // Self-admitted technical debt
    clippy_warnings: usize,
    documentation_coverage: f64,
    unsafe_blocks: usize,
}

pub struct QualityThresholds {
    min_test_coverage: f64,      // 80%
    max_complexity: u32,          // 10
    max_satd: usize,             // 0
    max_clippy_warnings: usize,  // 0
    min_doc_coverage: f64,        // 90%
}

impl QualityGates {
    pub fun check(&self) -> Result<QualityReport> {
        let mut violations = Vec::new();
        
        if self.metrics.test_coverage < self.thresholds.min_test_coverage {
            violations.push(Violation::InsufficientCoverage {
                current: self.metrics.test_coverage,
                required: self.thresholds.min_test_coverage,
            });
        }
        
        if self.metrics.cyclomatic_complexity > self.thresholds.max_complexity {
            violations.push(Violation::ExcessiveComplexity {
                current: self.metrics.cyclomatic_complexity,
                maximum: self.thresholds.max_complexity,
            });
        }
        
        if self.metrics.satd_count > 0 {
            violations.push(Violation::TechnicalDebt {
                count: self.metrics.satd_count,
            });
        }
        
        if violations.is_empty() {
            Ok(QualityReport::Pass)
        } else {
            Err(QualityReport::Fail { violations })
        }
    }
}
```

### 20.2 Continuous Quality Monitoring

```rust
// Integration with CI/CD
pub struct CiQualityEnforcer {
    gates: QualityGates,
    reporting: ReportingBackend,
}

impl CiQualityEnforcer {
    pub async fun run_checks(&self) -> ExitCode {
        // Collect metrics
        let coverage = self.measure_coverage().await;
        let complexity = self.analyze_complexity().await;
        let satd = self.scan_for_satd().await;
        
        // Apply gates
        let report = self.gates.check();
        
        // Report results
        self.reporting.publish(report).await;
        
        match report {
            Ok(_) => ExitCode::SUCCESS,
            Err(_) => ExitCode::FAILURE,
        }
    }
}
```

## 21. Provability

### 21.1 Property-Based Testing

```rust
#[property]
fun prop_pipeline_associativity(
    data: DataFrame,
    f: fun(DataFrame) -> DataFrame,
    g: fun(DataFrame) -> DataFrame,
    h: fun(DataFrame) -> DataFrame
) {
    // (f |> g) |> h == f |> (g |> h)
    let left = (data |> f |> g) |> h
    let right = data |> f |> (g |> h)
    assert_eq!(left, right)
}

#[property]
fun prop_actor_message_ordering(
    actor: TestActor,
    messages: Vec<Message>
) {
    for msg in messages {
        actor ! msg
    }
    
    let responses = messages.map(|_| actor ? GetState)
    
    // Messages processed in order
    assert_monotonic(responses)
}
```

### 21.2 Refinement Types with SMT

```rust
// Future: SMT-based verification
#[refine]
fun safe_divide(x: i32, y: {y: i32 | y != 0}) -> i32 {
    x / y  // Statically verified safe
}

#[refine]
fun bounded_index<T>(
    array: [T; N],
    index: {i: usize | i < N}
) -> T {
    array[index]  // No bounds check needed
}

// Verification conditions generated
pub struct SmtVerifier {
    solver: Z3Solver,
}

impl SmtVerifier {
    pub fun verify_refinement(&self, constraint: Constraint) -> VerificationResult {
        let formula = self.translate_to_smt(constraint);
        
        match self.solver.check_sat(formula) {
            Sat => VerificationResult::Valid,
            Unsat => VerificationResult::Invalid { 
                counterexample: self.solver.get_model() 
            },
            Unknown => VerificationResult::Unknown,
        }
    }
}
```

## 22. Lint Specification

### 22.1 Multi-Level Lint Architecture

```rust
pub enum LintLevel {
    Allow,      // Suppress warning
    Warn,       // Display warning, continue
    Deny,       // Error, halt compilation
    Forbid,     // Error, cannot be overridden
}

pub struct LintEngine {
    rules: HashMap<LintId, LintRule>,
    overrides: LintOverrides,
    severity_map: HashMap<LintId, LintLevel>,
}

pub struct LintRule {
    id: LintId,
    category: LintCategory,
    default_level: LintLevel,
    machine_applicable: bool,  // Can auto-fix
    mir_required: bool,        // Needs MIR analysis
}
```

### 22.2 Lint Categories

```rust
pub enum LintCategory {
    // Correctness - Always Deny/Forbid
    Correctness {
        undefined_behavior: bool,
        memory_safety: bool,
        type_safety: bool,
    },
    
    // Performance - Context-dependent
    Performance {
        complexity: ComplexityMetric,
        allocation_overhead: bool,
        unnecessary_clone: bool,
        suboptimal_collection: bool,
    },
    
    // Style - Project-configurable
    Style {
        naming_convention: NamingStyle,
        formatting: FormatSpec,
        import_organization: ImportStyle,
    },
    
    // Ruchy-specific
    RuchyIdioms {
        prefer_pipeline: bool,       // x.f().g() → x |> f |> g
        prefer_dataframe: bool,      // Vec<Vec<T>> → DataFrame
        actor_message_exhaustive: bool,
        unnecessary_transpilation: bool,
    },
}
```

### 22.3 MIR-Based Lints

```rust
impl MirLintPass {
    // DataFrame operation fusion opportunities
    fun lint_unfused_operations(&self, mir: &Mir) -> Vec<LintDiagnostic> {
        let mut diagnostics = vec![];
        
        for block in mir.blocks() {
            if let Some(chain) = self.find_dataframe_chain(block) {
                if !chain.is_fused() {
                    diagnostics.push(LintDiagnostic {
                        id: UNFUSED_DATAFRAME_OPS,
                        span: chain.span(),
                        message: "DataFrame operations can be fused",
                        suggestion: Some(chain.fused_version()),
                        machine_applicable: true,
                    });
                }
            }
        }
        diagnostics
    }
    
    // Actor message flow analysis
    fun lint_message_patterns(&self, mir: &Mir) -> Vec<LintDiagnostic> {
        self.analyze_actor_messages(mir)
            .filter(|flow| flow.has_race_condition())
            .map(|flow| LintDiagnostic {
                id: ACTOR_RACE_CONDITION,
                span: flow.span(),
                message: "Potential race condition in actor message ordering",
                severity: LintLevel::Deny,
            })
            .collect()
    }
}
```

### 22.4 AST-Based Lints

```rust
impl AstLintPass {
    // Naming conventions
    fun lint_naming(&self, item: &Item) -> Option<LintDiagnostic> {
        match item {
            Item::Function(f) if !f.name.is_snake_case() => {
                Some(LintDiagnostic {
                    id: NON_SNAKE_CASE_FUNCTION,
                    message: format!("function `{}` should be snake_case", f.name),
                    suggestion: Some(f.name.to_snake_case()),
                    machine_applicable: true,
                })
            }
            Item::Struct(s) if !s.name.is_pascal_case() => {
                Some(LintDiagnostic {
                    id: NON_PASCAL_CASE_TYPE,
                    message: format!("type `{}` should be PascalCase", s.name),
                    suggestion: Some(s.name.to_pascal_case()),
                    machine_applicable: true,
                })
            }
            _ => None,
        }
    }
    
    // Pipeline opportunities
    fun lint_pipeline_opportunity(&self, expr: &Expr) -> Option<LintDiagnostic> {
        if let Expr::MethodChain(chain) = expr {
            if chain.length() >= 3 && !chain.uses_pipeline() {
                return Some(LintDiagnostic {
                    id: PREFER_PIPELINE,
                    message: "Long method chain could use pipeline operator",
                    suggestion: Some(chain.to_pipeline()),
                    machine_applicable: true,
                });
            }
        }
        None
    }
}
```

### 22.5 Incremental Linting

```rust
pub struct IncrementalLinter {
    previous_state: LintState,
    dependency_graph: DependencyGraph,
    cache: DashMap<FileId, LintResults>,
}

impl IncrementalLinter {
    pub fun lint_changed(&mut self, changes: &[FileChange]) -> LintResults {
        // Only re-lint affected modules
        let affected = self.dependency_graph.affected_modules(changes);
        
        let mut results = LintResults::new();
        for module in affected {
            // Check cache validity
            if let Some(cached) = self.cache.get(&module.id) {
                if cached.is_valid_for(module.hash) {
                    results.merge(cached.clone());
                    continue;
                }
            }
            
            // Perform linting
            let module_results = self.lint_module(module);
            self.cache.insert(module.id, module_results.clone());
            results.merge(module_results);
        }
        
        results
    }
}
```

### 22.6 Configuration

```toml
# .ruchy-lint.toml
[lints]
# Correctness lints cannot be disabled
correctness = "forbid"

# Performance lints
performance.unnecessary_clone = "warn"
performance.suboptimal_collection = "deny"
performance.complexity_threshold = 10

# Style preferences
style.naming_convention = "rust_standard"
style.max_line_length = 100
style.import_grouping = "std_external_local"

# Ruchy-specific
ruchy.prefer_pipeline = "warn"
ruchy.prefer_dataframe = "warn"
ruchy.actor_exhaustive = "deny"

# Project-specific overrides
[[overrides]]
path = "src/experimental/**"
lints.performance = "allow"

[[overrides]]
path = "src/generated/**"
lints.style = "allow"
```

### 22.7 Auto-Fix Implementation

```rust
pub struct AutoFixer {
    fixes: Vec<Fix>,
    source: SourceMap,
}

impl AutoFixer {
    pub fun apply_fixes(&mut self, diagnostics: &[LintDiagnostic]) -> Result<()> {
        // Collect machine-applicable fixes
        let applicable: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.machine_applicable && d.suggestion.is_some())
            .collect();
        
        // Sort by span to avoid conflicts
        let mut fixes = self.organize_fixes(applicable)?;
        
        // Apply in reverse order to preserve spans
        fixes.sort_by_key(|f| std::cmp::Reverse(f.span.start));
        
        for fix in fixes {
            self.source.replace_range(fix.span, &fix.replacement);
        }
        
        Ok(())
    }
    
    fun organize_fixes(&self, diagnostics: &[&LintDiagnostic]) -> Result<Vec<Fix>> {
        // Detect and resolve conflicts
        let mut fixes = Vec::new();
        let mut occupied_spans = IntervalTree::new();
        
        for diag in diagnostics {
            if !occupied_spans.overlaps(diag.span) {
                fixes.push(Fix {
                    span: diag.span,
                    replacement: diag.suggestion.clone().unwrap(),
                });
                occupied_spans.insert(diag.span);
            }
        }
        
        Ok(fixes)
    }
}
```

### 22.8 Integration with Quality Gates

```rust
impl QualityGate for LintEngine {
    fun check(&self, results: &LintResults) -> GateStatus {
        // Any deny-level lint fails the gate
        if results.has_errors() {
            return GateStatus::Failed {
                reason: format!("{} lint errors", results.error_count()),
            };
        }
        
        // Warning threshold check
        if results.warning_count() > self.config.max_warnings {
            return GateStatus::Failed {
                reason: format!("Too many warnings: {} > {}", 
                    results.warning_count(), 
                    self.config.max_warnings),
            };
        }
        
        // Complexity metrics
        for func in results.functions() {
            if func.complexity > 10 {
                return GateStatus::Failed {
                    reason: format!("Function {} exceeds complexity limit", func.name),
                };
            }
        }
        
        GateStatus::Passed
    }
}
```

### 22.9 Custom Lint Rules

```rust
// User-defined lints via procedural macros
#[ruchy_lint]
pub fun no_magic_numbers(expr: &Expr) -> Option<LintDiagnostic> {
    if let Expr::Literal(Literal::Integer(n)) = expr {
        if *n != 0 && *n != 1 && !expr.in_const_context() {
            return Some(LintDiagnostic {
                id: MAGIC_NUMBER,
                message: format!("Magic number {} should be a named constant", n),
                suggestion: None,
                machine_applicable: false,
            });
        }
    }
    None
}

// Register custom lints
impl LintRegistry {
    pub fun register_plugin(&mut self, plugin: LintPlugin) {
        for rule in plugin.rules() {
            self.rules.insert(rule.id.clone(), rule);
        }
    }
}
```

## 31. Language Completeness Validation

### 31.1 Native Tool Validation Protocol

**MANDATORY**: All language completeness documentation (LANG-COMP tickets) MUST validate features using ALL 15 native Ruchy tools.

**CRITICAL REQUIREMENT**: Each LANG-COMP test MUST be named `test_langcomp_XXX_YY_feature` and invoke ALL 15 tools as acceptance criteria.

### 31.2 Test Pattern (MANDATORY/BLOCKING)

```rust
#[test]
fn test_langcomp_XXX_YY_feature_name() {
    let example = example_path("XX-feature/YY_example.ruchy");

    // TOOL 1-15: Invoke ALL tools via assert_cmd
    ruchy_cmd().arg("check").arg(&example).assert().success();
    ruchy_cmd().arg("transpile").arg(&example).assert().success();
    // ... (repl skipped - interactive)
    ruchy_cmd().arg("lint").arg(&example).assert().success();
    ruchy_cmd().arg("compile").arg(&example).assert().success();
    ruchy_cmd().arg("run").arg(&example).assert().success();
    ruchy_cmd().arg("coverage").arg(&example).assert().success();
    ruchy_cmd().arg("runtime").arg(&example).arg("--bigo").assert().success();
    ruchy_cmd().arg("ast").arg(&example).assert().success();
    ruchy_cmd().arg("wasm").arg(&example).assert().success();
    ruchy_cmd().arg("provability").arg(&example).assert().success();
    ruchy_cmd().arg("property-tests").arg(&example).assert().success();
    ruchy_cmd().arg("mutations").arg(&example).assert().success();
    ruchy_cmd().arg("fuzz").arg(&example).assert().success();
    // ... (notebook skipped - requires server)
}
```

**ACCEPTANCE CRITERIA**: Test passes ONLY if ALL 15 tools succeed.

### 31.3 15 Native Tool Validation Requirements

Every example in language completeness documentation MUST pass validation from ALL 15 tools:

#### 1. **`ruchy check`** - Syntax Validation
- Fast syntax-only validation without execution
- Validates: Parse correctness, syntax errors
- Output: Success/failure with error locations
- Use: Pre-flight check before other validations

#### 2. **`ruchy transpile`** - Rust Code Generation
- Generate readable Rust code from Ruchy source
- Validates: Transpilation correctness, code generation
- Output: Rust source code (.rs file)
- Use: Verify transpiler generates valid Rust

#### 3. **`ruchy repl`** - Interactive Validation
- Test examples in REPL environment
- Validates: Interactive behavior, incremental execution
- Output: REPL session results
- Use: Manual verification of interactive features

#### 4. **`ruchy lint`** - Static Analysis
- Zero linting issues for all examples
- Validates: Variable usage, type correctness, style compliance
- Blocks: Unused variables, undefined variables, shadowing issues

#### 5. **`ruchy compile`** - Compilation
- Successful compilation to Rust
- Validates: Syntax correctness, type inference, transpilation
- Blocks: Parse errors, type errors, transpilation failures

#### 6. **`ruchy run`** - Execution
- Correct execution with expected output
- Validates: Runtime behavior, output correctness
- Blocks: Runtime errors, incorrect results, crashes

#### 7. **`ruchy coverage`** - Test Coverage Analysis
- Example must have corresponding property tests
- Target: ≥80% line coverage per module
- Validates: Test completeness, code reachability
- Output: Line/branch/function coverage metrics

#### 8. **`ruchy big-o`** - Algorithmic Complexity Analysis
- Automatic complexity class detection (O(1), O(n), O(n²), etc.)
- Validates: Performance characteristics, scalability
- Output: Complexity class, analysis justification, optimization suggestions
- Blocks: Exponential complexity without justification

#### 9. **`ruchy ast`** - AST Inspection
- Display Abstract Syntax Tree for verification
- Validates: Parse tree correctness, semantic structure
- Output: Formatted AST with type annotations
- Use: Verify parser handles syntax correctly

#### 10. **`ruchy wasm`** - WASM Compilation
- Compile example to WebAssembly
- Validates: WASM backend correctness, browser compatibility
- Output: .wasm module with size metrics
- Blocks: WASM-incompatible features without fallback

#### 11. **`ruchy provability`** - Formal Verification
- Generate verification conditions for correctness
- Validates: Logical soundness, invariant preservation
- Output: Proof obligations, SMT solver results
- Use: Critical algorithms, security-sensitive code

#### 12. **`ruchy property-tests`** - Property-Based Testing
- Run property-based tests with thousands of generated test cases
- Validates: Mathematical invariants, roundtrip properties, metamorphic relations
- Output: Test results, failure cases with shrinking
- Requirement: ≥10,000 test cases per property, 100% success rate

#### 13. **`ruchy mutations`** - Mutation Testing
- Validate test suite quality by introducing deliberate bugs
- Validates: Test effectiveness, edge case coverage
- Output: Mutation score, surviving mutants, killed mutants
- Requirement: ≥75% mutation coverage (target: 95%+)

#### 14. **`ruchy fuzz`** - Fuzz Testing
- Stress-test code with millions of random inputs to find crashes
- Validates: Robustness, panic-freedom, undefined behavior detection
- Output: Crash reports, corpus of interesting inputs
- Requirement: ≥1M iterations, zero crashes/panics

#### 15. **`ruchy notebook`** - Interactive WASM Notebook Server
- Launch interactive notebook server for browser-based development
- Validates: WASM integration, interactive execution, real-time feedback
- Output: HTTP server on localhost with live code evaluation
- Use: Developer experience, tutorials, live documentation, WASM demos
- **CRITICAL for WASM notebooks and interactive development**

### 31.3 Validation Workflow

```bash
# MANDATORY validation sequence for each LANG-COMP example (ALL 15 TOOLS):

# 1. Syntax check (fast pre-flight validation)
ruchy check examples/lang_comp/XX-feature/example.ruchy || exit 1

# 2. Transpile to Rust (verify code generation)
ruchy transpile examples/lang_comp/XX-feature/example.ruchy --output=example.rs || exit 1

# 3. REPL validation (interactive behavior check)
echo "load examples/lang_comp/XX-feature/example.ruchy" | ruchy repl || exit 1

# 4. Lint check (zero issues required)
ruchy lint examples/lang_comp/XX-feature/example.ruchy || exit 1

# 5. Compilation (must succeed)
ruchy compile examples/lang_comp/XX-feature/example.ruchy || exit 1

# 6. Execution (verify output)
ruchy run examples/lang_comp/XX-feature/example.ruchy > actual_output.txt
diff actual_output.txt expected_output.txt || exit 1

# 7. Coverage analysis (≥80% target)
ruchy coverage tests/lang_comp/XX_feature_test.rs --min-coverage 80 || exit 1

# 8. Complexity analysis (O(n) or better preferred)
ruchy big-o examples/lang_comp/XX-feature/example.ruchy --max-class quadratic || exit 1

# 9. AST verification (inspect structure)
ruchy ast examples/lang_comp/XX-feature/example.ruchy --format=debug > ast_output.txt

# 10. WASM compilation (if applicable)
ruchy wasm examples/lang_comp/XX-feature/example.ruchy --output=example.wasm || exit 1

# 11. Provability check (if applicable to algorithm)
ruchy provability examples/lang_comp/XX-feature/example.ruchy --generate-proofs || exit 1

# 12. Property-based tests (≥10K cases, 100% pass rate)
ruchy property-tests tests/lang_comp/XX_feature_test.rs --cases 10000 || exit 1

# 13. Mutation tests (≥75% coverage)
ruchy mutations tests/lang_comp/XX_feature_test.rs --min-coverage 0.75 || exit 1

# 14. Fuzz tests (≥1M iterations, zero crashes)
ruchy fuzz parse_XX_feature --iterations 1000000 || exit 1

# 15. Notebook server (verify help works - server start tested separately)
ruchy notebook --help || exit 1
```

### 31.4 Documentation Requirements

Each LANG-COMP chapter MUST document:

```markdown
**Validated with**:
- ✅ `ruchy check` - Syntax validated successfully
- ✅ `ruchy transpile` - Generated valid Rust code (142 LOC)
- ✅ `ruchy repl` - Interactive execution verified
- ✅ `ruchy lint` - Zero issues
- ✅ `ruchy compile` - Successful compilation
- ✅ `ruchy run` - Executes correctly, output matches expected
- ✅ `ruchy coverage` - 82.3% line coverage (target: ≥80%)
- ✅ `ruchy big-o` - O(n) complexity detected, optimal
- ✅ `ruchy ast` - Parse tree verified, all nodes correct
- ✅ `ruchy wasm` - 1.2KB .wasm module generated
- ✅ `ruchy provability` - 3/3 proof obligations satisfied (Z3 solver)
- ✅ `ruchy property-tests` - 10,000 cases passed (100% success rate)
- ✅ `ruchy mutations` - 95.6% mutation coverage (43/45 mutations killed)
- ✅ `ruchy fuzz` - 1,000,000 iterations completed (0 crashes)
- ✅ `ruchy notebook` - Server ready, WASM integration verified
```

### 31.5 Tool Implementation Status

**ALL 15 TOOLS ARE MANDATORY AND BLOCKING**

| Tool | Status | Coverage | Requirement | Notes |
|------|--------|----------|-------------|-------|
| `ruchy check` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Syntax validation without execution |
| `ruchy transpile` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Generates readable Rust code |
| `ruchy repl` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Interactive REPL with replay support |
| `ruchy lint` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Full linter with scope analysis |
| `ruchy compile` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Compiles to standalone binary |
| `ruchy run` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Script execution validation |
| `ruchy coverage` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Code coverage analysis |
| `ruchy runtime --bigo` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Algorithmic complexity detection |
| `ruchy ast` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | AST pretty-printer |
| `ruchy wasm` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | WASM compilation |
| `ruchy provability` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Formal verification |
| `ruchy property-tests` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Property-based testing with 10K+ cases |
| `ruchy mutations` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Mutation testing |
| `ruchy fuzz` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | Fuzz testing |
| `ruchy notebook` | ✅ Implemented | 100% | **MANDATORY/BLOCKING** | **WASM notebook server (CRITICAL for DX)** |

### 31.6 Quality Gates Integration

**Pre-commit Hook** must verify all 15 tools on changed examples:

```bash
#!/bin/bash
# .git/hooks/pre-commit - LANG-COMP validation (ALL 15 TOOLS MANDATORY/BLOCKING)

changed_examples=$(git diff --cached --name-only | grep "examples/lang_comp/.*\.ruchy$")

for example in $changed_examples; do
    echo "Validating $example with 15 native tools (ALL MANDATORY)..."

    # ALL 15 TOOLS ARE MANDATORY AND BLOCKING
    ruchy check "$example" || exit 1
    ruchy transpile "$example" --output="${example%.ruchy}.rs" || exit 1
    echo "load $example" | ruchy repl || exit 1
    ruchy lint "$example" || exit 1
    ruchy compile "$example" || exit 1
    ruchy run "$example" > /dev/null || exit 1
    ruchy coverage "${example%.ruchy}_test.rs" --min-coverage 80 || exit 1
    ruchy big-o "$example" --max-class quadratic || exit 1
    ruchy ast "$example" --validate || exit 1
    ruchy wasm "$example" --output="${example%.ruchy}.wasm" || exit 1
    ruchy provability "$example" --generate-proofs || exit 1
    ruchy property-tests "${example%.ruchy}_test.rs" --cases 10000 || exit 1
    ruchy mutations "${example%.ruchy}_test.rs" --min-coverage 0.75 || exit 1
    ruchy fuzz "parse_${example##*/}" --iterations 1000000 || exit 1
done

echo "✅ All LANG-COMP examples validated with 14 tools"
```

### 31.7 LANG-COMP Ticket Requirements

Every LANG-COMP-XXX ticket MUST include:

1. **Feature Description**: What language feature is being documented
2. **Example Files**: Minimum 3 runnable examples demonstrating feature
3. **Property Tests**: 10K+ test cases validating feature correctness
4. **8-Tool Validation**: Results from all 8 native tools documented
5. **Quality Metrics**: Coverage ≥80%, complexity ≤10, zero SATD
6. **Documentation Chapter**: Comprehensive README.md with validation results

### 31.8 Automation

**CI Pipeline** must run full 8-tool validation on every PR:

```yaml
# .github/workflows/lang-comp-validation.yml
name: Language Completeness Validation

on: [pull_request]

jobs:
  validate-examples:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run 8-tool validation
        run: |
          for example in examples/lang_comp/**/*.ruchy; do
            ./scripts/validate_8_tools.sh "$example" || exit 1
          done
```

## 23. Master TODO

### 22.1 Implementation Roadmap (REVISED)

```yaml
phase_0_foundation:  # Weeks 0-2 (CRITICAL - blocks everything)
  - [ ] Fix 124 SATD comments (zero tolerance)
  - [ ] Parser completion (remaining 30%)
  - [ ] Test coverage to 80% minimum
  - [ ] Reduce all functions to complexity ≤10
  - [ ] CI enforcement of quality gates
  - [ ] Implement core lint infrastructure

phase_1_mvt:  # Weeks 3-6 (Minimal Viable Transpiler)
  - [ ] Basic type inference (no generics)
  - [ ] Direct syn generation (no MIR)
  - [ ] DataFrame literal support only
  - [ ] Simple function transpilation
  - [ ] Basic Rust code emission

phase_2_interactive:  # Weeks 7-10
  - [ ] Tree-walk interpreter for REPL
  - [ ] Basic DataFrame operations (filter, select)
  - [ ] Pipeline operator implementation
  - [ ] Error recovery in parser
  - [ ] REPL with syntax highlighting

phase_3_mir_architecture:  # Weeks 11-14
  - [ ] MIR representation design
  - [ ] AST → MIR lowering
  - [ ] DataFrame IR fusion
  - [ ] Basic MIR optimization
  - [ ] MIR → Rust codegen

phase_4_core_features:  # Weeks 15-20
  - [ ] Full type inference with generics
  - [ ] Pattern matching compilation
  - [ ] Actor system basics
  - [ ] String interpolation
  - [ ] List comprehensions

phase_5_tooling:  # Weeks 21-26
  - [ ] LSP basic implementation
  - [ ] VS Code extension
  - [ ] Cargo integration finalization
  - [ ] Documentation generator
  - [ ] Benchmark suite

phase_6_optimization:  # Weeks 27-32
  - [ ] Cranelift JIT integration
  - [ ] Incremental compilation
  - [ ] Query optimization for DataFrames
  - [ ] Memory pooling
  - [ ] SIMD vectorization

phase_7_advanced:  # Future (post-v1.0)
  - [ ] Refinement types (SMT)
  - [ ] Effect system
  - [ ] Row polymorphism
  - [ ] Symbolic mathematics
  - [ ] WASM backend
```

## 23. Project Status

### 23.1 Current Metrics (REALITY CHECK)

```rust
pub struct ProjectStatus {
    version: Version,           // 0.3.0-alpha
    loc: usize,                 // 15,234
    test_count: usize,          // 342
    test_coverage: f64,         // 65.3% → MUST reach 80%
    satd_count: usize,          // 124 → MUST be 0
    max_complexity: u32,        // 37 → MUST be ≤10
    documentation: f64,         // 45.2% → target 90%
    dependencies: usize,        // 47
    compile_time: Duration,     // 12.3s
    binary_size: usize,        // 4.2 MB
}

// Quality enforcement via CI
#[cfg(ci)]
compile_error_if!(coverage < 80.0, "Coverage must be ≥80% for CI");
compile_error_if!(satd_count > 0, "Zero SATD tolerance in CI");
compile_error_if!(max_complexity > 10, "Max complexity is 10");

pub struct CriticalPath {
    // Phase 0 is MANDATORY before any features
    phase_0_blockers: vec![
        "124 SATD comments removal",
        "15% coverage increase",
        "27 functions need complexity reduction",
        "Parser completion (30% remaining)",
    ],
    estimated_days: 14,
    blocking_all_features: true,
}
```

## 24. Deep Context

### 24.1 Architecture Insights

Key architectural decisions and their rationale:

1. **Transpilation over interpretation**: Leverage Rust ecosystem
2. **Polars as default**: DataFrame-first for data science
3. **Actor model**: Proven concurrency without shared state
4. **Hand-written parser**: Control and error messages
5. **Property testing**: Correctness over coverage

### 24.2 Performance Characteristics

```rust
pub struct PerformanceProfile {
    startup_time: Duration,        // <10ms target
    repl_latency: Duration,       // <15ms target
    transpile_speed: f64,         // 100K loc/s
    runtime_overhead: f64,        // <5% vs handwritten Rust
    memory_overhead: f64,         // <10% vs handwritten Rust
}

// Benchmark suite
#[bench]
fun bench_fibonacci(b: &mut Bencher) {
    let ruchy = "fun fib(n) = if n < 2 { n } else { fib(n-1) + fib(n-2) }";
    let rust = compile_to_rust(ruchy);
    
    b.iter(|| {
        black_box(execute_rust(&rust, 30))
    });
}
```

## 25. PMAT Integration

### 25.1 Quality Enforcement via PMAT

```rust
pub struct PmatIntegration {
    quality_proxy: QualityProxy,
    analyzer: PmatAnalyzer,
}

impl PmatIntegration {
    pub async fun validate_code(&self, code: &str) -> ValidationResult {
        let metrics = self.analyzer.analyze(code).await?;
        
        // Enforce Toyota Way standards
        if metrics.complexity > 10 {
            return Err(ValidationError::ComplexityExceeded {
                current: metrics.complexity,
                max: 10,
            });
        }
        
        if metrics.satd_count > 0 {
            return Err(ValidationError::TechnicalDebt {
                count: metrics.satd_count,
            });
        }
        
        if metrics.coverage < 80.0 {
            return Err(ValidationError::InsufficientCoverage {
                current: metrics.coverage,
                min: 80.0,
            });
        }
        
        Ok(ValidationResult::Pass)
    }
    
    pub async fun suggest_improvements(&self, code: &str) -> Vec<Suggestion> {
        self.quality_proxy.analyze_and_suggest(code).await
    }
}
```

### 25.2 MCP Tool Integration

```rust
// PMAT provides 18 MCP tools via unified pmcp SDK
pub struct PmatMcpServer {
    server: McpServer,
    tools: Vec<McpTool>,
}

impl PmatMcpServer {
    pub fun new() -> Self {
        let tools = vec![
            // Analysis tools
            McpTool::new("analyze_complexity", analyze_complexity_handler),
            McpTool::new("analyze_deep_context", analyze_deep_context_handler),
            McpTool::new("analyze_big_o", analyze_big_o_handler),
            McpTool::new("analyze_dead_code", analyze_dead_code_handler),
            McpTool::new("analyze_satd", analyze_satd_handler),
            
            // Quality tools
            McpTool::new("quality_gate", quality_gate_handler),
            McpTool::new("quality_proxy", quality_proxy_handler),
            
            // Refactoring tools
            McpTool::new("refactor_start", refactor_start_handler),
            McpTool::new("refactor_next", refactor_next_handler),
            
            // Project tools
            McpTool::new("scaffold_project", scaffold_project_handler),
            McpTool::new("scaffold_agent", scaffold_agent_handler),
        ];
        
        PmatMcpServer {
            server: McpServer::new(tools),
            tools,
        }
    }
}

// Example: Complexity analysis with composition
async fun analyze_complexity_handler(params: Value) -> Result<Value> {
    let path = params["path"].as_str().unwrap();
    let include_cognitive = params["include_cognitive"].as_bool().unwrap_or(true);
    
    let analysis = ComplexityAnalyzer::new()
        .with_cognitive(include_cognitive)
        .analyze_path(path)?;
    
    Ok(json!({
        "cyclomatic": analysis.cyclomatic,
        "cognitive": analysis.cognitive,
        "hotspots": analysis.hotspots,
        "suggestions": analysis.suggestions,
    }))
}
```