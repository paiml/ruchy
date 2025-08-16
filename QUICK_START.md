# Ruchy Quick Start Guide

Welcome to Ruchy! This guide will get you up and running in 5 minutes.

## 1. Installation (30 seconds)

Choose your preferred installation method:

### Option A: Via Cargo
```bash
cargo install ruchy-cli
```

### Option B: Pre-built Binary
```bash
# Linux/macOS
curl -LO https://github.com/paiml/ruchy/releases/latest/download/ruchy-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m | sed 's/x86_64/amd64/')
chmod +x ruchy-*
sudo mv ruchy-* /usr/local/bin/ruchy

# Verify installation
ruchy --version
```

## 2. Your First Ruchy Experience (1 minute)

### Start the REPL
```bash
ruchy
```

Try these commands in the REPL:
```ruchy
ruchy> 42 + 8
ruchy> "Hello, " ++ "World!"
ruchy> [1, 2, 3] |> map(_ * 2)
ruchy> :ast if true { 42 } else { 0 }
ruchy> :rust [1..10] |> filter(_ % 2 == 0)
ruchy> :help
ruchy> :quit
```

## 3. Write Your First Script (2 minutes)

Create `hello.ruchy`:
```rust
// hello.ruchy
fun greet(name: String) {
    println("Hello, {name}!")
}

fun main() {
    greet("World")
    
    let numbers = [1, 2, 3, 4, 5]
    let evens = numbers |> filter(_ % 2 == 0)
    println("Even numbers: {evens}")
}
```

Run it:
```bash
ruchy run hello.ruchy
```

## 4. Transpile to Rust (1 minute)

See the generated Rust code:
```bash
ruchy transpile hello.ruchy
```

Compile to native binary:
```bash
ruchy compile hello.ruchy -o hello
./hello
```

## 5. Pipeline Operations (30 seconds)

Create `pipeline.ruchy`:
```rust
// Pipeline example
fun analyze_data() {
    [1..100]
    |> filter(_ % 2 == 0)     // Keep even numbers
    |> map(_ * 2)              // Double them
    |> filter(_ < 50)          // Keep under 50
    |> sum()                   // Sum them up
    |> println()
}

analyze_data()
```

Run it:
```bash
ruchy run pipeline.ruchy
```

## 6. Pattern Matching

Create `patterns.ruchy`:
```rust
// Pattern matching example
fun describe(value) {
    match value {
        0 => "zero",
        1..10 => "single digit",
        n if n < 0 => "negative",
        _ => "something else"
    }
}

println(describe(5))   // "single digit"
println(describe(-3))  // "negative"
println(describe(42))  // "something else"
```

## REPL Commands Reference

| Command | Description |
|---------|-------------|
| `:help` | Show all commands |
| `:ast <expr>` | Show Abstract Syntax Tree |
| `:rust <expr>` | Show Rust transpilation |
| `:type <expr>` | Show expression type (coming soon) |
| `:clear` | Clear session |
| `:save <file>` | Save session |
| `:load <file>` | Load session |
| `:quit` | Exit REPL |

## CLI Commands Reference

```bash
# Start REPL (default)
ruchy

# Parse and show AST
ruchy parse script.ruchy

# Transpile to Rust
ruchy transpile script.ruchy -o output.rs

# Run script
ruchy run script.ruchy

# Compile to binary
ruchy compile script.ruchy -o binary
```

## Language Features Available in v0.1.0

✅ **Working:**
- Literals (integers, floats, strings, booleans)
- Binary operators (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||)
- Variables and functions
- If/else expressions
- While and for loops
- Arrays and ranges
- Pipeline operators (|>)
- Lambdas and closures
- Pattern matching (basic)
- Comments

🚧 **Coming Soon:**
- Type inference
- Actor system
- Async/await
- Traits and impls
- Module system
- Package management
- Full MCP integration

## Examples Repository

Find more examples in the `examples/` directory:
- `fibonacci.ruchy` - Recursive Fibonacci
- `quicksort.ruchy` - Sorting algorithm
- `marco_polo.ruchy` - Pattern matching game

## Getting Help

- **Documentation**: [docs.rs/ruchy](https://docs.rs/ruchy)
- **GitHub Issues**: [github.com/paiml/ruchy/issues](https://github.com/paiml/ruchy/issues)
- **Examples**: [github.com/paiml/ruchy/tree/main/examples](https://github.com/paiml/ruchy/tree/main/examples)

## Next Steps

1. Explore the [examples directory](https://github.com/paiml/ruchy/tree/main/examples)
2. Read the [language specification](https://github.com/paiml/ruchy/blob/main/docs/architecture/ruchy-design-architectur.md)
3. Try transpiling your Python scripts to Ruchy
4. Contribute to the project!

Happy coding with Ruchy! 🚀