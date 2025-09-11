# Ruchy Quick Start Guide

## Installation

### From Crates.io (Recommended)
```bash
cargo install ruchy
```

### From Source
```bash
git clone https://github.com/paiml/ruchy.git
cd ruchy
cargo build --release
cargo install --path .
```

### Verify Installation
```bash
ruchy --version
# Output: ruchy 3.0.3
```

## Your First Ruchy Program

### 1. Hello World

Create a file `hello.ruchy`:
```ruchy
fun main() {
    println("Hello, World!")
}
```

Run it:
```bash
ruchy run hello.ruchy
# Output: Hello, World!
```

### 2. Interactive REPL

Start the REPL:
```bash
ruchy repl
```

Try some expressions:
```ruchy
> 2 + 2
4

> let name = "Ruchy"
"Ruchy"

> println(f"Hello, {name}!")
Hello, Ruchy!

> let double = x => x * 2
<function>

> double(21)
42

> [1, 2, 3].map(double)
[2, 4, 6]

> exit
```

### 3. Variables and Functions

Create `basics.ruchy`:
```ruchy
// Variables
let x = 10
let mut y = 20
y = y + 5

// Functions
fun add(a, b) {
    return a + b
}

// Fat arrow functions
let multiply = (a, b) => a * b

// Main function
fun main() {
    let result = add(x, y)
    println(f"x + y = {result}")
    
    let product = multiply(3, 4)
    println(f"3 * 4 = {product}")
}
```

Run it:
```bash
ruchy run basics.ruchy
# Output:
# x + y = 35
# 3 * 4 = 12
```

### 4. Control Flow

Create `control.ruchy`:
```ruchy
fun check_number(n) {
    if n > 0 {
        println(f"{n} is positive")
    } else if n < 0 {
        println(f"{n} is negative")
    } else {
        println("Zero!")
    }
}

fun main() {
    // If expressions
    check_number(42)
    check_number(-5)
    check_number(0)
    
    // For loops
    println("\nCounting:")
    for i in 1..5 {
        println(f"  {i}")
    }
    
    // Pattern matching
    let value = Some(42)
    match value {
        Some(x) => println(f"\nGot value: {x}"),
        None => println("\nNo value"),
    }
}
```

### 5. Collections and Data Structures

Create `collections.ruchy`:
```ruchy
fun main() {
    // Arrays
    let numbers = [1, 2, 3, 4, 5]
    println(f"Numbers: {numbers}")
    println(f"First: {numbers[0]}")
    println(f"Last: {numbers[numbers.len() - 1]}")
    
    // Array operations
    let doubled = numbers.map(x => x * 2)
    println(f"Doubled: {doubled}")
    
    let evens = numbers.filter(x => x % 2 == 0)
    println(f"Evens: {evens}")
    
    let sum = numbers.reduce(0, (acc, x) => acc + x)
    println(f"Sum: {sum}")
    
    // Objects
    let person = {
        name: "Alice",
        age: 30,
        greet: fun() { 
            println(f"Hi, I'm {this.name}!")
        }
    }
    
    person.greet()
    println(f"{person.name} is {person.age} years old")
    
    // Tuples
    let point = (10, 20)
    let (x, y) = point  // Destructuring
    println(f"Point: x={x}, y={y}")
}
```

### 6. Error Handling

Create `errors.ruchy`:
```ruchy
fun divide(a, b) {
    if b == 0 {
        panic!("Division by zero!")
    }
    return a / b
}

fun safe_divide(a, b) {
    if b == 0 {
        return None
    }
    return Some(a / b)
}

fun main() {
    // Safe division with Option
    let result1 = safe_divide(10, 2)
    match result1 {
        Some(x) => println(f"10 / 2 = {x}"),
        None => println("Cannot divide")
    }
    
    let result2 = safe_divide(10, 0)
    match result2 {
        Some(x) => println(f"10 / 0 = {x}"),
        None => println("Cannot divide by zero")
    }
    
    // Try-catch for error handling
    try {
        let x = divide(10, 2)
        println(f"Success: {x}")
    } catch (e) {
        println(f"Error: {e}")
    }
}
```

### 7. WebAssembly Compilation

Create `wasm_example.ruchy`:
```ruchy
fun fibonacci(n) {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

fun factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

fun main() {
    let fib10 = fibonacci(10)
    let fact5 = factorial(5)
    println(f"fibonacci(10) = {fib10}")
    println(f"factorial(5) = {fact5}")
    return fib10
}
```

Compile to WebAssembly:
```bash
# Compile to WASM
ruchy wasm compile wasm_example.ruchy -o example.wasm

# Validate the module
ruchy wasm validate example.wasm
# Output: ✓ WASM module is valid

# Check the size
ls -lh example.wasm
# Output: -rw-r--r-- 1 user user 256B Sep 11 10:00 example.wasm
```

### 8. Notebook Server

Start the notebook server:
```bash
ruchy notebook serve --port 8888
# Output: Notebook server running at http://127.0.0.1:8888
```

Open your browser to `http://localhost:8888` to use the Jupyter-style notebook interface.

### 9. Testing

