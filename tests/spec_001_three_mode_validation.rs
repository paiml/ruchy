// SPEC-001: Three-Mode Grammar Validation
// Ensures every "implemented: true" feature works in ALL THREE MODES:
// 1. Interpreter (ruchy run, -e)
// 2. Transpile (ruchy transpile)
// 3. Compile (ruchy compile + rustc)

use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to run a Ruchy file in ALL THREE MODES
fn validate_three_modes(code: &str, feature_name: &str) -> ThreeModeResult {
    let temp_dir = TempDir::new().unwrap();
    let ruchy_file = temp_dir.path().join("test.ruchy");
    fs::write(&ruchy_file, code).unwrap();

    let interpreter_works = test_interpreter_mode(&ruchy_file);
    let transpile_works = test_transpile_mode(&ruchy_file, &temp_dir);
    let compile_works = test_compile_mode(&ruchy_file, &temp_dir);

    ThreeModeResult {
        feature: feature_name.to_string(),
        interpreter: interpreter_works,
        transpile: transpile_works,
        compile: compile_works,
    }
}

#[derive(Debug)]
struct ThreeModeResult {
    feature: String,
    interpreter: bool,
    transpile: bool,
    compile: bool,
}

impl ThreeModeResult {
    fn all_pass(&self) -> bool {
        self.interpreter && self.transpile && self.compile
    }

    fn failure_report(&self) -> String {
        let mut failures = Vec::new();
        if !self.interpreter {
            failures.push("INTERPRETER");
        }
        if !self.transpile {
            failures.push("TRANSPILE");
        }
        if !self.compile {
            failures.push("COMPILE");
        }

        if failures.is_empty() {
            format!("✅ {} - ALL MODES PASS", self.feature)
        } else {
            format!("❌ {} - FAILED: {}", self.feature, failures.join(", "))
        }
    }
}

fn test_interpreter_mode(ruchy_file: &PathBuf) -> bool {
    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("run")
        .arg(ruchy_file)
        .assert()
        .try_success()
        .is_ok()
}

fn test_transpile_mode(ruchy_file: &PathBuf, temp_dir: &TempDir) -> bool {
    let rs_file = temp_dir.path().join("output.rs");

    let transpile_result = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg(ruchy_file)
        .arg("-o")
        .arg(&rs_file)
        .assert()
        .try_success();

    if transpile_result.is_err() {
        return false;
    }

    // Verify Rust file was created
    rs_file.exists()
}

fn test_compile_mode(ruchy_file: &PathBuf, temp_dir: &TempDir) -> bool {
    let rs_file = temp_dir.path().join("compiled.rs");

    // First transpile
    let transpile_ok = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg(ruchy_file)
        .arg("-o")
        .arg(&rs_file)
        .assert()
        .try_success()
        .is_ok();

    if !transpile_ok || !rs_file.exists() {
        return false;
    }

    // Then verify rustc compiles it
    let rlib_file = temp_dir.path().join("compiled.rlib");

    Command::new("rustc")
        .arg("--crate-type")
        .arg("lib")
        .arg(&rs_file)
        .arg("-o")
        .arg(&rlib_file)
        .assert()
        .try_success()
        .is_ok()
}

// =============================================================================
// EXPRESSIONS - All marked "implemented: true" in grammar.yaml
// =============================================================================

