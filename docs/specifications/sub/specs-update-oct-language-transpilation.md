# Sub-spec: Specifications Update Oct -- Language, Grammar, Types, and Transpilation

**Parent:** [SPECIFICATIONS-UPDATE-OCT.md](../SPECIFICATIONS-UPDATE-OCT.md) Sections 1-5

---

*Canonical reference for the Ruchy scripting language that transpiles to Rust*

## 1. Language Overview

### 1.1 Design Philosophy

Ruchy achieves Python-like ergonomics through mechanical transformation to idiomatic Rust. Core principles:

- **Zero-cost abstractions**: All features compile to efficient Rust
- **DataFrame-first**: Collections default to Polars types
- **Progressive complexity**: Simple code remains simple
- **Type inference**: Explicit types only at module boundaries
- **Direct transpilation**: Source maps 1:1 to Rust constructs

### 1.2 Execution Modes

```rust
pub enum ExecutionMode {
    Script,       // .ruchy files -> Rust transpilation
    Repl,         // Interactive -> tree-walk interpreter
    Compiled,     // AOT -> native binary via cargo
    OneLiner,     // -e flag -> immediate evaluation
}
```

### 1.3 Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| REPL startup | <10ms | Interactive responsiveness |
| REPL response | <15ms | Perceived instant |
| Transpile speed | 100K LoC/s | CI/CD viability |
| Runtime overhead | <5% | vs handwritten Rust |
| Binary size | <5MB | Minimal runtime |

## 2. Grammar Definition

### 2.1 Formal Grammar (EBNF)

```ebnf
program         = item*
item            = function | struct_def | enum_def | trait_def 
                | impl_block | actor_def | import_stmt | type_alias

// Functions
function        = 'fun' identifier generic_params? '(' params? ')' 
                  return_type? where_clause? (block | '=' expr)
params          = param (',' param)*
param           = identifier ':' type default_value?
default_value   = '=' expr
return_type     = '->' type

// Expressions
expr            = assignment
assignment      = pipeline ('=' assignment)?
pipeline        = logical_or ('>>' pipeline)*    // Note: >> not |>
logical_or      = logical_and ('||' logical_and)*
logical_and     = equality ('&&' equality)*
equality        = comparison (('==' | '!=') comparison)*
comparison      = term (('<' | '<=' | '>' | '>=') term)*
term            = factor (('+' | '-') factor)*
factor          = unary (('*' | '/' | '%' | '**') unary)*
unary           = ('!' | '-' | 'await')? postfix
postfix         = primary suffix*
suffix          = '.' identifier | '[' expr ']' | '(' args? ')' | '?'

primary         = literal | identifier | '(' expr ')' | if_expr 
                | match_expr | for_expr | while_expr | loop_expr
                | lambda | array_expr | tuple_expr | try_expr
                | actor_send | actor_ask | string_interp

// Lambda - single canonical form
lambda          = '|' params? '|' ('->' type)? (expr | block)

// Try-catch
try_expr        = 'try' block catch_clause+ finally_clause?
catch_clause    = 'catch' pattern ('if' expr)? block
finally_clause  = 'finally' block

// Pattern matching
match_expr      = 'match' expr '{' match_arm (',' match_arm)* '}'
match_arm       = pattern ('if' expr)? '=>' expr

// Actor operations
actor_send      = expr '<-' expr    // Fire and forget
actor_ask       = expr '<?' expr    // Request-reply

// String interpolation
string_interp   = 'f"' (text | '{' expr '}')* '"'

// Patterns
pattern         = literal | identifier | '_' | tuple_pattern 
                | array_pattern | struct_pattern | enum_pattern

// Types
type            = primitive | array_type | tuple_type | function_type
                | generic_type | reference_type
primitive       = 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
                | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
                | 'f32' | 'f64' | 'bool' | 'char' | 'String'
array_type      = '[' type ']'
function_type   = 'fun' '(' types? ')' '->' type
reference_type  = '&' 'mut'? type
```

### 2.2 Keywords

```
fun let var const if else match for while loop break continue
return struct enum trait impl actor receive send async await
try catch finally throw import export module pub mut
type alias where in is as true false null
df col mean std quantile filter groupby agg sort select
```

### 2.3 Operator Precedence

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `.` `?.` | Left |
| 2 | `()` `[]` | Left |
| 3 | `!` `-` (unary) `await` | Right |
| 4 | `**` | Right |
| 5 | `*` `/` `%` | Left |
| 6 | `+` `-` | Left |
| 7 | `<<` `>>` (shift) | Left |
| 8 | `<` `<=` `>` `>=` | Left |
| 9 | `==` `!=` | Left |
| 10 | `&&` | Left |
| 11 | `\|\|` | Left |
| 12 | `>>` (pipeline) | Left |
| 13 | `=` `+=` `-=` | Right |

## 3. Type System

### 3.1 Type Categories

```rust
// Primitive types
i8, i16, i32, i64, i128
u8, u16, u32, u64, u128
f32, f64
bool, char, String, ()

// Collection types (default to Polars)
[T]                  // -> Series
[[T]]               // -> DataFrame
Vec<T>              // Explicit Vec only
HashMap<K,V>        // Explicit HashMap only

// Composite types
(T1, T2, ...)       // Tuples
Option<T>           // Nullable
Result<T, E>        // Error handling

// Mathematical types
DataFrame           // Polars DataFrame
LazyFrame          // Lazy evaluation
Series             // Column data
Matrix<T>          // nalgebra
```

