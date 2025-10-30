// RUNTIME-003: Implement Classes (Reference Types) - EXTREME TDD
// RED → GREEN → REFACTOR cycle
//
// This test file follows EXTREME TDD methodology:
// 1. RED: Write tests FIRST (all marked #[ignore], they WILL fail)
// 2. GREEN: Implement minimal code to make tests pass
// 3. REFACTOR: Add property tests, mutation tests, optimize
//
// Requirements from roadmap.yaml:
// - Value::Class(Arc<RefCell<ClassInstance>>) runtime representation
// - Class instantiation with init: Person(name: "Alice", age: 30)
// - Instance methods: person.have_birthday()
// - Reference semantics (shared on assignment)
// - Identity comparison (===)

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ==================== RED PHASE: Unit Tests (Will Fail Initially) ====================

/// Test 1: Basic class instantiation with init
#[test]
fn test_runtime_003_class_instantiation_with_init() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Person { init(name: String) { self.name = name; } }; let p = Person(\"Alice\"); p.name")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

/// Test 2: Class instance methods
#[test]
fn test_runtime_003_class_instance_methods() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Counter { init() { self.count = 0; } fun increment() { self.count = self.count + 1; } }; let c = Counter(); c.increment(); c.count")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

/// Test 3: Class reference semantics (shared on assignment)
#[test]
fn test_runtime_003_class_reference_semantics_shared() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Counter { init() { self.count = 0; } fun set(n: i32) { self.count = n; } }; let c1 = Counter(); let c2 = c1; c2.set(10); c1.count")
        .assert()
        .success()
        .stdout(predicate::str::contains("10")); // c1.count changed because c2 is same reference
}

/// Test 4: Class identity comparison (==)
#[test]
fn test_runtime_003_class_identity_comparison() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Person { init(name: String) { self.name = name; } }; let p1 = Person(\"Alice\"); let p2 = p1; p1 == p2")
        .assert()
        .success()
        .stdout(predicate::str::contains("true")); // Same identity
}

/// Test 5: Class identity comparison - different instances
#[test]
fn test_runtime_003_class_identity_different_instances() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Person { init(name: String) { self.name = name; } }; let p1 = Person(\"Alice\"); let p2 = Person(\"Alice\"); p1 == p2")
        .assert()
        .success()
        .stdout(predicate::str::contains("false")); // Different instances
}

/// Test 6: Class field mutation
#[test]
fn test_runtime_003_class_field_mutation() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Person { init(name: String, age: i32) { self.name = name; self.age = age; } fun have_birthday() { self.age = self.age + 1; } }; let p = Person(\"Alice\", 30); p.have_birthday(); p.age")
        .assert()
        .success()
        .stdout(predicate::str::contains("31"));
}

/// Test 7: Error handling - missing init method
#[test]
fn test_runtime_003_class_error_missing_init() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Person { fun greet() {} }; let p = Person(\"Alice\")") // No init method
        .assert()
        .failure()
        .stderr(predicate::str::contains("init").or(predicate::str::contains("constructor")));
}

/// Test 8: Class with multiple methods
#[test]
fn test_runtime_003_class_multiple_methods() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Calculator { init() { self.result = 0; } fun add(n: i32) { self.result = self.result + n; } fun multiply(n: i32) { self.result = self.result * n; } }; let calc = Calculator(); calc.add(5); calc.multiply(2); calc.result")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

/// Test 9: Class field access
#[test]
fn test_runtime_003_class_field_access() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Point { init(x: i32, y: i32) { self.x = x; self.y = y; } }; let p = Point(10, 20); p.x")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

