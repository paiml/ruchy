//! TDD tests for runtime/replay.rs - achieving 90%+ coverage
//! QDD Metrics Target:
//! - Line Coverage: ≥90%
//! - Branch Coverage: ≥85%  
//! - All public APIs: 100%

use ruchy::runtime::replay::*;
use ruchy::runtime::repl::Value;
use std::collections::{HashMap, BTreeMap};

// ============================================================================
// Core Data Structure Tests
// ============================================================================

#[test]
fn test_event_id_creation() {
    let id1 = EventId(1);
    let id2 = EventId(2);
    assert_eq!(id1, EventId(1));
    assert_ne!(id1, id2);
    assert!(id1 < id2);
}

#[test]
fn test_event_id_ordering() {
    let mut ids = vec![EventId(3), EventId(1), EventId(2)];
    ids.sort();
    assert_eq!(ids, vec![EventId(1), EventId(2), EventId(3)]);
}

#[test]
fn test_semver_creation() {
    let ver = SemVer::new(1, 2, 3);
    // Can't assert fields directly as they're private, but can test equality
    let ver2 = SemVer::new(1, 2, 3);
    assert_eq!(ver, ver2);
}

#[test]
fn test_semver_equality() {
    let v1 = SemVer::new(1, 0, 0);
    let v2 = SemVer::new(1, 0, 0);
    let v3 = SemVer::new(2, 0, 0);
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

#[test]
fn test_input_mode_variants() {
    assert_eq!(InputMode::Interactive, InputMode::Interactive);
    assert_ne!(InputMode::Interactive, InputMode::Paste);
    assert_ne!(InputMode::File, InputMode::Script);
}

#[test]
fn test_eval_result_success() {
    let result = EvalResult::Success { value: "42".to_string() };
    match result {
        EvalResult::Success { value } => assert_eq!(value, "42"),
        _ => panic!("Expected Success"),
    }
}

#[test]
fn test_eval_result_error() {
    let result = EvalResult::Error { message: "Division by zero".to_string() };
    match result {
        EvalResult::Error { message } => assert!(message.contains("Division")),
        _ => panic!("Expected Error"),
    }
}

#[test]
fn test_eval_result_unit() {
    let result = EvalResult::Unit;
    assert!(matches!(result, EvalResult::Unit));
}

#[test]
fn test_resource_limits_creation() {
    let limits = ResourceLimits {
        heap_mb: 256,
        stack_kb: 8192,
        cpu_ms: 5000,
    };
    assert_eq!(limits.heap_mb, 256);
    assert_eq!(limits.stack_kb, 8192);
    assert_eq!(limits.cpu_ms, 5000);
}

#[test]
fn test_resource_usage_creation() {
    let usage = ResourceUsage {
        heap_bytes: 1024 * 1024,
        stack_depth: 42,
        cpu_ns: 1_000_000,
    };
    assert_eq!(usage.heap_bytes, 1024 * 1024);
    assert_eq!(usage.stack_depth, 42);
    assert_eq!(usage.cpu_ns, 1_000_000);
}

#[test]
fn test_environment_creation() {
    let env = Environment {
        seed: 12345,
        feature_flags: vec!["async".to_string(), "dataframes".to_string()],
        resource_limits: ResourceLimits {
            heap_mb: 512,
            stack_kb: 4096,
            cpu_ms: 10000,
        },
    };
    assert_eq!(env.seed, 12345);
    assert_eq!(env.feature_flags.len(), 2);
    assert!(env.feature_flags.contains(&"async".to_string()));
}

#[test]
fn test_session_metadata() {
    let metadata = SessionMetadata {
        session_id: "test-123".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: Some("student1".to_string()),
        assignment_id: None,
        tags: vec!["test".to_string()],
    };
    assert_eq!(metadata.session_id, "test-123");
    assert!(metadata.student_id.is_some());
    assert!(metadata.assignment_id.is_none());
}

// ============================================================================
// Event Tests
// ============================================================================

#[test]
fn test_event_input() {
    let event = Event::Input {
        text: "let x = 42".to_string(),
        mode: InputMode::Interactive,
    };
    match event {
        Event::Input { text, mode } => {
            assert_eq!(text, "let x = 42");
            assert_eq!(mode, InputMode::Interactive);
        }
        _ => panic!("Expected Input event"),
    }
}

#[test]
fn test_event_output() {
    let event = Event::Output {
        result: EvalResult::Success { value: "42".to_string() },
        stdout: vec![72, 101, 108, 108, 111], // "Hello"
        stderr: vec![],
    };
    match event {
        Event::Output { result, stdout, stderr } => {
            assert!(matches!(result, EvalResult::Success { .. }));
            assert_eq!(stdout.len(), 5);
            assert!(stderr.is_empty());
        }
        _ => panic!("Expected Output event"),
    }
}

#[test]
fn test_event_state_change() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), "42".to_string());
    
    let event = Event::StateChange {
        bindings_delta: bindings.clone(),
        state_hash: "abc123".to_string(),
    };
    
    match event {
        Event::StateChange { bindings_delta, state_hash } => {
            assert_eq!(bindings_delta.len(), 1);
            assert_eq!(bindings_delta.get("x").unwrap(), "42");
            assert_eq!(state_hash, "abc123");
        }
        _ => panic!("Expected StateChange event"),
    }
}