### 3.2 Type Inference

Bidirectional type checking with Hindley-Milner inference:

```rust
impl TypeChecker {
    fn infer(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Lambda { params, body, .. } => {
                let param_types = params.iter()
                    .map(|p| self.fresh_type_var())
                    .collect();
                let body_type = self.infer(body);
                Type::Function(param_types, Box::new(body_type))
            }
            Expr::Pipeline { left, right } => {
                // x >> f infers as f(x)
                let left_type = self.infer(left);
                let func_type = self.infer(right);
                self.apply_function(func_type, left_type)
            }
            _ => self.infer_standard(expr),
        }
    }
}
```

### 3.3 Collection Type Mapping

Arrays and array literals default to Polars types:

```rust
[1, 2, 3]           // -> Series::new("", &[1, 2, 3])
[[1, 2], [3, 4]]    // -> df!["c0" => [1,3], "c1" => [2,4]]

// Explicit Rust collections require type annotation
let v: Vec<i32> = vec![1, 2, 3];
```

## 4. Core Language Features

### 4.1 Functions

```rust
// Basic function
fun add(x: i32, y: i32) -> i32 {
    x + y
}

// Expression body
fun double(x: i32) = x * 2

// Default parameters
fun greet(name: String, greeting = "Hello") {
    println!(f"{greeting}, {name}!")
}

// Generic functions
fun map<T, U>(list: [T], f: fun(T) -> U) -> [U] {
    list.iter().map(f).collect()
}
```

### 4.2 Pattern Matching

```rust
match value {
    0 => "zero",
    1 | 2 => "small",
    n if n > 10 => "large",
    _ => "other"
}

// List patterns
match list {
    [] => "empty",
    [x] => f"single: {x}",
    [head, ...tail] => f"head: {head}, rest: {tail.len()}",
}
```

### 4.3 Error Handling

Dual paradigm: try/catch for imperative, Result for functional:

```rust
// Try-catch for multi-step operations
try {
    let conn = db.connect()?;
    let data = conn.query(sql)?;
    conn.commit()?;
} catch DbError(e) {
    log_and_retry(e)
} catch ParseError(e) if e.is_recoverable() {
    use_default()
} finally {
    cleanup()
}

// Result chains for pipelines
db.connect()
    .and_then(|conn| conn.query(sql))
    .map(transform)
    .unwrap_or_else(|e| default)
```

### 4.4 Pipeline Operator

```rust
// Pipeline operator >> for function composition
data 
    >> filter(|x| x > 0)
    >> map(|x| x * 2)
    >> reduce(0, |a, b| a + b)

// Transpiles to method chain
data.filter(|x| x > 0)
    .map(|x| x * 2)
    .reduce(0, |a, b| a + b)
```

### 4.5 String Interpolation

```rust
let name = "Alice";
let age = 30;
let msg = f"Hello, {name}! You are {age} years old.";

// Expressions in interpolation
let result = f"The answer is {2 + 2}";

// Format specifiers
let pi = 3.14159;
let formatted = f"Pi to 2 places: {pi:.2}";
```

### 4.6 Actors

```rust
actor Counter {
    count: i32,
    
    receive {
        Inc => self.count += 1,
        Dec => self.count -= 1,
        Get => reply(self.count),
    }
}

let counter = spawn Counter { count: 0 };
counter <- Inc;                    // Fire and forget
let value = counter <? Get;        // Request-reply
```

### 4.7 DataFrames

```rust
// DataFrame literals
let df = df![
    "name" => ["Alice", "Bob"],
    "age" => [30, 25]
];

// Operations default to lazy evaluation
let result = df
    >> filter(col("age") > 25)
    >> groupby("department")
    >> agg([
        col("salary").mean().alias("avg_salary"),
        col("name").count().alias("count")
    ]);
```

## 5. Transpilation Architecture

### 5.1 Pipeline Stages

```rust
pub struct TranspilationPipeline {
    stages: [
        Lexer,           // Source -> Tokens
        Parser,          // Tokens -> AST
        TypeChecker,     // AST -> TypedAST
        MirGenerator,    // TypedAST -> MIR
        Optimizer,       // MIR -> OptimizedMIR
        CodeGenerator,   // MIR -> Rust AST
    ]
}
```

### 5.2 MIR (Mid-level IR)

```rust
pub enum MirNode {
    // Core constructs
    Let { binding: Ident, value: Box<MirNode> },
    Function { params: Vec<Param>, body: Box<MirNode> },
    Application { func: Box<MirNode>, args: Vec<MirNode> },
    
    // DataFrame operations (for fusion)
    DataFrameOp { op: DfOp, input: Box<MirNode> },
    LazyDataFrameOp { op: DfOp, input: Box<MirNode> },
    
    // Actor operations
    ActorSpawn { actor: ActorDef },
    ActorSend { target: Box<MirNode>, msg: Box<MirNode> },
    
    // Try-catch lowers to Result
    TryCatch { body: Box<MirNode>, handlers: Vec<Handler> },
}
```

### 5.3 Optimization Passes

```rust
impl MirOptimizer {
    pub fn optimize(&mut self, mir: MirNode) -> MirNode {
        mir
            .inline_small_functions()
            .fuse_dataframe_ops()      // Combine adjacent DF operations
            .eliminate_dead_code()
            .constant_fold()
            .escape_analysis()          // Determine borrowing
    }
}
```
