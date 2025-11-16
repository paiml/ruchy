# Getting Started with Ruchy

Welcome to Ruchy, a systems scripting language that transpiles to idiomatic Rust. This guide will help you get started with writing and running Ruchy code.

## Installation

### From Crates.io (Recommended)

```bash
cargo install ruchy
```

### From Source

```bash
git clone https://github.com/paiml/ruchy.git
cd ruchy
cargo install --path .
```

## Your First Ruchy Program

### Using the REPL

The easiest way to start with Ruchy is using the interactive REPL:

```bash
ruchy repl
```

Try these examples:

```ruchy
>>> println("Hello, Ruchy!")
Hello, Ruchy!

>>> let x = 42
>>> x * 2
84

>>> fn greet(name) {
...   println(f"Hello, {name}!")
... }
>>> greet("World")
Hello, World!
```

### Writing Ruchy Scripts

Create a file called `hello.ruchy`:

```ruchy
// hello.ruchy
fn main() {
    let name = "Ruchy"
    println(f"Welcome to {name}!")

    // Variables and types
    let x = 10
    let y = 3.14
    let message = "Systems scripting made easy"

    // Functions
    fn add(a, b) {
        a + b
    }

    println(f"10 + 20 = {add(10, 20)}")
}
```

Run it with:

```bash
ruchy run hello.ruchy
```

## Core Language Features

### Variables and Mutability

```ruchy
// Immutable by default
let x = 5

// Mutable variables
let mut counter = 0
counter = counter + 1

// Type annotations (optional)
let age: int = 25
let price: float = 19.99
```

### Functions

```ruchy
// Basic function
fn square(x) {
    x * x
}

// With type annotations
fn add(a: int, b: int) -> int {
    a + b
}

// Fat arrow syntax
let double = x => x * 2

// Default parameters
fn greet(name = "Guest") {
    println(f"Hello, {name}!")
}
```

### Control Flow

```ruchy
// If expressions
let result = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
}

// Pattern matching
match value {
    0 => "zero",
    1..=10 => "small",
    Some(x) => f"value: {x}",
    _ => "other"
}

// Loops
for i in 0..10 {
    println(i)
}

while condition {
    // loop body
}
```

### Collections

```ruchy
// Lists
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(x => x * 2)

// List comprehensions
let squares = [x * x for x in 0..10 if x % 2 == 0]

// Objects (HashMaps)
let person = {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}

// Accessing fields
println(person.name)
```

### String Interpolation

```ruchy
let name = "Ruchy"
let version = 3.6

// f-strings for interpolation
println(f"Welcome to {name} v{version}!")

// Expressions in interpolation
println(f"2 + 2 = {2 + 2}")
```

### Error Handling

```ruchy
// Using Result type
fn divide(a, b) {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

// Try-catch blocks
try {
    risky_operation()
} catch e {
    println(f"Error: {e}")
}

// The ? operator
fn read_config() -> Result<Config, Error> {
    let content = read_file("config.json")?
    parse_json(content)
}
```

### Pipeline Operator

```ruchy
// Chain operations with |>
let result = "hello world"
    |> upper
    |> split(" ")
    |> map(reverse)
    |> join("-")
// Result: "OLLEH-DLROW"
```

## Advanced Features

### DataFrames

```ruchy
// Create a DataFrame
let df = df![
    "name" => ["Alice", "Bob", "Charlie"],
    "age" => [30, 25, 35],
    "city" => ["NYC", "SF", "LA"]
]

// Operations
df.filter(row => row.age > 25)
  .select(["name", "city"])
  .sort("age")
```

### Async/Await

```ruchy
async fn fetch_data(url) {
    let response = await http_get(url)
    response.json()
}

// In async context
let data = await fetch_data("https://api.example.com/data")
```

### Pattern Destructuring

