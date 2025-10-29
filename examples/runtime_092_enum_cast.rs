// Example: RUNTIME-092 Enum Variable Cast Support (GitHub Issue #79)
//
// Demonstrates enum variable casts to integer types
// Run with: cargo run --example runtime_092_enum_cast

fn main() {
    println!("=== RUNTIME-092: Enum Variable Cast Examples ===\n");

    // Example 1: Basic enum variable cast
    println!("Example 1: Basic enum variable cast");
    let code1 = r#"
enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

fun main() {
    let level = LogLevel::Info;
    let level_int = level as i32;
    println("Level value: ");
    println(level_int);
}
"#;
    run_example(code1);

    // Example 2: Multiple enum variables
    println!("\nExample 2: Multiple enum variables");
    let code2 = r#"
enum Status {
    Pending = 10,
    Active = 20,
    Complete = 30,
}

fun main() {
    let s1 = Status::Pending;
    let s2 = Status::Active;
    let s3 = Status::Complete;

    println("Pending: ");
    println(s1 as i32);
    println("Active: ");
    println(s2 as i32);
    println("Complete: ");
    println(s3 as i32);
}
"#;
    run_example(code2);

    // Example 3: Enum casts in arithmetic expressions
    println!("\nExample 3: Enum casts in arithmetic");
    let code3 = r#"
enum Priority {
    Low = 1,
    Medium = 5,
    High = 10,
}

fun main() {
    let p = Priority::Medium;
    let doubled = (p as i32) * 2;
    println("Priority doubled: ");
    println(doubled);

    let p2 = Priority::High;
    let sum = (p as i32) + (p2 as i32);
    println("Priority sum: ");
    println(sum);
}
"#;
    run_example(code3);

    // Example 4: Cast to different integer types
    println!("\nExample 4: Cast to different integer types");
    let code4 = r#"
enum Value {
    X = 42,
}

fun main() {
    let v1 = Value::X;
    let as_i32 = v1 as i32;

    let v2 = Value::X;
    let as_i64 = v2 as i64;

    let v3 = Value::X;
    let as_isize = v3 as isize;

    println("i32: ");
    println(as_i32);
    println("i64: ");
    println(as_i64);
    println("isize: ");
    println(as_isize);
}
"#;
    run_example(code4);

    // Example 5: Direct enum literal cast (v3.147.3 feature)
    println!("\nExample 5: Direct enum literal cast");
    let code5 = r#"
enum Color {
    Red = 0xFF0000,
    Green = 0x00FF00,
    Blue = 0x0000FF,
}

fun main() {
    let red_val = Color::Red as i32;
    println("Red color code: ");
    println(red_val);

    let blue = Color::Blue;
    let blue_val = blue as i32;
    println("Blue color code: ");
    println(blue_val);
}
"#;
    run_example(code5);

    println!("\n=== All examples completed successfully ===");
}

fn run_example(code: &str) {
    use std::process::{Command, Stdio};

    let child = Command::new("./target/debug/ruchy")
        .arg("-e")
        .arg(code)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn ruchy process");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for ruchy process");

    if output.status.success() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }
}
