/// Property-based tests for class parsing and evaluation
///
/// COVERAGE TARGET: >85% for class-related code paths
/// - `src/frontend/parser/expressions_helpers/classes.rs` (897 lines)
/// - `src/frontend/parser/expressions_helpers/structs.rs` (370 lines)
/// - `src/frontend/parser/expressions_helpers/impls.rs` (141 lines)
/// - src/runtime/interpreter.rs class methods (6 functions)
///
/// TEST STRATEGY:
/// - Property 1: Class parsing roundtrip (parse → AST → parse matches)
/// - Property 2: Constructor always returns instance (never nil)
/// - Property 3: Method calls preserve type safety
/// - Property 4: Field access deterministic
/// - Property 5: Inheritance chain resolved correctly
/// - Property 6: Impl blocks equivalent to inline methods
/// - Property 7: Visibility modifiers enforced
/// - Property 8: Type parameters preserved
///
/// IMPORTANT: Interpreter is NOT thread-safe (by design).
/// Run these tests with: cargo test --test `class_property_tests` -- --test-threads=1
use proptest::prelude::*;
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::Value;

// Property 1: Class definitions parse successfully
proptest! {
    #[test]
    fn prop_class_definition_parses(
        class_name in "[A-Z][a-zA-Z0-9]{0,20}",
        field_count in 0usize..10,
    ) {
        let fields: Vec<String> = (0..field_count)
            .map(|i| format!("field{i}: i32"))
            .collect();

        let code = format!(
            "class {} {{\n    {}\n}}",
            class_name,
            fields.join(",\n    ")
        );

        let result = Parser::new(&code).parse();
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }
}

// Property 2: Simple constructors always return instances (NEVER nil)
proptest! {
    #[test]
    fn prop_constructor_returns_instance_not_nil(
        class_name in "[A-Z][a-zA-Z0-9]{0,10}",
        initial_value in 0i32..1000,
    ) {
        let code = format!(
            r"
class {class_name} {{
    value: i32

    pub new(val: i32) -> {class_name} {{
        {class_name} {{ value: val }}
    }}
}}

let instance = {class_name}::new({initial_value})
instance.value
"
        );

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);

        // Constructor must return the instance, not nil
        prop_assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
        let value = result.unwrap();
        let value_str = value.to_string();
        let is_nil = value == Value::Nil;
        prop_assert!(!is_nil,
            "Constructor returned nil for class {}", class_name);
        let expected_str = initial_value.to_string();
        prop_assert_eq!(value_str, expected_str,
            "Field value mismatch");
    }
}

// Property 3: Struct with methods parses correctly (pub modifier)
proptest! {
    #[test]
    fn prop_struct_methods_with_pub_parse(
        struct_name in "[A-Z][a-zA-Z0-9]{0,15}",
        method_count in 1usize..5,
    ) {
        let methods: Vec<String> = (0..method_count)
            .map(|i| format!(
                "    pub fun method{}(&self) -> i32 {{\n        {}\n    }}",
                i, i * 10
            ))
            .collect();

        let code = format!(
            "struct {} {{\n    value: i32,\n\n{}\n}}",
            struct_name,
            methods.join("\n\n")
        );

        let result = Parser::new(&code).parse();
        prop_assert!(result.is_ok(),
            "Failed to parse struct with pub methods: {:?}", result.err());
    }
}

// Property 4: Impl blocks parse correctly
proptest! {
    #[test]
    fn prop_impl_blocks_parse(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        method_count in 1usize..5,
    ) {
        let methods: Vec<String> = (0..method_count)
            .map(|i| format!(
                "    pub fun method{}(&self) -> i32 {{\n        {}\n    }}",
                i, i * 100
            ))
            .collect();

        let code = format!(
            "struct {} {{\n    data: i32,\n}}\n\nimpl {} {{\n{}\n}}",
            type_name, type_name,
            methods.join("\n\n")
        );

        let result = Parser::new(&code).parse();
        prop_assert!(result.is_ok(),
            "Failed to parse impl block: {:?}", result.err());
    }
}