#[test]
fn test_spec_001_if_expr_three_modes() {
    let code = r#"
fun main() {
    let x = 5
    if x > 3 {
        println("yes")
    } else {
        println("no")
    }
}
"#;

    let result = validate_three_modes(code, "if_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_match_expr_three_modes() {
    let code = r#"
fun main() {
    let x = 2
    match x {
        1 => println("one"),
        2 => println("two"),
        _ => println("other")
    }
}
"#;

    let result = validate_three_modes(code, "match_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_for_expr_three_modes() {
    let code = r#"
fun main() {
    for i in 0..3 {
        println(i.to_string())
    }
}
"#;

    let result = validate_three_modes(code, "for_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_while_expr_three_modes() {
    let code = r#"
fun main() {
    let mut i = 0
    while i < 3 {
        println(i.to_string())
        i += 1
    }
}
"#;

    let result = validate_three_modes(code, "while_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_loop_expr_three_modes() {
    let code = r#"
fun main() {
    let mut i = 0
    loop {
        if i >= 3 {
            break
        }
        i += 1
    }
}
"#;

    let result = validate_three_modes(code, "loop_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_lambda_expr_three_modes() {
    let code = r#"
fun main() {
    let add = |x: i32, y: i32| -> i32 { x + y }
    let result = add(2, 3)
    println(result.to_string())
}
"#;

    let result = validate_three_modes(code, "lambda_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_pipeline_expr_three_modes() {
    // SPEC-001-C: Pipeline operator is |> (F#/Elixir style), NOT >> (bitwise shift)
    let code = r#"
fun double(x: i32) -> i32 { x * 2 }
fun add_one(x: i32) -> i32 { x + 1 }

fun main() {
    let result = 5 |> double |> add_one
    println(result.to_string())
}
"#;

    let result = validate_three_modes(code, "pipeline_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_lazy_expr_three_modes() {
    // SPEC-001-D: Lazy evaluation - computation deferred until value accessed
    let code = r#"
fun expensive_computation() -> i32 {
    println("Computing...")
    42
}

fun main() {
    let deferred = lazy expensive_computation()
    println("Before access")
    println(deferred.to_string())
}
"#;

    let result = validate_three_modes(code, "lazy_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_async_block_three_modes() {
    // SPEC-001-E: Async block - simplified synchronous evaluation
    // For full async support, would need tokio runtime
    let code = r#"
fun main() {
    let result = async {
        println("Async block")
        42
    }
    println(result.to_string())
}
"#;

    let result = validate_three_modes(code, "async_block");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_tuple_expr_three_modes() {
    let code = r#"
fun main() {
    let t = (1, 2, 3)
    println(t.0.to_string())
}
"#;

    let result = validate_three_modes(code, "tuple_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_array_expr_three_modes() {
    let code = r#"
fun main() {
    let arr = [1, 2, 3]
    println(arr[0].to_string())
}
"#;

    let result = validate_three_modes(code, "array_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_struct_expr_three_modes() {
    let code = r#"
struct Point {
    x: i32,
    y: i32
}

fun main() {
    let p = Point { x: 1, y: 2 }
    println(p.x.to_string())
}
"#;

    let result = validate_three_modes(code, "struct_expr");
    assert!(result.all_pass(), "{}", result.failure_report());
}

// =============================================================================
// GRAMMAR - Top-level declarations
// =============================================================================

#[test]
fn test_spec_001_function_decl_three_modes() {
    let code = r#"
fun add(x: i32, y: i32) -> i32 {
    x + y
}

fun main() {
    println(add(2, 3).to_string())
}
"#;

    let result = validate_three_modes(code, "function_decl");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_struct_decl_three_modes() {
    let code = r#"
struct User {
    name: String,
    age: i32
}

fun main() {
    let u = User { name: "Alice".to_string(), age: 30 }
    println(u.name)
}
"#;

    let result = validate_three_modes(code, "struct_decl");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_enum_decl_three_modes() {
    let code = r#"
enum Color {
    Red,
    Green,
    Blue
}

fun main() {
    let c = Color::Red
    match c {
        Color::Red => println("red"),
        Color::Green => println("green"),
        Color::Blue => println("blue")
    }
}
"#;

    let result = validate_three_modes(code, "enum_decl");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_const_decl_three_modes() {
    let code = r#"
const MAX_SIZE: i32 = 100

fun main() {
    println(MAX_SIZE.to_string())
}
"#;

    let result = validate_three_modes(code, "const_decl");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_impl_block_three_modes() {
    let code = r#"
struct Counter {
    value: i32
}

impl Counter {
    fun new() -> Counter {
        Counter { value: 0 }
    }

    fun increment(&mut self) {
        self.value += 1
    }
}

fun main() {
    let mut c = Counter::new()
    c.increment()
    println(c.value.to_string())
}
"#;

    let result = validate_three_modes(code, "impl_block");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_actor_decl_three_modes() {
    // SPEC-001-F: Actor declarations - simplified synchronous message handling
    // Actors transpile to plain structs wrapped in Arc<Mutex<>>
    let code = r#"
actor Counter {
    value: i32
}

fun main() {
    let counter = spawn Counter { value: 0 }
    println("Actor created")
}
"#;

    let result = validate_three_modes(code, "actor_decl");
    assert!(result.all_pass(), "{}", result.failure_report());
}

// =============================================================================
// PATTERNS - All marked "implemented: true"
// =============================================================================

#[test]
fn test_spec_001_literal_pattern_three_modes() {
    let code = r#"
fun main() {
    let x = 5
    match x {
        5 => println("five"),
        _ => println("other")
    }
}
"#;

    let result = validate_three_modes(code, "literal_pattern");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_wildcard_pattern_three_modes() {
    let code = r#"
fun main() {
    let x = 10
    match x {
        _ => println("anything")
    }
}
"#;

    let result = validate_three_modes(code, "wildcard_pattern");
    assert!(result.all_pass(), "{}", result.failure_report());
}

#[test]
fn test_spec_001_tuple_pattern_three_modes() {
    let code = r#"
fun main() {
    let t = (1, 2)
    match t {
        (a, b) => println(a.to_string())
    }
}
"#;

    let result = validate_three_modes(code, "tuple_pattern");
    assert!(result.all_pass(), "{}", result.failure_report());
}

// =============================================================================
// INTEGRATION TEST - Run all and report failures
// =============================================================================

#[test]
fn test_spec_001_full_grammar_validation_report() {
    println!("\n🔍 SPEC-001: Three-Mode Grammar Validation Report");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let features = vec![
        ("if_expr", r#"fun main() { if true { println("ok") } }"#),
        ("match_expr", r#"fun main() { match 1 { 1 => println("one"), _ => println("other") } }"#),
        ("for_expr", r#"fun main() { for i in 0..3 { println(i.to_string()) } }"#),
        ("while_expr", r#"fun main() { let mut i = 0; while i < 3 { i += 1 } }"#),
        ("loop_expr", r#"fun main() { let mut i = 0; loop { if i >= 3 { break }; i += 1 } }"#),
        ("tuple_expr", r#"fun main() { let t = (1, 2); println(t.0.to_string()) }"#),
        ("array_expr", r#"fun main() { let arr = [1, 2]; println(arr[0].to_string()) }"#),
    ];

    let mut all_pass = true;
    let mut failures = Vec::new();

    for (feature, code) in features {
        let result = validate_three_modes(code, feature);
        println!("{}", result.failure_report());

        if !result.all_pass() {
            all_pass = false;
            failures.push(result);
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if !all_pass {
        println!("\n❌ FAILURES DETECTED:");
        for failure in &failures {
            println!("   - {}: interpreter={}, transpile={}, compile={}",
                failure.feature,
                failure.interpreter,
                failure.transpile,
                failure.compile
            );
        }
        println!("\n🚨 CREATE ROADMAP TICKETS FOR EACH FAILURE (EXTREME TDD)");
        panic!("Three-mode validation failed - see report above");
    } else {
        println!("✅ ALL FEATURES PASS IN ALL THREE MODES");
    }
}
