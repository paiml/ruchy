# Ruchy Language Features - Complete Reference

## Table of Contents
- [Language Features](#language-features)
- [CLI Commands](#cli-commands)
- [WASM Support](#wasm-support)
- [Notebook System](#notebook-system)
- [Testing Framework](#testing-framework)
- [Quality Engineering](#quality-engineering)
- [Standard Library](#standard-library)

## Language Features

### Core Syntax
```ruchy
// Variables and Constants
let x = 42
let mut y = 10
const PI = 3.14159

// Functions
fun add(a, b) {
    return a + b
}

// Fat arrow functions
let double = x => x * 2
let sum = (a, b) => a + b

// Async functions
async fun fetch_data(url) {
    let response = await http.get(url)
    return response.data
}
```

### Type System
```ruchy
// Type annotations
let x: Int = 42
let name: String = "Ruchy"
let numbers: Vec<Int> = [1, 2, 3]
let maybe: Option<String> = Some("value")

// Generic functions
fun identity<T>(x: T) -> T {
    return x
}

// Type aliases
type Point = { x: Float, y: Float }
type Result<T> = Option<T>
```

### Control Flow
```ruchy
// If expressions
let result = if x > 10 { "big" } else { "small" }

// Pattern matching
match value {
    Some(x) => println(f"Got {x}"),
    None => println("Nothing"),
    _ => println("Unknown")
}

// Pattern guards
match x {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero"
}

// Loops
for i in 0..10 {
    println(i)
}

while condition {
    // body
}

loop {
    if done { break }
}
```

### Error Handling
```ruchy
// Try-catch-finally
try {
    risky_operation()
} catch (e) {
    println(f"Error: {e}")
} finally {
    cleanup()
}

// Result type
fun divide(a, b) -> Result<Float> {
    if b == 0 {
        return Err("Division by zero")
    }
    return Ok(a / b)
}

// Panic for unrecoverable errors
panic!("This should never happen")
```

### Data Structures
```ruchy
// Arrays
let arr = [1, 2, 3, 4, 5]
let first = arr[0]
let slice = arr[1..3]

// Tuples
let point = (10, 20)
let (x, y) = point  // Destructuring

// Objects/Records
let person = {
    name: "Alice",
    age: 30,
    greet: fun() { println(f"Hello, I'm {this.name}") }
}

// DataFrames (with dataframe feature)
let df = df![
    "name" => ["Alice", "Bob"],
    "age" => [30, 25]
]
```

### Functional Programming
```ruchy
// Higher-order functions
let doubled = [1, 2, 3].map(x => x * 2)
let evens = [1, 2, 3, 4].filter(x => x % 2 == 0)
let sum = [1, 2, 3].reduce(0, (acc, x) => acc + x)

// Pipeline operator
let result = data
    |> filter(x => x > 0)
    |> map(x => x * 2)
    |> reduce(0, +)

// Partial application
let add5 = add(5, _)
let result = add5(10)  // 15

// Function composition
let process = compose(validate, transform, save)
```

### String Interpolation
```ruchy
let name = "World"
let greeting = f"Hello, {name}!"
let calculation = f"2 + 2 = {2 + 2}"

// Format specifiers
let pi = 3.14159
let formatted = f"Pi: {pi:.2}"  // "Pi: 3.14"
```

### Module System
```ruchy
// Importing modules
import std.io
import math.{sin, cos, PI}
from collections import HashMap

// Exporting from modules
export fun public_function() { }
export let constant = 42
export { helper1, helper2 }

// Module aliases
import very.long.module.path as short
```

### Macros and Metaprogramming
```ruchy
// DataFrame macro
let data = df![
    "column1" => [1, 2, 3],
    "column2" => ["a", "b", "c"]
]

// Custom macros (coming soon)
macro_rules! assert_eq {
    ($left, $right) => {
        if $left != $right {
            panic!(f"Assertion failed: {$left} != {$right}")
        }
    }
}
```

## CLI Commands

### Core Commands
```bash
# REPL - Interactive shell
ruchy repl
  --verbose    # Enable verbose output
  --quiet      # Suppress output

# Run - Execute scripts
ruchy run script.ruchy
  --verbose    # Show execution details

# Format - Code formatting
ruchy fmt path/
  --check      # Check without modifying
```

### WebAssembly Commands
```bash
# Compile to WASM
ruchy wasm compile input.ruchy
  -o, --output <file>    # Output file (default: input.wasm)
  --optimize             # Enable optimizations
  --validate             # Validate output (default: true)

# Validate WASM module
ruchy wasm validate module.wasm

# Run WASM module (coming soon)
ruchy wasm run module.wasm [args...]
```

### Notebook Commands
```bash
# Start notebook server
ruchy notebook serve
  -p, --port <port>      # Port number (default: 8888)
  --host <host>          # Host address (default: 127.0.0.1)

# Test notebooks
ruchy notebook test notebook.ipynb
  --coverage             # Generate coverage report
  --format <fmt>         # Output format: text, json, html

# Convert notebooks
ruchy notebook convert input.ipynb output.html
  --format <fmt>         # Output format: html, markdown, script
```

### Testing Commands
```bash
# Run tests
ruchy test run path/
  --coverage             # Generate coverage report
  --parallel             # Run tests in parallel
  --filter <pattern>     # Filter tests by name

# Generate test report
ruchy test report
  --format <fmt>         # Format: json, html, junit
  -o, --output <file>    # Output file
```

## WASM Support

### Compilation Features
- **Deterministic compilation**: Same source always produces identical bytecode
- **Validation**: Built-in wasmparser validation
- **Optimization**: Optional optimization passes
- **Security**: Sandboxed execution environment

### Supported WASM Features
- Functions with parameters and return values
- Local variables
- Basic arithmetic operations
- Control flow (if/else, loops)
- Memory operations
- Module exports

### Example WASM Compilation
```ruchy
// input.ruchy
fun fibonacci(n) {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

fun main() {
    return fibonacci(10)
}
```

```bash
# Compile to WASM
ruchy wasm compile input.ruchy -o fib.wasm --optimize

# Validate the module
ruchy wasm validate fib.wasm
# Output: ✓ WASM module is valid
```

## Notebook System

### Features
- **Jupyter-compatible**: Works with .ipynb files
- **Web interface**: Professional notebook UI
- **Cell execution**: Run cells individually or all at once
- **State management**: Persistent state between cells
- **Testing framework**: Comprehensive notebook testing

### Notebook Testing
```bash
# Test a notebook
ruchy notebook test my_notebook.ipynb --coverage

# Output:
# ✓ Cell 1: Passed
# ✓ Cell 2: Passed  
# ✓ Cell 3: Passed
# Coverage: 85%
```

### Testing Framework Features
- **Golden file testing**: Compare against expected outputs
- **Property-based testing**: Random input generation
- **Differential testing**: Compare implementations
- **Mutation testing**: Code mutation detection
- **Performance testing**: Regression detection

## Testing Framework

### Test Types
1. **Unit Tests**: Function-level testing
2. **Integration Tests**: Component interaction
3. **Property Tests**: Invariant verification
4. **Fuzz Tests**: Random input testing
5. **Acceptance Tests**: End-to-end validation

### Property Testing
```rust
// Example property test
proptest! {
    #[test]
    fn prop_compilation_deterministic(x in 0i32..100) {
        let source = format!("fun main() {{ return {} }}", x);
        let result1 = compile(&source);
        let result2 = compile(&source);
        prop_assert_eq!(result1, result2);
    }
}
```

### Fuzz Testing
```bash
# Run fuzz tests
cargo fuzz run wasm_comprehensive
cargo fuzz run wasm_security  
cargo fuzz run wasm_stress
```

## Quality Engineering

### PMAT Integration
- **TDG Score**: Technical Debt Grading (must be A- or higher)
- **Complexity Limits**: All functions <10 cyclomatic complexity
- **SATD Detection**: Zero tolerance for technical debt comments
- **Documentation Coverage**: >70% for public APIs

### Quality Gates
```bash
# Check quality before commit
pmat tdg . --min-grade A-
pmat analyze complexity --max-cyclomatic 10
pmat analyze satd --fail-on-violation
```

### Continuous Monitoring
```bash
# Start real-time dashboard
pmat tdg dashboard --port 8080 --open
```

## Standard Library

### Core Modules
- **std.io**: Input/output operations
- **std.fs**: File system operations
- **std.net**: Networking
- **std.math**: Mathematical functions
- **std.collections**: Data structures
- **std.string**: String utilities
- **std.regex**: Regular expressions
- **std.json**: JSON parsing/serialization

### Math Functions
```ruchy
import math

let x = math.sqrt(16)      // 4
let y = math.pow(2, 8)      // 256
let z = math.sin(math.PI)   // ~0
let a = math.abs(-42)       // 42
let b = math.floor(3.7)     // 3
let c = math.ceil(3.2)      // 4
let d = math.round(3.5)     // 4
let e = math.min(5, 3)      // 3
let f = math.max(5, 3)      // 5
```

### String Methods
```ruchy
let s = "Hello, World!"

s.len()                    // 13
s.to_upper()              // "HELLO, WORLD!"
s.to_lower()              // "hello, world!"
s.trim()                  // Remove whitespace
s.split(",")              // ["Hello", " World!"]
s.replace("World", "Ruchy") // "Hello, Ruchy!"
s.contains("World")       // true
s.starts_with("Hello")    // true
s.ends_with("!")          // true
s[0..5]                   // "Hello" (slicing)
```

### Collection Methods
```ruchy
let arr = [1, 2, 3, 4, 5]

arr.len()                 // 5
arr.push(6)              // [1, 2, 3, 4, 5, 6]
arr.pop()                // 5, arr = [1, 2, 3, 4]
arr.first()              // 1
arr.last()               // 5
arr.reverse()            // [5, 4, 3, 2, 1]
arr.sort()               // [1, 2, 3, 4, 5]
arr.contains(3)          // true
arr.index_of(3)          // 2
arr.slice(1, 3)          // [2, 3]
```

### Type Conversions
```ruchy
// String conversions
let s = 42.to_string()           // "42"
let n = "42".parse_int()         // 42
let f = "3.14".parse_float()     // 3.14

// Numeric conversions
let i = 3.14.to_int()            // 3
let f = 42.to_float()            // 42.0

// Boolean conversions
let b = 1.to_bool()              // true
let b = 0.to_bool()              // false
```

## Configuration

### Project Configuration (ruchy.toml)
```toml
[project]
name = "my-project"
version = "0.1.0"
authors = ["Your Name"]

[dependencies]
std = "1.0"
dataframe = { version = "0.5", optional = true }

[features]
default = ["batteries-included"]
batteries-included = ["dataframe", "notebook", "wasm-compile"]
minimal = []

[quality]
max-complexity = 10
min-coverage = 80
tdg-min-grade = "A-"
```

### Environment Variables
```bash
RUCHY_PATH=/usr/local/lib/ruchy    # Module search path
RUCHY_HOME=~/.ruchy                # Configuration directory
RUCHY_DEBUG=1                       # Enable debug output
RUCHY_COLORS=1                      # Enable colored output
```

## Platform Support

### Operating Systems
- Linux (x86_64, aarch64)
- macOS (x86_64, arm64)
- Windows (x86_64)
- FreeBSD (x86_64)

### Architectures
- x86_64
- aarch64/arm64
- armv7
- wasm32

### Minimum Requirements
- Rust 1.75+
- 2GB RAM
- 100MB disk space

## Performance

### Benchmarks
- **Parser**: ~1M lines/second
- **Type checking**: ~500K lines/second
- **Code generation**: ~300K lines/second
- **WASM compilation**: ~100K lines/second

### Optimization Levels
- **Debug**: No optimizations, full debug info
- **Release**: Standard optimizations
- **Release with LTO**: Link-time optimization
- **Release with PGO**: Profile-guided optimization

## Integration

### Editor Support
- **VS Code**: Official extension available
- **Vim/Neovim**: LSP support
- **Emacs**: LSP support
- **IntelliJ**: Plugin in development

### Build Systems
- **Cargo**: Native integration
- **Make**: Makefile templates
- **CMake**: CMake modules
- **Bazel**: Build rules

### CI/CD
- **GitHub Actions**: Workflows provided
- **GitLab CI**: Pipeline templates
- **Jenkins**: Jenkinsfile examples
- **CircleCI**: Config examples

## Resources

### Documentation
- [API Documentation](https://docs.rs/ruchy)
- [Language Specification](SPECIFICATION.md)
- [Tutorial](https://ruchy-lang.org/tutorial)

### Community
- [GitHub Repository](https://github.com/paiml/ruchy)
- [GitHub Issues](https://github.com/paiml/ruchy/issues)

### Learning Resources
- [Examples Directory](https://github.com/paiml/ruchy/tree/main/examples)
- [Language Completeness Documentation](../docs/lang-completeness-book/)

---

*Last updated: September 2025 - Version 3.0.3*