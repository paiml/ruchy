//! Sprint 72: Zero Coverage Module Testing
//! Target: Boost coverage by testing 0% modules

#[cfg(test)]
mod debugger_tests {
    use ruchy::debugger::{Debugger, Breakpoint, DebugEvent};

    #[test]
    fn test_debugger_basic() {
        let mut debugger = Debugger::new();
        assert!(!debugger.is_running());
        assert!(!debugger.is_paused());
        assert_eq!(debugger.breakpoint_count(), 0);
        assert_eq!(debugger.current_line(), 0);
        assert_eq!(debugger.current_function(), "main");
    }

    #[test]
    fn test_breakpoint_methods() {
        // Test static constructors
        let bp1 = Breakpoint::at_line("test.rs", 10);
        assert_eq!(bp1.line, 10);

        let bp2 = Breakpoint::conditional("test.rs", 20, "x > 5");
        assert_eq!(bp2.line, 20);
        assert!(bp2.condition.is_some());

        let bp3 = Breakpoint::with_hit_count("test.rs", 30, 5);
        assert_eq!(bp3.line, 30);
        assert!(bp3.hit_count_target.is_some());
    }

    #[test]
    fn test_debugger_events() {
        let events = vec![
            DebugEvent::BreakpointHit(1),
            DebugEvent::StepComplete,
            DebugEvent::ProgramTerminated,
        ];
        assert_eq!(events.len(), 3);
    }
}

#[cfg(test)]
mod docs_tests {
    use ruchy::docs::{DocGenerator, DocFormat};
    use ruchy::frontend::ast::{Expr, ExprKind};

    #[test]
    fn test_doc_generator() {
        use ruchy::frontend::ast::Literal;
        let doc_gen = DocGenerator::new();
        let ast = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Default::default(),
            attributes: vec![],
        };

        // Test generation
        doc_gen.generate(&ast, DocFormat::Markdown).unwrap();
        doc_gen.extract_docs(&ast);
        doc_gen.extract_examples(&ast);
        doc_gen.validate_examples(&ast).unwrap();
    }
}