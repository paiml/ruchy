# Sub-spec: Class Sugar — Advanced Features, Restrictions & Examples

**Parent:** [class-sugar-spec.md](../class-sugar-spec.md) Sections 7-15

---
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

