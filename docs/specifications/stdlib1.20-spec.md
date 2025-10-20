# Ruchy Standard Library Specification v1.20.1

## Core Design Principles

1. **Zero-cost abstractions**: Transpiles directly to Rust standard library methods
2. **Dual-mode execution**: Interpreter implementation + Rust transpilation mapping
3. **Progressive disclosure**: Simple defaults, advanced features when needed
4. **Rust ecosystem leverage**: Direct method mapping without wrapper overhead

## Implementation Status Overview

### Actual Coverage (Revised Analysis)
- **Total Methods**: 71 specified
- **Fully Implemented** (✅): 52 methods (73%) - STDLIB-006 discovered .unique() and .slice() already implemented
- **Partial Implementation** (🟡): 12 methods (17%) - Need custom Rust implementation
- **Not Implemented** (❌): 7 methods (10%)

### Implementation Strategy
The standard library uses a dual-mode approach:
1. **Interpreter Mode**: Direct implementation in `repl.rs` for interactive use
2. **Transpiler Mode**: Maps to Rust's standard library methods (e.g., `.to_upper()` → `.to_uppercase()`)

This leverages Rust's extensive stdlib, achieving zero-cost abstractions for most operations.

### Remaining Gaps
1. **Custom Methods**: Operations without direct Rust equivalents (e.g., `substring`, `unique`)
2. **Advanced File I/O**: Beyond basic read/write operations
3. **Collection Operations**: Some higher-level operations need custom implementation

## Module Organization

```
std/
├── prelude/        # Auto-imported core functions
├── io/             # Input/output operations
├── fs/             # File system operations
├── collections/    # Data structures
├── math/           # Mathematical functions
├── string/         # String manipulation
├── testing/        # Assertion and testing primitives
├── process/        # Process and environment
└── fmt/            # Formatting utilities
```

## Prelude (Auto-imported)

### Core Functions
```rust
// I/O - Status: ✅ Fully implemented
println(value: Any...)         // Print with newline
print(value: Any...)           // Print without newline
eprintln(value: Any...)        // Print to stderr with newline
eprint(value: Any...)          // Print to stderr

// Input - Status: ✅ Fully implemented
input(prompt: String) -> String    // Prompted input
readline() -> String               // Raw line input

// Type conversions - Status: ✅ Fully implemented (STDLIB-001)
str(x: Any) -> String              // Convert to string (wraps Rust Display/to_string)
int(x: Any) -> Int                 // Convert to integer (wraps parse/type casting)
float(x: Any) -> Float             // Convert to float (wraps parse/type casting)
bool(x: Any) -> Bool               // Convert to boolean (truthiness logic)

// Assertions - Status: ✅ Fully implemented
assert(condition: Bool, msg?: String)           // Assert condition
assert_eq(left: Any, right: Any, msg?: String)  // Assert equality
assert_ne(left: Any, right: Any, msg?: String)  // Assert inequality
```

### Collection Constructors
```rust
Vec<T>()           // Empty vector - ✅ Implemented
HashMap<K,V>()     // Empty hashmap - ✅ Implemented
HashSet<T>()       // Empty hashset - ✅ Implemented
```

## Math Module

### Basic Functions - Status: ✅ Fully implemented
```rust
// Arithmetic
abs(x: Number) -> Number
min(a: Number, b: Number) -> Number
max(a: Number, b: Number) -> Number
pow(base: Number, exp: Number) -> Number
sqrt(x: Number) -> Float

// Rounding  
floor(x: Number) -> Int
ceil(x: Number) -> Int
round(x: Number) -> Int

// Constants
PI: Float = 3.14159265358979323846
E: Float = 2.71828182845904523536
```

### Advanced Math - Status: ✅ Fully implemented (STDLIB-002)
```rust
sin(x: Float) -> Float      // Natural sine (wraps f64::sin)
cos(x: Float) -> Float      // Natural cosine (wraps f64::cos)
tan(x: Float) -> Float      // Natural tangent (wraps f64::tan)
log(x: Float) -> Float      // Natural logarithm (wraps f64::ln)
log10(x: Float) -> Float    // Base-10 logarithm (wraps f64::log10)
random() -> Float           // Random float in [0.0, 1.0) (wraps rand::random)
```

