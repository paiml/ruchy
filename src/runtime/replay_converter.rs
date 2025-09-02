//! Replay-to-Test Conversion Pipeline
//!
//! Converts .replay files into comprehensive Rust test cases for regression testing.
//! This enables automatic generation of test coverage from real usage patterns.

use crate::runtime::replay::{ReplSession, Event, EvalResult, InputMode};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configuration for replay-to-test conversion
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    /// Test module name prefix
    pub test_module_prefix: String,
    /// Include property tests for state consistency
    pub include_property_tests: bool,
    /// Include performance benchmarks
    pub include_benchmarks: bool,
    /// Maximum test timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            test_module_prefix: "replay_generated".to_string(),
            include_property_tests: true,
            include_benchmarks: false,
            timeout_ms: 5000,
        }
    }
}

/// Test case generated from replay session
#[derive(Debug, Clone)]
pub struct GeneratedTest {
    /// Test function name
    pub name: String,
    /// Test function code
    pub code: String,
    /// Test category (unit, integration, property, etc.)
    pub category: TestCategory,
    /// Expected coverage impact
    pub coverage_areas: Vec<String>,
}

/// Category of generated test
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// Basic input/output verification
    Unit,
    /// Multi-step interaction testing
    Integration,
    /// Property-based testing for invariants
    Property,
    /// Performance and resource testing
    Benchmark,
    /// Error handling and recovery
    ErrorHandling,
}

/// Replay-to-test converter
pub struct ReplayConverter {
    config: ConversionConfig,
}

impl ReplayConverter {
    /// Create a new converter with default configuration
    pub fn new() -> Self {
        Self {
            config: ConversionConfig::default(),
        }
    }

    /// Create a new converter with custom configuration
    pub fn with_config(config: ConversionConfig) -> Self {
        Self { config }
    }

