//! PARSER-084: Range Expression and Slicing Example
//!
//! Demonstrates the fix for GitHub Issue #67 - parsing open-ended and closed
//! range expressions in various contexts.
//!
//! Run with: cargo run --example `parser_084_range_slicing`

use ruchy::Parser;

fn main() {
    println!("PARSER-084: Range Expression Parsing Examples\n");
    println!("==============================================\n");

    // Example 1: Closed ranges (start..end)
    let code1 = r#"
fun example_closed_range() {
    let range = 2..5;
    println("Closed range: 2..5");
}
"#;
    test_parse("Closed Range (2..5)", code1);

    // Example 2: Open-ended ranges (start..)
    let code2 = r#"
fun example_open_ended() {
    let range = 2..;
    println("Open-ended range: 2..");
}
"#;
    test_parse("Open-Ended Range (2..)", code2);

    // Example 3: Open-start ranges (..end)
    let code3 = r#"
fun example_open_start() {
    let range = ..5;
    println("Open-start range: ..5");
}
"#;
    test_parse("Open-Start Range (..5)", code3);

    // Example 4: Array slicing with open-ended range
    let code4 = r#"
fun example_array_slice() {
    let arr = [1, 2, 3, 4, 5];
    let slice = arr[2..];
    println("Array slice: arr[2..]");
}
"#;
    test_parse("Array Slice (arr[2..])", code4);

    // Example 5: String slicing with open-start range
    let code5 = r#"
fun example_string_slice() {
    let s = "hello world";
    let slice = &s[..5];
    println("String slice: &s[..5]");
}
"#;
    test_parse("String Slice (&s[..5])", code5);

    // Example 6: Original failing case from GitHub Issue #67
    let code6 = r#"
use std::collections::HashMap;

fun parse_args(args: Vec<String>) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            let key_part = &arg[2..];
            parsed.insert(key_part.to_string(), String::from("value"));
        }

        i += 1;
    }

    parsed
}
"#;
    test_parse("GitHub Issue #67 - Original Failing Case", code6);

    // Example 7: Range in let statement inside if block
    let code7 = r#"
fun example_nested_context() {
    let data = "example";
    if true {
        let slice = &data[2..];
        println("Nested slice in if block");
    }
}
"#;
    test_parse("Range in Let Statement in If Block", code7);

    println!("\n✅ All examples parsed successfully!");
    println!("\nPARSER-084 Fix Summary:");
    println!("  - Open-ended ranges (2..) now work");
    println!("  - Open-start ranges (..5) now work");
    println!("  - Ranges in all contexts (let, if, while, slicing) work");
    println!("  - GitHub Issue #67 is resolved");
}

fn test_parse(description: &str, code: &str) {
    print!("Testing: {description}... ");
    match Parser::new(code).parse() {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => {
            println!("✗ Failed!");
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
