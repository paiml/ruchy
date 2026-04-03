# Sub-spec: Classes & OOP — Design Rationale, Structs, Traits, Actors & Composition

**Parent:** [ruchy_classes_spec.md](../ruchy_classes_spec.md) Sections 1-13

---

## 1. Design Rationale

### 1.1 Why No Classes

Traditional class hierarchies require one of three implementation strategies in Rust, each problematic:

| Strategy | Implementation | Runtime Cost | Compile Cost |
|----------|---------------|--------------|--------------|
| Trait Objects | `Box<dyn Trait>` | Virtual dispatch, heap allocation | Moderate |
| Enum Dispatch | Tagged unions | Branch prediction miss | Closed set |
| Macro Generation | Code generation | None | Exponential growth |

Ruchy instead leverages Rust's existing composition patterns with syntax sugar.

### 1.2 Mechanical Transformation Principle

Every Ruchy construct must have a deterministic, zero-cost Rust equivalent:

```rust
// Ruchy source
struct Point { x: f64, y: f64 }
impl Point {
    fun distance(&self) -> f64 = (self.x² + self.y²).sqrt()
}

// Generated Rust (character-for-character predictable)
struct Point { x: f64, y: f64 }
impl Point {
    fn distance(&self) -> f64 { (self.x.powi(2) + self.y.powi(2)).sqrt() }
}
```

## 2. Struct Definition

### 2.1 Basic Syntax

```rust
struct Name {
    field: Type,                // Private by default
    pub optional: Type? = None, // Public optional with default
    mutable: mut Type,          // Interior mutability hint
    pub public_field: Type,     // Explicit public field
}
```

### 2.2 Transpilation Rules

| Ruchy | Rust Output |
|-------|-------------|
| `field: Type` | `field: Type` (private by default) |
| `pub field: Type` | `pub field: Type` |
| `field: Type?` | `field: Option<Type>` |
| `pub field: Type?` | `pub field: Option<Type>` |
| `field: Type = value` | Constructor default |
| `mut field: Type` | `field: RefCell<Type>` |
| `pub mut field: Type` | `pub field: RefCell<Type>` |

### 2.3 Constructor Generation

Structs automatically get a builder pattern constructor:

```rust
// Ruchy
struct Config {
    host: String = "localhost",
    port: u16 = 8080,
    timeout: Duration? = None
}

// Generated Rust
impl Config {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout: None,
        }
    }
    
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}
```

## 3. Implementation Blocks

### 3.1 Method Syntax

```rust
impl StructName {
    // Constructor convention
    fun new(...) -> Self { ... }
    
    // Instance methods
    fun method(&self) -> T { ... }
    fun mut_method(&mut self) { ... }
    
    // Associated functions
    fun static_fn() -> T { ... }
    
    // Property getter/setter sugar
    get property(&self) -> T { self.field }
    set property(&mut self, val: T) { self.field = val }
}
```

### 3.2 Method Transpilation

```rust
// Ruchy compact form
impl Point {
    fun distance(&self) = (self.x² + self.y²).sqrt()
}

// Expands to
impl Point {
    pub fn distance(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

## 4. Trait System

### 4.1 Trait Definition

```rust
trait Drawable {
    // Required methods
    fun draw(&self);
    
    // Default implementations
    fun bounds(&self) -> Rect = {
        Rect::default()
    }
    
    // Associated types
    type Color;
    
    // Associated constants
    const MAX_SIZE: u32 = 1000;
}
```

### 4.2 Trait Implementation

```rust
impl Drawable for Circle {
    type Color = RGB;
    
    fun draw(&self) {
        // Implementation
    }
}
```

## 5. Extension Methods

### 5.1 Syntax

```rust
// Extend existing types with new methods
extend String {
    fun is_palindrome(&self) -> bool {
        let clean = self.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        clean == clean.chars().rev().collect()
    }
}

// Usage
"racecar".is_palindrome()  // true
```

### 5.2 Transpilation

Extension methods generate trait implementations:

```rust
// Generated Rust
trait StringExt {
    fn is_palindrome(&self) -> bool;
}

impl StringExt for String {
    fn is_palindrome(&self) -> bool {
        let clean = self.chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        clean == clean.chars().rev().collect::<String>()
    }
}
```

## 6. Actor Pattern

### 6.1 Actor Definition

Actors provide encapsulation similar to classes but with message-passing semantics:

```rust
actor Counter {
    // Private state
    count: i32 = 0,
    
    // Message handlers
    receive increment() {
        self.count += 1;
    }
    
    receive decrement() {
        self.count -= 1;
    }
    
    receive get() -> i32 {
        self.count
    }
    
    // Lifecycle hooks
    on_start() {
        println!("Counter starting");
    }
    
    on_stop() {
        println!("Final count: {}", self.count);
    }
}
```

### 6.2 Actor Transpilation

Actors generate approximately 200 lines of boilerplate:

```rust
// Message enum generation
enum CounterMessage {
    Increment,
    Decrement,
    Get { reply: oneshot::Sender<i32> },
}

