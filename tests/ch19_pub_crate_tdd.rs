// TDD Tests for Chapter 19: Field Visibility (CH19-002)
//
// Requirements:
// 1. Fields default to private (no visibility modifier)
// 2. pub fields are publicly accessible
// 3. pub(crate) fields are crate-visible
// 4. Private fields cannot be accessed outside struct methods
// 5. Error messages indicate visibility violations

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_public_field_accessible() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define struct with public field
    repl.eval(
        r#"
struct User {
    pub name: String
}
"#,
    )
    .unwrap();

    // Create instance
    repl.eval(r#"let u = User { name: "Alice" }"#).unwrap();

    // Access public field
    let result = repl.eval("u.name").unwrap();
    assert!(
        result.contains("Alice"),
        "Expected 'Alice' but got: {}",
        result
    );
}

#[test]
fn test_private_field_not_accessible() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define struct with private field (no modifier)
    repl.eval(
        r#"
struct BankAccount {
    pub owner: String,
    balance: f64
}
"#,
    )
    .unwrap();

    // Create instance
    repl.eval(r#"let account = BankAccount { owner: "Bob", balance: 100.0 }"#)
        .unwrap();

    // Access public field works
    let result = repl.eval("account.owner").unwrap();
    assert!(result.contains("Bob"), "Expected 'Bob' but got: {}", result);

    // Access private field should fail
    let result = repl.eval("account.balance");
    match result {
        Err(_) => {} // Expected - field is private
        Ok(msg) => {
            assert!(
                msg.contains("private") || msg.contains("not accessible"),
                "Expected error accessing private field but got: {}",
                msg
            );
        }
    }
}

#[test]
fn test_pub_crate_field_accessible() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define struct with pub(crate) field
    repl.eval(
        r#"
struct Config {
    pub name: String,
    pub(crate) id: i32,
    secret: String
}
"#,
    )
    .unwrap();

    // Create instance
    repl.eval(r#"let cfg = Config { name: "prod", id: 42, secret: "xyz" }"#)
        .unwrap();

    // Access pub field
    let result = repl.eval("cfg.name").unwrap();
    assert!(result.contains("prod"), "Expected 'prod' but got: {}", result);

    // Access pub(crate) field - should work in REPL context
    let result = repl.eval("cfg.id").unwrap();
    assert!(result.contains("42"), "Expected 42 but got: {}", result);

    // Access private field should fail
    let result = repl.eval("cfg.secret");
    match result {
        Err(_) => {} // Expected - field is private
        Ok(msg) => {
            assert!(
                msg.contains("private") || msg.contains("not accessible"),
                "Expected error accessing private field but got: {}",
                msg
            );
        }
    }
}

#[test]
fn test_mixed_visibility_fields() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval(
        r#"
struct Person {
    pub name: String,
    pub(crate) age: i32,
    ssn: String
}
"#,
    )
    .unwrap();

    repl.eval(r#"let p = Person { name: "Charlie", age: 30, ssn: "123-45-6789" }"#)
        .unwrap();

    // Public field accessible
    let name = repl.eval("p.name").unwrap();
    assert!(
        name.contains("Charlie"),
        "Expected 'Charlie' but got: {}",
        name
    );

    // Crate-visible field accessible in REPL
    let age = repl.eval("p.age").unwrap();
    assert!(age.contains("30"), "Expected 30 but got: {}", age);
}

#[test]
fn test_default_visibility_is_private() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Fields without modifier should be private
    repl.eval(
        r#"
struct Secret {
    data: String
}
"#,
    )
    .unwrap();

    repl.eval(r#"let s = Secret { data: "hidden" }"#).unwrap();

    // Should not be accessible
    let result = repl.eval("s.data");
    match result {
        Err(_) => {} // Expected - field is private by default
        Ok(msg) => {
            assert!(
                msg.contains("private") || msg.contains("not accessible"),
                "Expected error accessing private field by default but got: {}",
                msg
            );
        }
    }
}

#[test]
fn test_visibility_with_defaults() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Combine visibility with default values
    repl.eval(
        r#"
struct Settings {
    pub theme: String = "dark",
    pub(crate) version: i32 = 1,
    internal_id: String = "auto"
}
"#,
    )
    .unwrap();

    // Create with all defaults
    repl.eval("let s = Settings {}").unwrap();

    // Access public field with default
    let theme = repl.eval("s.theme").unwrap();
    assert!(theme.contains("dark"), "Expected 'dark' but got: {}", theme);

    // Access pub(crate) field with default
    let version = repl.eval("s.version").unwrap();
    assert!(
        version.contains("1"),
        "Expected 1 but got: {}",
        version
    );

    // Private field with default should still be private
    let result = repl.eval("s.internal_id");
    match result {
        Err(_) => {} // Expected - field is private even with default
        Ok(msg) => {
            assert!(
                msg.contains("private") || msg.contains("not accessible"),
                "Expected error accessing private field even with default but got: {}",
                msg
            );
        }
    }
}
