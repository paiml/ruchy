# Ruchy REPL Guide

## 🚀 Quick Start - The Golden Path

The Ruchy REPL is your interactive playground for learning and experimenting with the language. Like Elixir's IEx or Julia's REPL, it provides immediate feedback and is the best way to learn Ruchy.

### Start the REPL

```bash
cargo run -p ruchy-cli -- repl
# Or if installed: ruchy repl
```

You'll see:
```
Welcome to Ruchy REPL v0.4.0
Type :help for commands, :quit to exit

ruchy> 
```

## 📚 Essential Examples That Work Today

### 1. Basic Arithmetic

```ruchy
ruchy> 1 + 2
3

ruchy> 10 * 5
50

ruchy> 100 / 4
25

ruchy> 7 % 3
1

ruchy> 2 ** 8
256
```

### 2. Variables and Bindings

```ruchy
ruchy> let x = 10
10

ruchy> let y = 20
20

ruchy> x + y
30

ruchy> let message = "Hello"
"Hello"

ruchy> let pi = 3.14159
3.14159
```

### 3. String Operations

```ruchy
ruchy> "Hello" + " World"
"Hello World"

ruchy> let name = "Ruchy"
"Ruchy"

ruchy> let greeting = "Welcome to " + name
"Welcome to Ruchy"
```

### 4. Printing Output

```ruchy
ruchy> println("Hello, World!")
Hello, World!
()

ruchy> print("Loading")
Loading()

ruchy> println("The answer is", 42)
The answer is 42
()

ruchy> let x = 100
100
ruchy> println("x =", x)
x = 100
()
```

### 5. Boolean Logic

```ruchy
ruchy> true
true

ruchy> false
false

ruchy> true && false
false

ruchy> true || false
true

ruchy> !true
false

ruchy> 5 > 3
true

ruchy> 10 == 10
true

ruchy> "hello" == "hello"
true
```

### 6. Conditional Expressions

```ruchy
ruchy> if true { 1 } else { 2 }
1

ruchy> let age = 18
18

ruchy> if age >= 18 { "adult" } else { "minor" }
"adult"

ruchy> let score = 85
85

ruchy> if score > 90 { "A" } else { if score > 80 { "B" } else { "C" } }
"B"
```

### 7. Lists and Collections

```ruchy
ruchy> [1, 2, 3]
1

ruchy> let nums = [10, 20, 30]
10

ruchy> []
()
```

### 8. Blocks and Compound Expressions

```ruchy
ruchy> { 
    let a = 5;
    let b = 10;
    a + b
}
15

ruchy> {
    println("Computing...");
    42
}
Computing...
42
```

### 9. Pattern Matching

```ruchy
ruchy> match 5 {
    0 => "zero",
    1 => "one",
    _ => "other"
}
"other"

ruchy> let x = 2
2

ruchy> match x {
    1 | 2 | 3 => "small",
    _ => "big"
}
"small"
```

### 10. Functions (Basic Definition)

```ruchy
ruchy> fun add(a: i32, b: i32) -> i32 { a + b }
"fn add(a, b)"

ruchy> fun greet(name: String) { println("Hello", name) }
"fn greet(name)"
```

### 11. Lambda Expressions

```ruchy
ruchy> |x| x + 1
"|x| <body>"

ruchy> |x, y| x * y
"|x, y| <body>"
```

### 12. Range Expressions

```ruchy
ruchy> 0..10
"0..10"

ruchy> 1..5
"1..5"
```

## 🔧 REPL Commands

### Information Commands

```ruchy
:help              # Show available commands
:history           # Show command history
:bindings          # Show current variable bindings
```

### Session Management

```ruchy
:clear             # Clear all bindings and start fresh
:quit or :q        # Exit the REPL
```

### Development Commands

```ruchy
:compile           # Compile current session to Rust
:load <file>       # Load and execute a .ruchy file
```

## 📋 Currently Supported Grammar

### ✅ Working Features

| Feature | Example | Output |
|---------|---------|--------|
| **Integers** | `42` | `42` |
| **Floats** | `3.14` | `3.14` |
| **Strings** | `"hello"` | `"hello"` |
| **Booleans** | `true`, `false` | `true`, `false` |
| **Arithmetic** | `1 + 2 * 3` | `7` |
| **Comparisons** | `5 > 3` | `true` |
| **Logic** | `true && false` | `false` |
| **Variables** | `let x = 10` | `10` |
| **If/Else** | `if x > 0 { "pos" } else { "neg" }` | `"pos"` |
| **Blocks** | `{ let x = 1; x + 1 }` | `2` |
| **Match** | `match x { 1 => "one", _ => "other" }` | Result varies |
| **Functions** | `fun f(x: i32) { x }` | Function stored |
| **Lambdas** | `\|x\| x * 2` | Lambda stored |
| **Printing** | `println("Hi")` | Prints `Hi` |
| **String Concat** | `"a" + "b"` | `"ab"` |

### ⚠️ Partially Working

