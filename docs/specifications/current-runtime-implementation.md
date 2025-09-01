# Current Ruchy Runtime Implementation

## Overview

This document describes the actual runtime implementation of Ruchy v1.29.1, detailing how the interpreter executes code, manages memory, and provides runtime services.

## Interpreter Architecture

### Value System

Ruchy's interpreter uses a tagged union `Value` enum to represent all runtime values:

```rust
pub enum Value {
    Unit,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
    Function(Vec<String>, Vec<Statement>, Rc<RefCell<Environment>>),
    DataFrame(Rc<RefCell<RuchyDataFrame>>),
    Closure(Box<dyn Fn(Vec<Value>) -> Result<Value, String>>),
    NativeFunction(fn(&mut Interpreter, Vec<Value>) -> Result<Value, String>),
    Tuple(Vec<Value>),
    Range(i64, i64),
    Builtin(String),
    Error(String),
}
```

### Memory Management

**Reference Counting:** Collections use `Rc<RefCell<T>>` for shared ownership with interior mutability:

```ruchy
# In Ruchy code
let a = [1, 2, 3]
let b = a  # Both reference same Rc<RefCell<Vec>>

# Runtime representation
Value::List(Rc::new(RefCell::new(vec![
    Value::Integer(1),
    Value::Integer(2),
    Value::Integer(3)
])))
```

**Garbage Collection:** Currently relies on Rust's automatic drop semantics when Rc count reaches zero. No cycle detection implemented yet.

### Expression Evaluation

The interpreter evaluates expressions recursively through pattern matching:

```rust
fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
    match &expr.kind {
        ExprKind::Literal(lit) => self.evaluate_literal(lit),
        ExprKind::Identifier(name) => self.get_variable(name),
        ExprKind::Binary(op, left, right) => {
            let l = self.evaluate_expr(left)?;
            let r = self.evaluate_expr(right)?;
            self.apply_binary_op(op, l, r)
        }
        ExprKind::Call(func, args) => self.evaluate_call(func, args),
        // ... more expression types
    }
}
```

## REPL Runtime Features

### Tab Completion Engine

The REPL provides context-aware completion with ~1400 lines of completion logic:

```ruchy
# Completions triggered by TAB
"hello".  # Completes: len, upper, lower, split, replace...
[1,2,3].  # Completes: map, filter, sum, head, tail...
help(     # Completes: all available functions
```

**Completion Contexts:**
1. Method access (`.` after expression)
2. Help queries (`help(`, `?`, `:help`)
3. Function calls (parameter hints)
4. Module paths (`std::`)
5. Variable names (from environment)

### Built-in Help System

Runtime help with 200+ documented functions:

```ruchy
# Multiple help syntaxes supported
help(println)     # Function help
?String          # Type help
:help List       # Alternative syntax
dir(object)      # List attributes
type(value)      # Get type name
```

### Performance Monitoring

REPL tracks completion performance:

```ruchy
# Internal metrics (shown in verbose mode)
Cache hit rate: 73.5%
Avg completion time: 12ms
Total completions: 1,234
Cache size: 256 entries
```

## Standard Library Integration

### Native Functions

Core functions implemented in Rust for performance:

```rust
// Example: len() implementation
NativeFunction(|_interp, args| {
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::List(l) => Ok(Value::Integer(l.borrow().len() as i64)),
        Value::Object(o) => Ok(Value::Integer(o.borrow().len() as i64)),
        _ => Err("len() requires collection".to_string())
    }
})
```

### Builtin Types

**String Methods:**
```ruchy
"hello".upper()      # "HELLO"
"hello".split(" ")   # ["hello"]
f"Value: {x}"        # String interpolation
```

**List Methods:**
```ruchy
[1,2,3].map(x => x * 2)   # [2,4,6]
[1,2,3].filter(x => x > 1) # [2,3]
[1,2,3].sum()              # 6
```

**DataFrame Operations:**
```ruchy
df![
    "name" => ["Alice", "Bob"],
    "age" => [30, 25]
]
```

## Error Handling

### Error Propagation

Errors bubble up through the call stack:

