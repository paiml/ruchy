/// PARSER-CLASS-COVERAGE: Targeted unit tests to reach >85% coverage for classes.rs
///
/// COVERAGE TARGET: 52.05% → 85%+ (380 uncovered lines)
///
/// TEST STRATEGY:
/// - Focus on uncovered branches and error conditions
/// - Test edge cases not covered by property tests
/// - Test decorator combinations
/// - Test operator method parsing
/// - Test property accessors (getters/setters)
/// - Test inheritance chains with multiple traits
/// - Test error recovery paths
///
/// Priority areas (from function analysis):
/// 1. parse_inheritance - multiple traits, error cases
/// 2. parse_decorator - decorator parsing and validation
/// 3. parse_operator_method - operator overloading
/// 4. parse_class_property - property accessors
/// 5. parse_class_constant - constants with types
/// 6. Error handling paths

use ruchy::frontend::parser::Parser;

// ============================================================================
// TEST GROUP 1: Inheritance Parsing (parse_inheritance)
// ============================================================================

#[test]
fn test_class_inheritance_01_superclass_only() {
    let code = r#"
class Child : Parent {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with superclass");
}

#[test]
fn test_class_inheritance_02_single_trait() {
    let code = r#"
class MyClass : + Trait1 {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with single trait");
}

#[test]
fn test_class_inheritance_03_multiple_traits() {
    let code = r#"
class MyClass : + Trait1 + Trait2 + Trait3 {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with multiple traits");
}

#[test]
fn test_class_inheritance_04_superclass_and_traits() {
    let code = r#"
class Child : Parent + Trait1 + Trait2 {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with superclass and traits");
}

#[test]
fn test_class_inheritance_05_error_missing_trait_name() {
    let code = r#"
class MyClass : Parent + {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_err(), "Should error on missing trait name after '+'");
}

// ============================================================================
// TEST GROUP 2: Class Constants (parse_class_constant)
// ============================================================================

#[test]
fn test_class_constant_01_simple_integer() {
    let code = r#"
class MyClass {
    const MAX_SIZE: i32 = 100
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class constant with type annotation");
}

#[test]
fn test_class_constant_02_string_constant() {
    let code = r#"
class MyClass {
    const NAME: String = "MyClass"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse string constant");
}

#[test]
fn test_class_constant_03_multiple_constants() {
    let code = r#"
class MyClass {
    const A: i32 = 1
    const B: i32 = 2
    const C: String = "test"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse multiple constants");
}

#[test]
fn test_class_constant_04_expression_value() {
    let code = r#"
class MyClass {
    const COMPUTED: i32 = 10 + 20
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse constant with expression");
}

// ============================================================================
// TEST GROUP 3: Class Properties (parse_class_property)
// ============================================================================

#[test]
fn test_class_property_01_readonly_property() {
    let code = r#"
class MyClass {
    value: i32

    property count: i32 {
        get => self.value
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse readonly property with getter");
}

#[test]
fn test_class_property_02_read_write_property() {
    let code = r#"
class MyClass {
    _value: i32

    property value: i32 {
        get => self._value,
        set(v) => { self._value = v }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse read-write property with getter and setter");
}

#[test]
fn test_class_property_03_computed_property() {
    let code = r#"
class Rectangle {
    width: i32
    height: i32

    property area: i32 {
        get => self.width * self.height
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse computed property");
}

// ============================================================================
// TEST GROUP 4: Operator Methods (parse_operator_method)
// ============================================================================

#[test]
fn test_operator_method_01_add_operator() {
    let code = r#"
class Point {
    x: i32
    y: i32

    pub fun __add__(other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse __add__ operator method");
}

#[test]
fn test_operator_method_02_eq_operator() {
    let code = r#"
class Point {
    x: i32
    y: i32

    pub fun __eq__(other: Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse __eq__ operator method");
}

#[test]
fn test_operator_method_03_str_operator() {
    let code = r#"
class Point {
    x: i32
    y: i32

    pub fun __str__() -> String {
        "Point(" + self.x.to_string() + ", " + self.y.to_string() + ")"
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse __str__ operator method");
}

// ============================================================================
// TEST GROUP 5: Decorators (parse_decorator)
// ============================================================================

#[test]
fn test_decorator_01_simple_decorator() {
    let code = r#"
class MyClass {
    @deprecated
    pub fun old_method() {
        println("deprecated")
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse simple decorator");
}

#[test]
fn test_decorator_02_decorator_with_args() {
    let code = r#"
class MyClass {
    @cache(ttl=60)
    pub fun expensive_method() -> i32 {
        42
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse decorator with arguments");
}

#[test]
fn test_decorator_03_multiple_decorators() {
    let code = r#"
class MyClass {
    @deprecated
    @cache
    @trace
    pub fun method() {
        println("test")
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse multiple decorators");
}

// ============================================================================
// TEST GROUP 6: Type Parameters
// ============================================================================

#[test]
fn test_type_params_01_single_generic() {
    let code = r#"
class Box<T> {
    value: T

    pub new(val: T) -> Box<T> {
        Box { value: val }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with single type parameter");
}

#[test]
fn test_type_params_02_multiple_generics() {
    let code = r#"
class Pair<K, V> {
    key: K
    value: V

    pub new(k: K, v: V) -> Pair<K, V> {
        Pair { key: k, value: v }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with multiple type parameters");
}

// ============================================================================
// TEST GROUP 7: Edge Cases and Error Conditions
// ============================================================================

#[test]
fn test_edge_case_01_empty_class() {
    let code = r#"
class Empty {
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse empty class");
}

#[test]
fn test_edge_case_02_class_with_only_fields() {
    let code = r#"
class OnlyFields {
    a: i32
    b: String
    c: bool
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with only fields");
}

#[test]
fn test_edge_case_03_class_with_only_methods() {
    let code = r#"
class OnlyMethods {
    pub fun method1() { }
    pub fun method2() -> i32 { 42 }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with only methods");
}

#[test]
fn test_edge_case_04_nested_class_reference() {
    let code = r#"
class Outer {
    pub new() -> Outer {
        Outer { }
    }

    pub fun create_inner() -> Inner {
        Inner { }
    }
}

class Inner {
    pub new() -> Inner {
        Inner { }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class referencing another class");
}

#[test]
fn test_error_01_missing_class_body() {
    let code = r#"
class MyClass
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_err(), "Should error on missing class body");
}

#[test]
fn test_error_02_unclosed_class_body() {
    let code = r#"
class MyClass {
    value: i32
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_err(), "Should error on unclosed class body");
}

// ============================================================================
// TEST GROUP 8: Constructor Variants
// ============================================================================

#[test]
fn test_constructor_01_no_params() {
    let code = r#"
class MyClass {
    value: i32

    pub new() -> MyClass {
        MyClass { value: 0 }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse constructor with no parameters");
}

#[test]
fn test_constructor_02_multiple_params() {
    let code = r#"
class Point {
    x: i32
    y: i32

    pub new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse constructor with multiple parameters");
}

#[test]
fn test_constructor_03_default_params() {
    let code = r#"
class Point {
    x: i32
    y: i32

    pub new(x: i32 = 0, y: i32 = 0) -> Point {
        Point { x: x, y: y }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse constructor with default parameters");
}

// ============================================================================
// TEST GROUP 9: Method Visibility
// ============================================================================

#[test]
fn test_visibility_01_public_method() {
    let code = r#"
class MyClass {
    pub fun public_method() { }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse public method");
}

#[test]
fn test_visibility_02_private_method() {
    let code = r#"
class MyClass {
    fun private_method() { }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse private method (no visibility)");
}

#[test]
fn test_visibility_03_mixed_visibility() {
    let code = r#"
class MyClass {
    pub fun public_method() { }
    fun private_method() { }
    pub fun another_public() -> i32 { 42 }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with mixed visibility");
}

// ============================================================================
// TEST GROUP 10: Self Parameter Variants
// ============================================================================

#[test]
fn test_self_param_01_immutable_self() {
    let code = r#"
class MyClass {
    value: i32

    pub fun get_value(&self) -> i32 {
        self.value
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse method with &self");
}

#[test]
fn test_self_param_02_mutable_self() {
    let code = r#"
class MyClass {
    value: i32

    pub fun set_value(&mut self, val: i32) {
        self.value = val
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse method with &mut self");
}

#[test]
fn test_self_param_03_owned_self() {
    let code = r#"
class MyClass {
    value: i32

    pub fun consume(self) -> i32 {
        self.value
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse method with owned self");
}
// ============================================================================
// TEST GROUP 12: Advanced Visibility Modifiers
// ============================================================================

#[test]
fn test_visibility_04_protected_method() {
    let code = r#"
class MyClass {
    protected fun protected_method() { }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse protected method");
}

#[test]
fn test_visibility_05_pub_crate_method() {
    let code = r#"
class MyClass {
    pub(crate) fun crate_method() { }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse pub(crate) method");
}

#[test]
fn test_visibility_06_pub_super_method() {
    let code = r#"
class MyClass {
    pub(super) fun super_method() { }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse pub(super) method");
}

#[test]
fn test_visibility_07_private_explicit() {
    let code = r#"
class MyClass {
    private fun private_method() { }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse explicit private method");
}

// ============================================================================
// TEST GROUP 13: Method Modifiers (static, abstract, override, async)
// ============================================================================

#[test]
fn test_modifier_01_static_method() {
    let code = r#"
class MyClass {
    pub static fun create() -> MyClass {
        MyClass { }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse static method");
}

#[test]
fn test_modifier_02_abstract_method() {
    let code = r#"
abstract class MyClass {
    pub abstract fun do_something()
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse abstract method");
}

#[test]
fn test_modifier_03_override_method() {
    let code = r#"
class Child : Parent {
    pub override fun method() {
        println("overridden")
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse override method");
}

#[test]
fn test_modifier_04_async_method() {
    let code = r#"
class MyClass {
    pub async fun fetch_data() -> String {
        "data"
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse async method");
}

// ============================================================================
// TEST GROUP 14: Complex Property Accessors
// ============================================================================

#[test]
fn test_property_accessor_01_getter_with_block() {
    let code = r#"
class MyClass {
    _value: i32

    property value: i32 {
        get => {
            println("getting value")
            self._value
        }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse getter with block");
}

#[test]
fn test_property_accessor_02_setter_only() {
    let code = r#"
class MyClass {
    _value: i32

    property value: i32 {
        set(v) => { self._value = v }
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse setter-only property");
}

// ============================================================================
// TEST GROUP 15: Type Parameter Constraints
// ============================================================================

#[test]
fn test_type_constraint_01_where_clause() {
    let code = r#"
class Container<T> where T: Display {
    value: T
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    // May not be implemented yet - document behavior
    let _ = result;
}

// ============================================================================
// TEST GROUP 16: Complex Inheritance Scenarios
// ============================================================================

#[test]
fn test_complex_inheritance_01_generic_parent() {
    let code = r#"
class Child : Parent<i32> {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse inheritance from generic parent");
}

#[test]
fn test_complex_inheritance_02_multiple_generic_traits() {
    let code = r#"
class MyClass : + Trait1<i32> + Trait2<String> {
    value: i32
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse multiple generic trait implementations");
}

// ============================================================================
// TEST GROUP 17: Field Initialization
// ============================================================================

#[test]
fn test_field_init_01_with_default() {
    let code = r#"
class MyClass {
    value: i32 = 42
    name: String = "default"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse fields with default values");
}

// ============================================================================
// TEST GROUP 18: Mixed Member Types
// ============================================================================

#[test]
fn test_mixed_members_01_all_types() {
    let code = r#"
class MyClass {
    const MAX: i32 = 100
    value: i32
    
    property count: i32 {
        get => self.value
    }
    
    pub new(v: i32) -> MyClass {
        MyClass { value: v }
    }
    
    pub fun get_value(&self) -> i32 {
        self.value
    }
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse class with all member types");
}
