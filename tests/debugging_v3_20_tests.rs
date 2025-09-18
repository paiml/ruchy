//! TDD Tests for Debugging Support
//! Sprint v3.20.0 - Debugging infrastructure and tools

use ruchy::debugger::{Debugger, Breakpoint, DebugEvent, StackFrame};
use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

#[cfg(test)]
mod debugger_basics {
    use super::*;

    #[test]
    fn test_create_debugger() {
        let debugger = Debugger::new();
        assert!(!debugger.is_running());
        assert_eq!(debugger.breakpoint_count(), 0);
    }

    #[test]
    fn test_set_breakpoint() {
        let mut debugger = Debugger::new();

        let bp = Breakpoint::at_line("test.ruchy", 10);
        debugger.add_breakpoint(bp);

        assert_eq!(debugger.breakpoint_count(), 1);
        assert!(debugger.has_breakpoint_at("test.ruchy", 10));
    }

    #[test]
    fn test_remove_breakpoint() {
        let mut debugger = Debugger::new();

        let bp = Breakpoint::at_line("test.ruchy", 10);
        let id = debugger.add_breakpoint(bp);

        assert_eq!(debugger.breakpoint_count(), 1);

        debugger.remove_breakpoint(id);
        assert_eq!(debugger.breakpoint_count(), 0);
    }

    #[test]
    fn test_conditional_breakpoint() {
        let mut debugger = Debugger::new();

        let bp = Breakpoint::conditional("test.ruchy", 10, "x > 5");
        debugger.add_breakpoint(bp);

        assert!(debugger.has_breakpoint_at("test.ruchy", 10));
    }

    #[test]
    fn test_breakpoint_hit_count() {
        let mut debugger = Debugger::new();

        let bp = Breakpoint::with_hit_count("test.ruchy", 10, 3);
        let id = debugger.add_breakpoint(bp);

        // Should not break on first two hits
        assert!(!debugger.should_break_at("test.ruchy", 10));
        assert!(!debugger.should_break_at("test.ruchy", 10));

        // Should break on third hit
        assert!(debugger.should_break_at("test.ruchy", 10));
    }
}

#[cfg(test)]
mod stepping_control {
    use super::*;

    #[test]
    fn test_step_over() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            let x = 5;
            let y = add(x, 2);
            println(y);
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.set_breakpoint_at_line(3); // Break at 'let y = add(x, 2);'
        debugger.run();

        // Should be paused at breakpoint
        assert!(debugger.is_paused());

        // Step over should execute the function call without entering it
        debugger.step_over();
        assert_eq!(debugger.current_line(), 4); // Should be at 'println(y);'
    }

    #[test]
    fn test_step_into() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        fn main() {
            let x = 5;
            let y = add(x, 2);
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.set_breakpoint_at_line(7); // Break at 'let y = add(x, 2);'
        debugger.run();

        // Step into should enter the function
        debugger.step_into();
        assert_eq!(debugger.current_function(), "add");
        assert_eq!(debugger.current_line(), 2); // Inside add function
    }

    #[test]
    fn test_step_out() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn helper() -> i32 {
            42  // We're here
        }

        fn main() {
            let result = helper();
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.set_breakpoint_at_line(3); // Break inside helper
        debugger.run();

        // Step out should return to caller
        debugger.step_out();
        assert_eq!(debugger.current_function(), "main");
    }

    #[test]
    fn test_continue() {
        let mut debugger = Debugger::new();

        debugger.set_breakpoint_at_line(5);
        debugger.set_breakpoint_at_line(10);

        debugger.run();
        assert_eq!(debugger.current_line(), 5);

        debugger.continue_execution();
        assert_eq!(debugger.current_line(), 10);
    }
}

#[cfg(test)]
mod stack_inspection {
    use super::*;