| Feature | Issue | Workaround |
|---------|-------|------------|
| **Mixed arithmetic** | `3.14 * 2` fails (type mismatch) | Use same types: `3.14 * 2.0` |
| **Function calls** | User functions not callable yet | Use built-ins like `println` |
| **For loops** | Not implemented in evaluator | Use recursion or match |
| **List operations** | Limited evaluation | Lists define but don't operate |

### 🚧 Not Yet Implemented

- Async/await expressions
- Actor system (`!` and `?` operators)
- DataFrame operations
- Method calls on objects
- Import statements
- Type annotations in let bindings
- Complex pattern matching
- List comprehensions
- Try/catch blocks

## 💡 Tips and Tricks

### 1. Multi-line Input
The REPL supports multi-line expressions. Just keep typing:

```ruchy
ruchy> if true {
    println("This is");
    println("multi-line");
    42
}
This is
multi-line
42
```

### 2. Expression Values
Everything is an expression and returns a value:

```ruchy
ruchy> let result = if 5 > 3 { "yes" } else { "no" }
"yes"

ruchy> let computation = {
    let x = 10;
    let y = 20;
    x + y
}
30
```

### 3. Debugging with Print
Use `println` liberally to understand what's happening:

```ruchy
ruchy> {
    let x = 5;
    println("x is", x);
    let y = x * 2;
    println("y is", y);
    x + y
}
x is 5
y is 10
15
```

### 4. Type Exploration
The REPL shows you the type of expressions through their values:

```ruchy
ruchy> 42
42                    # Integer

ruchy> 3.14
3.14                  # Float

ruchy> "text"
"text"                # String

ruchy> true
true                  # Boolean

ruchy> ()
()                    # Unit type

ruchy> [1, 2, 3]
1                     # List (shows first element)
```

## 🎯 Common Patterns

### Calculator Mode
```ruchy
ruchy> let tax_rate = 0.08
0.08

ruchy> let price = 100
100

ruchy> let tax = price * tax_rate
Error: Type mismatch    # Oops! Need same types

ruchy> let price = 100.0
100.0

ruchy> let tax = price * tax_rate
8.0

ruchy> let total = price + tax
108.0

ruchy> println("Total with tax:", total)
Total with tax: 108.0
()
```

### Decision Making
```ruchy
ruchy> let score = 75
75

ruchy> let grade = if score >= 90 {
    "A"
} else { if score >= 80 {
    "B"
} else { if score >= 70 {
    "C"
} else {
    "F"
}}}
"C"

ruchy> println("Your grade:", grade)
Your grade: C
()
```

### Building Up Computations
```ruchy
ruchy> let base = 100
100

ruchy> let bonus = 20
20

ruchy> let penalty = 5
5

ruchy> let final_score = base + bonus - penalty
115

ruchy> println("Final score:", final_score)
Final score: 115
()
```

## 🔍 Troubleshooting

### Common Errors and Solutions

**Type Mismatch**
```ruchy
ruchy> 3.14 * 2
Error: Type mismatch in binary operation

# Solution: Use consistent types
ruchy> 3.14 * 2.0
6.28
```

**Undefined Variable**
```ruchy
ruchy> x + 1
Error: Undefined variable: x

# Solution: Define the variable first
ruchy> let x = 10
10
ruchy> x + 1
11
```

**String + Number**
```ruchy
ruchy> "The answer is " + 42
Error: Type mismatch

# Solution: Use println for mixed types
ruchy> println("The answer is", 42)
The answer is 42
()
```

## 📖 Learning Path

1. **Start Simple**: Basic arithmetic and variables
2. **Add Logic**: Boolean expressions and if/else
3. **Use Functions**: Define and understand function syntax
4. **Pattern Match**: Learn match expressions
5. **Combine**: Build larger expressions from smaller ones

## 🚦 Quick Reference Card

```ruchy
# Numbers
42, 3.14, 2 ** 8

# Strings  
"hello", "a" + "b"

# Booleans
true, false, !true, a && b, x || y

# Variables
let x = 10
let name = "Ruchy"

# Conditionals
if condition { expr1 } else { expr2 }

# Pattern Matching
match value {
    pattern1 => result1,
    pattern2 => result2,
    _ => default
}

# Functions
fun name(param: Type) -> RetType { body }

# Lambdas
|param| expression
|x, y| x + y

# Printing
println("text", value1, value2)
print("no newline")

# Blocks
{
    statement1;
    statement2;
    final_expression
}

# Commands
:help, :quit, :history, :clear, :bindings
```

## Next Steps

Once comfortable with the REPL basics:

1. Try loading example files with `:load examples/fibonacci.ruchy`
2. Experiment with more complex expressions
3. Use `:compile` to see generated Rust code
4. Read the [Language Specification](./SPECIFICATION.md) for advanced features
5. Check [ROADMAP.md](../ROADMAP.md) to see what's coming next

Remember: The REPL is your friend! It's the fastest way to learn Ruchy and test ideas. Keep it open while coding and use it to verify your understanding.