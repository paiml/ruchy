#![allow(missing_docs)]
//! REGRESSION TEST: Actor State Block Default Values
//!
//! ROOT CAUSE: After DEFECT-PARSER-001 fix (removing `Token::State` keyword),
//! discovered that `parse_state_block` didn't support default values.
//!
//! Example that failed:
//! ```ruchy
//! actor Counter {
//!     state {
//!         count: i32 = 0  // ❌ Error: Expected field name
//!     }
//! }
//! ```
//!
//! FIX: Added default value parsing to `parse_state_block` (lines 106-112)
//! to match `parse_inline_state_field` behavior.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_actor_state_block_with_default_value() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r"
actor Counter {
    state {
        count: i32 = 0
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_actor_state_block_without_default_value() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r"
actor Counter {
    state {
        count: i32
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_actor_state_block_multiple_fields_with_defaults() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
actor User {
    state {
        name: &str = "Anonymous"
        age: i32 = 0
        active: bool = true
    }
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_actor_state_block_mixed_defaults() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r"
actor Database {
    state {
        url: &str
        port: i32 = 5432
        timeout: i32
    }
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}

#[test]
fn test_actor_combining_state_variable_and_block() {
    // This test ensures BOTH fixes work together:
    // 1. 'state' variable in functions (DEFECT-PARSER-001)
    // 2. state { } block with defaults (REGRESSION fix)
    let temp = TempDir::new().expect("Failed to create temp dir");
    let source = temp.path().join("test.ruchy");

    let code = r#"
actor OrderProcessor {
    state {
        order_state: &str = "new"
    }

    receive Process => {
        let mut state = "processing";
        self.order_state = state
    }
}

fun process_order() -> &str {
    let mut state = "pending";
    if true {
        state = "confirmed";
    }
    state
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("check").arg(&source).assert().success();
}