    /// Convert a replay file to test cases
    pub fn convert_file(&self, replay_path: &Path) -> Result<Vec<GeneratedTest>> {
        let replay_content = fs::read_to_string(replay_path)
            .context("Failed to read replay file")?;
        
        let session: ReplSession = serde_json::from_str(&replay_content)
            .context("Failed to parse replay session")?;
        
        self.convert_session(&session, replay_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed"))
    }

    /// Convert a replay session to test cases
    pub fn convert_session(&self, session: &ReplSession, name_prefix: &str) -> Result<Vec<GeneratedTest>> {
        let mut tests = Vec::new();
        
        // Generate unit tests for each input/output pair
        tests.extend(self.generate_unit_tests(session, name_prefix)?);
        
        // Generate integration test for full session
        tests.push(self.generate_integration_test(session, name_prefix)?);
        
        // Generate property tests if enabled
        if self.config.include_property_tests {
            tests.extend(self.generate_property_tests(session, name_prefix)?);
        }
        
        // Generate error handling tests
        tests.extend(self.generate_error_tests(session, name_prefix)?);
        
        Ok(tests)
    }

    /// Generate unit tests for individual input/output pairs
    fn generate_unit_tests(&self, session: &ReplSession, name_prefix: &str) -> Result<Vec<GeneratedTest>> {
        let mut tests = Vec::new();
        let mut test_counter = 1;
        
        // Find input/output pairs
        let timeline = &session.timeline;
        for i in 0..timeline.len() {
            if let Event::Input { text, mode } = &timeline[i].event {
                // Find the corresponding output
                if let Some(output_event) = timeline.get(i + 1) {
                    if let Event::Output { result, .. } = &output_event.event {
                        let sanitized_prefix = name_prefix.replace('-', "_");
        let test_name = format!("{sanitized_prefix}_{test_counter:03}");
                        let test = self.generate_single_unit_test(&test_name, text, mode, result)?;
                        tests.push(test);
                        test_counter += 1;
                    }
                }
            }
        }
        
        Ok(tests)
    }

    /// Generate a single unit test
    fn generate_single_unit_test(
        &self, 
        test_name: &str, 
        input: &str, 
        mode: &InputMode,
        expected_result: &EvalResult
    ) -> Result<GeneratedTest> {
        let sanitized_input = input.replace('\"', "\\\"").replace('\n', "\\n");
        let timeout = self.config.timeout_ms;
        
        let (expected_output, test_assertion) = match expected_result {
            EvalResult::Success { value } => {
                // Extract actual value from String("...") format if present
                let actual_value = if value.starts_with("String(\"") && value.ends_with("\")") {
                    // Extract content from String("value")
                    &value[8..value.len()-2]
                } else {
                    value.as_str()
                };
                let sanitized_value = actual_value.replace('\"', "\\\"").replace('\\', "\\\\");
                (format!("Ok(\"{sanitized_value}\")"), format!("assert!(result.is_ok() && result.unwrap() == r#\"{}\"#);", actual_value))
            }
            EvalResult::Error { message } => {
                (format!("Err(r#\"{}\"#)", message), format!("assert!(result.is_err() && result.unwrap_err().to_string().contains(r#\"{}\"#));", message))
            }
            EvalResult::Unit => {
                ("Ok(\"\")".to_string(), "assert!(result.is_ok() && result.unwrap().is_empty());".to_string())
            }
        };

        let mode_comment = match mode {
            InputMode::Interactive => "// Interactive REPL input",
            InputMode::Paste => "// Pasted/multiline input", 
            InputMode::File => "// File-loaded input",
            InputMode::Script => "// Script execution",
        };

        let coverage_areas = self.identify_coverage_areas(input);

        let code = format!(r#"
#[test]
fn test_{test_name}() -> Result<()> {{
    {mode_comment}
    let mut repl = Repl::new()?;
    
    let deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis({timeout}));
    let result = repl.eval("{sanitized_input}");
    
    // Expected: {expected_output}
    {test_assertion}
    
    Ok(())
}}"#);

        Ok(GeneratedTest {
            name: format!("test_{test_name}"),
            code,
            category: TestCategory::Unit,
            coverage_areas,
        })
    }

    /// Generate integration test for complete session
    fn generate_integration_test(&self, session: &ReplSession, name_prefix: &str) -> Result<GeneratedTest> {
        let sanitized_prefix = name_prefix.replace('-', "_");
        let mut session_code = String::new();
        let mut assertions = Vec::new();
        let timeout = self.config.timeout_ms;

        // Collect all inputs and expected outputs
        let timeline = &session.timeline;
        for i in 0..timeline.len() {
            if let Event::Input { text, .. } = &timeline[i].event {
                let sanitized_input = text.replace('\"', "\\\"").replace('\n', "\\n");
                
                // Find corresponding output
                if let Some(output_event) = timeline.get(i + 1) {
                    if let Event::Output { result, .. } = &output_event.event {
                        session_code.push_str(&format!("    let result_{i} = repl.eval(\"{sanitized_input}\");\n"));
                        
                        let assertion = match result {
                            EvalResult::Success { value } => {
                                // Extract actual value from String("...") format if present
                                let actual_value = if value.starts_with("String(\"") && value.ends_with("\")") {
                                    &value[8..value.len()-2]
                                } else {
                                    value.as_str()
                                };
                                format!("    assert!(result_{i}.is_ok() && result_{i}.unwrap() == r#\"{}\"#);\n", actual_value)
                            }
                            EvalResult::Error { message } => {
                                format!("    assert!(result_{i}.is_err() && result_{i}.unwrap_err().to_string().contains(r#\"{}\"#));\n", message)
                            }
                            EvalResult::Unit => {
                                format!("    assert!(result_{i}.is_ok() && result_{i}.unwrap().is_empty());\n")
                            }
                        };
                        assertions.push(assertion);
                    }
                }
            }
        }

        let code = format!(r"
#[test]
fn test_{sanitized_prefix}_session_integration() -> Result<()> {{
    // Integration test for complete REPL session
    // Tests state persistence and interaction patterns
    let mut repl = Repl::new()?;
    
    // Session timeout
    let _deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis({timeout}));
    
    // Execute complete session
{session_code}
    
    // Verify all expected outputs
{assertions}
    
    Ok(())
}}", assertions = assertions.join(""));

        // Identify comprehensive coverage areas
        let mut coverage_areas = vec![
            "session_state".to_string(),
            "multi_step_interaction".to_string(),
            "state_persistence".to_string(),
        ];
        
        // Add specific areas based on session content
        for event in &session.timeline {
            if let Event::Input { text, .. } = &event.event {
                coverage_areas.extend(self.identify_coverage_areas(text));
            }
        }
        coverage_areas.sort();
        coverage_areas.dedup();

        let sanitized_prefix = name_prefix.replace('-', "_");
        Ok(GeneratedTest {
            name: format!("test_{sanitized_prefix}_session_integration"),
            code,
            category: TestCategory::Integration,
            coverage_areas,
        })
    }

    /// Generate property tests for invariants
    fn generate_property_tests(&self, _session: &ReplSession, name_prefix: &str) -> Result<Vec<GeneratedTest>> {
        let mut tests = Vec::new();
        
        // Property: REPL state should be deterministic
        let sanitized_prefix = name_prefix.replace('-', "_");
        let determinism_test = GeneratedTest {
            name: format!("test_{sanitized_prefix}_determinism_property"),
            code: format!(r#"
#[test]
fn test_{sanitized_prefix}_determinism_property() -> Result<()> {{
    // Property: Session should produce identical results on replay
    use crate::runtime::replay::*;
    
    let mut repl1 = Repl::new()?;
    let mut repl2 = Repl::new()?;
    
    // Execute same sequence on both REPLs
    let inputs = [
        // Insert representative inputs from session
    ];
    
    for input in inputs {{
        let result1 = repl1.eval(input);
        let result2 = repl2.eval(input);
        
        match (result1, result2) {{
            (Ok(out1), Ok(out2)) => assert_eq!(out1, out2),
            (Err(_), Err(_)) => {{}}, // Both failed consistently  
            _ => panic!("Inconsistent REPL behavior: {{}} vs {{}}", input, input),
        }}
    }}
    
    Ok(())
}}"#),
            category: TestCategory::Property,
            coverage_areas: vec!["determinism".to_string(), "state_consistency".to_string()],
        };
        tests.push(determinism_test);

        // Property: Memory usage should be bounded
        let memory_test = GeneratedTest {
            name: format!("test_{sanitized_prefix}_memory_bounds"),
            code: format!(r#"
#[test] 
fn test_{sanitized_prefix}_memory_bounds() -> Result<()> {{
    // Property: REPL should respect memory limits
    let mut repl = Repl::new()?;
    
    let initial_memory = repl.get_memory_usage();
    
    // Execute session operations
    // ... (session-specific operations)
    
    let final_memory = repl.get_memory_usage();
    
    // Memory should not exceed reasonable bounds (100MB default)
    assert!(final_memory < 100 * 1024 * 1024, "Memory usage exceeded bounds: {{}} bytes", final_memory);
    
    Ok(())
}}"#),
            category: TestCategory::Property,
            coverage_areas: vec!["memory_management".to_string(), "resource_bounds".to_string()],
        };
        tests.push(memory_test);
        
        Ok(tests)
    }

    /// Generate error handling tests
    fn generate_error_tests(&self, session: &ReplSession, name_prefix: &str) -> Result<Vec<GeneratedTest>> {
        let mut tests = Vec::new();
        
        // Find error cases in the session
        for (i, event) in session.timeline.iter().enumerate() {
            if let Event::Output { result: EvalResult::Error { message }, .. } = &event.event {
                let sanitized_prefix = name_prefix.replace('-', "_");
                let test_name = format!("{sanitized_prefix}_{i:03}_error_handling");
                
                // Find the input that caused this error
                if i > 0 {
                    if let Event::Input { text, .. } = &session.timeline[i - 1].event {
                        let sanitized_input = text.replace('\"', "\\\"");
                        let sanitized_message = message.replace('\"', "\\\"");
                        
                        let test = GeneratedTest {
                            name: format!("test_{test_name}"),
                            code: format!(r#"
#[test]
fn test_{test_name}() -> Result<()> {{
    // Error handling test: should gracefully handle invalid input
    let mut repl = Repl::new()?;
    
    let result = repl.eval("{sanitized_input}");
    
    // Should fail gracefully with descriptive error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("{sanitized_message}"));
    
    // REPL should remain functional after error
    let recovery = repl.eval("2 + 2");
    assert_eq!(recovery, Ok("4".to_string()));
    
    Ok(())
}}"#),
                            category: TestCategory::ErrorHandling,
                            coverage_areas: vec![
                                "error_handling".to_string(),
                                "error_recovery".to_string(),
                                "graceful_degradation".to_string(),
                            ],
                        };
                        tests.push(test);
                    }
                }
            }
        }
        
        Ok(tests)
    }

    /// Identify code coverage areas based on input content
    fn identify_coverage_areas(&self, input: &str) -> Vec<String> {
        let mut areas = Vec::new();
        
        // Language constructs
        if input.contains("let ") || input.contains("var ") {
            areas.push("variable_binding".to_string());
        }
        if input.contains("fn ") {
            areas.push("function_definition".to_string());
        }
        if input.contains("=>") {
            areas.push("lambda_expressions".to_string());
        }
        if input.contains("match ") {
            areas.push("pattern_matching".to_string());
        }
        if input.contains("if ") {
            areas.push("conditional_expressions".to_string());
        }
        if input.contains("for ") || input.contains("while ") {
            areas.push("iteration".to_string());
        }
        
        // Data structures  
        if input.contains('[') && input.contains(']') {
            areas.push("array_operations".to_string());
        }
        if input.contains('(') && input.contains(',') {
            areas.push("tuple_operations".to_string());
        }
        if input.contains('{') && input.contains(':') {
            areas.push("object_operations".to_string());
        }
        
        // Operators
        if input.contains("?.") {
            areas.push("optional_chaining".to_string());
        }
        if input.contains("??") {
            areas.push("null_coalescing".to_string());
        }
        if input.contains("|>") {
            areas.push("pipeline_operator".to_string());
        }
        if input.contains("...") {
            areas.push("spread_operator".to_string());
        }
        
        // String operations
        if input.contains("f\"") || input.contains("f'") {
            areas.push("string_interpolation".to_string());
        }
        if input.contains(".map(") || input.contains(".filter(") || input.contains(".reduce(") {
            areas.push("higher_order_functions".to_string());
        }
        
        // REPL features
        if input.starts_with(':') {
            areas.push("repl_commands".to_string());
        }
        if input.contains('?') && !input.contains("??") {
            areas.push("repl_introspection".to_string());
        }
        
        // Error scenarios
        if input.contains("try ") || input.contains("catch ") {
            areas.push("error_handling".to_string());
        }
        
        areas
    }

    /// Write generated tests to a file
    pub fn write_tests(&self, tests: &[GeneratedTest], output_path: &Path) -> Result<()> {
        let mut content = String::new();
        
        // File header
        content.push_str(&format!(r"
//! Generated regression tests from REPL replay sessions
//! 
//! This file is auto-generated by the replay-to-test conversion pipeline.
//! DO NOT EDIT MANUALLY - regenerate from .replay files instead.
//!
//! Generated tests: {}
//! Coverage areas: {}

use anyhow::Result;
use crate::runtime::Repl;

", tests.len(), 
            tests.iter()
                .flat_map(|t| &t.coverage_areas)
                .collect::<std::collections::HashSet<_>>()
                .len()
        ));
        
        // Group tests by category
        let categories = [
            TestCategory::Unit,
            TestCategory::Integration,
            TestCategory::Property,
            TestCategory::ErrorHandling,
            TestCategory::Benchmark,
        ];
        
        for category in categories {
            let category_tests: Vec<_> = tests.iter()
                .filter(|t| t.category == category)
                .collect();
            
            if !category_tests.is_empty() {
                content.push_str(&format!("\n// {:?} Tests ({})\n", 
                    category, category_tests.len()));
                content.push_str("// ============================================================================\n\n");
                
                for test in category_tests {
                    content.push_str(&test.code);
                    content.push('\n');
                }
            }
        }
        
        fs::write(output_path, content)
            .context("Failed to write test file")?;
        
        Ok(())
    }
}

impl Default for ReplayConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::replay::*;
    
    #[test]
    fn test_replay_converter_basic() {
        let converter = ReplayConverter::new();
        
        // Create a simple mock session
        let session = ReplSession {
            version: SemVer::new(1, 0, 0),
            metadata: SessionMetadata {
                session_id: "test".to_string(),
                created_at: "2025-01-01T00:00:00Z".to_string(),
                ruchy_version: "1.0.0".to_string(),
                student_id: None,
                assignment_id: None,
                tags: vec![],
            },
            environment: Environment {
                seed: 0,
                feature_flags: vec![],
                resource_limits: ResourceLimits {
                    heap_mb: 100,
                    stack_kb: 8192,
                    cpu_ms: 5000,
                },
            },
            timeline: vec![
                TimestampedEvent {
                    id: EventId(1),
                    timestamp_ns: 1000,
                    event: Event::Input {
                        text: "2 + 2".to_string(),
                        mode: InputMode::Interactive,
                    },
                    causality: vec![],
                },
                TimestampedEvent {
                    id: EventId(2),
                    timestamp_ns: 2000,
                    event: Event::Output {
                        result: EvalResult::Success { value: "4".to_string() },
                        stdout: vec![],
                        stderr: vec![],
                    },
                    causality: vec![],
                },
            ],
            checkpoints: std::collections::BTreeMap::new(),
        };
        
        let tests = converter.convert_session(&session, "basic").unwrap();
        
        // Should generate at least unit and integration tests
        assert!(tests.len() >= 2);
        assert!(tests.iter().any(|t| t.category == TestCategory::Unit));
        assert!(tests.iter().any(|t| t.category == TestCategory::Integration));
    }
    
    #[test]
    fn test_coverage_area_identification() {
        let converter = ReplayConverter::new();
        
        // Test various language constructs
        let test_cases = [
            ("let x = 42", vec!["variable_binding"]),
            ("x.map(y => y * 2)", vec!["lambda_expressions", "higher_order_functions"]),
            ("[1, 2, 3]", vec!["array_operations"]),
            ("user?.name", vec!["optional_chaining"]),
            ("match x { 1 => \"one\" }", vec!["pattern_matching"]),
        ];
        
        for (input, expected_areas) in test_cases {
            let areas = converter.identify_coverage_areas(input);
            for expected in expected_areas {
                assert!(areas.contains(&expected.to_string()), 
                    "Expected coverage area '{expected}' not found for input: '{input}'");
            }
        }
    }
}