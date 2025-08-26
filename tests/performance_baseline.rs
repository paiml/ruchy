//! Performance Baseline Test (QUALITY-010)
//! 
//! Establishes baseline performance metrics for critical compilation paths

#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};
use std::time::Instant;

#[test]
fn test_compilation_performance_baseline() {
    // Test programs of varying complexity
    let test_cases = vec![
        (
            "simple",
            r#"let x = 42"#,
            100, // Expected max ms
        ),
        (
            "hello_world",
            r#"println("Hello, World!")"#,
            100,
        ),
        (
            "arithmetic",
            r#"let x = 10 + 20 * 30 - 40 / 5"#,
            100,
        ),
        (
            "function",
            r#"
            fun add(a: i32, b: i32) -> i32 {
                a + b
            }
            add(5, 3)
            "#,
            100,
        ),
        (
            "fibonacci",
            r#"
            fun fibonacci(n: i32) -> i32 {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
            fibonacci(10)
            "#,
            100,
        ),
        (
            "match_expression",
            r#"
            match x {
                0 => "zero",
                1 => "one",
                _ => "other"
            }
            "#,
            100,
        ),
        (
            "list_operations",
            r#"
            let numbers = [1, 2, 3, 4, 5]
            numbers.map(|x| x * 2).filter(|x| x > 5)
            "#,
            100,
        ),
    ];

    println!("\n=== Compilation Performance Baseline ===\n");
    println!("{:<20} {:>15} {:>15} {:>15}", "Test Case", "Parse (ms)", "Transpile (ms)", "Total (ms)");
    println!("{:-<65}", "");

    let mut total_parse_time = 0.0;
    let mut total_transpile_time = 0.0;
    let mut all_under_target = true;

    for (name, code, target_ms) in test_cases {
        // Warm up
        for _ in 0..3 {
            let mut parser = Parser::new(code);
            let _ = parser.parse();
        }

        // Measure parsing
        let parse_start = Instant::now();
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Should parse successfully");
        let parse_duration = parse_start.elapsed();
        let parse_ms = parse_duration.as_secs_f64() * 1000.0;

        // Measure transpilation
        let transpile_start = Instant::now();
        let transpiler = Transpiler::new();
        let _ = transpiler.transpile(&ast).expect("Should transpile successfully");
        let transpile_duration = transpile_start.elapsed();
        let transpile_ms = transpile_duration.as_secs_f64() * 1000.0;

        let total_ms = parse_ms + transpile_ms;
        
        println!(
            "{:<20} {:>15.3} {:>15.3} {:>15.3} {}",
            name,
            parse_ms,
            transpile_ms,
            total_ms,
            if total_ms < target_ms as f64 { "✓" } else { "✗ SLOW" }
        );

        total_parse_time += parse_ms;
        total_transpile_time += transpile_ms;
        
        if total_ms >= target_ms as f64 {
            all_under_target = false;
        }
    }

    println!("{:-<65}", "");
    println!(
        "{:<20} {:>15.3} {:>15.3} {:>15.3}",
        "Average",
        total_parse_time / 7.0,
        total_transpile_time / 7.0,
        (total_parse_time + total_transpile_time) / 7.0
    );

    println!("\n=== Performance Summary ===");
    println!("Target: <100ms for typical compilation");
    println!("Status: {}", if all_under_target { "✓ All tests meet target" } else { "✗ Some tests exceed target" });
    
    // Assert that simple cases are under 100ms
    assert!(all_under_target, "Not all test cases met the <100ms target");
}

#[test]
fn test_parsing_throughput() {
    let sizes = vec![100, 1000, 10000];
    
    println!("\n=== Parsing Throughput Test ===\n");
    println!("{:<15} {:>15} {:>15} {:>15}", "Statements", "Time (ms)", "Throughput (stmt/s)", "MB/s");
    println!("{:-<60}", "");

    for size in sizes {
        let input = format!("let x{} = {}; ", "x".repeat(size % 10), size).repeat(size);
        let bytes = input.len();
        
        // Warm up
        for _ in 0..3 {
            let mut parser = Parser::new(&input);
            let _ = parser.parse();
        }
        
        // Measure
        let start = Instant::now();
        let mut parser = Parser::new(&input);
        let _ = parser.parse().expect("Should parse");
        let duration = start.elapsed();
        
        let ms = duration.as_secs_f64() * 1000.0;
        let throughput = size as f64 / duration.as_secs_f64();
        let mb_per_sec = (bytes as f64 / 1_000_000.0) / duration.as_secs_f64();
        
        println!(
            "{:<15} {:>15.3} {:>15.0} {:>15.2}",
            size, ms, throughput, mb_per_sec
        );
    }
}

#[test]
fn test_nested_expression_performance() {
    println!("\n=== Nested Expression Performance ===\n");
    println!("{:<15} {:>15} {:>15}", "Depth", "Parse (ms)", "Status");
    println!("{:-<45}", "");

    for depth in vec![5, 10, 20, 50] {
        let mut expr = "42".to_string();
        for _ in 0..depth {
            expr = format!("({} + 1)", expr);
        }
        
        // Warm up
        for _ in 0..3 {
            let mut parser = Parser::new(&expr);
            let _ = parser.parse();
        }
        
        // Measure
        let start = Instant::now();
        let mut parser = Parser::new(&expr);
        let _ = parser.parse().expect("Should parse nested expression");
        let duration = start.elapsed();
        let ms = duration.as_secs_f64() * 1000.0;
        
        println!(
            "{:<15} {:>15.3} {:>15}",
            depth,
            ms,
            if ms < 100.0 { "✓" } else { "✗ SLOW" }
        );
    }
}

#[test]
fn test_complex_program_performance() {
    println!("\n=== Complex Program Performance ===\n");
    
    let complex_program = r#"
        fun quicksort(arr: [i32]) -> [i32] {
            if arr.len() <= 1 {
                arr
            } else {
                let pivot = arr[0]
                let less = arr.filter(|x| x < pivot)
                let equal = arr.filter(|x| x == pivot)
                let greater = arr.filter(|x| x > pivot)
                quicksort(less) + equal + quicksort(greater)
            }
        }
        
        let numbers = [5, 2, 8, 1, 9, 3, 7, 4, 6]
        let sorted = quicksort(numbers)
        println(sorted)
    "#;
    
    // Warm up
    for _ in 0..3 {
        let mut parser = Parser::new(complex_program);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);
        }
    }
    
    // Measure
    let start = Instant::now();
    let mut parser = Parser::new(complex_program);
    let ast = parser.parse().expect("Should parse complex program");
    let parse_time = start.elapsed();
    
    let transpile_start = Instant::now();
    let transpiler = Transpiler::new();
    let _ = transpiler.transpile(&ast).expect("Should transpile complex program");
    let transpile_time = transpile_start.elapsed();
    
    let total_time = parse_time + transpile_time;
    
    println!("Parse time: {:.3}ms", parse_time.as_secs_f64() * 1000.0);
    println!("Transpile time: {:.3}ms", transpile_time.as_secs_f64() * 1000.0);
    println!("Total time: {:.3}ms", total_time.as_secs_f64() * 1000.0);
    println!("Status: {}", if total_time.as_millis() < 100 { "✓ Meets target" } else { "✗ Exceeds target" });
}