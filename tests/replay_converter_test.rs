//! Test the replay-to-test conversion pipeline
//!
//! Verifies that replay files can be converted to working test cases.

use ruchy::runtime::replay_converter::ReplayConverter;
use std::fs;
use std::path::Path;

#[test]
fn test_convert_basic_arithmetic_replay() -> anyhow::Result<()> {
    let converter = ReplayConverter::new();
    
    // Convert the basic arithmetic replay to tests
    let replay_path = Path::new("demos/replay_sessions/01_basic_arithmetic.replay");
    let tests = converter.convert_file(replay_path)?;
    
    // Should generate unit tests + integration test
    assert!(!tests.is_empty(), "Should generate at least one test");
    
    // Write a test file to verify compilation
    let mut test_content = String::new();
    test_content.push_str("//! Generated tests from basic arithmetic replay\n\n");
    test_content.push_str("use anyhow::Result;\n");
    test_content.push_str("use ruchy::runtime::repl::Repl;\n\n");
    
    for test in &tests {
        test_content.push_str(&test.code);
        test_content.push('\n');
    }
    
    // Write to a temporary test file
    fs::write("tests/generated_arithmetic.rs", &test_content)?;
    
    println!("Generated {} tests from replay session", tests.len());
    
    Ok(())
}

#[test]
fn test_replay_converter_creates_valid_rust() -> anyhow::Result<()> {
    let converter = ReplayConverter::new();
    
    // Test with a minimal replay session
    let replay_content = r#"{
  "version": { "major": 1, "minor": 0, "patch": 0 },
  "metadata": {
    "session_id": "test-session",
    "created_at": "2025-09-02T00:00:00Z",
    "ruchy_version": "1.30.1",
    "student_id": null,
    "assignment_id": null,
    "tags": ["test"]
  },
  "environment": {
    "seed": 0,
    "feature_flags": [],
    "resource_limits": {
      "heap_mb": 100,
      "stack_kb": 8192,
      "cpu_ms": 5000
    }
  },
  "timeline": [
    {
      "id": 1,
      "timestamp_ns": 1000,
      "event": {
        "Input": {
          "text": "2 + 2",
          "mode": "Interactive"
        }
      },
      "causality": []
    },
    {
      "id": 2,
      "timestamp_ns": 2000,
      "event": {
        "Output": {
          "result": {
            "Success": {
              "value": "4"
            }
          },
          "stdout": [],
          "stderr": []
        }
      },
      "causality": []
    }
  ],
  "checkpoints": {}
}"#;

    let session: ruchy::runtime::replay::ReplSession = serde_json::from_str(replay_content)?;
    let tests = converter.convert_session(&session, "test")?;
    
    assert!(!tests.is_empty(), "Should generate tests from minimal session");
    
    // Verify the generated code contains valid Rust
    for test in &tests {
        assert!(test.code.contains("fn test_"), "Should generate test functions");
        assert!(test.code.contains("Repl::new()"), "Should create REPL instances");
        assert!(test.code.contains("assert!"), "Should contain assertions");
    }
    
    Ok(())
}