**Implementation Details**:
- Zero-cost abstraction pattern: Direct wrapping of Rust stdlib methods
- Dual-mode execution: Both interpreter and transpiler modes supported
- Property tested: 30,000 iterations validating mathematical invariants
  - Pythagorean identity: sin²(x) + cos²(x) = 1
  - Logarithm product rule: log(a*b) = log(a) + log(b)
  - Random range: All values in [0.0, 1.0)
- Complexity: All functions ≤4 (well within Toyota Way limit of 10)
- Test coverage: 16 unit tests + 3 property tests (all passing)

## String Methods

| Method | Status | Transpiler Mapping | Example |
|--------|--------|-------------------|---------|
| `.len()` | ✅ Full | `.len()` | `"hello".len() // 5` |
| `.is_empty()` | ✅ Full | `.is_empty()` | `"".is_empty() // true` |
| `.to_upper()` | ✅ Full | `.to_uppercase()` | `"hello".to_upper() // "HELLO"` |
| `.to_lower()` | ✅ Full | `.to_lowercase()` | `"HELLO".to_lower() // "hello"` |
| `.trim()` | ✅ Full | `.trim()` | `"  hello  ".trim() // "hello"` |
| `.split()` | ✅ Full | `.split().collect()` | `"a,b,c".split(",") // ["a","b","c"]` |
| `.contains()` | ✅ Full | `.contains()` | `"hello".contains("ll") // true` |
| `.starts_with()` | ✅ Full | `.starts_with()` | `"hello".starts_with("he") // true` |
| `.ends_with()` | ✅ Full | `.ends_with()` | `"hello".ends_with("lo") // true` |
| `.replace()` | ✅ Full | `.replace()` | `"hello".replace("l", "r") // "herro"` |
| `.substring()` | 🟡 Partial | Custom impl needed | `"hello".substring(1, 3) // "el"` |
| `.chars()` | ✅ Full | `.chars().collect()` | `"hello".chars() // ['h','e','l','l','o']` |
| `.reverse()` | ✅ Full | `.chars().rev().collect()` | `"hello".reverse() // "olleh"` |
| `.repeat()` | ✅ Full | `.repeat()` | `"ab".repeat(3) // "ababab"` |

## Array/Vec Methods

| Method | Status | Transpiler Mapping | Example |
|--------|--------|-------------------|---------|
| `.len()` | ✅ Full | `.len()` | `[1,2,3].len() // 3` |
| `.is_empty()` | ✅ Full | `.is_empty()` | `[].is_empty() // true` |
| `.first()` | ✅ Full | `.first().cloned()` | `[1,2,3].first() // Some(1)` |
| `.last()` | ✅ Full | `.last().cloned()` | `[1,2,3].last() // Some(3)` |
| `.get()` | ✅ Full | `.get().cloned()` | `[1,2,3].get(1) // Some(2)` |
| `.push()` | ✅ Full | `.push()` | `vec.push(3)` |
| `.pop()` | ✅ Full | `.pop()` | `vec.pop() // Some(3)` |
| `.reverse()` | ✅ Full | `.reverse()` | `[1,2,3].reverse() // [3,2,1]` |
| `.map()` | ✅ Full | `.iter().map().collect()` | `[1,2].map(\|x\| x*2) // [2,4]` |
| `.filter()` | ✅ Full | `.iter().filter().collect()` | `[1,2,3].filter(\|x\| x>1) // [2,3]` |
| `.reduce()` | ✅ Full | `.iter().fold()` | `[1,2,3].reduce(0, \|a,b\| a+b) // 6` |
| `.sum()` | ✅ Full | `.iter().sum()` | `[1,2,3].sum() // 6` |
| `.take()` | ✅ Full | `.iter().take().collect()` | `[1,2,3,4].take(2) // [1,2]` |
| `.skip()` | ✅ Full | `.iter().skip().collect()` | `[1,2,3,4].skip(2) // [3,4]` |
| `.sort()` | 🟡 Partial | `.sort()` (mutable) | `vec.sort()` |
| `.join()` | 🟡 Partial | Custom impl | `["a","b"].join(",") // "a,b"` |
| `.slice()` | ✅ Full | `[start..end].to_vec()` | `[1,2,3,4].slice(1,3) // [2,3]` |
| `.concat()` | ✅ Full | `.extend_from_slice()` | `[1,2].concat([3,4]) // [1,2,3,4]` |
| `.flatten()` | ✅ Full | Custom impl | `[[1,2],[3]].flatten() // [1,2,3]` |
| `.unique()` | ✅ Full | HashSet-based dedup | `[1,2,1,3].unique() // [1,2,3]` |

