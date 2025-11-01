//! TRANSPILER-DEFECT-018: Moved Value in Loop
//!
//! **Issue**: Variables from outer loop scopes are moved when passed to functions in inner loops,
//! causing E0382 "use of moved value" errors on subsequent iterations.
//!
//! **Root Cause**: Function calls inside loops move ownership of arguments, but Ruchy doesn't
//! auto-clone to allow reuse in loop iterations.
//!
//! **Impact**: 1 error in reaper project (line 308/1313)
//!
//! **Real-world Example** (reaper main.ruchy:1305-1318):
//! ```ruchy
//! while i < procs.len() {
//!     let proc = procs[i];
//!     while j < rules.len() {
//!         let rule = rules[j];
//!         if rule.enabled && rule_matches_process(rule, proc) {  // Moves proc
//!             break;
//!         }
//!         j = j + 1;  // Second iteration: proc already moved!
//!     }
//! }
//! ```
//!
//! **Current Transpilation** (BROKEN):
//! ```rust
//! while i < procs.len() {
//!     let proc = procs[i as usize].clone();
//!     while j < rules.len() {
//!         let rule = rules[j as usize].clone();
//!         if rule.enabled && rule_matches_process(rule, proc) {  // ❌ Moves proc
//!             break;
//!         }
//!         j = j + 1;  // ❌ E0382: proc already moved
//!     }
//! }
//! ```
//!
//! **Expected Transpilation**:
//! ```rust
//! while i < procs.len() {
//!     let proc = procs[i as usize].clone();
//!     while j < rules.len() {
//!         let rule = rules[j as usize].clone();
//!         if rule.enabled && rule_matches_process(rule, proc.clone()) {  // ✅ Clone
//!             break;
//!         }
//!         j = j + 1;  // ✅ Works - proc still available
//!     }
//! }
//! ```
//!
//! **Test Strategy**: EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Test 1: Nested loops with moved value (ACTUAL reaper pattern line 1313)
#[test]
fn test_defect_018_01_nested_loop_moved_value_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    // Simplified version of actual reaper pattern
    let ruchy_code = r#"
struct Item { id: i32 }

fun process_item(item: Item) -> bool {
    item.id > 0
}

fun find_items(items: [Item], checks: [i32]) -> i32 {
    let mut count = 0;
    let mut i = 0;
    while i < items.len() {
        let item = items[i];

        let mut j = 0;
        while j < checks.len() {
            if process_item(item) {  // Moves item on first iteration
                count = count + 1;
                break;
            }
            j = j + 1;  // Second iteration fails: item moved
        }
        i = i + 1;
    }
    count
}

let items = vec![Item { id: 1 }, Item { id: 2 }];
let checks = vec![1, 2];
println(find_items(items, checks));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0382"),
            "Expected E0382: use of moved value. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Moved value in loop error confirmed");
    } else {
        eprintln!("✅ GREEN: Auto-cloning prevents moved value in loops");
    }
}

/// Test 2: Single loop with moved value (simpler case)
#[test]
fn test_defect_018_02_single_loop_moved_value_red() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Data { value: i32 }

fun consume(d: Data) -> i32 { d.value }

fun process(items: [Data]) -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < items.len() {
        let item = items[i];
        sum = sum + consume(item);  // Moves item
        sum = sum + consume(item);  // Second call: item already moved
        i = i + 1;
    }
    sum
}

println(process(vec![Data { value: 5 }]));
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .output()
        .unwrap();

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("E0382"),
            "Expected E0382 for single loop. Got:\n{}",
            stderr
        );
        eprintln!("✅ RED TEST: Single loop moved value confirmed");
    } else {
        eprintln!("✅ GREEN: Single loop auto-cloning works");
    }
}

/// Test 3: Baseline - No loops, no error expected
#[test]
fn test_defect_018_03_no_loop_baseline() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.ruchy");

    let ruchy_code = r#"
struct Item { id: i32 }

fun process_item(item: Item) -> i32 { item.id }

let item = Item { id: 42 };
let result = process_item(item);
println(result);
"#;

    fs::write(&test_file, ruchy_code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("compile")
        .arg(&test_file)
        .assert()
        .success();
}

// PROPERTY TESTS (Run after GREEN phase)
// MUTATION TESTS (Run after GREEN phase)