#[test]
fn test_event_resource_usage() {
    let event = Event::ResourceUsage {
        heap_bytes: 1024,
        stack_depth: 10,
        cpu_ns: 500_000,
    };
    match event {
        Event::ResourceUsage { heap_bytes, stack_depth, cpu_ns } => {
            assert_eq!(heap_bytes, 1024);
            assert_eq!(stack_depth, 10);
            assert_eq!(cpu_ns, 500_000);
        }
        _ => panic!("Expected ResourceUsage event"),
    }
}

#[test]
fn test_timestamped_event() {
    let event = TimestampedEvent {
        id: EventId(1),
        timestamp_ns: 1_000_000_000,
        event: Event::Input {
            text: "test".to_string(),
            mode: InputMode::Interactive,
        },
        causality: vec![],
    };
    assert_eq!(event.id, EventId(1));
    assert_eq!(event.timestamp_ns, 1_000_000_000);
    assert!(event.causality.is_empty());
}

#[test]
fn test_timestamped_event_with_causality() {
    let event = TimestampedEvent {
        id: EventId(3),
        timestamp_ns: 3_000_000_000,
        event: Event::Output {
            result: EvalResult::Unit,
            stdout: vec![],
            stderr: vec![],
        },
        causality: vec![EventId(1), EventId(2)],
    };
    assert_eq!(event.causality.len(), 2);
    assert!(event.causality.contains(&EventId(1)));
}

// ============================================================================
// State Checkpoint Tests
// ============================================================================

#[test]
fn test_state_checkpoint_creation() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), "Int(42)".to_string());
    
    let mut types = HashMap::new();
    types.insert("x".to_string(), "Int".to_string());
    
    let checkpoint = StateCheckpoint {
        bindings,
        type_environment: types,
        state_hash: "hash123".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 1024,
            stack_depth: 5,
            cpu_ns: 100_000,
        },
    };
    
    assert_eq!(checkpoint.bindings.len(), 1);
    assert_eq!(checkpoint.type_environment.len(), 1);
    assert_eq!(checkpoint.state_hash, "hash123");
}

// ============================================================================
// Session Tests
// ============================================================================

#[test]
fn test_repl_session_creation() {
    let session = ReplSession {
        version: SemVer::new(1, 0, 0),
        metadata: SessionMetadata {
            session_id: "test".to_string(),
            created_at: "2024-01-01".to_string(),
            ruchy_version: "1.0.0".to_string(),
            student_id: None,
            assignment_id: None,
            tags: vec![],
        },
        environment: Environment {
            seed: 0,
            feature_flags: vec![],
            resource_limits: ResourceLimits {
                heap_mb: 128,
                stack_kb: 1024,
                cpu_ms: 1000,
            },
        },
        timeline: vec![],
        checkpoints: BTreeMap::new(),
    };
    
    assert_eq!(session.version, SemVer::new(1, 0, 0));
    assert!(session.timeline.is_empty());
    assert!(session.checkpoints.is_empty());
}

