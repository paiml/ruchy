# Sub-spec: Specifications Update Oct — Core Features & Transpilation Architecture

**Parent:** [SPECIFICATIONS-UPDATE-OCT.md](../SPECIFICATIONS-UPDATE-OCT.md) Sections 4-5

---
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
        Lexer,           // Source → Tokens
        Parser,          // Tokens → AST
        TypeChecker,     // AST → TypedAST
        MirGenerator,    // TypedAST → MIR
        Optimizer,       // MIR → OptimizedMIR
        CodeGenerator,   // MIR → Rust AST
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

## 6. Interpreter Specification

### 6.1 Tree-Walk Interpreter

For REPL and rapid prototyping:

```rust
pub struct Interpreter {
    globals: Environment,
    locals: Stack<Environment>,
    heap: Arena<Value>,
}

pub enum Value {
    // Stack allocated
    Int(i64),
    Float(f64),
    Bool(bool),
    
    // Heap allocated
    String(ArenaRef<String>),
    DataFrame(ArenaRef<DataFrame>),
    Function(ArenaRef<Closure>),
    
    // Special
    Null,
}
```

### 6.2 Execution Strategy

```rust
impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        self.heat_counter.track(expr);
        
        match self.heat_counter.get_heat(expr) {
            0..=2 => self.interpret(expr),      // Cold: interpret
            3..=10 => self.jit_compile(expr),   // Warm: JIT
            _ => self.aot_compile(expr),        // Hot: full compile
        }
    }
}
```