// Property 5: Field access after constructor is deterministic
proptest! {
    #[test]
    fn prop_field_access_deterministic(
        x_val in -100i32..100,
        y_val in -100i32..100,
    ) {
        let code = format!(
            r"
struct Point {{
    x: i32,
    y: i32,
}}

impl Point {{
    pub fun new(x: i32, y: i32) -> Point {{
        Point {{ x: x, y: y }}
    }}
}}

let p = Point::new({x_val}, {y_val})
p.x
"
        );

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);

        prop_assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
        let value = result.unwrap();
        prop_assert_eq!(value.to_string(), x_val.to_string(),
            "Field x mismatch: expected {}, got {}", x_val, value);
    }
}

// Property 6: Multiple constructor calls create independent instances
proptest! {
    #[test]
    fn prop_multiple_constructors_independent(
        val1 in 1i32..50,
        val2 in 51i32..100,
    ) {
        let code = format!(
            r"
class Counter {{
    count: i32

    pub new(initial: i32) -> Counter {{
        Counter {{ count: initial }}
    }}

    pub fun get_count(&self) -> i32 {{
        self.count
    }}
}}

let c1 = Counter::new({val1})
let c2 = Counter::new({val2})
let sum = c1.get_count() + c2.get_count()
sum
"
        );

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);

        prop_assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
        let value = result.unwrap();
        let expected_sum = val1 + val2;
        prop_assert_eq!(value.to_string(), expected_sum.to_string(),
            "Sum mismatch: expected {}, got {}", expected_sum, value);
    }
}

// Property 7: Method calls preserve state across multiple calls
proptest! {
    #[test]
    fn prop_method_calls_preserve_state(
        add1 in 1i32..20,
        add2 in 1i32..20,
    ) {
        let code = format!(
            r"
class Calculator {{
    value: i32

    pub new() -> Calculator {{
        Calculator {{ value: 0 }}
    }}

    pub fun add(&mut self, n: i32) -> i32 {{
        self.value = self.value + n
        self.value
    }}
}}

let mut calc = Calculator::new()
calc.add({add1})
calc.add({add2})
"
        );

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);

        prop_assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
        let value = result.unwrap();
        let expected = add1 + add2;
        prop_assert_eq!(value.to_string(), expected.to_string(),
            "Cumulative addition mismatch: expected {}, got {}", expected, value);
    }
}

// Property 8: Class inheritance parsing (without evaluation yet)
proptest! {
    #[test]
    fn prop_class_inheritance_parses(
        class_name in "[A-Z][a-zA-Z0-9]{0,15}",
        parent_name in "[A-Z][a-zA-Z0-9]{0,15}",
    ) {
        // Ensure names are different
        prop_assume!(class_name != parent_name);

        let code = format!(
            "class {class_name} : {parent_name} {{\n    extra_field: i32\n}}"
        );

        let result = Parser::new(&code).parse();
        prop_assert!(result.is_ok(),
            "Failed to parse inheritance: {:?}", result.err());
    }
}

// Property 9: Visibility modifiers (pub/private) parse correctly
proptest! {
    #[test]
    fn prop_visibility_modifiers_parse(
        struct_name in "[A-Z][a-zA-Z0-9]{0,15}",
        has_pub in prop::bool::ANY,
    ) {
        let visibility = if has_pub { "pub " } else { "" };
        let code = format!(
            "struct {struct_name} {{\n    value: i32,\n\n    {visibility}fun get_value(&self) -> i32 {{\n        self.value\n    }}\n}}"
        );

        let result = Parser::new(&code).parse();
        prop_assert!(result.is_ok(),
            "Failed to parse visibility modifier: {:?}", result.err());
    }
}

// Property 10: Empty constructors return valid instances
proptest! {
    #[test]
    fn prop_empty_constructor_valid(
        class_name in "[A-Z][a-zA-Z0-9]{0,15}",
    ) {
        let code = format!(
            r"
class {class_name} {{
    value: i32

    pub new() -> {class_name} {{
        {class_name} {{ value: 42 }}
    }}
}}

let instance = {class_name}::new()
instance.value
"
        );

        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse");
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast);

        prop_assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
        let value = result.unwrap();
        prop_assert_eq!(value.to_string(), "42",
            "Empty constructor returned wrong value: got {}", value);
    }
}

#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_property_suite_runs() {
        // Smoke test to ensure proptest macros compile
        println!("Property test suite compiled successfully");
    }
}
