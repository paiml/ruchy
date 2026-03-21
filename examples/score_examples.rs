#![allow(clippy::too_many_lines)]

/// Examples demonstrating different quality scores
/// Run with: cargo run --example `score_examples`
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn build_examples() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "perfect_code",
            r#"
// Perfect code - Score ~1.0
fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let result = calculate_sum(5, 3);
    println("Sum: {}", result);
}
"#,
            "1.00",
        ),
        (
            "good_code",
            r"
// Good code with minor complexity - Score ~0.8
fn process_data(values: Vec<i32>) -> i32 {
    let mut sum = 0;
    for value in values {
        if value > 0 {
            sum += value;
        }
    }
    sum
}
",
            "0.80-0.90",
        ),
        (
            "moderate_complexity",
            r"
// Moderate complexity - Score ~0.5
fn complex_logic(a: i32, b: i32, c: i32, d: i32, e: i32) -> i32 {
    let mut result = 0;

    if a > 0 {
        if b > 0 {
            if c > 0 {
                result = a + b + c;
            } else {
                result = a + b;
            }
        } else {
            if d > 0 {
                result = a + d;
            }
        }
    } else {
        if e > 0 {
            for i in 0..e {
                if i % 2 == 0 {
                    result += i;
                }
            }
        }
    }

    result
}
",
            "0.40-0.60",
        ),
        (
            "poor_quality",
            r"
// Poor quality - too many parameters, deep nesting - Score ~0.2
fn terrible_function(
    param1: i32, param2: i32, param3: i32, param4: i32,
    param5: i32, param6: i32, param7: i32, param8: i32,
    param9: i32, param10: i32, param11: i32, param12: i32,
    param13: i32, param14: i32, param15: i32
) -> i32 {
    let mut x = 0;
    if param1 > 0 {
        if param2 > 0 {
            if param3 > 0 {
                if param4 > 0 {
                    if param5 > 0 {
                        if param6 > 0 {
                            if param7 > 0 {
                                x = param1 + param2 + param3;
                            }
                        }
                    }
                }
            }
        }
    }

    for i in 0..param8 {
        for j in 0..param9 {
            for k in 0..param10 {
                if i > j {
                    if j > k {
                        x += i * j * k;
                    }
                }
            }
        }
    }

    x
}
",
            "0.10-0.30",
        ),
        (
            "catastrophic_quality",
            r"
// Catastrophic quality - Score ~0.05
fn nightmare(
    a1: i32, a2: i32, a3: i32, a4: i32, a5: i32,
    b1: i32, b2: i32, b3: i32, b4: i32, b5: i32,
    c1: i32, c2: i32, c3: i32, c4: i32, c5: i32,
    d1: i32, d2: i32, d3: i32, d4: i32, d5: i32,
    e1: i32, e2: i32, e3: i32, e4: i32, e5: i32,
    f1: i32
) -> i32 {
    let mut result = 0;

    // 8+ levels of nesting
    if a1 > 0 {
        if a2 > 0 {
            if a3 > 0 {
                if a4 > 0 {
                    if a5 > 0 {
                        if b1 > 0 {
                            if b2 > 0 {
                                if b3 > 0 {
                                    if b4 > 0 {
                                        result = a1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Multiple nested loops
    for i in 0..10 {
        for j in 0..10 {
            for k in 0..10 {
                for l in 0..10 {
                    if i > j && j > k && k > l {
                        result += 1;
                    }
                }
            }
        }
    }

    result
}
",
            "0.01-0.10",
        ),
    ]
}

fn run_score_command(file_path: &Path) {
    let output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute ruchy score");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Some(score_line) = stdout.lines().find(|l| l.contains("Score:")) {
        println!("{score_line}");
    } else {
        println!("Output: {stdout}");
    }
}

fn run_json_score_and_validate(file_path: &Path, expected_score: &str) {
    let json_output = Command::new("./target/debug/ruchy")
        .args(["score", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("Failed to execute ruchy score");

    if !json_output.status.success() {
        return;
    }

    let json_str = String::from_utf8_lossy(&json_output.stdout);
    let json: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    let score = match json["score"].as_f64() {
        Some(s) => s,
        None => return,
    };

    println!("Actual Score: {score:.2}");

    let in_range = match expected_score {
        "1.00" => score >= 0.95,
        "0.80-0.90" => (0.80..=0.90).contains(&score),
        "0.40-0.60" => (0.40..=0.60).contains(&score),
        "0.10-0.30" => (0.10..=0.30).contains(&score),
        "0.01-0.10" => (0.01..=0.10).contains(&score),
        _ => true,
    };

    if in_range {
        println!("✅ Score is in expected range");
    } else {
        println!("❌ Score {score} is outside expected range {expected_score}");
    }
}

fn run_example(temp_dir: &Path, name: &str, code: &str, expected_score: &str) {
    println!("📝 Example: {name} (Expected: {expected_score})");
    println!("{}", "─".repeat(50));

    let file_path = temp_dir.join(format!("{name}.ruchy"));
    fs::write(&file_path, code).unwrap();

    run_score_command(&file_path);
    run_json_score_and_validate(&file_path, expected_score);

    println!("\n");
}

fn print_summary() {
    println!("🏁 Score Examples Complete");
    println!("\nTo test individual files:");
    println!("  ruchy score path/to/file.ruchy");
    println!("  ruchy score path/to/file.ruchy --format json");
    println!("  ruchy score path/to/file.ruchy --min 0.8  # Enforce minimum score");
}

fn main() {
    println!("🎯 Ruchy Score Examples - Demonstrating Quality Spectrum\n");

    let examples = build_examples();
    let temp_dir = TempDir::new().unwrap();

    for (name, code, expected_score) in examples {
        run_example(temp_dir.path(), name, code, expected_score);
    }

    print_summary();
}
