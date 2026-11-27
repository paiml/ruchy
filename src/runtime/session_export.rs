//! Public REPL session export for entrenar (ruchy#166).
//!
//! Converts a recorded REPL session into cells (input + output pairs with
//! timestamps) and renders it as a literate Markdown document.

use crate::runtime::replay::{EvalResult, Event, ReplSession};
use chrono::{DateTime, TimeDelta, Utc};
use std::fmt;

/// One REPL input and the output that followed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// Source text as typed or pasted.
    pub input: String,
    /// Rendered result; `None` for unit results and for inputs with no output.
    pub output: Option<String>,
    /// Nanoseconds since session start when the input was recorded.
    pub timestamp_ns: u64,
}

/// A recorded REPL session in export form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuchySession {
    pub cells: Vec<Cell>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

/// Why a session could not be exported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionExportError {
    /// `metadata.created_at` is not an RFC 3339 timestamp.
    BadTimestamp(String),
}

impl fmt::Display for SessionExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadTimestamp(raw) => write!(f, "session created_at is not RFC 3339: {raw:?}"),
        }
    }
}

impl std::error::Error for SessionExportError {}

impl RuchySession {
    /// Build an export from a recorded session.
    ///
    /// Each input starts a cell; the first output after it fills the cell.
    /// `started_at` comes from `metadata.created_at`.
    pub fn from_repl_session(session: &ReplSession) -> Result<Self, SessionExportError> {
        let started_at = DateTime::parse_from_rfc3339(&session.metadata.created_at)
            .map_err(|_| SessionExportError::BadTimestamp(session.metadata.created_at.clone()))?
            .with_timezone(&Utc);
        let mut cells: Vec<Cell> = Vec::new();
        for event in &session.timeline {
            match &event.event {
                Event::Input { text, .. } => cells.push(Cell {
                    input: text.clone(),
                    output: None,
                    timestamp_ns: event.timestamp_ns,
                }),
                Event::Output { result, .. } => {
                    if let Some(cell) = cells.last_mut().filter(|c| c.output.is_none()) {
                        cell.output = render_output(result);
                    }
                }
                Event::StateChange { .. } | Event::ResourceUsage { .. } => {}
            }
        }
        let ended_at = session
            .timeline
            .last()
            .and_then(|e| i64::try_from(e.timestamp_ns).ok())
            .map(TimeDelta::nanoseconds)
            .and_then(|d| started_at.checked_add_signed(d));
        Ok(Self {
            cells,
            started_at,
            ended_at,
        })
    }

    /// Render the session as a literate Markdown document: one `ruchy` code
    /// block per input, followed by a `text` block for its output.
    pub fn to_literate_document(&self) -> String {
        let mut out = String::new();
        for cell in &self.cells {
            out.push_str("```ruchy\n");
            out.push_str(&cell.input);
            out.push_str("\n```\n");
            if let Some(output) = &cell.output {
                out.push_str("\n```text\n");
                out.push_str(output);
                out.push_str("\n```\n");
            }
            out.push('\n');
        }
        out
    }
}

fn render_output(result: &EvalResult) -> Option<String> {
    match result {
        EvalResult::Success { value } => Some(value.clone()),
        EvalResult::Error { message } => Some(format!("error: {message}")),
        EvalResult::Unit => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::replay::{InputMode, SessionMetadata, SessionRecorder};
    use crate::runtime::value::Value;
    use anyhow::anyhow;

    fn recorder_at(created_at: &str) -> SessionRecorder {
        SessionRecorder::new(SessionMetadata {
            session_id: "s-166".to_string(),
            created_at: created_at.to_string(),
            ruchy_version: "5.4.0".to_string(),
            student_id: None,
            assignment_id: None,
            tags: vec![],
        })
    }

    #[test]
    fn test_ruchy_166_export_pairs_inputs_with_outputs() {
        let mut r = recorder_at("2026-11-27T00:00:00Z");
        r.record_input("let x = 2".to_string(), InputMode::Interactive);
        r.record_output(Ok(Value::Nil));
        r.record_input("x + 1".to_string(), InputMode::Interactive);
        r.record_output(Ok(Value::Integer(3)));

        let s = RuchySession::from_repl_session(r.get_session()).expect("export");

        assert_eq!(s.cells.len(), 2);
        assert_eq!(s.cells[0].input, "let x = 2");
        assert_eq!(s.cells[0].output, None);
        assert_eq!(s.cells[1].input, "x + 1");
        assert!(s.cells[1].output.as_deref().unwrap_or("").contains('3'));
    }

    #[test]
    fn test_ruchy_166_export_keeps_errors_as_output() {
        let mut r = recorder_at("2026-11-27T00:00:00Z");
        r.record_input("1 / 0".to_string(), InputMode::Interactive);
        r.record_output(Err(anyhow!("division by zero")));

        let s = RuchySession::from_repl_session(r.get_session()).expect("export");

        assert_eq!(
            s.cells[0].output.as_deref(),
            Some("error: division by zero")
        );
    }

    #[test]
    fn test_ruchy_166_export_timestamps_from_created_at() {
        let mut r = recorder_at("2026-11-27T00:00:00Z");
        r.record_input("1".to_string(), InputMode::Interactive);
        r.record_output(Ok(Value::Integer(1)));

        let s = RuchySession::from_repl_session(r.get_session()).expect("export");

        let started = "2026-11-27T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!(s.started_at, started);
        let ended = s.ended_at.expect("ended_at set when timeline is non-empty");
        assert!(ended >= started);
    }

    #[test]
    fn test_ruchy_166_export_rejects_bad_created_at() {
        let mut r = recorder_at("yesterday");
        r.record_input("1".to_string(), InputMode::Interactive);

        assert!(RuchySession::from_repl_session(r.get_session()).is_err());
    }

    #[test]
    fn test_ruchy_166_to_literate_document_renders_cells() {
        let mut r = recorder_at("2026-11-27T00:00:00Z");
        r.record_input("x + 1".to_string(), InputMode::Interactive);
        r.record_output(Ok(Value::Integer(3)));

        let s = RuchySession::from_repl_session(r.get_session()).expect("export");
        let md = s.to_literate_document();

        assert!(md.contains("```ruchy\nx + 1\n```"), "{md}");
        assert!(md.contains("```text\nInteger(3)\n```"), "{md}");
    }
}
