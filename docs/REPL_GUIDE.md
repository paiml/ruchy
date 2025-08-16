# Ruchy REPL User Guide

The Ruchy REPL (Read-Eval-Print Loop) provides an interactive environment for experimenting with Ruchy code, exploring language features, and rapid prototyping.

## Quick Start

Start the REPL with:
```bash
cargo run --bin ruchy-cli repl
```

## Features

### 🚀 Expression Evaluation
```ruchy
> 42 + 8
50

> let x = [1, 2, 3, 4, 5]
> [n * 2 for n in x if n > 2]
[6, 8, 10]

> fun fibonacci(n: i32) -> i32 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }
> fibonacci(10)
55
```

### 🔍 Code Inspection Commands

#### `:type <expression>` - Show Type Information
```ruchy
> :type 42
42: i32

> :type [1, 2, 3]
[1, 2, 3]: List<i32>

> :type fibonacci
fibonacci: fn(i32) -> i32
```

#### `:ast <expression>` - Show Abstract Syntax Tree
```ruchy
> :ast 1 + 2
Binary {
    left: Literal(Integer(1)),
    op: Add,
    right: Literal(Integer(2))
}
```

#### `:rust <expression>` - Show Generated Rust Code
```ruchy
> :rust 1 + 2
1 + 2
```

### 📜 Session Management

#### `:history` - Show Command History
```ruchy
> :history
1: let x = 42
2: x + 8
3: fun double(n: i32) -> i32 { n * 2 }
```

#### `:clear` - Clear Session
```ruchy
> :clear
Session cleared
```

#### `:save <filename>` - Save Session
```ruchy
> :save my_session.ruchy
Session saved to my_session.ruchy
```

#### `:load <filename>` - Load Session
```ruchy
> :load my_session.ruchy
Loading: let x = 42
Loading: x + 8
Session loaded from my_session.ruchy
```

## Language Features in REPL

### 🔢 Data Types and Literals
```ruchy
> 42                    # Integer
> 3.14                  # Float  
> "hello"               # String
> true                  # Boolean
> [1, 2, 3]             # List
> ()                    # Unit type
```

### 🔄 List Comprehensions
```ruchy
> [x * x for x in [1, 2, 3, 4, 5]]
[1, 4, 9, 16, 25]

> [word for word in ["hello", "world", "ruchy"] if word.len() > 4]
["hello", "world", "ruchy"]
```

### 🎯 Pattern Matching
```ruchy
> match 42 {
    0 => "zero",
    n if n > 0 => "positive", 
    _ => "negative"
  }
"positive"
```

### ⚡ Pipeline Operations
```ruchy
> [1, 2, 3, 4, 5] |> map(|x| x * 2) |> filter(|x| x > 4) |> sum()
18
```

### 🏗️ Function Definitions
```ruchy
> fun add(a: i32, b: i32) -> i32 { a + b }
> add(3, 4)
7

> let multiply = |x: i32, y: i32| x * y
> multiply(6, 7)
42
```

### 📊 DataFrames
```ruchy
> let df = df![
    "name" => ["Alice", "Bob", "Charlie"];
    "age" => [25, 30, 35];
    "city" => ["NYC", "SF", "LA"]
  ]

> df.filter(col("age") > 30)
# Filtered DataFrame
```

### 🏛️ Structs and Methods
```ruchy
> struct Point { x: f64, y: f64 }
> let p = Point { x: 3.0, y: 4.0 }
> p.x
3.0
```

## Error Handling and Debugging

### Syntax Errors
```ruchy
> let x = 
Error: Unexpected end of input, expected expression

> fun incomplete(
Error: Expected parameter list
```

### Type Errors
```ruchy
> "hello" + 5
Error: Cannot add String and Integer

> :type "hello" + 5
Type error: Mismatched types in binary operation
```

### Runtime Exploration
```ruchy
> fun buggy_division(a: i32, b: i32) -> i32 { a / b }
> buggy_division(10, 0)
Error: Division by zero

> :ast buggy_division(10, 0)
# Examine the structure to understand the issue
```

## Performance and Quality

### Built-in Quality Assurance
- **Property Testing**: All REPL operations are tested with property-based tests
- **Fuzzing**: Input handling is continuously fuzzed for robustness
- **State Machine Testing**: Session state transitions are formally verified
- **Performance Monitoring**: Latency and memory usage are continuously benchmarked

### Performance Characteristics
- **Startup Time**: < 10ms
- **Evaluation Latency**: < 1ms for simple expressions
- **Type Lookup**: < 10µs
- **Memory Usage**: < 100MB for typical sessions

## Tips and Best Practices

### 🎯 Effective REPL Usage

1. **Start Small**: Test individual expressions before building complex programs
2. **Use Type Inspection**: Check types frequently with `:type` to understand your data
3. **Save Progress**: Use `:save` to preserve valuable explorations
4. **Inspect Generated Code**: Use `:rust` to understand performance implications

### 🔧 Debugging Workflows

1. **Isolate Problems**: Test components individually
2. **Check Types**: Use `:type` to verify assumptions
3. **Examine Structure**: Use `:ast` to understand parsing
4. **Review History**: Use `:history` to track your exploration

### 📈 Performance Tips

1. **Prefer Immutable Operations**: They optimize better
2. **Use List Comprehensions**: They compile to efficient iterators
3. **Avoid Deep Recursion**: Stack-intensive operations may be slow
4. **Clear Session Periodically**: Use `:clear` to reset accumulated state

## Integration with Development

### IDE Integration
The REPL can be used alongside your favorite editor:
- Copy code from your editor and paste into REPL for testing
- Use `:save` to preserve REPL experiments as files
- Use `:rust` to see how Ruchy compiles to Rust

### Testing Integration  
Use the REPL to develop and test ideas before adding them to your test suite:
```ruchy
> # Develop a complex function interactively
> fun complex_calc(data: List<i32>) -> f64 { /* ... */ }
> # Test with various inputs
> complex_calc([1, 2, 3])
> complex_calc([])
> # Once confident, copy to your source files
```

## Advanced Features

### Session Persistence
```ruchy
> :save exploration.ruchy    # Save current session
> :clear                     # Clear session  
> :load exploration.ruchy    # Restore previous work
```

### Cross-Session Development
```ruchy
# Session 1: Data preparation
> let raw_data = load_csv("data.csv")
> :save data_prep.ruchy

# Session 2: Analysis  
> :load data_prep.ruchy
> let analysis = analyze(raw_data)
> :save analysis.ruchy
```

## Error Recovery

The REPL is designed to be robust and never crash:

- **Syntax Errors**: Clearly reported with suggestions
- **Type Errors**: Detailed explanations with context
- **Runtime Errors**: Graceful handling with stack traces
- **Resource Limits**: Automatic recovery from memory/time limits

## Getting Help

- `:help` - Show available commands
- `:quit` or `:q` - Exit the REPL
- Visit [Ruchy Documentation](https://github.com/paiml/ruchy) for more examples
- Report issues at [GitHub Issues](https://github.com/paiml/ruchy/issues)

---

*The Ruchy REPL is continuously tested for correctness, performance, and robustness with property-based testing, fuzzing, and formal verification.*