//! EXTREME TDD Test - println stdout capture
//!
//! This validates that the WASM REPL correctly captures println output
//! Bug: <https://github.com/paiml/ruchy/issues/PRINTLN_STDOUT>

use ruchy::wasm::repl::{WasmRepl, ReplOutput};

fn main() {
    println!("=== EXTREME TDD - Testing println stdout capture ===\n");

    let mut repl = WasmRepl::new().unwrap();

    // Test 1: Simple println
    println!("Test 1: Simple println");
    let result = repl.eval(r#"println("Hello, World!")"#).unwrap();
    let output: ReplOutput = serde_json::from_str(&result).unwrap();
    println!("  Success: {}", output.success);
    println!("  Display: {:?}", output.display);
    assert_eq!(output.display, Some("Hello, World!".to_string()));
    println!("  ✅ PASS\n");

    // Test 2: Multiple println
    println!("Test 2: Multiple println");
    let code = r#"
        println("Line 1");
        println("Line 2");
        println("Line 3");
    "#;
    let result = repl.eval(code).unwrap();
    let output: ReplOutput = serde_json::from_str(&result).unwrap();
    println!("  Success: {}", output.success);
    println!("  Display: {:?}", output.display);
    assert_eq!(output.display, Some("Line 1\nLine 2\nLine 3".to_string()));
    println!("  ✅ PASS\n");

    // Test 3: println with variables
    println!("Test 3: println with variables");
    let code = r#"
        let name = "Alice";
        println("Hello,", name);
    "#;
    let result = repl.eval(code).unwrap();
    let output: ReplOutput = serde_json::from_str(&result).unwrap();
    println!("  Success: {}", output.success);
    println!("  Display: {:?}", output.display);
    assert_eq!(output.display, Some("Hello, Alice".to_string()));
    println!("  ✅ PASS\n");

    // Test 4: Expression vs println
    println!("Test 4: Expression vs println");
    let expr_result = repl.eval("1 + 1").unwrap();
    let expr_output: ReplOutput = serde_json::from_str(&expr_result).unwrap();
    assert_eq!(expr_output.display, Some("2".to_string()));
    println!("  Expression: ✅ PASS");

    let print_result = repl.eval(r#"println("Hello")"#).unwrap();
    let print_output: ReplOutput = serde_json::from_str(&print_result).unwrap();
    assert_eq!(print_output.display, Some("Hello".to_string()));
    println!("  Println: ✅ PASS\n");

    println!("=== ALL TESTS PASSED ===");
    println!("\nGREEN phase complete! println stdout capture working correctly.");
}
