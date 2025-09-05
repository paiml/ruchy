//! REPL history management module
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// History manager for REPL commands
#[derive(Debug, Clone)]
pub struct HistoryManager {
    entries: VecDeque<HistoryEntry>,
    max_size: usize,
    file_path: Option<String>,
    current_session: Vec<String>,
}

/// Single history entry with metadata
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: std::time::SystemTime,
    pub execution_time: Option<std::time::Duration>,
    pub success: bool,
}

impl HistoryManager {
    /// Create a new history manager
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
            file_path: None,
            current_session: Vec::new(),
        }
    }

    /// Set the history file path
    pub fn with_file(mut self, path: String) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Add a command to history
    pub fn add(&mut self, command: String, success: bool) {
        let entry = HistoryEntry {
            command: command.clone(),
            timestamp: std::time::SystemTime::now(),
            execution_time: None,
            success,
        };

        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        
        self.entries.push_back(entry);
        self.current_session.push(command);
    }

    /// Add a command with execution time
    pub fn add_with_timing(
        &mut self,
        command: String,
        execution_time: std::time::Duration,
        success: bool,
    ) {
        let entry = HistoryEntry {
            command: command.clone(),
            timestamp: std::time::SystemTime::now(),
            execution_time: Some(execution_time),
            success,
        };

        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        
        self.entries.push_back(entry);
        self.current_session.push(command);
    }

    /// Get the last n commands
    pub fn last(&self, n: usize) -> Vec<String> {
        self.entries
            .iter()
            .rev()
            .take(n)
            .map(|e| e.command.clone())
            .collect()
    }

    /// Search history for commands containing a pattern
    pub fn search(&self, pattern: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.command.contains(pattern))
            .map(|e| e.command.clone())
            .collect()
    }

    /// Get all successful commands
    pub fn successful_commands(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.success)
            .map(|e| e.command.clone())
            .collect()
    }

    /// Load history from file
    pub fn load(&mut self) -> Result<usize, std::io::Error> {
        let path = match &self.file_path {
            Some(p) => p,
            None => return Ok(0),
        };

        if !Path::new(path).exists() {
            return Ok(0);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            if let Ok(command) = line {
                if !command.trim().is_empty() {
                    self.add(command, true);
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Save history to file
    pub fn save(&self) -> Result<usize, std::io::Error> {
        let path = match &self.file_path {
            Some(p) => p,
            None => return Ok(0),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let mut count = 0;
        for entry in &self.entries {
            writeln!(file, "{}", entry.command)?;
            count += 1;
        }

        Ok(count)
    }

    /// Append current session to file
    pub fn append_session(&self) -> Result<usize, std::io::Error> {
        let path = match &self.file_path {
            Some(p) => p,
            None => return Ok(0),
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let mut count = 0;
        for command in &self.current_session {
            writeln!(file, "{}", command)?;
            count += 1;
        }

        Ok(count)
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_session.clear();
    }

    /// Get history size
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get session history
    pub fn session_history(&self) -> &[String] {
        &self.current_session
    }

    /// Get history statistics
    pub fn stats(&self) -> HistoryStats {
        let total = self.entries.len();
        let successful = self.entries.iter().filter(|e| e.success).count();
        let failed = total - successful;
        
        let total_time: std::time::Duration = self.entries
            .iter()
            .filter_map(|e| e.execution_time)
            .sum();

        HistoryStats {
            total_commands: total,
            successful_commands: successful,
            failed_commands: failed,
            total_execution_time: total_time,
            session_commands: self.current_session.len(),
        }
    }
}

/// History statistics
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub successful_commands: usize,
    pub failed_commands: usize,
    pub total_execution_time: std::time::Duration,
    pub session_commands: usize,
}

impl HistoryStats {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_commands == 0 {
            0.0
        } else {
            (self.successful_commands as f64 / self.total_commands as f64) * 100.0
        }
    }

    /// Get average execution time
    pub fn average_execution_time(&self) -> std::time::Duration {
        if self.total_commands == 0 {
            std::time::Duration::from_secs(0)
        } else {
            self.total_execution_time / self.total_commands as u32
        }
    }
}