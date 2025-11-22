//! Server process management
//!
//! This module provides PID file management for the HTTP server,
//! enabling automatic process lifecycle management and fixing zsh
//! background execution issues.
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::server::PidFile;
//! use std::path::PathBuf;
//!
//! let pid_file = PidFile::new(PathBuf::from("/tmp/ruchy.pid")).unwrap();
//! // Server runs...
//! // PID file automatically cleaned up on drop
//! ```

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

/// PID file manager with RAII cleanup
///
/// Automatically manages process lifecycle:
/// 1. Check if PID file exists
/// 2. If exists and process running → kill old process
/// 3. Write current PID to file
/// 4. Clean up PID file on drop (graceful shutdown)
///
/// # Complexity
/// Cyclomatic complexity: TBD (target ≤10)
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    /// Create new PID file from a path reference
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Cannot read existing PID file
    /// - Cannot write new PID file
    /// - Cannot kill existing process
    pub fn create(path: &std::path::Path) -> io::Result<Self> {
        Self::new(path.to_path_buf())
    }

    /// Create new PID file, killing existing process if needed
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Cannot read existing PID file
    /// - Cannot write new PID file
    /// - Cannot kill existing process
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::server::PidFile;
    /// use std::path::PathBuf;
    ///
    /// let pid_file = PidFile::new(PathBuf::from("/tmp/ruchy.pid"))?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn new(path: PathBuf) -> io::Result<Self> {
        // Check if PID file exists
        if path.exists() {
            // Read existing PID
            let pid_str = fs::read_to_string(&path)?;
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                // Check if process is still running
                #[cfg(unix)]
                {
                    use nix::sys::signal::kill;
                    use nix::unistd::Pid;

                    // Send signal 0 to check if process exists
                    let result = kill(Pid::from_raw(pid as i32), None);
                    if result.is_ok() {
                        // Process exists, kill it with SIGTERM
                        use nix::sys::signal::Signal;
                        let _ = kill(Pid::from_raw(pid as i32), Some(Signal::SIGTERM));
                        // Give process time to terminate
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }
            }
            // Remove stale PID file
            fs::remove_file(&path)?;
        }

        // Write current PID to file
        let current_pid = process::id();
        fs::write(&path, current_pid.to_string())?;

        Ok(PidFile { path })
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        // Clean up PID file on drop (RAII pattern)
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    /// Test that PID file is created with current process ID
    ///
    /// RED: This test should FAIL because `PidFile::new()` is unimplemented
    #[test]
    fn test_pid_file_creation() {
        let temp_dir = tempfile::tempdir().expect("operation should succeed in test");
        let pid_path = temp_dir.path().join("test.pid");

        let _pid_file = PidFile::new(pid_path.clone()).expect("operation should succeed in test");

        // PID file should exist
        assert!(pid_path.exists(), "PID file should be created");

        // Should contain current process ID
        let contents = fs::read_to_string(&pid_path).expect("operation should succeed in test");
        let expected = process::id().to_string();
        assert_eq!(
            contents, expected,
            "PID file should contain current process ID"
        );
    }

    /// Test that PID file is cleaned up on drop
    ///
    /// RED: This test should FAIL because Drop is not implemented
    #[test]
    fn test_pid_file_cleanup() {
        let temp_dir = tempfile::tempdir().expect("operation should succeed in test");
        let pid_path = temp_dir.path().join("test.pid");

        {
            let _pid_file =
                PidFile::new(pid_path.clone()).expect("operation should succeed in test");
            assert!(pid_path.exists(), "PID file should exist while in scope");
        } // PidFile dropped here

        // PID file should be cleaned up
        assert!(!pid_path.exists(), "PID file should be removed after drop");
    }

    /// Test that stale PID file (non-existent process) is replaced
    ///
    /// RED: This test should FAIL because `PidFile::new()` doesn't check for stale PIDs
    #[test]
    fn test_pid_file_replaces_stale() {
        let temp_dir = tempfile::tempdir().expect("operation should succeed in test");
        let pid_path = temp_dir.path().join("test.pid");

        // Write stale PID (non-existent process)
        fs::write(&pid_path, "999999").expect("operation should succeed in test");

        let _pid_file = PidFile::new(pid_path.clone()).expect("operation should succeed in test");

        // Should have replaced with current PID
        let contents = fs::read_to_string(&pid_path).expect("operation should succeed in test");
        let expected = process::id().to_string();
        assert_eq!(
            contents, expected,
            "Stale PID should be replaced with current process ID"
        );
    }

    /// Test that PID file with running process is handled gracefully
    ///
    /// RED: This test should FAIL because `PidFile::new()` doesn't kill running processes
    #[test]
    #[ignore = "Requires Unix signals - run with: cargo test -- --ignored"]
    fn test_pid_file_kills_running_process() {
        let temp_dir = tempfile::tempdir().expect("operation should succeed in test");
        let pid_path = temp_dir.path().join("test.pid");

        // Spawn a child process and write its PID
        let mut child = std::process::Command::new("sleep")
            .arg("10")
            .spawn()
            .expect("operation should succeed in test");
        let child_pid = child.id();
        fs::write(&pid_path, child_pid.to_string()).expect("operation should succeed in test");

        // Create PidFile - should kill the child process
        let _pid_file = PidFile::new(pid_path).expect("operation should succeed in test");

        // Wait a moment for kill to take effect
        thread::sleep(Duration::from_millis(100));

        // Wait on child to clean up zombie process (child should already be killed)
        let _ = child.wait();

        // Child process should be dead
        // On Unix: check if process exists via kill -0 (sends no signal, just checks existence)
        #[cfg(unix)]
        {
            use nix::sys::signal::kill;
            use nix::unistd::Pid;
            let result = kill(Pid::from_raw(child_pid as i32), None);
            assert!(
                result.is_err(),
                "Old process should be killed before creating new PID file"
            );
        }
    }

    /// Property test: PID file always contains valid process ID
    ///
    /// RED: This test should FAIL because `PidFile::new()` is unimplemented
    #[test]
    #[ignore = "Property test - run with: cargo test -- --ignored"]
    fn prop_pid_file_always_valid() {
        use proptest::prelude::*;

        proptest!(|(seed in any::<u32>())| {
            let temp_dir = tempfile::tempdir().expect("operation should succeed in test");
            let pid_path = temp_dir.path().join(format!("test_{seed}.pid"));

            let _pid_file = PidFile::new(pid_path.clone()).expect("operation should succeed in test");

            // PID file should exist and contain valid PID
            prop_assert!(pid_path.exists());

            let contents = fs::read_to_string(&pid_path).expect("operation should succeed in test");
            let pid: u32 = contents.parse().expect("operation should succeed in test");
            prop_assert_eq!(pid, process::id());
        });
    }
}

pub mod watcher;