#[test]
fn test_repl_session_with_timeline() {
    let timeline = vec![
        TimestampedEvent {
            id: EventId(1),
            timestamp_ns: 1000,
            event: Event::Input {
                text: "1 + 1".to_string(),
                mode: InputMode::Interactive,
            },
            causality: vec![],
        },
        TimestampedEvent {
            id: EventId(2),
            timestamp_ns: 2000,
            event: Event::Output {
                result: EvalResult::Success { value: "2".to_string() },
                stdout: vec![],
                stderr: vec![],
            },
            causality: vec![EventId(1)],
        },
    ];
    
    let session = ReplSession {
        version: SemVer::new(1, 0, 0),
        metadata: SessionMetadata {
            session_id: "test".to_string(),
            created_at: "2024-01-01".to_string(),
            ruchy_version: "1.0.0".to_string(),
            student_id: None,
            assignment_id: None,
            tags: vec![],
        },
        environment: Environment {
            seed: 0,
            feature_flags: vec![],
            resource_limits: ResourceLimits {
                heap_mb: 128,
                stack_kb: 1024,
                cpu_ms: 1000,
            },
        },
        timeline,
        checkpoints: BTreeMap::new(),
    };
    
    assert_eq!(session.timeline.len(), 2);
    assert_eq!(session.timeline[0].id, EventId(1));
    assert_eq!(session.timeline[1].id, EventId(2));
}

// ============================================================================
// Replay Result Tests
// ============================================================================

#[test]
fn test_replay_result_success() {
    let result = ReplayResult {
        output: Ok(Value::Int(42)),
        state_hash: "hash42".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 512,
            stack_depth: 3,
            cpu_ns: 50_000,
        },
    };
    
    assert!(result.output.is_ok());
    assert_eq!(result.state_hash, "hash42");
    assert_eq!(result.resource_usage.heap_bytes, 512);
}

#[test]
fn test_replay_result_error() {
    let result = ReplayResult {
        output: Err(anyhow::anyhow!("Test error")),
        state_hash: "error_hash".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 256,
            stack_depth: 2,
            cpu_ns: 10_000,
        },
    };
    
    assert!(result.output.is_err());
    assert_eq!(result.state_hash, "error_hash");
}

// ============================================================================
// Divergence Tests
// ============================================================================

#[test]
fn test_divergence_output() {
    let div = Divergence::Output {
        expected: "42".to_string(),
        actual: "43".to_string(),
    };
    
    match div {
        Divergence::Output { expected, actual } => {
            assert_eq!(expected, "42");
            assert_eq!(actual, "43");
        }
        _ => panic!("Expected Output divergence"),
    }
}

#[test]
fn test_divergence_state() {
    let div = Divergence::State {
        expected_hash: "abc".to_string(),
        actual_hash: "def".to_string(),
    };
    
    match div {
        Divergence::State { expected_hash, actual_hash } => {
            assert_eq!(expected_hash, "abc");
            assert_eq!(actual_hash, "def");
        }
        _ => panic!("Expected State divergence"),
    }
}

