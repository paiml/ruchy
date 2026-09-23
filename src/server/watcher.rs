//! File watcher with debouncing for auto-restart
//!
//! Provides hot-reload functionality similar to nodemon/cargo-watch.

#![cfg(feature = "watch-mode")]

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

/// File watcher with debouncing to prevent restart spam
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    last_change: Option<Instant>,
    debounce_duration: Duration,
}

impl FileWatcher {
    /// Create new file watcher
    pub fn new(paths: Vec<PathBuf>, debounce_ms: u64) -> notify::Result<Self> {
        let (tx, rx): (
            Sender<notify::Result<Event>>,
            Receiver<notify::Result<Event>>,
        ) = channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        // Watch all paths
        for path in paths {
            watcher.watch(&path, RecursiveMode::Recursive)?;
        }

        Ok(Self {
            _watcher: watcher,
            rx,
            last_change: None,
            debounce_duration: Duration::from_millis(debounce_ms),
        })
    }

    /// Check if files changed (debounced)
    pub fn check_changes(&mut self) -> Option<Vec<PathBuf>> {
        let mut changed_files = Vec::new();

        // Drain all pending events
        while let Ok(event) = self.rx.try_recv() {
            if let Ok(event) = event {
                if matches!(
                    event.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    changed_files.extend(event.paths);
                }
            }
        }

        if changed_files.is_empty() {
            return None;
        }

        // Debouncing: only trigger if enough time passed since last change
        let now = Instant::now();
        if let Some(last) = self.last_change {
            if now.duration_since(last) < self.debounce_duration {
                return None; // Too soon, ignore
            }
        }

        self.last_change = Some(now);
        Some(changed_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use tempfile::tempdir;

    /// A watcher for `path`, or `None` when the host has no inotify instance
    /// left (EMFILE/ENOSPC). The limit (`fs.inotify.max_user_instances`, 128
    /// here) is shared by every process of the user, so on a busy host it is
    /// an environment limit, not a watcher defect (FLAKE-1). The skip is
    /// printed; any other creation error still fails the test.
    fn watcher_or_skip(path: PathBuf, debounce_ms: u64) -> Option<FileWatcher> {
        match FileWatcher::new(vec![path], debounce_ms) {
            Ok(w) => Some(w),
            Err(e) if is_host_watch_limit(&e) => {
                eprintln!("skipped: host inotify limit reached: {e}");
                None
            }
            Err(e) => panic!("watcher creation failed: {e}"),
        }
    }

    fn is_host_watch_limit(e: &notify::Error) -> bool {
        match &e.kind {
            notify::ErrorKind::Io(io) => matches!(io.raw_os_error(), Some(24 | 28)),
            notify::ErrorKind::MaxFilesWatch => true,
            _ => false,
        }
    }

    #[test]
    fn test_flake_1_host_watch_limit_is_recognised() {
        let emfile = notify::Error::io(std::io::Error::from_raw_os_error(24));
        let enoent = notify::Error::io(std::io::Error::from_raw_os_error(2));
        assert!(is_host_watch_limit(&emfile));
        assert!(!is_host_watch_limit(&enoent));
    }

    #[test]
    fn test_watcher_detects_file_creation() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();

        let Some(mut watcher) = watcher_or_skip(watch_path.clone(), 100) else {
            return;
        };

        // Create a file
        let test_file = watch_path.join("test.txt");
        fs::write(&test_file, "hello").expect("operation should succeed in test");

        // Wait for event to propagate
        thread::sleep(Duration::from_millis(200));

        // Should detect change
        let changes = watcher.check_changes();
        assert!(changes.is_some(), "Should detect file creation");
    }

    #[test]
    fn test_watcher_debounces_rapid_changes() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();

        // Use longer debounce time for reliable testing under load
        let Some(mut watcher) = watcher_or_skip(watch_path.clone(), 3000) else {
            return;
        };

        let test_file = watch_path.join("test.txt");

        // Rapid changes
        fs::write(&test_file, "1").expect("operation should succeed in test");
        thread::sleep(Duration::from_millis(50));
        fs::write(&test_file, "2").expect("operation should succeed in test");
        thread::sleep(Duration::from_millis(50));
        fs::write(&test_file, "3").expect("operation should succeed in test");

        // Wait for events with retry - file system events can be slow under load
        let mut first = None;
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(100));
            first = watcher.check_changes();
            if first.is_some() {
                break;
            }
        }
        assert!(first.is_some(), "First check should detect changes");

        // Immediate second check should be debounced (within 3000ms window)
        fs::write(&test_file, "4").expect("operation should succeed in test");
        thread::sleep(Duration::from_millis(200)); // Wait for event
        let second = watcher.check_changes();
        assert!(second.is_none(), "Should debounce rapid changes");
    }

    #[test]
    fn test_watcher_new_creates_instance() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();
        // watcher_or_skip fails the test on any error but the host limit.
        let _ = watcher_or_skip(watch_path, 500);
    }

    #[test]
    fn test_watcher_check_changes_no_events() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();
        let Some(mut watcher) = watcher_or_skip(watch_path, 100) else {
            return;
        };
        // No file changes made
        let changes = watcher.check_changes();
        assert!(changes.is_none(), "Should return None when no changes");
    }

    #[test]
    fn test_watcher_detects_file_modification() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();
        let test_file = watch_path.join("existing.txt");
        fs::write(&test_file, "initial").expect("operation should succeed in test");

        let Some(mut watcher) = watcher_or_skip(watch_path, 100) else {
            return;
        };

        // Modify file
        thread::sleep(Duration::from_millis(100));
        fs::write(&test_file, "modified").expect("operation should succeed in test");
        thread::sleep(Duration::from_millis(200));

        let changes = watcher.check_changes();
        assert!(changes.is_some(), "Should detect file modification");
    }

    #[test]
    fn test_watcher_detects_file_deletion() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();
        let test_file = watch_path.join("to_delete.txt");
        fs::write(&test_file, "content").expect("operation should succeed in test");

        let Some(mut watcher) = watcher_or_skip(watch_path, 100) else {
            return;
        };

        // Delete file
        thread::sleep(Duration::from_millis(100));
        fs::remove_file(&test_file).expect("operation should succeed in test");
        thread::sleep(Duration::from_millis(200));

        let changes = watcher.check_changes();
        assert!(changes.is_some(), "Should detect file deletion");
    }

    #[test]
    fn test_watcher_with_zero_debounce() {
        let temp_dir = tempdir().expect("operation should succeed in test");
        let watch_path = temp_dir.path().to_path_buf();
        // Zero debounce should still work
        let _ = watcher_or_skip(watch_path, 0);
    }
}