Create `test_math.ruchy`:
```ruchy
fun test_addition() {
    assert(2 + 2 == 4, "Basic addition failed")
    assert(10 + 5 == 15, "Addition test 2 failed")
}

fun test_multiplication() {
    assert(3 * 4 == 12, "Basic multiplication failed")
    assert(5 * 0 == 0, "Multiply by zero failed")
}

fun main() {
    test_addition()
    test_multiplication()
    println("All tests passed!")
}
```

Run tests:
```bash
ruchy test run test_math.ruchy
# Output: All tests passed!
```

### 10. String Interpolation

Create `strings.ruchy`:
```ruchy
fun main() {
    let name = "Ruchy"
    let version = 3.0
    let users = 1000
    
    // Basic interpolation
    println(f"Welcome to {name} v{version}!")
    
    // Expressions in interpolation
    println(f"2 + 2 = {2 + 2}")
    println(f"Users doubled: {users * 2}")
    
    // Format specifiers
    let pi = 3.14159
    println(f"Pi to 2 decimals: {pi:.2}")
    
    // Multi-line strings
    let message = f"""
    Language: {name}
    Version: {version}
    Users: {users}
    """
    println(message)
}
```

## Advanced Features

### Pipeline Operator
```ruchy
let result = [1, 2, 3, 4, 5]
    |> filter(x => x > 2)
    |> map(x => x * 2)
    |> reduce(0, +)
// Result: 24 (3*2 + 4*2 + 5*2)
```

### Async/Await
```ruchy
async fun fetch_data(url) {
    let response = await http.get(url)
    return response.json()
}

async fun main() {
    let data = await fetch_data("https://api.example.com/data")
    println(data)
}
```

### Pattern Guards
```ruchy
fun classify(x) {
    match x {
        n if n > 100 => "huge",
        n if n > 50 => "big",
        n if n > 10 => "medium",
        n if n > 0 => "small",
        0 => "zero",
        _ => "negative"
    }
}
```

### Destructuring
```ruchy
// Array destructuring
let [first, second, ...rest] = [1, 2, 3, 4, 5]
// first = 1, second = 2, rest = [3, 4, 5]

// Object destructuring
let {name, age} = {name: "Bob", age: 25, city: "NYC"}
// name = "Bob", age = 25

// Nested destructuring
let {user: {name, email}} = {
    user: {name: "Alice", email: "alice@example.com"}
}
```

## CLI Tools

### Code Formatting
```bash
# Format a file
ruchy fmt mycode.ruchy

# Check formatting without changing
ruchy fmt mycode.ruchy --check

# Format entire directory
ruchy fmt src/
```

### Running Tests
```bash
# Run all tests in directory
ruchy test run tests/

# Run with coverage report
ruchy test run tests/ --coverage

# Generate HTML report
ruchy test report --format html -o report.html
```

### WASM Development
```bash
# Compile with optimization
ruchy wasm compile app.ruchy -o app.wasm --optimize

# Validate WASM module
ruchy wasm validate app.wasm

# Future: Run WASM module
ruchy wasm run app.wasm --args "arg1" "arg2"
```

## Project Structure

Recommended project layout:
```
my-project/
├── ruchy.toml          # Project configuration
├── src/
│   ├── main.ruchy      # Main entry point
│   ├── lib.ruchy       # Library code
│   └── modules/        # Additional modules
├── tests/
│   └── test_*.ruchy    # Test files
├── examples/
│   └── *.ruchy         # Example scripts
└── docs/
    └── *.md            # Documentation
```

## Configuration

Create `ruchy.toml`:
```toml
[project]
name = "my-project"
version = "0.1.0"
authors = ["Your Name <email@example.com>"]

[dependencies]
std = "1.0"

[features]
default = ["batteries-included"]

[quality]
max-complexity = 10
min-coverage = 80
```

## Next Steps

1. **Explore the REPL**: Best way to learn is interactive experimentation
2. **Read the Documentation**: Check out [FEATURES.md](FEATURES.md) for complete reference
3. **Try the Examples**: Look in the `examples/` directory
4. **Join the Community**: Get help and share your projects
5. **Build Something**: Start with a small project and grow from there

## Getting Help

- **Documentation**: [docs/](../docs/)
- **Examples**: [examples/](../examples/)
- **GitHub Issues**: [Report bugs or request features](https://github.com/paiml/ruchy/issues)
- **Community**: Join our Discord or forum

## Tips and Tricks

1. **REPL Commands**:
   - `:help` - Show help
   - `:load file.ruchy` - Load a file
   - `:type expr` - Show type of expression
   - `:time expr` - Time execution
   - `:clear` - Clear screen

2. **Performance**:
   - Use `--release` flag for optimized builds
   - Profile with `:time` in REPL
   - Use `ruchy test run --parallel` for faster test execution

3. **Debugging**:
   - Set `RUCHY_DEBUG=1` for debug output
   - Use `println` for simple debugging
   - REPL supports step-by-step evaluation

4. **Best Practices**:
   - Keep functions under 10 lines (complexity)
   - Use descriptive names
   - Write tests alongside code
   - Use type annotations for clarity
   - Leverage pattern matching over if-else chains

---

*Happy coding with Ruchy! 🚀*