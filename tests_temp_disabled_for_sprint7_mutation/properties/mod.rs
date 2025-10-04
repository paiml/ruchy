// tests/properties/mod.rs
// Property-based testing for Ruchy language features
// Inspired by Haskell QuickCheck and Elixir StreamData

pub mod parser; // NEW: PROPTEST-003 - Parser property tests
pub mod parser_properties;
pub mod runtime_properties;
// pub mod transpiler_properties;  // Module file does not exist

use std::fs;
use std::process::Command;

/// Generate random valid Ruchy identifiers
pub fn valid_identifier() -> impl quickcheck::Arbitrary {
    use quickcheck::{Arbitrary, Gen};

    #[derive(Clone, Debug)]
    pub struct ValidIdentifier(pub String);

    impl Arbitrary for ValidIdentifier {
        fn arbitrary(g: &mut Gen) -> Self {
            let first_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
            let other_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

            let len = g.size() % 20 + 1; // 1-20 chars
            let mut name = String::new();

            // First character
            let first_idx = usize::arbitrary(g) % first_chars.len();
            name.push(first_chars.chars().nth(first_idx).unwrap());

            // Remaining characters
            for _ in 1..len {
                let idx = usize::arbitrary(g) % other_chars.len();
                name.push(other_chars.chars().nth(idx).unwrap());
            }

            ValidIdentifier(name)
        }
    }

    ValidIdentifier::arbitrary
}

/// Test if code can be parsed without errors
pub fn can_parse(code: &str) -> bool {
    let test_file = "/tmp/quickcheck_parse_test.ruchy";
    fs::write(test_file, code).unwrap();

    let output = Command::new("./target/release/ruchy")
        .arg("transpile")
        .arg(test_file)
        .output()
        .unwrap();

    output.status.success()
}

/// Test if code can be executed without errors
pub fn can_execute(code: &str) -> bool {
    let test_file = "/tmp/quickcheck_exec_test.ruchy";
    fs::write(test_file, code).unwrap();

    let output = Command::new("./target/release/ruchy")
        .arg("run")
        .arg(test_file)
        .output()
        .unwrap();

    output.status.success()
}