```ruchy
// Tuple destructuring
let (x, y) = (10, 20)

// List destructuring with rest
let [first, second, ..rest] = [1, 2, 3, 4, 5]

// Object destructuring
let {name, age} = person
```

### Module System

```ruchy
// Import from standard library
import std::{fs, path}

// Import specific items
import math::{sqrt, pow, PI}

// Your own modules
import my_utils::{helper_function}
```

## REPL Magic Commands

The Ruchy REPL includes special commands for enhanced productivity:

```ruchy
// Show type of expression
:type 42
// Output: int

// Time execution
:time heavy_computation()
// Output: Execution time: 125ms

// Show variable bindings
:vars
// Shows all defined variables

// Load a file
:load my_script.ruchy

// Clear the screen
:clear

// Exit REPL
:quit
```

## Command Line Interface

```bash
# Run a Ruchy file
ruchy run script.ruchy

# Start the REPL
ruchy repl

# Transpile to Rust (for debugging)
ruchy transpile script.ruchy -o output.rs

# Parse and show AST (for debugging)
ruchy parse script.ruchy

# Show version
ruchy --version
```

## Best Practices

### 1. Use Type Inference

Ruchy has powerful type inference, so you don't need to annotate everything:

```ruchy
// Good - type is inferred
let x = 42

// Only annotate when necessary
fn process_data(data: DataFrame) -> Result<Stats, Error> {
    // implementation
}
```

### 2. Leverage Pattern Matching

Pattern matching is powerful and idiomatic:

```ruchy
// Good
match result {
    Ok(value) => process(value),
    Err(e) => handle_error(e)
}

// Also good with if-let for single patterns
if let Some(value) = optional_value {
    use_value(value)
}
```

### 3. Use Pipeline Operators for Data Transformation

```ruchy
// Clear and readable
data
    |> filter(valid)
    |> map(transform)
    |> reduce(combine)
```

### 4. Handle Errors Properly

```ruchy
// Use Result for fallible operations
fn parse_number(s: string) -> Result<int, ParseError> {
    // parsing logic
}

// Use ? for propagation
fn process() -> Result<(), Error> {
    let num = parse_number(input)?
    // continue processing
    Ok(())
}
```

## Common Patterns

### Reading Files

```ruchy
import std::fs

fn read_config(path) {
    let content = fs::read_to_string(path)?
    parse_json(content)
}
```

### Working with Collections

```ruchy
// Filter and map
let adults = people
    .filter(p => p.age >= 18)
    .map(p => p.name)

// Reduce
let sum = numbers.reduce(0, (acc, n) => acc + n)

// Zip multiple lists
let pairs = zip(keys, values)
```

### String Manipulation

```ruchy
let text = "Hello, World!"

// Common operations
text.upper()           // "HELLO, WORLD!"
text.lower()           // "hello, world!"
text.split(", ")       // ["Hello", "World!"]
text.replace("o", "0") // "Hell0, W0rld!"
text.contains("World") // true
```

## Troubleshooting

### Common Issues

1. **"Memory limit exceeded"**: The REPL has safety limits. For larger computations, use script files or adjust limits.

2. **"Type mismatch"**: Ruchy is strongly typed. Ensure types match or use explicit conversions:
   ```ruchy
   let x = 42
   let y = x.to_string()  // Convert to string
   ```

3. **"Variable not found"**: Check scope and spelling. Variables are block-scoped.

## Next Steps

- Explore the [Language Reference](SPECIFICATION.md) for complete feature documentation
- Check out [Examples](../examples/) for more code samples
- Report issues and contribute at [GitHub Issues](https://github.com/paiml/ruchy/issues)

## Getting Help

- **Documentation**: Full API docs at [docs.rs/ruchy](https://docs.rs/ruchy)
- **Issues**: Report bugs at [GitHub Issues](https://github.com/paiml/ruchy/issues)
- **Examples**: See the [examples/](../examples/) directory
- **REPL Help**: Type `:help` in the REPL

Happy coding with Ruchy! 🚀