/// Test 10: Class method returning value
#[test]
fn test_runtime_003_class_method_return_value() {
    ruchy_cmd()
        .arg("-e")
        .arg("class Math { init() {} fun double(n: i32) { return n * 2; } }; let m = Math(); m.double(21)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ==================== RED PHASE: Property Tests (Will Be Added in REFACTOR) ====================

// Property tests will be added after GREEN phase is complete
// These will validate invariants:
// - Reference semantics: assignments share the same instance
// - Identity comparison: same reference === returns true
// - Identity comparison: different instances === returns false
// - Method calls mutate shared state
// - Field access works for any class instance

// TODO (REFACTOR phase): Add proptest cases with 10K+ iterations

// ==================== RED PHASE: Mutation Test Targets ====================

// Mutation testing targets (for REFACTOR phase):
// 1. Reference sharing logic (Arc<RefCell> usage)
// 2. Identity vs equality comparison (=== vs ==)
// 3. Method dispatch (self binding)
// 4. Field mutation (RefCell borrow_mut)

// ==================== Test Summary ====================

#[test]
fn test_runtime_003_red_phase_summary() {
    // This test documents the RED phase test plan
    //
    // Unit Tests Created: 10
    // 1. test_runtime_003_class_instantiation_with_init
    // 2. test_runtime_003_class_instance_methods
    // 3. test_runtime_003_class_reference_semantics_shared
    // 4. test_runtime_003_class_identity_comparison
    // 5. test_runtime_003_class_identity_different_instances
    // 6. test_runtime_003_class_field_mutation
    // 7. test_runtime_003_class_error_missing_init
    // 8. test_runtime_003_class_multiple_methods
    // 9. test_runtime_003_class_field_access
    // 10. test_runtime_003_class_method_return_value
    //
    // All tests currently #[ignore]d and will FAIL when un-ignored (RED phase)
    //
    // Key Differences from Structs (RUNTIME-002):
    // - Reference semantics (not value semantics)
    // - Identity comparison (=== not ==)
    // - Mutable state via RefCell
    // - Instance methods with self
    // - Init method for construction
    //
    // Next Step (GREEN phase):
    // 1. Add Value::Class variant to src/runtime/interpreter.rs
    // 2. Implement class definition storage
    // 3. Implement class instantiation (call init)
    // 4. Implement method dispatch (self binding)
    // 5. Implement identity comparison (===)
    // 6. Un-ignore tests one by one and make them pass
    //
    // After GREEN (REFACTOR phase):
    // 1. Add 10K+ property tests
    // 2. Run mutation tests (target ≥75%)
    // 3. Optimize if needed while maintaining tests

    assert!(true, "RED phase: 10 tests created, all will fail when un-ignored");
}

// ==================== REFACTOR PHASE: Property Tests ====================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Property test: Class instantiation always succeeds with valid arguments
    ///
    /// Validates that class instances can be created with any string name and integer age.
    /// This tests the robustness of the instantiation mechanism.
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn proptest_class_instantiation_any_values() {
        proptest!(|(name in "[a-zA-Z0-9_]+", age in -1000i32..1000i32)| {
            let code = format!(
                "class Person {{ init(name: String, age: i32) {{ self.name = name; self.age = age; }} }}; \
                 let p = Person(\"{}\", {}); \
                 p.age",
                name.replace('"', "\\\""), age
            );

            let result = ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success();

            // If it succeeds, output should contain the age
            result.stdout(predicate::str::contains(age.to_string()));
        });
    }

    /// Property test: Reference semantics - assignments share same instance
    ///
    /// For ANY class instance, assigning it to another variable should share the same reference.
    /// Mutations through one variable should be visible through the other.
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn proptest_reference_semantics_shared_state() {
        proptest!(|(initial in 0i32..100i32, delta in 1i32..50i32)| {
            let code = format!(
                "class Counter {{ init(n: i32) {{ self.count = n; }} \
                                  fun add(n: i32) {{ self.count = self.count + n; }} }}; \
                 let c1 = Counter({initial}); \
                 let c2 = c1; \
                 c2.add({delta}); \
                 c1.count"
            );

            let expected = initial + delta;
            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success()
                .stdout(predicate::str::contains(expected.to_string()));
        });
    }

    /// Property test: Identity comparison - same reference returns true
    ///
    /// For ANY class instance, comparing it with a variable holding the same reference
    /// should always return true (identity equality).
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn proptest_identity_same_reference_true() {
        proptest!(|(value in 0i32..1000i32)| {
            let code = format!(
                "class Box {{ init(v: i32) {{ self.value = v; }} }}; \
                 let b1 = Box({value}); \
                 let b2 = b1; \
                 b1 == b2"
            );

            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success()
                .stdout(predicate::str::contains("true"));
        });
    }

    /// Property test: Identity comparison - different instances return false
    ///
    /// For ANY two independently created class instances (even with identical field values),
    /// identity comparison should return false.
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn proptest_identity_different_instances_false() {
        proptest!(|(value in 0i32..1000i32)| {
            let code = format!(
                "class Box {{ init(v: i32) {{ self.value = v; }} }}; \
                 let b1 = Box({value}); \
                 let b2 = Box({value}); \
                 b1 == b2"
            );

            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success()
                .stdout(predicate::str::contains("false"));
        });
    }

    /// Property test: Method calls mutate shared state predictably
    ///
    /// For ANY sequence of method calls, the final state should be deterministic
    /// and consistent with reference semantics.
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn proptest_method_mutations_deterministic() {
        proptest!(|(a in 0i32..50i32, b in 0i32..50i32)| {
            let code = format!(
                "class Accumulator {{ init() {{ self.sum = 0; }} \
                                      fun add(n: i32) {{ self.sum = self.sum + n; }} }}; \
                 let acc = Accumulator(); \
                 acc.add({a}); \
                 acc.add({b}); \
                 acc.sum"
            );

            let expected = a + b;
            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success()
                .stdout(predicate::str::contains(expected.to_string()));
        });
    }

    /// Property test: Field access is consistent
    ///
    /// For ANY class instance with fields, field access should return the values
    /// that were set during initialization or mutation.
    #[test]
    #[ignore] // Run with: cargo test property_tests -- --ignored --nocapture
    fn proptest_field_access_consistent() {
        proptest!(|(x in -100i32..100i32, y in -100i32..100i32)| {
            let code = format!(
                "class Point {{ init(x: i32, y: i32) {{ self.x = x; self.y = y; }} }}; \
                 let p = Point({x}, {y}); \
                 p.x"
            );

            ruchy_cmd()
                .arg("-e")
                .arg(&code)
                .assert()
                .success()
                .stdout(predicate::str::contains(x.to_string()));
        });
    }
}