#[test]
fn test_divergence_resources() {
    let div = Divergence::Resources {
        expected: ResourceUsage {
            heap_bytes: 1024,
            stack_depth: 5,
            cpu_ns: 100_000,
        },
        actual: ResourceUsage {
            heap_bytes: 2048,
            stack_depth: 6,
            cpu_ns: 150_000,
        },
    };
    
    match div {
        Divergence::Resources { expected, actual } => {
            assert_eq!(expected.heap_bytes, 1024);
            assert_eq!(actual.heap_bytes, 2048);
        }
        _ => panic!("Expected Resources divergence"),
    }
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validation_result() {
    let result = ValidationResult {
        is_deterministic: true,
        divergences: vec![],
    };
    assert!(result.is_deterministic);
    assert!(result.divergences.is_empty());
}

#[test]
fn test_validation_result_with_divergences() {
    let result = ValidationResult {
        is_deterministic: false,
        divergences: vec![
            Divergence::Output {
                expected: "1".to_string(),
                actual: "2".to_string(),
            },
        ],
    };
    assert!(!result.is_deterministic);
    assert_eq!(result.divergences.len(), 1);
}

#[test]
fn test_resource_tolerance_default() {
    let tolerance = ResourceTolerance::default();
    assert_eq!(tolerance.heap_bytes_percent, 10.0);
    assert_eq!(tolerance.cpu_ns_percent, 20.0);
}

#[test]
fn test_resource_tolerance_custom() {
    let tolerance = ResourceTolerance {
        heap_bytes_percent: 5.0,
        cpu_ns_percent: 15.0,
    };
    assert_eq!(tolerance.heap_bytes_percent, 5.0);
    assert_eq!(tolerance.cpu_ns_percent, 15.0);
}

#[test]
fn test_validation_report_new() {
    let report = ValidationReport::new();
    assert!(!report.passed);
    assert_eq!(report.total_events, 0);
    assert_eq!(report.successful_events, 0);
    assert!(report.divergences.is_empty());
}

#[test]
fn test_validation_report_add_divergence() {
    let mut report = ValidationReport::new();
    report.add_divergence(
        EventId(1),
        Divergence::Output {
            expected: "a".to_string(),
            actual: "b".to_string(),
        },
    );
    
    assert!(!report.passed);
    assert_eq!(report.divergences.len(), 1);
    assert_eq!(report.divergences[0].0, EventId(1));
}

#[test]
fn test_validation_report_success() {
    let mut report = ValidationReport::new();
    report.total_events = 10;
    report.successful_events = 10;
    report.passed = true;
    
    assert!(report.passed);
    assert_eq!(report.total_events, 10);
    assert_eq!(report.successful_events, 10);
}

// ============================================================================
// Replay Validator Tests
// ============================================================================

#[test]
fn test_replay_validator_new() {
    let validator = ReplayValidator::new(true);
    assert!(validator.strict_mode);
    assert_eq!(validator.tolerance.heap_bytes_percent, 10.0);
}

#[test]
fn test_replay_validator_relaxed() {
    let validator = ReplayValidator::new(false);
    assert!(!validator.strict_mode);
}

// Mock implementation for testing
struct MockDeterministicRepl {
    outputs: Vec<ReplayResult>,
    call_count: usize,
}

impl MockDeterministicRepl {
    fn new() -> Self {
        Self {
            outputs: vec![],
            call_count: 0,
        }
    }
    
    fn with_output(mut self, output: ReplayResult) -> Self {
        self.outputs.push(output);
        self
    }
}

impl DeterministicRepl for MockDeterministicRepl {
    fn execute_with_seed(&mut self, _input: &str, _seed: u64) -> ReplayResult {
        if self.call_count < self.outputs.len() {
            // Can't clone Result<Value, anyhow::Error>, so we need to handle it differently
            let output = match &self.outputs[self.call_count].output {
                Ok(val) => Ok(val.clone()),
                Err(e) => Err(anyhow::anyhow!(e.to_string())),
            };
            let result = ReplayResult {
                output,
                state_hash: self.outputs[self.call_count].state_hash.clone(),
                resource_usage: self.outputs[self.call_count].resource_usage.clone(),
            };
            self.call_count += 1;
            result
        } else {
            ReplayResult {
                output: Ok(Value::Unit),
                state_hash: "default".to_string(),
                resource_usage: ResourceUsage {
                    heap_bytes: 0,
                    stack_depth: 0,
                    cpu_ns: 0,
                },
            }
        }
    }
    
    fn checkpoint(&self) -> StateCheckpoint {
        StateCheckpoint {
            bindings: HashMap::new(),
            type_environment: HashMap::new(),
            state_hash: "checkpoint".to_string(),
            resource_usage: ResourceUsage {
                heap_bytes: 0,
                stack_depth: 0,
                cpu_ns: 0,
            },
        }
    }
    
    fn restore(&mut self, _checkpoint: &StateCheckpoint) -> anyhow::Result<()> {
        Ok(())
    }
    
    fn validate_determinism(&self, _other: &Self) -> ValidationResult {
        ValidationResult {
            is_deterministic: true,
            divergences: vec![],
        }
    }
}

#[test]
fn test_validate_empty_session() {
    let validator = ReplayValidator::new(true);
    let session = ReplSession {
        version: SemVer::new(1, 0, 0),
        metadata: SessionMetadata {
            session_id: "test".to_string(),
            created_at: "2024".to_string(),
            ruchy_version: "1.0.0".to_string(),
            student_id: None,
            assignment_id: None,
            tags: vec![],
        },
        environment: Environment {
            seed: 0,
            feature_flags: vec![],
            resource_limits: ResourceLimits {
                heap_mb: 128,
                stack_kb: 1024,
                cpu_ms: 1000,
            },
        },
        timeline: vec![],
        checkpoints: BTreeMap::new(),
    };
    
    let mut repl = MockDeterministicRepl::new();
    let report = validator.validate_session(&session, &mut repl);
    
    assert!(report.passed);
    assert_eq!(report.total_events, 0);
}

#[test]
fn test_validate_session_with_matching_output() {
    let validator = ReplayValidator::new(false);
    
    let timeline = vec![
        TimestampedEvent {
            id: EventId(1),
            timestamp_ns: 1000,
            event: Event::Input {
                text: "1 + 1".to_string(),
                mode: InputMode::Interactive,
            },
            causality: vec![],
        },
        TimestampedEvent {
            id: EventId(2),
            timestamp_ns: 2000,
            event: Event::Output {
                result: EvalResult::Success { value: "Int(2)".to_string() },
                stdout: vec![],
                stderr: vec![],
            },
            causality: vec![EventId(1)],
        },
    ];
    
    let session = ReplSession {
        version: SemVer::new(1, 0, 0),
        metadata: SessionMetadata {
            session_id: "test".to_string(),
            created_at: "2024".to_string(),
            ruchy_version: "1.0.0".to_string(),
            student_id: None,
            assignment_id: None,
            tags: vec![],
        },
        environment: Environment {
            seed: 42,
            feature_flags: vec![],
            resource_limits: ResourceLimits {
                heap_mb: 128,
                stack_kb: 1024,
                cpu_ms: 1000,
            },
        },
        timeline,
        checkpoints: BTreeMap::new(),
    };
    
    let mut repl = MockDeterministicRepl::new()
        .with_output(ReplayResult {
            output: Ok(Value::Int(2)),
            state_hash: "hash".to_string(),
            resource_usage: ResourceUsage {
                heap_bytes: 100,
                stack_depth: 1,
                cpu_ns: 1000,
            },
        });
    
    let report = validator.validate_session(&session, &mut repl);
    
    assert!(report.passed);
    assert_eq!(report.successful_events, 1);
}

// ============================================================================
// Session Recorder Tests
// ============================================================================

#[test]
fn test_session_recorder_new() {
    let metadata = SessionMetadata {
        session_id: "test-123".to_string(),
        created_at: "2024-01-01".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: Some("student1".to_string()),
        assignment_id: Some("hw1".to_string()),
        tags: vec!["test".to_string()],
    };
    
    let recorder = SessionRecorder::new(metadata.clone());
    let session = recorder.get_session();
    
    assert_eq!(session.metadata.session_id, "test-123");
    assert!(session.metadata.student_id.is_some());
    assert!(session.timeline.is_empty());
    assert!(session.checkpoints.is_empty());
}

#[test]
fn test_session_recorder_record_input() {
    let metadata = SessionMetadata {
        session_id: "test".to_string(),
        created_at: "2024".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: None,
        assignment_id: None,
        tags: vec![],
    };
    
    let mut recorder = SessionRecorder::new(metadata);
    
    let id1 = recorder.record_input("let x = 1".to_string(), InputMode::Interactive);
    assert_eq!(id1, EventId(1));
    
    let id2 = recorder.record_input("let y = 2".to_string(), InputMode::Paste);
    assert_eq!(id2, EventId(2));
    
    let session = recorder.get_session();
    assert_eq!(session.timeline.len(), 2);
    
    match &session.timeline[0].event {
        Event::Input { text, mode } => {
            assert_eq!(text, "let x = 1");
            assert_eq!(*mode, InputMode::Interactive);
        }
        _ => panic!("Expected Input event"),
    }
}

#[test]
fn test_session_recorder_record_output() {
    let metadata = SessionMetadata {
        session_id: "test".to_string(),
        created_at: "2024".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: None,
        assignment_id: None,
        tags: vec![],
    };
    
    let mut recorder = SessionRecorder::new(metadata);
    
    // Record success output
    let id1 = recorder.record_output(Ok(Value::Int(42)));
    assert_eq!(id1, EventId(1));
    
    // Record unit output
    let id2 = recorder.record_output(Ok(Value::Unit));
    assert_eq!(id2, EventId(2));
    
    // Record error output
    let id3 = recorder.record_output(Err(anyhow::anyhow!("Test error")));
    assert_eq!(id3, EventId(3));
    
    let session = recorder.get_session();
    assert_eq!(session.timeline.len(), 3);
    
    // Check success output
    match &session.timeline[0].event {
        Event::Output { result, .. } => {
            assert!(matches!(result, EvalResult::Success { .. }));
        }
        _ => panic!("Expected Output event"),
    }
    
    // Check unit output
    match &session.timeline[1].event {
        Event::Output { result, .. } => {
            assert!(matches!(result, EvalResult::Unit));
        }
        _ => panic!("Expected Output event"),
    }
    
    // Check error output
    match &session.timeline[2].event {
        Event::Output { result, .. } => {
            assert!(matches!(result, EvalResult::Error { .. }));
        }
        _ => panic!("Expected Output event"),
    }
}

#[test]
fn test_session_recorder_add_checkpoint() {
    let metadata = SessionMetadata {
        session_id: "test".to_string(),
        created_at: "2024".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: None,
        assignment_id: None,
        tags: vec![],
    };
    
    let mut recorder = SessionRecorder::new(metadata);
    
    let checkpoint = StateCheckpoint {
        bindings: HashMap::new(),
        type_environment: HashMap::new(),
        state_hash: "hash123".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 1024,
            stack_depth: 5,
            cpu_ns: 100_000,
        },
    };
    
    recorder.add_checkpoint(EventId(1), checkpoint);
    
    let session = recorder.get_session();
    assert_eq!(session.checkpoints.len(), 1);
    assert!(session.checkpoints.contains_key(&EventId(1)));
}

