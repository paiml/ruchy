# Ruchy Class Sugar Specification
Version 1.0.0

## Abstract

This specification defines syntactic sugar for class-like constructs in Ruchy that provide familiar object-oriented syntax while transpiling to idiomatic Rust code with zero runtime overhead.

## Motivation

- Lower barrier to entry for developers from OOP backgrounds
- Provide progressive disclosure path from familiar to idiomatic
- Maintain zero-cost abstraction principle
- Enable gradual migration from class-based codebases

## Syntax Grammar

```ebnf
class_decl = [visibility] "class" ident [inheritance] [traits] "{" class_body "}"

visibility = "pub" | "pub(crate)" | "pub(super)"

inheritance = ":" ident

traits = "+" ident ("+" ident)*

class_body = (field_decl | constructor | method_decl | static_decl)*

field_decl = [visibility] ["mut"] ident ":" type ["=" expr]

constructor = "new" "(" params ")" ["->" "Self"] block

method_decl = [visibility] ["override"] "fn" ident "(" self_param ["," params] ")" ["->" type] block

self_param = "&self" | "&mut self" | "self"

static_decl = [visibility] "static" "fn" ident "(" params ")" ["->" type] block
```

## Core Semantics

### 1. Fields

Fields are struct members with controlled mutability:

```ruchy
class Point {
    x: f64           # Private, immutable
    mut y: f64       # Private, mutable
    pub z: f64       # Public, immutable
    pub mut w: f64   # Public, mutable
    
    # With defaults
    name: String = "origin"
    mut counter: i32 = 0
}
```

**Rules:**
- Fields without `mut` cannot be modified after construction
- Default visibility is private
- Defaults must be const expressions or `Default::default()`

### 2. Constructors

Constructors use the `new` keyword:

```ruchy
class Rectangle {
    width: f64
    height: f64
    
    # Primary constructor
    new(width: f64, height: f64) {
        self.width = width
        self.height = height
    }
    
    # Overloaded constructor
    new square(size: f64) {
        self.width = size
        self.height = size
    }
    
    # Constructor with validation
    new safe(width: f64, height: f64) -> Result<Self> {
        if width <= 0.0 || height <= 0.0 {
            return Err("Dimensions must be positive")
        }
        Ok(Self { width, height })
    }
}
```

**Rules:**
- Multiple constructors allowed via name overloading
- Must initialize all fields without defaults
- Cannot access methods during construction
- `self` refers to fields being constructed, not instance

### 3. Methods

Three borrowing modes for self:

```ruchy
class Counter {
    mut count: i32 = 0
    
    # Immutable borrow
    fn get(&self) -> i32 {
        self.count
    }
    
    # Mutable borrow
    fn increment(&mut self) {
        self.count += 1
    }
    
    # Move/consume
    fn into_value(self) -> i32 {
        self.count
    }
    
    # Static (no self)
    static fn new_zero() -> Self {
        Counter { count: 0 }
    }
}
```

### 4. Inheritance

Single inheritance with explicit super calls:

```ruchy
class Vehicle {
    wheels: u32
    
    new(wheels: u32) {
        self.wheels = wheels
    }
    
    fn description(&self) -> String {
        format!("Vehicle with {} wheels", self.wheels)
    }
}

class Car : Vehicle {
    brand: String
    
    new(brand: String) {
        super(wheels: 4)  # Must be first statement
        self.brand = brand
    }
    
    override fn description(&self) -> String {
        format!("{} car with {} wheels", self.brand, self.wheels)
    }
}
```

**Rules:**
- Single inheritance only
- `super()` must be first statement in constructor
- `override` required for method overriding
- No access to parent private fields
- `final` prevents further overriding

### 5. Trait Implementation

Traits can be mixed in:

```ruchy
class Point3D : Point + Debug + Display {
    z: f64
    
    new(x: f64, y: f64, z: f64) {
        super(x, y)
        self.z = z
    }
    
    # Implement Display
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
```

## Transpilation Rules

### Basic Class

**Ruchy:**
```ruchy
pub class Person {
    name: String
    mut age: u32 = 0
    
    new(name: String) {
        self.name = name
    }
    
    fn greet(&self) -> String {
        format!("Hello, I'm {}", self.name)
    }
    
    fn birthday(&mut self) {
        self.age += 1
    }
}
```

**Generated Rust:**
```rust
#[derive(Clone)]
pub struct Person {
    name: String,
    age: u32,
}

impl Person {
    pub fn new(name: String) -> Self {
        Self { 
            name,
            age: 0,
        }
    }
    
    pub fn greet(&self) -> String {
        format!("Hello, I'm {}", self.name)
    }
    
    pub fn birthday(&mut self) {
        self.age += 1;
    }
}

impl Default for Person {
    fn default() -> Self {
        Self {
            name: Default::default(),
            age: 0,
        }
    }
}
```

### Inheritance

**Ruchy:**
```ruchy
class Animal {
    name: String
    
    new(name: String) {
        self.name = name
    }
    
    fn speak(&self) -> String {
        "..."
    }
}

class Dog : Animal {
    breed: String
    
    new(name: String, breed: String) {
        super(name)
        self.breed = breed
    }
    
    override fn speak(&self) -> String {
        "Woof!"
    }
}
```

