//! Examples demonstrating pattern matching in Ruchy
//!
//! Run with: `cargo run --example pattern_matching`
#![allow(clippy::print_stdout)] // Examples should print output
#![allow(clippy::unwrap_used)] // Examples can use unwrap for simplicity

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{ExprKind, Pattern};
use ruchy::frontend::parser::Parser;

fn main() {
    println!("=== Ruchy Pattern Matching Examples ===\n");

    // Example 1: Basic patterns
    example_basic_patterns();

    // Example 2: List patterns
    example_list_patterns();

    // Example 3: Tuple patterns
    example_tuple_patterns();

    // Example 4: Complex patterns
    example_complex_patterns();
}

fn example_basic_patterns() {
    println!("1. Basic Patterns");
    println!("-----------------");

    let input = r#"
        match value {
            0 => "zero",
            1 | 2 => "small",
            n if n > 10 => "large",
            _ => "other"
        }
    "#;

    println!("Basic match with literals and guards:");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::Match { arms, .. } = &ast.kind {
                println!("  {} match cases", arms.len());
                for (i, arm) in arms.iter().enumerate() {
                    match &arm.pattern {
                        Pattern::Literal(_) => println!("  Case {}: Literal pattern", i + 1),
                        Pattern::Or(_) => println!("  Case {}: Or pattern (|)", i + 1),
                        Pattern::Wildcard => println!("  Case {}: Wildcard (_)", i + 1),
                        Pattern::Identifier(name) => {
                            println!("  Case {}: Identifier ({})", i + 1, name);
                            if arm.guard.is_some() {
                                println!("    with guard condition");
                            }
                        }
                        _ => println!("  Case {}: Other pattern", i + 1),
                    }
                }
            }

            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(output) => {
                    let output_str = output.to_string();
                    if output_str.contains("match") && output_str.contains("=>") {
                        println!("\n✓ Successfully transpiled to Rust match");
                    }
                }
                Err(e) => println!("\n✗ Transpilation error: {e}"),
            }
        }
        Err(e) => println!("✗ Parse error: {e}"),
    }
    println!();
}

fn example_list_patterns() {
    println!("2. List Patterns");
    println!("----------------");

    let examples = vec![
        ("Empty list", "match list { [] => \"empty\" }"),
        ("Single element", "match list { [x] => x }"),
        ("Head and tail", "match list { [head, ...tail] => head }"),
        ("Multiple elements", "match list { [a, b, c] => a + b + c }"),
    ];

    for (description, input) in examples {
        println!(
            "{description}: {}",
            &input[12..].chars().take(30).collect::<String>()
        );

        match Parser::new(input).parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(_) => println!("  ✓ Transpiles successfully"),
                    Err(e) => println!("  ✗ Transpilation error: {e}"),
                }
            }
            Err(e) => println!("  ✗ Parse error: {e}"),
        }
    }
    println!();
}

fn example_tuple_patterns() {
    println!("3. Tuple Patterns");
    println!("-----------------");

    let input = r#"
        match point {
            (0, 0) => "origin",
            (x, 0) => "on x-axis",
            (0, y) => "on y-axis",
            (x, y) => "point at ({x}, {y})"
        }
    "#;

    println!("Tuple pattern matching:");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::Match { arms, .. } = &ast.kind {
                println!("  {} tuple patterns", arms.len());
                for arm in arms {
                    if matches!(arm.pattern, Pattern::Tuple(_)) {
                        println!("  ✓ Tuple pattern detected");
                        break;
                    }
                }
            }

            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(output) => {
                    let output_str = output.to_string();
                    if output_str.contains("(0, 0)") {
                        println!("  ✓ Literal tuple patterns preserved");
                    }
                }
                Err(e) => println!("  ✗ Transpilation error: {e}"),
            }
        }
        Err(e) => println!("  ✗ Parse error: {e}"),
    }
    println!();
}

fn example_complex_patterns() {
    println!("4. Complex Patterns");
    println!("-------------------");

    let examples = vec![
        (
            "Struct pattern",
            r#"match user { User { name, age } => "{name} is {age}" }"#,
        ),
        ("Nested patterns", r"match data { Some([x, y]) => x + y }"),
        (
            "Range pattern",
            r#"match score { 0..60 => "fail", 60..100 => "pass" }"#,
        ),
    ];

    for (description, input) in examples {
        println!("{description}:");
        println!("  Input: {}", &input[..input.len().min(50)]);

        match Parser::new(input).parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(output) => {
                        let output_str = output.to_string();
                        println!("  ✓ Transpiled ({} chars)", output_str.len());
                    }
                    Err(e) => println!("  ✗ Transpilation error: {e}"),
                }
            }
            Err(e) => println!("  ✗ Parse error: {e}"),
        }
    }

    println!("\n=== Pattern Matching Examples Complete ===");
}
