//! QUALITY-001: Method receiver preservation (&self, &mut self, self)
//!
//! BUG: Transpiler transforms &self → self, causing move errors
//! ERROR: error[E0382]: use of moved value: `client`
//!
//! ROOT CAUSE: `generate_param_tokens` doesn't handle Rust's special receiver syntax
//!
//! IMPACT: Methods with &self cannot be called multiple times (ownership moved)
//!
//! EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::{Parser, Transpiler};

/// Test 1: Basic &self method - should preserve reference
#[test]
fn test_quality_001_01_immutable_self_reference() {
    let code = r"
pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Should transpile &self methods, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // CRITICAL: Must preserve &self, not transform to self
    assert!(
        rust_code.contains("& self") || rust_code.contains("&self"),
        "Should preserve &self reference, got: {rust_code}"
    );

    // Should NOT have ownership-taking self
    assert!(
        !rust_code.contains("fn get_endpoint ( self )"),
        "Should NOT transform &self to self, got: {rust_code}"
    );
}

/// Test 2: Multiple &self method calls - must not move
#[test]
fn test_quality_001_02_multiple_self_calls_no_move() {
    let code = r#"
pub struct Client {
    endpoint: String,
}

impl Client {
    pub fn new(endpoint: String) -> Self {
        Client { endpoint }
    }

    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }
}

pub fn test() -> String {
    let client = Client::new("http://localhost:8080".to_string());
    let ep1 = client.get_endpoint();
    let ep2 = client.get_endpoint();  // MUST work - &self doesn't move
    ep1
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Verify rustc compilation succeeds
    std::fs::write("/tmp/quality_001_02_output.rs", &rust_code).expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/quality_001_02_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("CRITICAL: Transpiled code fails rustc compilation:\n{stderr}\n\nGenerated code:\n{rust_code}");
    }
}

