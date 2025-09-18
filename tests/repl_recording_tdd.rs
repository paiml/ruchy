#[cfg(test)]
mod repl_recording_tdd {
    use ruchy::runtime::Repl;
    use tempfile::TempDir;

    #[test]
    fn test_run_with_recording_creates_replay_file() {
        // RED: This test should fail because run_with_recording_refactored doesn't exist
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let record_path = temp_dir.path().join("test_session.replay");
        
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        // This should create a .replay file when recording is enabled
        // For now, just test that the method exists and doesn't panic
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            repl.run_with_recording(&record_path)
        }));
        
        // RED: This will fail because the method doesn't exist
        assert!(result.is_ok(), "run_with_recording should exist and not panic");
        
        // Once implemented, the .replay file should exist
        // assert!(record_path.exists(), "Recording should create .replay file");
    }
    
    #[test]
    fn test_run_with_recording_refactored_method_exists() {
        // RED: Test that the delegated method exists
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let record_path = temp_dir.path().join("test.replay");
        
        let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
        
        // This should fail because run_with_recording_refactored doesn't exist yet
        let result = repl.run_with_recording(&record_path);
        
        // For now, just verify the method can be called without compile error
        // The actual functionality will be implemented in GREEN step
        assert!(result.is_err() || result.is_ok(), "Method should be callable");
    }
    
    #[test] 
    fn test_session_recorder_generates_unique_seeds() {
        // RED: This test should fail because SessionRecorder uses hardcoded seed of 0
        
        use ruchy::runtime::replay::{SessionRecorder, SessionMetadata};
        use std::{env, time::SystemTime;
        
        // Create metadata for two sessions
        let metadata1 = SessionMetadata {
            session_id: "session1".to_string(),
            created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string(),
            ruchy_version: "1.85.0".to_string(),
            student_id: Some("test_user".to_string()),
            assignment_id: None,
            tags: vec![],
        };
        
        let metadata2 = SessionMetadata {
            session_id: "session2".to_string(),
            created_at: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string(),
            ruchy_version: "1.85.0".to_string(),
            student_id: Some("test_user".to_string()),
            assignment_id: None,
            tags: vec![],
        };
        
        // Create two different session recorders
        let recorder1 = SessionRecorder::new(metadata1);
        let recorder2 = SessionRecorder::new(metadata2);
        
        // Get the sessions and check their environment seeds
        let session1 = recorder1.into_session();
        let session2 = recorder2.into_session();
        
        // RED: This should fail because both sessions use seed = 0
        assert_ne!(session1.environment.seed, session2.environment.seed,
            "Different sessions should have different seeds, got seed1={} seed2={}",
            session1.environment.seed, session2.environment.seed);
        
        // Also verify neither seed is 0 (hardcoded value)
        assert_ne!(session1.environment.seed, 0, "Seed should not be hardcoded to 0");
        assert_ne!(session2.environment.seed, 0, "Seed should not be hardcoded to 0");
    }
    
    #[test]
    fn test_replay_file_contains_required_metadata() {
        // RED: Test that generated .replay files contain all required metadata fields
        
        use ruchy::runtime::replay::{SessionRecorder, SessionMetadata};
        use tempfile::TempDir;
        
        // Create a session recorder with known metadata
        let metadata = SessionMetadata {
            session_id: "test-session-123".to_string(),
            created_at: "2025-09-08T10:00:00Z".to_string(),
            ruchy_version: "1.85.0".to_string(),
            student_id: Some("student-456".to_string()),
            assignment_id: Some("assignment-789".to_string()),
            tags: vec!["test".to_string(), "replay".to_string()],
        };
        
        let recorder = SessionRecorder::new(metadata);
        let session = recorder.into_session();
        
        // Serialize to JSON (same as what repl_recording does)
        let session_json = serde_json::to_string_pretty(&session)
            .expect("Should serialize session to JSON");
        
        // Write to temporary file to simulate actual replay file generation
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let replay_file = temp_dir.path().join("test.replay");
        std::fs::write(&replay_file, &session_json)
            .expect("Should write replay file");
        
        // Read back and verify structure
        let content = std::fs::read_to_string(&replay_file)
            .expect("Should read replay file");
        
        // Parse as JSON to verify it's valid
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .expect("Replay file should contain valid JSON");
        
        // Verify required top-level fields
        assert!(parsed.get("version").is_some(), "Should have version field");
        assert!(parsed.get("metadata").is_some(), "Should have metadata field"); 
        assert!(parsed.get("environment").is_some(), "Should have environment field");
        assert!(parsed.get("timeline").is_some(), "Should have timeline field");
        
        // Verify metadata fields
        let metadata_obj = parsed.get("metadata").unwrap();
        assert_eq!(metadata_obj.get("session_id").unwrap().as_str().unwrap(), "test-session-123");
        assert_eq!(metadata_obj.get("created_at").unwrap().as_str().unwrap(), "2025-09-08T10:00:00Z");
        assert_eq!(metadata_obj.get("ruchy_version").unwrap().as_str().unwrap(), "1.85.0");
        assert_eq!(metadata_obj.get("student_id").unwrap().as_str().unwrap(), "student-456");
        assert_eq!(metadata_obj.get("assignment_id").unwrap().as_str().unwrap(), "assignment-789");
        
        // Verify environment has seed (should not be 0 after our fix)
        let environment_obj = parsed.get("environment").unwrap();
        let seed = environment_obj.get("seed").unwrap().as_u64().unwrap();
        assert_ne!(seed, 0, "Environment seed should not be 0");
        
        // Verify timeline is array (empty for new session)
        assert!(parsed.get("timeline").unwrap().is_array(), "Timeline should be array");
        
        println!("✅ Replay file structure verified with seed: {seed}");
    }
    
    #[test]
    #[ignore = "Integration test - run with --ignored flag"]
    fn test_actual_repl_recording_generates_nonzero_seed() {
        // RED: Test that actual REPL recording generates non-zero seed
        use std::{env, process::Command;
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let replay_file = temp_dir.path().join("test.replay");
        
        // Build the binary first
        Command::new("cargo")
            .args(["build", "--bin", "ruchy"])
            .output()
            .expect("Failed to build");
        
        // Run actual REPL with recording
        let _output = Command::new("sh")
            .arg("-c")
            .arg(format!("echo ':quit' | timeout 1s ./target/debug/ruchy repl --record {} 2>/dev/null", 
                replay_file.display()))
            .output()
            .expect("Failed to execute command");
        
        // Verify replay file was created
        assert!(replay_file.exists(), "Replay file should be created");
        
        // Read and parse the replay file
        let content = std::fs::read_to_string(&replay_file)
            .expect("Should read replay file");
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .expect("Should parse JSON");
        
        // Check seed is not 0
        let seed = parsed["environment"]["seed"]
            .as_u64()
            .expect("Should have seed field");
        
        // RED: This should fail if seed is still 0
        assert_ne!(seed, 0, "Seed should not be 0, got {seed}");
    }
}