    #[test]
    fn test_get_call_stack() {
        let _debugger = Debugger::new();
        let code = r#"
        fn deep() { /* breakpoint here */ }
        fn middle() { deep() }
        fn main() { middle() }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        let mut debugger = Debugger::new();
        debugger.load_program(&ast);
        debugger.set_breakpoint_at_function("deep");
        debugger.run();

        let stack = debugger.get_call_stack();
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0].function_name, "deep");
        assert_eq!(stack[1].function_name, "middle");
        assert_eq!(stack[2].function_name, "main");
    }

    #[test]
    fn test_get_local_variables() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            let x = 5;
            let y = "hello";
            let z = true;
            // breakpoint here
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.set_breakpoint_at_line(5);
        debugger.run();

        let locals = debugger.get_local_variables();
        assert_eq!(locals.len(), 3);
        assert!(locals.contains_key("x"));
        assert!(locals.contains_key("y"));
        assert!(locals.contains_key("z"));
    }

    #[test]
    fn test_evaluate_expression() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            let x = 5;
            let y = 10;
            // breakpoint here
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.set_breakpoint_at_line(4);
        debugger.run();

        let result = debugger.evaluate("x + y");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "15");
    }

    #[test]
    fn test_modify_variable() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            let mut x = 5;
            // breakpoint here
            println(x);
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.set_breakpoint_at_line(3);
        debugger.run();

        debugger.set_variable("x", "10");
        debugger.continue_execution();

        // Should print 10 instead of 5
        assert_eq!(debugger.get_output(), "10\n");
    }
}

#[cfg(test)]
mod watch_expressions {
    use super::*;

    #[test]
    fn test_add_watch() {
        let mut debugger = Debugger::new();

        let id = debugger.add_watch("x + y");
        assert_eq!(debugger.watch_count(), 1);

        debugger.remove_watch(id);
        assert_eq!(debugger.watch_count(), 0);
    }

    #[test]
    fn test_evaluate_watches() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            let x = 5;
            let y = 10;
            // breakpoint here
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.add_watch("x");
        debugger.add_watch("y");
        debugger.add_watch("x + y");

        debugger.set_breakpoint_at_line(4);
        debugger.run();

        let watches = debugger.evaluate_watches();
        assert_eq!(watches.len(), 3);
        assert_eq!(watches[0].1, "5");
        assert_eq!(watches[1].1, "10");
        assert_eq!(watches[2].1, "15");
    }

    #[test]
    fn test_watch_change_notification() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            let mut x = 5;
            x = 10;
            x = 15;
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        let watch_id = debugger.add_watch("x");
        debugger.enable_watch_notifications();

        debugger.run();

        let notifications = debugger.get_watch_changes(watch_id);
        assert_eq!(notifications.len(), 2); // Changed from 5->10 and 10->15
    }
}

#[cfg(test)]
mod debug_events {
    use super::*;

    #[test]
    fn test_breakpoint_hit_event() {
        let mut debugger = Debugger::new();

        debugger.set_breakpoint_at_line(5);
        debugger.run();

        let events = debugger.get_events();
        assert!(events.iter().any(|e| matches!(e, DebugEvent::BreakpointHit(_))));
    }

    #[test]
    fn test_step_complete_event() {
        let mut debugger = Debugger::new();

        debugger.set_breakpoint_at_line(5);
        debugger.run();
        debugger.step_over();

        let events = debugger.get_events();
        assert!(events.iter().any(|e| matches!(e, DebugEvent::StepComplete)));
    }

    #[test]
    fn test_program_terminated_event() {
        let mut debugger = Debugger::new();
        let code = "fn main() { }";

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.run();

        let events = debugger.get_events();
        assert!(events.iter().any(|e| matches!(e, DebugEvent::ProgramTerminated)));
    }

    #[test]
    fn test_exception_thrown_event() {
        let mut debugger = Debugger::new();
        let code = r#"
        fn main() {
            panic("Something went wrong")
        }
        "#;

        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();

        debugger.load_program(&ast);
        debugger.run();

        let events = debugger.get_events();
        assert!(events.iter().any(|e| matches!(e, DebugEvent::ExceptionThrown(_))));
    }
}

#[cfg(test)]
mod source_mapping {
    use super::*;

    #[test]
    fn test_line_to_offset() {
        let debugger = Debugger::new();
        let source = "fn main() {\n    let x = 5;\n    println(x);\n}";

        let offset = debugger.line_to_offset(source, 2);
        assert_eq!(source.chars().nth(offset).unwrap(), 'l'); // First 'l' in 'let'
    }

    #[test]
    fn test_offset_to_line() {
        let debugger = Debugger::new();
        let source = "fn main() {\n    let x = 5;\n    println(x);\n}";

        let line = debugger.offset_to_line(source, 20);
        assert_eq!(line, 2);
    }

    #[test]
    fn test_get_source_context() {
        let debugger = Debugger::new();
        let source = "line1\nline2\nline3\nline4\nline5";

        let context = debugger.get_source_context(source, 3, 1);
        assert_eq!(context.len(), 3);
        assert_eq!(context[0], "line2");
        assert_eq!(context[1], "line3"); // Current line
        assert_eq!(context[2], "line4");
    }
}