# Traits - Feature 32/41

Traits define shared behavior across types. They're similar to interfaces in other languages but more powerful with default implementations and associated types.

## Defining Traits

```ruchy
trait Drawable {
  fn draw(&self)
}

struct Circle { radius: f64 }

impl Drawable for Circle {
  fn draw(&self) {
    println!("Drawing circle with radius {}", self.radius)
  }
}
```

**Test Coverage**: ✅ [tests/lang_comp/methods.rs](../../../../../tests/lang_comp/methods.rs)

### Try It in the Notebook

```ruchy
trait Describable {
  fn describe(&self) -> String
}

impl Describable for i32 {
  fn describe(&self) -> String {
    format!("Number: {}", self)
  }
}

42.describe()  // Returns: "Number: 42"
```

**Expected Output**: `"Number: 42"`

## Default Implementations

```ruchy
trait Greet {
  fn greet(&self) -> String {
    "Hello!".to_string()  // Default
  }
}

struct Person { name: String }

impl Greet for Person {
  fn greet(&self) -> String {
    format!("Hello, {}!", self.name)
  }
}
```

**Expected Output**: Custom or default greeting

## Trait Bounds

```ruchy
fn print_it<T: Display>(item: T) {
  println!("{}", item)
}

fn compare<T: PartialOrd>(a: T, b: T) -> bool {
  a > b
}
```

**Expected Output**: Functions constrained by traits

## Multiple Traits

```ruchy
fn process<T>(item: T)
where
  T: Display + Clone + Debug
{
  println!("{}", item)
  let cloned = item.clone()
  println!("{:?}", cloned)
}
```

**Expected Output**: Multi-trait bounds

## Associated Types

```ruchy
trait Container {
  type Item

  fn get(&self) -> &Self::Item
}

struct Box<T> {
  value: T
}

impl<T> Container for Box<T> {
  type Item = T

  fn get(&self) -> &T {
    &self.value
  }
}
```

**Expected Output**: Type-associated containers

## Common Standard Traits

### Display & Debug

```ruchy
use std::fmt

impl Display for Person {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Person: {}", self.name)
  }
}

impl Debug for Person {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Person {{ name: {:?} }}", self.name)
  }
}
```

**Expected Output**: Formatted output

### Clone & Copy

```ruchy
#[derive(Clone)]
struct Data {
  value: i32
}

let d1 = Data { value: 42 }
let d2 = d1.clone()
```

**Expected Output**: Cloned data

### Eq & PartialEq

```ruchy
#[derive(PartialEq, Eq)]
struct Point {
  x: i32,
  y: i32
}

let p1 = Point { x: 1, y: 2 }
let p2 = Point { x: 1, y: 2 }
p1 == p2  // Returns: true
```

**Expected Output**: `true`

## Trait Objects

```ruchy
trait Animal {
  fn sound(&self) -> String
}

struct Dog;
struct Cat;

impl Animal for Dog {
  fn sound(&self) -> String { "Woof!".to_string() }
}

impl Animal for Cat {
  fn sound(&self) -> String { "Meow!".to_string() }
}

let animals: Vec<Box<dyn Animal>> = vec![
  Box::new(Dog),
  Box::new(Cat)
]

for animal in animals {
  println!("{}", animal.sound())
}
```

**Expected Output**: "Woof!", "Meow!"

## Supertraits

```ruchy
trait Printable: Display {
  fn print(&self) {
    println!("{}", self)
  }
}
```

**Expected Output**: Trait requiring Display

## Operator Overloading

```ruchy
use std::ops::Add

struct Point { x: i32, y: i32 }

impl Add for Point {
  type Output = Point

  fn add(self, other: Point) -> Point {
    Point {
      x: self.x + other.x,
      y: self.y + other.y
    }
  }
}

let p1 = Point { x: 1, y: 2 }
let p2 = Point { x: 3, y: 4 }
let p3 = p1 + p2  // Point { x: 4, y: 6 }
```

**Expected Output**: Point { x: 4, y: 6 }

## Best Practices

### ✅ Use Traits for Shared Behavior

```ruchy
// Good: Common interface
trait Serializable {
  fn to_json(&self) -> String
}

// Bad: Separate methods per type
impl User {
  fn user_to_json(&self) -> String { ... }
}
impl Product {
  fn product_to_json(&self) -> String { ... }
}
```

### ✅ Prefer Trait Bounds Over Concrete Types

```ruchy
// Good: Works with any displayable type
fn log<T: Display>(msg: T) {
  println!("{}", msg)
}

// Bad: Only works with String
fn log(msg: String) {
  println!("{}", msg)
}
```

### ✅ Use Derive for Common Traits

```ruchy
// Good: Automatic implementation
#[derive(Debug, Clone, PartialEq)]
struct Data {
  value: i32
}

// Bad: Manual implementation
impl Debug for Data { ... }
impl Clone for Data { ... }
impl PartialEq for Data { ... }
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 96%

Traits define shared behavior and enable polymorphism. Use trait bounds for generic functions and trait objects for runtime polymorphism.

**Key Takeaways**:
- Define behavior: `trait Name { fn method(&self) }`
- Implement: `impl Trait for Type`
- Bounds: `<T: Trait>` or `where T: Trait`
- Objects: `Box<dyn Trait>`
- Standard traits: Clone, Debug, Display, PartialEq
- Derive: `#[derive(Trait)]`

---

[← Previous: Generics](./01-generics.md) | [Next: Lifetimes →](./03-lifetimes.md)
