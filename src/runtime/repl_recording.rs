//! Refactored recording functionality with reduced complexity
//!
//! Following TDD approach: Each function has complexity < 20
use crate::runtime::repl::Repl;
use crate::runtime::replay::{SessionMetadata, SessionRecorder, InputMode};
use crate::runtime::completion::RuchyCompleter;
use crate::runtime::Value;
use anyhow::Result;
use colored::Colorize;
use rustyline::{Config, CompletionType, EditMode};
use rustyline::history::DefaultHistory;
use std::path::Path;
use std::time::SystemTime;
impl Repl {
    /// Create session metadata for recording (complexity: 3)
    fn create_session_metadata() -> Result<SessionMetadata> {
        Ok(SessionMetadata {
            session_id: format!("ruchy-session-{}", 
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs()),
            created_at: chrono::Utc::now().to_rfc3339(),
            ruchy_version: env!("CARGO_PKG_VERSION").to_string(),
            student_id: None,
            assignment_id: None,
            tags: vec!["interactive".to_string()],
        })
    }
    /// Setup rustyline editor with configuration (complexity: 5)
    fn setup_recording_editor(&self) -> Result<rustyline::Editor<RuchyCompleter, DefaultHistory>> {
        let config = Config::builder()
            .history_ignore_space(true)
            .history_ignore_dups(true)?
            .completion_type(CompletionType::List)
            .edit_mode(EditMode::Emacs)
            .build();
        let mut rl = rustyline::Editor::<RuchyCompleter, DefaultHistory>::with_config(config)?;
        let completer = RuchyCompleter::new();
        rl.set_helper(Some(completer));
        // Create a session-specific directory for history
        let temp_dir = std::env::temp_dir().join(format!("ruchy-{}", std::process::id()));
        std::fs::create_dir_all(&temp_dir)?;
        let history_path = temp_dir.join("history.txt");
        let _ = rl.load_history(&history_path);
        Ok(rl)
    }
    /// Process single line input during recording (complexity: 8)
    fn process_recorded_input(
        &mut self,
        line: String,
        recorder: &mut SessionRecorder,
        rl: &mut rustyline::Editor<RuchyCompleter, DefaultHistory>,
    ) -> Result<bool> {
        let input = line.trim();
        // Record the input
        let _input_id = recorder.record_input(
            line.clone(), 
            InputMode::Interactive
        );
        // Check for quit commands
        if input == ":quit" || input == ":exit" {
            return Ok(true); // Signal to exit
        }
        if !input.is_empty() {
            rl.add_history_entry(input)?;
            // Evaluate and record result
            let result = self.eval(input);
            let result_for_recording = match &result {
                Ok(s) => Ok(Value::String(s.clone())),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            };
            recorder.record_output(result_for_recording);
            // Display result
            match result {
                Ok(output) if !output.is_empty() => {
                    println!("{output}");
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".bright_red(), e);
                }
                _ => {}
            }
        }
        Ok(false) // Continue running
    }
    /// Process multiline input during recording (complexity: 10)
    fn process_multiline_recorded_input(
        &mut self,
        line: String,
        multiline_buffer: &mut String,
        in_multiline: &mut bool,
        recorder: &mut SessionRecorder,
        rl: &mut rustyline::Editor<RuchyCompleter, DefaultHistory>,
    ) -> Result<()> {
        let input = line.trim();
        if input.is_empty() {
            // Empty line ends multiline input
            let full_input = multiline_buffer.trim().to_string();
            if !full_input.is_empty() {
                rl.add_history_entry(&full_input)?;
                // Evaluate and record result
                let result = self.eval(&full_input);
                let result_for_recording = match &result {
                    Ok(s) => Ok(Value::String(s.clone())),
                    Err(e) => Err(anyhow::anyhow!("{}", e)),
                };
                recorder.record_output(result_for_recording);
                match result {
                    Ok(output) if !output.is_empty() => {
                        println!("{output}");
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Error".bright_red(), e);
                    }
                    _ => {}
                }
            }
            multiline_buffer.clear();
            *in_multiline = false;
        } else {
            multiline_buffer.push_str(&line);
            multiline_buffer.push('\n');
        }
        Ok(())
    }
    /// Main recording loop - refactored with reduced complexity (complexity: 15)
    pub fn run_with_recording_refactored(&mut self, record_file: &Path) -> Result<()> {
        // Create session metadata
        let metadata = Self::create_session_metadata()?;
        let mut recorder = SessionRecorder::new(metadata);
        println!("{}", format!("🎬 Recording session to: {}", record_file.display()).bright_yellow());
        // Setup editor
        let mut rl = self.setup_recording_editor()?;
        let mut multiline_buffer = String::new();
        let mut in_multiline = false;
        // Main loop
        loop {
            let prompt = if in_multiline {
                format!("{} ", "   ...".bright_black())
            } else {
                format!("{} ", self.get_prompt().bright_green())
            };
            match rl.readline(&prompt) {
                Ok(line) => {
                    if in_multiline {
                        // Record multiline input
                        let _input_id = recorder.record_input(
                            line.clone(), 
                            InputMode::Paste
                        );
                        self.process_multiline_recorded_input(
                            line,
                            &mut multiline_buffer,
                            &mut in_multiline,
                            &mut recorder,
                            &mut rl
                        )?;
                    } else {
                        let input = line.trim();
                        if Self::needs_continuation(input) {
                            // Start multiline input
                            multiline_buffer = format!("{line}\n");
                            in_multiline = true;
                            // Record the start of multiline
                            let _input_id = recorder.record_input(
                                line.clone(), 
                                InputMode::Paste
                            );
                        } else {
                            // Process single line
                            let should_exit = self.process_recorded_input(
                                line,
                                &mut recorder,
                                &mut rl
                            )?;
                            if should_exit {
                                break;
                            }
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("{}", "Use :quit to exit".bright_yellow());
                }
                Err(rustyline::error::ReadlineError::Eof) => break,
                Err(err) => {
                    eprintln!("{}: {:?}", "Error".bright_red(), err);
                    break;
                }
            }
        }
        // Save recording
        let session = recorder.into_session();
        let session_json = serde_json::to_string_pretty(&session)?;
        std::fs::write(record_file, session_json)?;
        println!("{}", format!("📼 Session saved to: {}", record_file.display()).bright_green());
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_session_metadata() {
        let metadata = Repl::create_session_metadata().unwrap();
        assert!(metadata.session_id.starts_with("ruchy-session-"));
        assert_eq!(metadata.ruchy_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(metadata.tags, vec!["interactive"]);
    }
    #[test]
    fn test_setup_recording_editor() -> Result<()> {
        let repl = Repl::new()?;
        // Just verify it doesn't panic
        let _editor = repl.setup_recording_editor()?;
        Ok(())
    }
}