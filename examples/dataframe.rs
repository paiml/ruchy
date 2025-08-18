//! Examples demonstrating `DataFrame` functionality in Ruchy
//!
//! Run with: cargo run --example dataframe
#![allow(clippy::print_stdout)] // Examples should print output
#![allow(clippy::unwrap_used)] // Examples can use unwrap for simplicity

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::ExprKind;
use ruchy::frontend::parser::Parser;

fn main() {
    println!("=== Ruchy DataFrame Examples ===\n");

    // Example 1: Empty DataFrame
    example_empty_dataframe();

    // Example 2: Simple DataFrame with data
    example_simple_dataframe();

    // Example 3: DataFrame with multiple columns
    example_multi_column_dataframe();

    // Example 4: DataFrame operations
    example_dataframe_operations();
}

fn example_empty_dataframe() {
    println!("1. Empty DataFrame");
    println!("------------------");

    let input = "df![]";
    println!("Input: {input}");

    let ast = Parser::new(input).parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();

    println!("Transpiled: {output}");
    println!();
}

fn example_simple_dataframe() {
    println!("2. Simple DataFrame");
    println!("-------------------");

    let input = r#"df![names => ["Alice", "Bob", "Charlie"]]"#;
    println!("Input: {input}");

    let ast = Parser::new(input).parse().unwrap();

    if let ExprKind::DataFrame { columns } = &ast.kind {
        println!("Columns: {}", columns.len());
        for col in columns {
            println!("  - Column '{}' with {} values", col.name, col.values.len());
        }
    }

    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    println!("Transpiled: {output}");
    println!();
}

fn example_multi_column_dataframe() {
    println!("3. Multi-Column DataFrame");
    println!("-------------------------");

    let input = r#"df![
        names => ["Alice", "Bob", "Charlie"],
        ages => [25, 30, 35],
        scores => [92.5, 87.3, 95.1]
    ]"#;
    println!("Input: {input}");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::DataFrame { columns } = &ast.kind {
                println!("DataFrame with {} columns:", columns.len());
                for col in columns {
                    println!("  - '{}': {} values", col.name, col.values.len());
                }
            }

            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(tokens) => {
                    println!("✓ Successfully transpiled");
                    let output = tokens.to_string();
                    // Show first 200 chars of output
                    if output.len() > 200 {
                        println!("Output (truncated): {}...", &output[..200]);
                    } else {
                        println!("Output: {output}");
                    }
                }
                Err(e) => println!("✗ Transpilation error: {e}"),
            }
        }
        Err(e) => println!("✗ Parse error: {e}"),
    }
    println!();
}

fn example_dataframe_operations() {
    println!("4. DataFrame Operations");
    println!("-----------------------");

    let operations = vec![
        (
            "Filter",
            r#"df![scores => [92.5, 87.3, 95.1]].filter(col("scores") > 90)"#,
        ),
        ("Select", r#"df![a => [1], b => [2]].select(["a"])"#),
        ("Mean", r"df![values => [1, 2, 3, 4, 5]].mean()"),
    ];

    for (op_name, input) in operations {
        println!("{op_name}: {input}");

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

    println!("\n=== DataFrame Examples Complete ===");
}
