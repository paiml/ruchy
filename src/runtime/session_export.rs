//! Public REPL session export for entrenar (ruchy#166).
//!
//! Converts a recorded REPL session into cells (input + output pairs with
//! timestamps) and renders it as a literate Markdown document.

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
