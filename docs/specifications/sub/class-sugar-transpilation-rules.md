# Sub-spec: Class Sugar — Transpilation Rules

**Parent:** [class-sugar-spec.md](../class-sugar-spec.md) Section 6

---
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

