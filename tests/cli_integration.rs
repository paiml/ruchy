//! Integration tests for CLI commands
//! Toyota Way: Comprehensive CLI testing to prevent regressions

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

mod fmt_tests {
    //! Integration tests for `ruchy fmt` command
    //! Toyota Way: Test every possible failure mode

    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_fmt_formats_simple_function() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");
        
        // Write unformatted code
        fs::write(&test_file, "fun test(x:i32)->i32{x*2}").unwrap();
        
        // Run formatter
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success()
            .stdout(predicate::str::contains("✓ Formatted"));
        
        // Verify formatting
        let formatted = fs::read_to_string(&test_file).unwrap();
        let expected = "fun test(x: i32) -> i32 {\nx * 2\n}";
        assert_eq!(formatted.trim(), expected);
    }

    #[test]
    fn test_fmt_handles_complex_expressions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("complex.ruchy");
        
        // Complex unformatted code
        fs::write(&test_file, "fun complex(x:i32,y:i32)->i32{if x>y{x*2}else{y*2}}").unwrap();
        
        // Run formatter
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();
        
        // Verify proper formatting structure
        let formatted = fs::read_to_string(&test_file).unwrap();
        assert!(formatted.contains("fun complex(x: i32, y: i32) -> i32"));
        assert!(formatted.contains("if x > y"));
        assert!(formatted.contains("} else {"));
    }

    #[test]
    fn test_fmt_preserves_semantics() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("semantics.ruchy");
        
        // Test various constructs
        let original = r"
    fun add(a: i32, b: i32) -> i32 { a + b }
    fun multiply(x: i32) -> i32 { x * 2 }
    ";
        fs::write(&test_file, original).unwrap();
        
        // Format
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();
        
        // Verify key elements preserved
        let formatted = fs::read_to_string(&test_file).unwrap();
        assert!(formatted.contains("fun add(a: i32, b: i32) -> i32"));
        assert!(formatted.contains("a + b"));
        assert!(formatted.contains("fun multiply(x: i32) -> i32"));
        assert!(formatted.contains("x * 2"));
    }

    #[test]
    fn test_fmt_error_on_missing_file() {
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", "nonexistent.ruchy"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("No such file or directory").or(
                predicate::str::contains("cannot find the file")
            ));
    }

    #[test]
    fn test_fmt_error_on_invalid_syntax() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("invalid.ruchy");
        
        // Invalid syntax
        fs::write(&test_file, "fun invalid_syntax(((((").unwrap();
        
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .failure();
    }

    #[test]
    fn test_fmt_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("idempotent.ruchy");
        
        // Write initial code
        fs::write(&test_file, "fun test(x: i32) -> i32 { x * 2 }").unwrap();
        
        // Format first time
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();
        
        let first_format = fs::read_to_string(&test_file).unwrap();
        
        // Format second time
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();
        
        let second_format = fs::read_to_string(&test_file).unwrap();
        
        // Should be identical
        assert_eq!(first_format, second_format);
    }

    #[test]
    fn test_fmt_multiple_functions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("multiple.ruchy");
        
        let code = r"
    fun first(x:i32)->i32{x+1}
    fun second(y:i32)->i32{y*2}
    fun third(z:i32)->i32{if z>0{z}else{0}}
    ";
        fs::write(&test_file, code).unwrap();
        
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();
        
        let formatted = fs::read_to_string(&test_file).unwrap();
        
        // Verify all functions are properly formatted
        assert!(formatted.contains("fun first(x: i32) -> i32"));
        assert!(formatted.contains("fun second(y: i32) -> i32"));  
        assert!(formatted.contains("fun third(z: i32) -> i32"));
        assert!(formatted.contains("if z > 0"));
    }

    #[test]
    fn test_fmt_block_expressions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("blocks.ruchy");
        
        let code = "fun test()->i32{{let x=1;let y=2;x+y}}";
        fs::write(&test_file, code).unwrap();
        
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();
        
        let formatted = fs::read_to_string(&test_file).unwrap();
        assert!(formatted.contains("fun test() -> i32"));
        // Block formatting should be preserved/improved
        assert!(formatted.contains('{'));
        assert!(formatted.contains('}'));
    }
}