// State struct
struct CounterState {
    count: i32,
}

// Actor implementation
struct CounterActor {
    state: CounterState,
    receiver: mpsc::Receiver<CounterMessage>,
}

// Handle struct for external interface
pub struct Counter {
    sender: mpsc::Sender<CounterMessage>,
}

impl Counter {
    pub async fn increment(&self) -> Result<()> {
        self.sender.send(CounterMessage::Increment).await
    }
    
    pub async fn get(&self) -> Result<i32> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(CounterMessage::Get { reply: tx }).await?;
        rx.await
    }
}
```

## 7. Composition Patterns

### 7.1 Delegation

```rust
struct Engine {
    horsepower: u32
}

struct Car {
    engine: Engine,
    
    // Delegate to engine
    delegate horsepower to engine;
}

// Generates
impl Car {
    pub fn horsepower(&self) -> u32 {
        self.engine.horsepower
    }
}
```

### 7.2 Mixins via Traits

```rust
// Define reusable behavior
trait Timestamped {
    fun created_at(&self) -> DateTime;
    fun updated_at(&self) -> DateTime;
}

// Mix into structs
struct Post with Timestamped {
    title: String,
    content: String,
}

// Generates fields and impl
struct Post {
    title: String,
    content: String,
    created_at: DateTime,
    updated_at: DateTime,
}
```

## 8. Property System

### 8.1 Computed Properties

```rust
impl Rectangle {
    get area(&self) -> f64 {
        self.width * self.height
    }
    
    get perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

// Usage
let r = Rectangle { width: 10.0, height: 20.0 };
println!("{}", r.area);  // Calls area()
```

### 8.2 Observable Properties

```rust
struct Model {
    @observable name: String,
}

// Generates
impl Model {
    pub fn set_name(&mut self, name: String) {
        let old = std::mem::replace(&mut self.name, name);
        self.notify_observers("name", &old, &self.name);
    }
}
```

## 9. Syntax Sugar Mappings

| Ruchy Feature | Rust Equivalent | Line Reduction |
|---------------|-----------------|----------------|
| `extend Type` | Trait + impl | 40% |
| `actor Name` | Struct + channels + tasks | 95% |
| `get/set` | Getter/setter methods | 60% |
| `delegate` | Forwarding methods | 80% |
| `with Trait` | Trait fields + impl | 70% |

## 10. Non-Features (Deliberate Omissions)

### 10.1 No Inheritance

```rust
// NOT SUPPORTED
struct Animal { name: String }
struct Dog extends Animal { breed: String }  // ❌

// INSTEAD USE
trait Animal {
    fun name(&self) -> &str;
}

struct Dog {
    name: String,
    breed: String,
}

impl Animal for Dog {
    fun name(&self) -> &str { &self.name }
}
```

### 10.2 No Method Overloading

```rust
// NOT SUPPORTED
impl Calculator {
    fun add(x: i32, y: i32) -> i32 { x + y }      // ❌
    fun add(x: f64, y: f64) -> f64 { x + y }      // ❌
}

// INSTEAD USE
impl Calculator {
    fun add_i32(x: i32, y: i32) -> i32 { x + y }
    fun add_f64(x: f64, y: f64) -> f64 { x + y }
}
```

### 10.3 No Dynamic Dispatch by Default

```rust
// NOT SUPPORTED (implicit boxing)
let shapes: Vec<Shape> = vec![Circle::new(), Square::new()];  // ❌

// EXPLICIT WHEN NEEDED
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle::new()),
    Box::new(Square::new()),
];
```

## 11. Implementation Timeline

| Feature | Status | Target | Complexity |
|---------|--------|--------|------------|
| Structs | ✅ Complete | - | Low |
| Impl blocks | ✅ Complete | - | Low |
| Traits | ✅ Complete | - | Medium |
| Extension methods | [x] In Progress | v0.7.12 | Medium |
| Actors | 📅 Planned | v0.4 | High |
| Properties | 📅 Planned | v0.5 | Medium |
| Delegation | 📅 Planned | v0.6 | Low |

## 12. Performance Guarantees

All object-oriented features maintain zero-cost abstraction:

```rust
// Ruchy struct method call
point.distance()

// Compiles to identical assembly as
Point::distance(&point)

// No vtable lookup, no indirection
```

## 13. Testing Requirements

Each OOP feature requires:

1. **Transpilation tests**: Ruchy input → expected Rust output
2. **Semantic tests**: Behavior preservation
3. **Performance tests**: Zero overhead verification
4. **Error tests**: Meaningful error messages

## Appendix A: Actor Implementation Detail

Actor implementation details to be documented.

## Appendix B: Extension Method Resolution

Resolution follows Rust's trait rules:
1. Inherent methods first
2. Extension methods in scope
3. Error on ambiguity