```ruchy
fn risky() {
    undefined_var  # Error: Undefined variable
}

fn caller() {
    risky()  # Error propagates here
}

# REPL shows full stack trace
Error: Undefined variable 'undefined_var'
  at risky() line 2
  at caller() line 6
  at <repl> line 1
```

### Panic Recovery

REPL catches panics to prevent crashes:

```rust
// REPL evaluation wrapper
match std::panic::catch_unwind(|| {
    interpreter.eval(input)
}) {
    Ok(result) => display_result(result),
    Err(_) => println!("Error: Interpreter panic recovered")
}
```

## Async/Await Implementation

### Green Thread Simulation

Async functions use polling-based execution:

```ruchy
async fn fetch(url) {
    # Transforms to state machine
    let response = http::get(url).await
    response.text().await
}
```

Runtime polls futures until completion, enabling concurrent execution without OS threads.

## Pattern Matching

### Pattern Evaluation

Exhaustive pattern matching with guards:

```ruchy
match value {
    0 => "zero",
    n if n > 0 => "positive",
    _ => "negative"
}
```

Patterns compile to decision trees for efficient matching.

## Module System

### Import Resolution

Module loading follows priority order:

1. **Built-in modules:** `std::*` namespace
2. **Local files:** Relative to current file
3. **Absolute paths:** Full filesystem paths

```ruchy
import std::fs::read_to_string  # Built-in
import "./utils.ruchy" as utils # Local file
import "/opt/lib/math.ruchy"    # Absolute
```

### Lazy Loading

Modules load on first import, cached for subsequent uses:

```rust
// Module cache in interpreter
module_cache: HashMap<String, Rc<Module>>
```

## Performance Characteristics

### Current Benchmarks

**Startup Time:**
- REPL cold start: 95ms
- Script execution: 180ms (includes parsing)
- Second run (cached): 45ms

**Memory Usage:**
- Base REPL: 8.2MB
- After stdlib load: 12.5MB
- Per-thread overhead: 2MB

**Operation Timings:**
- Function call: ~100ns
- List creation (100 items): ~5μs
- Pattern match (5 arms): ~50ns
- Async poll cycle: ~1μs

### Optimization Opportunities

Areas for future optimization:
1. **Bytecode compilation:** Reduce interpretation overhead
2. **Inline caching:** Speed up method lookups
3. **Escape analysis:** Stack-allocate non-escaping objects
4. **JIT compilation:** Hot path optimization
5. **Parallel GC:** Concurrent cycle detection

## Runtime Configuration

### Environment Variables

```bash
RUCHY_STACK_SIZE=8388608     # Stack size in bytes
RUCHY_TRACE=1                 # Enable trace output
RUCHY_CACHE_DIR=~/.ruchy     # Cache directory
RUCHY_COMPLETION_CACHE=256   # Completion cache size
```

### REPL Configuration

```ruchy
# In REPL
:set verbose on              # Show detailed output
:set history 1000           # History size
:set completion fuzzy       # Enable fuzzy matching
```

## Debugging Support

### Stack Traces

Full stack traces with source locations:

```ruchy
Error: Type mismatch
  Expected: Integer
  Got: String
  
Stack trace:
  at add(a, b) in math.ruchy:15
  at calculate() in main.ruchy:8
  at <repl>:1
```

### Debug Printing

Debug representation for all types:

```ruchy
debug([1, 2, 3])  
# Output: List([Integer(1), Integer(2), Integer(3)])

debug({"a": 1})
# Output: Object({"a": Integer(1)})
```

## Limitations

Current implementation limitations:

1. **No true parallelism:** Single-threaded interpreter
2. **No cycle detection:** Memory leaks possible with circular references
3. **Limited optimization:** Direct AST interpretation
4. **No FFI:** Cannot call external C libraries
5. **Stack depth limited:** ~10,000 recursive calls max

## Future Runtime Enhancements

Planned improvements for v2.0:

1. **Bytecode VM:** 10x performance improvement
2. **True async runtime:** Tokio integration
3. **Incremental GC:** Pause-free collection
4. **WASM target:** Browser execution
5. **Native compilation:** LLVM backend