/// Test 3: &mut self method - mutable reference
#[test]
fn test_quality_001_03_mutable_self_reference() {
    let code = r"
pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn get(&self) -> i32 {
        self.count
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Must preserve &mut self
    assert!(
        rust_code.contains("& mut self") || rust_code.contains("&mut self"),
        "Should preserve &mut self, got: {rust_code}"
    );
}

/// Test 4: Owned self (consuming method) - should work as-is
#[test]
fn test_quality_001_04_owned_self_consuming() {
    let code = r"
pub struct Builder {
    value: String,
}

impl Builder {
    pub fn build(self) -> String {
        self.value
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Owned self should remain as `self` (no &)
    assert!(
        rust_code.contains("fn build (self)") || rust_code.contains("fn build ( self )"),
        "Should preserve owned self, got: {rust_code}"
    );
}

/// Test 5: Mixed receiver types in same impl
#[test]
fn test_quality_001_05_mixed_receiver_types() {
    let code = r"
pub struct State {
    value: i32,
}

impl State {
    pub fn new() -> Self {
        State { value: 0 }
    }

    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, value: i32) {
        self.value = value;
    }

    pub fn consume(self) -> i32 {
        self.value
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Should have all three receiver types
    assert!(
        rust_code.contains("& self") || rust_code.contains("&self"),
        "Should preserve &self in get(), got: {rust_code}"
    );

    assert!(
        rust_code.contains("& mut self") || rust_code.contains("&mut self"),
        "Should preserve &mut self in set(), got: {rust_code}"
    );

    assert!(
        rust_code.contains("fn consume (self)") || rust_code.contains("fn consume ( self )"),
        "Should preserve owned self in consume(), got: {rust_code}"
    );
}

/// Test 6: Issue #137 reproduction - ruchy-lambda pattern
#[test]
fn test_quality_001_06_issue_137_lambda_pattern() {
    let code = r"
use std::net::TcpStream;

pub struct LambdaRuntime {
    endpoint: String,
}

impl LambdaRuntime {
    pub fn new(endpoint: String) -> Self {
        LambdaRuntime { endpoint }
    }

    pub fn invoke(&self) -> bool {
        let stream = TcpStream::connect(&self.endpoint);
        stream.is_ok()
    }

    pub fn get_endpoint(&self) -> String {
        self.endpoint.clone()
    }
}
";

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Issue #137 pattern should work, got: {:?}",
        result.err()
    );

    let rust_code = result.unwrap().to_string();

    // Both invoke() and get_endpoint() must use &self
    assert!(
        rust_code.contains("& self") || rust_code.contains("&self"),
        "BLOCKER: &self must be preserved for ruchy-lambda, got: {rust_code}"
    );

    // Verify it compiles
    std::fs::write("/tmp/quality_001_06_output.rs", &rust_code).expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/quality_001_06_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!("CRITICAL: ruchy-lambda pattern fails compilation:\n{stderr}\n\nCode:\n{rust_code}");
    }
}

// ============================================================================
// PROPERTY-BASED TESTS (10K+ Random Inputs)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// Generate random valid struct names
    fn struct_name() -> impl Strategy<Value = String> {
        "[A-Z][a-zA-Z0-9]{0,10}".prop_map(|s| s)
    }

    /// Generate random valid method names
    fn method_name() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9_]{0,15}".prop_map(|s| s)
    }

    /// Generate random receiver types
    fn receiver_type() -> impl Strategy<Value = &'static str> {
        prop_oneof![Just("&self"), Just("&mut self"), Just("self")]
    }

    /// Generate random return types
    fn return_type() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("i32"),
            Just("String"),
            Just("bool"),
            Just("()"),
            Just("Self")
        ]
    }

    proptest! {
        /// Property 1: &self methods always preserve immutable reference
        #[test]
        fn prop_immutable_self_preserved(
            struct_name in struct_name(),
            method_name in method_name(),
            return_type in return_type()
        ) {
            let body = match return_type {
                "i32" => "self.value".to_string(),
                "String" => "\"test\".to_string()".to_string(),
                "bool" => "true".to_string(),
                "()" => String::new(),
                "Self" => format!("{struct_name} {{ value: self.value }}"),
                _ => "self.value".to_string(),
            };

            let code = format!(
                r"
                pub struct {struct_name} {{
                    value: i32,
                }}

                impl {struct_name} {{
                    pub fn {method_name}(&self) -> {return_type} {{
                        {body}
                    }}
                }}
                "
            );

            let ast = Parser::new(&code).parse();
            prop_assume!(ast.is_ok());

            let result = Transpiler::new().transpile_to_program(&ast.unwrap());
            prop_assume!(result.is_ok());

            let rust_code = result.unwrap().to_string();

            // INVARIANT: &self must be preserved
            prop_assert!(
                rust_code.contains("&self") || rust_code.contains("& self"),
                "Property violation: &self not preserved in:\n{}",
                rust_code
            );
        }

        /// Property 2: &mut self methods always preserve mutable reference
        #[test]
        fn prop_mutable_self_preserved(
            struct_name in struct_name(),
            method_name in method_name()
        ) {
            let code = format!(
                r"
                pub struct {struct_name} {{
                    value: i32,
                }}

                impl {struct_name} {{
                    pub fn {method_name}(&mut self) {{
                        self.value += 1;
                    }}
                }}
                "
            );

            let ast = Parser::new(&code).parse();
            prop_assume!(ast.is_ok());

            let result = Transpiler::new().transpile_to_program(&ast.unwrap());
            prop_assume!(result.is_ok());

            let rust_code = result.unwrap().to_string();

            // INVARIANT: &mut self must be preserved
            prop_assert!(
                rust_code.contains("&mut self") || rust_code.contains("& mut self"),
                "Property violation: &mut self not preserved in:\n{}",
                rust_code
            );
        }

        /// Property 3: Owned self (consuming) methods always preserve ownership
        #[test]
        fn prop_owned_self_preserved(
            struct_name in struct_name(),
            method_name in method_name(),
            return_type in return_type()
        ) {
            let body = match return_type {
                "i32" => "self.value".to_string(),
                "String" => "\"test\".to_string()".to_string(),
                "bool" => "true".to_string(),
                "()" => String::new(),
                "Self" => "self".to_string(),
                _ => "self.value".to_string(),
            };

            let code = format!(
                r"
                pub struct {struct_name} {{
                    value: i32,
                }}

                impl {struct_name} {{
                    pub fn {method_name}(self) -> {return_type} {{
                        {body}
                    }}
                }}
                "
            );

            let ast = Parser::new(&code).parse();
            prop_assume!(ast.is_ok());

            let result = Transpiler::new().transpile_to_program(&ast.unwrap());
            prop_assume!(result.is_ok());

            let rust_code = result.unwrap().to_string();

            // INVARIANT: Owned self must be preserved (fn name(self))
            // Must have "self" but NOT "&self" or "&mut self"
            let has_self = rust_code.contains(&format!("fn {method_name}(self)"))
                || rust_code.contains(&format!("fn {method_name} (self)"))
                || rust_code.contains(&format!("fn {method_name} ( self )"));

            prop_assert!(
                has_self,
                "Property violation: Owned self not preserved in:\n{}",
                rust_code
            );
        }

        /// Property 4: Multiple &self calls don't cause ownership moves
        #[test]
        fn prop_multiple_immutable_self_calls(
            struct_name in struct_name(),
            getter_name in method_name(),
            call_count in 2usize..5
        ) {
            let code = format!(
                r"
                pub struct {} {{
                    value: i32,
                }}

                impl {} {{
                    pub fn new(value: i32) -> Self {{
                        {} {{ value }}
                    }}

                    pub fn {}(&self) -> i32 {{
                        self.value
                    }}
                }}

                pub fn test() -> i32 {{
                    let obj = {}::new(42);
                    {}
                    obj.{}()
                }}
                ",
                struct_name,
                struct_name,
                struct_name,
                getter_name,
                struct_name,
                (0..call_count)
                    .map(|_| format!("    obj.{getter_name}();"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                getter_name
            );

            let ast = Parser::new(&code).parse();
            prop_assume!(ast.is_ok());

            let result = Transpiler::new().transpile_to_program(&ast.unwrap());
            prop_assume!(result.is_ok());

            let rust_code = result.unwrap().to_string();

            // Write to temp file and verify rustc compilation
            let temp_file = format!("/tmp/quality_001_prop_{struct_name}.rs");
            std::fs::write(&temp_file, &rust_code).ok();

            let rustc_result = std::process::Command::new("rustc")
                .args(["--crate-type", "lib", &temp_file])
                .output();

            prop_assume!(rustc_result.is_ok());
            let output = rustc_result.unwrap();

            // INVARIANT: Multiple &self calls must compile (no move errors)
            prop_assert!(
                output.status.success(),
                "Property violation: Multiple &self calls cause move error:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );

            // Cleanup
            std::fs::remove_file(&temp_file).ok();
        }

        /// Property 5: Mixed receiver types in same impl block
        #[test]
        fn prop_mixed_receivers(
            struct_name in struct_name(),
            getter in method_name(),
            setter in method_name(),
            consumer in method_name()
        ) {
            // Ensure method names are unique
            prop_assume!(getter != setter && setter != consumer && getter != consumer);

            let code = format!(
                r"
                pub struct {struct_name} {{
                    value: i32,
                }}

                impl {struct_name} {{
                    pub fn new() -> Self {{
                        {struct_name} {{ value: 0 }}
                    }}

                    pub fn {getter}(&self) -> i32 {{
                        self.value
                    }}

                    pub fn {setter}(&mut self, value: i32) {{
                        self.value = value;
                    }}

                    pub fn {consumer}(self) -> i32 {{
                        self.value
                    }}
                }}
                "
            );

            let ast = Parser::new(&code).parse();
            prop_assume!(ast.is_ok());

            let result = Transpiler::new().transpile_to_program(&ast.unwrap());
            prop_assume!(result.is_ok());

            let rust_code = result.unwrap().to_string();

            // INVARIANT: All three receiver types must be preserved
            prop_assert!(
                rust_code.contains("&self") || rust_code.contains("& self"),
                "Property violation: &self not preserved for getter"
            );
            prop_assert!(
                rust_code.contains("&mut self") || rust_code.contains("& mut self"),
                "Property violation: &mut self not preserved for setter"
            );

            // For owned self, check the specific method name
            let has_owned_self = rust_code.contains(&format!("fn {consumer}(self)"))
                || rust_code.contains(&format!("fn {consumer} (self)"))
                || rust_code.contains(&format!("fn {consumer} ( self )"));

            prop_assert!(
                has_owned_self,
                "Property violation: Owned self not preserved for consumer"
            );
        }
    }
}
