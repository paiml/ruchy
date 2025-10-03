// TDD Tests for Chapter 19: Default Field Values (CH19-001)
//
// Requirements:
// 1. Struct fields can have default values: field: Type = default_value
// 2. Default values used when field not specified in initialization
// 3. Can override defaults by specifying field in initializer
// 4. Empty struct initializer {} uses all defaults
// 5. Mix of defaulted and non-defaulted fields

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_struct_with_all_defaults() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Define struct with default values (public fields for testing)
    repl.eval(
        r#"
struct Settings {
    pub theme: String = "dark",
    pub font_size: i32 = 14,
    pub auto_save: bool = true
}
"#,
    )
    .unwrap();

    // Create instance with all defaults
    let result = repl.eval("let s = Settings {}").unwrap();
    let _ = result;

    // Check default values
    let theme = repl.eval("s.theme").unwrap();
    assert!(theme.contains("dark"), "Expected 'dark' but got: {theme}");

    let font_size = repl.eval("s.font_size").unwrap();
    assert!(font_size.contains("14"), "Expected 14 but got: {font_size}");

    let auto_save = repl.eval("s.auto_save").unwrap();
    assert!(
        auto_save.contains("true"),
        "Expected true but got: {auto_save}"
    );
}

#[test]
fn test_struct_override_one_default() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval(
        r#"
struct Settings {
    pub theme: String = "dark",
    pub font_size: i32 = 14,
    pub auto_save: bool = true
}
"#,
    )
    .unwrap();

    // Override one field
    let result = repl.eval("let s = Settings { font_size: 16 }").unwrap();
    let _ = result;

    // Check overridden value
    let font_size = repl.eval("s.font_size").unwrap();
    assert!(font_size.contains("16"), "Expected 16 but got: {font_size}");

    // Check other defaults still used
    let theme = repl.eval("s.theme").unwrap();
    assert!(theme.contains("dark"), "Expected 'dark' but got: {theme}");
}

#[test]
fn test_struct_override_all_defaults() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval(
        r#"
struct Settings {
    pub theme: String = "dark",
    pub font_size: i32 = 14,
    pub auto_save: bool = true
}
"#,
    )
    .unwrap();

    // Override all fields
    let result = repl
        .eval(r#"let s = Settings { theme: "light", font_size: 12, auto_save: false }"#)
        .unwrap();
    let _ = result;

    // Check all overrides
    let theme = repl.eval("s.theme").unwrap();
    assert!(theme.contains("light"), "Expected 'light' but got: {theme}");

    let font_size = repl.eval("s.font_size").unwrap();
    assert!(font_size.contains("12"), "Expected 12 but got: {font_size}");

    let auto_save = repl.eval("s.auto_save").unwrap();
    assert!(
        auto_save.contains("false"),
        "Expected false but got: {auto_save}"
    );
}

#[test]
fn test_struct_mixed_defaults_and_required() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Mix of fields with and without defaults (all public for testing)
    repl.eval(
        r#"
struct User {
    pub name: String,           // Required (no default)
    pub role: String = "user",  // Optional (has default)
    pub active: bool = true     // Optional (has default)
}
"#,
    )
    .unwrap();

    // Must provide required field
    let result = repl.eval(r#"let u = User { name: "Alice" }"#).unwrap();
    let _ = result;

    // Check required field
    let name = repl.eval("u.name").unwrap();
    assert!(name.contains("Alice"), "Expected 'Alice' but got: {name}");

    // Check defaults
    let role = repl.eval("u.role").unwrap();
    assert!(role.contains("user"), "Expected 'user' but got: {role}");

    let active = repl.eval("u.active").unwrap();
    assert!(active.contains("true"), "Expected true but got: {active}");
}

#[test]
fn test_struct_numeric_defaults() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval(
        r"
struct Config {
    pub port: i32 = 8080,
    pub timeout: f64 = 30.0,
    pub retries: i32 = 3
}
",
    )
    .unwrap();

    let result = repl.eval("let c = Config {}").unwrap();
    let _ = result;

    let port = repl.eval("c.port").unwrap();
    assert!(port.contains("8080"), "Expected 8080 but got: {port}");

    let timeout = repl.eval("c.timeout").unwrap();
    assert!(timeout.contains("30"), "Expected 30.0 but got: {timeout}");

    let retries = repl.eval("c.retries").unwrap();
    assert!(retries.contains('3'), "Expected 3 but got: {retries}");
}

#[test]
fn test_struct_boolean_defaults() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    repl.eval(
        r"
struct Flags {
    pub debug: bool = false,
    pub verbose: bool = true,
    pub strict: bool = false
}
",
    )
    .unwrap();

    let result = repl.eval("let f = Flags {}").unwrap();
    let _ = result;

    let debug = repl.eval("f.debug").unwrap();
    assert!(debug.contains("false"), "Expected false but got: {debug}");

    let verbose = repl.eval("f.verbose").unwrap();
    assert!(verbose.contains("true"), "Expected true but got: {verbose}");
}
