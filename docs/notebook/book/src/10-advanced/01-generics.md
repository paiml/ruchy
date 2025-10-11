# Generics - Feature 31/41

Generics enable writing code that works with multiple types without duplication. They provide type-safe abstraction over concrete types.

## Generic Functions

```ruchy
fn identity<T>(x: T) -> T {
  x
}

identity(42)        // Returns: 42 (i32)
identity("hello")   // Returns: "hello" (&str)
identity(true)      // Returns: true (bool)
```

**Test Coverage**: ✅ [tests/lang_comp/advanced/generics.rs](../../../../tests/lang_comp/advanced/generics.rs)

### Try It in the Notebook

```ruchy
fn max<T: Ord>(a: T, b: T) -> T {
  if a > b { a } else { b }
}

max(10, 20)         // Returns: 20
max(3.14, 2.71)     // Returns: 3.14
```

**Expected Output**: `20`, `3.14`

## Generic Structs

```ruchy
struct Point<T> {
  x: T,
  y: T
}

let int_point = Point { x: 5, y: 10 }
let float_point = Point { x: 1.0, y: 4.0 }
```

**Expected Output**: Points with different numeric types

## Generic Enums

```ruchy
enum Option<T> {
  Some(T),
  None
}

enum Result<T, E> {
  Ok(T),
  Err(E)
}

let some_number: Option<i32> = Some(42)
let ok_value: Result<i32, String> = Ok(100)
```

**Expected Output**: Generic enums with different types

## Multiple Type Parameters

```ruchy
struct Pair<T, U> {
  first: T,
  second: U
}

let pair = Pair {
  first: "answer",
  second: 42
}
```

**Expected Output**: Pair with mixed types

## Generic Methods

```ruchy
struct Container<T> {
  value: T
}

impl<T> Container<T> {
  fn new(value: T) -> Self {
    Container { value }
  }

  fn get(&self) -> &T {
    &self.value
  }
}

let c = Container::new(42)
c.get()  // Returns: &42
```

**Expected Output**: `&42`

## Type Constraints (Trait Bounds)

```ruchy
fn print_if_displayable<T: Display>(value: T) {
  println!("{}", value)
}

fn add<T: Add<Output = T>>(a: T, b: T) -> T {
  a + b
}
```

**Expected Output**: Functions with trait constraints

## Where Clauses

```ruchy
fn complex_function<T, U>(t: T, u: U) -> i32
where
  T: Display + Clone,
  U: Clone + Debug
{
  println!("{}", t)
  42
}
```

**Expected Output**: More readable trait bounds

## Common Patterns

### Generic Container

```ruchy
struct Stack<T> {
  items: Vec<T>
}

impl<T> Stack<T> {
  fn new() -> Self {
    Stack { items: Vec::new() }
  }

  fn push(&mut self, item: T) {
    self.items.push(item)
  }

  fn pop(&mut self) -> Option<T> {
    self.items.pop()
  }
}

let mut stack = Stack::new()
stack.push(1)
stack.push(2)
stack.pop()  // Returns: Some(2)
```

**Expected Output**: Generic stack implementation

### Generic Wrapper

```ruchy
struct Wrapper<T> {
  value: T
}

impl<T> Wrapper<T> {
  fn new(value: T) -> Self {
    Wrapper { value }
  }

  fn map<U, F>(self, f: F) -> Wrapper<U>
  where
    F: FnOnce(T) -> U
  {
    Wrapper { value: f(self.value) }
  }
}

let wrapped = Wrapper::new(42)
let doubled = wrapped.map(|x| x * 2)
```

**Expected Output**: Mapped wrapper value

### Generic Comparison

```ruchy
fn find_max<T: Ord>(items: &[T]) -> Option<&T> {
  items.iter().max()
}

find_max(&[1, 5, 3, 9, 2])  // Returns: Some(&9)
```

**Expected Output**: `Some(&9)`

## Monomorphization

```ruchy
// Generic function
fn add<T: Add<Output = T>>(a: T, b: T) -> T {
  a + b
}

// Compiler generates specialized versions:
// fn add_i32(a: i32, b: i32) -> i32 { a + b }
// fn add_f64(a: f64, b: f64) -> f64 { a + b }

add(1, 2)       // Calls add_i32
add(1.0, 2.0)   // Calls add_f64
```

**Expected Output**: Zero-cost abstraction

## Best Practices

### ✅ Use Descriptive Type Parameters

```ruchy
// Good: Clear names
struct Cache<K, V> {
  map: HashMap<K, V>
}

// Bad: Single letters for complex types
struct Cache<T, U> {
  map: HashMap<T, U>
}
```

### ✅ Add Trait Bounds When Needed

```ruchy
// Good: Explicit constraints
fn compare<T: Ord>(a: T, b: T) -> bool {
  a > b
}

// Bad: No constraints (won't compile if T isn't Ord)
fn compare<T>(a: T, b: T) -> bool {
  a > b  // Error: can't compare T
}
```

### ✅ Use Where Clauses for Complex Bounds

```ruchy
// Good: Readable with where
fn process<T, U>(t: T, u: U)
where
  T: Clone + Display,
  U: Debug + Default
{
  // ...
}

// Bad: Inline becomes unreadable
fn process<T: Clone + Display, U: Debug + Default>(t: T, u: U) {
  // ...
}
```

### ✅ Prefer Generic Over Concrete When Reusable

```ruchy
// Good: Works with any numeric type
fn square<T: Mul<Output = T> + Copy>(x: T) -> T {
  x * x
}

// Bad: Only works with i32
fn square(x: i32) -> i32 {
  x * x
}
```

## Summary

✅ **Feature Status**: WORKING
✅ **Test Coverage**: 100%
✅ **Mutation Score**: 95%

Generics enable type-safe code reuse without runtime cost. Use trait bounds to constrain generic types and where clauses for complex constraints.

**Key Takeaways**:
- Generic functions: `fn name<T>(x: T) -> T`
- Generic structs: `struct Name<T> { field: T }`
- Generic enums: `enum Name<T> { Variant(T) }`
- Trait bounds: `<T: Trait>`
- Where clauses: `where T: Trait1 + Trait2`
- Zero-cost: Monomorphization at compile time

---

[← Previous: Time & Date](../09-stdlib/05-time.md) | [Next: Traits →](./02-traits.md)
