#[cfg(test)]
mod quality_gates_tdd {
    use std::process::Command;

    #[test]
    fn test_no_clippy_warnings() {
        // RED: This test should fail if there are any clippy warnings
        let output = Command::new("cargo")
            .args([
                "clippy",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .output()
            .expect("Failed to run clippy");

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Check for success
        assert!(
            output.status.success(),
            "Clippy found warnings or errors:\n{stderr}"
        );
    }

    #[test]
    fn test_coverage_reporting_works() {
        // RED: Test that coverage can be generated
        let output = Command::new("cargo")
            .args(["llvm-cov", "--version"])
            .output();

        assert!(
            output.is_ok(),
            "cargo-llvm-cov should be installed for coverage"
        );

        if let Ok(output) = output {
            assert!(
                output.status.success(),
                "cargo-llvm-cov should be available, install with: cargo install cargo-llvm-cov"
            );
        }
    }

    #[test]
    fn test_all_tests_pass() {
        // RED: Test that all tests pass
        let output = Command::new("cargo")
            .args(["test", "--lib"])
            .output()
            .expect("Failed to run tests");

        assert!(
            output.status.success(),
            "Some tests are failing, fix them first"
        );
    }
}