## HashMap Methods

| Method | Status | Signature | Example |
|--------|--------|-----------|---------|
| `.len()` | ✅ Full | `() -> Int` | `map.len()` |
| `.is_empty()` | ✅ Full | `() -> Bool` | `map.is_empty()` |
| `.insert()` | ✅ Full | `(K, V) -> Option<V>` | `map.insert("key", 42)` |
| `.get()` | ✅ Full | `(K) -> Option<V>` | `map.get("key") // Some(42)` |
| `.remove()` | ✅ Full | `(K) -> Option<V>` | `map.remove("key")` |
| `.contains_key()` | ✅ Full | `(K) -> Bool` | `map.contains_key("key")` |
| `.clear()` | ✅ Full | `()` | `map.clear()` |
| `.keys()` | ✅ Full | `() -> Vec<K>` | `{"a":1}.keys() // ["a"]` |
| `.values()` | ✅ Full | `() -> Vec<V>` | `{"a":1}.values() // [1]` |
| `.items()` | ✅ Full | `() -> Vec<(K,V)>` | `{"a":1}.items() // [("a",1)]` |

## HashSet Methods

| Method | Status | Signature | Example |
|--------|--------|-----------|---------|
| `.len()` | ✅ Full | `() -> Int` | `set.len()` |
| `.is_empty()` | ✅ Full | `() -> Bool` | `set.is_empty()` |
| `.insert()` | ✅ Full | `(T) -> Bool` | `set.insert(42) // true if new` |
| `.remove()` | ✅ Full | `(T) -> Bool` | `set.remove(42)` |
| `.contains()` | ✅ Full | `(T) -> Bool` | `set.contains(42)` |
| `.clear()` | ✅ Full | `()` | `set.clear()` |
| `.union()` | ❌ | `(HashSet<T>) -> HashSet<T>` | `set1.union(set2)` |
| `.intersection()` | ❌ | `(HashSet<T>) -> HashSet<T>` | `set1.intersection(set2)` |
| `.difference()` | ❌ | `(HashSet<T>) -> HashSet<T>` | `set1.difference(set2)` |

## Option/Result Types

### Option<T>
```rust
Some(value: T) -> Option<T>
None -> Option<T>

// Methods
.unwrap() -> T                    // Panics on None
.expect(msg: String) -> T          // Panics with message on None
.unwrap_or(default: T) -> T        // Returns default on None
.map<U>(f: T -> U) -> Option<U>
.and_then<U>(f: T -> Option<U>) -> Option<U>
.is_some() -> Bool
.is_none() -> Bool
```

### Result<T, E>
```rust
Ok(value: T) -> Result<T, E>
Err(error: E) -> Result<T, E>

// Methods
.unwrap() -> T                     // Panics on Err
.expect(msg: String) -> T           // Panics with message on Err
.unwrap_or(default: T) -> T         // Returns default on Err
.map<U>(f: T -> U) -> Result<U, E>
.and_then<U>(f: T -> Result<U, E>) -> Result<U, E>
.is_ok() -> Bool
.is_err() -> Bool

// Try operator
?  // Propagates Err, unwraps Ok
```

## File I/O

| Function | Status | Signature | Example |
|----------|--------|-----------|---------|
| `read_file()` | ✅ Full | `(String) -> String` | `read_file("data.txt")` |
| `write_file()` | ✅ Full | `(String, String) -> Result<(), Error>` | `write_file("out.txt", content)` |
| `append_file()` | ❌ | `(String, String) -> Result<(), Error>` | `append_file("log.txt", line)` |
| `file_exists()` | ❌ | `(String) -> Bool` | `file_exists("config.json")` |
| `delete_file()` | ❌ | `(String) -> Result<(), Error>` | `delete_file("temp.txt")` |
| `File::open()` | ❌ | `(String) -> Result<File>` | `File::open("data.txt")` |
| `File::create()` | ❌ | `(String) -> Result<File>` | `File::create("out.txt")` |

