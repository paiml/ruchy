# Ruchy Class and Struct Specification (Swift Model)
Version 2.0.0

## Abstract

This specification defines the exact Swift model for structs (value types) and classes (reference types) in Ruchy, providing clear semantics that developers already understand while transpiling to idiomatic Rust code.

## Core Principle

**Structs are value types. Classes are reference types.** This fundamental distinction, borrowed directly from Swift, drives all design decisions.

## Motivation

- Adopt proven semantics from Swift that millions of developers understand
- Clear mental model: value vs reference semantics
- Performance by default with structs (stack allocation, no ARC)
- Flexibility when needed with classes (inheritance, shared state)
- Zero surprises for developers coming from Swift/modern languages

## Syntax Grammar

```ebnf
// Struct declaration (value type)
struct_decl = [visibility] "struct" ident [generic_params] "{" struct_body "}"
struct_body = (field_decl | method_impl)*

// Class declaration (reference type)
class_decl = [visibility] "class" ident [inheritance] "{" class_body "}"
class_body = (field_decl | init_decl | method_decl | deinit_decl)*

// Common elements
visibility = "pub" | "pub(crate)" | "pub(super)"
field_decl = [visibility] ident ":" type ["=" expr]

// Struct-specific
method_impl = ["mutating"] "fn" ident "(" [self_param] ["," params] ")" ["->" type] block

// Class-specific
inheritance = ":" ident
init_decl = "init" "(" params ")" block
deinit_decl = "deinit" block
method_decl = [visibility] ["override"] "fn" ident "(" [params] ")" ["->" type] block

// Self parameters
self_param = "self" | "&self" | "&mut self"  // structs
// Classes don't need explicit self param in method signature
```

## Core Semantics

### 1. Value Types (Structs)

Structs have value semantics - assignment copies the value:

```ruchy
struct Point {
    x: f64
    y: f64
}

// Automatic memberwise initializer
let p1 = Point(x: 3.0, y: 4.0)

// Assignment creates a COPY
var p2 = p1
p2.x = 5.0
assert(p1.x == 3.0)  // p1 unchanged
assert(p2.x == 5.0)  // p2 modified

// Methods that modify need 'mutating'
impl Point {
    mutating fun move_by(dx: f64, dy: f64) {
        self.x += dx
        self.y += dy
    }

    fun distance_to(other: Point) -> f64 {
        let dx = self.x - other.x
        let dy = self.y - other.y
        sqrt(dx * dx + dy * dy)
    }
}
```

### 2. Reference Types (Classes)

Classes have reference semantics - assignment shares the reference:

```ruchy
class Person {
    name: String
    age: i32

    init(name: String, age: i32) {
        self.name = name
        self.age = age
    }

    fun have_birthday() {
        self.age += 1  // Can mutate without 'mutating'
    }
}

// Assignment shares reference
let person1 = Person(name: "Alice", age: 30)
let person2 = person1  // Same object
person2.age = 31
assert(person1.age == 31)  // Both see change

// Identity comparison
assert(person1 === person2)  // Same instance
```

### 3. Initialization

#### Struct Initialization
Structs get automatic memberwise initializers:

```ruchy
struct Rectangle {
    width: f64
    height: f64
}

// Automatic memberwise init
let r1 = Rectangle(width: 10.0, height: 5.0)

// Can add custom initializers
impl Rectangle {
    static fun square(size: f64) -> Rectangle {
        Rectangle(width: size, height: size)
    }
}

let r2 = Rectangle::square(7.0)
```

#### Class Initialization
Classes require explicit `init`:

```ruchy
class BankAccount {
    owner: String
    balance: f64

    // Primary initializer
    init(owner: String, initialDeposit: f64) {
        self.owner = owner
        self.balance = initialDeposit
    }

    // Convenience initializer
    convenience init(owner: String) {
        self.init(owner: owner, initialDeposit: 0.0)
    }

    // Failable initializer
    init?(owner: String, initialDeposit: f64) {
        if initialDeposit < 0 {
            return nil
        }
        self.owner = owner
        self.balance = initialDeposit
    }
}
```

**Rules:**
- Classes must have at least one designated `init`
- Convenience initializers must call designated init
- All stored properties must be initialized
- Failable initializers return Optional

### 4. Methods

#### Struct Methods
Structs need `mutating` for methods that modify:

```ruchy
struct Counter {
    count: i32
}

impl Counter {
    // Immutable method
    fun get() -> i32 {
        self.count
    }

    // Mutating method
    mutating fun increment() {
        self.count += 1
    }

    // Static method
    static fun zero() -> Counter {
        Counter(count: 0)
    }
}

var c = Counter(count: 0)
c.increment()  // OK because c is var
let c2 = Counter(count: 5)
// c2.increment()  // ERROR: cannot mutate let
```

#### Class Methods
Classes don't need `mutating`:

```ruchy
class Counter {
    count: i32

    init(count: i32 = 0) {
        self.count = count
    }

    fun increment() {
        self.count += 1  // Always allowed
    }

    class fun zero() -> Counter {
        Counter(count: 0)
    }
}

let c = Counter()
c.increment()  // OK even though c is let
```

### 5. Inheritance (Classes Only)

Single inheritance with explicit super calls:

```ruchy
class Vehicle {
    wheels: i32

    init(wheels: i32) {
        self.wheels = wheels
    }

    fun description() -> String {
        "Vehicle with \(wheels) wheels"
    }
}

class Car : Vehicle {
    brand: String

    init(brand: String) {
        self.brand = brand
        super.init(wheels: 4)  // Must call super.init
    }

    override fun description() -> String {
        "\(brand) car with \(wheels) wheels"
    }
}
```

**Rules:**
- Single inheritance only
- Must call `super.init()` in initializer
- `override` required for overriding
- `final` prevents further overriding
- Structs do NOT support inheritance

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
    fun fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
```

## Transpilation Rules

### Struct (Value Type)

**Ruchy:**
```ruchy
struct Point {
    x: f64
    y: f64
}

impl Point {
    mutating fun move_by(dx: f64, dy: f64) {
        self.x += dx
        self.y += dy
    }

    fun distance() -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}
```

**Generated Rust:**
```rust
#[derive(Clone, Copy, Debug, PartialEq)]  // Copy if all fields are Copy
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    // Automatic memberwise init
    pub fun new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fun move_by(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }

    pub fun distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

### Class (Reference Type)

**Ruchy:**
```ruchy
class Person {
    name: String
    age: i32

    init(name: String, age: i32) {
        self.name = name
        self.age = age
    }

    fun have_birthday() {
        self.age += 1
    }

    deinit {
        println("Person {} deallocated", self.name)
    }
}
```

**Generated Rust:**
```rust
use std::rc::Rc;
use std::cell::RefCell;

struct PersonData {
    name: String,
    age: i32,
}

#[derive(Clone)]
pub struct Person(Rc<RefCell<PersonData>>);

impl Person {
    pub fun new(name: String, age: i32) -> Self {
        Person(Rc::new(RefCell::new(PersonData { name, age })))
    }

    pub fun have_birthday(&self) {
        self.0.borrow_mut().age += 1;
    }

    pub fun name(&self) -> String {
        self.0.borrow().name.clone()
    }
}

impl Drop for PersonData {
    fun drop(&mut self) {
        println!("Person {} deallocated", self.name);
    }
}

// Reference equality
impl PartialEq for Person {
    fun eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)  // Identity comparison
    }
}
```

### Inheritance

**Ruchy:**
```ruchy
class Animal {
    name: String

    init(name: String) {
        self.name = name
    }

    fun speak() -> String {
        "..."
    }
}

class Dog : Animal {
    breed: String

    init(name: String, breed: String) {
        self.breed = breed
        super.init(name: name)
    }

    override fun speak() -> String {
        "Woof!"
    }
}
```

**Generated Rust:**
```rust
trait Animal: AnimalBase {
    fun speak(&self) -> String {
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
    pub fun new(name: String, breed: String) -> Self {
        Self {
            _base: AnimalBase { name },
            breed,
        }
    }
}

impl Animal for Dog {
    fun speak(&self) -> String {
        "Woof!"
    }
}

impl AnimalBase for Dog {
    fun name(&self) -> &String {
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
    
    init(value: T) {
        self.value = value
    }
    
    fun map<U>(&self, f: impl Fn(&T) -> U) -> Box<U> {
        Box(value: f(&self.value))
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
Error: Field 'name' not initialized in init
  --> src/main.ruchy:8:5
   |
 8 |     init(age: u32) {
   |     ^^^^
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
    
    init(x: f64, y: f64) {
        self.x = x
        self.y = y
    }
    
    fun distance(&self) -> f64 {
        (self.x.pow(2) + self.y.pow(2)).sqrt()
    }
}
```

### To Ruchy (Idiomatic)
```ruchy
type Point = { x: f64, y: f64 }

extend Point {
    fun distance(&self) -> f64 {
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
    fun area(&self) -> f64
    fun perimeter(&self) -> f64
}

class Rectangle : Shape {
    width: f64
    height: f64
    
    init(width: f64, height: f64) {
        self.width = width
        self.height = height
    }
    
    override fun area(&self) -> f64 {
        self.width * self.height
    }
    
    override fun perimeter(&self) -> f64 {
        2.0 * (self.width + self.height)
    }
}

class Circle : Shape {
    radius: f64
    
    init(radius: f64) {
        self.radius = radius
    }
    
    override fun area(&self) -> f64 {
        PI * self.radius * self.radius
    }
    
    override fun perimeter(&self) -> f64 {
        2.0 * PI * self.radius
    }
}

# Usage
let shapes: Vec<dyn Shape> = vec![
    Rectangle(width: 10.0, height: 5.0),
    Circle(radius: 3.0),
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