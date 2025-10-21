//! File watcher with debouncing for auto-restart
//!
//! Provides hot-reload functionality similar to nodemon/cargo-watch.

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
        let (tx, rx): (Sender<notify::Result<Event>>, Receiver<notify::Result<Event>>) = channel();

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

    #[test]
    fn test_watcher_detects_file_creation() {
        let temp_dir = tempdir().unwrap();
        let watch_path = temp_dir.path().to_path_buf();

        let mut watcher = FileWatcher::new(vec![watch_path.clone()], 100).unwrap();

        // Create a file
        let test_file = watch_path.join("test.txt");
        fs::write(&test_file, "hello").unwrap();

        // Wait for event to propagate
        thread::sleep(Duration::from_millis(200));

        // Should detect change
        let changes = watcher.check_changes();
        assert!(changes.is_some(), "Should detect file creation");
    }

    #[test]
    fn test_watcher_debounces_rapid_changes() {
        let temp_dir = tempdir().unwrap();
        let watch_path = temp_dir.path().to_path_buf();

        let mut watcher = FileWatcher::new(vec![watch_path.clone()], 200).unwrap();

        let test_file = watch_path.join("test.txt");

        // Rapid changes
        fs::write(&test_file, "1").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&test_file, "2").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&test_file, "3").unwrap();

        thread::sleep(Duration::from_millis(100));

        // First check should detect
        let first = watcher.check_changes();
        assert!(first.is_some(), "First check should detect changes");

        // Immediate second check should be debounced
        fs::write(&test_file, "4").unwrap();
        thread::sleep(Duration::from_millis(50));
        let second = watcher.check_changes();
        assert!(second.is_none(), "Should debounce rapid changes");
    }
}