**Generated Rust:**
```rust
trait Animal: AnimalBase {
    fn speak(&self) -> String {
        "..."
    }
}

struct AnimalBase {
    name: String,
}

struct Dog {
    _base: AnimalBase,
    breed: String,
}

impl Dog {
    pub fn new(name: String, breed: String) -> Self {
        Self {
            _base: AnimalBase { name },
            breed,
        }
    }
}

impl Animal for Dog {
    fn speak(&self) -> String {
        "Woof!"
    }
}

impl AnimalBase for Dog {
    fn name(&self) -> &String {
        &self._base.name
    }
}
```

## Advanced Features

### Property Syntax

Computed properties with getters/setters:

```ruchy
class Circle {
    mut radius: f64
    
    property area: f64 {
        get { PI * self.radius * self.radius }
    }
    
    property diameter: f64 {
        get { self.radius * 2.0 }
        set(value) { self.radius = value / 2.0 }
    }
}
```

### Operator Overloading

```ruchy
class Vector {
    x: f64
    y: f64
    
    operator +(other: &Vector) -> Vector {
        Vector { x: self.x + other.x, y: self.y + other.y }
    }
    
    operator [](index: usize) -> f64 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => panic!("Index out of bounds")
        }
    }
}
```

### Generic Classes

```ruchy
class Box<T> {
    value: T
    
    new(value: T) {
        self.value = value
    }
    
    fn map<U>(&self, f: impl Fn(&T) -> U) -> Box<U> {
        Box::new(f(&self.value))
    }
}
```

## Restrictions

1. **No Multiple Inheritance** - Use composition and traits
2. **No Class Variables** - Use associated constants or statics
3. **No Metaclasses** - No runtime class manipulation
4. **No Dynamic Dispatch by Default** - Must use `dyn` explicitly
5. **No Null Fields** - Use `Option<T>` for nullable fields
6. **No Implicit Conversions** - All conversions explicit
7. **No Protected Visibility** - Only public and private
8. **No Friend Classes** - Use module visibility

## Compatibility Mode

Enable stricter Python/Ruby compatibility:

```ruchy
#[compat(python)]
class MyClass:  # Python-style colon
    def __init__(self, x):  # Python __init__
        self.x = x
    
    def method(self):
        return self.x
```

## Error Messages

Class sugar provides specialized error messages:

```
Error: Field 'name' not initialized in constructor
  --> src/main.ruchy:8:5
   |
 8 |     new(age: u32) {
   |     ^^^
   |
help: Initialize field 'name' or provide a default value
   |
 3 |     name: String = String::new()
   |                  ^^^^^^^^^^^^^^^^
```

## Migration Path

### From Python
```python
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    
    def distance(self):
        return (self.x**2 + self.y**2)**0.5
```

### To Ruchy (Sugar)
```ruchy
class Point {
    x: f64
    y: f64
    
    new(x: f64, y: f64) {
        self.x = x
        self.y = y
    }
    
    fn distance(&self) -> f64 {
        (self.x.pow(2) + self.y.pow(2)).sqrt()
    }
}
```

### To Ruchy (Idiomatic)
```ruchy
type Point = { x: f64, y: f64 }

extend Point {
    fn distance(&self) -> f64 {
        (self.x.pow(2) + self.y.pow(2)).sqrt()
    }
}
```

## Performance Guarantees

- Zero vtable overhead unless `dyn` trait objects used
- Methods inline by default
- No hidden allocations
- Field access compiles to direct memory access
- Inheritance via composition has no runtime cost

## Future Extensions

1. **Async Methods** - `async fn` in classes
2. **Const Methods** - `const fn` support
3. **Delegates** - Property/method delegation
4. **Mixins** - Trait-based mixins with state
5. **Sealed Classes** - Algebraic data types with methods

## Examples

### Complete Example: Shape Hierarchy

```ruchy
abstract class Shape {
    fn area(&self) -> f64
    fn perimeter(&self) -> f64
}

class Rectangle : Shape {
    width: f64
    height: f64
    
    new(width: f64, height: f64) {
        self.width = width
        self.height = height
    }
    
    override fn area(&self) -> f64 {
        self.width * self.height
    }
    
    override fn perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

class Circle : Shape {
    radius: f64
    
    new(radius: f64) {
        self.radius = radius
    }
    
    override fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
    
    override fn perimeter(&self) -> f64 {
        2.0 * PI * self.radius
    }
}

# Usage
let shapes: Vec<dyn Shape> = vec![
    Rectangle::new(10.0, 5.0),
    Circle::new(3.0),
]

for shape in shapes {
    println!("Area: {}", shape.area())
}
```

## Implementation Notes

- Parser must distinguish class blocks from struct blocks
- Type checker treats classes as nominal types
- Inheritance requires trait elaboration pass
- Method resolution follows Rust rules (no late binding)
- Generated code must pass `rustfmt` for debugging

## Open Questions

1. Should we allow `partial` classes split across files?
2. How to handle diamond inheritance problem?
3. Support for abstract classes or just traits?
4. Allow class-level attributes/decorators?
5. Support for inner/nested classes?

---

*This specification is subject to change during implementation based on technical constraints and user feedback.*