#[test]
fn test_session_recorder_into_session() {
    let metadata = SessionMetadata {
        session_id: "test".to_string(),
        created_at: "2024".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: None,
        assignment_id: None,
        tags: vec![],
    };
    
    let mut recorder = SessionRecorder::new(metadata);
    recorder.record_input("test".to_string(), InputMode::Interactive);
    
    let session = recorder.into_session();
    assert_eq!(session.timeline.len(), 1);
}

#[test]
fn test_session_recorder_complete_workflow() {
    let metadata = SessionMetadata {
        session_id: "complete-test".to_string(),
        created_at: "2024-01-01T10:00:00Z".to_string(),
        ruchy_version: "1.0.0".to_string(),
        student_id: Some("student123".to_string()),
        assignment_id: Some("assignment1".to_string()),
        tags: vec!["integration".to_string(), "test".to_string()],
    };
    
    let mut recorder = SessionRecorder::new(metadata);
    
    // Simulate a complete REPL interaction
    let input_id = recorder.record_input("let x = 10".to_string(), InputMode::Interactive);
    let output_id = recorder.record_output(Ok(Value::Unit));
    
    // Add a checkpoint after binding
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), "Int(10)".to_string());
    
    let checkpoint = StateCheckpoint {
        bindings,
        type_environment: HashMap::new(),
        state_hash: "after_x_binding".to_string(),
        resource_usage: ResourceUsage {
            heap_bytes: 100,
            stack_depth: 1,
            cpu_ns: 1000,
        },
    };
    
    recorder.add_checkpoint(output_id, checkpoint);
    
    // More interactions
    recorder.record_input("x * 2".to_string(), InputMode::Interactive);
    recorder.record_output(Ok(Value::Int(20)));
    
    let session = recorder.into_session();
    
    assert_eq!(session.timeline.len(), 4);
    assert_eq!(session.checkpoints.len(), 1);
    assert!(session.metadata.student_id.is_some());
    assert_eq!(session.metadata.tags.len(), 2);
}