## Process and Environment

```rust
// Functions
env(key: String) -> Option<String>
set_env(key: String, value: String)
args() -> Vec<String>
exit(code: Int)
current_dir() -> String
set_current_dir(path: String) -> Result<(), Error>
```

## Control Flow

### Loops
```rust
// For loops
for i in 0..10 { }               // Range (exclusive)
for i in 0..=10 { }              // Range (inclusive)
for item in collection { }       // Iterator
for (index, item) in collection.enumerate() { }
for (key, value) in map.items() { }

// While loops
while condition { }

// Loop with break
loop {
    if condition { break }
}
```

### Pattern Matching
```rust
match value {
    pattern => result,
    pattern if guard => result,
    1..10 => "range",
    1 | 2 | 3 => "or pattern",
    Some(x) => x,
    None => default,
    _ => "wildcard"
}
```

## Function Syntax

```rust
// Regular function
fun add(x: Int, y: Int) -> Int {
    x + y
}

// Generic function
fun identity<T>(x: T) -> T {
    x
}

// Lambda expressions
|x| x + 1              // Rust-style
x => x + 1             // Arrow style

// Higher-order functions
fun map<T, U>(list: Vec<T>, f: T -> U) -> Vec<U> {
    list.map(f)
}

// Async functions
async fun fetch_data() -> Result<String, Error> {
    let response = await http_get("url")?;
    Ok(response.body)
}
```

## Type System

### Basic Types
```rust
// Primitives
Bool, Int, Float, Char, String

// Numeric types (Rust compatible)
i8, i16, i32, i64, i128, isize
u8, u16, u32, u64, u128, usize
f32, f64

// Compound types
(T, U, ...)           // Tuples
Vec<T>                // Vectors
HashMap<K, V>         // Hash maps
HashSet<T>            // Hash sets
```

### Custom Types
```rust
// Structs
struct Point {
    x: Float,
    y: Float
}

// Enums
enum Result<T, E> {
    Ok(T),
    Err(E)
}

// Traits
trait Display {
    fun display(self) -> String
}

// Implementations
impl Display for Point {
    fun display(self) -> String {
        f"Point({self.x}, {self.y})"
    }
}
```

## Pipeline Operator

```rust
// Function chaining with |>
data
  |> transform
  |> filter(|x| x > 0)
  |> map(|x| x * 2)
  |> reduce(0, |acc, x| acc + x)

// Equivalent to nested calls
reduce(map(filter(transform(data), |x| x > 0), |x| x * 2), 0, |acc, x| acc + x)
```

## Module System

```rust
// Module declaration
mod math {
    pub fun sqrt(x: Float) -> Float { /* ... */ }
}

// Use statements
use std::collections::HashMap
use math::sqrt

// Path resolution
std::fs::read_file("path")
::global::path::to::module
```

## Testing

```rust
#[test]
fun test_addition() {
    assert_eq(2 + 2, 4)
}

#[test]
fun test_with_assertion() {
    let result = compute();
    assert(result > 0, "Result must be positive")
}
```

## String Interpolation

```rust
let name = "World"
let greeting = f"Hello, {name}!"  // f-string style
println("Value: {}", value)       // Format string style
```

## Execution Model

1. **REPL Mode**: Interactive evaluation with persistent state
2. **Script Mode**: Top-level statements with optional main()
3. **Compiled Mode**: Transpiles to optimized Rust code

## Performance Guarantees

- **Zero-cost abstractions**: Standard library functions compile to optimal Rust
- **Inline caching**: Method lookups cached for performance
- **Type specialization**: Generic functions monomorphized at compile time
- **Iterator fusion**: Chain operations compiled to single loop
- **String interning**: Duplicate strings share memory

## Sister Project Integration

The standard library is designed to support:
- **ruchy-book**: All educational examples
- **rosetta-ruchy**: Algorithm implementations
- **ruchyruchy**: Self-hosting compiler requirements

## Version History

- **v1.20.1**: Fixed while loop and object.items() bugs
- **v1.18.0**: Higher-order function support
- **v1.9.x**: Collection methods, I/O functions
- **v1.0.0**: Core language features