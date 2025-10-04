// TDD tests to ensure destructuring quality improvements
// These tests ensure our refactoring maintains correct behavior

use std::process::Command;

fn run_ruchy_code(code: &str) -> String {
    let output = Command::new("ruchy")
        .arg("-e")
        .arg(code)
        .output()
        .expect("Failed to execute ruchy");

    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_list_with_defaults_basic() {
    // Check basic default destructuring
    let code = r"
let [a = 10, b = 20] = [1];
println(a);
println(b);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "1\n20");
}

#[test]
fn test_list_with_defaults_all_present() {
    // Check when all values are present
    let code = r"
let [a = 10, b = 20] = [1, 2];
println(a);
println(b);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "1\n2");
}

#[test]
fn test_list_with_defaults_empty_array() {
    // Check with empty array - all defaults should be used
    let code = r"
let [a = 10, b = 20] = [];
println(a);
println(b);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "10\n20");
}

#[test]
fn test_list_with_defaults_mixed() {
    // Check mixed patterns - some with defaults, some without
    let code = r"
let arr = [1, 2, 3];
let [x, y = 99, z = 88] = arr;
println(x);
println(y);
println(z);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "1\n2\n3");
}

#[test]
fn test_object_destructuring_basic() {
    // Check basic object destructuring
    let code = r#"
let obj = {name: "Alice", age: 30};
let {name, age} = obj;
println(name);
println(age);
"#;

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "Alice\n30");
}

#[test]
fn test_mixed_destructuring() {
    // Check mixed tuple and object destructuring
    let code = r"
let data = ([1, 2], {x: 10, y: 20});
let ([a, b], {x, y}) = data;
println(a);
println(b);
println(x);
println(y);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "1\n2\n10\n20");
}

#[test]
fn test_function_param_destructuring() {
    // Check function parameter destructuring
    let code = r"
fun add([x, y]) {
    return x + y;
}
println(add([5, 3]));
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "8");
}

#[test]
fn test_nested_destructuring_with_defaults() {
    // Check more complex nested patterns
    let code = r"
let [[a = 1, b = 2], [c = 3, d = 4]] = [[10], []];
println(a);
println(b);
println(c);
println(d);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "10\n2\n3\n4");
}

#[test]
fn test_rest_pattern_with_defaults() {
    // Check rest patterns combined with defaults
    let code = r"
let [first = 0, ...rest] = [1, 2, 3, 4];
println(first);
for item in rest {
    println(item);
}
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "1\n2\n3\n4");
}

#[test]
fn test_complex_default_expressions() {
    // Check that complex default expressions work
    let code = r"
fun get_default() { return 42; }
let [a = get_default(), b = 10 * 2] = [];
println(a);
println(b);
";

    let output = run_ruchy_code(code);
    assert_eq!(output.trim(), "42\n20");
}

// Performance test - ensure we don't regress on compilation time
#[test]
fn test_compilation_performance() {
    use std::time::Instant;

    let code = r"
let [a = 1, b = 2, c = 3, d = 4, e = 5] = [10, 20];
let {x, y, z} = {x: 100, y: 200, z: 300};
fun process([p1, p2, p3]) { return p1 + p2 + p3; }
println(a + b + c + d + e);
println(x + y + z);
println(process([1, 2, 3]));
";

    let start = Instant::now();
    let _ = run_ruchy_code(code);
    let duration = start.elapsed();

    // Should compile and run in under 2 seconds
    assert!(
        duration.as_secs() < 2,
        "Compilation took too long: {duration:?}"
    );
}
