# Sub-spec: Class Sugar — Core Semantics

**Parent:** [class-sugar-spec.md](../class-sugar-spec.md) Section 5

---
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

