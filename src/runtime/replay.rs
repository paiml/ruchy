//! REPL Replay Testing System
//!
//! Provides deterministic replay capabilities for testing and educational assessment.
//! Based on docs/specifications/repl-replay-testing-spec.md
use crate::runtime::interpreter::Value;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
// ============================================================================
// Core Data Structures
// ============================================================================
/// Unique identifier for events in a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId(pub u64);
/// Semantic version for compatibility checking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    major: u32,
    minor: u32,
    patch: u32,
}
impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}
/// Complete REPL session recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplSession {
    pub version: SemVer,
    pub metadata: SessionMetadata,
    pub environment: Environment,
    pub timeline: Vec<TimestampedEvent>,
    pub checkpoints: BTreeMap<EventId, StateCheckpoint>,
}
/// Session metadata for tracking and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub created_at: String,
    pub ruchy_version: String,
    pub student_id: Option<String>,
    pub assignment_id: Option<String>,
    pub tags: Vec<String>,
}
/// Environment configuration for deterministic replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub seed: u64,
    pub feature_flags: Vec<String>,
    pub resource_limits: ResourceLimits,
}
/// Resource limits for bounded execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub heap_mb: usize,
    pub stack_kb: usize,
    pub cpu_ms: u64,
}
/// Timestamped event with causality tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedEvent {
    pub id: EventId,
    pub timestamp_ns: u64,
    pub event: Event,
    pub causality: Vec<EventId>, // Lamport clock for distributed replay
}
/// Event types that can occur in a REPL session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Input {
        text: String,
        mode: InputMode,
    },
    Output {
        result: EvalResult,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    StateChange {
        bindings_delta: HashMap<String, String>, // Simplified for now
        state_hash: String,
    },
    ResourceUsage {
        heap_bytes: usize,
        stack_depth: usize,
        cpu_ns: u64,
    },
}
/// Input modes for different interaction patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    Interactive,
    Paste,
    File,
    Script,
}
/// Result of evaluating an expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvalResult {
    Success { value: String },
    Error { message: String },
    Unit,
}
/// Complete state checkpoint for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateCheckpoint {
    pub bindings: HashMap<String, String>,
    pub type_environment: HashMap<String, String>,
    pub state_hash: String,
    pub resource_usage: ResourceUsage,
}
/// Resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub heap_bytes: usize,
    pub stack_depth: usize,
    pub cpu_ns: u64,
}
// ============================================================================
// Deterministic Execution
// ============================================================================
/// Result of deterministic replay execution
#[derive(Debug)]
pub struct ReplayResult {
    pub output: Result<Value>,
    pub state_hash: String,
    pub resource_usage: ResourceUsage,
}
/// Trait for deterministic REPL execution
pub trait DeterministicRepl {
    /// Execute with a fixed seed for deterministic behavior
    fn execute_with_seed(&mut self, input: &str, seed: u64) -> ReplayResult;
    /// Create a state checkpoint
    fn checkpoint(&self) -> StateCheckpoint;
    /// Restore from a checkpoint
    fn restore(&mut self, checkpoint: &StateCheckpoint) -> Result<()>;
    /// Validate determinism against another instance
    fn validate_determinism(&self, other: &Self) -> ValidationResult;
}
/// Result of determinism validation
#[derive(Debug)]
pub struct ValidationResult {
    pub is_deterministic: bool,
    pub divergences: Vec<Divergence>,
}
/// Types of divergence in replay
#[derive(Debug, Clone)]
pub enum Divergence {
    Output {
        expected: String,
        actual: String,
    },
    State {
        expected_hash: String,
        actual_hash: String,
    },
    Resources {
        expected: ResourceUsage,
        actual: ResourceUsage,
    },
}
// ============================================================================
// Replay Validation Engine
// ============================================================================
/// Validates replay sessions for correctness
pub struct ReplayValidator {
    pub strict_mode: bool,
    pub tolerance: ResourceTolerance,
}
/// Tolerance for resource usage variations
#[derive(Debug, Clone)]
pub struct ResourceTolerance {
    pub heap_bytes_percent: f64,
    pub cpu_ns_percent: f64,
}
impl Default for ResourceTolerance {
    fn default() -> Self {
        Self {
            heap_bytes_percent: 10.0, // Allow 10% variation
            cpu_ns_percent: 20.0,     // Allow 20% CPU variation
        }
    }
}
/// Report from validation
#[derive(Debug, Default)]
pub struct ValidationReport {
    pub passed: bool,
    pub total_events: usize,
    pub successful_events: usize,
    pub divergences: Vec<(EventId, Divergence)>,
}
impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_divergence(&mut self, event_id: EventId, divergence: Divergence) {
        self.divergences.push((event_id, divergence));
        self.passed = false;
    }
}
impl ReplayValidator {
    pub fn new(strict_mode: bool) -> Self {
        Self {
            strict_mode,
            tolerance: ResourceTolerance::default(),
        }
    }
    pub fn validate_session(
        &self,
        recorded: &ReplSession,
        implementation: &mut impl DeterministicRepl,
    ) -> ValidationReport {
        let mut report = ValidationReport::new();
        report.total_events = recorded.timeline.len();
        for event in &recorded.timeline {
            if let Event::Input { text, .. } = &event.event {
                let result = implementation.execute_with_seed(text, recorded.environment.seed);
                // Find corresponding output event
                if let Some(expected_output) = self.find_next_output(recorded, event.id) {
                    if self.outputs_equivalent(&result, expected_output) {
                        report.successful_events += 1;
                    } else {
                        report.add_divergence(
                            event.id,
                            Divergence::Output {
                                expected: format!("{expected_output:?}"),
                                actual: format!("{:?}", result.output),
                            },
                        );
                    }
                }
                // Validate resource bounds
                if !self.tolerance_accepts(&result.resource_usage) {
                    report.add_divergence(
                        event.id,
                        Divergence::Resources {
                            expected: ResourceUsage {
                                heap_bytes: 0,
                                stack_depth: 0,
                                cpu_ns: 0,
                            },
                            actual: result.resource_usage.clone(),
                        },
                    );
                }
            }
        }
        if report.divergences.is_empty() {
            report.passed = true;
        }
        report
    }
    fn find_next_output<'a>(
        &self,
        session: &'a ReplSession,
        after_id: EventId,
    ) -> Option<&'a Event> {
        session
            .timeline
            .iter()
            .find(|e| e.id > after_id && matches!(e.event, Event::Output { .. }))
            .map(|e| &e.event)
    }
    fn outputs_equivalent(&self, result: &ReplayResult, expected: &Event) -> bool {
        match expected {
            Event::Output {
                result: expected_result,
                ..
            } => match (&result.output, expected_result) {
                (Ok(value), EvalResult::Success { value: expected }) => {
                    format!("{value:?}") == *expected
                }
                (Err(e), EvalResult::Error { message }) => e.to_string().contains(message),
                _ => false,
            },
            _ => false,
        }
    }
    fn tolerance_accepts(&self, _usage: &ResourceUsage) -> bool {
        // For now, always accept - will implement proper bounds checking later
        true
    }
}
// ============================================================================
// Session Recording
// ============================================================================
/// Records REPL sessions for later replay
pub struct SessionRecorder {
    session: ReplSession,
    next_event_id: u64,
    start_time: Instant,
}
impl SessionRecorder {
    pub fn new(metadata: SessionMetadata) -> Self {
        use std::time::SystemTime;
        // Generate unique seed based on current time + session_id hash
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let session_hash = {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hasher::write(&mut hasher, metadata.session_id.as_bytes());
            std::hash::Hasher::finish(&hasher)
        };
        let unique_seed = now.wrapping_add(session_hash);
        Self {
            session: ReplSession {
                version: SemVer::new(1, 0, 0),
                metadata,
                environment: Environment {
                    seed: unique_seed,
                    feature_flags: vec![],
                    resource_limits: ResourceLimits {
                        heap_mb: 100,
                        stack_kb: 8192,
                        cpu_ms: 5000,
                    },
                },
                timeline: vec![],
                checkpoints: BTreeMap::new(),
            },
            next_event_id: 1,
            start_time: Instant::now(),
        }
    }
    pub fn record_input(&mut self, text: String, mode: InputMode) -> EventId {
        let id = EventId(self.next_event_id);
        self.next_event_id += 1;
        let event = TimestampedEvent {
            id,
            timestamp_ns: self.elapsed_ns(),
            event: Event::Input { text, mode },
            causality: vec![],
        };
        self.session.timeline.push(event);
        id
    }
    pub fn record_output(&mut self, result: Result<Value>) -> EventId {
        let id = EventId(self.next_event_id);
        self.next_event_id += 1;
        let eval_result = match result {
            Ok(Value::Nil) => EvalResult::Unit,
            Ok(value) => EvalResult::Success {
                value: format!("{value:?}"),
            },
            Err(e) => EvalResult::Error {
                message: e.to_string(),
            },
        };
        let event = TimestampedEvent {
            id,
            timestamp_ns: self.elapsed_ns(),
            event: Event::Output {
                result: eval_result,
                stdout: vec![],
                stderr: vec![],
            },
            causality: vec![],
        };
        self.session.timeline.push(event);
        id
    }
    pub fn add_checkpoint(&mut self, event_id: EventId, checkpoint: StateCheckpoint) {
        self.session.checkpoints.insert(event_id, checkpoint);
    }
    pub fn get_session(&self) -> &ReplSession {
        &self.session
    }
    pub fn into_session(self) -> ReplSession {
        self.session
    }
    fn elapsed_ns(&self) -> u64 {
        self.start_time.elapsed().as_nanos() as u64
    }
}
// ============================================================================
// Testing Utilities
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Helper function to create test metadata
    fn create_test_metadata() -> SessionMetadata {
        SessionMetadata {
            session_id: "test-001".to_string(),
            created_at: "2025-08-28T10:00:00Z".to_string(),
            ruchy_version: "1.23.0".to_string(),
            student_id: Some("student-123".to_string()),
            assignment_id: Some("assignment-456".to_string()),
            tags: vec!["test".to_string(), "development".to_string()],
        }
    }

    // Helper function to create test environment
    fn create_test_environment() -> Environment {
        Environment {
            seed: 12345,
            feature_flags: vec![],
            resource_limits: ResourceLimits {
                heap_mb: 1024,
                stack_kb: 256,
                cpu_ms: 5000,
            },
        }
    }

    // Helper function to create test checkpoint
    fn create_test_checkpoint() -> StateCheckpoint {
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), "42".to_string());
        bindings.insert("y".to_string(), "\"hello\"".to_string());

        let mut type_environment = HashMap::new();
        type_environment.insert("x".to_string(), "Int".to_string());
        type_environment.insert("y".to_string(), "String".to_string());

        StateCheckpoint {
            bindings,
            type_environment,
            state_hash: "abc123def456".to_string(),
            resource_usage: ResourceUsage {
                heap_bytes: 1024,
                stack_depth: 5,
                cpu_ns: 1000000,
            },
        }
    }

    // Tests for replay functionality
    #[cfg(test)]
    mod enabled_tests {
        use super::*;

        // Test 1: SemVer Creation and Equality
        #[test]
        fn test_semver_creation() {
            let version = SemVer::new(1, 2, 3);
            assert_eq!(version.major, 1);
            assert_eq!(version.minor, 2);
            assert_eq!(version.patch, 3);

            // Test equality
            let version2 = SemVer::new(1, 2, 3);
            assert_eq!(version, version2);

            // Test inequality
            let version3 = SemVer::new(1, 2, 4);
            assert_ne!(version, version3);
        }

        // Test 2: Session Metadata Creation and Serialization
        #[test]
        fn test_session_metadata() {
            let metadata = create_test_metadata();

            assert_eq!(metadata.session_id, "test-001");
            assert_eq!(metadata.ruchy_version, "1.23.0");
            assert_eq!(metadata.student_id, Some("student-123".to_string()));
            assert_eq!(metadata.assignment_id, Some("assignment-456".to_string()));
            assert_eq!(metadata.tags.len(), 2);
            assert!(metadata.tags.contains(&"test".to_string()));
            assert!(metadata.tags.contains(&"development".to_string()));

            // Test serialization/deserialization
            let json = serde_json::to_string(&metadata).unwrap();
            let deserialized: SessionMetadata = serde_json::from_str(&json).unwrap();
            assert_eq!(metadata.session_id, deserialized.session_id);
            assert_eq!(metadata.student_id, deserialized.student_id);
        }

        // Test 3: Environment Configuration
        #[test]
        fn test_environment() {
            let environment = create_test_environment();
            assert_eq!(environment.seed, 12345);
            assert!(environment.feature_flags.is_empty());
            assert_eq!(environment.resource_limits.heap_mb, 1024);

            // Test serialization
            let json = serde_json::to_string(&environment).unwrap();
            let deserialized: Environment = serde_json::from_str(&json).unwrap();
            assert_eq!(environment.seed, deserialized.seed);
        }

        // Test 4: Event ID Ordering and Comparison
        #[test]
        fn test_event_id_ordering() {
            let id1 = EventId(1);
            let id2 = EventId(2);
            let id3 = EventId(1);

            // Test ordering
            assert!(id1 < id2);
            assert!(id2 > id1);
            assert_eq!(id1, id3);

            // Test hash consistency
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(id1);
            set.insert(id2);
            set.insert(id3); // Should not add duplicate
            assert_eq!(set.len(), 2);
        }

        // Test 5: Resource Usage Tracking
        #[test]
        fn test_resource_usage() {
            let usage = ResourceUsage {
                heap_bytes: 2048,
                stack_depth: 10,
                cpu_ns: 5000000,
            };

            assert_eq!(usage.heap_bytes, 2048);
            assert_eq!(usage.stack_depth, 10);
            assert_eq!(usage.cpu_ns, 5000000);

            // Test serialization
            let json = serde_json::to_string(&usage).unwrap();
            let deserialized: ResourceUsage = serde_json::from_str(&json).unwrap();
            assert_eq!(usage.heap_bytes, deserialized.heap_bytes);
            assert_eq!(usage.stack_depth, deserialized.stack_depth);
            assert_eq!(usage.cpu_ns, deserialized.cpu_ns);
        }

        // Test 6: State Checkpoint Creation and Validation
        #[test]
        fn test_state_checkpoint() {
            let checkpoint = create_test_checkpoint();

            assert_eq!(checkpoint.bindings.len(), 2);
            assert_eq!(checkpoint.bindings.get("x"), Some(&"42".to_string()));
            assert_eq!(checkpoint.bindings.get("y"), Some(&"\"hello\"".to_string()));

            assert_eq!(checkpoint.type_environment.len(), 2);
            assert_eq!(
                checkpoint.type_environment.get("x"),
                Some(&"Int".to_string())
            );
            assert_eq!(
                checkpoint.type_environment.get("y"),
                Some(&"String".to_string())
            );

            assert_eq!(checkpoint.state_hash, "abc123def456");
            assert_eq!(checkpoint.resource_usage.heap_bytes, 1024);

            // Test serialization
            let json = serde_json::to_string(&checkpoint).unwrap();
            let deserialized: StateCheckpoint = serde_json::from_str(&json).unwrap();
            assert_eq!(checkpoint.state_hash, deserialized.state_hash);
            assert_eq!(checkpoint.bindings.len(), deserialized.bindings.len());
        }

        // Test 7: EvalResult Variants
        #[test]
        fn test_eval_result_variants() {
            // Test Success variant
            let success = EvalResult::Success {
                value: "42".to_string(),
            };
            match success {
                EvalResult::Success { ref value } => assert_eq!(value, "42"),
                _ => panic!("Expected Success variant"),
            }

            // Test Error variant
            let error = EvalResult::Error {
                message: "Division by zero".to_string(),
            };
            match error {
                EvalResult::Error { message } => assert_eq!(message, "Division by zero"),
                _ => panic!("Expected Error variant"),
            }

            // Test Unit variant
            let unit = EvalResult::Unit;
            match unit {
                EvalResult::Unit => {}
                _ => panic!("Expected Unit variant"),
            }

            // Test serialization
            let json = serde_json::to_string(&success).unwrap();
            let deserialized: EvalResult = serde_json::from_str(&json).unwrap();
            match deserialized {
                EvalResult::Success { value } => assert_eq!(value, "42"),
                _ => panic!("Deserialization failed"),
            }
        }

        // Test 8: Session Recorder Basic Operations
        #[test]
        fn test_session_recorder_basic() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Test initial state
            let session = recorder.get_session();
            assert_eq!(session.metadata.session_id, "test-001");
            assert_eq!(session.timeline.len(), 0);
            assert_eq!(session.checkpoints.len(), 0);

            // Test input recording
            let input_id = recorder.record_input("let x = 42".to_string(), InputMode::Interactive);
            assert_eq!(input_id, EventId(1));

            // Test output recording
            let output_id = recorder.record_output(Ok(Value::Nil));
            assert_eq!(output_id, EventId(2));

            // Verify timeline
            let session = recorder.get_session();
            assert_eq!(session.timeline.len(), 2);
        }

        // Test 9: Session Recorder Sequential Operations
        #[test]
        fn test_session_recorder_sequential() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Record sequence of operations
            let inputs = ["let x = 1", "let y = 2", "x + y", "println(x + y)"];

            let mut event_ids = Vec::new();
            for (i, input) in inputs.iter().enumerate() {
                let input_id = recorder.record_input((*input).to_string(), InputMode::Interactive);
                event_ids.push(input_id);
                assert_eq!(input_id, EventId((i as u64 * 2) + 1));

                let output_id = recorder.record_output(Ok(Value::Integer(i as i64 + 1)));
                event_ids.push(output_id);
                assert_eq!(output_id, EventId((i as u64 * 2) + 2));
            }

            // Verify all events recorded
            let session = recorder.get_session();
            assert_eq!(session.timeline.len(), 8); // 4 inputs + 4 outputs
            assert_eq!(event_ids.len(), 8);
        }

        // Test 10: Session Recorder with Checkpoints
        #[test]
        fn test_session_recorder_with_checkpoints() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Record some operations
            let _input_id1 =
                recorder.record_input("let x = 42".to_string(), InputMode::Interactive);
            let output_id1 = recorder.record_output(Ok(Value::Nil));

            // Add checkpoint after first operation
            let checkpoint1 = create_test_checkpoint();
            recorder.add_checkpoint(output_id1, checkpoint1);

            // Record more operations
            let _input_id2 = recorder.record_input("x * 2".to_string(), InputMode::Interactive);
            let output_id2 = recorder.record_output(Ok(Value::Integer(84)));

            // Add another checkpoint
            let mut checkpoint2 = create_test_checkpoint();
            checkpoint2.state_hash = "def456abc789".to_string();
            recorder.add_checkpoint(output_id2, checkpoint2);

            // Verify checkpoints
            let session = recorder.get_session();
            assert_eq!(session.checkpoints.len(), 2);
            assert!(session.checkpoints.contains_key(&output_id1));
            assert!(session.checkpoints.contains_key(&output_id2));

            let retrieved_checkpoint1 = session.checkpoints.get(&output_id1).unwrap();
            assert_eq!(retrieved_checkpoint1.state_hash, "abc123def456");

            let retrieved_checkpoint2 = session.checkpoints.get(&output_id2).unwrap();
            assert_eq!(retrieved_checkpoint2.state_hash, "def456abc789");
        }

        // Test 11: Session Recorder Input Modes
        #[test]
        fn test_input_modes() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Test different input modes
            let _interactive_id =
                recorder.record_input("2 + 2".to_string(), InputMode::Interactive);
            let _batch_id =
                recorder.record_input("let batch = true".to_string(), InputMode::Script);

            let session = recorder.get_session();
            assert_eq!(session.timeline.len(), 2);

            // Verify modes are stored correctly in timeline
            for event in &session.timeline {
                if let Event::Input { mode, .. } = &event.event {
                    assert!(matches!(mode, InputMode::Interactive | InputMode::Script));
                }
            }
        }

        // Test 12: Session Conversion and Ownership
        #[test]
        fn test_session_conversion() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Record some data
            recorder.record_input("test input".to_string(), InputMode::Interactive);
            recorder.record_output(Ok(Value::from_string("test output".to_string())));

            // Test borrowing session
            {
                let session_ref = recorder.get_session();
                assert_eq!(session_ref.metadata.session_id, "test-001");
                assert_eq!(session_ref.timeline.len(), 2);
            }

            // Test taking ownership
            let session = recorder.into_session();
            assert_eq!(session.metadata.session_id, "test-001");
            assert_eq!(session.timeline.len(), 2);
            assert_eq!(session.checkpoints.len(), 0);

            // Recorder should be consumed at this point
            // (can't test this directly, but the into_session() call above confirms it)
        }

        // Test 13: Validation Report Creation
        #[test]
        fn test_validation_report() {
            let mut report = ValidationReport::new();
            assert!(report.divergences.is_empty());

            // Add divergences
            let divergence1 = Divergence::Output {
                expected: "42".to_string(),
                actual: "43".to_string(),
            };

            let divergence2 = Divergence::State {
                expected_hash: "abc123".to_string(),
                actual_hash: "def456".to_string(),
            };

            report.add_divergence(EventId(1), divergence1);
            report.add_divergence(EventId(2), divergence2);

            assert_eq!(report.divergences.len(), 2);
            assert!(report.divergences.iter().any(|(id, _)| *id == EventId(1)));
            assert!(report.divergences.iter().any(|(id, _)| *id == EventId(2)));
        }

        // Test 14: Complex Session with Error Handling
        #[test]
        fn test_session_with_errors() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Record successful operation
            recorder.record_input("let x = 42".to_string(), InputMode::Interactive);
            recorder.record_output(Ok(Value::Nil));

            // Record failed operation
            recorder.record_input("x / 0".to_string(), InputMode::Interactive);
            recorder.record_output(Err(anyhow::anyhow!("Division by zero")));

            // Record recovery
            recorder.record_input("x / 2".to_string(), InputMode::Interactive);
            recorder.record_output(Ok(Value::Integer(21)));

            let session = recorder.get_session();
            assert_eq!(session.timeline.len(), 6); // 3 inputs + 3 outputs

            // Verify error is recorded properly in timeline
            let error_output = &session.timeline[3]; // Second output (index 3)
            match &error_output.event {
                Event::Output { result, .. } => {
                    match result {
                        EvalResult::Error { .. } => {} // Expected error
                        _ => panic!("Expected error result"),
                    }
                }
                _ => panic!("Expected output event"),
            }
        }

        // Test 15: Resource Tolerance and Validation
        #[test]
        fn test_resource_tolerance() {
            let tolerance = ResourceTolerance::default();

            // Default values should be reasonable
            assert!(tolerance.heap_bytes_percent > 0.0);
            assert!(tolerance.cpu_ns_percent > 0.0);

            // Create custom tolerance
            let custom_tolerance = ResourceTolerance {
                heap_bytes_percent: 15.0,
                cpu_ns_percent: 25.0,
            };

            assert_eq!(custom_tolerance.heap_bytes_percent, 15.0);
            assert_eq!(custom_tolerance.cpu_ns_percent, 25.0);
        }

        // Test 16: Complete Session Serialization
        #[test]
        fn test_complete_session_serialization() {
            let metadata = create_test_metadata();
            let mut recorder = SessionRecorder::new(metadata);

            // Create a complete session
            recorder.record_input("let x = 42".to_string(), InputMode::Interactive);
            recorder.record_output(Ok(Value::Integer(42)));

            let checkpoint = create_test_checkpoint();
            recorder.add_checkpoint(EventId(2), checkpoint);

            let session = recorder.into_session();

            // Test full session serialization
            let json = serde_json::to_string(&session).unwrap();
            let deserialized: ReplSession = serde_json::from_str(&json).unwrap();

            assert_eq!(
                session.metadata.session_id,
                deserialized.metadata.session_id
            );
            assert_eq!(session.timeline.len(), deserialized.timeline.len());
            assert_eq!(session.checkpoints.len(), deserialized.checkpoints.len());
            assert_eq!(session.environment.seed, deserialized.environment.seed);
            assert_eq!(session.version, deserialized.version);
        }
    } // End enabled_tests module
}
