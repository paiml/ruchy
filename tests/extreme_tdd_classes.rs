//! Extreme TDD tests for class functionality
//! These tests are written BEFORE implementation to drive development
//! Target: 100% class feature coverage

use ruchy::compile;

// ==================== PROPERTIES WITH GETTERS/SETTERS ====================

#[test]
fn test_class_property_basic() {
    let code = r#"
        class Temperature {
            celsius: f64,

            property fahrenheit: f64 {
                get => self.celsius * 9.0/5.0 + 32.0,
                set(value) => self.celsius = (value - 32.0) * 5.0/9.0
            }

            new(c: f64) {
                Temperature { celsius: c }
            }
        }

        fn main() {
            let mut t = Temperature::new(0.0)
            println(t.fahrenheit)  // Should print 32.0
            t.fahrenheit = 212.0
            println(t.celsius)     // Should print 100.0
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Class properties with getters/setters should work");
}

#[test]
fn test_class_readonly_property() {
    let code = r#"
        class Circle {
            radius: f64,

            property area: f64 {
                get => 3.14159 * self.radius * self.radius
            }

            property circumference: f64 {
                get => 2.0 * 3.14159 * self.radius
            }

            new(r: f64) {
                Circle { radius: r }
            }
        }

        fn main() {
            let c = Circle::new(5.0)
            println(c.area)
            println(c.circumference)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Readonly properties should work");
}

#[test]
fn test_class_property_validation() {
    let code = r#"
        class Age {
            value: i32,

            property years: i32 {
                get => self.value,
                set(v) => {
                    if v >= 0 && v <= 150 {
                        self.value = v
                    } else {
                        panic("Invalid age")
                    }
                }
            }

            new(initial: i32) {
                Age { value: initial }
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Property setters with validation should work");
}

// ==================== STATIC METHODS AND CONSTANTS ====================

#[test]
fn test_class_static_methods() {
    let code = r#"
        class Math {
            static fn square(x: f64) -> f64 {
                x * x
            }

            static fn cube(x: f64) -> f64 {
                x * x * x
            }

            static fn power(base: f64, exp: i32) -> f64 {
                let mut result = 1.0
                for i in 0..exp {
                    result *= base
                }
                result
            }
        }

        fn main() {
            println(Math::square(5.0))
            println(Math::cube(3.0))
            println(Math::power(2.0, 10))
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Static methods should work");
}

#[test]
fn test_class_constants() {
    let code = r#"
        class Physics {
            const SPEED_OF_LIGHT: f64 = 299792458.0
            const PLANCK_CONSTANT: f64 = 6.626e-34
            const GRAVITY: f64 = 9.81

            static fn kinetic_energy(mass: f64, velocity: f64) -> f64 {
                0.5 * mass * velocity * velocity
            }

            static fn relativistic_mass(rest_mass: f64, velocity: f64) -> f64 {
                let ratio = velocity / Physics::SPEED_OF_LIGHT
                rest_mass / sqrt(1.0 - ratio * ratio)
            }
        }

        fn main() {
            println(Physics::GRAVITY)
            println(Physics::kinetic_energy(10.0, 5.0))
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Class constants should work");
}

#[test]
fn test_class_static_factory_methods() {
    let code = r#"
        class Point {
            x: f64,
            y: f64,

            new(x: f64, y: f64) {
                Point { x: x, y: y }
            }

            static fn origin() -> Point {
                Point::new(0.0, 0.0)
            }

            static fn from_polar(r: f64, theta: f64) -> Point {
                Point::new(r * cos(theta), r * sin(theta))
            }
        }

        fn main() {
            let p1 = Point::origin()
            let p2 = Point::from_polar(5.0, 3.14159/4.0)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Static factory methods should work");
}

// ==================== GENERIC METHODS ====================

#[test]
fn test_class_generic_methods() {
    let code = r#"
        class Container<T> {
            items: Vec<T>,

            new() {
                Container { items: vec![] }
            }

            fn add<U: Into<T>>(mut self, item: U) {
                self.items.push(item.into())
            }

            fn get(self, index: usize) -> Option<T> {
                if index < self.items.len() {
                    Some(self.items[index])
                } else {
                    None
                }
            }

            fn map<U, F>(self, f: F) -> Container<U>
            where F: Fn(T) -> U {
                Container {
                    items: self.items.iter().map(f).collect()
                }
            }
        }

        fn main() {
            let mut c = Container::<i32>::new()
            c.add(42)
            c.add(100)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Generic methods should work");
}

#[test]
fn test_class_associated_types() {
    let code = r#"
        class Iterator<T> {
            data: Vec<T>,
            index: usize,

            type Item = T

            new(data: Vec<T>) {
                Iterator { data: data, index: 0 }
            }

            fn next(mut self) -> Option<Self::Item> {
                if self.index < self.data.len() {
                    let item = self.data[self.index]
                    self.index += 1
                    Some(item)
                } else {
                    None
                }
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Associated types should work");
}

// ==================== TRAIT IMPLEMENTATION ====================

#[test]
fn test_class_impl_display() {
    let code = r#"
        class Point {
            x: f64,
            y: f64,

            impl Display {
                fn fmt(self, f: Formatter) -> Result {
                    write!(f, "({}, {})", self.x, self.y)
                }
            }

            new(x: f64, y: f64) {
                Point { x: x, y: y }
            }
        }

        fn main() {
            let p = Point::new(3.0, 4.0)
            println(f"{p}")  // Should use Display impl
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Implementing Display trait should work");
}

#[test]
fn test_class_impl_multiple_traits() {
    let code = r#"
        class Value {
            data: i32,

            impl Clone {
                fn clone(self) -> Value {
                    Value { data: self.data }
                }
            }

            impl PartialEq {
                fn eq(self, other: Value) -> bool {
                    self.data == other.data
                }
            }

            impl Debug {
                fn fmt(self, f: Formatter) -> Result {
                    write!(f, "Value({})", self.data)
                }
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Implementing multiple traits should work");
}

#[test]
fn test_class_custom_trait_impl() {
    let code = r#"
        trait Drawable {
            fn draw(self)
        }

        class Rectangle {
            width: f64,
            height: f64,

            impl Drawable {
                fn draw(self) {
                    for y in 0..self.height as i32 {
                        for x in 0..self.width as i32 {
                            print("*")
                        }
                        println("")
                    }
                }
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Implementing custom traits should work");
}

// ==================== INHERITANCE AND POLYMORPHISM ====================

#[test]
fn test_class_inheritance_with_super() {
    let code = r#"
        class Animal {
            name: String,

            fn speak(self) -> String {
                f"{self.name} makes a sound"
            }

            new(name: String) {
                Animal { name: name }
            }
        }

        class Dog : Animal {
            breed: String,

            fn speak(self) -> String {
                f"{super.speak()} - Woof!"
            }

            new(name: String, breed: String) {
                super(name)
                Dog { breed: breed }
            }
        }

        fn main() {
            let dog = Dog::new("Rex", "Labrador")
            println(dog.speak())
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Inheritance with super calls should work");
}

#[test]
fn test_class_abstract_methods() {
    let code = r#"
        abstract class Shape {
            abstract fn area(self) -> f64
            abstract fn perimeter(self) -> f64

            fn describe(self) -> String {
                f"Area: {self.area()}, Perimeter: {self.perimeter()}"
            }
        }

        class Circle : Shape {
            radius: f64,

            fn area(self) -> f64 {
                3.14159 * self.radius * self.radius
            }

            fn perimeter(self) -> f64 {
                2.0 * 3.14159 * self.radius
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Abstract classes and methods should work");
}

#[test]
fn test_class_interface_implementation() {
    let code = r#"
        interface Flyable {
            fn fly(self)
            fn altitude(self) -> f64
        }

        interface Swimmable {
            fn swim(self)
            fn depth(self) -> f64
        }

        class Duck : Flyable + Swimmable {
            fn fly(self) {
                println("Duck is flying")
            }

            fn altitude(self) -> f64 {
                100.0
            }

            fn swim(self) {
                println("Duck is swimming")
            }

            fn depth(self) -> f64 {
                2.0
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Interface implementation should work");
}

// ==================== MULTIPLE CONSTRUCTORS ====================

#[test]
fn test_class_multiple_constructors() {
    let code = r#"
        class Rectangle {
            width: f64,
            height: f64,

            new(w: f64, h: f64) {
                Rectangle { width: w, height: h }
            }

            new square(size: f64) {
                Rectangle { width: size, height: size }
            }

            new from_area(area: f64, aspect_ratio: f64) {
                let width = sqrt(area * aspect_ratio)
                let height = area / width
                Rectangle { width: width, height: height }
            }
        }

        fn main() {
            let r1 = Rectangle::new(10.0, 20.0)
            let r2 = Rectangle::square(15.0)
            let r3 = Rectangle::from_area(100.0, 1.5)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Multiple named constructors should work");
}

#[test]
fn test_class_constructor_with_defaults() {
    let code = r#"
        class Config {
            host: String,
            port: i32,
            timeout: i32,

            new(host: String = "localhost", port: i32 = 8080, timeout: i32 = 30) {
                Config { host: host, port: port, timeout: timeout }
            }
        }

        fn main() {
            let c1 = Config::new()
            let c2 = Config::new("example.com")
            let c3 = Config::new("example.com", 443)
            let c4 = Config::new("example.com", 443, 60)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Constructor with default parameters should work");
}

// ==================== VISIBILITY MODIFIERS ====================

#[test]
fn test_class_member_visibility() {
    let code = r#"
        pub class BankAccount {
            pub owner: String,
            pub(crate) account_number: String,
            private balance: f64,
            protected interest_rate: f64,

            pub fn deposit(mut self, amount: f64) {
                self.balance += amount
            }

            private fn calculate_interest(self) -> f64 {
                self.balance * self.interest_rate
            }

            protected fn apply_interest(mut self) {
                self.balance += self.calculate_interest()
            }

            pub fn get_balance(self) -> f64 {
                self.balance
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Member visibility modifiers should work");
}

#[test]
fn test_class_method_visibility_inheritance() {
    let code = r#"
        class Base {
            protected fn helper(self) -> i32 {
                42
            }

            pub fn public_method(self) -> i32 {
                self.helper()
            }
        }

        class Derived : Base {
            pub fn use_helper(self) -> i32 {
                self.helper() * 2  // Can access protected method
            }
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Protected methods accessible in derived classes");
}

// ==================== OPERATOR OVERLOADING ====================

#[test]
fn test_class_operator_overloading() {
    let code = r#"
        class Vector {
            x: f64,
            y: f64,

            operator+(self, other: Vector) -> Vector {
                Vector { x: self.x + other.x, y: self.y + other.y }
            }

            operator*(self, scalar: f64) -> Vector {
                Vector { x: self.x * scalar, y: self.y * scalar }
            }

            operator==(self, other: Vector) -> bool {
                self.x == other.x && self.y == other.y
            }
        }

        fn main() {
            let v1 = Vector { x: 1.0, y: 2.0 }
            let v2 = Vector { x: 3.0, y: 4.0 }
            let v3 = v1 + v2
            let v4 = v1 * 2.0
            assert(v3 == Vector { x: 4.0, y: 6.0 })
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Operator overloading should work");
}

// ==================== INNER CLASSES ====================

#[test]
fn test_inner_classes() {
    let code = r#"
        class Outer {
            value: i32,

            class Inner {
                inner_value: i32,

                fn access_outer(self, outer: Outer) -> i32 {
                    self.inner_value + outer.value
                }
            }

            fn create_inner(self, val: i32) -> Inner {
                Inner { inner_value: val }
            }
        }

        fn main() {
            let outer = Outer { value: 10 }
            let inner = outer.create_inner(20)
            println(inner.access_outer(outer))
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Inner classes should work");
}

// ==================== SEALED AND FINAL CLASSES ====================

#[test]
fn test_sealed_class() {
    let code = r#"
        sealed class Option<T> {
            class Some(value: T)
            class None
        }

        fn main() {
            let opt1: Option<i32> = Option::Some(42)
            let opt2: Option<i32> = Option::None
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Sealed classes should work");
}

#[test]
fn test_final_class() {
    let code = r#"
        final class SecurityManager {
            fn validate(self, token: String) -> bool {
                token.len() > 10
            }
        }

        // This should fail at compile time:
        // class ExtendedManager : SecurityManager { }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Final classes should work");
}

// ==================== MIXINS AND TRAITS ====================

#[test]
fn test_class_with_mixins() {
    let code = r#"
        mixin Timestamped {
            created_at: DateTime,
            updated_at: DateTime,

            fn touch(mut self) {
                self.updated_at = DateTime::now()
            }
        }

        mixin Versioned {
            version: i32,

            fn increment_version(mut self) {
                self.version += 1
            }
        }

        class Document with Timestamped, Versioned {
            title: String,
            content: String
        }

        fn main() {
            let mut doc = Document {
                title: "Test",
                content: "Content",
                created_at: DateTime::now(),
                updated_at: DateTime::now(),
                version: 1
            }
            doc.touch()
            doc.increment_version()
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Mixins should work");
}

// ==================== DECORATORS/ATTRIBUTES ====================

#[test]
fn test_class_decorators() {
    let code = r#"
        @Serializable
        @Table("users")
        class User {
            @PrimaryKey
            @Column("user_id")
            id: i32,

            @Column("username")
            @Unique
            name: String,

            @Transient
            temp_data: String
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Class and field decorators should work");
}

// ==================== ASYNC METHODS ====================

#[test]
fn test_class_async_methods() {
    let code = r#"
        class HttpClient {
            base_url: String,

            async fn get(self, path: String) -> Result<String> {
                // Simulated async operation
                await fetch(f"{self.base_url}{path}")
            }

            async fn post(self, path: String, body: String) -> Result<String> {
                await fetch_with_body(f"{self.base_url}{path}", body)
            }
        }

        async fn main() {
            let client = HttpClient { base_url: "https://api.example.com" }
            let result = await client.get("/users")
            println(result)
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok(), "Async methods in classes should work");
}